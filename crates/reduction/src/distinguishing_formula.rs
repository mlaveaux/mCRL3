#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use merc_lts::LabelIndex;
use rustc_hash::FxHashMap;

use crate::{NodeIndex, Partition, PartitionTree};

/// The internal formula representation used during reconstruction. It carries the
/// set of final blocks in which it holds (`truths`), which the greedy strategy
/// needs.
struct Formula {
    label: LabelIndex,
    negated: bool,
    conjuncts: Vec<Formula>,
    truths: BTreeSet<NodeIndex>,
}

impl Formula {
    /// Converts the internal formula to the public [`DistinguishingFormula`],
    /// resolving label indices through `labels` and dropping the `truths`
    /// bookkeeping.
    fn into_public<L: Clone>(self, labels: &[L]) -> DistinguishingFormula<L> {
        let diamond = DistinguishingFormula::Diamond {
            label: labels[self.label.value()].clone(),
            conjuncts: self
                .conjuncts
                .into_iter()
                .map(|conjunct| conjunct.into_public(labels))
                .collect(),
        };

        if self.negated {
            DistinguishingFormula::Negate(Box::new(diamond))
        } else {
            diamond
        }
    }
}

/// Walks a [`PartitionTree`] to reconstruct the distinguishing formula.
struct Reconstructor<'a> {
    tree: &'a PartitionTree,
    /// Memoizes the greatest-common-ancestor level for pairs of nodes.
    gca_memo: FxHashMap<(NodeIndex, NodeIndex), usize>,
}

impl Reconstructor<'_> {
    /// Returns the level `i` such that the two blocks are `i`-bisimilar, i.e. the
    /// level at which they last shared an ancestor.
    fn gca_level(&mut self, b1: NodeIndex, b2: NodeIndex) -> usize {
        let hi = b1.max(b2);
        let lo = b1.min(b2);

        if let Some(&result) = self.gca_memo.get(&(hi, lo)) {
            return result;
        }

        let result = if hi == lo {
            self.tree.level(hi)
        } else {
            let level_hi = self.tree.level(hi);
            let level_lo = self.tree.level(lo);

            // Lift the deeper block(s) one step towards the root.
            let mut parent_hi = hi;
            let mut parent_lo = lo;
            if level_hi <= level_lo {
                parent_lo = self.tree.parent(lo);
            }
            if level_lo <= level_hi {
                parent_hi = self.tree.parent(hi);
            }

            if parent_hi == parent_lo {
                self.tree.level(hi)
            } else {
                self.gca_level(parent_hi, parent_lo)
            }
        };

        self.gca_memo.insert((hi, lo), result);
        result
    }

    /// Climbs to the ancestor of `block` at the given level.
    fn lift_block(&self, mut block: NodeIndex, goal: usize) -> NodeIndex {
        while self.tree.level(block) > goal {
            block = self.tree.parent(block);
        }
        block
    }

    /// Computes the set of final blocks in which the formula holds.
    fn set_truths(&self, formula: &mut Formula) {
        // Start from the whole partition and intersect with each conjunct.
        let mut image: BTreeSet<NodeIndex> = self.tree.final_blocks.clone();
        for conjunct in &formula.conjuncts {
            image = image.intersection(&conjunct.truths).copied().collect();
        }

        // Pre-image along the label: every block with a `label`-edge into `image`.
        let mut pre_image: BTreeSet<NodeIndex> = BTreeSet::new();
        for &block in &self.tree.final_blocks {
            if self
                .tree
                .outgoing(block)
                .any(|(label, target)| label == formula.label && image.contains(&target))
            {
                pre_image.insert(block);
            }
        }

        formula.truths = if formula.negated {
            self.tree.final_blocks.difference(&pre_image).copied().collect()
        } else {
            pre_image
        };
    }

    /// Builds a formula that distinguishes `b1` from `b2`, where the formula
    /// holds in `b1` but not in `b2`.
    fn distinguish(&mut self, b1: NodeIndex, b2: NodeIndex) -> Formula {
        debug_assert_ne!(b1, b2, "cannot distinguish a block from itself");

        let level = self.gca_level(b1, b2);
        // When `level` is 0 the blocks differ on their immediate successors, so
        // targets are compared directly (no lifting). `wrapping_sub` makes the
        // goal larger than any level, so `lift_block` then never climbs.
        let lift_goal = level.wrapping_sub(1);

        // The observations of `b2`, with targets lifted to the distinguishing
        // level minus one.
        let b2_delta: BTreeSet<(LabelIndex, NodeIndex)> = self
            .tree
            .outgoing(b2)
            .map(|(label, target)| (label, self.lift_block(target, lift_goal)))
            .collect();

        // Find an observation of `b1` that `b2` cannot match.
        let distinguishing = self
            .tree
            .outgoing(b1)
            .find(|&(label, target)| !b2_delta.contains(&(label, self.lift_block(target, lift_goal))));

        let (dist_label, dist_target) = match distinguishing {
            Some(observation) => observation,
            None => {
                // The greedy strategy needs a negation: swap the blocks and
                // negate the result.
                let mut negated = self.distinguish(b2, b1);
                negated.negated = true;
                self.set_truths(&mut negated);
                return negated;
            }
        };

        // Collect the `b2`-targets reachable with the distinguishing label that
        // must all be excluded.
        let mut remaining: BTreeSet<NodeIndex> = self
            .tree
            .outgoing(b2)
            .filter(|&(label, _)| label == dist_label)
            .map(|(_, target)| target)
            .collect();

        let mut conjuncts = Vec::new();
        while !remaining.is_empty() {
            // Greedily distinguish the target most similar to `dist_target` first.
            let mut best_level = 0;
            let mut split_block = *remaining.iter().next().expect("remaining is non-empty");
            for &candidate in &remaining {
                let candidate_level = self.gca_level(dist_target, candidate);
                if candidate_level > best_level {
                    best_level = candidate_level;
                    split_block = candidate;
                }
            }

            let conjunct = self.distinguish(dist_target, split_block);

            // Keep only the targets that the conjunct does not yet exclude.
            remaining = remaining.intersection(&conjunct.truths).copied().collect();
            conjuncts.push(conjunct);
        }

        let mut formula = Formula {
            label: dist_label,
            negated: false,
            conjuncts,
            truths: BTreeSet::new(),
        };
        self.set_truths(&mut formula);
        formula
    }
}