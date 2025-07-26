#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, Ord, PartialEq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum CellType {
    Green,
    Pink,
}
