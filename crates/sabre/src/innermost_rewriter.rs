use log::info;
use log::trace;
use mcrl3_aterm::ATermRef;
use mcrl3_aterm::ThreadTermPool;
use mcrl3_data::BoolSort;
use mcrl3_data::DataApplication;
use mcrl3_data::DataExpression;
use mcrl3_data::DataExpressionRef;

use crate::matching::conditions::extend_conditions;
use crate::matching::conditions::EMACondition;
use crate::matching::nonlinear::check_equivalence_classes;
use crate::matching::nonlinear::derive_equivalence_classes;
use crate::matching::nonlinear::EquivalenceClass;
use crate::set_automaton::MatchAnnouncement;
use crate::set_automaton::SetAutomaton;
use crate::utilities::Config;
use crate::utilities::InnermostStack;
use crate::utilities::PositionIndexed;
use crate::utilities::RHSStack;
use crate::utilities::SCCTBuilder;
use crate::RewriteEngine;
use crate::RewriteSpecification;
use crate::RewritingStatistics;
use crate::Rule;

impl RewriteEngine for InnermostRewriter {
    fn rewrite(&mut self, t: DataExpression) -> DataExpression {
        let mut stats = RewritingStatistics::default();

        trace!("input: {}", t);

        let mut tp = ThreadTermPool::reuse();

        let result = InnermostRewriter::rewrite_aux(
            &mut tp,
            &mut self.stack,
            &mut self.builder,
            &mut stats,
            &self.apma,
            t,
        );

        info!(
            "{} rewrites, {} single steps and {} symbol comparisons",
            stats.recursions, stats.rewrite_steps, stats.symbol_comparisons
        );
        result
    }
}

impl InnermostRewriter {
    pub fn new(spec: &RewriteSpecification) -> InnermostRewriter {
        let apma = SetAutomaton::new(spec, AnnouncementInnermost::new, true);

        InnermostRewriter {
            apma,
            stack: InnermostStack::default(),
            builder: SCCTBuilder::new(),
        }
    }

    /// Function to rewrite a term 't'. The elements of the automaton 'states'
    /// and 'tp' are passed as separate parameters to satisfy the borrow
    /// checker.
    ///
    /// # Details
    ///
    /// Uses a stack of terms and configurations to avoid recursions and to keep
    /// track of terms in normal forms without explicit tagging. The configuration
    /// stack consists of three different possible values with the following semantics
    ///     - Return(): Returns the top of the stack.
    ///     - Rewrite(index): Updates the configuration to rewrite the top of the term stack
    ///                       and places the result on the given index.
    ///     - Construct(arity, index, result):
    ///
    pub(crate) fn rewrite_aux(
        tp: &mut ThreadTermPool,
        stack: &mut InnermostStack,
        builder: &mut SCCTBuilder,
        stats: &mut RewritingStatistics,
        automaton: &SetAutomaton<AnnouncementInnermost>,
        input_term: DataExpression,
    ) -> DataExpression {
        debug_assert!(!input_term.is_default(), "Cannot rewrite the default term");

        stats.recursions += 1;
        {
            let mut write_terms = stack.terms.write();
            let mut write_configs = stack.configs.write();

            // Push the result term to the stack.
            let top_of_stack = write_terms.len();
            write_configs.push(Config::Return());
            write_terms.push(DataExpressionRef::default());
            InnermostStack::add_rewrite(&mut write_configs, &mut write_terms, input_term.copy(), top_of_stack);
        }

        loop {
            trace!("{}", stack);

            let mut write_configs = stack.configs.write();
            if let Some(config) = write_configs.pop() {
                match config {
                    Config::Rewrite(result) => {
                        let mut write_terms = stack.terms.write();
                        let term = write_terms.pop().unwrap();

                        let symbol = term.data_function_symbol();
                        let arguments = term.data_arguments();

                        // For all the argument we reserve space on the stack.
                        let top_of_stack = write_terms.len();
                        for _ in 0..arguments.len() {
                            write_terms.push(Default::default());
                        }

                        let symbol = write_configs.protect(&symbol.into());
                        InnermostStack::add_result(&mut write_configs, symbol.into(), arguments.len(), result);
                        for (offset, arg) in arguments.into_iter().enumerate() {
                            InnermostStack::add_rewrite(
                                &mut write_configs,
                                &mut write_terms,
                                arg.into(),
                                top_of_stack + offset,
                            );
                        }
                        drop(write_configs);
                    }
                    Config::Construct(symbol, arity, index) => {
                        // Take the last arity arguments.
                        let mut write_terms = stack.terms.write();
                        let length = write_terms.len();

                        let arguments = &write_terms[length - arity..];

                        let term: DataExpression = if arguments.is_empty() {
                            symbol.protect().into()
                        } else {
                            DataApplication::new(&symbol, arguments).into()
                        };

                        // Remove the arguments from the stack.
                        write_terms.drain(length - arity..);
                        drop(write_terms);
                        drop(write_configs);

                        match InnermostRewriter::find_match(tp, stack, builder, stats, automaton, &term) {
                            Some((announcement, annotation)) => {
                                trace!(
                                    "rewrite {} => {} using rule {}",
                                    term,
                                    annotation.rhs_stack.evaluate(&term),
                                    announcement.rule
                                );

                                // Reacquire the write access and add the matching RHSStack.
                                let mut write_terms = stack.terms.write();
                                let mut write_configs = stack.configs.write();
                                InnermostStack::integrate(
                                    &mut write_configs,
                                    &mut write_terms,
                                    &annotation.rhs_stack,
                                    &term,
                                    index,
                                );
                                stats.rewrite_steps += 1;
                            }
                            None => {
                                // Add the term on the stack.
                                let mut write_terms = stack.terms.write();
                                let t = write_terms.protect(&term);
                                write_terms[index] = t.into();
                            }
                        }
                    }
                    Config::Return() => {
                        let mut write_terms = stack.terms.write();

                        return write_terms
                            .pop()
                            .expect("The result should be the last element on the stack")
                            .protect();
                    }
                }

                let read_configs = stack.configs.read();
                for (index, term) in stack.terms.read().iter().enumerate() {
                    if term.is_default() {
                        debug_assert!(
                            read_configs.iter().any(|x| {
                                match x {
                                    Config::Construct(_, _, result) => index == *result,
                                    Config::Rewrite(result) => index == *result,
                                    Config::Return() => true,
                                }
                            }),
                            "The default term at index {index} is not a result of any operation."
                        );
                    }
                }
            }
        }
    }

