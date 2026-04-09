use crate::engine::{GameOver, prelude::*};
use crate::players::competitive_bot_v1::CompetitiveBot as CompetitiveBotV1;
use crate::players::strategy_bots::strategy_chaser::StrategyChaserBot;
use crate::players::strategy_bots::strategy_center::StrategyCenterBot;
use crate::players::strategy_bots::strategy_corner::StrategyCornerBot;
use crate::players::strategy_bots::strategy_greedy_space::StrategyGreedySpaceBot;
use crate::players::strategy_bots::strategy_max_branch::StrategyMaxBranchBot;
use crate::players::strategy_bots::strategy_safe::StrategySafeBot;
use crate::players::strategy_bots::strategy_straight::StrategyStraightBot;
use crate::players::strategy_bots::strategy_wall_hug::StrategyWallHugBot;

use super::analysis::{
    calculate_voronoi_territory, connected_component_count, connected_regions,
    distance_map_from_cell, distance_map_from_head, edge_escape_routes, empty_neighbor_count,
    is_articulation_candidate, is_semi_split_pressure, largest_reachable_region_ratio,
    local_open_area_score, normalized_relative_offset, reachable_area_count,
};
use super::heuristic::HeuristicEvaluator;
use super::phase::{classify_phase, classify_phase_profile};
use super::safety::MoveSafetyAnalyzer;
use super::types::{
    GamePhase, HeuristicWeights, LossReason, MoveFeatures, MoveSafety, OpponentProfile,
    ScoredMove,
};
use super::CompetitiveBot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BenchmarkBotKind {
    CompBotCur,
    CompBotV1,
    Safe,
    WallHug,
    Center,
    Corner,
    GreedySpace,
    MaxBranch,
    Chaser,
    Straight,
}

impl BenchmarkBotKind {
    const COMP_BOTS: [Self; 2] = [Self::CompBotCur, Self::CompBotV1];

    const BASELINE_BOTS: [Self; 8] = [
        Self::Safe,
        Self::WallHug,
        Self::Center,
        Self::Corner,
        Self::GreedySpace,
        Self::MaxBranch,
        Self::Chaser,
        Self::Straight,
    ];

    const ALL: [Self; 10] = [
        Self::CompBotCur,
        Self::CompBotV1,
        Self::Safe,
        Self::WallHug,
        Self::Center,
        Self::Corner,
        Self::GreedySpace,
        Self::MaxBranch,
        Self::Chaser,
        Self::Straight,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::CompBotCur => "compBotCur",
            Self::CompBotV1 => "compBotV1",
            Self::Safe => "strategy_safe",
            Self::WallHug => "strategy_wall_hug",
            Self::Center => "strategy_center",
            Self::Corner => "strategy_corner",
            Self::GreedySpace => "strategy_greedy_space",
            Self::MaxBranch => "strategy_max_branch",
            Self::Chaser => "strategy_chaser",
            Self::Straight => "strategy_straight",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OpeningScenario {
    label: &'static str,
    script: &'static [(Direction, Direction)],
}

const OPENING_SCENARIOS: [OpeningScenario; 8] = [
    OpeningScenario {
        label: "default",
        script: &[],
    },
    OpeningScenario {
        label: "vertical_split",
        script: &[
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::PositiveY, Direction::NegativeY),
        ],
    },
    OpeningScenario {
        label: "horizontal_escape",
        script: &[
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::NegativeX, Direction::PositiveX),
        ],
    },
    OpeningScenario {
        label: "edge_bias",
        script: &[
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::NegativeX, Direction::PositiveX),
        ],
    },
    OpeningScenario {
        label: "center_pressure",
        script: &[
            (Direction::PositiveX, Direction::NegativeX),
        ],
    },
    OpeningScenario {
        label: "offset_corridor",
        script: &[
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::NegativeX, Direction::NegativeY),
            (Direction::NegativeY, Direction::NegativeX),
        ],
    },
    OpeningScenario {
        label: "wall_hug_lane_race",
        script: &[
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::PositiveY, Direction::NegativeY),
        ],
    },
    OpeningScenario {
        label: "edge_stability_probe",
        script: &[
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::PositiveY, Direction::NegativeY),
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::NegativeX, Direction::PositiveX),
            (Direction::NegativeY, Direction::PositiveY),
        ],
    },
];

