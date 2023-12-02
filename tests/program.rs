mod util;

use crate::util::{
    builder::{direct, using_main},
    compare, execute, expected, inline, state, TestResult,
};

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
