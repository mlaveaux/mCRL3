#[test]
fn test_derive_term() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/input/aterm_lifetime.rs");
}