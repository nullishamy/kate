use std::borrow::Borrow;
use std::env;
use std::rc::Rc;

use crate::classfile::parse::ClassFileParser;
use crate::interface::cli::{CLICommand, CLI};
use crate::interface::tui::TUI;
use crate::runtime::classload::loader::{ClassLoader, MutableClassLoader};
use crate::runtime::classload::system::SystemClassLoader;
use crate::runtime::context::Context;
use crate::runtime::threading::thread::VMThread;
use crate::runtime::vm::VM;
use crate::structs::bitflag::MethodAccessFlag;
use crate::structs::descriptor::{DescriptorType, ReferenceType};
use crate::structs::loaded::classfile::LoadedClassFile;
use anyhow::{anyhow, Result};
use clap::Parser;
use tracing::{error, info, Level};
use tracing_subscriber::fmt;

mod classfile;
mod error;
mod interface;
mod runtime;
mod structs;

fn main() {
    let args = CLI::parse();
    let cmd = args.command;

    let format = fmt::format()
        .with_ansi(true)
        .with_level(true)
        .with_target(false)
        .with_thread_names(true)
        .compact();

    if args.tui {
        let tui = TUI::new();

        if let Err(err) = tui {
            error!("tui returned error `{}`", err)
        }

        info!("tui selected and loaded");
    }

    tracing_subscriber::fmt().event_format(format).init();

    match cmd {
        CLICommand::Run { file } => {
            let res = start(&file);

            if let Err(err) = res {
                error!("runtime returned error `{}`", err)
            }
        }
    }
}

fn start(class_path: &str) -> Result<()> {
    let mut vm = VM::new();

    let loader = vm.system_classloader();
    let main_class = loader.find_class(class_path)?;
    let (loader, main_class) = loader.define_class(&main_class)?;

    let main_method = main_class.methods.find(|m| {
        m.name.as_str == "main"
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

    vm.interpret(
        main_method,
        &mut Context {
            class: Rc::clone(&main_class),
            thread: VMThread::new(),
        },
    )?;

    Ok(())
}
