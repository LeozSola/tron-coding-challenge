use crate::engine::prelude::*;

use super::analysis::{empty_neighbor_count, head_direction, is_wall_hugging, manhattan_distance};
use super::types::OpponentProfile;

pub fn update_opponent_profile(
    profile: &mut OpponentProfile,
    my_player_id: PlayerId,
    game_state: &GameState,
) {
    let opponent_id = my_player_id.other();
    let history: Vec<&Grid> = game_state.grid_history().collect();

    if history.len() <= 1 {
        *profile = OpponentProfile::default();
        return;
    }

    let turns_observed = history.len() - 1;
    let mut wall_hug_turns = 0usize;
    let mut aggression_turns = 0usize;
    let mut corridor_turns = 0usize;
    let mut horizontal_moves = 0usize;
    let mut vertical_moves = 0usize;

    for grid in history {
        let opponent_head = grid.player_head_position(opponent_id);
        let my_head = grid.player_head_position(my_player_id);

        if is_wall_hugging(opponent_head) {
            wall_hug_turns += 1;
        }

        if manhattan_distance(opponent_head, my_head) <= 3 {
            aggression_turns += 1;
        }

        if empty_neighbor_count(grid, opponent_head) <= 2 {
            corridor_turns += 1;
        }

        if let Some(direction) = head_direction(grid, opponent_id) {
            match direction {
                Direction::PositiveX | Direction::NegativeX => horizontal_moves += 1,
                Direction::PositiveY | Direction::NegativeY => vertical_moves += 1,
            }
        }
    }

    let total_samples = (turns_observed + 1) as f32;
    let axis_total = (horizontal_moves + vertical_moves) as f32;
    let horizontal_bias = if axis_total > 0.0 {
        (horizontal_moves as f32 - vertical_moves as f32) / axis_total
    } else {
        0.0
    };

    *profile = OpponentProfile {
        turns_observed,
        wall_hug_ratio: wall_hug_turns as f32 / total_samples,
        aggression_ratio: aggression_turns as f32 / total_samples,
        corridor_ratio: corridor_turns as f32 / total_samples,
        horizontal_bias,
    };
}
