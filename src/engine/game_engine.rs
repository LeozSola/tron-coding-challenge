use crate::engine::prelude::*;

pub struct GameEngine {
    game_state: GameState,
    o: Box<dyn BotActionGenerator>,
    x: Box<dyn BotActionGenerator>,
}

/// Settings for the game engine. Passed in when constructing a new GameEngine.
/// 
/// Your bot should only leave debug print statements when `debug_mode` is true.
/// You can check this by calling `game_state.settings.debug_mode`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameSettings {
    pub debug_mode: bool,
    pub random_spawns: bool,
}

impl GameEngine {
    pub fn new(o: &dyn BotFactory, x: &dyn BotFactory, settings: GameSettings) -> Self {
        Self {
            game_state: GameState::new(settings),
            o: o.new_bot(BotArgs::new(PlayerId::O)),
            x: x.new_bot(BotArgs::new(PlayerId::X)),
        }
    }
}
impl GameEngine {
    /// returns true if game not over
    pub fn go_to_next_frame(&mut self) -> NextFrameResult {
        let a_action = std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(||self.o.generate_next_action(&self.game_state))
        );
        let b_action = std::panic::catch_unwind(
            std::panic::AssertUnwindSafe(||self.x.generate_next_action(&self.game_state))
        );

        match (a_action.is_err(), b_action.is_err()){
            (true, true) => NextFrameResult::Draw,
            (true, false) => NextFrameResult::Winner { player_who_won: PlayerId::X },
            (false, true) => NextFrameResult::Winner { player_who_won: PlayerId::O },
            (false, false) => {
                let Ok(a_action) = a_action else {unreachable!()};
                let Ok(b_action) = b_action else {unreachable!()};
                self.game_state.go_to_next_frame(
                    a_action,
                    b_action,
                )
            },
        }
    }

    pub fn print_current_game_state(&self){
        println!("{}", self.game_state)
    }
    pub fn run_game_print(&mut self) -> GameOver {
        loop{
            self.print_current_game_state();

            if let Some(out) = self.go_to_next_frame().game_over() {
                self.print_current_game_state();
                println!("{}", out);
                return out;
            }
        }
    }
    pub fn run_game(&mut self) -> GameOver {
        loop{
            if let Some(out) = self.go_to_next_frame().game_over() {
                return out;
            }
        }
    }
}

