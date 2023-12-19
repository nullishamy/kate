use std::{env::args, path::PathBuf, process::Command};

pub const SOURCE_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let include_dir = PathBuf::from(SOURCE_DIR).join("include");
    let mut args = args();
    args.next();

    let tmp_dir = args.next().expect("no tmp dir specified");

    let compilation = Command::new("javac")
        .args(["-cp", &include_dir.display().to_string()])
        .arg("-d")
        .arg(&tmp_dir)
        .arg(include_dir.join("kate/Util.java"))
        .output()
        .unwrap();

    if !compilation.status.success() {
        let stderr = String::from_utf8(compilation.stderr).unwrap();
        panic!("could not compile includes:\n{}", stderr);
    }

    println!("Compiled includes");
}
