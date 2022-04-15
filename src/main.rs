extern crate core;

use std::borrow::{BorrowMut};

use std::io::Write;

use std::sync::Arc;



use crate::classfile::parse::ClassFileParser;
use crate::interface::cli::{CLICommand, CLI};
use crate::interface::tui::{start_tui, TuiCommand};
use crate::runtime::classload::loader::ClassLoader;
use crate::runtime::classload::system::SystemClassLoader;
use crate::runtime::context::Context;
use crate::runtime::threading::thread::VMThread;
use crate::runtime::vm::VM;
use crate::structs::bitflag::{MethodAccessFlag};
use crate::structs::descriptor::{DescriptorType};
use crate::structs::loaded::classfile::LoadedClassFile;
use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};




use tokio_stream::StreamExt;
use tracing::{error, Level};
use tracing_subscriber::fmt;



mod classfile;
mod error;
mod interface;
mod runtime;
mod stdlib;
mod structs;

#[tokio::main]
async fn main() -> Result<()> {
    let args = CLI::parse();
    let cmd = args.command;

    let format = fmt::format()
        .with_ansi(true)
        .without_time()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .with_source_location(true)
        .compact();

    let (write, read) = tokio::sync::mpsc::unbounded_channel::<TuiCommand>();
    //TODO: start runtime with these channels to pass messages to tui
    //this will no-op if TUI is not enabled as nothing is listening

    if args.tui {
        start_tui(write.clone(), read)?;
    } else {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .event_format(format)
            .init();
    }

    match cmd {
        CLICommand::Run { file } => {
            let res = start(&file);

            if let Err(err) = res {
                error!("runtime returned error `{}`", err);
            }
        }
    }

    if args.tui {
        let mut events = EventStream::new();

        while let Some(event) = events.next().await {
            if let Ok(event) = event {
                match event {
                    Event::Key(k) => {
                        if k.modifiers.contains(KeyModifiers::CONTROL)
                            && k.code == KeyCode::Char('c')
                        {
                            write.clone().borrow_mut().send(TuiCommand::Close).unwrap();
                        }
                    }
                    Event::Mouse(_) => {}
                    Event::Resize(_, _) => {
                        write
                            .clone()
                            .borrow_mut()
                            .send(TuiCommand::Refresh)
                            .unwrap();
                    }
                }
            } else {
                return Err(anyhow!(event.unwrap_err()));
            }
        }
    }

    Ok(())
}

fn start(class_path: &str) -> Result<()> {
    let mut vm = VM::new();

    let mut loader = vm.system_classloader.write();
    let main_class = loader.find_class(class_path)?;
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

    vm.interpret(
        &main_method,
        Context {
            class: Arc::clone(&main_class),
            thread: VMThread::new("main".to_string()),
        },
    )?;

    Ok(())
}
