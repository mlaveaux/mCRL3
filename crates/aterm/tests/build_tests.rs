#[test]
fn test_soundness() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/input/aterm_lifetime.rs");
    // TODO: Fix this test.
    //t.compile_fail("tests/input/aterm_container_lifetime.rs");
}