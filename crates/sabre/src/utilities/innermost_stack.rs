use std::fmt;

use itertools::Itertools;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::Markable;
use mcrl3_aterm::Protected;
use mcrl3_aterm::Protector;
use mcrl3_aterm::ThreadTermPool;
use mcrl3_aterm::Marker;
use mcrl3_data::is_data_expression;
use mcrl3_data::is_data_machine_number;
use mcrl3_data::is_data_variable;
use mcrl3_data::DataApplication;
use mcrl3_data::DataExpression;
use mcrl3_data::DataExpressionRef;
use mcrl3_data::DataFunctionSymbolRef;

use crate::utilities::PositionIndexed;
use crate::Rule;

use super::create_var_map;
use super::ExplicitPosition;
use super::PositionIterator;

use log::trace;

/// This stack is used to avoid recursion and also to keep track of terms in
/// normal forms by explicitly representing the rewrites of a right hand
/// side.
#[derive(Default)]
pub struct InnermostStack {
    pub configs: Protected<Vec<Config>>,
    pub terms: Protected<Vec<DataExpressionRef<'static>>>,
}

impl InnermostStack {
    /// Updates the InnermostStack to integrate the rhs_stack instructions.
    pub fn integrate(
        write_configs: &mut Protector<Vec<Config>>,
        write_terms: &mut Protector<Vec<DataExpressionRef<'static>>>,
        rhs_stack: &RHSStack,
        term: &DataExpression,
        result_index: usize,
    ) {
        // TODO: This ignores the first element of the stack, but that is kind of difficult to deal with.
        let top_of_stack = write_terms.len();
        write_terms.reserve(rhs_stack.stack_size - 1); // We already reserved space for the result.
        for _ in 0..rhs_stack.stack_size - 1 {
            write_terms.push(Default::default());
        }

        let mut first = true;
        for config in rhs_stack.innermost_stack.read().iter() {
            match config {
                Config::Construct(symbol, arity, offset) => {
                    if first {
                        // The first result must be placed on the original result index.
                        InnermostStack::add_result(write_configs, symbol.copy(), *arity, result_index);
                    } else {
                        // Otherwise, we put it on the end of the stack.
                        InnermostStack::add_result(write_configs, symbol.copy(), *arity, top_of_stack + offset - 1);
                    }
                }
                Config::Rewrite(_) => {
                    unreachable!("This case should not happen");
                }
                Config::Return() => {
                    unreachable!("This case should not happen");
                }
            }
            first = false;
        }
        trace!(
            "\t applied stack size: {}, substitution: {}, stack: [{}]",
            rhs_stack.stack_size,
            rhs_stack.variables.iter().format_with(", ", |element, f| {
                f(&format_args!("{} -> {}", element.0, element.1))
            }),
            rhs_stack.innermost_stack.read().iter().format("\n")
        );

        debug_assert!(
            rhs_stack.stack_size != 1 || rhs_stack.variables.len() <= 1,
            "There can only be a single variable in the right hand side"
        );
        if rhs_stack.stack_size == 1 && rhs_stack.variables.len() == 1 {
            // This is a special case where we place the result on the correct position immediately.
            // The right hand side is only a variable
            let t: ATermRef<'_> = write_terms.protect(&term.get_position(&rhs_stack.variables[0].0));
            write_terms[result_index] = t.into();
        } else {
            for (position, index) in &rhs_stack.variables {
                // Add the positions to the stack.
                let t = write_terms.protect(&term.get_position(position));
                write_terms[top_of_stack + index - 1] = t.into();
            }
        }
    }

    /// Indicate that the given symbol with arity can be constructed at the given index.
    pub fn add_result(
        write_configs: &mut Protector<Vec<Config>>,
        symbol: DataFunctionSymbolRef<'_>,
        arity: usize,
        index: usize,
    ) {
        let symbol = write_configs.protect(&symbol.into());
        write_configs.push(Config::Construct(symbol.into(), arity, index));
    }

