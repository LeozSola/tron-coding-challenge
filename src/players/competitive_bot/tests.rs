use crate::engine::prelude::*;

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
