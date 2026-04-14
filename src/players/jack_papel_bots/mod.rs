use std::{collections::{HashMap, HashSet}, fmt::Display, rc::Rc, usize};

use crate::engine::{grid::Grid, prelude::{Direction, GameState, GridPosition, PlayerId}};

pub mod hallucinator;
pub mod freedom_eater;
pub mod rip_and_tear;

#[derive(Eq)]
struct CellScore<O: Ord + PartialEq + PartialOrd + Eq>(O, GridPosition);

#[derive(Eq, Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Hash)]
enum RelativeDirection {
    Forward,
    Left,
    Right
}

impl<O: PartialEq + Ord> PartialEq for CellScore<O> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<O: PartialOrd + Ord> PartialOrd for CellScore<O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<O: Ord> Ord for CellScore<O> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

pub fn a_star_pathfinding(
    start: GridPosition,
    goal: GridPosition,
    get_neighbors: impl Fn(GridPosition, usize) -> Vec<GridPosition>,
    heuristic: impl Fn(GridPosition, GridPosition) -> usize,
) -> Option<Vec<GridPosition>> {
    use std::collections::{BinaryHeap, HashMap};

    // This is usually a max-heap, but CellScore has reverse ordering, so it's basically a min-heap.
    let mut open_set = BinaryHeap::new();

    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();

    g_score.insert(start, 0);
    f_score.insert(start, heuristic(start, goal));
    open_set.push(CellScore(f_score[&start], start));

    while let Some(CellScore(_, current)) = open_set.pop() {
        if current == goal {
            let mut path = vec![current];
            while let Some(&prev) = came_from.get(path.last().unwrap()) {
                path.push(prev);
            }
            path.reverse();
            return Some(path);
        }

        let tentative_g_score = g_score[&current] + 1;
        for neighbor in get_neighbors(current, tentative_g_score) {
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&usize::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);
                f_score.insert(neighbor, tentative_g_score + heuristic(neighbor, goal));
                if !open_set.iter().any(|&CellScore(_, pos)| pos == neighbor) {
                    open_set.push(CellScore(f_score[&neighbor], neighbor));
                }
            }
        }
    }

    None
}