    /// Use the APMA to find a match for the given term.
    fn find_match<'a>(
        tp: &mut ThreadTermPool,
        stack: &mut InnermostStack,
        builder: &mut SCCTBuilder,
        stats: &mut RewritingStatistics,
        automaton: &'a SetAutomaton<AnnouncementInnermost>,
        t: &ATermRef<'_>,
    ) -> Option<(&'a MatchAnnouncement, &'a AnnouncementInnermost)> {
        // Start at the initial state
        let mut state_index = 0;
        loop {
            let state = &automaton.states[state_index];

            // Get the symbol at the position state.label
            stats.symbol_comparisons += 1;
            let pos: DataExpressionRef<'_> = t.get_position(&state.label).into();
            let symbol = pos.data_function_symbol();

            // Get the transition for the label and check if there is a pattern match
            if let Some(transition) = automaton.transitions.get(&(state_index, symbol.operation_id())) {
                for (announcement, annotation) in &transition.announcements {
                    if check_equivalence_classes(t, &annotation.equivalence_classes)
                        && InnermostRewriter::check_conditions(tp, stack, builder, stats, automaton, annotation, t)
                    {
                        // We found a matching pattern
                        return Some((announcement, annotation));
                    }
                }

                // If there is no pattern match we check if the transition has a destination state
                if transition.destinations.is_empty() {
                    // If there is no destination state there is no pattern match
                    return None;
                } else {
                    // Continue matching in the next state
                    state_index = transition.destinations.first().unwrap().1;
                }
            } else {
                return None;
            }
        }
    }

    /// Checks whether the condition holds for given match announcement.
    fn check_conditions(
        tp: &mut ThreadTermPool,
        stack: &mut InnermostStack,
        builder: &mut SCCTBuilder,
        stats: &mut RewritingStatistics,
        automaton: &SetAutomaton<AnnouncementInnermost>,
        announcement: &AnnouncementInnermost,
        t: &ATermRef<'_>,
    ) -> bool {
        for c in &announcement.conditions {
            let rhs: DataExpression = c.semi_compressed_rhs.evaluate_with(builder, t, tp).into();
            let lhs: DataExpression = c.semi_compressed_lhs.evaluate_with(builder, t, tp).into();

            let rhs_normal = InnermostRewriter::rewrite_aux(tp, stack, builder, stats, automaton, rhs);
            let lhs_normal = if lhs == BoolSort::true_term() {
                // TODO: Store the conditions in a better way. REC now uses a list of equalities while mCRL2 specifications have a simple condition.
                lhs
            } else {
                InnermostRewriter::rewrite_aux(tp, stack, builder, stats, automaton, lhs)
            };

            if lhs_normal != rhs_normal && c.equality || lhs_normal == rhs_normal && !c.equality {
                return false;
            }
        }

        true
    }
}

/// Innermost Adaptive Pattern Matching Automaton (APMA) rewrite engine.
pub struct InnermostRewriter {
    apma: SetAutomaton<AnnouncementInnermost>,
    stack: InnermostStack,
    builder: SCCTBuilder,
}

pub(crate) struct AnnouncementInnermost {
    /// Positions in the pattern with the same variable, for non-linear patterns
    equivalence_classes: Vec<EquivalenceClass>,

    /// Conditions for the left hand side.
    conditions: Vec<EMACondition>,

    /// The innermost stack for the right hand side of the rewrite rule.
    rhs_stack: RHSStack,
}

impl AnnouncementInnermost {
    fn new(rule: &Rule) -> AnnouncementInnermost {
        AnnouncementInnermost {
            conditions: extend_conditions(rule),
            equivalence_classes: derive_equivalence_classes(rule),
            rhs_stack: RHSStack::new(rule),
        }
    }
}

#[cfg(test)]
mod tests {
    use ahash::AHashSet;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;
    use test_log::test;

    use mcrl3_aterm::random_term;

    use crate::utilities::to_untyped_data_expression;
    use crate::InnermostRewriter;
    use crate::RewriteEngine;
    use crate::RewriteSpecification;

    #[test]
    fn test_innermost_simple() {
        let spec = RewriteSpecification { rewrite_rules: vec![] };
        let mut inner = InnermostRewriter::new(&spec);

        let seed: u64 = rand::rng().random();
        println!("seed: {}", seed);
        let mut rng = StdRng::seed_from_u64(seed);

        let term = random_term(
            &mut rng,
            &[("f".to_string(), 2)],
            &["a".to_string(), "b".to_string()],
            5,
        );
        let term = to_untyped_data_expression(&term, &AHashSet::new());

        assert_eq!(
            inner.rewrite(term.clone().into()),
            term.into(),
            "Should be in normal form for no rewrite rules"
        );
    }
}
