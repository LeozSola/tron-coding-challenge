use std::collections::{BinaryHeap, HashSet};

use crate::engine::prelude::*;

pub struct Astar<'grid>{
    open: BinaryHeap<SearchNode>,
    checked: HashSet<GridPosition>,
    grid: &'grid Grid,
    end: GridPosition,
}

#[derive(Eq, PartialEq, Debug)]
struct SearchNode{
    cost_plus_heuristic: u8,
    cost_from_start: u8,
    pos: GridPosition,
    first_step: Direction,
}
impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost_plus_heuristic.cmp(&self.cost_plus_heuristic)
    }
}
impl PartialOrd for SearchNode { 
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { 
        Some(self.cmp(other)) 
    }
}
impl<'grid> Astar<'grid>{
    pub fn a_star_direction(grid: &'grid Grid, start: GridPosition, end: GridPosition) -> Option<Direction> {
        if start == end {return None;}

        let mut a_star = Self::new(grid, end);

        start.neighbors_with_direction().for_each(|(dir, pos)|a_star.insert(1, pos, dir));
        a_star.checked.insert(start);

        while let Some(node) = a_star.open.pop() {
            if a_star.checked.contains(&node.pos) {continue;}
            if node.pos == end {return Some(node.first_step);}
            a_star.checked.insert(node.pos);

            node.pos.neighbors().for_each(|pos|a_star.insert(node.cost_from_start + 1, pos, node.first_step));
        }

        None
    }
    fn new(grid: &'grid Grid, end: GridPosition)->Self{
        Astar { open: BinaryHeap::new(), checked: HashSet::new(), grid, end }
    }
    fn pathfindable(&self, pos: GridPosition)->bool{
        pos.is_empty(self.grid) || pos.get_cell(self.grid).is_head()
    }
    fn insert(
        &mut self,
        cost_from_start: u8,
        pos: GridPosition,
        first_step: Direction
    ){
        if !self.checked.contains(&pos) && self.pathfindable(pos) {
            self.open.push(SearchNode {
                cost_from_start: cost_from_start,
                cost_plus_heuristic: cost_from_start + pos.manhattan_distance(&self.end),
                pos,
                first_step,
            });
        }
    }
}


