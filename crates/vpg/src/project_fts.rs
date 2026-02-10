use merc_lts::LTS;
use merc_lts::LabelledTransitionSystem;
use merc_lts::LtsBuilderFast;
use merc_lts::TransitionLabel;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::bdd::BDDFunction;

use merc_symbolic::CubeIterAll;
use merc_symbolic::bits_to_bdd;
use merc_utilities::MercError;
use merc_utilities::Timing;
use oxidd::util::OptBool;

use crate::FeatureDiagram;
use crate::FeatureTransitionSystem;

/// Projects a variability parity game into a standard parity game by removing
/// edges that are not enabled by the given feature selection.
pub fn project_feature_transition_system<L: TransitionLabel>(
    fts: &FeatureTransitionSystem<L>,
    feature_selection: &BDDFunction,
) -> Result<LabelledTransitionSystem<L>, MercError> {
    let mut builder = LtsBuilderFast::new(
        fts.labels().iter().map(|label| label.label()).cloned().collect(),
        Vec::new(),
    );

    for v in fts.iter_states() {
        for edge in fts.outgoing_transitions(v) {
            // Check if the edge is enabled by the feature selection, if so, include it.
            if feature_selection
                .and(fts.labels()[edge.label].feature_expr())?
                .satisfiable()
            {
                builder.add_transition(v, fts.labels()[edge.label].label(), edge.to);
            }
        }
    }

    Ok(builder.finish(fts.initial_state_index(), false))
}

/// A projected configuration of a featured transition system.
pub struct ProjectedLts<L: TransitionLabel> {
    pub bits: Vec<OptBool>,
    pub bdd: BDDFunction,
    pub lts: LabelledTransitionSystem<L>,
}

/// Projects all configurations of a variability parity game into standard parity games.
pub fn project_feature_transition_system_iter<'a, L: TransitionLabel>(
    fts: &'a FeatureTransitionSystem<L>,
    fd: &'a FeatureDiagram,
    timing: &'a Timing,
) -> impl Iterator<Item = Result<(ProjectedLts<L>, &'a Timing), MercError>> {
    CubeIterAll::new(fd.configuration()).map(move |cube| {
        let cube = cube?;
        let variables = fd.features().values().cloned().collect::<Vec<_>>();

        let bdd = match bits_to_bdd(&fd.configuration().manager_ref(), &variables, &cube) {
            Ok(bdd) => bdd,
            Err(e) => return Err(MercError::from(e)),
        };

        let lts = timing.measure("project", || -> Result<_, MercError> {
            project_feature_transition_system(fts, &bdd)
        })?;

        Ok((ProjectedLts { bits: cube, bdd, lts }, timing))
    })
}
