#![allow(dead_code)]

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
use crate::interface::tui::{start_tui, TuiCommand, TuiWriter};
use crate::runtime::bytecode::args::Args;
use crate::runtime::callsite::CallSite;
use crate::runtime::classload::loader::ClassLoader;
use crate::runtime::classload::system::SystemClassLoader;
use crate::runtime::config::VmConfig;
use crate::runtime::threading::thread::VmThread;
use crate::runtime::vm::{Vm, VmState};
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

    let mut tui: Option<TuiWriter> = None;

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

    let vm = Vm::new(VmConfig { tui });

    match cmd {
        CliCommand::Run { file } => {
            let res = start(&vm, &file);

            if let Err(err) = res {
                vm.state(VmState::Shutdown);
                error!("runtime returned error `{}`", err);
            }
        }
    }

    // start up an event stream listener after we boot the VM
    // this will listen for terminal inputs and react accordingly
    if let Some(tui) = &vm.tui {
        let mut events = EventStream::new();

        while let Some(event) = events.next().await {
            match event {
                Ok(event) => {
                    match event {
                        Event::Key(k) => {
                            // if we hit ctrl+c, send the quit signal
                            if k.modifiers.contains(KeyModifiers::CONTROL)
                                && k.code == KeyCode::Char('c')
                            {
                                tui.send(TuiCommand::Close)?;
                            }

                            match &k.code {
                                KeyCode::Char('L' | 'l') => tui.send(TuiCommand::Tab(0)),
                                KeyCode::Char('C' | 'c') => tui.send(TuiCommand::Tab(1)),
                                KeyCode::Char('H' | 'h') => tui.send(TuiCommand::Tab(2)),
                                KeyCode::Char('G' | 'g') => tui.send(TuiCommand::Tab(3)),
                                KeyCode::Char('R' | 'r') => tui.send(TuiCommand::Tab(4)),
                                _ => Ok(()),
                            }?;
                        }
                        Event::Mouse(_) => {}
                        Event::Resize(_, _) => {
                            tui.send(TuiCommand::Refresh)?;
                        }
                    }
                }
                Err(why) => return Err(anyhow!(why)),
            }
        }
    }

    Ok(())
}

fn start(vm: &Vm, main_class_path: &str) -> Result<()> {
    let mut loader = vm.system_classloader.write();

    let main_class = loader.find_class(main_class_path)?;
    let main_class = loader.define_class(main_class)?;

    drop(loader);

    let methods_lock = main_class.methods.read();

    let main_method = methods_lock.find(|m| {
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
                    p.type_
                        .as_reference()
                        .filter(|a| a.internal_name == "java/lang/String")
                        .is_some()
                })
                .is_some()
    });

    drop(methods_lock);

    let main_method = main_method.ok_or_else(|| anyhow!("main method not found"))?;

    let mut native_lock = vm.native.write();
    let entries = &mut native_lock.entries;

    entries.insert(
        "java/lang/System.registerNatives:()V".to_string(),
        |_vm, _args, _ctx| Ok(()),
    );

    entries.insert(
        "java/lang/Object.registerNatives:()V".to_string(),
        |_vm, _args, _ctx| Ok(()),
    );

    entries.insert(
        "java/lang/Shutdown.exit:(I)V".to_string(),
        |_vm, args, _ctx| {
            let code = args.entries.pop().unwrap();
            let code = code.as_primitive().unwrap().as_int().unwrap();
            exit(*code);
        },
    );

    drop(native_lock);

    let main_thread = vm
        .threads
        .write()
        .new_thread("main".to_string(), Arc::clone(&main_method));

    vm.interpret(
        CallSite::new(Arc::clone(&main_class), main_thread, main_method, None),
        Args { entries: vec![] }, // TODO: replace this with a string[] of cli args
        false,
    )
}
