use crate::engine::prelude::*;

use super::analysis::{connected_component_count, count_empty_cells, distance_map_from_head};
use super::types::GamePhase;

pub fn detect_phase(my_player_id: PlayerId, game_state: &GameState) -> GamePhase {
    let grid = game_state.current_grid();
    let empty_cells = count_empty_cells(grid);
    let empty_components = connected_component_count(grid);

    if empty_cells <= 48 || empty_components >= 4 {
        return GamePhase::Endgame;
    }

    let my_map = distance_map_from_head(grid, grid.player_head_position(my_player_id));
    let opponent_map = distance_map_from_head(grid, grid.player_head_position(my_player_id.other()));

    let shared_space = my_map
        .iter()
        .zip(opponent_map.iter())
        .any(|(mine, theirs)| mine.is_some() && theirs.is_some());

    if shared_space {
        GamePhase::Contact
    } else {
        GamePhase::Split
    }
}