#[derive(Debug, Clone, Copy, Default)]
struct MatchSummary {
    wins: usize,
    losses: usize,
    draws: usize,
}

impl MatchSummary {
    fn record_as_player_o(&mut self, result: GameOver) {
        match result {
            GameOver::Winner { player_who_won } if player_who_won == PlayerId::new_o() => {
                self.wins += 1;
            }
            GameOver::Winner { .. } => {
                self.losses += 1;
            }
            GameOver::Draw => {
                self.draws += 1;
            }
        }
    }

    fn merge(&mut self, other: Self) {
        self.wins += other.wins;
        self.losses += other.losses;
        self.draws += other.draws;
    }
}

fn instantiate_bot(kind: BenchmarkBotKind, player_id: PlayerId) -> Box<dyn BotRunner> {
    match kind {
        BenchmarkBotKind::CompBotCur => Box::new(CompetitiveBot::new(player_id)),
        BenchmarkBotKind::CompBotV1 => Box::new(CompetitiveBotV1::new(player_id)),
        BenchmarkBotKind::Safe => Box::new(StrategySafeBot::new(player_id)),
        BenchmarkBotKind::WallHug => Box::new(StrategyWallHugBot::new(player_id)),
        BenchmarkBotKind::Center => Box::new(StrategyCenterBot::new(player_id)),
        BenchmarkBotKind::Corner => Box::new(StrategyCornerBot::new(player_id)),
        BenchmarkBotKind::GreedySpace => Box::new(StrategyGreedySpaceBot::new(player_id)),
        BenchmarkBotKind::MaxBranch => Box::new(StrategyMaxBranchBot::new(player_id)),
        BenchmarkBotKind::Chaser => Box::new(StrategyChaserBot::new(player_id)),
        BenchmarkBotKind::Straight => Box::new(StrategyStraightBot::new(player_id)),
    }
}

trait BotRunner {
    fn choose_action(&mut self, game_state: &GameState) -> Direction;
}

impl<T: Bot> BotRunner for T {
    fn choose_action(&mut self, game_state: &GameState) -> Direction {
        Bot::next_action(self, game_state)
    }
}

fn build_game_state(script: &[(Direction, Direction)]) -> Option<GameState> {
    let mut game_state = GameState::new();

    for &(player_o, player_x) in script {
        if !game_state.go_to_next_frame(player_o, player_x) {
            return None;
        }
    }

    Some(game_state)
}

fn run_matchup(
    player_o_kind: BenchmarkBotKind,
    player_x_kind: BenchmarkBotKind,
    script: &[(Direction, Direction)],
) -> Option<GameOver> {
    let mut game_state = build_game_state(script)?;
    let mut player_o = instantiate_bot(player_o_kind, PlayerId::new_o());
    let mut player_x = instantiate_bot(player_x_kind, PlayerId::new_x());

    while game_state.go_to_next_frame(
        player_o.choose_action(&game_state),
        player_x.choose_action(&game_state),
    ) {}

    game_state.game_over()
}

fn summarize_pairing(player_o_kind: BenchmarkBotKind, player_x_kind: BenchmarkBotKind) -> MatchSummary {
    let mut summary = MatchSummary::default();

    for scenario in OPENING_SCENARIOS {
        if let Some(result) = run_matchup(player_o_kind, player_x_kind, scenario.script) {
            summary.record_as_player_o(result);
        }
    }

    summary
}

