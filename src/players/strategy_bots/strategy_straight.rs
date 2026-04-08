use crate::engine::prelude::*;

pub struct StrategyStraightBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyStraightBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let head = grid.player_head_position(self.my_player_id);

        if let Some(direction) = current_heading(grid, self.my_player_id)
            .filter(|direction| head.after_moved(*direction).is_some_and(|next| grid.cell_is_empty(next)))
        {
            return direction;
        }

        Direction::all()
            .find(|direction| head.after_moved(*direction).is_some_and(|next| grid.cell_is_empty(next)))
            .unwrap_or(Direction::NegativeX)
    }
}

fn current_heading(grid: &Grid, player_id: PlayerId) -> Option<Direction> {
    let head = grid.player_head_position(player_id);

    match grid.get_cell(head) {
        GridCell::Head(_, direction) => Some(*direction),
        _ => None,
    }
}