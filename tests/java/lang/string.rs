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
#[ignore = "broken, throws assertion errors in j.l.CharacterData00.<clinit>. the string length is wrong. see also comments in class"]
pub fn is_blank() -> TestResult {
    let state = state().init().init_std();

    let path = using_relative(file!(), "IsBlank");

    let got = execute(state, compile_abs(path)?)?;
    let expected = expected().has_success();

    compare(got, expected);

    Ok(())
}