fn summarize_pairing_for_scenarios(
    player_o_kind: BenchmarkBotKind,
    player_x_kind: BenchmarkBotKind,
    scenarios: &[OpeningScenario],
) -> MatchSummary {
    let mut summary = MatchSummary::default();

    for scenario in scenarios {
        if let Some(result) = run_matchup(player_o_kind, player_x_kind, scenario.script) {
            summary.record_as_player_o(result);
        }
    }

    summary
}

fn summarize_named_scenarios(
    player_o_kind: BenchmarkBotKind,
    player_x_kind: BenchmarkBotKind,
    scenarios: &[OpeningScenario],
) -> Vec<(&'static str, Option<GameOver>)> {
    scenarios
        .iter()
        .map(|scenario| {
            (
                scenario.label,
                run_matchup(player_o_kind, player_x_kind, scenario.script),
            )
        })
        .collect()
}

fn print_pairing_result(label: &str, summary: MatchSummary, rounds: usize) {
    println!(
        "{label}: wins={}, losses={}, draws={} ({} scenarios)",
        summary.wins, summary.losses, summary.draws, rounds
    );
}

#[test]
fn benchmark_simple_strategies_against_heuristic_bot_v1() {
    println!(
        "=== Comp bot focused 1v1 benchmark ({} opening scenarios each) ===",
        OPENING_SCENARIOS.len()
    );
    println!("Openings:");
    for scenario in OPENING_SCENARIOS {
        println!("- {} ({} ply)", scenario.label, scenario.script.len());
    }

    let mut totals = [(BenchmarkBotKind::CompBotCur, MatchSummary::default()); 2];
    for (slot, kind) in totals.iter_mut().zip(BenchmarkBotKind::COMP_BOTS) {
        slot.0 = kind;
    }

    for comp_kind in BenchmarkBotKind::COMP_BOTS {
        println!("=== {} matchups ===", comp_kind.label());

        let opponents = BenchmarkBotKind::BASELINE_BOTS
            .into_iter()
            .chain(BenchmarkBotKind::COMP_BOTS.into_iter().filter(|kind| *kind != comp_kind));

        for opponent_kind in opponents {

            let as_player_o = summarize_pairing(comp_kind, opponent_kind);
            let label = format!("{} vs {}", comp_kind.label(), opponent_kind.label());
            print_pairing_result(&label, as_player_o, OPENING_SCENARIOS.len());
            totals
                .iter_mut()
                .find(|(kind, _)| *kind == comp_kind)
                .expect("comp bot total slot exists")
                .1
                .merge(as_player_o);

            let as_player_x_for_opponent = summarize_pairing(opponent_kind, comp_kind);
            let as_player_x = MatchSummary {
                wins: as_player_x_for_opponent.losses,
                losses: as_player_x_for_opponent.wins,
                draws: as_player_x_for_opponent.draws,
            };
            let label = format!("{} vs {}", comp_kind.label(), opponent_kind.label());
            print_pairing_result(&format!("{label} (comp as X)"), as_player_x, OPENING_SCENARIOS.len());
            totals
                .iter_mut()
                .find(|(kind, _)| *kind == comp_kind)
                .expect("comp bot total slot exists")
                .1
                .merge(as_player_x);
        }
    }

    println!("=== Aggregate comp bot totals across targeted matchups and openings ===");
    for (kind, summary) in totals {
        let games = summary.wins + summary.losses + summary.draws;
        println!(
            "{}: wins={}, losses={}, draws={}, non_loss_rate={:.3}",
            kind.label(),
            summary.wins,
            summary.losses,
            summary.draws,
            if games == 0 {
                0.0
            } else {
                (summary.wins + summary.draws) as f32 / games as f32
            }
        );
    }
}

