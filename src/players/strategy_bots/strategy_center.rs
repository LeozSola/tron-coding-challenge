use crate::engine::prelude::*;

pub struct StrategyCenterBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyCenterBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);
        let center = (GRID_SIZE / 2) as isize;

        Direction::all()
            .filter_map(|direction| {
                let next = my_head.after_moved(direction)?;
                grid.cell_is_empty(next)
                    .then_some((distance_to_center(next, center), direction_priority(direction), direction))
            })
            .min_by_key(|(distance, priority, _)| (*distance, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn distance_to_center(position: GridPosition, center: isize) -> usize {
    (position.x() as isize - center).unsigned_abs()
        + (position.y() as isize - center).unsigned_abs()
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 0,
        Direction::PositiveX => 1,
        Direction::NegativeY => 2,
        Direction::NegativeX => 3,
    }
}