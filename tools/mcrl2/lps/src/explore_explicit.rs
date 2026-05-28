use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use log::debug;

use mcrl2::_aterm;
use mcrl2::ATerm;
use mcrl2::ATermList;
use mcrl2::DataExpression;
use mcrl2::DataExpressionRef;
use mcrl2::DataVariable;
use mcrl2::LearnSuccessorsContext;
use mcrl2::LinearProcessSpecification;
use mcrl2::LinearSummand;
use mcrl2::free_variables_data_expression;
use mcrl2::preprocess;
use merc_collections::IndexedSet;
use merc_explore::LPS;
use merc_explore::Summand;
use merc_explore::explore;
use merc_lts::LtsBuilder;
use merc_utilities::MercError;
use merc_utilities::Timing;

/// Explore the linear process specification explicitly, forwarding the
/// discovered transitions to `builder`.
pub fn explore_lps_explicit<B>(
    builder: &mut B,
    lps: &LinearProcessSpecification,
    timing: &Timing,
) -> Result<(), MercError>
where
    B: LtsBuilder<String>,
{
    let lps = ExplicitLinearProcessSpecification::new(lps)?;
    debug!("{lps:?}");
    explore(builder, &lps, timing)
}

/// Explicit-state view of a [mcrl2::LinearProcessSpecification] that implements
/// the [merc_explore::LPS] trait.
///
/// State vectors are `Vec<u32>` where each entry is an index into a
/// per-parameter [IndexedSet] of the data expressions observed for that
/// parameter (see [Shared::mapping]). Labels are the printed multi-actions of
/// the summands.
struct ExplicitLinearProcessSpecification {
    /// The (preprocessed) underlying LPS.
    _lps: LinearProcessSpecification,

    /// The summands extracted from the LPS.
    summands: Vec<ExplicitSummand>,

    /// Information shared between all summands.
    _shared: Rc<Shared>,

    /// The initial state vector.
    initial_state: Vec<u32>,
}

impl ExplicitLinearProcessSpecification {
    fn new(lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps)?;

        let parameters = lps.parameters();
        let num_parameters = parameters.len();

        let shared = Rc::new(Shared {
            context: LearnSuccessorsContext::new(&lps),
            mapping: RefCell::new((0..num_parameters).map(|_| IndexedSet::new()).collect()),
        });

        let mut summands = Vec::new();
        for index in 0..lps.num_summands() {
            summands.push(ExplicitSummand::new(
                &lps.action_summand(index)?,
                &parameters,
                Rc::clone(&shared),
            ));
        }

        let initial_state = lps
            .initial_process()
            .expressions()
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let (index, _) = shared.mapping.borrow_mut()[i].insert(param.clone());
                *index as u32
            })
            .collect::<Vec<u32>>();

        debug_assert_eq!(
            initial_state.len(),
            num_parameters,
            "Initial state vector length must match number of parameters"
        );

        Ok(ExplicitLinearProcessSpecification {
            _lps: lps,
            summands,
            _shared: shared,
            initial_state,
        })
    }
}

/// State shared between [ExplicitSummand]s and the enclosing LPS.
struct Shared {
    /// Context used by mCRL2 to perform the enumeration. Behind a `RefCell` so
    /// the enumeration callback can re-enter Rust code that touches
    /// [Shared::mapping] while a call into the context is in progress.
    context: LearnSuccessorsContext,

    /// Bidirectional mapping between data expressions and indices, one
    /// [IndexedSet] per process parameter.
    mapping: RefCell<Vec<IndexedSet<DataExpression>>>,
}

/// A single summand of the LPS, prepared for explicit enumeration.
struct ExplicitSummand {
    /// The indices of the parameters that this summand reads.
    read_indices: Vec<u32>,

    /// Cached aterm pointers for the read parameters, in the same order as
    /// `read_indices`.
    read_parameters: Vec<*const _aterm>,

    /// The indices of the parameters that this summand writes (non-identity
    /// assignments).
    write_indices: Vec<u32>,

    /// The condition of this summand.
    condition: DataExpression,

    /// The summation variables of this summand.
    summation_variables: ATermList<DataVariable>,

    /// Only the non-identity assignments (write parameters) of this summand.
    write_assignments: ATermList<ATerm>,

    /// The static label associated with this summand. This is the printed
    /// multi-action template; data parameters bound by summation variables are
    /// not currently substituted into the label.
    label: String,

    /// Shared context owning the rewriter/enumerator and parameter mappings.
    shared: Rc<Shared>,
}

impl ExplicitSummand {
    fn new(summand: &LinearSummand, parameters: &ATermList<DataVariable>, shared: Rc<Shared>) -> Self {
        // Collect free variables from the condition.
        let mut read_vars = free_variables_data_expression(&summand.condition().copy());
        let parameters = parameters.to_vec();

        // Collect free variables from the update expressions and identify
        // which parameters are actually written.
        let mut write_vars = Vec::new();
        let mut write_assignments = Vec::new();

        for assignment in summand.assignments().iter() {
            let lhs: DataVariable = assignment.arg(0).protect().into();
            let rhs = assignment.arg(1);

            let rhs_vars = free_variables_data_expression(&rhs.copy().into());
            read_vars.extend(rhs_vars);

            if DataExpressionRef::from(lhs.copy()) != DataExpressionRef::from(rhs.copy()) {
                write_vars.push(lhs);
                write_assignments.push(assignment.protect());
            }
        }

        let write_assignments = ATermList::from_double_iter(write_assignments.into_iter());

        let read_indices: Vec<u32> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| read_vars.contains(param))
            .map(|(i, _)| i as u32)
            .collect();

