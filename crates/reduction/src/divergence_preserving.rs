#![forbid(unsafe_code)]

use delegate::delegate;

use merc_lts::LTS;
use merc_lts::LabelIndex;
use merc_lts::LabelledTransitionSystem;
use merc_lts::StateIndex;
use merc_lts::Transition;

/// For divergence-preserving branching bisimulation, we only need to treat
/// tau-self-loops as non-inert transitions. This can be achieved by temporarily
/// renaming the tau-self-loops to self-loops with a special label.
pub struct DivergencePreservingLts<'a, L: LTS> {
    /// The special label used to mark tau-self-loops, this should not be used in the original LTS.
    tau_self_loops_label: LabelIndex,

    /// We copy the labels and add the special label at the end.
    labels: Vec<L::Label>,

    lts: &'a L,
}

impl<'a, L: LTS> DivergencePreservingLts<'a, L> {
    pub fn new(lts: &'a L) -> Self {
        // We simply use the same tau label for the special label.
        let tau_self_loops = lts.num_of_labels();
        let labels = lts
            .labels()
            .iter()
            .cloned()
            .chain(std::iter::once(lts.labels()[0].clone()))
            .collect();

        DivergencePreservingLts {
            tau_self_loops_label: LabelIndex::new(tau_self_loops),
            labels,
            lts,
        }
    }
}

impl<L: LTS> LTS for DivergencePreservingLts<'_, L> {
    type Label = L::Label;

    fn outgoing_transitions(&self, state_index: StateIndex) -> impl Iterator<Item = Transition> + '_ {
        self.lts.outgoing_transitions(state_index).map(move |transition| {
            // A self-loop with tau should be renamed to the special label.
            if self.lts.is_hidden_label(transition.label) && state_index == transition.to {
                Transition {
                    label: self.tau_self_loops_label,
                    to: transition.to,
                }
            } else {
                transition
            }
        })
    }

    fn num_of_labels(&self) -> usize {
        self.lts.num_of_labels() + 1
    }

    fn labels(&self) -> &[Self::Label] {
        &self.labels
    }

    fn merge_disjoint<U: LTS<Label = Self::Label>>(
        self,
        _other: &U,
    ) -> (LabelledTransitionSystem<Self::Label>, StateIndex) {
        unimplemented!(
            "merge_disjoint is not implemented for DivergencePreservingLts, because this should only be used as a view on the original LTS."
        );
    }

    delegate! {
        to self.lts {
            fn initial_state_index(&self) -> StateIndex;
            fn num_of_states(&self) -> usize;
            fn num_of_transitions(&self) -> usize;
            fn iter_states(&self) -> impl Iterator<Item = StateIndex> + '_;
            fn is_hidden_label(&self, label_index: LabelIndex) -> bool;
        }
    }
}
