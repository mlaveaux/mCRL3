use mcrl2::LinearProcessSpecification;
use merc_explore::LPS;
use merc_utilities::MercError;

use crate::control_flow::ControlFlowAnalysis;
use crate::explore_explicit::ExplicitContext;
use crate::explore_explicit::ExplicitLinearProcessSpecification;
use crate::explore_explicit::ExplicitSummand;
use crate::explore_explicit::Mcrl2MultiActionLabel;

/// An explicit-state LPS view that selectively enables summands based on a
/// [`ControlFlowAnalysis`].
///
/// The [`LPS::prepare`] implementation drops summands whose control flow guard
/// cannot hold in the current state, mirroring how a PBES in SRF form only
/// explores the summands belonging to its current equation.
///
/// Pruning never changes the explored transition system: a dropped summand has a
/// guard that is false in the current state and would have produced no
/// transitions anyway.
pub(crate) struct CfgLinearProcessSpecification {
    /// The underlying explicit LPS that performs the actual enumeration.
    inner: ExplicitLinearProcessSpecification,

    /// The control flow graph analysis driving the summand selection.
    analysis: ControlFlowAnalysis,
}

impl CfgLinearProcessSpecification {
    /// Builds the explicit LPS and runs the control flow graph analysis on top.
    pub(crate) fn new(lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let inner = ExplicitLinearProcessSpecification::new(lps)?;
        let analysis = ControlFlowAnalysis::new(&inner);
        Ok(Self { inner, analysis })
    }

    /// Returns the process parameter indices identified as control flow
    /// parameters.
    pub(crate) fn control_flow_parameters(&self) -> &[usize] {
        &self.analysis.control_flow_parameters
    }
}

impl LPS for CfgLinearProcessSpecification {
    type Value = usize;
    type Label = Mcrl2MultiActionLabel;
    type StateInfo = ();
    type Summand = ExplicitSummand;

    fn initial_state(&self) -> Vec<usize> {
        self.inner.initial_state()
    }

    fn summands(&self) -> &[Self::Summand] {
        self.inner.summands()
    }

    fn create_context(&self) -> ExplicitContext {
        self.inner.create_context()
    }

    fn prepare<'a>(
        &'a self,
        context: &mut ExplicitContext,
        state: &'a [Self::Value],
    ) -> impl Iterator<Item = usize> + 'a {
        // Stage the per-state substitution in the enumeration backend; the
        // unfiltered summand list returned by the explicit LPS is discarded in
        // favour of the control-flow-filtered one below.
        let _staged = self.inner.prepare(context, state);

        // The returned iterator borrows `state` directly and reads the control
        // flow values on demand, avoiding a per-state allocation.
        let control_flow_parameters = &self.analysis.control_flow_parameters;
        let source_constraints = &self.analysis.source_constraints;

        (0..source_constraints.len()).filter(move |&index| {
            source_constraints[index]
                .iter()
                .all(|&(position, value)| state[control_flow_parameters[position]] == value)
        })
    }

    fn state_info(&self, _state: &[Self::Value]) -> Self::StateInfo {}
}
