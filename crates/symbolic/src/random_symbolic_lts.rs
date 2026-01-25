use core::num;

use merc_aterm::ATermString;
use merc_data::DataExpression;
use merc_data::DataSpecification;
use merc_data::DataVariable;
use merc_lts::LtsMultiAction;
use merc_lts::TransitionLabel;
use merc_utilities::MercError;
use rand::Rng;
use rand::seq::IndexedRandom;
use rand::seq::IteratorRandom;

use merc_ldd::Storage;
use merc_ldd::from_iter;
use merc_ldd::random_vector_set;

use crate::SummandGroup;
use crate::SymbolicLts;

/// Generates random symbolic LTSs for testing purposes.
pub fn random_symbolic_lts(
    rng: &mut impl Rng,
    storage: &mut Storage,
    num_state_variables: usize,
    num_action_labels: usize,
) -> Result<SymbolicLts, MercError> {
    let num_of_values = 5usize;

    // Determine the state set.
    let states = random_vector_set(rng, 100, num_state_variables, num_of_values as u32);
    let initial_state = states.iter().choose(rng).ok_or("at least one state generated")?;

    // The action labels
    let mut action_labels = Vec::new();
    for i in 0..num_action_labels {
        action_labels.push(LtsMultiAction::from_index(i));
    }

    // The parameter values
    let mut parameter_values = Vec::new();
    let value = DataExpression::from_string("1")?;
    for _ in 0..num_state_variables {
        parameter_values.push(Vec::from_iter(std::iter::repeat(value.clone()).take(num_of_values)));
    }

    // The parameter names
    let parameters = (0..num_state_variables)
        .map(|i| DataVariable::new(ATermString::new(format!("p{i}"))))
        .collect::<Vec<_>>();

    // Determine the summand groups.
    let mut summand_groups = Vec::new();
    for _ in 0..5 {
        let num_of_read_variables = rng.random_range(0..parameters.len());
        let read_parameters = parameters
            .choose_multiple(rng, num_of_read_variables)
            .cloned()
            .collect::<Vec<_>>();

        let num_of_write_variables = rng.random_range(0..parameters.len());
        let write_parameters = parameters
            .choose_multiple(rng, num_of_write_variables)
            .cloned()
            .collect::<Vec<_>>();

        // Reserve additional space for action labels.
        let relation = random_vector_set(rng, 10, read_parameters.len() + write_parameters.len() + 1, 5);
        let relation_bdd = from_iter(storage, relation.iter());

        summand_groups.push(SummandGroup::new(
            storage,
            &parameters,
            read_parameters,
            write_parameters,
            relation_bdd,
        )?)
    }

    let initial_state_ldd = from_iter(storage, std::iter::once(initial_state));
    let states_ldd = from_iter(storage, states.iter());

    Ok(SymbolicLts::new(
        DataSpecification::default(),
        states_ldd,
        initial_state_ldd,
        summand_groups,
        action_labels,
        parameter_values,
    ))
}
