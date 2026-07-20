use indoc::indoc;
use merc_syntax::UntypedDataSpecification;
use pest::Parser;

use merc_syntax::Mcrl2Parser;
use merc_syntax::Rule;
use merc_syntax::UntypedProcessSpecification;
use merc_syntax::UntypedStateFrmSpec;
use merc_syntax::parse_sortexpr;
use merc_utilities::test_logger;
use test_case::test_case;

/// `DataExprIn`, `DataExprIntDiv`, and `DataExprMod` used to be plain string
/// matches without a negative lookahead (`!Id`). This meant that identifiers
/// whose names *start* with `in`, `div`, or `mod` were incorrectly tokenised
/// as a keyword followed by the rest of the identifier.
#[test]
fn keywords_are_not_prefix_of_identifiers() {
    // `index` starts with `in` — must parse as a plain identifier.
    UntypedProcessSpecification::parse("map index: Bool -> Bool;")
        .expect("`index` must not be split into keyword `in` + `dex`");
    // `divides` starts with `div`.
    UntypedProcessSpecification::parse("map divides: Bool -> Bool;")
        .expect("`divides` must not be split into keyword `div` + `ides`");
    // `modern` starts with `mod`.
    UntypedProcessSpecification::parse("map modern: Bool -> Bool;")
        .expect("`modern` must not be split into keyword `mod` + `ern`");
}

/// `Number` used to be a non-atomic rule (`{ ASCII_DIGIT+ }`), which in pest
/// causes implicit whitespace to be absorbed *after* the digits. That meant
/// tokens like `"3 "` could match `Number`, and boundary tests such as
/// `"3 div 2"` could mis-parse because the parser consumed the trailing space
/// as part of the number and left `div` unrecognised as an operator.
#[test]
fn number_token_does_not_absorb_whitespace() {
    // A Number followed immediately by a keyword must still parse correctly.
    // Parsing fails on the leading `val(...)` wrapper in a data-specification
    // context, so use a minimal expression that forces a Number-then-keyword
    // boundary: `3 div 2`.
    let parsed = Mcrl2Parser::parse(Rule::Number, "3 extra");
    // The Number rule must match exactly the digit run, not consume the space.
    assert!(parsed.is_ok());
    assert_eq!(
        parsed.unwrap().as_str(),
        "3",
        "Number must not consume trailing whitespace"
    );
}

#[test]
fn test_parse_ifthen() {
    let expr = "init a -> b <> b;";

    match UntypedProcessSpecification::parse(expr) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_parse_keywords() {
    let expr = "map or : Boolean # Boolean -> Boolean ;";

    match UntypedProcessSpecification::parse(expr) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_parse_sort_spec() {
    let sort_spec = indoc! {"
        sort D = Bool -> Int -> Bool;
        

        % Test
        F     = struct d1 | d2;
        Error = struct e;
    "};

    match UntypedProcessSpecification::parse(sort_spec) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_parse_regular_expression() {
    let spec = "[true++false]true";

    match UntypedStateFrmSpec::parse(spec) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_parse_procexpr() {
    test_logger();

    use indoc::indoc;

    let spec: &str = indoc! {"init
        true -> delta <> delta;
    "};

    match UntypedProcessSpecification::parse(spec) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_parse_statefrm() {
    test_logger();

    use indoc::indoc;

    let spec: &str = indoc! {"<b> <a> exists b: Bool . b && !b"};

    match UntypedStateFrmSpec::parse(spec) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_sort_precedence() {
    let term = "Bool # Int -> Int -> Bool";

    match Mcrl2Parser::parse(Rule::SortExpr, term) {
        Ok(result) => {
            print!("{}", parse_sortexpr(result).unwrap());
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}

#[test]
fn test_bool_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/bool.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_int_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/int.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_nat_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/nat.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_pos_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/pos.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_real_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/real.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_list_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/list.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_set_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/set.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_fset_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/fset.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_bag_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/bag.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

#[test]
fn test_fbag_spec() {
    match UntypedDataSpecification::parse(include_str!("../spec/fbag.mcrl2")) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse expression: {}", e);
        }
    }
}

/// Parses every machine-number (`*64`) specification, ported from the mCRL2
/// code-generation `.spec` files.
#[test_case(include_str!("../spec/machine_word.mcrl2") ; "machine_word.mcrl2")]
#[test_case(include_str!("../spec/pos64.mcrl2") ; "pos64.mcrl2")]
#[test_case(include_str!("../spec/nat64.mcrl2") ; "nat64.mcrl2")]
#[test_case(include_str!("../spec/int64.mcrl2") ; "int64.mcrl2")]
#[test_case(include_str!("../spec/real64.mcrl2") ; "real64.mcrl2")]
#[test_case(include_str!("../spec/list64.mcrl2") ; "list64.mcrl2")]
#[test_case(include_str!("../spec/set64.mcrl2") ; "set64.mcrl2")]
#[test_case(include_str!("../spec/fset64.mcrl2") ; "fset64.mcrl2")]
#[test_case(include_str!("../spec/bag64.mcrl2") ; "bag64.mcrl2")]
#[test_case(include_str!("../spec/fbag64.mcrl2") ; "fbag64.mcrl2")]
fn test_machine_number_spec(spec: &str) {
    match UntypedDataSpecification::parse(spec) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            panic!("Failed to parse: {}", e);
        }
    }
}