#[test]
fn baseline_prefers_a_non_losing_opening_move() {
    let mut bot = CompetitiveBot::new(PlayerId::new_o());
    let state = GameState::new();
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let action = bot.next_action(&state);

    assert!(matches!(
        safety.classify_move(&state, action),
        MoveSafety::Safe
    ));
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
fn heuristic_rewards_territory_and_open_space() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let strong = MoveFeatures {
        reachable_area: 24,
        projected_reachable_area: 28,
        branching_factor: 3,
        local_open_area: 12,
        center_preference: 0.8,
        opponent_distance: 3,
        largest_region_ratio: 1.0,
        region_fragmentation: 1,
        edge_escape_routes: 3,
        semi_split_pressure: false,
        narrow_corridor: false,
        wall_hugging: false,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 30,
        voronoi_theirs: 18,
        voronoi_contested: 6,
        territory_balance: 12,
        cut_potential: 12,
        phase: GamePhase::Contact,
        contact_score: 0.8,
        split_score: 0.2,
        endgame_score: 0.1,
        phase_contested_ratio: 0.12,
        phase_shared_reachable: 20,
        phase_corridor_severity: 0.1,
    };
    let weak = MoveFeatures {
        reachable_area: 24,
        projected_reachable_area: 18,
        branching_factor: 2,
        local_open_area: 5,
        center_preference: 0.5,
        opponent_distance: 3,
        largest_region_ratio: 0.75,
        region_fragmentation: 2,
        edge_escape_routes: 1,
        semi_split_pressure: true,
        narrow_corridor: false,
        wall_hugging: false,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 18,
        voronoi_theirs: 28,
        voronoi_contested: 4,
        territory_balance: -10,
        cut_potential: 0,
        phase: GamePhase::Contact,
        contact_score: 0.8,
        split_score: 0.2,
        endgame_score: 0.1,
        phase_contested_ratio: 0.12,
        phase_shared_reachable: 20,
        phase_corridor_severity: 0.1,
    };

    assert!(evaluator.score_features(strong) > evaluator.score_features(weak));
}

#[test]
fn heuristic_penalizes_self_trap_signals() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let safe = MoveFeatures {
        reachable_area: 16,
        projected_reachable_area: 18,
        branching_factor: 3,
        local_open_area: 10,
        center_preference: 0.7,
        opponent_distance: 5,
        largest_region_ratio: 1.0,
        region_fragmentation: 1,
        edge_escape_routes: 3,
        semi_split_pressure: false,
        narrow_corridor: false,
        wall_hugging: false,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 20,
        voronoi_theirs: 19,
        voronoi_contested: 3,
        territory_balance: 1,
        cut_potential: 1,
        phase: GamePhase::Split,
        contact_score: 0.15,
        split_score: 0.75,
        endgame_score: 0.15,
        phase_contested_ratio: 0.05,
        phase_shared_reachable: 6,
        phase_corridor_severity: 0.2,
    };
    let trapped = MoveFeatures {
        reachable_area: 16,
        projected_reachable_area: 8,
        branching_factor: 1,
        local_open_area: 3,
        center_preference: 0.3,
        opponent_distance: 5,
        largest_region_ratio: 0.65,
        region_fragmentation: 2,
        edge_escape_routes: 0,
        semi_split_pressure: true,
        narrow_corridor: true,
        wall_hugging: true,
        articulation_risk: true,
        self_trap_risk: true,
        voronoi_mine: 10,
        voronoi_theirs: 20,
        voronoi_contested: 2,
        territory_balance: -10,
        cut_potential: 0,
        phase: GamePhase::Split,
        contact_score: 0.15,
        split_score: 0.75,
        endgame_score: 0.15,
        phase_contested_ratio: 0.05,
        phase_shared_reachable: 6,
        phase_corridor_severity: 0.7,
    };

    assert!(evaluator.score_features(safe) > evaluator.score_features(trapped));
}