        let read_parameters: Vec<*const _aterm> = read_indices
            .iter()
            .map(|&index| parameters[index as usize].address())
            .collect();

        let write_indices: Vec<u32> = parameters
            .iter()
            .enumerate()
            .filter(|(_, param)| write_vars.contains(param))
            .map(|(i, _)| i as u32)
            .collect();

        let condition: DataExpression = summand.condition();
        let summation_variables: ATermList<DataVariable> = summand.summation_variables();

        debug_assert_eq!(
            read_indices.len(),
            read_parameters.len(),
            "Number of read indices must match number of read parameters"
        );
        debug_assert_eq!(
            write_indices.len(),
            write_assignments.iter().count(),
            "Number of write indices must match number of write assignments"
        );
        debug_assert!(read_indices.iter().is_sorted(), "Read indices must be strictly sorted");
        debug_assert!(
            write_indices.iter().is_sorted(),
            "Write indices must be strictly sorted"
        );

        let label = format!("{}", summand.multi_action());

        Self {
            read_indices,
            read_parameters,
            write_indices,
            condition,
            summation_variables,
            write_assignments,
            label,
            shared,
        }
    }
}

impl LPS for ExplicitLinearProcessSpecification {
    type State = Vec<u32>;
    type Label = String;
    type Summand = ExplicitSummand;

    fn initial_state(&self) -> Self::State {
        self.initial_state.clone()
    }

    fn summands(&self) -> &[Self::Summand] {
        &self.summands
    }
}

impl Summand for ExplicitSummand {
    type State = Vec<u32>;
    type Label = String;

    fn enumerate<F>(&self, state: &Self::State, mut report: F) -> Result<(), MercError>
    where
        F: FnMut(Self::Label, Self::State) -> Result<(), MercError>,
    {
        // Translate the read parameter values from the current state into the
        // aterm pointers expected by the mCRL2 enumerator.
        let read_values: Vec<*const _aterm> = {
            let mapping = self.shared.mapping.borrow();
            self.read_indices
                .iter()
                .map(|&i| {
                    mapping[i as usize]
                        .get_by_index(state[i as usize] as usize)
                        .expect("Value must be in the mapping")
                        .address()
                })
                .collect()
        };

        self.shared.context.enumerate_raw(
            &self.condition,
            &self.summation_variables,
            &self.write_assignments,
            &self.read_parameters,
            &read_values,
            |values: &[*const _aterm]| {
                debug_assert_eq!(
                    values.len(),
                    self.write_indices.len(),
                    "Enumerated values must match number of write indices"
                );

                let mut next_state = state.clone();
                {
                    let mut mapping = self.shared.mapping.borrow_mut();
                    for (i, &value) in values.iter().enumerate() {
                        let param_index = self.write_indices[i] as usize;
                        let new_index = mapping[param_index]
                            .insert(DataExpression::from(ATerm::from_ptr(value)))
                            .0;
                        next_state[param_index] = *new_index as u32;
                    }
                }

                // We cannot propagate errors from the C callback, so we panic on error and catch it in the caller.
                report(self.label.clone(), next_state).expect("Failed to report successor state");
            },
        );

        Ok(())
    }
}

impl fmt::Debug for ExplicitLinearProcessSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ExplicitLinearProcessSpecification:")?;

        writeln!(f, "  Parameters:")?;
        for (i, param) in self._lps.parameters().iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, param)?;
        }

        writeln!(f, "  Summands:")?;
        for (i, summand) in self.summands.iter().enumerate() {
            writeln!(f, "    {:?}: {:?}", i, summand)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ExplicitSummand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} -> {}", self.condition.pretty_print(), self.label)?;
        writeln!(f, "\t\tread indices: {:?}", self.read_indices)?;
        writeln!(f, "\t\twrite indices: {:?}", self.write_indices)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use mcrl2::read_lps;
    use merc_lts::LTS;
    use merc_lts::LtsBuilderFast;
    use merc_lts::StateIndex;
    use merc_utilities::Timing;

    use super::explore_lps_explicit;

    #[test]
    fn test_mcrl2_explore_explicit_abp() {
        let Ok(mcrl2_path) = std::env::var("MCRL2_PATH") else {
            println!("Skipping test: MCRL2_PATH not set");
            return;
        };

        let mcrl22lps = Path::new(&mcrl2_path).join("mcrl22lps");

        let temp_dir = tempfile::tempdir().unwrap();
        let lps_path = temp_dir.path().join("abp.lps");

        let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/mCRL2/academic/abp/abp.mcrl2");

        let status = Command::new(&mcrl22lps)
            .arg(&spec_path)
            .arg(&lps_path)
            .status()
            .expect("Failed to execute mcrl22lps");
        assert!(status.success(), "mcrl22lps failed with status: {status}");

        let lps = read_lps(lps_path.to_str().expect("LPS path is valid UTF-8")).expect("Failed to read LPS");

        let mut builder: LtsBuilderFast<String> = LtsBuilderFast::new(Vec::new(), Vec::new());
        explore_lps_explicit(&mut builder, &lps, &Timing::new()).expect("Failed to explore LPS");
        let lts = builder.finish(StateIndex::new(0), false);

        assert_eq!(
            lts.num_of_states(),
            74,
            "ABP should have 74 reachable states (see examples/lts/abp.aut)"
        );
    }
}