pub fn a_star_diagnostic(
    start: GridPosition,
    facing: Direction,
    goal: GridPosition,
    grid: &Grid,
) -> AStarDiagnostic {
    use std::collections::{BinaryHeap, HashMap};
    let heuristic = base_heuristic;

    // This is usually a max-heap, but CellScore has reverse ordering, so it's basically a min-heap.
    let mut open_set = BinaryHeap::new();

    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    let mut rel_dirs = HashMap::new();
    let mut f_score = HashMap::new();
    let mut farthest = CellScore(0, start);

    g_score.insert(start, 0);
    rel_dirs.insert(start, HashSet::new());
    f_score.insert(start, heuristic(start, goal));
    open_set.push(CellScore(f_score[&start], start));

    for immediate_neighbor in [
        (RelativeDirection::Forward, facing),
        (RelativeDirection::Right, facing.right_of()),
        (RelativeDirection::Left, facing.left_of())
    ] {
        let (relative_direction, direction) = immediate_neighbor;
        if
            let Some(pos) = start.after_moved(direction) &&
            pos.is_empty(grid)
        {
            came_from.insert(pos, start);
            g_score.insert(pos, 1);
            rel_dirs.insert(pos, HashSet::new());
            rel_dirs.get_mut(&pos).unwrap().insert(relative_direction);
            f_score.insert(pos, 1 + heuristic(pos, goal));
            open_set.push(CellScore(f_score[&start], pos));
        }
    }

    let mut path_to_goal = None;

    while let Some(CellScore(_, current)) = open_set.pop() {
        if current == goal && path_to_goal.is_none() {
            let mut path = vec![current];
            while let Some(&prev) = came_from.get(path.last().unwrap()) {
                path.push(prev);
            }
            path.reverse();
            path_to_goal = Some(path);
            
            if grid.cell_is_not_empty(goal) {
                continue;
            }
        }

        let current_g_score = g_score[&current];

        if current_g_score > farthest.0 && current != goal {
            farthest = CellScore(current_g_score, current);
        }

        let tentative_g_score = current_g_score + 1;
        let mut neighbors = get_neighbors_or_goal_neighbor(current, goal, grid);
        let mut to_push = HashSet::new();
        for neighbor in neighbors.iter() {
            if let Some(new_dirs) = rel_dirs.get(&current).cloned() {
                let dirs = rel_dirs.entry(*neighbor).or_insert_with(HashSet::new);
                dirs.extend(new_dirs);
            }
            
            if tentative_g_score < g_score.get(neighbor).cloned().unwrap_or(usize::MAX) {
                came_from.insert(*neighbor, current);
                g_score.insert(*neighbor, tentative_g_score);
                f_score.insert(*neighbor, tentative_g_score + heuristic(*neighbor, goal));
                to_push.insert(*neighbor);
            }
        }
        neighbors.sort_by_key(|n| f_score.get(n));
        for neighbor in neighbors {
            if 
                to_push.contains(&neighbor) &&
                !open_set.iter().any(|&CellScore(_, pos)| pos == neighbor)
            {
                open_set.push(CellScore(f_score[&neighbor], neighbor));
            }
        }
    }

    let farthest_path = {
        let mut path = vec![farthest.1];
        while let Some(&prev) = came_from.get(path.last().unwrap()) {
            path.push(prev);
        }
        path.reverse();
        path
    };

    let (forward_area, left_area, right_area, forward_sum_distances, left_sum_distances, right_sum_distances) = {
        let mut forward_area = 0;
        let mut left_area = 0;
        let mut right_area = 0;

        let mut forward_sum_distances = 0;
        let mut left_sum_distances = 0;
        let mut right_sum_distances = 0;

        for (pos, dirs) in &rel_dirs {
            if dirs.contains(&RelativeDirection::Forward) {
                forward_area += 1;
                forward_sum_distances += g_score.get(pos).unwrap_or(&0);
            }
            if dirs.contains(&RelativeDirection::Left) {
                left_area += 1;
                left_sum_distances += g_score.get(pos).unwrap_or(&0);
            }
            if dirs.contains(&RelativeDirection::Right) {
                right_area += 1;
                right_sum_distances += g_score.get(pos).unwrap_or(&0);
            }
        }

        (forward_area, left_area, right_area, forward_sum_distances, left_sum_distances, right_sum_distances)
    };

    AStarDiagnostic {
        start,
        forward_area,
        left_area,
        right_area,
        forward_sum_distances,
        left_sum_distances,
        right_sum_distances,
        to_farthest_point: farthest_path,
        to_goal: path_to_goal,
        distances: g_score,
        came_from,
    }
}

#[derive(Clone)]
pub struct AStarDiagnostic {
    pub start: GridPosition,
    pub forward_area: usize,
    pub left_area: usize,
    pub right_area: usize,
    pub forward_sum_distances: usize,
    pub left_sum_distances: usize,
    pub right_sum_distances: usize,
    pub to_farthest_point: Vec<GridPosition>,
    pub to_goal: Option<Vec<GridPosition>>,
    pub distances: HashMap<GridPosition, usize>,
    pub came_from: HashMap<GridPosition, GridPosition>,
}

pub fn base_heuristic(pos: GridPosition, goal: GridPosition) -> usize {
    // Manhattan distance heuristic
    let (x1, y1): (usize, usize) = pos.into();
    let (x2, y2): (usize, usize) = goal.into();
    (x1 as isize - x2 as isize).unsigned_abs() + (y1 as isize - y2 as isize).unsigned_abs()
}

pub fn next_direction_from_path(next_pos: GridPosition, a_star: &AStarDiagnostic, game_state: &GameState) -> Option<Direction> {
    // Collect path, then grab second position

    let mut path = vec![next_pos];
    while let Some(&prev) = a_star.came_from.get(path.last().unwrap()) {
        path.push(prev);
    }
    path.reverse();
    if path.len() >= 2 {
        if
            game_state.settings.debug_mode &&
            path[0] != a_star.start
        {
            println!("Expected: {:?}, Actual: {:?}", path[0], a_star.start);
        }
        Some(direction_to(path[0], path[1]))
    } else {
        None
    }

}

