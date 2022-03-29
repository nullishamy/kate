use std::borrow::Borrow;
use std::env;
use std::rc::Rc;

use crate::classfile::parse::ClassFileParser;
use crate::runtime::classload::loader::{ClassLoader, MutableClassLoader};
use crate::runtime::classload::system::SystemClassLoader;
use crate::runtime::context::Context;
use crate::runtime::threading::thread::VMThread;
use crate::runtime::vm::VM;
use crate::structs::bitflag::MethodAccessFlag;
use crate::structs::descriptor::{DescriptorType, ReferenceType};
use crate::structs::loaded::classfile::LoadedClassFile;
use anyhow::{anyhow, Result};

mod classfile;
mod runtime;
mod structs;
mod error;
mod interface;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => quit("No class file passed"),
        2 => start(args[1].borrow()),
        _ => quit("Too many args passed"),
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
fn quit(msg: &str) -> ! {
    eprintln!("[ERROR] {}", msg);
    std::process::exit(1);
}
