use crate::engine::prelude::*;

use super::types::{LossReason, MoveSafety, ScoredMove};

pub struct MoveSafetyAnalyzer {
    my_player_id: PlayerId,
}

impl MoveSafetyAnalyzer {
    pub const fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    pub fn classify_move(&self, game_state: &GameState, direction: Direction) -> MoveSafety {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        let Some(next_pos) = my_head.after_moved(direction) else {
            return MoveSafety::Losing(LossReason::OutOfBounds);
        };

        match *grid.get_cell(next_pos) {
            GridCell::Empty => {}
            GridCell::Head(player, _) if player == self.my_player_id.other() => {
                return MoveSafety::Losing(LossReason::OpponentHeadCell);
            }
            _ => return MoveSafety::Losing(LossReason::OccupiedCell),
        }

        if self.is_immediate_head_to_head_risk(grid, direction) {
            return MoveSafety::RiskyHeadToHead;
        }

        MoveSafety::Safe
    }

    pub fn all_losing_fallback(&self, candidates: &[ScoredMove]) -> Direction {
        candidates
            .iter()
            .filter(|candidate| matches!(candidate.safety, MoveSafety::Losing(_)))
            .min_by(|a, b| {
                self.loss_priority(a.safety)
                    .cmp(&self.loss_priority(b.safety))
                    .then_with(|| {
                        self.direction_priority(a.direction)
                            .cmp(&self.direction_priority(b.direction))
                    })
            })
            .map(|candidate| candidate.direction)
            .unwrap_or(Direction::PositiveX)
    }

    pub fn legal_moves_for(&self, grid: &Grid, player_id: PlayerId) -> Vec<Direction> {
        let head = grid.player_head_position(player_id);

        Direction::all()
            .filter(|direction| {
                head.after_moved(*direction)
                    .is_some_and(|next_pos| grid.cell_is_empty(next_pos))
            })
            .collect()
    }

    pub fn simulate_paired_move(
        &self,
        grid: &Grid,
        my_direction: Direction,
        opponent_direction: Direction,
    ) -> NextFrameResult {
        if self.my_player_id.is_o() {
            grid.next_grid(my_direction, opponent_direction)
        } else {
            grid.next_grid(opponent_direction, my_direction)
        }
    }

    pub fn safety_rank(&self, safety: MoveSafety) -> usize {
        match safety {
            MoveSafety::Safe => 0,
            MoveSafety::RiskyHeadToHead => 1,
            MoveSafety::Losing(_) => 2,
        }
    }

    pub fn direction_priority(&self, direction: Direction) -> usize {
        match direction {
            Direction::PositiveX => 0,
            Direction::PositiveY => 1,
            Direction::NegativeX => 2,
            Direction::NegativeY => 3,
        }
    }

    fn is_immediate_head_to_head_risk(&self, grid: &Grid, my_direction: Direction) -> bool {
        self.legal_moves_for(grid, self.my_player_id.other())
            .into_iter()
            .any(|opponent_direction| {
                matches!(
                    self.simulate_paired_move(grid, my_direction, opponent_direction),
                    NextFrameResult::Draw
                )
            })
    }

    fn loss_priority(&self, safety: MoveSafety) -> usize {
        match safety {
            MoveSafety::Losing(LossReason::OpponentHeadCell) => 0,
            MoveSafety::Losing(LossReason::OccupiedCell) => 1,
            MoveSafety::Losing(LossReason::OutOfBounds) => 2,
            MoveSafety::Safe | MoveSafety::RiskyHeadToHead => usize::MAX,
        }
    }
}
