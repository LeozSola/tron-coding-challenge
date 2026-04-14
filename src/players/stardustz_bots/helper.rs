use std::collections::HashSet;

use crate::engine::prelude::*;

pub trait PlayerIdFunctions{
    fn left_of(&self, grid: &Grid)->Direction;
    fn right_of(&self, grid: &Grid)->Direction;
}
impl PlayerIdFunctions for PlayerId {
    fn left_of(&self, grid: &Grid) -> Direction {
        let GridCell::Head(_, direction) = grid.player_head_position(*self).get_cell(grid) else {unreachable!()};
        *&direction.left_of()
    }
    fn right_of(&self, grid: &Grid) -> Direction {
        let GridCell::Head(_, direction) = grid.player_head_position(*self).get_cell(grid) else {unreachable!()};
        *&direction.right_of()
    }
}


pub trait PositionFunctions{
    fn blocked_side_count(&self, player: PlayerId, grid: &Grid, flood: &HashSet<GridPosition>)->u8;
    fn blocked_in_direction(&self, player: PlayerId, grid: &Grid, direction: Direction, flood: &HashSet<GridPosition>)->bool;
}
impl PositionFunctions for GridPosition{
    fn blocked_side_count(&self, player: PlayerId, grid: &Grid, flood: &HashSet<GridPosition>)->u8 {
        (if self.blocked_in_direction(player, grid, Direction::NegativeX, flood) {1} else {0}) + 
        (if self.blocked_in_direction(player, grid, Direction::NegativeY, flood) {1} else {0}) + 
        (if self.blocked_in_direction(player, grid, Direction::PositiveX, flood) {1} else {0}) + 
        (if self.blocked_in_direction(player, grid, Direction::PositiveY, flood) {1} else {0})
    }
    fn blocked_in_direction(&self, player: PlayerId, grid: &Grid, direction: Direction, flood: &HashSet<GridPosition>)->bool {
        if let Some(pos) = self.after_moved(direction) {
            if player.get_head_pos(grid) == pos {return false}
            if pos.is_empty(grid) && flood.contains(&pos) {false} else {true}
        }else{
            true
        }
    }
}

pub trait PositionIterator{
    fn filter_empty(self, grid: &Grid) -> impl Iterator<Item = GridPosition>;
}

impl<T: Iterator<Item = GridPosition>> PositionIterator for T{
    fn filter_empty(self, grid: &Grid) -> impl Iterator<Item = GridPosition> {
        self
            .filter(|pos|pos.is_empty(grid))
    }
}



pub trait DirectionIterator{
    fn filter_not_crash(self, player: PlayerId, grid: &Grid) -> impl Iterator<Item = Direction>;
    fn filter_not_crash_into_head(self, player: PlayerId, grid: &Grid) -> impl Iterator<Item = Direction>;
}

impl<T: Iterator<Item = Direction>> DirectionIterator for T{
    fn filter_not_crash(self, player: PlayerId, grid: &Grid) -> impl Iterator<Item = Direction> {
        self.filter(move |d|
            grid.player_head_position(player)
                .after_moved(*d)
                .filter(|p|p.is_empty(grid))
                .is_some()
        )
    }
    fn filter_not_crash_into_head(self, player: PlayerId, grid: &Grid) -> impl Iterator<Item = Direction> {
        self.filter_not_crash(player, grid)
            .filter(move |d| {
                grid.player_head_position(player)
                    .after_moved(*d)
                    .is_some_and(|p|
                        !p.borders_cell(
                            grid,
                            |cell|cell.is_players_head(player.other())
                        )
                    )
            })
    }
}

pub fn players_only_not_crash_direction(player: PlayerId, grid: &Grid)->Option<GridPosition>{
    let enemy_pos = player.get_head_pos(grid);

    let not_crash_dirs: Box<[Direction]> = Direction::all().filter_not_crash(player, grid).collect();

    if not_crash_dirs.len() == 1 {
        not_crash_dirs
            .iter()
            .next()
            .and_then(|d|enemy_pos.after_moved(*d))
    }else{
        None
    }
}