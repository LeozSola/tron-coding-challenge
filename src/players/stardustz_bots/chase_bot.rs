use crate::{engine::prelude::*, players::stardustz_bots::{a_star::Astar, helper::DirectionIterator}};

pub struct ChaseBot{
    my_player_id: PlayerId,
}

impl Bot for ChaseBot{
    fn new(args: BotArgs)->Self {
        ChaseBot{my_player_id: args.my_player()}
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_pos = self.my_player_id.get_head_pos(grid);
        let enemy_pos = self.my_player_id.other().get_head_pos(grid);


        let agro = Astar::a_star_direction(grid, my_pos, enemy_pos);
        let safest = ||Direction::all().filter_not_crash_into_head(self.my_player_id, grid);
        let last_resort = ||Direction::all().filter_not_crash(self.my_player_id, grid);

        if safest().any(|d|agro.is_some_and(|agro|agro==d)){
            agro.expect("just checked on previous line")
        }else if let Some(safest) = safest().next(){
            safest
        }else if let Some(last_resort) = last_resort().next() {
            last_resort
        }else{
            Direction::up()
        }
    }
}