#[test]
fn heuristic_softens_wall_penalty_for_stable_edge_positions() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let stable_edge = MoveFeatures {
        reachable_area: 22,
        projected_reachable_area: 24,
        branching_factor: 3,
        local_open_area: 10,
        center_preference: 0.2,
        opponent_distance: 5,
        largest_region_ratio: 1.0,
        region_fragmentation: 1,
        edge_escape_routes: 3,
        semi_split_pressure: false,
        narrow_corridor: false,
        wall_hugging: true,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 24,
        voronoi_theirs: 18,
        voronoi_contested: 4,
        territory_balance: 6,
        cut_potential: 6,
        phase: GamePhase::Split,
        contact_score: 0.1,
        split_score: 0.8,
        endgame_score: 0.1,
        phase_contested_ratio: 0.04,
        phase_shared_reachable: 4,
        phase_corridor_severity: 0.1,
    };
    let same_but_not_wall = MoveFeatures {
        wall_hugging: false,
        ..stable_edge
    };

    let score_gap = evaluator.score_features(same_but_not_wall) - evaluator.score_features(stable_edge);
    assert!(score_gap < 0.0);
}

#[test]
fn heuristic_keeps_fuller_wall_penalty_for_cramped_edge_positions() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let cramped_edge = MoveFeatures {
        reachable_area: 10,
        projected_reachable_area: 8,
        branching_factor: 1,
        local_open_area: 3,
        center_preference: 0.2,
        opponent_distance: 2,
        largest_region_ratio: 0.7,
        region_fragmentation: 2,
        edge_escape_routes: 0,
        semi_split_pressure: true,
        narrow_corridor: true,
        wall_hugging: true,
        articulation_risk: true,
        self_trap_risk: true,
        voronoi_mine: 8,
        voronoi_theirs: 15,
        voronoi_contested: 2,
        territory_balance: -7,
        cut_potential: 0,
        phase: GamePhase::Contact,
        contact_score: 0.7,
        split_score: 0.25,
        endgame_score: 0.15,
        phase_contested_ratio: 0.1,
        phase_shared_reachable: 12,
        phase_corridor_severity: 0.8,
    };
    let same_but_not_wall = MoveFeatures {
        wall_hugging: false,
        ..cramped_edge
    };

    let score_gap = evaluator.score_features(same_but_not_wall) - evaluator.score_features(cramped_edge);
    assert!(score_gap >= HeuristicWeights::default().wall_hugging_penalty);
}

#[test]
fn benchmark_targeted_wall_hug_scenarios_show_no_losing_record() {
    let targeted = [OPENING_SCENARIOS[3], OPENING_SCENARIOS[5], OPENING_SCENARIOS[6], OPENING_SCENARIOS[7]];

    let as_player_o = summarize_pairing_for_scenarios(
        BenchmarkBotKind::CompBotCur,
        BenchmarkBotKind::WallHug,
        &targeted,
    );
    let as_player_x = summarize_pairing_for_scenarios(
        BenchmarkBotKind::WallHug,
        BenchmarkBotKind::CompBotCur,
        &targeted,
    );
    let as_player_x_for_comp = MatchSummary {
        wins: as_player_x.losses,
        losses: as_player_x.wins,
        draws: as_player_x.draws,
    };

    assert!(
        as_player_o.wins >= as_player_o.losses,
        "compBotCur should not have a losing record as player O in targeted wall-hug scenarios: {:?}",
        as_player_o
    );
    assert!(
        as_player_x_for_comp.wins >= as_player_x_for_comp.losses,
        "compBotCur should not have a losing record as player X in targeted wall-hug scenarios: {:?}",
        as_player_x_for_comp
    );
}

#[test]
fn debug_targeted_wall_hug_scenario_results() {
    let targeted = [OPENING_SCENARIOS[3], OPENING_SCENARIOS[5], OPENING_SCENARIOS[6], OPENING_SCENARIOS[7]];

    println!("CompBotCur as O vs WallHug as X:");
    for (label, result) in summarize_named_scenarios(BenchmarkBotKind::CompBotCur, BenchmarkBotKind::WallHug, &targeted) {
        println!("- {label}: {:?}", result);
    }

    println!("WallHug as O vs CompBotCur as X:");
    for (label, result) in summarize_named_scenarios(BenchmarkBotKind::WallHug, BenchmarkBotKind::CompBotCur, &targeted) {
        println!("- {label}: {:?}", result);
    }
}

