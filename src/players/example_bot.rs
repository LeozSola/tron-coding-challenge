use crate::engine::prelude::*;

/// An example bot implementation.
/// 
/// This bot tries to move in an ideal direction
/// (a direction that doesn't result in immediately losing and also doesn't
/// border the opponent's head), and if one is unavailable, it settles for a
/// direction that just doesn't immediately lose, and if that's also
/// unavailable, it just moves left.
pub struct ExampleBot {
    my_player_id: PlayerId,
}

impl Bot for ExampleBot {
    fn new(args: BotArgs) -> Self {
        ExampleBot { my_player_id: args.my_player() }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        if let Some(out) = self.ideal_directions(game_state).next() {
            return out;
        }
        
        if let Some(out) = self.not_instant_crash_directions(game_state).next() {
            return out;
        }

        Direction::NegativeX
    }
}
impl ExampleBot {
    fn not_instant_crash_directions(
        &self,
        game_state: &GameState,
    ) -> impl Iterator<Item = Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id);

        Direction::all().filter(move |d| {
            my_pos
                .after_moved(*d)
                .filter(|p| p.is_empty(grid))
                .is_some()
        })
    }
    fn ideal_directions(&self, game_state: &GameState) -> impl Iterator<Item = Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id);

        self.not_instant_crash_directions(game_state)
            .filter(move |d| {
                my_pos.after_moved(*d).is_some_and(|p| {
                    !p.borders_cell(grid, |cell| cell.is_players_head(self.my_player_id.other()))
                })
            })
    }
}
