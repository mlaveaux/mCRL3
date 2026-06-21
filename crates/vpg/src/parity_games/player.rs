//! Authors: Maurice Laveaux and Sjef van Loo
use core::fmt;

use crate::Priority;

/// The two players in a parity game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Even,
    Odd,
}

impl Player {
    /// Constructs a player from its index. This can be used in algorithms where
    /// we have a 2-array, and 0 is Even and 1 is Odd.
    pub fn from_index(index: u8) -> Self {
        Self::try_from_index(index).unwrap_or_else(|| panic!("Invalid player index {}", index))
    }

    /// Constructs a player from its index, returning `None` for any value other
    /// than 0 (Even) or 1 (Odd). Use this when the index comes from untrusted
    /// input that should be reported as a recoverable error.
    pub fn try_from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Player::Even),
            1 => Some(Player::Odd),
            _ => None,
        }
    }

    /// Constructs a player from a priority.
    pub fn from_priority(priority: Priority) -> Self {
        if priority.value().is_multiple_of(2) {
            Player::Even
        } else {
            Player::Odd
        }
    }

    /// Returns the index of the player, the inverse of [Self::from_index].
    pub fn to_index(&self) -> usize {
        match self {
            Player::Even => 0,
            Player::Odd => 1,
        }
    }

    /// Returns the opponent of the current player.
    pub fn opponent(&self) -> Self {
        match self {
            Player::Even => Player::Odd,
            Player::Odd => Player::Even,
        }
    }

    /// Returns the string representation of the solution for this player.
    pub fn solution(&self) -> &'static str {
        match self {
            Player::Even => "true",
            Player::Odd => "false",
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::Even => write!(f, "even"),
            Player::Odd => write!(f, "odd"),
        }
    }
}
