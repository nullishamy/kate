extern crate core;

use std::process::exit;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use tokio_stream::StreamExt;
use tracing::{error, Level};
use tracing_subscriber::fmt;

use crate::classfile::parse::ClassFileParser;
use crate::interface::cli::{Cli, CliCommand};
use crate::interface::tui::{start_tui, TUIWriter, TuiCommand};
use crate::runtime::bytecode::args::Args;
use crate::runtime::callsite::CallSite;
use crate::runtime::classload::loader::ClassLoader;
use crate::runtime::classload::system::SystemClassLoader;
use crate::runtime::config::VMConfig;
use crate::runtime::threading::thread::VMThread;
use crate::runtime::vm::{VMState, VM};
use crate::structs::bitflag::MethodAccessFlag;
use crate::structs::descriptor::DescriptorType;
use crate::structs::loaded::classfile::LoadedClassFile;

mod classfile;
mod error;
mod interface;
mod runtime;
mod stdlib;
mod structs;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let cmd = args.command;

    let (write, read) = tokio::sync::mpsc::unbounded_channel::<TuiCommand>();
    //TODO: start runtime with these channels to pass messages to tui
    //this will no-op if TUI is not enabled as nothing is listening

    let mut tui: Option<TUIWriter> = None;

    if args.tui {
        tui = Some(start_tui(write.clone(), read)?);
    } else {
        let format = fmt::format()
            .with_ansi(true)
            .without_time()
            .with_level(true)
            .with_target(false)
            .with_thread_names(false)
            .with_source_location(true)
            .compact();

        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .event_format(format)
            .init();
    }

    let vm = VM::new(VMConfig { tui });

    match cmd {
        CliCommand::Run { file } => {
            let res = start(&vm, &file);

            if let Err(err) = res {
                vm.state(VMState::Shutdown);
                error!("runtime returned error `{}`", err);
            }
        }
    }

    // start up an event stream listener after we boot the VM
    // this will listen for terminal inputs and react accordingly
    if let Some(tui) = &vm.tui {
        let mut events = EventStream::new();

        while let Some(event) = events.next().await {
            if let Ok(event) = event {
                match event {
                    Event::Key(k) => {
                        // if we hit ctrl+c, send the quit signal
                        if k.modifiers.contains(KeyModifiers::CONTROL)
                            && k.code == KeyCode::Char('c')
                        {
                            tui.send(TuiCommand::Close)?;
                        }

                        match &k.code {
                            KeyCode::Char('L') | KeyCode::Char('l') => tui.send(TuiCommand::Tab(0)),
                            KeyCode::Char('C') | KeyCode::Char('c') => tui.send(TuiCommand::Tab(1)),
                            KeyCode::Char('H') | KeyCode::Char('h') => tui.send(TuiCommand::Tab(2)),
                            KeyCode::Char('G') | KeyCode::Char('g') => tui.send(TuiCommand::Tab(3)),
                            KeyCode::Char('R') | KeyCode::Char('r') => tui.send(TuiCommand::Tab(4)),
                            _ => Ok(()),
                        }?;
                    }
                    Event::Mouse(_) => {}
                    Event::Resize(_, _) => {
                        tui.send(TuiCommand::Refresh)?;
                    }
                }
            } else {
                return Err(anyhow!(event.unwrap_err()));
            }
        }
    }

    Ok(())
}

fn start(vm: &VM, main_class_path: &str) -> Result<()> {
    let mut loader = vm.system_classloader.write();

    let main_class = loader.find_class(main_class_path)?;
    let main_class = loader.define_class(main_class)?;

    let lock = main_class.methods.read();

    let main_method = lock.find(|m| {
        m.name.str == "main"
            && m.access_flags.has(MethodAccessFlag::STATIC)
            && m.descriptor.return_type == DescriptorType::Void
            && m.descriptor.parameters.len() == 1
            && m.descriptor
                .parameters
                .get(0)
                .unwrap()
                .as_array()
                .filter(|p| {
                    p._type
                        .as_reference()
                        .filter(|a| a.internal_name == "java/lang/String")
                        .is_some()
                })
                .is_some()
    });

    if main_method.is_none() {
        return Err(anyhow!("main method not found"));
    }

    let main_method = main_method.unwrap();
    drop(loader); // we need to explicitly drop the loader before borrowing 'vm' as mut
                  // otherwise we would have 'loader' holding imut borrow whilst mut which isnt ok

    vm.native.write().entries.insert(
        "java/lang/System.registerNatives:()V".to_string(),
        |_vm, _args, _ctx| Ok(()),
    );

    vm.native.write().entries.insert(
        "java/lang/Object.registerNatives:()V".to_string(),
        |_vm, _args, _ctx| Ok(()),
    );

    vm.native.write().entries.insert(
        "java/lang/Shutdown.exit:(I)V".to_string(),
        |_vm, args, _ctx| {
            let code = args.entries.pop().unwrap();
            let code = code.as_primitive().unwrap().as_int().unwrap();
            exit(*code);
        },
    );

    let main_thread = vm.threads.write().new_thread("main".to_string(), Arc::clone(&main_method));

    vm.interpret(
        CallSite::new(Arc::clone(&main_class), main_thread, main_method, None),
        Args { entries: vec![] }, // TODO: replace this with a string[] of cli args
        false,
    )
}