pub fn get_neighbors(pos: GridPosition, grid: &Grid) -> Vec<GridPosition> {
    pos.neighbors()
        .filter(|neighbor| neighbor.is_empty(grid))
        .collect()
}

pub fn other_cant_block_filter(relevant_info: &RelevantInformation) -> impl Fn(&GridPosition, usize) -> bool {
    move |n, distance| {
        relevant_info.other_a_star.distances.get(n)
            .is_none_or(|&d| d > distance)
    }
}

pub fn other_likely_wont_block_filter(relevant_info: &RelevantInformation) -> impl Fn(&GridPosition, usize) -> bool {
    move |n, distance| {
        relevant_info.other_a_star.distances.get(n)
            .is_none_or(|&d|
                d > distance ||
                !(
                    (d <= 2) ||
                    (relevant_info.other_bot_skill.chases.is_confidently_higher_than(CHASE_THRESHOLD) && d <= 5) ||
                    (relevant_info.other_bot_skill.cuts_off.is_confidently_higher_than(CUTOFF_THRESHOLD) && d <= 10)
                )
            )
    }
}

fn get_neighbors_or_goal_neighbor(pos: GridPosition, goal: GridPosition, grid: &Grid) -> Vec<GridPosition> {
    pos.neighbors()
        .filter(|neighbor| neighbor.is_empty(grid) || *neighbor == goal)
        .collect()
}

fn pathfind(start: GridPosition, goal: GridPosition, grid: &Grid) -> Option<Vec<GridPosition>> {
    a_star_pathfinding(
        start,
        goal,
        |pos, _| get_neighbors_or_goal_neighbor(pos, goal, grid),
        base_heuristic
    )
}

fn shortest_distance(start: GridPosition, goal: GridPosition, grid: &Grid) -> Option<usize> {
    pathfind(start, goal, grid).map(|path| path.len() - 1)
}

// This is a modified A* algorithm that finds the farthest reachable point from the current position.
fn find_farthest_point(start: GridPosition, grid: &Grid, neighbor_predicate: &impl Fn(&GridPosition, usize) -> bool) -> CellScore<usize> {
    use std::collections::{BinaryHeap, HashMap};

    // This is usually a max-heap, but CellScore has reverse ordering, so it's basically a min-heap.
    let mut open_set = BinaryHeap::new();

    let mut g_score = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(CellScore(g_score[&start], start));
    let mut farthest = CellScore(0, start);

    while let Some(CellScore(score, current)) = open_set.pop() {
        if score > farthest.0 {
            farthest = CellScore(score, current);
        }

        for neighbor in get_neighbors(current, grid) {
            let tentative_g_score = g_score[&current] + 1;
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&usize::MAX) {
                if !neighbor_predicate(&neighbor, tentative_g_score) {
                    continue;
                }

                g_score.insert(neighbor, tentative_g_score);
                if !open_set.iter().any(|&CellScore(_, pos)| pos == neighbor) {
                    open_set.push(CellScore(tentative_g_score, neighbor));
                }
            }
        }
    }

    farthest
}

fn find_farthest_point_in_general(start: GridPosition, grid: &Grid) -> CellScore<usize> {
    find_farthest_point(start, grid, &|_, _| true)
}

// This is a modified A* algorithm that finds the total area reachable from a given position.
/// Returns (area, sum of distances to all reachable points)
fn count_area(start: GridPosition, game_state: &GameState, neighbor_predicate: &impl Fn(&GridPosition, usize) -> bool) -> (usize, usize) {
    use std::collections::{BinaryHeap, HashMap};
    let grid = game_state.current_grid();

    // This is usually a max-heap, but CellScore has reverse ordering, so it's basically a min-heap.
    let mut open_set = BinaryHeap::new();

    let mut g_score = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(CellScore(g_score[&start], start));

    while let Some(CellScore(_, current)) = open_set.pop() {
        for neighbor in get_neighbors(current, grid) {
            let tentative_g_score = g_score[&current] + 1;
            if !neighbor_predicate(&neighbor, tentative_g_score) {
                continue;
            }
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&usize::MAX) {
                g_score.insert(neighbor, tentative_g_score);
                if !open_set.iter().any(|&CellScore(_, pos)| pos == neighbor) {
                    open_set.push(CellScore(tentative_g_score, neighbor));
                }
            }
        }
    }

    (g_score.len(), g_score.values().sum())
}

