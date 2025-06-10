pub mod parser;

use crate::field::cell::Cell;

pub use parser::parse_neos_output;

#[derive(Debug)]
pub struct Path {
    cells: Vec<Cell>,
    pub id: usize,
}

impl Path {
    pub fn new(path: Vec<Cell>, id: usize) -> Self {
        Self { cells: path, id }
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
}
