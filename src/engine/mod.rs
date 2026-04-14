#![allow(unused)]

use crate::engine::prelude::*;
use std::fmt::Display;

pub mod prelude;

pub mod bot;
pub mod direction;
pub mod game_engine;
pub mod game_state;
pub mod grid;
pub mod grid_cell;
pub mod grid_position;
pub mod player_id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameOver {
    Winner { player_who_won: PlayerId },
    Draw,
}
impl Display for GameOver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameOver::Winner { player_who_won } =>
                    format!("Game over: Player {} won", player_who_won),
                GameOver::Draw => "Game over: Draw".to_owned(),
            }
        )
    }
}
impl From<GameOver> for NextFrameResult {
    fn from(value: GameOver) -> Self {
        match value {
            GameOver::Winner { player_who_won } => NextFrameResult::Winner { player_who_won },
            GameOver::Draw => NextFrameResult::Draw,
        }
    }
}
impl TryFrom<&NextFrameResult> for GameOver {
    type Error = &'static str;
    
    fn try_from(value: &NextFrameResult) -> Result<Self, Self::Error> {
        match &value {
            NextFrameResult::Winner { player_who_won } => Ok(GameOver::Winner { player_who_won: *player_who_won }),
            NextFrameResult::Draw => Ok(GameOver::Draw),
            NextFrameResult::NextFrame(_) => Err("next frame cant be cast into game over")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextFrameResult {
    NextFrame(Grid),
    Winner { player_who_won: PlayerId },
    Draw,
}
impl NextFrameResult {
    fn game_over(&self) -> Option<GameOver> {
        self.try_into().ok()
    }
}