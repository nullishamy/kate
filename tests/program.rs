mod util;

use std::{fs::File, io::Write, path::PathBuf};

use util::{
    builder::{direct, using_helpers},
    TMP_DIR,
};

use crate::util::{builder::using_main, compare, execute, expected, inline, state, TestResult};

#[test]
pub fn hello_world() -> TestResult {
    let state = state().init().init_std();

    let source = using_main(
        "HelloWorld",
        r#"
        System.out.print("Hello, World!\n");
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success().with_output("Hello, World!");

    compare(got, expected);

    Ok(())
}

// Previously, we had to use `print` because the newline prop hadn't been set.
// This should be operational now, so test it separately.
#[test]
pub fn hello_world_println() -> TestResult {
    let state = state().init().init_std();

    let source = using_main(
        "HelloWorldLN",
        r#"
        System.out.println("Hello, World!");
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success().with_output("Hello, World!");

    compare(got, expected);

    Ok(())
}

#[test]
pub fn exit_success() -> TestResult {
    let state = state().init();

    let source = using_main(
        "ExitSuccess",
        r#"
        System.exit(0);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}

#[test]
pub fn exit_error() -> TestResult {
    let state = state().init();

    let source = using_main(
        "ExitError",
        r#"
        System.exit(1);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().with_exit(1);

    compare(got, expected);

    Ok(())
}

#[test]
pub fn exit_arbitrary() -> TestResult {
    let state = state().init();

    let source = using_main(
        "ExitArbitrary",
        r#"
        int arbitrary_exit = 249;
        System.exit(arbitrary_exit);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().with_exit(249);

    compare(got, expected);

    Ok(())
}

#[test]
pub fn internal_error() -> TestResult {
    let state = state().init().opt("test.throwinternal", "true");

    let source = using_main(
        "InternalError",
        r#"
        assertNotReached();
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected()
        .with_exit(1)
        .with_output("/----------------------------------------------------------\\")
        .with_output("|The VM encountered an unrecoverable error and had to abort.|")
        .with_output("\\----------------------------------------------------------/")
        .with_output("Uncaught exception in main: testing, internal errors")
        .with_output("  at InternalError.main");

    compare(got, expected);

    Ok(())
}

#[test]
pub fn read_file() -> TestResult {
    let state = state().init().init_std();

    let tmp_dir: PathBuf = TMP_DIR.into();
    let tmp_path = tmp_dir.join("basic-read.txt");
    let mut tmp_file = File::create(tmp_path.clone())?;
    tmp_file.write_all("test test test".as_bytes())?;

    let content = format!(
        r#"
        import java.io.File;
        import java.io.FileInputStream;

        class ReadFile {{
            public static void main(String[] args) throws Exception {{
                File file = new File("{}");
                FileInputStream inputStream = new FileInputStream(file);
                byte[] bytes = inputStream.readAllBytes();
                System.out.print("Content: ");
                System.out.println(bytes);
            }}
        }}
        "#,
        tmp_path.display()
    );

    let source = direct("ReadFile", &content);

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}
#[test]
pub fn new_thread_set_priority() -> TestResult {
    let state = state().init();

    let source = using_main(
        "NewThreadSetPriority",
        r#"
            Thread t = new Thread();
            t.setPriority(1);
        "#,
    );

    let got = execute(state, inline(source)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}