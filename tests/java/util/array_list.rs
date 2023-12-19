use crate::util::{
    builder::using_relative, compare, compile_abs, execute, expected, state, TestResult,
};

#[test]
pub fn add_all() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "AddAll");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}
