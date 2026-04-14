use crate::{GRID_SIZE, engine::{game_engine::GameSettings, prelude::*}, get_bot_name};

const REPEATS: u16 = 3;

pub struct Competition(CompetitionSettings);

impl Competition{
    pub fn new(settings: CompetitionSettings) -> Self {
        Self(settings)
    }

    pub fn run_and_print(
        &self,
        mut players: Vec<CompetitionPlayer>
    ) {
        println!("Running the competition with {} players...\n", players.len());
        println!("Currently running:\n");

        // Round-robin tournament.
        // This 2d for-loop already accounts for making each player in a pair play as both O and X.
        for i in 0..players.len(){
            for j in 0..players.len() {
                let Some([a, b]) = players.get_disjoint_mut([i, j]).ok() else {continue};

                // Clear previous line.
                clear_terminal_lines(1);
                println!("{} vs {}", a.name, b.name);

                std::panic::set_hook(Box::new(|_| {}));

                self.run_competition_round(a, b);
            }
        }

        players.sort_by(|a,b|b.points().total_cmp(&a.points()));

        clear_terminal_lines(4);
        println!("Competition results:\n");

        for player in players {
            println!("{}: {:.2} points.", player.name, player.points());
            println!("  - W / L / D : {} / {} / {}", player.wins, player.loses, player.draws);
        }
    }

    fn run_competition_round(
        &self,
        a: &mut CompetitionPlayer,
        b: &mut CompetitionPlayer
    ){
        for _ in 0..REPEATS {
            self.run_one_competition_game_add_points(a, b)
        } 
        // No need to run with the positions swapped --- the 2D for-loop will do that for us.
    }

    fn run_one_competition_game_add_points(
        &self, 
        o: &mut CompetitionPlayer,
        x: &mut CompetitionPlayer
    ) {
        let settings = GameSettings { debug_mode: false, random_spawns: self.0.random_spawns };
        match GameEngine::new(o.bot_factory.as_ref(), x.bot_factory.as_ref(), settings).run_game() {
            GameOver::Winner { player_who_won: PlayerId::O } => {
                o.wins += 1;
                x.loses += 1;
            },
            GameOver::Winner { player_who_won: PlayerId::X } => {
                o.loses += 1;
                x.wins += 1;
            },
            GameOver::Draw => {
                o.draws += 1;
                x.draws += 1;
            },
        }
    }
}

// Users using CMD might see gunk in the terminal...
// but I don't want to add a dependency just to clear the terminal.
// If you're reading this, use powershell (or linux!).
fn clear_terminal_lines(num_lines: usize) {
    for _ in 0..num_lines {
        print!("\x1B[1A\x1B[2K");
    }
}

pub struct CompetitionSettings {
    pub random_spawns: bool,
}

pub struct CompetitionPlayer{
    name: String,
    bot_factory: Box<dyn BotFactory>,
    wins: u16,
    loses: u16,
    draws: u16
}
impl CompetitionPlayer{
    pub fn new_player<B: Bot + 'static>() -> Self {
        Self {
            name: get_bot_name::<B>(),
            bot_factory: BuildBot::<B>::new_boxed(),
            wins: 0,
            loses: 0,
            draws: 0,
        }
    }
    pub fn points(&self) -> f32 {
        self.wins as f32  - self.loses as f32 - (self.draws as f32 * 0.5f32)
    }
}