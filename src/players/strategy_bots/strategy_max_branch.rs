use crate::engine::prelude::*;

pub struct StrategyMaxBranchBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyMaxBranchBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        Direction::all()
            .filter_map(|direction| {
                let next = my_head.after_moved(direction)?;
                if grid.cell_is_not_empty(next) {
                    return None;
                }

                Some((empty_neighbor_count(grid, next), direction_priority(direction), direction))
            })
            .max_by_key(|(branching, priority, _)| (*branching, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn empty_neighbor_count(grid: &Grid, position: GridPosition) -> usize {
    Direction::all()
        .filter_map(|direction| position.after_moved(direction))
        .filter(|&next| grid.cell_is_empty(next))
        .count()
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 3,
        Direction::PositiveX => 2,
        Direction::NegativeY => 1,
        Direction::NegativeX => 0,
    }
}