use crate::util::{TestResult, state, builder::using_relative, execute, expected, compare, compile_abs};

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
#[ignore = "broken, we don't support nested arrays yet"]
pub fn is_blank() -> TestResult {
    let state = state().init();

    let path = using_relative(file!(), "IsBlank");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}