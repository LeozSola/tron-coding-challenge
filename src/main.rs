#![allow(clippy::large_enum_variant, reason = "It's not that serious")]

use regex::Regex;

use crate::competition::CompetitionSettings;
use crate::engine::game_engine::GameSettings;
use crate::players::example_bot::ExampleBot;
use crate::{engine::prelude::*, players::human_controlled_bot::HumanControlledBot};

use crate::players::*;
use competition::{Competition, CompetitionPlayer};

mod engine;
mod players;
mod competition;

/// Switch this to your desired mode for testing!
const MODE: Mode = Mode::Test;
/// If your bot is deterministic, you can set this in order to test different starting positions.
pub const RANDOM_SPAWNS: bool = false;
/// The size of the grid. The grid is a square, so this is both the width and
/// height of the grid. The grid has GRID_SIZE * GRID_SIZE cells in total.
pub const GRID_SIZE: usize = 21;

// Set these to your desired bots for testing!
/// The bot controlling player "O"
type OBot = HumanControlledBot;
/// The bot controlling player "X"
type XBot = ExampleBot;

fn main() {
    match MODE {
        Mode::Test => {
            run_test_game_print::<OBot, XBot>();
        },
        Mode::TestMany => {
            loop {
                run_test_game_print::<OBot, XBot>();
            }
        },
        Mode::TestUntilFailure | Mode::TestUntilFailureOrDraw => {
            let mut games_in_a_row = 0;
            loop {
                let game_over = run_test_game_print::<OBot, XBot>();
                if 
                    let Mode::TestUntilFailureOrDraw = MODE &&
                    let GameOver::Draw = game_over
                {
                    break;
                }
                if let GameOver::Winner { player_who_won: PlayerId::X } = game_over {
                    break;
                }
                games_in_a_row += 1;
            }
            println!("O won {} games in a row!", games_in_a_row);
        },
        Mode::Sample { simulations } => sample_games::<OBot, XBot>(simulations),
        Mode::Competition => {
            let competition = Competition::new(CompetitionSettings {
                random_spawns: RANDOM_SPAWNS
            });
            
            competition.run_and_print(vec![
                CompetitionPlayer::new_player::<example_bot::ExampleBot>(),
                CompetitionPlayer::new_player::<bot_template::BotTemplate>(),
                CompetitionPlayer::new_player::<stardustz_bots::StardustzBot>(),
                CompetitionPlayer::new_player::<jack_papel_bots::hallucinator::Hallucinator>(),
                CompetitionPlayer::new_player::<jack_papel_bots::rip_and_tear::RipAndTear>(),
                CompetitionPlayer::new_player::<jack_papel_bots::freedom_eater::FreedomEater>(),
                // Add your bot here!
            ])
        },
    }
}

/// The mode to run the game in.
#[allow(unused)]
enum Mode {
    /// Run a single game with debug output. Useful for iterating on your bot.
    Test,
    /// Run games with debug output until O bot loses.
    TestUntilFailure,
    /// Run games with debug output until O bot loses or the game draws.
    TestUntilFailureOrDraw,
    /// Run games with debug output.
    TestMany,
    /// Run 100 games and print the results. Useful for testing how well your bot does against another bot on average.
    Sample {
        simulations: usize
    },
    /// Run a full competition between every bot. Useful for seeing how your bot does compared to all other bots.
    Competition,
}

pub fn get_bot_name<B: Bot>() -> String {
    let regex = Regex::new(r"([a-zA-Z0-9_]*::)*").unwrap();
    regex.replace(std::any::type_name::<B>(), "").to_string()
}

fn sample_games<O: Bot, X: Bot>(simulations: usize) {
    let mut o_games = 0;
    let mut draw_games = 0;
    let mut x_games = 0;

    let o_name = get_bot_name::<O>();
    let x_name = get_bot_name::<X>();

    println!("Simulating {} games between {} and {}...", simulations, o_name, x_name);

    for i in 0..simulations {
        
        match run_test_game::<O, X>() {
            GameOver::Winner { player_who_won: PlayerId::O } => {
                println!("Round {}: {}", i + 1, o_name);
                o_games += 1;
            },
            GameOver::Winner { player_who_won: PlayerId::X } => {
                println!("Round {}: {}", i + 1, x_name);
                x_games += 1;
            },
            GameOver::Draw => {
                println!("Round {}: Draw", i + 1);
                draw_games += 1;
            },
        }
    }

    let total_games = o_games + draw_games + x_games;

    println!("\nRan {} simulations: {}\n", simulations, total_games);
    println!("{}: {} ({:.2}%)", o_name, o_games, o_games as f64 / total_games as f64 * 100.0);
    println!("{}: {} ({:.2}%)", x_name, x_games, x_games as f64 / total_games as f64 * 100.0);
    println!("Draw: {} ({:.2}%)", draw_games, draw_games as f64 / total_games as f64 * 100.0);
}

fn run_test_game<O: Bot, X: Bot>() -> GameOver {
    GameEngine::new(
        BuildBot::<O>::new_boxed().as_ref(),
        BuildBot::<X>::new_boxed().as_ref(),
        GameSettings {
            debug_mode: false,
            random_spawns: RANDOM_SPAWNS,
        }
    ).run_game()
}

fn run_test_game_print<O: Bot, X: Bot>() -> GameOver {
    GameEngine::new(
        BuildBot::<O>::new_boxed().as_ref(),
        BuildBot::<X>::new_boxed().as_ref(),
        GameSettings {
            debug_mode: true,
            random_spawns: RANDOM_SPAWNS,
        }
    ).run_game_print()
}