use std::cell::RefCell;

use mcrl2_sys::atermpp::ffi::_aterm;
use mcrl2_sys::cxx::UniquePtr;
use mcrl2_sys::lps::ffi::learn_successors_context;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_assignments;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_condition;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_multi_action;
use mcrl2_sys::lps::ffi::mcrl2_lps_action_summand_summation_variables;
use mcrl2_sys::lps::ffi::mcrl2_lps_create_learn_successors_context;
use mcrl2_sys::lps::ffi::mcrl2_lps_create_learn_successors_context_from_data_spec;
use mcrl2_sys::lps::ffi::mcrl2_lps_enumerate;
use mcrl2_sys::lps::ffi::mcrl2_lps_load_from_lps_file;
use mcrl2_sys::lps::ffi::mcrl2_lps_load_from_text_file;
use mcrl2_sys::lps::ffi::mcrl2_lps_multi_action_to_string;
use mcrl2_sys::lps::ffi::mcrl2_lps_num_of_action_summands;
use mcrl2_sys::lps::ffi::mcrl2_lps_preprocess_symbolic_exploration;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_initializer_expressions;
use mcrl2_sys::lps::ffi::mcrl2_lps_process_parameters;
use mcrl2_sys::lps::ffi::mcrl2_lps_set_assignments;
use mcrl2_sys::lps::ffi::mcrl2_lps_tau_multi_action;
use mcrl2_sys::lps::ffi::stochastic_action_summand;
use mcrl2_sys::lps::ffi::stochastic_process_initializer;
use mcrl2_sys::lps::ffi::stochastic_specification;

use merc_utilities::MercError;

use crate::ATerm;
use crate::ATermList;
use crate::DataExpression;
use crate::DataSpecification;
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

/// Pretty-prints a multi-action term using the mCRL2 pretty printer.
pub fn pretty_print_multi_action(multi_action: &ATerm) -> String {
    mcrl2_lps_multi_action_to_string(multi_action.get())
}

/// Returns the tau (empty) multi-action term as a protected [`ATerm`].
pub fn tau_multi_action() -> ATerm {
    ATerm::from_ptr(mcrl2_lps_tau_multi_action())
}

/// Read an LPS from a file in the binary mCRL2 format.
pub fn read_lps(filename: &str) -> Result<LinearProcessSpecification, MercError> {
    Ok(LinearProcessSpecification {
        lps: mcrl2_lps_load_from_lps_file(filename)?,
    })
}

/// Read an LPS from a textual mCRL2 process specification file.
pub fn read_lps_text(filename: &str) -> Result<LinearProcessSpecification, MercError> {
    Ok(LinearProcessSpecification {
        lps: mcrl2_lps_load_from_text_file(filename)?,
    })
}

/// Preprocess the LPS to make it suitable for symbolic exploration
pub fn preprocess(lps: &LinearProcessSpecification) -> Result<LinearProcessSpecification, MercError> {
    Ok(LinearProcessSpecification {
        lps: mcrl2_lps_preprocess_symbolic_exploration(lps.lps.as_ref().expect("The lps is always defined"))?,
    })
}

/// Context for learning successors during symbolic exploration.
///
/// Contains a rewriter, substitution (sigma), and enumerator instance
/// that are shared across all summands. Uses interior mutability so that
/// callers can keep a shared reference to the context while it is being
/// used (e.g. so the enumeration callback can borrow other state held
/// alongside the context).
pub struct LearnSuccessorsContext {
    context: RefCell<UniquePtr<learn_successors_context>>,
}

impl LearnSuccessorsContext {
    /// Creates a new context from the given LPS specification.
    pub fn new(lps: &LinearProcessSpecification) -> Self {
        LearnSuccessorsContext {
            context: RefCell::new(mcrl2_lps_create_learn_successors_context(
                lps.lps.as_ref().expect("The lps is always defined"),
            )),
        }
    }

    /// Creates a new context from a data specification (used when no LPS is available).
    pub fn from_data_spec(data_spec: &DataSpecification) -> Self {
        LearnSuccessorsContext {
            context: RefCell::new(mcrl2_lps_create_learn_successors_context_from_data_spec(
                data_spec.get(),
            )),
        }
    }

