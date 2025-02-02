#[test]
fn check_compilation_errors() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/compile_time_errors/*.rs");
}
