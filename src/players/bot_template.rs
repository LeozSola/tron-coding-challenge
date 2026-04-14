//! A template bot implementation. 
//! 
//! To write your own bot, copy this file's contents and modify it with your 
//! bot's logic for the competition. 

use crate::engine::prelude::*;

/// A template bot implementation.
pub struct BotTemplate {
    my_player_id: PlayerId,
}

impl Bot for BotTemplate {
    fn new(args: BotArgs) -> Self {
        BotTemplate { my_player_id: args.my_player() }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        // Put your bot's logic here!
        
        Direction::PositiveX
    }
}
