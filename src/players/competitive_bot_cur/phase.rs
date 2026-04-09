use crate::engine::prelude::*;

use super::analysis::{
    calculate_voronoi_territory, connected_component_count, count_empty_cells,
    distance_map_from_head, empty_neighbor_count, local_open_area_score, manhattan_distance,
    neighbors, reachable_area_count, reachable_region_fragmentation,
};
use super::types::{GamePhase, PhaseProfile, PhaseScores};

pub fn detect_phase(my_player_id: PlayerId, game_state: &GameState) -> GamePhase {
    detect_phase_profile(my_player_id, game_state).phase
}

pub(crate) fn detect_phase_profile(
    my_player_id: PlayerId,
    game_state: &GameState,
) -> PhaseProfile {
    classify_phase_profile(my_player_id, game_state.current_grid())
}

pub(crate) fn classify_phase(my_player_id: PlayerId, grid: &Grid) -> GamePhase {
    classify_phase_profile(my_player_id, grid).phase
}

pub(crate) fn classify_phase_profile(my_player_id: PlayerId, grid: &Grid) -> PhaseProfile {
    let mut profile = analyze_phase_profile(my_player_id, grid);
    profile.scores = phase_scores(&profile);
    profile.phase = if is_forced_endgame(&profile) {
        GamePhase::Endgame
    } else {
        best_phase(profile.scores)
    };
    profile
}

fn best_phase(scores: PhaseScores) -> GamePhase {
    if scores.endgame >= scores.contact && scores.endgame >= scores.split {
        GamePhase::Endgame
    } else if scores.contact >= scores.split {
        GamePhase::Contact
    } else {
        GamePhase::Split
    }
}

fn analyze_phase_profile(my_player_id: PlayerId, grid: &Grid) -> PhaseProfile {
    let my_head = grid.player_head_position(my_player_id);
    let opponent_head = grid.player_head_position(my_player_id.other());
    let empty_cells = count_empty_cells(grid);
    let empty_components = connected_component_count(grid);
    let my_map = distance_map_from_head(grid, my_head);
    let opponent_map = distance_map_from_head(grid, opponent_head);
    let my_reachable = my_map.iter().flatten().count();
    let opponent_reachable = opponent_map.iter().flatten().count();
    let shared_reachable = my_map
        .iter()
        .zip(opponent_map.iter())
        .filter(|(mine, theirs)| mine.is_some() && theirs.is_some())
        .count();
    let voronoi = calculate_voronoi_territory(grid, &[my_player_id, my_player_id.other()]);
    let contested_ratio = if empty_cells == 0 {
        0.0
    } else {
        voronoi.contested as f32 / empty_cells as f32
    };

    PhaseProfile {
        phase: GamePhase::Contact,
        scores: PhaseScores::default(),
        empty_cells,
        empty_components,
        my_reachable,
        opponent_reachable,
        shared_reachable,
        contested_cells: voronoi.contested,
        contested_ratio,
        head_distance: manhattan_distance(my_head, opponent_head),
        my_local_open_area: head_local_open_area(grid, my_head),
        opponent_local_open_area: head_local_open_area(grid, opponent_head),
        my_branching: empty_neighbor_count(grid, my_head),
        opponent_branching: empty_neighbor_count(grid, opponent_head),
        my_projected_space: best_projected_space(grid, my_head),
        opponent_projected_space: best_projected_space(grid, opponent_head),
        my_region_fragmentation: best_region_fragmentation(grid, my_head),
        opponent_region_fragmentation: best_region_fragmentation(grid, opponent_head),
        my_corridor_severity: head_corridor_severity(grid, my_head),
        opponent_corridor_severity: head_corridor_severity(grid, opponent_head),
    }
}

fn head_local_open_area(grid: &Grid, head: GridPosition) -> usize {
    Direction::all()
        .filter_map(|direction| head.after_moved(direction))
        .filter(|position| grid.cell_is_empty(*position))
        .map(|position| local_open_area_score(grid, position, 2))
        .max()
        .unwrap_or(0)
}

fn best_projected_space(grid: &Grid, head: GridPosition) -> usize {
    Direction::all()
        .filter_map(|direction| head.after_moved(direction))
        .filter(|position| grid.cell_is_empty(*position))
        .map(|position| reachable_area_count(grid, position))
        .max()
        .unwrap_or(0)
}

fn best_region_fragmentation(grid: &Grid, head: GridPosition) -> usize {
    Direction::all()
        .filter_map(|direction| head.after_moved(direction))
        .filter(|position| grid.cell_is_empty(*position))
        .map(|position| reachable_region_fragmentation(grid, position))
        .min()
        .unwrap_or(0)
}

