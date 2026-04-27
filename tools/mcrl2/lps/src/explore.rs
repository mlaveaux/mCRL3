use std::fmt;

use mcrl2::LinearSummand;
use merc_collections::IndexedSet;
use merc_ldd::Ldd;
use merc_ldd::Storage;
use merc_ldd::compute_proj;
use merc_ldd::project;
use merc_symbolic::SymbolicLTS;
use merc_symbolic::TransitionGroup;
use merc_symbolic::reachability;

use mcrl2::DataExpression;
use mcrl2::LinearProcessSpecification;
use mcrl2::preprocess;

use merc_utilities::MercError;

/// Explore the linear process specification using symbolic reachability.
pub fn explore_lps(storage: &mut Storage, lps: &LinearProcessSpecification) -> Result<usize, MercError> {
    let symbolic_lts = SymbolicLinearProcessSpecification::new(storage, lps)?;

    reachability(storage, &symbolic_lts)
}

/// This struct provides a [merc_symbolic::SymbolicLTS] interface to a [mcrl2::LinearProcessSpecification].
struct SymbolicLinearProcessSpecification {
    /// The underlying linear process specification.
    #[allow(dead_code)]
    lps: LinearProcessSpecification,

    /// Maps data expressions to indices in the LDDs.
    #[allow(dead_code)]
    mapping: IndexedSet<DataExpression>,

    /// The symbolic summands of the LPS, which are obtained by preprocessing the LPS.
    symbolic_summands: Vec<SymbolicSummand>,

    /// The initial state of the LPS.
    initial_state: Ldd,
}

impl SymbolicLinearProcessSpecification {
    pub fn new(storage: &mut Storage, lps: &LinearProcessSpecification) -> Result<Self, MercError> {
        let lps = preprocess(lps)?;

        let mut symbolic_summands = Vec::new();
        for index in 0..lps.num_summands() {
            symbolic_summands.push(SymbolicSummand::new(storage, &lps.action_summand(index)?));
        }

        let initial_state = storage.protect(storage.empty_vector());

        Ok(SymbolicLinearProcessSpecification {
            lps,
            symbolic_summands,
            mapping: IndexedSet::new(),
            initial_state,
        })
    }
}

/// Represents a symbolic summand of a [mcrl2::LinearProcessSpecification].
struct SymbolicSummand {
    /// The LDD encoding the projection of the state space on the read variables of this summand.
    project_ldd: Ldd,

    /// The indices of the variables that are read by this summand, which is
    /// used to determine the projection of the state space for this summand.
    read_indices: Vec<u32>,
}

impl SymbolicSummand {
    /// Extract the required information from the given action summand that is required for symbolic exploration.
    pub fn new(storage: &mut Storage, _summand: &LinearSummand) -> Self {
        let read_indices = Vec::new();

        let project_ldd = compute_proj(storage, &read_indices);

        Self {
            project_ldd,
            read_indices,
        }
    }
}

impl SymbolicLTS for SymbolicLinearProcessSpecification {
    fn states(&self) -> &Ldd {
        unreachable!("The SymbolicLTS interface can only be explored");
    }

    fn initial_state(&self) -> &Ldd {
        &self.initial_state
    }

    fn transition_groups(&self) -> &[impl TransitionGroup] {
        &self.symbolic_summands
    }

    fn action_labels(&self) -> &[String] {
        todo!()
    }

    fn parameter_values(&self) -> &[Vec<merc_data::DataExpression>] {
        unreachable!("The parameter values are not required for symbolic exploration");
    }
}

impl TransitionGroup for SymbolicSummand {
    fn relation(&self) -> &Ldd {
        todo!()
    }

    fn read_indices(&self) -> &[u32] {
        &self.read_indices
    }

    fn write_indices(&self) -> &[u32] {
        todo!()
    }

    fn action_label_index(&self) -> Option<usize> {
        todo!()
    }

    fn meta(&self) -> &Ldd {
        todo!()
    }

    fn learn_successors(&self, storage: &mut Storage, todo: &Ldd) -> Result<Ldd, MercError> {
        let proj = project(storage, todo, &self.project_ldd);
        Ok(proj)
    }
}

impl fmt::Debug for SymbolicSummand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolicSummand")
            .field("read_indices", &self.read_indices)
            .finish()
    }
}
