#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
    t.compile_fail("tests/compile_fail/*.rs");
}