#[test]
fn heuristic_evaluation_populates_phase_three_features() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );
    let state = GameState::new();
    let safety = MoveSafetyAnalyzer::new(PlayerId::new_o());
    let phase_profile = classify_phase_profile(PlayerId::new_o(), state.current_grid());
    let candidates = evaluator.evaluate_moves(&state, phase_profile, &safety);
    let best_safe = candidates
        .iter()
        .find(|candidate| candidate.direction == Direction::PositiveY)
        .and_then(|candidate| candidate.features);

    let features = best_safe.expect("opening safe move should have features");
    assert!(features.projected_reachable_area > 0);
    assert!(features.local_open_area > 0);
    assert!(features.voronoi_mine + features.voronoi_theirs + features.voronoi_contested > 0);
    assert!(features.largest_region_ratio > 0.0);
}

#[test]
fn heuristic_rewards_stronger_partition_quality() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let strong_partition = MoveFeatures {
        reachable_area: 20,
        projected_reachable_area: 22,
        branching_factor: 3,
        local_open_area: 9,
        center_preference: 0.4,
        opponent_distance: 4,
        largest_region_ratio: 1.0,
        region_fragmentation: 1,
        edge_escape_routes: 3,
        semi_split_pressure: false,
        narrow_corridor: false,
        wall_hugging: false,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 24,
        voronoi_theirs: 18,
        voronoi_contested: 3,
        territory_balance: 6,
        cut_potential: 6,
        phase: GamePhase::Split,
        contact_score: 0.1,
        split_score: 0.85,
        endgame_score: 0.1,
        phase_contested_ratio: 0.03,
        phase_shared_reachable: 2,
        phase_corridor_severity: 0.15,
    };
    let weak_partition = MoveFeatures {
        largest_region_ratio: 0.68,
        region_fragmentation: 2,
        edge_escape_routes: 0,
        semi_split_pressure: true,
        territory_balance: 1,
        cut_potential: 1,
        ..strong_partition
    };

    assert!(evaluator.score_features(strong_partition) > evaluator.score_features(weak_partition));
}

#[test]
fn analysis_detects_open_region_quality_signals() {
    let state = GameState::new();
    let grid = state.current_grid();
    let start = grid
        .player_head_position(PlayerId::new_o())
        .after_moved(Direction::PositiveY)
        .expect("in bounds");

    assert!(largest_reachable_region_ratio(grid, start) > 0.9);
    assert!(edge_escape_routes(grid, start) >= 2);
    assert!(!is_semi_split_pressure(24, 10, 3, 2, 1.0));
}

#[test]
fn phase_detection_classifies_opening_as_contact() {
    let state = GameState::new();

    assert_eq!(classify_phase(PlayerId::new_o(), state.current_grid()), GamePhase::Contact);
}

#[test]
fn phase_detection_classifies_separated_regions_as_split() {
    let mut grid = GameState::new().current_grid().clone();

    for y in 0..GRID_SIZE {
        let position = GridPosition::new(10, y).expect("in bounds");
        *grid.get_cell_mut(position) = GridCell::Tail(PlayerId::new_o(), Direction::PositiveY);
    }

    *grid.get_cell_mut(grid.player_head_position(PlayerId::new_o())) =
        GridCell::Head(PlayerId::new_o(), Direction::NegativeX);
    *grid.get_cell_mut(grid.player_head_position(PlayerId::new_x())) =
        GridCell::Head(PlayerId::new_x(), Direction::PositiveX);

    assert_eq!(classify_phase(PlayerId::new_o(), &grid), GamePhase::Split);
}

