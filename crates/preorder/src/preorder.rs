use merc_lts::LTS;
use merc_utilities::Timing;

use crate::ExplorationStrategy;
use crate::is_failures_refinement;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum RefinementType {
    Trace,
}

/// Checks whether `impl_lts` refines `spec_lts` according to the given
/// `preorder`.
/// 
/// # Details
/// 
/// The `preprocess` flag indicates whether preprocessing should be applied to
/// the LTSs. The refinement checks often involve product constructions, which
/// reducing the state space beforehand can lead to significant performance
/// improvements. However, for quick failing checks the preprocessing could cause
/// unnecessary overhead.
pub fn refines<L: LTS>(impl_lts: L, spec_lts: L, preorder: RefinementType, preprocess: bool, timing: &mut Timing) -> bool {
    match preorder {
        RefinementType::Trace => is_failures_refinement::<L, false>(
            impl_lts,
            spec_lts,
            preorder,
            ExplorationStrategy::BFS,
            preprocess,
            timing,
        ),
    }
}