fn head_corridor_severity(grid: &Grid, head: GridPosition) -> f32 {
    let mut best = f32::INFINITY;

    for position in neighbors(head)
        .into_iter()
        .flatten()
        .filter(|position| grid.cell_is_empty(*position))
    {
        let branching = empty_neighbor_count(grid, position) as f32;
        let local_open = local_open_area_score(grid, position, 2) as f32;
        let severity = (1.0 - (branching / 4.0)).clamp(0.0, 1.0) * 0.6
            + (1.0 - (local_open / 8.0)).clamp(0.0, 1.0) * 0.4;
        best = best.min(severity);
    }

    if best.is_infinite() { 1.0 } else { best }
}

fn phase_scores(profile: &PhaseProfile) -> PhaseScores {
    let constrained_projected_space = profile.my_projected_space.min(profile.opponent_projected_space);
    let max_fragmentation = profile
        .my_region_fragmentation
        .max(profile.opponent_region_fragmentation);
    let max_corridor_severity = profile
        .my_corridor_severity
        .max(profile.opponent_corridor_severity)
        .clamp(0.0, 1.0);

    let space_pressure = if profile.empty_cells <= 48 {
        1.0
    } else if profile.empty_cells <= 72 {
        0.7
    } else {
        0.0
    };
    let fragmentation_pressure = ((profile.empty_components.saturating_sub(1)) as f32 / 4.0).clamp(0.0, 1.0);
    let projected_pressure = if constrained_projected_space <= 18 {
        1.0
    } else if constrained_projected_space <= 28 {
        0.55
    } else {
        0.0
    };
    let region_pressure = (max_fragmentation.saturating_sub(1) as f32 / 3.0).clamp(0.0, 1.0);
    let low_shared_space = if profile.shared_reachable <= 10 { 1.0 } else { 0.0 };
    let cramped_local_space = if profile.my_local_open_area <= 5 || profile.opponent_local_open_area <= 5 {
        1.0
    } else {
        0.0
    };
    let low_branching = if profile.my_branching <= 2 && profile.opponent_branching <= 2 {
        1.0
    } else {
        0.0
    };
    let forced_endgame_pressure = if profile.empty_cells <= 48
        || profile.empty_components >= 4
        || (constrained_projected_space <= 18
            && (profile.shared_reachable <= 10 || max_corridor_severity >= 0.55))
        || (constrained_projected_space <= 28
            && max_fragmentation >= 2
            && max_corridor_severity >= 0.45
            && profile.head_distance >= 4)
    {
        1.0
    } else {
        0.0
    };

    let endgame = (space_pressure * 0.22
        + fragmentation_pressure * 0.16
        + projected_pressure * 0.18
        + region_pressure * 0.14
        + max_corridor_severity * 0.12
        + low_shared_space * 0.05
        + cramped_local_space * 0.02
        + low_branching * 0.01
        + forced_endgame_pressure * 0.20)
        .clamp(0.0, 1.0);

    let contact = if profile.shared_reachable == 0 {
        0.0
    } else {
        let head_proximity = if profile.head_distance <= 4 {
            1.0
        } else if profile.head_distance <= 7 {
            0.6
        } else {
            0.2
        };
        let shared_space = (profile.shared_reachable as f32 / 24.0).clamp(0.0, 1.0);
        let contested = ((profile.contested_cells as f32 / 16.0).clamp(0.0, 1.0) * 0.5)
            + ((profile.contested_ratio.clamp(0.0, 0.20) / 0.20) * 0.5);
        let room = if profile.my_local_open_area >= 8 && profile.opponent_local_open_area >= 8 {
            1.0
        } else {
            0.5
        };

        (head_proximity * 0.35 + shared_space * 0.30 + contested * 0.25 + room * 0.10).clamp(0.0, 1.0)
    };

    let split =
        (1.0 - contact * 0.75 - endgame * 0.55 + if profile.shared_reachable == 0 { 0.45 } else { 0.0 })
            .clamp(0.0, 1.0);

    PhaseScores {
        contact,
        split,
        endgame,
    }
}

fn is_forced_endgame(profile: &PhaseProfile) -> bool {
    let constrained_projected_space = profile.my_projected_space.min(profile.opponent_projected_space);
    let max_fragmentation = profile
        .my_region_fragmentation
        .max(profile.opponent_region_fragmentation);
    let max_corridor_severity = profile
        .my_corridor_severity
        .max(profile.opponent_corridor_severity);

    if profile.empty_cells <= 48 {
        return true;
    }

    if profile.empty_components >= 4 {
        return true;
    }

    if constrained_projected_space <= 18
        && (profile.shared_reachable <= 10 || max_corridor_severity >= 0.55)
    {
        return true;
    }

    constrained_projected_space <= 28
        && max_fragmentation >= 2
        && max_corridor_severity >= 0.45
        && profile.head_distance >= 4
}