#[test]
fn phase_detection_classifies_low_space_boards_as_endgame() {
    let mut grid = GameState::new().current_grid().clone();

    for position in GridPosition::iter_positions() {
        if grid.get_cell(position).is_head() {
            continue;
        }

        *grid.get_cell_mut(position) = GridCell::Tail(PlayerId::new_o(), Direction::PositiveY);
    }

    let preserved_open_cells = [
        GridPosition::new(0, 0).expect("in bounds"),
        GridPosition::new(1, 0).expect("in bounds"),
        GridPosition::new(2, 0).expect("in bounds"),
        GridPosition::new(3, 0).expect("in bounds"),
        GridPosition::new(4, 0).expect("in bounds"),
        GridPosition::new(5, 0).expect("in bounds"),
        GridPosition::new(6, 0).expect("in bounds"),
        GridPosition::new(7, 0).expect("in bounds"),
        GridPosition::new(8, 0).expect("in bounds"),
        GridPosition::new(9, 0).expect("in bounds"),
    ];

    for position in preserved_open_cells {
        *grid.get_cell_mut(position) = GridCell::Empty;
    }

    assert_eq!(classify_phase(PlayerId::new_o(), &grid), GamePhase::Endgame);
}

#[test]
fn phase_profile_exposes_soft_scores_and_prefers_endgame_when_corridors_tighten() {
    let mut grid = GameState::new().current_grid().clone();

    for position in GridPosition::iter_positions() {
        if grid.get_cell(position).is_head() {
            continue;
        }

        *grid.get_cell_mut(position) = GridCell::Tail(PlayerId::new_o(), Direction::PositiveY);
    }

    let preserved_open_cells = [
        GridPosition::new(8, 9).expect("in bounds"),
        GridPosition::new(9, 9).expect("in bounds"),
        GridPosition::new(10, 9).expect("in bounds"),
        GridPosition::new(10, 10).expect("in bounds"),
        GridPosition::new(10, 11).expect("in bounds"),
        GridPosition::new(11, 11).expect("in bounds"),
        GridPosition::new(12, 11).expect("in bounds"),
        GridPosition::new(12, 12).expect("in bounds"),
    ];

    for position in preserved_open_cells {
        if !grid.get_cell(position).is_head() {
            *grid.get_cell_mut(position) = GridCell::Empty;
        }
    }

    let profile = classify_phase_profile(PlayerId::new_o(), &grid);
    assert_eq!(profile.phase, GamePhase::Endgame);
    assert!(profile.scores.endgame > profile.scores.contact);
    assert!(profile.scores.endgame > profile.scores.split);
    assert!(profile.my_corridor_severity > 0.0 || profile.opponent_corridor_severity > 0.0);
}

#[test]
fn heuristic_phase_profile_changes_scoring_for_identical_move_shape() {
    let evaluator = HeuristicEvaluator::new(
        PlayerId::new_o(),
        HeuristicWeights::default(),
        OpponentProfile::default(),
    );

    let contact_features = MoveFeatures {
        reachable_area: 20,
        projected_reachable_area: 18,
        branching_factor: 3,
        local_open_area: 8,
        center_preference: 0.5,
        opponent_distance: 3,
        largest_region_ratio: 0.9,
        region_fragmentation: 1,
        edge_escape_routes: 2,
        semi_split_pressure: false,
        narrow_corridor: false,
        wall_hugging: false,
        articulation_risk: false,
        self_trap_risk: false,
        voronoi_mine: 20,
        voronoi_theirs: 18,
        voronoi_contested: 10,
        territory_balance: 2,
        cut_potential: 2,
        phase: GamePhase::Contact,
        contact_score: 0.85,
        split_score: 0.20,
        endgame_score: 0.10,
        phase_contested_ratio: 0.18,
        phase_shared_reachable: 18,
        phase_corridor_severity: 0.15,
    };
    let endgame_features = MoveFeatures {
        phase: GamePhase::Endgame,
        contact_score: 0.10,
        split_score: 0.15,
        endgame_score: 0.90,
        phase_contested_ratio: 0.03,
        phase_shared_reachable: 4,
        phase_corridor_severity: 0.75,
        ..contact_features
    };

    assert_ne!(
        evaluator.score_features(contact_features),
        evaluator.score_features(endgame_features)
    );
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
