#[test]
#[cfg_attr(miri, ignore)]
fn test_soundness() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/input/aterm_lifetime.rs");
    t.compile_fail("tests/input/aterm_container_lifetime.rs");
}
