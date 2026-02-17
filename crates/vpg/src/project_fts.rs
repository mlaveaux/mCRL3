use log::debug;
use merc_lts::LabelIndex;
use merc_lts::LabelledTransitionSystem;
use merc_lts::LtsBuilderFast;
use merc_lts::LTS;
use merc_symbolic::FormatConfigSet;
use oxidd::bdd::BDDFunction;
use oxidd::BooleanFunction;
use oxidd::Function;
use oxidd::bdd::BDDFunction;

use merc_symbolic::CubeIterAll;
use merc_symbolic::bits_to_bdd;
use merc_utilities::MercError;
use merc_utilities::Timing;
use oxidd::util::OptBool;
use oxidd::util::OutOfMemory;

use crate::FeatureDiagram;
use crate::FeatureTransitionSystem;

/// Projects a variability parity game into a standard parity game by removing
/// edges that are not enabled by the given feature selection.
pub fn project_feature_transition_system<L: TransitionLabel>(
    fts: &FeatureTransitionSystem<L>,
    feature_selection: &BDDFunction,
) -> Result<LabelledTransitionSystem<String>, MercError> {
    let mut builder = LtsBuilderFast::new(fts.labels().to_vec(), Vec::new());

    debug!("Projecting on feature selection {}", FormatConfigSet(feature_selection));

    let labels = fts
        .labels()
        .iter()
        .enumerate()
        .map(|(label_index, label)| -> Result<bool, OutOfMemory> {
            // Check if the edge is enabled by the feature selection, if so, include it.
            let result = feature_selection.and(fts.feature_label(LabelIndex::new(label_index)))?.satisfiable();
            debug!("Label {} is included: {}", label, result);

            Ok(result)
        })
        .collect::<Result<Vec<_>, _>>()?;

    for v in fts.iter_states() {
        for edge in fts.outgoing_transitions(v) {
            if labels[edge.label] {
                builder.add_transition(v, &fts.labels()[edge.label], edge.to);
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
) -> impl Iterator<Item = Result<(ProjectedLts, &'a Timing), MercError>> {
    let variables = fd.feature_names().iter().map(|name| {
        fd.features()
            .get(name)
            .expect("Feature diagram should contain all features defined in the feature names")
            .clone()
    }).collect::<Vec<_>>();

    CubeIterAll::new(fd.configuration()).map(move |cube| {
        let cube = cube?;

        let bdd = match bits_to_bdd(&fd.configuration().manager_ref(), &variables, &cube) {
            Ok(bdd) => bdd,
            Err(e) => return Err(MercError::from(e)),
        };

        let lts = timing.measure("project", || -> Result<_, MercError> {
            project_feature_transition_system(fts, &bdd)
        })?;

        Ok((ProjectedLts { bits: cube, bdd, lts }, timing))
        Ok((ProjectedLts { bits: cube, bdd, lts }, timing))
    })
}