    /// Indicate that the term must be rewritten and its result must be placed at the given index.
    pub fn add_rewrite(
        write_configs: &mut Protector<Vec<Config>>,
        write_terms: &mut Protector<Vec<DataExpressionRef<'static>>>,
        term: DataExpressionRef<'_>,
        index: usize,
    ) {
        let term = write_terms.protect(&term);
        write_configs.push(Config::Rewrite(index));
        write_terms.push(term.into());
    }
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Config {
    /// Rewrite the top of the stack and put result at the given index.
    Rewrite(usize),
    /// Constructs function symbol with given arity at the given index.
    Construct(DataFunctionSymbolRef<'static>, usize, usize),
    /// Yields the given index as returned term.
    Return(),
}

impl Markable for Config {
    fn mark(&self, marker: &mut Marker) {
        if let Config::Construct(t, _, _) = self {
            let t: ATermRef<'_> = t.copy().into();
            t.mark(marker);
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        if let Config::Construct(t, _, _) = self {
            term == &<DataFunctionSymbolRef as Into<ATermRef>>::into(t.copy())
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        if let Config::Construct(_, _, _) = self {
            1
        } else {
            0
        }
    }
}

impl fmt::Display for InnermostStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.read().iter().enumerate() {
            if !term.is_default() {
                writeln!(f, "{}\t{}", i, term)?;
            } else {
                writeln!(f, "{}\t<default>", i)?;
            }
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in self.configs.read().iter() {
            writeln!(f, "\t{}", config)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Rewrite(result) => write!(f, "Rewrite({})", result),
            Config::Construct(symbol, arity, result) => {
                write!(f, "Construct({}, {}, {})", symbol, arity, result)
            }
            Config::Return() => write!(f, "Return()"),
        }
    }
}

/// A stack for the right-hand side.
pub struct RHSStack {
    /// The innermost rewrite stack for the right hand side and the positions that must be added to the stack.
    innermost_stack: Protected<Vec<Config>>,
    variables: Vec<(ExplicitPosition, usize)>,
    stack_size: usize,
}

impl RHSStack {
    /// Construct a new right-hand stack for a given equation/rewrite rule.
    pub fn new(rule: &Rule) -> RHSStack {
        let var_map = create_var_map(&rule.lhs.clone().into());

        // Compute the extra information for the InnermostRewriter.
        let mut innermost_stack: Protected<Vec<Config>> = Protected::new(vec![]);
        let mut variables = vec![];
        let mut stack_size = 0;

        for (term, position) in PositionIterator::new(rule.rhs.copy().into()) {
            if let Some(index) = position.indices.last() {
                if *index == 1 {
                    continue; // Skip the function symbol.
                }
            }

            if is_data_variable(&term) {
                variables.push((
                    var_map
                        .get(&term.protect())
                        .expect("All variables in the right hand side must occur in the left hand side")
                        .clone(),
                    stack_size,
                ));
                stack_size += 1;
            } else if is_data_machine_number(&term) {
                // Skip SortId(@NoValue) and OpId
            } else if is_data_expression(&term) {
                let t: DataExpressionRef = term.into();
                let arity = t.data_arguments().len();
                let mut write = innermost_stack.write();
                let symbol = write.protect(&t.data_function_symbol().into());
                write.push(Config::Construct(symbol.into(), arity, stack_size));
                stack_size += 1;
            } else {
                // Skip intermediate terms such as UntypeSortUnknown.
            }
        }

        RHSStack {
            innermost_stack,
            stack_size,
            variables,
        }
    }

    /// Evaluate the rhs stack for the given term and returns the result.
    pub fn evaluate(&self, term: &DataExpression) -> DataExpression {
        let mut stack = InnermostStack::default();
        stack.terms.write().push(DataExpressionRef::default());

        InnermostStack::integrate(&mut stack.configs.write(), &mut stack.terms.write(), self, term, 0);
        loop {
            trace!("{}", stack);

            let mut write_configs = stack.configs.write();
            if let Some(config) = write_configs.pop() {
                match config {
                    Config::Construct(symbol, arity, index) => {
                        // Take the last arity arguments.
                        let mut write_terms = stack.terms.write();
                        let length = write_terms.len();

                        let arguments = &write_terms[length - arity..];

                        let term: DataExpression = if arguments.is_empty() {
                            symbol.protect().into()
                        } else {
                            DataApplication::new(&symbol.copy(), arguments).into()
                        };

                        // Add the term on the stack.
                        write_terms.drain(length - arity..);
                        let t = write_terms.protect(&term);
                        write_terms[index] = t.into();
                    }
                    Config::Rewrite(_) => {
                        unreachable!("This case should not happen");
                    }
                    Config::Return() => {
                        unreachable!("This case should not happen");
                    }
                }
            } else {
                break;
            }
        }

        debug_assert!(
            stack.terms.read().len() == 1,
            "Expect exactly one term on the result stack"
        );

        let mut write_terms = stack.terms.write();

        write_terms
            .pop()
            .expect("The result should be the last element on the stack")
            .protect()
    }
}

impl Clone for RHSStack {
    fn clone(&self) -> Self {
        // TODO: It would make sense if Protected could implement Clone.
        let mut innermost_stack: Protected<Vec<Config>> = Protected::new(vec![]);

        let mut write = innermost_stack.write();
        for t in self.innermost_stack.read().iter() {
            match t {
                Config::Rewrite(x) => write.push(Config::Rewrite(*x)),
                Config::Construct(f, x, y) => {
                    let f = write.protect(&f.copy().into());
                    write.push(Config::Construct(f.into(), *x, *y));
                }
                Config::Return() => write.push(Config::Return()),
            }
        }
        drop(write);

        Self {
            variables: self.variables.clone(),
            stack_size: self.stack_size,
            innermost_stack,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHashSet;
    use mcrl3_aterm::ATerm;
    use mcrl3_data::DataFunctionSymbol;

    use crate::test_utility::create_rewrite_rule;
    use crate::utilities::to_untyped_data_expression;

    use test_log::test;

    #[test]
    fn test_rhs_stack() {
        let rhs_stack =
            RHSStack::new(&create_rewrite_rule("fact(s(N))", "times(s(N), fact(N))", &["N"]).unwrap());
        let mut expected = Protected::new(vec![]);

        let mut write = expected.write();
        let t = write.protect(&DataFunctionSymbol::new("times").copy().into());
        write.push(Config::Construct(t.into(), 2, 0));

        let t = write.protect(&DataFunctionSymbol::new("s").copy().into());
        write.push(Config::Construct(t.into(), 1, 1));

        let t = write.protect(&DataFunctionSymbol::new("fact").copy().into());
        write.push(Config::Construct(t.into(), 1, 2));
        drop(write);

        // Check if the resulting construction succeeded.
        assert_eq!(
            rhs_stack.innermost_stack, expected,
            "The resulting config stack is not as expected"
        );

        assert_eq!(rhs_stack.stack_size, 5, "The stack size does not match");

        // Test the evaluation
        let lhs = ATerm::from_string("fact(s(a))").unwrap();
        let lhs_expression = to_untyped_data_expression(&lhs, &AHashSet::new());

        let rhs = ATerm::from_string("times(s(a), fact(a))").unwrap();
        let rhs_expression = to_untyped_data_expression(&rhs, &AHashSet::new());

        assert_eq!(
            rhs_stack.evaluate(&lhs_expression),
            rhs_expression,
            "The rhs stack does not evaluate to the expected term"
        );
    }

    #[test]
    fn test_rhs_stack_variable() {
        let rhs = RHSStack::new(&create_rewrite_rule("f(x)", "x", &["x"]).unwrap());

        // Check if the resulting construction succeeded.
        assert!(
            rhs.innermost_stack.read().is_empty(),
            "The resulting config stack is not as expected"
        );

        assert_eq!(rhs.stack_size, 1, "The stack size does not match");
    }
}
