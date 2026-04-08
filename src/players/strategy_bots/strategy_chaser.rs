use crate::engine::prelude::*;

pub struct StrategyChaserBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyChaserBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);
        let opponent_id = opposing_player(self.my_player_id);
        let opponent_head = grid.player_head_position(opponent_id);

        Direction::all()
            .filter_map(|direction| {
                let next = my_head.after_moved(direction)?;
                grid.cell_is_empty(next).then_some((
                    manhattan_distance(next, opponent_head),
                    direction_priority(direction),
                    direction,
                ))
            })
            .min_by_key(|(distance, priority, _)| (*distance, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn opposing_player(player_id: PlayerId) -> PlayerId {
    if player_id == PlayerId::new_o() {
        PlayerId::new_x()
    } else {
        PlayerId::new_o()
    }
}

fn manhattan_distance(a: GridPosition, b: GridPosition) -> usize {
    a.x().abs_diff(b.x()) + a.y().abs_diff(b.y())
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 0,
        Direction::PositiveX => 1,
        Direction::NegativeY => 2,
        Direction::NegativeX => 3,
    }
}