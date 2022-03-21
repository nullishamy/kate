mod classfile;
mod machine;
mod types;

use std::borrow::Borrow;
use std::env;

use crate::classfile::{invalid_format, ClassFile};
use crate::machine::Bootstrapper;
use crate::types::classfile_type::ByteCode;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => quit("No class file passed"),
        2 => start(args[1].borrow()),
        _ => quit("Too many args passed"),
    }
}

fn start(class_path: &str) {
    let mut f = File::open(class_path).unwrap_or_else(|e| {
        quit(format!("Failed to open file {} because of error {}", class_path, e).borrow())
    });

    let mut buffer = Vec::new();

    // read the whole file
    f.read_to_end(&mut buffer).unwrap_or_else(|e| {
        quit(
            format!(
                "Failed to open file '{}' because of error {}",
                class_path, e
            )
            .borrow(),
        )
    });

    let class_file = ClassFile::new(&mut ByteCode::new(buffer, class_path), class_path);

    let valid = class_file.validate();

    if let Err(msg) = valid {
        invalid_format(class_path, format!("validation failed {}", msg).borrow())
    }
}

fn quit(msg: &str) -> ! {
    eprintln!("[ERROR] {}", msg);
    std::process::exit(1);
}
