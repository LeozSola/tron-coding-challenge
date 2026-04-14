use crate::engine::prelude::*;
use std::collections::HashMap;

pub use crate::GRID_SIZE;

/// Represents the grid of the game, which contains information about where the
/// players are and where the walls are.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid([GridCell; GRID_SIZE * GRID_SIZE]);

impl Grid {
    pub fn new_default(random_spawns: bool) -> Self {
        let mut out = Self([const { GridCell::Empty }; GRID_SIZE * GRID_SIZE]);

        let (o_pos, x_pos) = if random_spawns {
            let mut o_pos = (GRID_SIZE / 2, GRID_SIZE / 2);
            // Ensure O doesn't spawn in the middle --- then you can't mirror/rotate it for X's spawn.
            while o_pos.0 == GRID_SIZE / 2 && o_pos.1 == GRID_SIZE / 2 {
                o_pos = (rand::random_range(0..GRID_SIZE), rand::random_range(0..GRID_SIZE));
            }

            // Ensure equal footing by mirroring / rotating the O spawn to get the X spawn.
            let mut x_pos = o_pos;
            while x_pos == o_pos {
                if rand::random() {
                    x_pos = (x_pos.1, x_pos.0);
                }
                if rand::random() {
                    x_pos = (GRID_SIZE - 1 - x_pos.0, x_pos.1);
                }
                if rand::random() {
                    x_pos = (x_pos.0, GRID_SIZE - 1 - x_pos.1);
                }
            }

            (o_pos, x_pos)
        } else {
            ((9, 10), (11, 10))
        };

        *out.try_get_cell_mut(o_pos).expect("pos is in bounds") =
            GridCell::Head(PlayerId::new_o(), Direction::NegativeX);
        *out.try_get_cell_mut(x_pos).expect("pos is in bounds") =
            GridCell::Head(PlayerId::new_x(), Direction::PositiveX);

        out
    }
    pub fn next_grid(&self, o_choice: Direction, x_choice: Direction, next_frame: usize) -> NextFrameResult {

        //function is a hot mess

        let (o_pos, x_pos) = self.player_head_positions();

        let next_o_pos = o_pos.after_moved(o_choice);
        let next_x_pos = x_pos.after_moved(x_choice);

        if next_o_pos.is_none() && next_x_pos.is_none() {
            return NextFrameResult::Draw;
        };

        let Some(next_o_pos) = next_o_pos else {
            return NextFrameResult::Winner {
                player_who_won: PlayerId::new_x(),
            };
        };
        let Some(next_x_pos) = next_x_pos else {
            return NextFrameResult::Winner {
                player_who_won: PlayerId::new_o(),
            };
        };

        if next_o_pos == next_x_pos {
            return NextFrameResult::Draw;
        };

        let next_o_cell = self.get_cell(next_o_pos);
        let next_x_cell = self.get_cell(next_x_pos);

        let o_blocked = next_o_cell.is_not_empty();
        let x_blocked = next_x_cell.is_not_empty();

        if o_blocked && x_blocked {
            return NextFrameResult::Draw;
        }else if o_blocked {
            return NextFrameResult::Winner {
                player_who_won: PlayerId::new_x(),
            };
        }else if x_blocked {
            return NextFrameResult::Winner {
                player_who_won: PlayerId::new_o(),
            };
        };

        let mut out = self.clone();
        *out.get_cell_mut(o_pos) = GridCell::Tail(PlayerId::new_o(), o_choice, next_frame);
        *out.get_cell_mut(x_pos) = GridCell::Tail(PlayerId::new_x(), x_choice, next_frame);
        *out.get_cell_mut(next_o_pos) = GridCell::Head(PlayerId::new_o(), o_choice);
        *out.get_cell_mut(next_x_pos) = GridCell::Head(PlayerId::new_x(), x_choice);

        NextFrameResult::NextFrame(out)
    }

    pub fn get_cell_mut(&mut self, pos: impl Into<GridPosition>) -> &mut GridCell {
        self.0
            .get_mut(pos.into().i())
            .expect("position is in bounds")
    }
    pub fn try_get_cell_mut(&mut self, pos: impl TryInto<GridPosition>) -> Option<&mut GridCell> {
        self.0.get_mut(pos.try_into().ok()?.i())
    }

    /// Gets the cell at the given position.
    pub fn get_cell(&self, pos: impl Into<GridPosition>) -> &GridCell {
        self.0
            .get(pos.into().i())
            .expect("position is in bounds")
    }
    /// Gets the cell at the given position, or None if the position is out of bounds.
    pub fn try_get_cell(&self, pos: impl TryInto<GridPosition>) -> Option<&GridCell> {
        self.0.get(pos.try_into().ok()?.i())
    }

    /// Returns a map from player ID to that player's head position.
    pub fn head_positions_map(&self) -> HashMap<PlayerId, GridPosition> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(pos, cell)| {
                let GridCell::Head(player_id, ..) = cell else {
                    return None;
                };
                Some((
                    *player_id,
                    GridPosition::new_from_usize(pos).expect("position is valid"),
                ))
            })
            .collect()
    }

    /// Returns (Player A Head Position, Player B Head Position)
    pub fn player_head_positions(&self) -> (GridPosition, GridPosition) {
        (
            self.player_head_position(PlayerId::new_o()),
            self.player_head_position(PlayerId::new_x()),
        )
    }
    /// Returns (Player A Head Position, Player B Head Position)
    pub fn player_head_positions_slice(&self) -> [GridPosition; 2] {
        [
            self.player_head_position(PlayerId::new_o()),
            self.player_head_position(PlayerId::new_x()),
        ]
    }
    /// Returns the head position of the given player ID.
    pub fn player_head_position(&self, id: PlayerId) -> GridPosition {
        self.0
            .iter()
            .enumerate()
            .find_map(|(pos, cell)| {
                let GridCell::Head(player_id, ..) = cell else {
                    return None;
                };
                if *player_id != id {
                    return None;
                };
                GridPosition::new_from_usize(pos)
            })
            .expect("both players must have a head")
    }
    /// Returns the head direction of the given player ID.
    pub fn player_head_direction(&self, id: PlayerId)->Direction{
        *self.0
            .iter()
            .find_map(|cell|{
                let GridCell::Head(player_id, direction) = cell else {return None};
                if *player_id != id {return None};
                Some(direction)
            })
            .expect("both players must have a head")
    }

    pub fn cell_is_empty(&self, pos: impl Into<GridPosition>) -> bool {
        self.get_cell(pos).is_empty()
    }
    pub fn cell_is_not_empty(&self, pos: impl Into<GridPosition>) -> bool {
        self.get_cell(pos).is_not_empty()
    }

    /// Returns a reference to the array of all cells in the grid, in order of their index (starting at 0).
    pub fn get_cells(&self) -> &[GridCell; GRID_SIZE * GRID_SIZE] {
        &self.0
    }
}
impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string: String = String::new();
        for row in (0..GRID_SIZE).rev() {
            for col in 0..GRID_SIZE {
                let cell = self.try_get_cell((col, row)).expect("in bounds");
                string += &format!("{}", cell);
            }
            string += "\n";
        }
        write!(f, "{}", string)
    }
}
