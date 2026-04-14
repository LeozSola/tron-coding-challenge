use crate::{engine::{game_engine::GameSettings, prelude::*}, players::jack_papel_bots::{JackBot, pathfind}};

/// This bot thinks it's playing Snake. It hallucinates that there is a fruit on the board,
/// and does A* pathfinding to try to eat it. It doesn't care about the other player---only
/// where it can and cannot move to get to the fruit.
/// 
/// When the fruit is fully blocked off or is "eaten" (i.e. there is a head or tail occupying
/// its position), it generates a new location for the next fruit.
pub struct Hallucinator {
    my_player_id: PlayerId,
    fruit_location: GridPosition,
}

impl Bot for Hallucinator {
    fn new(args: BotArgs) -> Self {
        Hallucinator {
            my_player_id: args.my_player(),
            fruit_location: Self::generate_initial_fruit_location(),
        }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let mut retries: usize = 0;

        while retries < 50 {
            if 
                let Some(path) = self.find_path_to_fruit(game_state) &&
                let Some(next_pos) = path.into_iter().nth(1)
            {
                return self.direction_to(game_state, next_pos);
            } else {
                // if there is no path to the fruit, try generating a new one again
                retries = retries.checked_add(1).unwrap_or(10);
                self.generate_new_fruit_location(game_state);
                continue;
            }
        }

        // We really couldn't generate a path to the fruit??
        // Damn. We cooked. Better luck next turn.
        if game_state.settings.debug_mode {
            println!("hallucinator: could not find path to fruit after {} retries. Giving up.", retries);
        }

        self.ideal_directions(game_state).next()
            .or_else(|| self.not_instant_crash_directions(game_state).next())
            .unwrap_or(Direction::NegativeX)
    }
}

use rand::prelude::IndexedRandom;

impl Hallucinator {
    fn generate_new_fruit_location(&mut self, game_state: &GameState) {
        let grid = game_state.current_grid();
        let mut empty_positions = Vec::new();
        for pos in GridPosition::iter_positions() {
            if pos.is_empty(grid) {
                empty_positions.push(pos);
            }
        }

        if let Some(fruit_location) = empty_positions.choose(&mut rand::rng()) {
            self.fruit_location = *fruit_location;
        } else {
            // if there are no empty positions, just put the fruit somewhere invalid.
            // the Game is supposed to be over at this point anyway.
            self.fruit_location = GridPosition::new_from_usize(GRID_SIZE * GRID_SIZE).unwrap();
        }
    }

    fn generate_initial_fruit_location() -> GridPosition {
        let empty_positions = GridPosition::iter_positions()
            .filter(|pos| pos.i() != GRID_SIZE * GRID_SIZE / 2 - 1)
            .filter(|pos| pos.i() != GRID_SIZE * GRID_SIZE / 2 + 1)
            .collect::<Vec<_>>();

        if let Some(fruit_location) = empty_positions.choose(&mut rand::rng()) {
            *fruit_location
        } else {
            // if there are no empty positions, just put the fruit somewhere invalid.
            // the Game is supposed to be over at this point anyway.
            GridPosition::new_from_usize(GRID_SIZE * GRID_SIZE / 3).unwrap()
        }
    }

    fn find_path_to_fruit(&self, game_state: &GameState) -> Option<Vec<GridPosition>> {
        let grid = game_state.current_grid();
        let start = grid.player_head_position(self.my_player_id);

        pathfind(start, self.fruit_location, grid)
    }
}

impl JackBot for Hallucinator {
    fn my_player_id(&self) -> PlayerId {
        self.my_player_id
    }
}