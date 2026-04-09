use crate::engine::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveSafety {
    Safe,
    RiskyHeadToHead,
    Losing(LossReason),
}

impl MoveSafety {
    pub const fn is_safe(self) -> bool {
        matches!(self, Self::Safe)
    }

    pub const fn is_risky(self) -> bool {
        matches!(self, Self::RiskyHeadToHead)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LossReason {
    OutOfBounds,
    OccupiedCell,
    OpponentHeadCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Contact,
    Split,
    Endgame,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PhaseScores {
    pub contact: f32,
    pub split: f32,
    pub endgame: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PhaseProfile {
    pub phase: GamePhase,
    pub scores: PhaseScores,
    pub empty_cells: usize,
    pub empty_components: usize,
    pub my_reachable: usize,
    pub opponent_reachable: usize,
    pub shared_reachable: usize,
    pub contested_cells: usize,
    pub contested_ratio: f32,
    pub head_distance: usize,
    pub my_local_open_area: usize,
    pub opponent_local_open_area: usize,
    pub my_branching: usize,
    pub opponent_branching: usize,
    pub my_projected_space: usize,
    pub opponent_projected_space: usize,
    pub my_region_fragmentation: usize,
    pub opponent_region_fragmentation: usize,
    pub my_corridor_severity: f32,
    pub opponent_corridor_severity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveFeatures {
    pub reachable_area: usize,
    pub projected_reachable_area: usize,
    pub branching_factor: usize,
    pub local_open_area: usize,
    pub center_preference: f32,
    pub opponent_distance: usize,
    pub largest_region_ratio: f32,
    pub region_fragmentation: usize,
    pub edge_escape_routes: usize,
    pub semi_split_pressure: bool,
    pub narrow_corridor: bool,
    pub wall_hugging: bool,
    pub articulation_risk: bool,
    pub self_trap_risk: bool,
    pub voronoi_mine: usize,
    pub voronoi_theirs: usize,
    pub voronoi_contested: usize,
    pub territory_balance: isize,
    pub cut_potential: usize,
    pub phase: GamePhase,
    pub contact_score: f32,
    pub split_score: f32,
    pub endgame_score: f32,
    pub phase_contested_ratio: f32,
    pub phase_shared_reachable: usize,
    pub phase_corridor_severity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ScoredMove {
    pub direction: Direction,
    pub safety: MoveSafety,
    pub score: f32,
    pub features: Option<MoveFeatures>,
}

#[derive(Debug, Clone, Copy)]
pub struct HeuristicWeights {
    pub reachable_area_weight: f32,
    pub contact_area_weight: f32,
    pub split_area_weight: f32,
    pub endgame_area_weight: f32,
    pub contact_branching_weight: f32,
    pub split_branching_weight: f32,
    pub endgame_branching_weight: f32,
    pub local_open_area_weight: f32,
    pub center_preference_weight: f32,
    pub territory_balance_weight: f32,
    pub contested_territory_weight: f32,
    pub cut_opponent_bonus_weight: f32,
    pub risky_head_to_head_penalty: f32,
    pub narrow_corridor_penalty: f32,
    pub wall_hugging_penalty: f32,
    pub articulation_penalty: f32,
    pub self_trap_penalty: f32,
    pub opponent_pressure_bonus: f32,
}

impl Default for HeuristicWeights {
    fn default() -> Self {
        Self {
            reachable_area_weight: 0.40,
            contact_area_weight: 1.25,
            split_area_weight: 1.65,
            endgame_area_weight: 1.75,
            contact_branching_weight: 3.00,
            split_branching_weight: 1.35,
            endgame_branching_weight: 0.75,
            local_open_area_weight: 1.00,
            center_preference_weight: 0.20,
            territory_balance_weight: 1.05,
            contested_territory_weight: 0.25,
            cut_opponent_bonus_weight: 0.75,
            risky_head_to_head_penalty: 500.0,
            narrow_corridor_penalty: 18.0,
            wall_hugging_penalty: 5.0,
            articulation_penalty: 15.0,
            self_trap_penalty: 30.0,
            opponent_pressure_bonus: 0.20,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OpponentProfile {
    pub turns_observed: usize,
    pub wall_hug_ratio: f32,
    pub aggression_ratio: f32,
    pub corridor_ratio: f32,
    pub horizontal_bias: f32,
}
