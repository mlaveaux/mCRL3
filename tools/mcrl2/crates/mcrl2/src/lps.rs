use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_assignments;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_condition;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_multi_action;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_summation_variables;
use mcrl2_sys::lps::ffi::mcrl2_lps_load_from_lps_file;
use mcrl2_sys::lps::ffi::mcrl2_lps_num_of_action_summands;
use mcrl2_sys::lps::ffi::mcrl2_lps_preprocess_symbolic_exploration;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer_expressions;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_parameters;
use mcrl2_sys::lps::ffi::stochastic_action_summand;
use mcrl2_sys::lps::ffi::stochastic_process_initializer;
use mcrl2_sys::lps::ffi::stochastic_specification;

use merc_utilities::MercError;

use crate::ATerm;
use crate::ATermList;
use crate::DataExpression;
use crate::DataVariable;

/// A linear process specification.
///
/// This is a wrapper around the `mcrl2::lps::specification` class, which
/// represents a linear process specification (LPS) in mCRL2. An LPS is a number
/// of summands that each specify a condition-action-effect triple, and it
/// has an initial state.
pub struct LinearProcessSpecification {
    lps: UniquePtr<stochastic_specification>,
}

impl LinearProcessSpecification {
    /// Returns the initial process of the LPS, which is the process that is specified by the initial state.
    pub fn initial_process(&self) -> LinearProcessInitializer {
        LinearProcessInitializer {
            init: mcrl2_lps_process_initializer(self.lps.as_ref().expect("The lps is always defined"))
                .expect("The initial process is always defined"),
        }
    }

    /// Returns the parameters of the LPS as an aterm list of data variables.
    pub fn parameters(&self) -> ATermList<DataVariable> {
        ATermList::from(ATerm::from_ptr(mcrl2_lps_process_parameters(
            self.lps.as_ref().expect("The lps is always defined"),
        )))
    }

    /// Returns the number of summands in the LPS.
    pub fn num_summands(&self) -> usize {
        mcrl2_lps_num_of_action_summands(self.lps.as_ref().expect("The lps is always defined"))
    }

    /// Returns the action summand at the given index.
    pub fn action_summand(&self, index: usize) -> Result<LinearSummand, MercError> {
        Ok(LinearSummand {
            summand: mcrl2_lps_action_summand(self.lps.as_ref().expect("The lps is always defined"), index)?,
        })
    }
}

/// Represents a `condition-action-effect` summand of an LPS.
pub struct LinearSummand {
    summand: UniquePtr<stochastic_action_summand>,
}

impl LinearSummand {
    /// Returns the condition of this summand.
    pub fn condition(&self) -> DataExpression {
        DataExpression::new(ATerm::from_ptr(mcrl2_lps_action_summand_condition(
            self.summand.as_ref().expect("The summand is always defined"),
        )))
    }

    /// Returns the summation variables of this summand (the "sum" variables).
    pub fn summation_variables(&self) -> ATermList<DataVariable> {
        ATermList::from(ATerm::from_ptr(mcrl2_lps_action_summand_summation_variables(
            self.summand.as_ref().expect("The summand is always defined"),
        )))
    }

    /// Returns the multi-action of this summand.
    pub fn multi_action(&self) -> ATerm {
        ATerm::from_ptr(mcrl2_lps_action_summand_multi_action(
            self.summand.as_ref().expect("The summand is always defined"),
        ))
    }

    /// Returns the assignments (update) of this summand as an aterm list.
    /// Each assignment represents `variable := expression` for the next state.
    pub fn assignments(&self) -> ATermList<ATerm> {
        ATermList::from(ATerm::from_ptr(mcrl2_lps_action_summand_assignments(
            self.summand.as_ref().expect("The summand is always defined"),
        )))
    }
}

/// Represents the initial process of an LPS.
pub struct LinearProcessInitializer {
    init: UniquePtr<stochastic_process_initializer>,
}

impl LinearProcessInitializer {
    /// Returns the initial state expressions as an aterm list of data expressions.
    pub fn expressions(&self) -> ATermList<DataExpression> {
        ATermList::from(ATerm::from_ptr(mcrl2_lps_process_initializer_expressions(
            self.init.as_ref().expect("The initializer is always defined"),
        )))
    }
}

/// Read an LPS from a file in the binary mCRL2 format.
pub fn read_lps(filename: &str) -> Result<LinearProcessSpecification, MercError> {
    Ok(LinearProcessSpecification {
        lps: mcrl2_lps_load_from_lps_file(filename)?,
    })
}

/// Preprocess the LPS to make it suitable for symbolic exploration
pub fn preprocess(lps: &LinearProcessSpecification) -> Result<LinearProcessSpecification, MercError> {
    Ok(LinearProcessSpecification {
        lps: mcrl2_lps_preprocess_symbolic_exploration(lps.lps.as_ref().expect("The lps is always defined"))?,
    })
}