    /// Enumerate all solutions for the summand's condition under the given
    /// read parameter assignments from the current state.
    ///
    /// For each solution found, calls `callback` with a slice of the
    /// next-state values (one per assignment in the summand) and a pointer
    /// to the rewritten multi-action term.
    pub fn enumerate<F>(
        &self,
        summand: &LinearSummand,
        read_parameters: &[*const _aterm],
        read_values: &[*const _aterm],
        callback: F,
    ) where
        F: FnMut(&[*const _aterm], *const _aterm),
    {
        let condition = summand.condition();
        let summation_variables = summand.summation_variables();
        let assignments = summand.assignments();
        let multi_action = summand.multi_action();
        self.set_assignments(read_parameters, read_values);
        self.enumerate_raw_inner(
            condition.get(),
            ATerm::from(summation_variables).get(),
            ATerm::from(assignments).get(),
            multi_action.get(),
            callback,
        );
    }

    /// Assign variables in the persistent substitution (sigma).
    pub fn set_assignments(&self, variables: &[*const _aterm], values: &[*const _aterm]) {
        assert_eq!(
            variables.len(),
            values.len(),
            "Variables and values must have equal length"
        );

        let mut context = self.context.borrow_mut();
        mcrl2_lps_set_assignments(
            context.as_mut().expect("The context is always defined"),
            variables,
            values,
        );
    }

    /// Enumerate using stored ATerm values directly.
    ///
    /// This is useful when the summand's condition, summation variables,
    /// assignments and multi-action have already been extracted and stored as
    /// aterms. For each solution the callback receives a slice of next-state
    /// values together with a pointer to the multi-action term rewritten under
    /// the current substitution.
    pub fn enumerate_raw<F>(
        &self,
        // Information of the summand
        condition: &DataExpression,
        summation_variables: &ATermList<DataVariable>,
        assignments: &ATermList<ATerm>,
        multi_action: &ATerm,
        // Encodes the current substitution
        read_parameters: &[*const _aterm],
        read_values: &[*const _aterm],
        callback: F,
    ) where
        F: FnMut(&[*const _aterm], *const _aterm),
    {
        self.set_assignments(read_parameters, read_values);
        self.enumerate_raw_inner(
            condition.get(),
            summation_variables.get(),
            assignments.get(),
            multi_action.get(),
            callback,
        );
    }

    /// Enumerate using stored ATerm values directly with assignments that are
    /// already present in sigma.
    pub fn enumerate_raw_with_current_assignments<F>(
        &self,
        condition: &DataExpression,
        summation_variables: &ATermList<DataVariable>,
        assignments: &ATermList<ATerm>,
        multi_action: &ATerm,
        callback: F,
    ) where
        F: FnMut(&[*const _aterm], *const _aterm),
    {
        self.enumerate_raw_inner(
            condition.get(),
            summation_variables.get(),
            assignments.get(),
            multi_action.get(),
            callback,
        );
    }

    /// The implementation of enumeration that takes raw aterm pointers. This is
    /// used by both `enumerate` and `enumerate_raw` to avoid code duplication.
    fn enumerate_raw_inner<F>(
        &self,
        condition: &_aterm,
        summation_variables: &_aterm,
        assignments: &_aterm,
        multi_action: &_aterm,
        mut callback: F,
    ) where
        F: FnMut(&[*const _aterm], *const _aterm),
    {
        /// Trampoline that casts the context pointer back to the closure and calls it.
        fn trampoline(context: *mut u8, values: &[*const _aterm], multi_action: *const _aterm) {
            // Safety: `context` points to a live `&mut dyn FnMut(...)` set by the caller below.
            let callback = unsafe { &mut *(context as *mut &mut dyn FnMut(&[*const _aterm], *const _aterm)) };
            callback(values, multi_action);
        }

        // Unsized coercion: &mut F → &mut dyn FnMut(...) produces a fat pointer (data + vtable).
        // Must be a named binding so its stack address is stable for the *mut u8 cast below;
        // an inline temporary would be dropped before the unsafe block, dangling the raw pointer.
        // NLL sees this borrow of the local `callback` as disjoint from the `self.context` borrow below.
        let mut callback_ref: &mut dyn FnMut(&[*const _aterm], *const _aterm) = &mut callback;
        let mut context = self.context.borrow_mut();
        unsafe {
            mcrl2_lps_enumerate(
                context.as_mut().expect("The context is always defined"),
                condition,
                summation_variables,
                assignments,
                multi_action,
                &mut callback_ref as *mut _ as *mut u8,
                trampoline,
            );
        }
    }
}
