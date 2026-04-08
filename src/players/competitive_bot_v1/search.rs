use crate::engine::prelude::*;

use super::types::{GamePhase, OpponentProfile, ScoredMove};

pub fn search_best_move(
    _my_player_id: PlayerId,
    opponent_profile: OpponentProfile,
    _game_state: &GameState,
    _phase: GamePhase,
    candidates: &[ScoredMove],
) -> Option<Direction> {
    let _strong_candidates = candidates.iter().filter(|candidate| !candidate.safety.is_safe() || candidate.features.is_some()).count();
    let _profile_signal = opponent_profile.turns_observed as f32
        + opponent_profile.wall_hug_ratio
        + opponent_profile.aggression_ratio
        + opponent_profile.corridor_ratio;

    None
}
