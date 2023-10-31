use crate::util::{TestResult, state, builder::{using_main, using_relative}, execute, inline, expected, compare, compile, compile_abs};

#[test]
pub fn is_empty() -> TestResult {
    let state = state().init();

    let path = using_relative(file!(), "IsEmpty");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}

#[test]
#[ignore = "broken"]
pub fn is_blank() -> TestResult {
    let state = state().init();

    let path = using_relative(file!(), "IsBlank");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}