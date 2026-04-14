use std::collections::HashSet;

use crate::{
    engine::prelude::*,
    players::stardustz_bots::helper::{PlayerIdFunctions, PositionFunctions, PositionIterator}
};


/// This is just going to go up and down 
#[derive(Debug)]
pub struct SimpleSpaceFillBot{
    my_player_id: PlayerId,
    state: State
}

#[derive(Debug, Clone)]
enum State{
    GoToWall(Option<Direction>),
    HugLeftWall
}

impl Bot for SimpleSpaceFillBot{
    fn new(args: BotArgs)->Self {
        SimpleSpaceFillBot{
            my_player_id: args.my_player(),
            state: State::GoToWall(None)
        }
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        let my_pos = game_state.current_grid().player_head_position(self.my_player_id);
        let grid = game_state.current_grid();
        let flood_fill = Self::floodfill(self.my_player_id, my_pos, grid);

        match self.state.clone() {
            State::GoToWall(_) => self.go_to_wall(game_state, &flood_fill),
            State::HugLeftWall => self.hug_left_wall(game_state, &flood_fill),
        }
    }
}

impl SimpleSpaceFillBot{
    fn go_to_wall(&mut self, game_state: &GameState, flood_fill: &HashSet<GridPosition>) -> Direction {
        let grid = game_state.current_grid();

        self.state = State::GoToWall(Some(self.find_wall(game_state, flood_fill)));
        let State::GoToWall(Some(go_to_wall_direction)) = &self.state else {unreachable!()};

        let direction_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(*go_to_wall_direction)
            .and_then(|pos|Some(flood_fill.contains(&pos) && pos.is_empty(grid)))
            .unwrap_or(false);
        
        if direction_clear {
            *go_to_wall_direction
        }else{
            self.state = State::HugLeftWall;
            self.next_action(game_state)
        }
    }
    fn hug_left_wall(&self, game_state: &GameState, flood_fill: &HashSet<GridPosition>) -> Direction {
        let grid = game_state.current_grid();
        let State::HugLeftWall = &self.state else {unreachable!()};
        
        let left_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(self.my_player_id.left_of(grid))
            .and_then(|pos|Some(flood_fill.contains(&pos) && pos.is_empty(grid)))
            .unwrap_or(false);

        if left_clear {
            return self.my_player_id.get_head_direction(grid).left_of();
        }

        let forward_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(self.my_player_id.get_head_direction(grid))
            .and_then(|pos|Some(flood_fill.contains(&pos) && pos.is_empty(grid)))
            .unwrap_or(false);

        if forward_clear {
            return self.my_player_id.get_head_direction(grid);
        }
        
        let right_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(self.my_player_id.get_head_direction(grid).right_of())
            .and_then(|pos|Some(flood_fill.contains(&pos) && pos.is_empty(grid)))
            .unwrap_or(false);

        if right_clear {
            return self.my_player_id.get_head_direction(grid).right_of();
        }

        //now while ignoring the flood

        let left_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(self.my_player_id.left_of(grid))
            .and_then(|pos|Some(pos.is_empty(grid)))
            .unwrap_or(false);

        if left_clear {
            return self.my_player_id.get_head_direction(grid).left_of();
        }

        let forward_clear = self.my_player_id
            .get_head_pos(grid)
            .after_moved(self.my_player_id.get_head_direction(grid))
            .and_then(|pos|Some(pos.is_empty(grid)))
            .unwrap_or(false);

        if forward_clear {
            return self.my_player_id.get_head_direction(grid);
        }
        
        self.my_player_id.get_head_direction(grid).right_of()
    }

    fn find_wall(&mut self, game_state: &GameState, flood_fill: &HashSet<GridPosition>) -> Direction{
        let my_pos = game_state.current_grid().player_head_position(self.my_player_id);
        let direction = Self::nearest_wall(my_pos, flood_fill);

        direction.unwrap_or(PositiveY)
    }

    

    /// Doesnt fill in 1 wide gaps that i wont be able to get out of
    fn floodfill(player: PlayerId, from: GridPosition, grid: &Grid) -> HashSet<GridPosition> {
        
        let mut to_add_neighbors = HashSet::new();
        let mut visited = HashSet::new();
        
        to_add_neighbors.insert(from);
        loop{
            let Some(current) = to_add_neighbors.iter().next().cloned() else {break};
            let current = to_add_neighbors.take(&current).expect("We know its in there because we just got it from there");

            visited.insert(current);

            current
                .neighbors()
                .filter_empty(grid)
                .filter(|pos|!visited.contains(pos))
                .for_each(|pos|{
                    to_add_neighbors.insert(pos);
                });
        }
        

        //now remove cells with 3 walls
        loop {
            let old_len = visited.len();
            visited = visited
                .iter()
                .filter(|pos|pos.blocked_side_count(player, grid, &visited) < 3)
                .cloned()
                .collect();
            if visited.len() == old_len {
                break;
            }
        }
        visited
    }



    fn nearest_wall(from: GridPosition, positions: &HashSet<GridPosition>) -> Option<Direction> {
        let left_most_x = positions
            .iter()
            .min_by(|a,b|a.x().cmp(&b.x()))?;

        let right_most_x = positions
            .iter()
            .max_by(|a,b|a.x().cmp(&b.x()))?;

        let down_most_y = positions
            .iter()
            .min_by(|a,b|a.y().cmp(&b.y()))?;

        let up_most_y = positions
            .iter()
            .max_by(|a,b|a.y().cmp(&b.y()))?;

        Direction::all()
            .map(|direction|{
                (
                    direction,
                    match direction {
                        PositiveY => up_most_y.y() - from.y(),
                        NegativeY => from.y() - down_most_y.y(),
                        PositiveX => right_most_x.x() - from.x(),
                        NegativeX => from.x() - left_most_x.x(),
                    }
                )
            })
            .min_by(|(_, a),(_, b)|a.cmp(b))
            .map(|(direction,_)|direction)
    }
}