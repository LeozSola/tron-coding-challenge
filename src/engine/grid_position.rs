use crate::engine::prelude::*;

/// Represents an in-bounds position on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition(usize);

impl GridPosition {
    /// Creates a new GridPosition from a usize index. Returns None if the 
    /// index is out of bounds.
    /// 
    /// Cells are indexed from 0 to GRID_SIZE * GRID_SIZE - 1, starting at the
    /// top left and going row by row.
    pub fn new_from_usize(i: usize) -> Option<GridPosition> {
        let out = GridPosition(i);
        if out.in_bounds() { Some(out) } else { None }
    }
    /// Creates a new GridPosition from x and y coordinates. Returns None if
    /// the coordinates are out of bounds.
    pub fn new(x: usize, y: usize) -> Option<GridPosition> {
        if x < GRID_SIZE && y < GRID_SIZE {
            Some(GridPosition(x + (y * GRID_SIZE)))
        } else {
            None
        }
    }
    fn in_bounds(&self) -> bool {
        self.0 < GRID_SIZE * GRID_SIZE
    }

    pub fn is_empty(&self, grid: &Grid) -> bool {
        grid.get_cell(*self).is_empty()
    }
    pub fn is_not_empty(&self, grid: &Grid) -> bool {
        grid.get_cell(*self).is_not_empty()
    }
    pub fn x(&self) -> usize {
        self.0 % GRID_SIZE
    }
    pub fn y(&self) -> usize {
        self.0 / GRID_SIZE
    }
    /// Returns the index of the cell corresponding to this position.
    /// The index is from 0 to GRID_SIZE * GRID_SIZE - 1, starting at the top 
    /// left and going row by row.
    pub fn i(&self) -> usize {
        self.0
    }
    pub fn get_cell<'a>(&self, grid: &'a Grid) -> &'a GridCell {
        grid.get_cell(*self)
    }

    /// Returns a boxed slice of all grid positions, in order of their index
    /// (starting at 0).
    pub fn all_slice() -> Box<[Self]> {
        Self::iter_positions().collect()
    }

    /// Returns the manhattan distance between self and other.
    /// 
    /// The manhattan distance is the number of steps it would take to get
    /// from self to other only moving up, down, left, or right.
    /// [https://en.wikipedia.org/wiki/Taxicab_geometry#Manhattan_distance]
    pub fn manhattan_distance(&self, other: &Self)->u8{
        (self.x() as i8 - other.x() as i8).unsigned_abs() + (self.y() as i8 - other.y() as i8).unsigned_abs()
    }
    /// Returns an iterator over all positions which neighbor this one
    pub fn neighbors(&self)->impl Iterator<Item = GridPosition>{
        Direction::all()
            .filter_map(|direction|self.after_moved(direction))
    }
    /// Returns an iterator over all positions which neighbor this one, along
    /// with the direction you would have to move to get there
    pub fn neighbors_with_direction(&self)->impl Iterator<Item = (Direction, GridPosition)>{
        Direction::all()
            .filter_map(|direction| self.after_moved(direction).map(|p| (direction, p)))
    }
    /// Returns a grid position that neighbors self and is in the direction
    /// given. None if out of bounds
    pub fn after_moved(&self, direction: Direction) -> Option<Self> {
        let (x, y) = self.into();
        match direction {
            Direction::PositiveY => GridPosition::new(x, y + 1),
            Direction::NegativeY => GridPosition::new(x, y.checked_sub(1)?),
            Direction::PositiveX => GridPosition::new(x + 1, y),
            Direction::NegativeX => GridPosition::new(x.checked_sub(1)?, y),
        }
    }
    /// Returns true if the given condition is met for any neighboring cell
    pub fn borders_cell<F: Fn(&GridCell) -> bool>(&self, grid: &Grid, condition: F) -> bool {
        Direction::all_slice()
            .iter()
            .map(|d| self.after_moved(*d))
            .any(|pos| {
                if let Some(pos) = pos {
                    condition(pos.get_cell(grid))
                } else {
                    false
                }
            })
    }
}

impl From<GridPosition> for (usize, usize) {
    fn from(value: GridPosition) -> Self {
        (value.0 % GRID_SIZE, value.0 / GRID_SIZE)
    }
}
impl From<&GridPosition> for (usize, usize) {
    fn from(value: &GridPosition) -> Self {
        (value.0 % GRID_SIZE, value.0 / GRID_SIZE)
    }
}

/// An error indicating that a grid position was out of bounds.
pub struct GridPositionOutOfBoundsError;

impl TryFrom<(usize, usize)> for GridPosition {
    type Error = GridPositionOutOfBoundsError;

    fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
        GridPosition::new(value.0, value.1).ok_or(GridPositionOutOfBoundsError)
    }
}

/// An iterator over all grid positions, in order of their index (starting at
/// 0).
pub struct GridPositionIterator(usize);

impl Iterator for GridPositionIterator {
    type Item = GridPosition;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 += 1;
        GridPosition::new_from_usize(self.0 - 1)
    }
}
impl GridPosition {
    pub fn iter_positions() -> GridPositionIterator {
        GridPositionIterator(0)
    }
}
