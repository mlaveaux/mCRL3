use test_case::test_case;

use stark::UntypedStarkSpecification;

#[test_case(include_str!("../examples/Engine.stark") ; "Engine")]
fn test_parse_stark_specification(spec: &str) {
    if let Err(y) = UntypedStarkSpecification::parse(spec) {
        panic!("{}", y);
    }
}
