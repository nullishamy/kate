use std::{
    fs::File,
    io::{Error, Write},
    path::PathBuf,
    process::Command,
};

pub mod builder;

const TMP_DIR: &str = env!("CARGO_TARGET_TMPDIR");
const SOURCE_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub type TestResult = Result<(), Error>;

pub fn compile_abs(path: impl Into<PathBuf>) -> Result<String, Error> {
    let tmp_dir: PathBuf = TMP_DIR.into();
    let path = path.into();

    // javac takes args with this form:
    // javac SOURCEFILE.java -d OUTPUT_DIR
    let compilation = Command::new("javac")
        .arg(format!("{}.java", path.display()))
        .arg("-d")
        .arg(&tmp_dir)
        .output()?;

    if !compilation.status.success() {
        let stderr = String::from_utf8(compilation.stderr).unwrap();
        panic!("could not compile {}:\n{}", path.display(), stderr);
    }

    // Compiling an absolute target means we need to drop the leading information
    // So we should give the resolver just the class name and let it find the class
    Ok(path.file_name().unwrap().to_string_lossy().to_string())
}

pub fn compile(path: impl Into<PathBuf>) -> Result<String, Error> {
    let tmp_dir: PathBuf = TMP_DIR.into();
    let path = path.into();

    // javac takes args with this form:
    // javac SOURCEFILE.java -d OUTPUT_DIR
    let compilation = Command::new("javac")
        .arg(tmp_dir.join(format!("{}.java", path.display())))
        .arg("-d")
        .arg(&tmp_dir)
        .output()?;

    if !compilation.status.success() {
        let stderr = String::from_utf8(compilation.stderr).unwrap();
        panic!("could not compile {}:\n{}", path.display(), stderr);
    }

    // But we return just the name from here so that the classfile resolver
    // can work with it to find the file as usual
    Ok(path.to_string_lossy().to_string())
}

pub fn inline(class: builder::Class) -> Result<String, Error> {
    let tmp_dir: PathBuf = TMP_DIR.into();
    let class_path = tmp_dir.join(format!("{}.java", class.name));
    let mut class_file = File::create(class_path)?;

    class_file.write_all(class.content.as_bytes())?;

    compile(class.name)
}

#[derive(Debug)]
pub struct Execution {
    out: Vec<String>,
    err: Vec<String>,

    code: i32,
}

impl PartialEq for Execution {
    // We do not compare stderr for executions.
    // It only contains debug logs. stdout will contain the important stuff
    fn eq(&self, other: &Self) -> bool {
        self.out == other.out && self.code == other.code
    }
}

impl Execution {
    pub fn with_output(mut self, line: impl Into<String>) -> Self {
        self.out.push(line.into());
        self
    }

    pub fn with_error(mut self, line: impl Into<String>) -> Self {
        self.err.push(line.into());
        self
    }

    pub fn with_exit(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    pub fn has_success(mut self) -> Self {
        self.code = 0;
        self
    }

    pub fn has_error(mut self) -> Self {
        self.code = 1;
        self
    }
}

pub fn expected() -> Execution {
    Execution {
        out: vec![],
        err: vec![],
        code: 0,
    }
}

pub fn state() -> State {
    State { init_std: false }
}

#[derive(Debug)]
pub struct State {
    init_std: bool,
}

impl State {
    pub fn init(self) -> Self {
        self
    }

    pub fn init_std(mut self) -> Self {
        self.init_std = true;
        self
    }
}

pub fn execute(state: State, class_name: String) -> Result<Execution, Error> {
    let mut command = Command::new("cargo");
    let exec = command
        .arg("run")
        .arg("--manifest-path")
        .arg(format!("{SOURCE_DIR}/../Cargo.toml"))
        .arg("--")
        .arg("--cp")
        .arg(TMP_DIR)
        .arg("-Dtest.init=true")
        .arg(class_name);

    if state.init_std {
        exec.arg("-Dtest.boot=true");
    }

    let output = exec.output()?;

    let (out, err) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let (out, err) = (
        out.lines().map(|l| l.to_string()).collect(),
        err.lines().map(|l| l.to_string()).collect(),
    );

    Ok(Execution {
        out,
        err,

        code: output.status.code().unwrap_or(0),
    })
}

const ERR_LIMIT: usize = 25;
pub fn compare(got: Execution, expected: Execution) {
    // Execution of the JVM failed, log the error.
    // This includes assertion failures
    if got.code != expected.code {
        let err = if got.err.len() > 25 {
            got.err.split_at(got.err.len() - ERR_LIMIT).1
        } else {
            &got.err
        };

        eprintln!("Execution failed:");
        eprintln!("Stdout:\n{}", got.out.join("\n"));
        eprintln!("\n\n");
        eprintln!("Stderr:\n{}", err.join("\n"));
        panic!("...");
    }

    // Regular comparison.
    // Realistically this shouldn't be tripped often as the assertions
    // should catch everything in Java, but for sanity we should check it
    assert_eq!(got.out, expected.out);
    assert_eq!(got.code, expected.code);
}
