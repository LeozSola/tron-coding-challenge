use crate::engine::prelude::*;

pub struct StrategyCornerBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyCornerBot {
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
                    .then_some((distance_to_nearest_corner(next), direction_priority(direction), direction))
            })
            .min_by_key(|(distance, priority, _)| (*distance, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn distance_to_nearest_corner(position: GridPosition) -> usize {
    let max = GRID_SIZE - 1;
    let corners = [(0, 0), (0, max), (max, 0), (max, max)];

    corners
        .into_iter()
        .map(|(x, y)| position.x().abs_diff(x) + position.y().abs_diff(y))
        .min()
        .unwrap_or(usize::MAX)
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 0,
        Direction::PositiveX => 1,
        Direction::NegativeY => 2,
        Direction::NegativeX => 3,
    }
}