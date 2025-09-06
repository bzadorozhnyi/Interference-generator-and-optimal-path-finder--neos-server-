use eframe::egui::Color32;

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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum CellType {
    Green,
    Pink,
    Yellow,
    Orange,
}

impl CellType {
    pub fn variants() -> &'static [CellType] {
        &[
            CellType::Green,
            CellType::Pink,
            CellType::Yellow,
            CellType::Orange,
        ]
    }

    pub fn color(&self) -> Color32 {
        match self {
            CellType::Green => Color32::DARK_GREEN,
            CellType::Pink => Color32::from_rgb(255, 64, 255),
            CellType::Yellow => Color32::YELLOW,
            CellType::Orange => Color32::ORANGE,
        }
    }
}
