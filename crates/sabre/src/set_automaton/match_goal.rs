use std::cmp::min;
use std::fmt;

use crate::utilities::DataPosition;
use ahash::HashMap;
use ahash::HashMapExt;
use log::trace;

use super::MatchAnnouncement;
use super::MatchObligation;

/// A match goal contains a number of obligations (positions that must still be
/// matched) and the corresponding rule that can be announced as being a match.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MatchGoal {
    pub obligations: Vec<MatchObligation>,
    pub announcement: MatchAnnouncement,
}

impl MatchGoal {
    pub fn new(announcement: MatchAnnouncement, obligations: Vec<MatchObligation>) -> Self {
        Self {
            obligations,
            announcement,
        }
    }

    /// Derive the greatest common prefix of the announcement and obligation positions
    /// of a list of match goals.
    pub fn greatest_common_prefix(goals: &Vec<MatchGoal>) -> DataPosition {
        // gcp is empty if there are no match goals
        if goals.is_empty() {
            return DataPosition::empty();
        }

        // Initialise the prefix with the first match goal, can only shrink afterwards
        let first_match_pos = &goals.first().unwrap().announcement.position;
        let mut gcp_length = first_match_pos.len();
        let prefix = &first_match_pos.clone();

        for g in goals {
            // Compare up to gcp_length or the length of the announcement position
            let compare_length = min(gcp_length, g.announcement.position.len());
            // gcp_length shrinks if they are not the same up to compare_length
            gcp_length = MatchGoal::common_prefix_length(
                &prefix.indices()[0..compare_length],
                &g.announcement.position.indices()[0..compare_length],
            );

            for mo in &g.obligations {
                // Compare up to gcp_length or the length of the match obligation position
                let compare_length = min(gcp_length, mo.position.len());
                // gcp_length shrinks if they are not the same up to compare_length
                gcp_length = MatchGoal::common_prefix_length(
                    &prefix.indices()[0..compare_length],
                    &mo.position.indices()[0..compare_length],
                );
            }
        }

        // The gcp is constructed by taking the first gcp_length indices of the first match goal prefix
        DataPosition::new(&prefix.indices()[0..gcp_length])
    }

    /// Removes the first len position indices of the match goal and obligation positions
    pub fn remove_prefix(mut goals: Vec<MatchGoal>, len: usize) -> Vec<MatchGoal> {
        for goal in &mut goals {
            // update match announcement
            goal.announcement.position = DataPosition::new(&goal.announcement.position.indices()[len..]);
            for mo_index in 0..goal.obligations.len() {
                let shortened = DataPosition::new(&goal.obligations.get(mo_index).unwrap().position.indices()[len..]);
                goal.obligations.get_mut(mo_index).unwrap().position = shortened;
            }
        }
        goals
    }

    /// Returns a Vec where each element is a partition containing the goals and
    /// the positions. This partitioning can be done in multiple ways, but
    /// currently match goals are equivalent when their match announcements have
    /// a comparable position.
    pub fn partition(goals: Vec<MatchGoal>) -> Vec<(Vec<MatchGoal>, Vec<DataPosition>)> {
        let mut partitions = vec![];

        trace!("=== partition(match_goals = [ ===");
        for mg in &goals {
            trace!("\t {mg:?}");
        }
        trace!("]");

        // If one of the goals has a root position all goals are related.
        partitions = if goals.iter().any(|g| g.announcement.position.is_empty()) {
            let mut all_positions = Vec::new();
            for g in &goals {
                if !all_positions.contains(&g.announcement.position) {
                    all_positions.push(g.announcement.position.clone())
                }
            }
            partitions.push((goals, all_positions));
            partitions
        } else {
            // Create a mapping from positions to goals, goals are represented with an index
            // on function parameter goals
            let mut position_to_goals = HashMap::new();
            for (i, g) in goals.iter().enumerate() {
                if !position_to_goals.contains_key(&g.announcement.position) {
                    position_to_goals.insert(g.announcement.position.clone(), vec![i]);
                } else {
                    let vec = position_to_goals.get_mut(&g.announcement.position).unwrap();
                    vec.push(i);
                }
            }

            // Sort the positions. They are now in depth first order.
            let mut all_positions: Vec<DataPosition> = position_to_goals.keys().cloned().collect();
            all_positions.sort_unstable();

            // Compute the partitions, finished when all positions are processed
            let mut p_index = 0; // position index
            while p_index < all_positions.len() {
                // Start the partition with a position
                let p = &all_positions[p_index];
                let mut pos_in_partition = vec![p.clone()];
                let mut goals_in_partition = vec![];

                // put the goals with position p in the partition
                let g = position_to_goals.get(p).unwrap();
                for i in g {
                    goals_in_partition.push(goals[*i].clone());
                }

                // Go over the positions until we find a position that is not comparable to p
                // Because all_positions is sorted we know that once we find a position that is not comparable
                // all subsequent positions will also not be comparable.
                // Moreover, all positions in the partition are related to p. p is the highest in the partition.
                p_index += 1;
                while p_index < all_positions.len() && MatchGoal::pos_comparable(p, &all_positions[p_index]) {
                    pos_in_partition.push(all_positions[p_index].clone());
                    // Put the goals with position all_positions[p_index] in the partition
                    let g = position_to_goals.get(&all_positions[p_index]).unwrap();
                    for i in g {
                        goals_in_partition.push(goals[*i].clone());
                    }
                    p_index += 1;
                }

                partitions.push((goals_in_partition, pos_in_partition));
            }

            partitions
        };

        for (goals, pos) in &partitions {
            trace!("pos {{");
            for mg in pos {
                trace!("\t {mg}");
            }
            trace!("}} -> {{");
            for mg in goals {
                trace!("\t {mg:?}");
            }
            trace!("}}");
        }

        partitions
    }

    // Assumes two slices are of the same length and computes to what length they are equal
    fn common_prefix_length(pos1: &[usize], pos2: &[usize]) -> usize {
        debug_assert_eq!(pos1.len(), pos2.len(), "Given arrays should be of the same length.");

        let mut common_length = 0;
        for i in 0..pos1.len() {
            if pos1.get(i).unwrap() == pos2.get(i).unwrap() {
                common_length += 1;
            } else {
                break;
            }
        }
        common_length
    }

    /// Checks for two positions whether one is a subposition of the other.
    /// For example 2.2.3 and 2 are comparable. 2.2.3 and 1 are not.
    pub fn pos_comparable(p1: &DataPosition, p2: &DataPosition) -> bool {
        let mut index = 0;
        loop {
            if p1.len() == index || p2.len() == index {
                return true;
            }

            if p1.indices()[index] != p2.indices()[index] {
                return false;
            }
            index += 1;
        }
    }
}

impl fmt::Debug for MatchGoal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for obligation in &self.obligations {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{obligation:?}")?;
            first = false;
        }

        write!(f, " ↪ {:?}", self.announcement)
    }
}
