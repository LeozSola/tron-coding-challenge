use crate::engine::prelude::*;

pub struct StrategySafeBot {
    my_player_id: PlayerId,
}

impl Bot for StrategySafeBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        Direction::all()
            .find(|direction| {
                my_head
                    .after_moved(*direction)
                    .is_some_and(|position| grid.cell_is_empty(position))
            })
            .unwrap_or(Direction::NegativeX)
    }
}