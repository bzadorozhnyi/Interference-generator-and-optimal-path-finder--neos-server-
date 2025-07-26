pub mod cell;
pub mod path;

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    consts::COLORS,
    error::AppError,
    field::{
        cell::CellType,
        path::{parse_neos_output, Path},
    },
};
use cell::Cell;
use eframe::egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

pub struct Field {
    pub width: usize,
    pub height: usize,
    cell_size: f32,
    pub filled_cells: HashMap<Cell, CellType>,
    pub pink_pair_map: HashMap<Cell, Cell>,
    pub start_cell: Option<Cell>,
    pub end_cell: Option<Cell>,
    pub paths: Option<Vec<Path>>,
    pub line_segment_start: Option<Pos2>,
    response: Option<Response>,
    painter: Option<Painter>,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            width: 40,
            height: 20,
            cell_size: 20.0,
            filled_cells: HashMap::new(),
            pink_pair_map: HashMap::new(),
            start_cell: None,
            end_cell: None,
            paths: None,
            line_segment_start: None,
            response: None,
            painter: None,
        }
    }
}

impl Field {
    pub fn new() -> Self {
        Default::default()
    }

    fn response(&self) -> &Response {
        self.response
            .as_ref()
            .expect("Field::setup() must be called first!")
    }

    fn painter(&self) -> &Painter {
        self.painter
            .as_ref()
            .expect("Field::setup() must be called first!")
    }

    pub fn painter_rect(&self) -> Rect {
        self.painter().clip_rect()
    }

    pub fn pixels_per_point(&self) -> f32 {
        self.painter().pixels_per_point()
    }

    pub fn setup(&mut self, ui: &mut Ui) {
        let desired_size = Vec2::new(
            self.width as f32 * self.cell_size,
            self.height as f32 * self.cell_size,
        );

        let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());

