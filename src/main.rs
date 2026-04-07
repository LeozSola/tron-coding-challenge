use crate::engine::prelude::*;

mod engine;
mod players;

fn main() {
    use players::competitive_bot::CompetitiveBot;
    // use players::example_bot::ExampleBot;
    // use players::bot_template::BotTemplate;
    use players::human_controlled_bot::HumanControlledBot;
    let mut game: GameEngine<CompetitiveBot, HumanControlledBot> = GameEngine::new();
    game.run_game();
}
