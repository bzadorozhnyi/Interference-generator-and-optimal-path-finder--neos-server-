use eframe::egui::{Color32, Response, Sense, Stroke, Ui, Vec2};

pub fn color_button(ui: &mut Ui, color: Color32, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());

    if ui.is_rect_visible(rect) {
        let stroke_color = if selected { Color32::WHITE } else { color };

        ui.painter().rect_filled(rect, 4.0, color);
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(2.0, stroke_color),
            eframe::egui::StrokeKind::Outside,
        );
    }

    response
}