        self.response = Some(response);
        self.painter = Some(painter);
    }

    pub fn area_height(&self) -> f32 {
        self.height as f32 * self.cell_size
    }

    fn path_from_links(&self, links: Vec<(Cell, Cell)>, id: usize) -> Result<Path, AppError> {
        let mut current_cell = match self.start_cell {
            Some(cell) => cell,
            None => return Err(AppError::StartNotSet),
        };

        let end_cell = match self.end_cell {
            Some(cell) => cell,
            None => return Err(AppError::EndNotSet),
        };

        let data: BTreeMap<Cell, Cell> = links.into_iter().collect();
        let mut path = Vec::new();

        while current_cell != end_cell {
            path.push(current_cell);
            current_cell = *data.get(&current_cell).ok_or(AppError::InvalidPath)?;
        }
        path.push(end_cell);

        Ok(Path::new(path, id))
    }

    pub fn parse_all_paths(&mut self, output: &str) -> Result<(), AppError> {
        let (_, links) = parse_neos_output(output).map_err(|_| AppError::InvalidPath)?;

        let paths: Result<Vec<Path>, AppError> = links
            .into_iter()
            .enumerate()
            .map(|(index, links)| self.path_from_links(links, index))
            .collect::<Vec<_>>()
            .into_iter()
            .collect();

        self.paths = Some(paths?);

        Ok(())
    }

    pub fn draw_path(&self, path: &Path) {
        path.cells().windows(2).for_each(|w| {
            let [prev_cell, next_cell] = w else { return };
            let color = COLORS[path.id % COLORS.len()];
            self.painter().line(
                vec![self.cell2pos2(prev_cell), self.cell2pos2(next_cell)],
                Stroke::new(4.0, color),
            );
        });
    }

    fn cell2pos2(&self, cell: &Cell) -> Pos2 {
        let field_rect = self.response().rect;

        Pos2::new(
            field_rect.left() + (cell.x - 1) as f32 * self.cell_size + self.cell_size / 2.0,
            field_rect.top() + (cell.y - 1) as f32 * self.cell_size + self.cell_size / 2.0,
        )
    }

    pub fn draw(&self) {
        self.draw_field();

        self.draw_paths();

        self.draw_endpoint(&self.start_cell, "S", Color32::RED);
        self.draw_endpoint(&self.end_cell, "T", Color32::ORANGE);

        self.draw_hovered_cell();
    }

    fn draw_field(&self) {
        for x in 1..=self.width {
            for y in 1..=self.height {
                let current_cell = Cell::new(x, y);

                let color = match self.filled_cells.get(&current_cell) {
                    Some(cell_type) => match cell_type {
                        CellType::Green => Color32::DARK_GREEN,
                        CellType::Pink => Color32::from_rgb(255, 64, 255),
                    },
                    None => Color32::LIGHT_GRAY,
                };

                self.painter().rect(
                    self.cell_rect(&current_cell),
                    0.0,
                    color,
                    Stroke::new(1.0, Color32::GRAY),
                    eframe::egui::StrokeKind::Inside,
                );
            }
        }
    }

    fn draw_paths(&self) {
        if let Some(paths) = &self.paths {
            for path in paths {
                self.draw_path(path);
            }
        }
    }

    fn draw_endpoint(&self, cell: &Option<Cell>, label: &str, color: Color32) {
        if let Some(cell) = cell {
            let end_pos = self.cell2pos2(cell);
            self.painter()
                .circle(end_pos, self.cell_size / 2.0, color, Stroke::NONE);
            self.painter().text(
                end_pos,
                Align2::CENTER_CENTER,
                label,
                FontId::default(),
                Color32::BLACK,
            );
        }
    }

    fn draw_hovered_cell(&self) {
        if let Some(cell) = self.hovered_cell() {
            self.painter().rect(
                self.cell_rect(&cell),
                0.0,
                Color32::TRANSPARENT,
                Stroke::new(2.0, Color32::BLUE),
                eframe::egui::StrokeKind::Middle,
            );
        }
    }

    fn cell_rect(&self, cell: &Cell) -> Rect {
        let field_rect = self.response().rect;

        let cell_min = Pos2::new(
            field_rect.left() + (cell.x - 1) as f32 * self.cell_size,
            field_rect.top() + (cell.y - 1) as f32 * self.cell_size,
        );
        let cell_max = Pos2::new(cell_min.x + self.cell_size, cell_min.y + self.cell_size);

        Rect::from_min_max(cell_min, cell_max)
    }

    fn pos2cell(&self, pos: Option<Pos2>) -> Option<Cell> {
        match pos {
            Some(pos) => {
                let field_rect = self.response().rect;

                let x = ((pos.x - field_rect.left()) / self.cell_size).floor() as usize + 1;
                let y = ((pos.y - field_rect.top()) / self.cell_size).floor() as usize + 1;

                if x <= self.width && y <= self.height {
                    Some(Cell::new(x, y))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn pointer_click_pos(&self) -> Option<Pos2> {
        self.response().interact_pointer_pos()
    }

    fn clicked_cell(&self) -> Option<Cell> {
        self.pos2cell(self.pointer_click_pos())
    }

    pub fn hovered_cell(&self) -> Option<Cell> {
        self.pos2cell(self.response().hover_pos())
    }

    fn is_endpoint(&self, cell: &Cell) -> bool {
        self.start_cell == Some(*cell)
    }

    fn bresenham_cells(&self, mut start_cell: Cell, end_cell: Cell) -> HashSet<Cell> {
        let dx = (end_cell.x as i32 - start_cell.x as i32).abs();
        let dy = -(end_cell.y as i32 - start_cell.y as i32).abs();

        let sx: i32 = if start_cell.x < end_cell.x { 1 } else { -1 };
        let sy: i32 = if start_cell.y < end_cell.y { 1 } else { -1 };

        let mut err = dx + dy;

        let mut cells = HashSet::new();
        if !self.is_endpoint(&start_cell) {
            cells.insert(start_cell);
        }

        while start_cell != end_cell {
            let err2 = 2 * err;

            if err2 >= dy {
                err += dy;
                start_cell.x = (start_cell.x as i32 + sx) as usize;
            }

            if err2 <= dx {
                err += dx;
                start_cell.y = (start_cell.y as i32 + sy) as usize;
            }

            if !self.is_endpoint(&start_cell) {
                cells.insert(start_cell);
            }
        }

        cells
    }

    pub fn handle_adding_cells(&mut self) {
        if let (Some(start_cell), Some(end_cell)) = (
            self.pos2cell(self.line_segment_start),
            self.pos2cell(self.pointer_click_pos()),
        ) {
            let cells_touched_by_line = self.bresenham_cells(start_cell, end_cell);
            self.filled_cells.extend(
                cells_touched_by_line
                    .into_iter()
                    .map(|c| (c, CellType::Green)),
            );
        }
        self.line_segment_start = self.pointer_click_pos();
    }

    pub fn handle_removing_cells(&mut self) {
        if let (Some(start_cell), Some(end_cell)) = (
            self.pos2cell(self.line_segment_start),
            self.pos2cell(self.pointer_click_pos()),
        ) {
            let cells_touched_by_line = self.bresenham_cells(start_cell, end_cell);
            self.filled_cells.retain(|x, cell_type| {
                !cells_touched_by_line.contains(x) || *cell_type == CellType::Pink
            });
        }
        self.line_segment_start = self.pointer_click_pos();
    }

    pub fn handle_start_cell_selection(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            if !self.is_cell_occupied(&cell) {
                self.start_cell = Some(cell)
            }
        }
    }

    pub fn handle_end_cell_selection(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            if !self.is_cell_occupied(&cell) {
                self.end_cell = Some(cell)
            }
        }
    }

    pub fn clear_paths(&mut self) {
        self.paths = None;
    }

    pub fn is_cell_occupied(&self, cell: &Cell) -> bool {
        self.filled_cells.contains_key(cell)
    }

    pub fn is_green_cell(&self, cell: &Cell) -> bool {
        let cell_type = self.filled_cells.get(cell);

        match cell_type {
            Some(ct) => ct == &CellType::Green,
            None => false,
        }
    }

    pub fn is_pink_cell(&self, cell: &Cell) -> bool {
        let cell_type = self.filled_cells.get(cell);

        match cell_type {
            Some(ct) => ct == &CellType::Pink,
            None => false,
        }
    }

    pub fn contains(&self, cell: &Cell) -> bool {
        1 <= cell.x && cell.x <= self.width && 1 <= cell.y && cell.y <= self.height
    }

    /// If the given cell and its diagonal opposite form a valid pink pattern,
    /// returns a pair (current_cell, diagonal_cell). Otherwise, returns `None`.
    pub fn find_pink_diagonal_match(&self, cell: &Cell) -> Option<(Cell, Cell)> {
        if self.is_cell_occupied(cell) {
            return None;
        }

        // Case 1: Top-left P, green right and down → bottom-right
        let right = Cell::new(cell.x + 1, cell.y);
        let down = Cell::new(cell.x, cell.y + 1);
        let opposite = Cell::new(cell.x + 1, cell.y + 1);

        if self.contains(&right)
            && self.is_green_cell(&right)
            && self.contains(&down)
            && self.is_green_cell(&down)
            && self.contains(&opposite)
            && !self.is_cell_occupied(&opposite)
        {
            return Some((*cell, opposite));
        }

        // Case 2: Bottom-right P, green up and left → top-left
        let left = Cell::new(cell.x - 1, cell.y);
        let up = Cell::new(cell.x, cell.y - 1);
        let opposite = Cell::new(cell.x - 1, cell.y - 1);

        if self.contains(&left)
            && self.is_green_cell(&left)
            && self.contains(&up)
            && self.is_green_cell(&up)
            && self.contains(&opposite)
            && !self.is_cell_occupied(&opposite)
        {
            return Some((*cell, opposite));
        }

        // Case 3: Top-right P, green down and left → bottom-left
        let left = Cell::new(cell.x - 1, cell.y);
        let down = Cell::new(cell.x, cell.y + 1);
        let opposite = Cell::new(cell.x - 1, cell.y + 1);

        if self.contains(&left)
            && self.is_green_cell(&left)
            && self.contains(&down)
            && self.is_green_cell(&down)
            && self.contains(&opposite)
            && !self.is_cell_occupied(&opposite)
        {
            return Some((*cell, opposite));
        }

        // Case 4: Bottom-left P, green up and right → top-right
        let up = Cell::new(cell.x, cell.y - 1);
        let right = Cell::new(cell.x + 1, cell.y);
        let opposite = Cell::new(cell.x + 1, cell.y - 1);

        if self.contains(&up)
            && self.is_green_cell(&up)
            && self.contains(&right)
            && self.is_green_cell(&right)
            && self.contains(&opposite)
            && !self.is_cell_occupied(&opposite)
        {
            return Some((*cell, opposite));
        }

        None
    }

    pub fn handle_add_pink_pair_constraint(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            match self.find_pink_diagonal_match(&cell) {
                Some((c1, c2)) => {
                    self.filled_cells.insert(c1, CellType::Pink);
                    self.filled_cells.insert(c2, CellType::Pink);

                    self.pink_pair_map.insert(c1, c2);
                    self.pink_pair_map.insert(c2, c1);
                }
                None => {}
            }
        }
    }

    pub fn handle_remove_pink_pair_constraint(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            if self.is_pink_cell(&cell) {
                if let Some(pair) = self.pink_pair_map.remove(&cell) {
                    self.pink_pair_map.remove(&pair);

                    self.filled_cells.remove(&cell);
                    self.filled_cells.remove(&pair);
                }
            }
        }
    }

    pub fn unique_pink_pairs(&self) -> HashSet<(&Cell, &Cell)> {
        let mut set = HashSet::new();

        for (a, b) in self.pink_pair_map.iter() {
            let pair = if a <= b { (a, b) } else { (b, a) };
            set.insert(pair);
        }

        set
    }
}
