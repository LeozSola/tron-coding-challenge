use crate::engine::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VoronoiCounts {
    pub mine: usize,
    pub theirs: usize,
    pub contested: usize,
}

/// Get adjacent positions (excluding out of bounds)
pub fn neighbors(position: GridPosition) -> [Option<GridPosition>; 4] {
    [
        position.after_moved(Direction::PositiveX),
        position.after_moved(Direction::PositiveY),
        position.after_moved(Direction::NegativeX),
        position.after_moved(Direction::NegativeY),
    ]
}

/// BFS flood-fill to count reachable empty cells from start
pub fn reachable_area_count(grid: &Grid, start: GridPosition) -> usize {
    if grid.cell_is_not_empty(start) {
        return 0;
    }

    let mut visited = vec![false; GRID_SIZE * GRID_SIZE];
    let mut queue = std::collections::VecDeque::from([start]);
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

/// BFS from head to compute distance map to all reachable cells
pub fn distance_map_from_head(grid: &Grid, head: GridPosition) -> Vec<Option<usize>> {
    let mut distances = vec![None; GRID_SIZE * GRID_SIZE];
    let mut queue = std::collections::VecDeque::new();

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

/// BFS from any empty starting cell.
pub fn distance_map_from_cell(grid: &Grid, start: GridPosition) -> Vec<Option<usize>> {
    if grid.cell_is_not_empty(start) {
        return vec![None; GRID_SIZE * GRID_SIZE];
    }

    let mut distances = vec![None; GRID_SIZE * GRID_SIZE];
    let mut queue = std::collections::VecDeque::from([start]);
    distances[start.i()] = Some(0);

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

/// Extract each connected empty region as an explicit list of cells.
pub fn connected_regions(grid: &Grid) -> Vec<Vec<GridPosition>> {
    let mut visited = vec![false; GRID_SIZE * GRID_SIZE];
    let mut regions = Vec::new();

    for position in GridPosition::iter_positions() {
        if visited[position.i()] || grid.cell_is_not_empty(position) {
            continue;
        }

        let mut queue = std::collections::VecDeque::from([position]);
        let mut region = Vec::new();
        visited[position.i()] = true;

        while let Some(current) = queue.pop_front() {
            region.push(current);

            for neighbor in neighbors(current).into_iter().flatten() {
                if visited[neighbor.i()] || grid.cell_is_not_empty(neighbor) {
                    continue;
                }

                visited[neighbor.i()] = true;
                queue.push_back(neighbor);
            }
        }

        regions.push(region);
    }

    regions
}

/// Fraction of reachable cells contained in the largest reachable region from a start cell.
pub fn largest_reachable_region_ratio(grid: &Grid, start: GridPosition) -> f32 {
    let reachable = distance_map_from_head(grid, start)
        .into_iter()
        .flatten()
        .count();
    if reachable == 0 {
        return 0.0;
    }

    let adjacent_reachable: Vec<GridPosition> = neighbors(start)
        .into_iter()
        .flatten()
        .filter(|position| grid.cell_is_empty(*position))
        .collect();

    let largest = connected_regions(grid)
        .into_iter()
        .filter(|region| {
            region
                .iter()
                .any(|position| adjacent_reachable.iter().any(|candidate| candidate == position))
        })
        .map(|region| region.len())
        .max()
        .unwrap_or(0);

    largest as f32 / reachable as f32
}

/// Number of connected empty regions on the board that are reachable from the start cell's component boundary.
pub fn reachable_region_fragmentation(grid: &Grid, start: GridPosition) -> usize {
    let adjacent_reachable: Vec<GridPosition> = neighbors(start)
        .into_iter()
        .flatten()
        .filter(|position| grid.cell_is_empty(*position))
        .collect();

    if adjacent_reachable.is_empty() {
        return 0;
    }

    connected_regions(grid)
        .into_iter()
        .filter(|region| {
            region
                .iter()
                .any(|position| adjacent_reachable.iter().any(|candidate| candidate == position))
        })
        .count()
}

/// Count how many adjacent empty cells provide roomy continuation near the edge.
pub fn edge_escape_routes(grid: &Grid, position: GridPosition) -> usize {
    neighbors(position)
        .into_iter()
        .flatten()
        .filter(|neighbor| {
            grid.cell_is_empty(*neighbor)
                && empty_neighbor_count(grid, *neighbor) >= 2
                && local_open_area_score(grid, *neighbor, 2) >= 4
        })
        .count()
}

/// Detect awkward semi-split geometry: some territory exists, but local exits are scarce and area is fragmented/tight.
pub fn is_semi_split_pressure(
    projected_reachable_area: usize,
    local_open_area: usize,
    branching_factor: usize,
    territory_balance: isize,
    largest_region_ratio: f32,
) -> bool {
    projected_reachable_area >= 10
        && projected_reachable_area <= 28
        && territory_balance >= -2
        && local_open_area <= 7
        && branching_factor <= 2
        && largest_region_ratio < 0.9
}

/// Count connected empty components on the board
pub fn connected_component_count(grid: &Grid) -> usize {
    connected_regions(grid).len()
}

/// Count total empty cells on the grid
pub fn count_empty_cells(grid: &Grid) -> usize {
    GridPosition::iter_positions()
        .filter(|position| grid.cell_is_empty(*position))
        .count()
}

/// Count empty neighbors of a position
pub fn empty_neighbor_count(grid: &Grid, position: GridPosition) -> usize {
    neighbors(position)
        .into_iter()
        .flatten()
        .filter(|neighbor| grid.cell_is_empty(*neighbor))
        .count()
}

/// Check if position has narrow corridor (<= 2 empty neighbors)
pub fn is_narrow_corridor_entry(grid: &Grid, position: GridPosition) -> bool {
    empty_neighbor_count(grid, position) <= 2
}

/// Small local-space proxy for corridor width / local freedom.
pub fn local_open_area_score(grid: &Grid, position: GridPosition, radius: usize) -> usize {
    distance_map_from_cell(grid, position)
        .into_iter()
        .flatten()
        .filter(|distance| *distance <= radius)
        .count()
}

/// Score for preferring central board positions
pub fn center_preference(position: GridPosition) -> f32 {
    let center = (GRID_SIZE / 2) as isize;
    let dx = (position.x() as isize - center).abs();
    let dy = (position.y() as isize - center).abs();
    let distance = (dx + dy) as f32;
    (GRID_SIZE as f32 - distance) / GRID_SIZE as f32
}

/// Check if position is near wall edges
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

/// Calculate Manhattan distance between two positions
pub fn manhattan_distance(a: GridPosition, b: GridPosition) -> usize {
    a.x().abs_diff(b.x()) + a.y().abs_diff(b.y())
}

/// Get minimum distance to nearest opponent head
pub fn distance_to_nearest_opponent_head(grid: &Grid, player_id: PlayerId, opponent_heads: &[GridPosition]) -> Option<usize> {
    if opponent_heads.is_empty() {
        return None;
    }
    
    let my_head = grid.player_head_position(player_id);
    
    let mut nearest_distance = usize::MAX;
    
    for &opp_head in opponent_heads {
        let dist = manhattan_distance(my_head, opp_head);
        nearest_distance = nearest_distance.min(dist);
    }
    
    if nearest_distance == usize::MAX {
        None
    } else {
        Some(nearest_distance)
    }
}

/// Compare territory ownership from both heads and count owned vs contested cells.
pub fn calculate_voronoi_territory(grid: &Grid, player_ids: &[PlayerId]) -> VoronoiCounts {
    if player_ids.len() < 2 {
        return VoronoiCounts::default();
    }

    let my_map = distance_map_from_head(grid, grid.player_head_position(player_ids[0]));
    let their_map = distance_map_from_head(grid, grid.player_head_position(player_ids[1]));
    let mut counts = VoronoiCounts::default();

    for position in GridPosition::iter_positions() {
        match (my_map[position.i()], their_map[position.i()]) {
            (Some(a), Some(b)) if a < b => counts.mine += 1,
            (Some(a), Some(b)) if b < a => counts.theirs += 1,
            (Some(_), Some(_)) => counts.contested += 1,
            (Some(_), None) => counts.mine += 1,
            (None, Some(_)) => counts.theirs += 1,
            (None, None) => {}
        }
    }

    counts
}

/// Get safe moves that avoid immediate collisions
pub fn get_safe_moves(grid: &Grid, player_id: PlayerId, opponent_heads: &[GridPosition]) -> Vec<Direction> {
    let head_pos = grid.player_head_position(player_id);
    
    let mut safe = Vec::new();
    
    for direction in Direction::all() {
        let new_pos = head_pos.after_moved(direction);
        
        if let Some(new_pos) = new_pos {
            // Check wall/boundary
            if grid.cell_is_empty(new_pos) {
                // Check opponent head collision
                let mut collision = false;
                for opp_head in opponent_heads {
                    if new_pos == *opp_head {
                        collision = true;
                        break;
                    }
                }
                if !collision {
                    safe.push(direction);
                }
            }
        }
    }
    
    safe
}

/// Calculate territory pressure (opponent occupancy near head)
pub fn calculate_territory_pressure(grid: &Grid, player_id: PlayerId, opponent_heads: &[GridPosition]) -> f32 {
    if opponent_heads.is_empty() {
        return 0.0;
    }
    
    let head_pos = grid.player_head_position(player_id);
    let mut pressure = 0.0;
    
    // Check 2-cell radius for opponent heads
    for &opponent_head in opponent_heads {
        let dist = manhattan_distance(head_pos, opponent_head);
        if dist <= 2 {
            pressure += 1.0;
        }
    }
    
    pressure
}

/// Get distance to nearest own tail segment, if any exist.
pub fn distance_to_own_tail(grid: &Grid, player_id: PlayerId) -> Option<usize> {
    let head_pos = grid.player_head_position(player_id);
    let distances = distance_map_from_head(grid, head_pos);

    GridPosition::iter_positions()
        .filter(|position| matches!(grid.get_cell(*position), GridCell::Tail(found, _) if *found == player_id))
        .filter_map(|position| distances[position.i()])
        .min()
}

/// Get opponent head positions
pub fn get_opponent_heads(grid: &Grid, player_id: PlayerId) -> Vec<GridPosition> {
    vec![grid.player_head_position(player_id.other())]
}

/// Get all heads on the board
pub fn get_all_heads(grid: &Grid) -> Vec<GridPosition> {
    let mut heads = Vec::new();
    let cells = grid.get_cells();
    
    for (pos, cell) in cells.iter().enumerate() {
        if let GridCell::Head(_player_id, _) = cell {
            if let Some(p) = GridPosition::new_from_usize(pos) {
                heads.push(p);
            }
        }
    }
    
    heads
}

/// Lightweight choke-point scaffold for future heuristics.
pub fn is_articulation_candidate(grid: &Grid, position: GridPosition) -> bool {
    grid.cell_is_empty(position) && empty_neighbor_count(grid, position) <= 2
}

/// Symmetry-normalized relative offset scaffold for future ML features.
pub fn normalized_relative_offset(origin: GridPosition, target: GridPosition) -> (isize, isize) {
    (
        target.x() as isize - origin.x() as isize,
        target.y() as isize - origin.y() as isize,
    )
}

/// Check if we're in a split phase (two separate reachable areas)
pub fn is_split_phase(grid: &Grid, player_id: PlayerId) -> bool {
    let head_pos = grid.player_head_position(player_id);
    let count = reachable_area_count(grid, head_pos);
    count <= (GRID_SIZE * GRID_SIZE) / 3
}

/// Check if we're in endgame phase (low empty cells)
pub fn is_endgame_phase(grid: &Grid, player_id: PlayerId) -> bool {
    let head_pos = grid.player_head_position(player_id);
    let reachable = reachable_area_count(grid, head_pos);
    reachable < 15
}

/// Check if we're in contact phase (high pressure near head)
pub fn is_contact_phase(grid: &Grid, player_id: PlayerId, opponent_heads: &[GridPosition]) -> bool {
    let head_pos = grid.player_head_position(player_id);
    let mut pressure = 0.0;
    
    for &opponent_head in opponent_heads {
        let dist = manhattan_distance(head_pos, opponent_head);
        if dist <= 2 {
            pressure += 1.0;
        }
    }
    
    pressure > 1.0
}