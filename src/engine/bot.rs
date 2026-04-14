use std::marker::PhantomData;

use crate::engine::{game_engine::GameSettings, prelude::*};

/// Arguments passed to the bot constructor.
#[derive(Debug, Clone, Copy)]
pub struct BotArgs {
    player: PlayerId,
}

impl BotArgs{
    pub(super) fn new(player: PlayerId) -> Self {
        Self { player }
    }

    /// Get which player you are
    pub fn my_player(&self) -> PlayerId {
        self.player
    }
}

/// A trait for defining bot behavior.
/// 
/// ## Writing your own bot
/// 
/// To write your own bot, implement this trait.
/// 
/// An example implementation can be found in `src/players/example_bot.rs`.
/// A bare-bones template can be found in `src/players/bot_template.rs`.
pub trait Bot: 'static {
    /// The "contructor" for your bot. Add any initialization logic you want here.
    fn new(args: BotArgs) -> Self;

    /// The main logic for your bot. Read the game state and determine which direction to move in.
    fn next_action(&mut self, game_state: &GameState) -> Direction;
}

pub trait BotFactory {
    fn new_bot(&self, args: BotArgs) -> Box<dyn BotActionGenerator>;
}

pub trait BotActionGenerator {
    fn generate_next_action(&mut self, game_state: &GameState) -> Direction;
}

pub struct BuildBot<B: Bot>{
    _marker: PhantomData<B>
}
impl<B: Bot> BuildBot<B> {
    pub fn new_boxed() -> Box<dyn BotFactory> {
        Box::new(Self{_marker: Default::default()})
    }
}

impl<B: Bot> BotFactory for BuildBot<B> {
    fn new_bot(&self, args: BotArgs) -> Box<dyn BotActionGenerator> {
        Box::new(B::new(args))
    }
}

impl<B: Bot> BotActionGenerator for B {
    fn generate_next_action(&mut self, game_state: &GameState) -> Direction {
        self.next_action(game_state)
    }
}