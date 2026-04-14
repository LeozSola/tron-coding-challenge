use crate::engine::prelude::*;

/// A bot that is controlled by a human through standard input. This is useful
/// for testing your bot.
pub struct HumanControlledBot;

impl Bot for HumanControlledBot {
    fn new(_: BotArgs) -> Self {
        HumanControlledBot
    }

    fn next_action(&mut self, _game_state: &GameState) -> Direction {
        use std::io::stdin;
        let mut s = String::new();
        let _ = stdin().read_line(&mut s);
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }

        match s.as_str() {
            "w" => Direction::PositiveY,
            "d" => Direction::PositiveX,
            "s" => Direction::NegativeY,
            "a" => Direction::NegativeX,
            _ => Direction::PositiveX,
        }
    }
}
