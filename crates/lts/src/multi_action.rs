#![forbid(unsafe_code)]

use std::fmt;
use std::hash::Hash;

use delegate::delegate;
use itertools::Itertools;

use merc_aterm::ATerm;
use merc_aterm::ATermArgs;
use merc_aterm::ATermIndex;
use merc_aterm::ATermList;
use merc_aterm::ATermRef;
use merc_aterm::ATermString;
use merc_aterm::Markable;
use merc_aterm::Symb;
use merc_aterm::SymbolRef;
use merc_aterm::Term;
use merc_aterm::TermIterator;
use merc_aterm::Transmutable;
use merc_aterm::storage::Marker;
use merc_collections::VecSet;
use merc_data::DataExpression;
use merc_data::DataVariable;
use merc_data::DataVariableRef;
use merc_data::is_data_variable;
use merc_macros::merc_derive_terms;
use merc_macros::merc_term;
use merc_utilities::MercError;

use crate::TransitionLabel;

/// Represents a multi-action, i.e., a set of action labels
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct LtsMultiAction {
    actions: VecSet<LtsAction>,
}

impl LtsMultiAction {
    /// Parses a multi-action from a string representation, typically found in the Aldebaran format.
    pub fn from_string(input: &str) -> Result<Self, MercError> {
        let mut actions = VecSet::new();

        for part in input.split('|') {
            let part = part.trim();
            if part.is_empty() {
                return Err("Empty action label in multi-action.".into());
            }

            if let Some(open_paren_index) = part.find('(') {
                if !part.ends_with(')') {
                    return Err(format!("Malformed action with arguments: {}", part).into());
                }

                let label = &part[..open_paren_index].trim();
                let args_str = &part[open_paren_index + 1..part.len() - 1];
                let arguments: Vec<DataExpression> = args_str
                    .split(',')
                    .map(|s| DataExpression::from_string(s.trim()))
                    .collect::<Result<_, MercError>>()?;
                actions.insert(LtsAction {
                    label: label.to_string(),
                    arguments,
                });
            } else {
                let label = part.trim();
                actions.insert(LtsAction {
                    label: label.to_string(),
                    arguments: Vec::new(),
                });
            }
        }

        Ok(LtsMultiAction { actions })
    }

