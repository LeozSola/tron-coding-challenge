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

#[derive(Debug, Clone, Copy)]
pub struct MoveFeatures {
    pub reachable_area: usize,
    pub projected_reachable_area: usize,
    pub branching_factor: usize,
    pub local_open_area: usize,
    pub center_preference: f32,
    pub opponent_distance: usize,
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
            reachable_area_weight: 0.35,
            contact_area_weight: 1.20,
            split_area_weight: 1.50,
            endgame_area_weight: 1.75,
            contact_branching_weight: 3.50,
            split_branching_weight: 1.50,
            endgame_branching_weight: 0.75,
            local_open_area_weight: 1.10,
            center_preference_weight: 0.35,
            territory_balance_weight: 0.90,
            contested_territory_weight: 0.20,
            cut_opponent_bonus_weight: 0.65,
            risky_head_to_head_penalty: 500.0,
            narrow_corridor_penalty: 18.0,
            wall_hugging_penalty: 6.0,
            articulation_penalty: 14.0,
            self_trap_penalty: 28.0,
            opponent_pressure_bonus: 0.25,
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
