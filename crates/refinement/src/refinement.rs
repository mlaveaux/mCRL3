use merc_lts::LTS;
use merc_utilities::Timing;

use crate::is_failures_refinement;
use crate::ExplorationStrategy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum RefinementType {
    Trace,
    Weaktrace,
}

/// Checks whether `impl_lts` refines `spec_lts` according to the given
/// `refinement`.
///
/// # Details
///
/// The `preprocess` flag indicates whether preprocessing should be applied to
/// the LTSs. The refinement checks often involve product constructions, and
/// reducing the state space beforehand can lead to significant performance
/// improvements. However, for quick failing checks the preprocessing could cause
/// unnecessary overhead.
pub fn refines<L: LTS>(
    impl_lts: L,
    spec_lts: L,
    refinement: RefinementType,
    preprocess: bool,
    counter_example: bool,
    timing: &mut Timing,
) -> (bool, Option<Vec<L::Label>>) {
    is_failures_refinement(
        impl_lts,
        spec_lts,
        refinement,
        ExplorationStrategy::BFS,
        preprocess,
        counter_example,
        timing,
    )
}
