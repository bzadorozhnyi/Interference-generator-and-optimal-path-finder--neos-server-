pub mod cell;
pub mod path;

use std::collections::{BTreeMap, HashSet};

use crate::{
    consts::COLORS,
    error::AppError,
    field::path::{parse_neos_output, Path},
};
use cell::Cell;
use eframe::egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

pub struct Field {
    pub width: usize,
    pub height: usize,
    cell_size: f32,
    pub filled_cells: HashSet<Cell>,
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
            filled_cells: HashSet::new(),
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
        let (_, mut links) = parse_neos_output(output).map_err(|_| AppError::InvalidPath)?;

        for link in &mut links {
            for (a, b) in link {
                a.to_zero_indexed();
                b.to_zero_indexed();
            }
        }

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

        let x = field_rect.left() + cell.x as f32 * self.cell_size + self.cell_size / 2.0;
        let y = field_rect.top() + cell.y as f32 * self.cell_size + self.cell_size / 2.0;

        Pos2::new(x, y)
    }

    pub fn draw(&self) {
        self.draw_field();

        self.draw_paths();

        self.draw_endpoint(&self.start_cell, "S", Color32::RED);
        self.draw_endpoint(&self.end_cell, "T", Color32::ORANGE);

        self.draw_hovered_cell();
    }

    fn draw_field(&self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let current_cell = Cell::new(x, y);

                let color = if self.filled_cells.contains(&current_cell) {
                    Color32::DARK_GREEN
                } else {
                    Color32::LIGHT_GRAY
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
        let field_rect = self.response().rect;

        if let Some(cell) = cell {
            let end_pos = Pos2::new(
                field_rect.left() + cell.x as f32 * self.cell_size + self.cell_size / 2.0,
                field_rect.top() + cell.y as f32 * self.cell_size + self.cell_size / 2.0,
            );
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
            field_rect.left() + cell.x as f32 * self.cell_size,
            field_rect.top() + cell.y as f32 * self.cell_size,
        );
        let cell_max = Pos2::new(cell_min.x + self.cell_size, cell_min.y + self.cell_size);

        Rect::from_min_max(cell_min, cell_max)
    }

    fn pos2cell(&self, pos: Option<Pos2>) -> Option<Cell> {
        match pos {
            Some(pos) => {
                let field_rect = self.response().rect;

                let x = ((pos.x - field_rect.left()) / self.cell_size).floor() as usize;
                let y = ((pos.y - field_rect.top()) / self.cell_size).floor() as usize;

                if x < self.width && y < self.height {
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
            self.filled_cells.extend(cells_touched_by_line);
        }
        self.line_segment_start = self.pointer_click_pos();
    }

    pub fn handle_removing_cells(&mut self) {
        if let (Some(start_cell), Some(end_cell)) = (
            self.pos2cell(self.line_segment_start),
            self.pos2cell(self.pointer_click_pos()),
        ) {
            let cells_touched_by_line = self.bresenham_cells(start_cell, end_cell);
            self.filled_cells
                .retain(|x| !cells_touched_by_line.contains(x));
        }
        self.line_segment_start = self.pointer_click_pos();
    }

    pub fn handle_start_cell_selection(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            if !self.filled_cells.contains(&cell) {
                self.start_cell = Some(cell)
            }
        }
    }

    pub fn handle_end_cell_selection(&mut self) {
        if let Some(cell) = self.clicked_cell() {
            if !self.filled_cells.contains(&cell) {
                self.end_cell = Some(cell)
            }
        }
    }

    pub fn clear_paths(&mut self) {
        self.paths = None;
    }
}
