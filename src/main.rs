use eframe::egui::{self, Color32};
use interference_generator::error::FieldError;
use interference_generator::{
    field::Field,
    toast::{Toast, ToastVariant},
};
use tera::Tera;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Interference generator",
        native_options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(PartialEq)]
#[non_exhaustive]
enum Mode {
    Draw,
    Erase,
    StartSelection,
    EndSelection,
}

#[derive(Debug, PartialEq)]
enum Template {
    Default,
    Eight,
    Disabled,
}

struct MyApp {
    field: Field,
    mode: Mode,
    template: Template,
    toast: Option<Toast>,
    input_path: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            field: Field::new(),
            mode: Mode::Draw,
            template: Template::Default,
            toast: None,
            input_path: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.selectable_value(&mut self.mode, Mode::Draw, "Draw");
                ui.selectable_value(&mut self.mode, Mode::Erase, "Erase");
                ui.selectable_value(&mut self.mode, Mode::StartSelection, "Start");
                ui.selectable_value(&mut self.mode, Mode::EndSelection, "Terminal");

                egui::ComboBox::from_label("Template")
                    .selected_text(self.template_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.template, Template::Default, "path");
                        ui.selectable_value(&mut self.template, Template::Eight, "path_8");
                        ui.selectable_value(
                            &mut self.template,
                            Template::Disabled,
                            "path_disabled",
                        );
                    });

                if ui.button("Generate file").clicked() {
                    self.generate_file();
                }

                if ui.button("Draw path").clicked() {
                    match self.field.parse_path(&self.input_path) {
                        Ok(_) => {}
                        Err(e) => match e {
                            FieldError::ParseStringError(message) => {
                                self.show_toast(&message, ToastVariant::Error)
                            }
                            FieldError::StartNotSet => {
                                self.show_toast("Start not set", ToastVariant::Error)
                            }
                            FieldError::EndNotSet => {
                                self.show_toast("End not set", ToastVariant::Error)
                            }
                            FieldError::InvalidPath => {
                                self.show_toast("Invalid path", ToastVariant::Error);
                            },
                        },
                    }
                }

                if ui.button("Clear path").clicked() {
                    self.field.clear_path();
                }
            });

            ui.add_space(20.0);

            ui.columns(2, |columns| {
                self.field.setup(&mut columns[0]);

                columns[1].text_edit_multiline(&mut self.input_path);
            });

            match self.field.hovered_cell() {
                Some(cell) => {
                    ui.label(format!("Cell: (x: {}, y: {})", cell.x + 1, cell.y + 1));
                }
                None => {
                    ui.label("Cell: None");
                }
            }

            self.field.draw_filled_cells();

            if let Some(toast) = &self.toast {
                egui::Area::new("toast_area".into())
                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
                    .show(ctx, |ui| {
                        ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                        let fill_color = match toast.variant {
                            ToastVariant::Error => Color32::DARK_RED,
                            ToastVariant::Success => Color32::DARK_GREEN,
                        };

                        egui::Frame::default()
                            .fill(fill_color)
                            .inner_margin(5.0)
                            .corner_radius(10.0)
                            .show(ui, |ui| ui.label(&toast.message))
                    });

                if toast.is_expired() {
                    self.toast = None;
                }
            }

            match self.mode {
                Mode::Draw => self.field.handle_adding_cells(),
                Mode::Erase => self.field.handle_removing_cells(),
                Mode::StartSelection => self.field.handle_start_cell_selection(),
                Mode::EndSelection => self.field.handle_end_cell_selection(),
            }
        });
    }
}

impl MyApp {
    fn generate_file(&mut self) {
        let tera = Tera::new("template/*.tera").expect("Failed to load template");

        let mut context = tera::Context::new();

        context.insert("size", &self.field.field_size);

        if self.field.start_cell.is_none() {
            self.show_toast("Start point wasn't set", ToastVariant::Error);
            return;
        }

        if self.field.end_cell.is_none() {
            self.show_toast("End point wasn't set", ToastVariant::Error);
            return;
        }

        context.insert("start_x", &(self.field.start_cell.as_ref().unwrap().x + 1));
        context.insert("start_y", &(self.field.start_cell.as_ref().unwrap().y + 1));

        context.insert("end_x", &(self.field.end_cell.as_ref().unwrap().x + 1));
        context.insert("end_y", &(self.field.end_cell.as_ref().unwrap().y + 1));

        context.insert(
            "disabled_nodes",
            &self
                .field
                .filled_cells
                .iter()
                .map(|c| format!("({},{})", c.x + 1, c.y + 1))
                .collect::<Vec<_>>()
                .join(" "),
        );

        match tera.render(self.template_name(), &context) {
            Ok(output) => match std::fs::write("path.txt", output) {
                Ok(_) => {
                    self.show_toast("File generated", ToastVariant::Success);
                }
                Err(_) => {
                    self.show_toast("Failed to write file", ToastVariant::Error);
                }
            },
            Err(_) => {
                self.show_toast("Failed to render", ToastVariant::Error);
            }
        }
    }

    fn template_name(&self) -> &str {
        match self.template {
            Template::Default => "path.tera",
            Template::Eight => "path_8.tera",
            Template::Disabled => "path_disabled.tera",
        }
    }

    fn show_toast(&mut self, message: &str, r#type: ToastVariant) {
        self.toast = Some(Toast::new(message, r#type));
    }
}
