use std::borrow::Borrow;
use std::env;

use crate::classfile::parse::ClassFileParser;
use crate::runtime::classload::system::SystemClassLoader;
use crate::structs::loaded::classfile::LoadedClassFile;
use anyhow::{anyhow, Result};

mod classfile;
mod runtime;
mod structs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => quit("No class file passed"),
        2 => start(args[1].borrow()),
        _ => quit("Too many args passed"),
    }
}

fn start(class_path: &str) -> Result<()> {
    let mut parser = ClassFileParser::from_path(class_path.to_string())?;
    let class_file = parser.parse()?;
    let class_file = LoadedClassFile::from_raw(class_file)?;
    Ok(())
}

fn quit(msg: &str) -> ! {
    eprintln!("[ERROR] {}", msg);
    std::process::exit(1);
}
