use std::fmt::Display;

use crate::engine::{game_engine::GameSettings, prelude::*};

/// The current state of the game.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    grid_history: Vec<Grid>,
    game_over: Option<GameOver>,
    /// Settings for the current game.
    pub settings: GameSettings,
}
impl GameState {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            grid_history: Vec::from([Grid::new_default(settings.random_spawns)]),
            game_over: None,
            settings,
        }
    }
    /// Returns the current grid.
    pub fn current_grid(&self) -> &Grid {
        self.grid_history
            .last()
            .expect("game state must have at least 1 grid")
    }
    /// Returns the current frame index (that corresponds with the current grid).
    /// The frame index starts at 0 on the first frame and increases by 1 with each new frame.
    pub fn current_time(&self) -> usize {
        self.grid_history
            .len()
            .checked_sub(1)
            .expect("game state must have at least 1 grid")
    }
    /// Returns the grid at a given frame index, or None if the frame hasn't happened yet.
    pub fn grid(&self, frame_index: usize) -> Option<&Grid> {
        self.grid_history.get(frame_index)
    }
    /// Returns an iterator of all grids up to and including the current frame.
    pub fn grid_history(&self) -> impl Iterator<Item = &Grid> {
        self.grid_history.iter()
    }
    pub fn is_game_over(&self) -> bool {
        self.game_over.is_some()
    }
    pub fn go_to_next_frame(
        &mut self,
        player_a_choice: Direction,
        player_b_choice: Direction,
    ) -> NextFrameResult {
        if let Some(game_over) = self.game_over {
            return game_over.into()
        }

        let next_frame_result = self
            .current_grid()
            .next_grid(
                player_a_choice,
                player_b_choice,
                self.current_time() + 1
            );
        match &next_frame_result {
            NextFrameResult::NextFrame(grid) => {
                self.grid_history.push(grid.clone());
            }
            NextFrameResult::Winner { player_who_won } => {
                self.game_over = Some(GameOver::Winner { player_who_won: *player_who_won });
            }
            NextFrameResult::Draw => {
                self.game_over = Some(GameOver::Draw);
            }
        }
        next_frame_result
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.game_over {
            Some(game_over) => writeln!(f, "{}", game_over),
            None => writeln!(f, "{}", self.current_grid()),
        }
    }
}
