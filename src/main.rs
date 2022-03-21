mod parse;
mod runtime;
mod types;

use std::borrow::Borrow;
use std::env;

use crate::parse::bytecode::ByteCode;
use crate::parse::classfile::RawClassFile;

use crate::types::classfile::{AttributeInfoEntry, ConstantPoolData};
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => quit("No class file passed"),
        2 => start(args[1].borrow()),
        _ => quit("Too many args passed"),
    }
}

fn start(class_path: &str) -> Result<()> {
    let f = File::open(class_path);

    if let Err(e) = f {
        return Err(anyhow!(
            "failed to open file {} because of error {}",
            class_path,
            e
        ));
    }

    let mut buffer = Vec::new();

    // read the whole file
    if let Err(e) = f.unwrap().read_to_end(&mut buffer) {
        return Err(anyhow!(
            "failed to open file '{}' because of error {}",
            class_path,
            e
        ));
    }

    let mut class_file = RawClassFile::new(&mut ByteCode::new(buffer, class_path), class_path)?;

    let prepared_class_file = class_file.prepare();

    if let Err(msg) = prepared_class_file {
        return Err(anyhow!("preparation failed ({})", msg));
    }

    let prepared_class_file = prepared_class_file.unwrap();
    let valid = prepared_class_file.validate();

    if let Err(msg) = valid {
        return Err(anyhow!("validation failed ({})", msg));
    }

    let valid = valid.unwrap();

    Ok(())
}

fn quit(msg: &str) -> ! {
    eprintln!("[ERROR] {}", msg);
    std::process::exit(1);
}
