use crate::engine::prelude::*;

use super::analysis::{
    calculate_voronoi_territory, connected_component_count, connected_regions,
    distance_map_from_cell, distance_map_from_head, empty_neighbor_count,
    is_articulation_candidate, local_open_area_score, normalized_relative_offset,
    reachable_area_count,
};
use super::heuristic::HeuristicEvaluator;
use super::safety::MoveSafetyAnalyzer;
use super::types::{HeuristicWeights, LossReason, MoveSafety, OpponentProfile, ScoredMove};
use super::CompetitiveBot;

#[test]
fn baseline_prefers_a_safe_opening_move() {
    let mut bot = CompetitiveBot::new(PlayerId::new_o());
    let state = GameState::new();

    assert_eq!(bot.next_action(&state), Direction::PositiveY);
}

#[test]
fn classifier_flags_opponent_contested_cell_as_risky() {
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let state = GameState::new();

    assert_eq!(
        safety.classify_move(&state, Direction::PositiveX),
        MoveSafety::RiskyHeadToHead
    );
}

#[test]
fn classifier_flags_tail_cell_as_losing() {
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let mut state = GameState::new();

    assert!(state.go_to_next_frame(Direction::PositiveY, Direction::NegativeY));

    assert_eq!(
        safety.classify_move(&state, Direction::NegativeY),
        MoveSafety::Losing(LossReason::OccupiedCell)
    );
}

#[test]
fn classifier_flags_wall_collision_as_losing() {
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let mut state = GameState::new();

    for _ in 0..9 {
        assert!(state.go_to_next_frame(Direction::NegativeX, Direction::PositiveX));
    }

    assert_eq!(
        safety.classify_move(&state, Direction::NegativeX),
        MoveSafety::Losing(LossReason::OutOfBounds)
    );
}

#[test]
fn paired_move_simulation_detects_simultaneous_draw() {
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let grid = GameState::new().current_grid().clone();

    assert!(matches!(
        safety.simulate_paired_move(&grid, Direction::PositiveX, Direction::NegativeX),
        NextFrameResult::Draw
    ));
}

#[test]
fn all_losing_fallback_is_deterministic() {
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let candidates = vec![
        ScoredMove {
            direction: Direction::NegativeY,
            safety: MoveSafety::Losing(LossReason::OutOfBounds),
            score: f32::NEG_INFINITY,
            features: None,
        },
        ScoredMove {
            direction: Direction::PositiveX,
            safety: MoveSafety::Losing(LossReason::OccupiedCell),
            score: f32::NEG_INFINITY,
            features: None,
        },
        ScoredMove {
            direction: Direction::PositiveY,
            safety: MoveSafety::Losing(LossReason::OpponentHeadCell),
            score: f32::NEG_INFINITY,
            features: None,
        },
    ];

    assert_eq!(safety.all_losing_fallback(&candidates), Direction::PositiveY);
}

#[test]
fn heuristic_sorting_remains_deterministic() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );
    let mut candidates = vec![
        ScoredMove {
            direction: Direction::NegativeX,
            safety: MoveSafety::Safe,
            score: 10.0,
            features: None,
        },
        ScoredMove {
            direction: Direction::PositiveY,
            safety: MoveSafety::Safe,
            score: 10.0,
            features: None,
        },
    ];

    evaluator.sort_moves(&mut candidates);
    assert_eq!(candidates[0].direction, Direction::PositiveY);
}

#[test]
fn analysis_reachable_area_matches_opening_space() {
    let state = GameState::new();
    let grid = state.current_grid();
    let start = GridPosition::new(0, 0).expect("in bounds");

    assert_eq!(reachable_area_count(grid, start), (GRID_SIZE * GRID_SIZE) - 2);
}

#[test]
fn analysis_distance_map_starts_adjacent_cells_at_one() {
    let state = GameState::new();
    let grid = state.current_grid();
    let head = grid.player_head_position(PlayerId::new_o());
    let map = distance_map_from_head(grid, head);

    let up = head.after_moved(Direction::PositiveY).expect("in bounds");
    assert_eq!(map[up.i()], Some(1));
}

#[test]
fn analysis_distance_map_from_cell_marks_origin_zero() {
    let state = GameState::new();
    let start = GridPosition::new(0, 0).expect("in bounds");
    let map = distance_map_from_cell(state.current_grid(), start);

    assert_eq!(map[start.i()], Some(0));
}

#[test]
fn analysis_connected_components_detect_single_open_region_initially() {
    let state = GameState::new();
    assert_eq!(connected_component_count(state.current_grid()), 1);
}

#[test]
fn analysis_connected_regions_extract_single_open_region_initially() {
    let state = GameState::new();
    let regions = connected_regions(state.current_grid());

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].len(), (GRID_SIZE * GRID_SIZE) - 2);
}

#[test]
fn analysis_voronoi_counts_include_contested_opening_cells() {
    let state = GameState::new();
    let counts = calculate_voronoi_territory(
        state.current_grid(),
        &[PlayerId::new_o(), PlayerId::new_x()],
    );

    assert_eq!(
        counts.mine + counts.theirs + counts.contested,
        (GRID_SIZE * GRID_SIZE) - 2
    );
    assert!(counts.contested > 0);
}

#[test]
fn analysis_neighbor_count_detects_opening_head_branching() {
    let state = GameState::new();
    let head = state.current_grid().player_head_position(PlayerId::new_o());

    assert_eq!(empty_neighbor_count(state.current_grid(), head), 4);
}

#[test]
fn analysis_local_open_area_score_is_positive_near_opening() {
    let state = GameState::new();
    let head = state.current_grid().player_head_position(PlayerId::new_o());
    let empty_start = head.after_moved(Direction::PositiveY).expect("in bounds");

    assert!(local_open_area_score(state.current_grid(), empty_start, 2) >= 2);
}

#[test]
fn analysis_articulation_scaffold_rejects_occupied_head_cell() {
    let state = GameState::new();
    let head = state.current_grid().player_head_position(PlayerId::new_o());

    assert!(!is_articulation_candidate(state.current_grid(), head));
}

#[test]
fn analysis_normalized_relative_offset_matches_expected_delta() {
    let origin = GridPosition::new(9, 10).expect("in bounds");
    let target = GridPosition::new(11, 12).expect("in bounds");

    assert_eq!(normalized_relative_offset(origin, target), (2, 2));
}
