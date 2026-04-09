use crate::engine::prelude::*;

mod analysis;
mod heuristic;
mod opponent;
mod phase;
mod safety;
mod search;
#[cfg(test)]
mod tests;
mod types;

use heuristic::HeuristicEvaluator;
use opponent::update_opponent_profile;
use phase::{detect_phase, detect_phase_profile};
use safety::MoveSafetyAnalyzer;
use search::search_best_move;
use types::{HeuristicWeights, OpponentProfile};

/// A competition-oriented bot scaffold split into pipeline layers.
pub struct CompetitiveBot {
    my_player_id: PlayerId,
    opponent_profile: OpponentProfile,
    weights: HeuristicWeights,
}

impl Bot for CompetitiveBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self {
            my_player_id,
            opponent_profile: OpponentProfile::default(),
            weights: HeuristicWeights::default(),
        }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        update_opponent_profile(&mut self.opponent_profile, self.my_player_id, game_state);

        let phase_profile = detect_phase_profile(self.my_player_id, game_state);
        let phase = phase_profile.phase;
        let safety = MoveSafetyAnalyzer::new(self.my_player_id);
        let evaluator = HeuristicEvaluator::new(self.my_player_id, self.weights, self.opponent_profile);

        let mut candidates = evaluator.evaluate_moves(game_state, phase_profile, &safety);
        evaluator.sort_moves(&mut candidates);

        if let Some(search_move) = search_best_move(
            self.my_player_id,
            self.opponent_profile,
            game_state,
            phase,
            &candidates,
        ) {
            return search_move;
        }

        if let Some(best_safe) = candidates.iter().copied().find(|c| c.safety.is_safe()) {
            return best_safe.direction;
        }

        if let Some(best_risky) = candidates.iter().copied().find(|c| c.safety.is_risky()) {
            return best_risky.direction;
        }

        safety.all_losing_fallback(&candidates)
    }
}
