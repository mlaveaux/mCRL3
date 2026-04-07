use merc_utilities::MercError;
use oxidd::bdd::BDDFunction;
use oxidd::bdd::BDDManagerRef;
use oxidd::BooleanFunctionQuant;
use oxidd::BooleanOperator;
use oxidd::FunctionSubst;
use oxidd::Subst;
use oxidd::VarNo;

use crate::compute_vars_bdd;
use crate::variable_rename_reverse;
use crate::SummandGroupBdd;
use crate::SymbolicLtsBdd;

/// Computes the symbolic quotient of the given partition.
pub fn quotient_symbolic(
    manager_ref: &BDDManagerRef,
    lts: &SymbolicLtsBdd,
    partition: &BDDFunction,
    block_vars: &[VarNo],
) -> Result<SymbolicLtsBdd, MercError> {
    let mut quotient_relations = Vec::new();
    for group in lts.transition_groups() {
        let relation = quotient_naive(
            manager_ref,
            group.relation(),
            partition,
            lts.state_variables(),
            lts.next_state_variables(),
            block_vars,
        )?;

        quotient_relations.push(SummandGroupBdd::new(
            relation,
            group.read_variables().to_vec(),
            group.write_variables().to_vec(),
        ));
    }

    Ok(SymbolicLtsBdd::with_transition_groups(lts, quotient_relations))
}

/// Computes a new "interactive" transition relation that can be used to compute
/// the quotient of a partition.
///
/// # Details
///
/// The `relation` is defined over T(s, s', a) and `partition` is defined over P(s', b).
fn quotient_naive(
    manager_ref: &BDDManagerRef,
    relation: &BDDFunction,
    partition: &BDDFunction,
    state_variables: &[VarNo],
    next_state_variables: &[VarNo],
    block_vars: &[VarNo],
) -> Result<BDDFunction, MercError> {
    // Merge target states to the new encoding (in b). T(s, b, a) := ∃s′ : T (s, s′, a) ∧ P(s′, b)
    let (_block_var, block_variables) = compute_vars_bdd(manager_ref, block_vars)?;
    let relation_blocks = relation.apply_exists(BooleanOperator::And, partition, &block_variables)?;

    // Rename b variables to s′ variables. T (s, s′, a) := T (s, b, a)[b ← s′].
    let (next_state_vars, _) = compute_vars_bdd(manager_ref, next_state_variables)?;
    let substitution = Subst::new(block_vars, &next_state_vars);
    let relation_next = relation_blocks.substitute(&substitution)?;

    // 4 P′(s, b) ← rename(P, [s′ ← s]).
    let partition_prime = variable_rename_reverse(
        manager_ref,
        partition,
        &state_variables
            .iter()
            .zip(next_state_variables.iter())
            .map(|(s, s_prime)| (*s, *s_prime))
            .collect::<Vec<(VarNo, VarNo)>>(),
    )?;

    // T (s′, b, a) := ∃s : T (s, s′, a) ∧ P′(s, b)
    let (_, state_variables_bdd) = compute_vars_bdd(manager_ref, state_variables)?;
    let relation_prev = relation_next.apply_exists(BooleanOperator::And, &partition_prime, &state_variables_bdd)?;

    Ok(relation_prev)
}
