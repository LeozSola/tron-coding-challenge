use std::fmt::Display;

use crate::engine::prelude::*;

/// A cell on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridCell {
    /// An empty cell. There is no player / wall in this cell.
    Empty,
    /// A cell that is part of a player's tail.
    /// 
    /// Contains the player id, the direction the tail is moving in, and how
    /// long the tail has been alive for (in frames).
    Tail(PlayerId, Direction, usize),
    /// A cell that is a player's head.
    /// 
    /// Contains the player id and the direction the head is moving in.
    Head(PlayerId, Direction)
}
impl GridCell {
    /// Returns true if the cell is empty (i.e. not a head or tail).
    pub fn is_empty(&self) -> bool {
        *self == GridCell::Empty
    }
    /// Returns true if the cell is not empty (i.e. is not a head or tail).
    pub fn is_not_empty(&self) -> bool {
        *self != GridCell::Empty
    }
    pub fn is_head(&self) -> bool {
        matches!(self, GridCell::Head(..))
    }
    pub fn is_tail(&self) -> bool {
        matches!(self, GridCell::Tail(..))
    }
    /// Returns true if the cell is part of the given player's head.
    pub fn is_players_head(&self, player: PlayerId) -> bool {
        if let GridCell::Head(p, ..) = self {
            player == *p
        } else {
            false
        }
    }
}
impl Display for GridCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self{
            GridCell::Empty => " .",
            GridCell::Tail(player_id, _direction, _) if player_id.is_o() => " o",
            GridCell::Tail(_player_id, _direction, _) => " x",
            GridCell::Head(_player_id, direction) => match direction {
                Direction::PositiveY => " ^",
                Direction::NegativeY => " v",
                Direction::PositiveX => " >",
                Direction::NegativeX => " <",
            },
        })
    }
}
