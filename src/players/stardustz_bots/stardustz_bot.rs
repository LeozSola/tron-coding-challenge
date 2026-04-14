use crate::{
    engine::prelude::*,
    players::stardustz_bots::{ChaseBot, SimpleSpaceFillBot, a_star::Astar}
};

pub struct StardustzBot{
    my_player_id: PlayerId,
    args: BotArgs,
    inner: InnerBot,
}
enum InnerBot{
    Chase(ChaseBot),
    SpaceFill(SimpleSpaceFillBot)
}

impl Bot for StardustzBot{
    fn new(args: BotArgs)->StardustzBot {
        return StardustzBot{
            my_player_id: args.my_player(),
            args,
            inner: InnerBot::Chase(ChaseBot::new(args))
        }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let split = Self::space_is_split(
            self.my_player_id.get_head_pos(game_state.current_grid()),
            self.my_player_id.other().get_head_pos(game_state.current_grid()),
            game_state.current_grid()
        );

        if split && !matches!(self.inner, InnerBot::SpaceFill(..)){
            self.inner = InnerBot::SpaceFill(SimpleSpaceFillBot::new(self.args));
        }

        match &mut self.inner {
            InnerBot::Chase(chase_bot) => chase_bot.next_action(game_state),
            InnerBot::SpaceFill(simple_space_fill_bot) => simple_space_fill_bot.next_action(game_state),
        }
    }
}

impl StardustzBot{
    /// invariant a != b
    fn space_is_split(a: GridPosition, b: GridPosition, grid: &Grid) -> bool {
        let a_star_result = Astar::a_star_direction(grid, a, b);
        a_star_result.is_none()
    }
}