use crate::util::{TestResult, state, builder::using_relative, execute, expected, compare, compile_abs};

#[test]
pub fn add_all() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "AddAll");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}
