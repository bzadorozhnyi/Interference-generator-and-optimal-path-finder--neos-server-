#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, Ord, PartialEq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn to_zero_indexed(&mut self) {
        self.x -= 1;
        self.y -= 1;
    }
}
