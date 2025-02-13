use std::fmt;

use ahash::HashSet;
use mcrl2::aterm::ATermRef;
use mcrl2::data::is_data_application;
use mcrl2::data::is_data_function_symbol;
use mcrl2::data::is_data_variable;
use mcrl2::data::DataExpressionRef;
use mcrl2::data::DataFunctionSymbolRef;
use mcrl2::data::DataVariableRef;
use sabre::set_automaton::is_supported_rule;
use sabre::RewriteSpecification;

/// Finds all data symbols in the term and adds them to the symbol index.
fn find_variables(t: &DataExpressionRef<'_>, variables: &mut HashSet<String>) {
    for child in t.iter() {
        if is_data_variable(&child) {
            variables.insert(DataVariableRef::from(child.copy()).name().into());
        }
    }
}

pub struct SimpleTermFormatter<'a> {
    term: ATermRef<'a>,
}

impl SimpleTermFormatter<'_> {
    pub fn new<'b>(term: &'b ATermRef<'b>) -> SimpleTermFormatter<'b> {
        SimpleTermFormatter { term: term.copy() }
    }
}

impl fmt::Display for SimpleTermFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if is_data_function_symbol(&self.term) {
            let symbol = DataFunctionSymbolRef::from(self.term.copy());
            write!(f, "{}_{}", symbol.name(), symbol.operation_id())
        } else if is_data_application(&self.term) {
            let mut args = self.term.arguments();

            let head = args.next().unwrap();
            write!(f, "{}", SimpleTermFormatter::new(&head))?;

            let mut first = true;
            for arg in args {
                if !first {
                    write!(f, ", ")?;
                } else {
                    write!(f, "(")?;
                }

                write!(f, "{}", SimpleTermFormatter::new(&arg))?;
                first = false;
            }

            if !first {
                write!(f, ")")?;
            }

            Ok(())
        } else if is_data_variable(&self.term) {
            write!(f, "{}", DataVariableRef::from(self.term.copy()).name())
        } else {
            write!(f, "{}", self.term)
        }
    }
}

pub struct TrsFormatter<'a> {
    spec: &'a RewriteSpecification,
}

impl TrsFormatter<'_> {
    pub fn new(spec: &RewriteSpecification) -> TrsFormatter {
        TrsFormatter { spec }
    }
}

impl fmt::Display for TrsFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Find all the variables in the specification
        let variables = {
            let mut variables = HashSet::default();

            for rule in &self.spec.rewrite_rules {
                find_variables(&rule.lhs.copy(), &mut variables);
                find_variables(&rule.rhs.copy(), &mut variables);

                for cond in &rule.conditions {
                    find_variables(&cond.lhs.copy(), &mut variables);
                    find_variables(&cond.rhs.copy(), &mut variables);
                }
            }

            variables
        };

        // Print the list of variables.
        writeln!(f, "(VAR ")?;
        for var in variables {
            writeln!(f, "\t {} ", var)?;
        }
        writeln!(f, ") ")?;

        // Print the list of rules.
        writeln!(f, "(RULES ")?;
        for rule in &self.spec.rewrite_rules {
            if is_supported_rule(rule) {
                let mut output = format!(
                    "\t {} -> {}",
                    SimpleTermFormatter::new(&rule.lhs),
                    SimpleTermFormatter::new(&rule.rhs)
                );
                for cond in &rule.conditions {
                    if cond.equality {
                        output += &format!(
                            " COND ==({},{}) -> true",
                            SimpleTermFormatter::new(&cond.lhs),
                            SimpleTermFormatter::new(&cond.rhs)
                        )
                    } else {
                        output += &format!(
                            " COND !=({},{}) -> true",
                            SimpleTermFormatter::new(&cond.lhs),
                            SimpleTermFormatter::new(&cond.rhs)
                        )
                    };
                }

                writeln!(
                    f,
                    "{}",
                    output.replace('|', "bar").replace('=', "eq").replace("COND", "|")
                )?;
            }
        }
        writeln!(f, ")")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mcrl2::data::DataSpecification;

    #[test]
    fn test_convert_trs_format() {
        // Although we do not check the output simply convert a concrete term rewrite system as test.
        let spec = DataSpecification::new(include_str!("../../../examples/REC/mcrl2/benchsym20.dataspec")).unwrap();
        let trs = RewriteSpecification::from(spec);

        println!("{}", TrsFormatter::new(&trs));
    }
}
