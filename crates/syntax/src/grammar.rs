use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "mcrl2_grammar.pest"]
pub struct Mcrl2Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UntypedProcessSpecification;

    use mcrl3_utilities::test_logger;

    use indoc::indoc;
    use pest::Parser;



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

        let result = UntypedProcessSpecification::parse(expr).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_sort_spec() {
        let sort_spec = indoc! {"
            sort D = Bool -> Int -> Bool;
            

            % Test
            F     = struct d1 | d2;
            Error = struct e;
        "};

        let result = UntypedProcessSpecification::parse(sort_spec).unwrap();
        print!("{}", result);
    }

    #[test]
    fn test_parse_regular_expression() {
        let spec = "[true++false]true";

        if let Err(y) = Mcrl2Parser::parse(Rule::StateFrmSpec, spec) {
            panic!("{}", y);
        }
    }

    #[test]
    fn test_parse_procexpr() {
        let _ = test_logger();

        use indoc::indoc;

        let spec: &str = indoc! {"init
            true -> delta <> delta;
        "};

        println!("{}", UntypedProcessSpecification::parse(spec).unwrap());
    }
}
