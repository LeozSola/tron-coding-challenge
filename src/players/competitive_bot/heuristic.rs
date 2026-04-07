use crate::engine::prelude::*;

use super::analysis::{
    center_preference, empty_neighbor_count, is_narrow_corridor_entry, manhattan_distance,
    reachable_area_count,
};
use super::safety::MoveSafetyAnalyzer;
use super::types::{GamePhase, HeuristicWeights, MoveFeatures, MoveSafety, OpponentProfile, ScoredMove};

pub struct HeuristicEvaluator {
    my_player_id: PlayerId,
    weights: HeuristicWeights,
    opponent_profile: OpponentProfile,
}

impl HeuristicEvaluator {
    pub const fn new(
        my_player_id: PlayerId,
        weights: HeuristicWeights,
        opponent_profile: OpponentProfile,
    ) -> Self {
        Self {
            my_player_id,
            weights,
            opponent_profile,
        }
    }

    pub fn evaluate_moves(
        &self,
        game_state: &GameState,
        phase: GamePhase,
        safety: &MoveSafetyAnalyzer,
    ) -> Vec<ScoredMove> {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        Direction::all()
            .map(|direction| {
                let move_safety = safety.classify_move(game_state, direction);
                let features = my_head
                    .after_moved(direction)
                    .filter(|next_pos| grid.cell_is_empty(*next_pos))
                    .map(|next_pos| self.extract_features(game_state, next_pos, phase));

                let mut score = match (move_safety, features) {
                    (MoveSafety::Losing(_), _) => f32::NEG_INFINITY,
                    (MoveSafety::Safe, Some(features)) => self.score_features(features),
                    (MoveSafety::RiskyHeadToHead, Some(features)) => {
                        self.score_features(features) - self.weights.risky_head_to_head_penalty
                    }
                    (_, None) => f32::NEG_INFINITY,
                };

                if let Some(features) = features {
                    score += self.opponent_profile.horizontal_bias * 0.05;
                    if features.opponent_distance <= 2 {
                        score += self.weights.opponent_pressure_bonus;
                    }
                }

                ScoredMove {
                    direction,
                    safety: move_safety,
                    score,
                    features,
                }
            })
            .collect()
    }

    pub fn sort_moves(&self, candidates: &mut [ScoredMove]) {
        let safety = MoveSafetyAnalyzer::new(self.my_player_id);
        candidates.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| safety.safety_rank(a.safety).cmp(&safety.safety_rank(b.safety)))
                .then_with(|| {
                    safety
                        .direction_priority(a.direction)
                        .cmp(&safety.direction_priority(b.direction))
                })
        });
    }

    fn score_features(&self, features: MoveFeatures) -> f32 {
        let area_weight = match features.phase {
            GamePhase::Contact => self.weights.contact_area_weight,
            GamePhase::Split => self.weights.split_area_weight,
            GamePhase::Endgame => self.weights.endgame_area_weight,
        };

        let branching_weight = match features.phase {
            GamePhase::Contact => self.weights.contact_branching_weight,
            GamePhase::Split => self.weights.split_branching_weight,
            GamePhase::Endgame => self.weights.endgame_branching_weight,
        };

        let mut score = 0.0;
        score += features.reachable_area as f32 * area_weight;
        score += features.branching_factor as f32 * branching_weight;
        score += features.center_preference * self.weights.center_preference_weight;
        score -= features.opponent_distance as f32 * self.weights.opponent_pressure_bonus;

        if features.narrow_corridor {
            score -= self.weights.narrow_corridor_penalty;
        }

        score
    }

    fn extract_features(
        &self,
        game_state: &GameState,
        candidate_position: GridPosition,
        phase: GamePhase,
    ) -> MoveFeatures {
        let grid = game_state.current_grid();
        let opponent_head = grid.player_head_position(self.my_player_id.other());

        MoveFeatures {
            reachable_area: reachable_area_count(grid, candidate_position),
            branching_factor: empty_neighbor_count(grid, candidate_position),
            center_preference: center_preference(candidate_position),
            opponent_distance: manhattan_distance(candidate_position, opponent_head),
            narrow_corridor: is_narrow_corridor_entry(grid, candidate_position),
            phase,
        }
    }
}