// **** SOME OF THIS CODE COPIED FROM example_bot.rs!!! HOPE THIS ISNT CHEATING ****
trait JackBot {
    fn my_player_id(&self) -> PlayerId;

    fn not_instant_crash_directions(
        &self,
        game_state: &GameState,
    ) -> impl Iterator<Item = Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        Direction::all().filter(move |d| {
            my_pos
                .after_moved(*d)
                .filter(|p| p.is_empty(grid))
                .is_some()
        })
    }

    fn ideal_directions(&self, game_state: &GameState) -> impl Iterator<Item = Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        self.not_instant_crash_directions(game_state)
            .filter(move |d| {
                my_pos.after_moved(*d).is_some_and(|p| {
                    !p.borders_cell(grid, |cell| cell.is_players_head(self.my_player_id().other()))
                })
            })
    }

    fn ideal_non_hole_directions(&self, game_state: &GameState) -> impl Iterator<Item = Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        self.ideal_directions(game_state)
            .filter(move |d| {
                my_pos.after_moved(*d).is_some_and(|p| {
                    p.borders_cell(grid, |cell| cell.is_empty())
                })
            })
    }

    fn direction_to(&self, game_state: &GameState, next_pos: GridPosition) -> Direction {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        direction_to(my_pos, next_pos)
    }

    fn get_the_hell_out_of_dodge(
        &mut self,
        game_state: &GameState
    ) -> Option<Direction> {
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());
        let other_pos = grid.player_head_position(self.my_player_id().other());

        let farthest_point = find_farthest_point_in_general(other_pos, grid).1;

        a_star_pathfinding(
            my_pos,
            farthest_point,
            // Pathfind around their head
            |pos, _| get_neighbors(pos, grid)
                .iter()
                .filter(|n| 
                    !n.borders_cell(grid, |cell|
                        cell.is_players_head(self.my_player_id().other())
                    )
                )
                .cloned()
                .collect(),
            base_heuristic
        )
            .and_then(|path| path.into_iter().nth(1))
            .map(|next_pos| self.direction_to(game_state, next_pos))
            .or_else(|| {
                pathfind(my_pos, farthest_point, grid)
                    .and_then(|path| path.into_iter().nth(1))
                    .map(|next_pos| self.direction_to(game_state, next_pos))
            })
            .or_else(|| {
                pathfind(my_pos, find_farthest_point_in_general(my_pos, grid).1, grid)
                    .and_then(|path| path.into_iter().nth(1))
                    .map(|next_pos| self.direction_to(game_state, next_pos))
            })
    }

    fn move_to_most_open_space(
        &mut self,
        relevant_info: &RelevantInformation
    ) -> Option<Direction> {
        let game_state = relevant_info.game_state;
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        if relevant_info.game_state.settings.debug_mode {
            println!("JackBot: Moving to most open space!");
        }

        // For close-quarters conflicts, use the big guns.
        if
            let Some(path_to_other) = &relevant_info.my_a_star.to_goal &&
            path_to_other.len() <= 10 &&
            let Some(direction) = my_pos.neighbors_with_direction()
                .filter(|(_, n)| n.is_empty(grid))
                .filter(|(_, n)| other_likely_wont_block_filter(relevant_info)(n, 1))
                .max_by_key(|(_, n)| {
                    let (area, sum_distances) = count_area(*n, game_state, &|p, d| other_likely_wont_block_filter(relevant_info)(p, d + 1));
                    (area, usize::MAX - sum_distances)
                })
                .map(|(d, _)| d)
        {
            return Some(direction);
        }

        // Otherwise, fall back to the faster precomputed method.
        [
            (RelativeDirection::Forward, grid.player_head_direction(self.my_player_id())),
            (RelativeDirection::Left, grid.player_head_direction(self.my_player_id()).left_of()),
            (RelativeDirection::Right, grid.player_head_direction(self.my_player_id()).right_of()),
        ]
            .iter()
            .flat_map(|(r, d)| my_pos.after_moved(*d).map(|n| (r, n, *d)))
            .filter(|(_, n, _)| n.is_empty(grid))
            .map(|(r, _, d)| {
                let (area, sum_distances) = match r {
                    RelativeDirection::Forward => (relevant_info.my_a_star.forward_area, relevant_info.my_a_star.forward_sum_distances),
                    RelativeDirection::Left => (relevant_info.my_a_star.left_area, relevant_info.my_a_star.left_sum_distances),
                    RelativeDirection::Right => (relevant_info.my_a_star.right_area, relevant_info.my_a_star.right_sum_distances)
                };
                (d, area, sum_distances)
            })
            .max_by_key(|(_, a, s)| (*a, usize::MAX - s))
            .map(|(d, _, _)| d)
            // I've literally never seen anything past this point get hit,
            // but it's here just in case everything above fails.
            .or_else(|| {
                my_pos.neighbors_with_direction()
                    .filter(|(_, n)| n.is_empty(grid))
                    .filter(|(_, n)| other_likely_wont_block_filter(relevant_info)(n, 1))
                    .max_by_key(|(_, n)| {
                        let (area, sum_distances) = count_area(*n, game_state, &|p, d| other_likely_wont_block_filter(relevant_info)(p, d + 1));
                        (area, usize::MAX - sum_distances)
                    })
                    .map(|(d, _)| d)
            })
            .or_else(|| {
                my_pos.neighbors_with_direction()
                    .filter(|(_, n)| n.is_empty(grid))
                    .max_by_key(|(_, n)| {
                        let (area, sum_distances) = count_area(*n, game_state, &|p, d| other_likely_wont_block_filter(relevant_info)(p, d + 1));
                        (area, usize::MAX - sum_distances)
                    })
                    .map(|(d, _)| d)
            })
            .or_else(|| {
                if game_state.settings.debug_mode {
                    println!("Jackbot: How did we get here?");
                }
                pathfind(
                    my_pos, 
                    find_farthest_point(my_pos, grid, &other_cant_block_filter(relevant_info)).1,
                    grid
                )
                    .and_then(|path| path.into_iter().nth(1))
                    .map(|next_pos| self.direction_to(game_state, next_pos))
            })
            .or_else(|| {
                relevant_info.my_a_star.to_farthest_point
                    .get(1)
                    .map(|next_pos| self.direction_to(game_state, *next_pos))
            })
    }

    fn fill_space(&mut self, relevant_info: &RelevantInformation) -> Option<Direction> {
        let game_state = relevant_info.game_state;
        let grid = game_state.current_grid();
        let my_pos = grid.player_head_position(self.my_player_id());

        let direction = game_state.current_grid().player_head_direction(self.my_player_id());

        let available_directions = self.not_instant_crash_directions(game_state).collect::<Vec<_>>();

        let can_go_right = available_directions.contains(&direction.right_of());
        let can_go_forward = available_directions.contains(&direction);
        let can_go_left = available_directions.contains(&direction.left_of());

        let front_left_open = my_pos.after_moved(direction)
            .and_then(|p| p.after_moved(direction.left_of()))
            .is_some_and(|p| p.is_empty(grid));
        let front_right_open = my_pos.after_moved(direction)
            .and_then(|p| p.after_moved(direction.right_of()))
            .is_some_and(|p| p.is_empty(grid));

        match (can_go_left, can_go_forward, can_go_right) {
            (false, false, false) => None,
            (false, false, true) => Some(direction.right_of()),
            (false, true, false) => Some(direction),
            (false, true, true) => {
                if front_right_open {
                    // If we're not about close off a path, keep following the right wall.
                    Some(direction.right_of())
                } else {
                    self.move_to_most_open_space(relevant_info)
                }
            },
            (true, false, false) => Some(direction.left_of()),
            (true, false, true) => self.move_to_most_open_space(relevant_info),
            (true, true, false) => {
                if front_left_open {
                    // If we're not about close off a path, keep moving along.
                    Some(direction)
                } else {
                    self.move_to_most_open_space(relevant_info)
                }
            },
            (true, true, true) => {
                match (front_left_open, front_right_open) {
                    (false, false) => self.move_to_most_open_space(relevant_info),
                    (false, true) => {
                        // We only want to turn left or right, since going forward would be equivalent to 
                        // going right, but without hugging the right wall.

                        // Determine whether to turn left or right by looking one step further in each direction and seeing if it's open.
                        // This should automatically exclude the other path since our head is blocking it.
                        let right = my_pos.after_moved(direction.right_of()).unwrap();
                        let left = my_pos.after_moved(direction.left_of()).unwrap();

                        if find_farthest_point_in_general(right, grid).0 > find_farthest_point_in_general(left, grid).0 {
                            Some(direction.right_of())
                        } else {
                            Some(direction.left_of())
                        }
                    },
                    (true, false) => {
                        // See above---same logic but for the left side.
                        let forward = my_pos.after_moved(direction).unwrap();
                        let right = my_pos.after_moved(direction.right_of()).unwrap();

                        if find_farthest_point_in_general(forward, grid).0 > find_farthest_point_in_general(right, grid).0 {
                            Some(direction)
                        } else {
                            Some(direction.right_of())
                        }
                    },
                    (true, true) => Some(direction.right_of())
                }
            },
        }
    }

    fn dont_cut_ourselves_off(
        &mut self,
        relevant_info: &RelevantInformation
    ) -> Option<Direction> {
        let grid = relevant_info.game_state.current_grid();
        let my_pos = relevant_info.my_a_star.start;

        let direction = relevant_info.game_state.current_grid().player_head_direction(self.my_player_id());

        let available_directions = self.ideal_non_hole_directions(relevant_info.game_state).collect::<Vec<_>>();

        let can_go_right = available_directions.contains(&direction.right_of());
        let can_go_forward = available_directions.contains(&direction);
        let can_go_left = available_directions.contains(&direction.left_of());

        let front_left_open = my_pos.after_moved(direction)
            .and_then(|p| p.after_moved(direction.left_of()))
            .is_some_and(|p| p.is_empty(grid));
        let front_right_open = my_pos.after_moved(direction)
            .and_then(|p| p.after_moved(direction.right_of()))
            .is_some_and(|p| p.is_empty(grid));

        let debug_mode = relevant_info.game_state.settings.debug_mode;

        let result = match (can_go_left, can_go_forward, can_go_right) {
            (false, true, true) if !front_right_open => self.move_to_most_open_space(relevant_info),
            (true, false, true) => self.move_to_most_open_space(relevant_info),
            (true, true, false) if !front_left_open => self.move_to_most_open_space(relevant_info),
            (true, true, true) => {
                match (front_left_open, front_right_open) {
                    (false, false) => self.move_to_most_open_space(relevant_info),
                    (false, true) => self.move_to_most_open_space(relevant_info),
                    (true, false) => self.move_to_most_open_space(relevant_info),
                    _ => None
                }
            },
            _ => None
        };

        if result.is_some() && debug_mode {
            println!("JackBot: Trying not to cut ourselves off!");
        }

        result
    }

    fn try_not_to_be_cut_off(&mut self, relevant_info: &RelevantInformation) -> Option<Direction> {
        if relevant_info.my_a_star.to_goal.as_ref().map(|p| p.len() - 1).is_none_or(|d| d > 3) && !relevant_info.other_bot_skill.chases.is_confidently_higher_than(CHASE_THRESHOLD) {
            return None;
        }
        if relevant_info.game_state.settings.debug_mode {
            println!("JackBot: Trying not to get cut off!");
        }

        relevant_info.my_a_star.to_farthest_point
            .iter()
            .enumerate()
            .map(|(my_distance, pos)| (pos, my_distance, relevant_info.other_a_star.distances.get(pos).unwrap_or(&usize::MAX)))
            .min_by_key(|&(_, _, distance)| distance)
            .and_then(|(next_pos, my_distance_to_my_cutoff, &other_distance_to_my_cutoff)| {
                if 
                    // Check if we are more capable of cutting them off than vice versa
                    !relevant_info.other_a_star.to_farthest_point
                        .iter()
                        .enumerate()
                        .map(|(other_distance, pos)| (pos, other_distance, relevant_info.my_a_star.distances.get(pos).unwrap_or(&usize::MAX)))
                        .min_by_key(|&(_, _, distance)| distance)
                        .is_some_and(|(_, other_distance_to_their_cutoff, &my_distance_to_their_cutoff)| 
                            my_distance_to_their_cutoff < other_distance_to_their_cutoff && 
                            (my_distance_to_their_cutoff <= other_distance_to_my_cutoff && relevant_info.other_a_star.to_farthest_point.len() > relevant_info.my_a_star.to_farthest_point.len()) ||
                            (my_distance_to_their_cutoff < other_distance_to_my_cutoff && relevant_info.other_a_star.to_farthest_point.len() >= relevant_info.my_a_star.to_farthest_point.len())
                        ) &&
                    (
                        (other_distance_to_my_cutoff <= my_distance_to_my_cutoff + 2 && relevant_info.other_bot_skill.cuts_off.is_confidently_higher_than(CUTOFF_THRESHOLD)) ||
                        (other_distance_to_my_cutoff <= my_distance_to_my_cutoff) ||
                        (other_distance_to_my_cutoff == 1)
                    )
                {
                    if relevant_info.game_state.settings.debug_mode {
                        println!("JackBot: They can cut us off!");
                    }

                    self.move_to_most_open_space(relevant_info)
                } else if other_distance_to_my_cutoff == my_distance_to_my_cutoff + 1 && relevant_info.other_bot_skill.cuts_off.is_confidently_higher_than(CUTOFF_THRESHOLD) {
                    if relevant_info.game_state.settings.debug_mode {
                        println!("JackBot: Oh, no you dont!");
                    }

                    next_direction_from_path(*next_pos, relevant_info.my_a_star, relevant_info.game_state)
                } else {
                    // They can't (or won't) cut us off...
                    // SO WE DON'T CARE!

                    None
                }
            })
    }

    fn estimate_other_bot_skill(&mut self, relevant_info: &mut RelevantInformation) {
        let RelevantInformation {
            game_state,
            other_bot_skill,
            my_a_star,
            other_a_star
        } = relevant_info;

        let frame_time = game_state.current_time();

        if frame_time == 0 {
            return;
        }

        let Some(other_last_a_star) = other_bot_skill.previous_diagnostic.as_ref() else {
            return;
        };

        let mut positive_chase_check = false;
        
        let Estimation { cases_checked, cases_matched } = &mut other_bot_skill.chases;
        {
            // Check if their distance to us increased when they had the opportunity
            if 
                let Some(my_last_pos) = other_last_a_star.to_goal.as_ref().and_then(|p| p.last()) &&
                let Some(dist_to_me) = other_last_a_star.distances.get(my_last_pos) &&
                let Some(new_dist_to_me) = other_a_star.distances.get(&my_a_star.start)
            {
                *cases_checked += 1;

                if *dist_to_me > *new_dist_to_me {
                    *cases_matched += 1;
                    positive_chase_check = true;
                }
            }

            if game_state.settings.debug_mode {
                println!("{} chases?\t{}", self.my_player_id().other(), other_bot_skill.chases);
            }
        }

        let Estimation { cases_checked, cases_matched } = &mut other_bot_skill.cuts_off;
        {
            // See if they're trying to cut us off regularly
            if 
                !positive_chase_check &&
                let Some(next_pos) = my_a_star.to_farthest_point
                    .iter()
                    .enumerate()
                    .filter_map(|(my_distance, &pos)| {
                        let other_distance = other_last_a_star.distances.get(&pos).copied().unwrap_or(usize::MAX);

                        if other_distance < my_distance {
                            Some((pos, my_distance, other_distance))
                        } else {
                            None
                        }
                    })
                    .min_by_key(|&(_, my_distance, other_distance)| (other_distance, my_distance))
                    .map(|(pos, _, _)| pos) && 
                let Some(dist_to_next_pos) = other_last_a_star.distances.get(&next_pos) &&
                let Some(new_dist_to_next_pos) = other_a_star.distances.get(&next_pos)
            {
                *cases_checked += 1;
                if *dist_to_next_pos >= *new_dist_to_next_pos {
                    *cases_matched += 1;
                }
            };

            if game_state.settings.debug_mode {
                println!("{} cuts off?\t{}", self.my_player_id().other(), other_bot_skill.cuts_off);
            }
        }
    }
}

