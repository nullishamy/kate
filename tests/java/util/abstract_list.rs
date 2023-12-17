use crate::util::{TestResult, state, builder::using_relative, execute, expected, compare, compile_abs};

#[test]
pub fn check_for_comodification() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "ALCheckForComodification");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}


#[test]
pub fn has_next() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "ALHasNextAfterException");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}