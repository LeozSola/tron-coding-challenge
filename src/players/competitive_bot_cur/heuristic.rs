use crate::engine::prelude::*;

use super::analysis::{
    calculate_voronoi_territory, center_preference, distance_map_from_head, edge_escape_routes,
    empty_neighbor_count, is_articulation_candidate, is_narrow_corridor_entry,
    is_semi_split_pressure, is_wall_hugging, largest_reachable_region_ratio,
    local_open_area_score, manhattan_distance, reachable_area_count, reachable_region_fragmentation,
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
                    .map(|next_pos| self.extract_features(game_state, direction, next_pos, phase));

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

    pub(crate) fn score_features(&self, features: MoveFeatures) -> f32 {
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

        let pressure_window = 4usize.saturating_sub(features.opponent_distance) as f32;
        let center_weight = self.contextual_center_weight(features);
        let mut score = 0.0;
        score += features.projected_reachable_area as f32 * area_weight;
        score += features.reachable_area as f32 * self.weights.reachable_area_weight;
        score += features.branching_factor as f32 * branching_weight;
        score += features.local_open_area as f32 * self.weights.local_open_area_weight;
        score += features.center_preference * center_weight;
        score += features.territory_balance as f32 * self.weights.territory_balance_weight;
        score += features.voronoi_contested as f32 * self.weights.contested_territory_weight;
        score += features.cut_potential as f32 * self.weights.cut_opponent_bonus_weight;
        score += pressure_window * self.weights.opponent_pressure_bonus;
        score += self.edge_stability_bonus(features);
        score += self.partition_quality_bonus(features);

        if features.narrow_corridor {
            score -= self.weights.narrow_corridor_penalty;
        }

        if features.wall_hugging {
            score -= self.contextual_wall_penalty(features);
        }

        if features.articulation_risk {
            score -= self.weights.articulation_penalty;
        }

        if features.self_trap_risk {
            score -= self.weights.self_trap_penalty;
        }

        score
    }

    fn contextual_center_weight(&self, features: MoveFeatures) -> f32 {
        let mut weight = self.weights.center_preference_weight;

        if features.phase == GamePhase::Split {
            weight *= 0.6;
        }

        if features.wall_hugging && self.is_stable_edge_position(features) {
            weight *= 0.25;
        }

        weight
    }

    fn edge_stability_bonus(&self, features: MoveFeatures) -> f32 {
        if !features.wall_hugging || !self.is_stable_edge_position(features) {
            return 0.0;
        }

        let territorial_margin = features.territory_balance.max(0) as f32 * 0.35;
        let local_space_margin = features.local_open_area.saturating_sub(6) as f32 * 0.45;
        let projected_margin = features.projected_reachable_area.saturating_sub(14) as f32 * 0.20;

        territorial_margin + local_space_margin + projected_margin
    }

    fn partition_quality_bonus(&self, features: MoveFeatures) -> f32 {
        let mut bonus = 0.0;

        bonus += features.largest_region_ratio * 8.0;
        bonus += features.edge_escape_routes as f32 * 1.8;
        bonus -= features.region_fragmentation.saturating_sub(1) as f32 * 3.0;

        if features.semi_split_pressure {
            bonus -= 6.0;
        }

        if features.phase == GamePhase::Split && features.territory_balance > 0 {
            bonus += features.territory_balance as f32 * 0.15;
        }

        bonus
    }

    fn is_stable_edge_position(&self, features: MoveFeatures) -> bool {
        match features.phase {
            GamePhase::Split => {
                features.projected_reachable_area >= 20
                    && features.local_open_area >= 8
                    && features.territory_balance >= 0
                    && !features.narrow_corridor
                    && !features.self_trap_risk
            }
            GamePhase::Endgame => {
                features.projected_reachable_area >= 10
                    && features.local_open_area >= 4
                    && !features.self_trap_risk
            }
            GamePhase::Contact => {
                features.projected_reachable_area >= 18
                    && features.local_open_area >= 7
                    && features.territory_balance >= 0
                    && features.opponent_distance >= 3
                    && !features.narrow_corridor
                    && !features.self_trap_risk
            }
        }
    }

    fn contextual_wall_penalty(&self, features: MoveFeatures) -> f32 {
        let mut penalty = self.weights.wall_hugging_penalty;

        if self.is_stable_edge_position(features) {
            penalty *= 0.15;
        } else if features.narrow_corridor || features.self_trap_risk || features.articulation_risk {
            penalty *= 1.25;
        }

        penalty
    }

    fn extract_features(
        &self,
        game_state: &GameState,
        direction: Direction,
        candidate_position: GridPosition,
        phase: GamePhase,
    ) -> MoveFeatures {
        let grid = game_state.current_grid();
        let opponent_head = grid.player_head_position(self.my_player_id.other());
        let projected_grid = self.projected_grid_after_move(grid, direction, candidate_position);
        let projected_reachable_area = distance_map_from_head(&projected_grid, candidate_position)
            .into_iter()
            .flatten()
            .count();
        let voronoi = calculate_voronoi_territory(
            &projected_grid,
            &[self.my_player_id, self.my_player_id.other()],
        );
        let local_open_area = local_open_area_score(grid, candidate_position, 3);
        let branching_factor = empty_neighbor_count(grid, candidate_position);
        let largest_region_ratio = largest_reachable_region_ratio(&projected_grid, candidate_position);
        let region_fragmentation = reachable_region_fragmentation(&projected_grid, candidate_position);
        let articulation_risk = is_articulation_candidate(grid, candidate_position);
        let narrow_corridor = is_narrow_corridor_entry(grid, candidate_position);
        let edge_escape_routes = edge_escape_routes(&projected_grid, candidate_position);
        let territory_balance = voronoi.mine as isize - voronoi.theirs as isize;
        let semi_split_pressure = is_semi_split_pressure(
            projected_reachable_area,
            local_open_area,
            branching_factor,
            territory_balance,
            largest_region_ratio,
        );
        let self_trap_risk = projected_reachable_area <= 12
            || (narrow_corridor && local_open_area <= 6)
            || articulation_risk
            || (semi_split_pressure && edge_escape_routes == 0);

        MoveFeatures {
            reachable_area: reachable_area_count(grid, candidate_position),
            projected_reachable_area,
            branching_factor,
            local_open_area,
            center_preference: center_preference(candidate_position),
            opponent_distance: manhattan_distance(candidate_position, opponent_head),
            largest_region_ratio,
            region_fragmentation,
            edge_escape_routes,
            semi_split_pressure,
            narrow_corridor,
            wall_hugging: is_wall_hugging(candidate_position),
            articulation_risk,
            self_trap_risk,
            voronoi_mine: voronoi.mine,
            voronoi_theirs: voronoi.theirs,
            voronoi_contested: voronoi.contested,
            territory_balance,
            cut_potential: voronoi.mine.saturating_sub(voronoi.theirs),
            phase,
        }
    }

    fn projected_grid_after_move(
        &self,
        grid: &Grid,
        direction: Direction,
        candidate_position: GridPosition,
    ) -> Grid {
        let mut projected = grid.clone();
        let current_head = grid.player_head_position(self.my_player_id);
        *projected.get_cell_mut(current_head) = GridCell::Tail(self.my_player_id, direction);
        *projected.get_cell_mut(candidate_position) = GridCell::Head(self.my_player_id, direction);
        projected
    }
}