pub struct RelevantInformation<'a> {
    game_state: &'a GameState,
    other_bot_skill: &'a mut SkillEstimate,
    my_a_star: &'a AStarDiagnostic,
    other_a_star: &'a AStarDiagnostic
}

#[derive(Clone)]
pub struct SkillEstimate {
    chases: Estimation,
    cuts_off: Estimation,
    previous_diagnostic: Rc<Option<AStarDiagnostic>>,
}

impl SkillEstimate {
    fn new() -> Self {
        Self {
            chases: Estimation { cases_checked: 0, cases_matched: 0 },
            cuts_off: Estimation { cases_checked: 0, cases_matched: 0 },
            previous_diagnostic: Rc::new(None),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Estimation {
    cases_checked: usize,
    cases_matched: usize
}

pub const CUTOFF_THRESHOLD: f32 = 0.40;
pub const CHASE_THRESHOLD: f32 = 0.40;

impl Estimation {
    pub fn percentage(&self) -> f32 {
        if self.cases_checked == 0 {
            return 0.0;
        }
        self.cases_matched as f32 / self.cases_checked as f32
    }

    // Wilson score interval for a Bernoulli parameter, with confidence level 99%.
    // Returns (lower_bound, upper_bound).
    pub fn confidence_interval(&self) -> (f32, f32) {
        if self.cases_checked == 0 {
            return (0.0, 1.0);
        }
        let z = 2.575; // 99% confidence
        let p = self.percentage();

        let zso2n = z * z / (2.0 * self.cases_checked as f32);
        let zso4ns = z * z / (4.0 * self.cases_checked as f32 * self.cases_checked as f32);

        let divisor = 1.0 + 2.0 * zso2n;

        (
            (p + zso2n - z * f32::sqrt((p * (1.0 - p) + zso4ns) / self.cases_checked as f32)) / divisor,
            (p + zso2n + z * f32::sqrt((p * (1.0 - p) + zso4ns) / self.cases_checked as f32)) / divisor
        )
    }

    pub fn is_confidently_higher_than(&self, percentage: f32) -> bool {
        self.confidence_interval().0 > percentage
    }

    pub fn is_maybe_higher_than(&self, percentage: f32) -> bool {
        self.confidence_interval().1 > percentage
    }

    pub fn is_confidently_lower_than(&self, percentage: f32) -> bool {
        self.confidence_interval().1 < percentage
    }

    pub fn is_maybe_lower_than(&self, percentage: f32) -> bool {
        self.confidence_interval().0 < percentage
    }
}

impl Display for Estimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{: >6.2}% - {: >6.2}%  ({})", self.confidence_interval().0 * 100.0, self.confidence_interval().1 * 100.0, self.cases_checked)
    }
}

fn direction_to(start: GridPosition, next_pos: GridPosition) -> Direction {
    let (head_x, head_y): (usize, usize) = start.into();
    let (next_x, next_y): (usize, usize) = next_pos.into();

    match (
        next_x as isize - head_x as isize,
        next_y as isize - head_y as isize
    ) {
        (0, 1) => Direction::PositiveY,
        (0, -1) => Direction::NegativeY,
        (1, 0) => Direction::PositiveX,
        (-1, 0) => Direction::NegativeX,
        _ => {
            println!("JackBot: next position is not adjacent to head! This should never happen. Defaulting to moving right.");
            Direction::NegativeY
        },
    }
}