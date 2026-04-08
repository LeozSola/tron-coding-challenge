use crate::engine::prelude::*;

pub struct StrategyWallHugBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyWallHugBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        Direction::all()
            .filter_map(|direction| {
                let next = my_head.after_moved(direction)?;
                grid.cell_is_empty(next)
                    .then_some((edge_distance(next), direction_priority(direction), direction))
            })
            .min_by_key(|(edge_distance, priority, _)| (*edge_distance, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn edge_distance(position: GridPosition) -> usize {
    let max = GRID_SIZE - 1;
    position
        .x()
        .min(max - position.x())
        .min(position.y().min(max - position.y()))
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 0,
        Direction::PositiveX => 1,
        Direction::NegativeY => 2,
        Direction::NegativeX => 3,
    }
}