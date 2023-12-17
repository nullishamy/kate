use crate::util::{
    builder::using_relative, compare, compile_abs, execute, expected, state, TestResult,
};

#[test]
pub fn to_string() -> TestResult {
    let state = state().init().init_std();
    let path = using_relative(file!(), "ACToString");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected()
        .has_success()
        .with_output("class ACToString$1")
        .with_output("class java/util/ArrayList")
        .with_output("class java/util/Vector")
        .with_output("class java/util/concurrent/CopyOnWriteArrayList")
        .with_output("class java/util/concurrent/CopyOnWriteArraySet")
        .with_output("Passed = 43, failed = 0");

    compare(got, expected);

    Ok(())
}
