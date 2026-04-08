use crate::engine::prelude::*;
use std::collections::VecDeque;

pub struct StrategyGreedySpaceBot {
    my_player_id: PlayerId,
}

impl Bot for StrategyGreedySpaceBot {
    fn new(my_player_id: PlayerId) -> Self {
        Self { my_player_id }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let grid = game_state.current_grid();
        let my_head = grid.player_head_position(self.my_player_id);

        Direction::all()
            .filter_map(|direction| {
                let next = my_head.after_moved(direction)?;
                if grid.cell_is_not_empty(next) {
                    return None;
                }

                Some((local_space_score(grid, next), direction_priority(direction), direction))
            })
            .max_by_key(|(score, priority, _)| (*score, *priority))
            .map(|(_, _, direction)| direction)
            .unwrap_or(Direction::NegativeX)
    }
}

fn local_space_score(grid: &Grid, start: GridPosition) -> usize {
    let mut visited = vec![false; GRID_SIZE * GRID_SIZE];
    let mut queue = VecDeque::from([(start, 0usize)]);
    visited[start.i()] = true;
    let mut score = 0usize;

    while let Some((position, distance)) = queue.pop_front() {
        score += 1;
        if distance >= 3 {
            continue;
        }

        for direction in Direction::all() {
            let Some(next) = position.after_moved(direction) else {
                continue;
            };

            if visited[next.i()] || grid.cell_is_not_empty(next) {
                continue;
            }

            visited[next.i()] = true;
            queue.push_back((next, distance + 1));
        }
    }

    score
}

fn direction_priority(direction: Direction) -> usize {
    match direction {
        Direction::PositiveY => 3,
        Direction::PositiveX => 2,
        Direction::NegativeY => 1,
        Direction::NegativeX => 0,
    }
}