    /// Converts the MultiAction into its mCRL2 ATerm representation.
    pub fn to_mcrl2_aterm(&self) -> Result<ATerm, MercError> {
        let action_terms: Vec<MCRL2Action> = self
            .actions
            .iter()
            .map(|action| -> Result<MCRL2Action, MercError> {
                let label_term = MCRL2ActionLabel::new(
                    ATermString::new(&action.label).copy(),
                    ATermList::<DataExpression>::empty(),
                );

                let arguments_term = ATermList::<DataExpression>::from_double_iter(action.arguments.iter().cloned());

                Ok(MCRL2Action::new(label_term.copy(), arguments_term))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let actions_list = ATermList::<MCRL2Action>::from_double_iter(action_terms.into_iter());
        let time_term: DataExpression = DataVariable::new("@undefined_real").into();
        Ok(MCRL2TimedMultiAction::new(actions_list, time_term.copy()).into())
    }

    /// Constructs a MultiAction from an mCRL2 ATerm representation.
    pub fn from_mcrl2_aterm(term: ATerm) -> Result<Self, MercError> {
        if is_mcrl2_timed_multi_action_symbol(&term.get_head_symbol()) {
            let multi_action = MCRL2TimedMultiAction::from(term);

            if is_data_variable(&multi_action.time()) {
                let variable: DataVariableRef = multi_action.time().into();
                if variable.name() != "@undefined_real" {
                    return Err("Timed multi-actions are not supported.".into());
                }
            } else {
                return Err("Timed multi-actions are not supported.".into());
            }

            let mut actions = VecSet::new();
            for action in multi_action.actions() {
                let arguments = action.arguments().to_vec();

                // Convert the label to string
                actions.insert(LtsAction {
                    label: action.label().name().to_string(),
                    arguments,
                });
            }

            Ok(LtsMultiAction { actions })
        } else {
            Err(format!("Expected TimedMultAction symbol, got {}.", term).into())
        }
    }
}

#[merc_derive_terms]
mod inner {
    use merc_aterm::ATermStringRef;
    use merc_aterm::Symbol;
    use merc_data::DataExpression;
    use merc_data::DataExpressionRef;
    use merc_macros::merc_ignore;

    use super::*;

    /// Represents a TimedMultiAction in mCRL2, which is a multi-action with an associated time.
    #[merc_term(is_mcrl2_timed_multi_action)]
    pub struct MCRL2TimedMultiAction {
        term: ATerm,
    }

    impl MCRL2TimedMultiAction {
        /// Creates a new TimedMultiAction with the given actions and time.
        #[merc_ignore]
        pub fn new(actions: ATermList<MCRL2Action>, time: DataExpressionRef<'_>) -> Self {
            let args: &[ATermRef<'_>] = &[actions.copy(), time.into()];
            let term = ATerm::with_args(&Symbol::new("TimedMultAct", 2), args);
            MCRL2TimedMultiAction { term: term.protect() }
        }

        /// Returns the actions contained in the multi-action.
        pub fn actions(&self) -> ATermList<MCRL2Action> {
            self.term.arg(0).into()
        }

        /// Returns the time at which the multi-action occurs.
        pub fn time(&self) -> DataExpressionRef<'_> {
            self.term.arg(1).into()
        }
    }

    #[merc_term(is_mcrl2_action)]
    pub struct MCRL2Action {
        term: ATerm,
    }

    impl MCRL2Action {
        /// Creates a new Action with the given label and arguments.
        #[merc_ignore]
        pub fn new(label: MCRL2ActionLabelRef<'_>, arguments: ATermList<DataExpression>) -> Self {
            let args: &[ATermRef<'_>] = &[label.into(), arguments.copy()];
            let term = ATerm::with_args(&Symbol::new("Action", 2), args);
            MCRL2Action { term: term.protect() }
        }

        /// Returns the label of the action.
        pub fn label(&self) -> MCRL2ActionLabelRef<'_> {
            self.term.arg(0).into()
        }

        /// Returns the data arguments of the action.
        pub fn arguments(&self) -> ATermList<DataExpression> {
            self.term.arg(1).into()
        }
    }

    #[merc_term(is_mcrl2_action_label)]
    pub struct MCRL2ActionLabel {
        term: ATerm,
    }

    impl MCRL2ActionLabel {
        /// Constructs a new action label with the given name and arguments.
        #[merc_ignore]
        pub fn new(name: ATermStringRef<'_>, args: ATermList<DataExpression>) -> Self {
            let args: &[ATermRef<'_>] = &[name.into(), args.copy()];
            let term = ATerm::with_args(&Symbol::new("ActId", 2), args);
            MCRL2ActionLabel { term: term.protect() }
        }

        /// Obtain the name of the action label.
        pub fn name(&self) -> ATermStringRef<'_> {
            self.term.arg(0).into()
        }
    }
}

pub use inner::*;

/// See [`is_mcrl2_timed_multi_action_symbol`]
fn is_mcrl2_timed_multi_action<'a, 'b, T: Term<'a, 'b>>(term: &'b T) -> bool {
    is_mcrl2_timed_multi_action_symbol(&term.get_head_symbol())
}

/// See [`is_mcrl2_action_symbol`]
fn is_mcrl2_action<'a, 'b, T: Term<'a, 'b>>(term: &'b T) -> bool {
    is_mcrl2_action_symbol(&term.get_head_symbol())
}

/// See [`is_mcrl2_action_label_symbol`]
fn is_mcrl2_action_label<'a, 'b, T: Term<'a, 'b>>(term: &'b T) -> bool {
    is_mcrl2_action_label_symbol(&term.get_head_symbol())
}

/// Checks if the given symbol represents a TimedMultiAction in mCRL2.
fn is_mcrl2_timed_multi_action_symbol(symbol: &SymbolRef<'_>) -> bool {
    symbol.name() == "TimedMultAct" && symbol.arity() == 2
}

/// Checks if the given symbol represents an Action in mCRL2.
fn is_mcrl2_action_symbol(symbol: &SymbolRef<'_>) -> bool {
    symbol.name() == "Action" && symbol.arity() == 2
}

/// Checks if the given symbol represents an ActionLabel in mCRL2.
fn is_mcrl2_action_label_symbol(symbol: &SymbolRef<'_>) -> bool {
    symbol.name() == "ActId" && symbol.arity() == 2
}

/// Represents a single action label, with its (data) arguments
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct LtsAction {
    label: String,
    arguments: Vec<DataExpression>,
}

impl LtsAction {
    /// Creates a new action label with the given name and arguments.
    pub fn new(label: String, arguments: Vec<DataExpression>) -> Self {
        LtsAction { label, arguments }
    }
}

impl TransitionLabel for LtsMultiAction {
    fn is_tau_label(&self) -> bool {
        self.actions.is_empty()
    }

    fn tau_label() -> Self {
        LtsMultiAction { actions: VecSet::new() }
    }

    fn matches_label(&self, label: &str) -> bool {
        // TODO: Is this correct, now a|b matches a?
        self.actions.iter().any(|action| action.label == label)
    }

    fn from_index(i: usize) -> Self {
        // For now we only generate single actions, but these could become multiactions as well
        LtsMultiAction {
            actions: VecSet::singleton(LtsAction::new(
                char::from_digit(i as u32, 36)
                    .expect("Radix is less than 37, so should not panic")
                    .to_string(),
                Vec::new(),
            )),
        }
    }
}

impl fmt::Display for LtsMultiAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.actions.is_empty() {
            write!(f, "τ")
        } else {
            write!(f, "{}", self.actions.iter().format("|"))
        }
    }
}

impl fmt::Display for LtsAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.arguments.is_empty() {
            write!(f, "{}", self.label)
        } else {
            write!(f, "{}({})", self.label, self.arguments.iter().format(", "))
        }
    }
}

impl fmt::Debug for LtsMultiAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the debug format to print the display format
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use merc_data::DataExpression;

    use crate::LtsMultiAction;

    #[test]
    fn test_multi_action_parse_string() {
        let action = LtsMultiAction::from_string("a | b(1, 2) | c").unwrap();

        assert_eq!(action.actions.len(), 3);
        assert!(
            action
                .actions
                .iter()
                .any(|act| act.label == "a" && act.arguments.is_empty())
        );
        assert!(action.actions.iter().any(|act| act.label == "b"
            && act.arguments
                == vec![
                    DataExpression::from_string("1").unwrap(),
                    DataExpression::from_string("2").unwrap()
                ]));
        assert!(
            action
                .actions
                .iter()
                .any(|act| act.label == "c" && act.arguments.is_empty())
        );
    }
}
