use crate::engine::prelude::*;
use std::collections::VecDeque;

pub fn neighbors(position: GridPosition) -> [Option<GridPosition>; 4] {
    [
        position.after_moved(Direction::PositiveX),
        position.after_moved(Direction::PositiveY),
        position.after_moved(Direction::NegativeX),
        position.after_moved(Direction::NegativeY),
    ]
}

pub fn reachable_area_count(grid: &Grid, start: GridPosition) -> usize {
    if grid.cell_is_not_empty(start) {
        return 0;
    }

    let mut visited = vec![false; GRID_SIZE * GRID_SIZE];
    let mut queue = VecDeque::from([start]);
    visited[start.i()] = true;

    let mut count = 0usize;
    while let Some(position) = queue.pop_front() {
        count += 1;

        for neighbor in neighbors(position).into_iter().flatten() {
            if visited[neighbor.i()] || grid.cell_is_not_empty(neighbor) {
                continue;
            }

            visited[neighbor.i()] = true;
            queue.push_back(neighbor);
        }
    }

    count
}

pub fn distance_map_from_head(grid: &Grid, head: GridPosition) -> Vec<Option<usize>> {
    let mut distances = vec![None; GRID_SIZE * GRID_SIZE];
    let mut queue = VecDeque::new();

    for neighbor in neighbors(head).into_iter().flatten() {
        if grid.cell_is_not_empty(neighbor) {
            continue;
        }

        distances[neighbor.i()] = Some(1);
        queue.push_back(neighbor);
    }

    while let Some(position) = queue.pop_front() {
        let current_distance = distances[position.i()].expect("distance is set before dequeue");

        for neighbor in neighbors(position).into_iter().flatten() {
            if grid.cell_is_not_empty(neighbor) || distances[neighbor.i()].is_some() {
                continue;
            }

            distances[neighbor.i()] = Some(current_distance + 1);
            queue.push_back(neighbor);
        }
    }

    distances
}

pub fn connected_component_count(grid: &Grid) -> usize {
    let mut visited = vec![false; GRID_SIZE * GRID_SIZE];
    let mut components = 0usize;

    for position in GridPosition::iter_positions() {
        if visited[position.i()] || grid.cell_is_not_empty(position) {
            continue;
        }

        components += 1;
        visited[position.i()] = true;

        let mut queue = VecDeque::from([position]);
        while let Some(current) = queue.pop_front() {
            for neighbor in neighbors(current).into_iter().flatten() {
                if visited[neighbor.i()] || grid.cell_is_not_empty(neighbor) {
                    continue;
                }

                visited[neighbor.i()] = true;
                queue.push_back(neighbor);
            }
        }
    }

    components
}

pub fn count_empty_cells(grid: &Grid) -> usize {
    GridPosition::iter_positions()
        .filter(|position| grid.cell_is_empty(*position))
        .count()
}

pub fn empty_neighbor_count(grid: &Grid, position: GridPosition) -> usize {
    neighbors(position)
        .into_iter()
        .flatten()
        .filter(|neighbor| grid.cell_is_empty(*neighbor))
        .count()
}

pub fn is_narrow_corridor_entry(grid: &Grid, position: GridPosition) -> bool {
    empty_neighbor_count(grid, position) <= 2
}

pub fn center_preference(position: GridPosition) -> f32 {
    let center = (GRID_SIZE / 2) as isize;
    let dx = (position.x() as isize - center).abs();
    let dy = (position.y() as isize - center).abs();
    let distance = (dx + dy) as f32;
    (GRID_SIZE as f32 - distance) / GRID_SIZE as f32
}

pub fn is_wall_hugging(position: GridPosition) -> bool {
    position.x() <= 1
        || position.x() >= GRID_SIZE - 2
        || position.y() <= 1
        || position.y() >= GRID_SIZE - 2
}

pub fn head_direction(grid: &Grid, player_id: PlayerId) -> Option<Direction> {
    match *grid.get_cell(grid.player_head_position(player_id)) {
        GridCell::Head(found_player, direction) if found_player == player_id => Some(direction),
        _ => None,
    }
}

pub fn manhattan_distance(a: GridPosition, b: GridPosition) -> usize {
    a.x().abs_diff(b.x()) + a.y().abs_diff(b.y())
}
