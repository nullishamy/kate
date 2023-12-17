use crate::util::{
    builder::using_relative, compare, compile_abs, execute, expected, state, TestResult,
};

#[test]
pub fn equals() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "AMEquals");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}

#[test]
pub fn to_string() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "AMToString");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}

#[test]
pub fn simple_entries() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "AMSimpleEntries");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected()
        .has_success()
        .with_output("Passed = 46, failed = 0");

    compare(got, expected);

    Ok(())
}
