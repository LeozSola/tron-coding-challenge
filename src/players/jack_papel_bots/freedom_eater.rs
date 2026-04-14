use crate::{engine::prelude::*, players::jack_papel_bots::{JackBot, find_farthest_point, find_farthest_point_in_general, pathfind}};

/// Eats freedom.
pub struct FreedomEater {
    my_player_id: PlayerId,
}

impl Bot for FreedomEater {
    fn new(args: BotArgs) -> Self {
        Self { my_player_id: args.my_player() }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let start = grid.player_head_position(self.my_player_id);

        let farthest_point = find_farthest_point_in_general(start, grid).1;

        pathfind(start, farthest_point, grid)
            .and_then(|path| path.into_iter().nth(1))
            .map(|next_pos| self.direction_to(game_state, next_pos))
            .unwrap_or_else(|| {
                // If we can't find a path to the farthest point, just try to move in any direction that doesn't immediately crash us.
                self.ideal_directions(game_state).next()
                    .or_else(|| self.not_instant_crash_directions(game_state).next())
                    .unwrap_or(Direction::NegativeX)
            })
    }
}

impl JackBot for FreedomEater {
    fn my_player_id(&self) -> PlayerId {
        self.my_player_id
    }
}