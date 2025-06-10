mod mode;

use eframe::egui::{self, Ui, UserData};

use crate::app::mode::Mode;
use crate::config::editor::ConfigEditor;
use crate::error::AppError;
use crate::utils::image::*;
use crate::neos::api::NeosAPI;
use crate::neos::response::NeosResponse;
use crate::neos::solver::Solver;
use crate::template::Template;
use crate::{field::Field, toast::Toast};

pub struct App {
    field: Field,
    mode: Mode,
    template: Template,
    toast: Option<Toast>,
    neos: NeosAPI,
    neos_output: String,
    solver: Solver,
    config_editor: ConfigEditor,
    // Use the flag because the screenshot event arrives in the next frame
    taking_screenshot: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            field: Field::new(),
            mode: Mode::Draw,
            template: Template::Disabled,
            toast: None,
            neos: NeosAPI::new(),
            neos_output: String::new(),
            solver: Solver::Cbc,
            config_editor: ConfigEditor::new(),
            taking_screenshot: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Config").clicked() {
                    self.config_editor.open();
                }

                ui.selectable_value(&mut self.mode, Mode::Draw, "Draw");
                ui.selectable_value(&mut self.mode, Mode::Erase, "Erase");
                ui.selectable_value(&mut self.mode, Mode::StartSelection, "Start");
                ui.selectable_value(&mut self.mode, Mode::EndSelection, "Terminal");

                egui::ComboBox::from_label("Template")
                    .selected_text(self.template.name())
                    .show_ui(ui, |ui| {
                        for variant in Template::variants() {
                            ui.selectable_value(&mut self.template, *variant, variant.name());
                        }
                    });

                if ui.button("Clear paths").clicked() {
                    self.field.clear_paths();
                }

                if ui.button("Ping NEOS").clicked() {
                    self.neos.ping();
                }

                egui::ComboBox::from_label("Solver")
                    .selected_text(self.solver.name())
                    .show_ui(ui, |ui| {
                        for variant in Solver::variants() {
                            ui.selectable_value(&mut self.solver, *variant, variant.name());
                        }
                    });

                if ui.button("Send to NEOS").clicked() {
                    match self.template.generate_neos_input_string(
                        &self.field,
                        &self.solver,
                        &self.config_editor.config.email,
                    ) {
                        Ok(input) => {
                            self.neos.submit_job(input);
                        }
                        Err(e) => self.handle_app_error(e),
                    }
                }

                if ui.button("Screenshot").clicked() {
                    self.taking_screenshot = true;
                }
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                self.field.setup(ui);

                if self.neos.is_solving_task {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Solving task");
                    });
                } else {
                    ui.vertical(|ui| {
                        egui::ScrollArea::vertical()
                            .max_height(self.field.area_height())
                            .show(ui, |ui| {
                                ui.label(&self.neos_output);
                            });
                    });
                }
            });

            match self.field.hovered_cell() {
                Some(cell) => {
                    ui.label(format!("Cell: (x: {}, y: {})", cell.x + 1, cell.y + 1));
                }
                None => {
                    ui.label("Cell: None");
                }
            }

            self.field.draw();

            if let Some(toast) = &self.toast {
                if toast.is_expired() {
                    self.toast = None;
                } else {
                    toast.show(ui);
                }
            }

            if self.config_editor.is_open() {
                self.config_editor.show(ui);
            }

            ui.label(format!("NEOS response :: {}", self.neos.response));

            match self.mode {
                Mode::Draw => self.field.handle_adding_cells(),
                Mode::Erase => self.field.handle_removing_cells(),
                Mode::StartSelection => self.field.handle_start_cell_selection(),
                Mode::EndSelection => self.field.handle_end_cell_selection(),
            }

            if let Ok(neos_response) = self.neos.rx.try_recv() {
                match neos_response {
                    NeosResponse::Error(msg) => {
                        self.neos.is_solving_task = false;
                        self.neos.response = msg
                    }
                    NeosResponse::Message(msg) => self.neos.response = msg,
                    NeosResponse::JobCredentials(job_number, job_password) => {
                        self.neos.response = format!(
                            "Submitted job: (number = {}, password = {})",
                            job_number, job_password
                        );

                        self.neos.get_final_results(job_number, job_password);
                    }
                    NeosResponse::JobOuput(output) => {
                        self.neos.is_solving_task = false;
                        match self.field.parse_all_paths(&output) {
                            Ok(_) => {}
                            Err(e) => self.handle_app_error(e),
                        }
                        self.neos_output = output;
                    }
                }
            }

            if self.taking_screenshot {
                if let Err(err) = self.take_screenshot(ui) {
                    self.handle_app_error(err);
                }
            }
        });
    }
}

impl App {
    fn show_error(&mut self, message: &str) {
        self.toast = Some(Toast::error(message));
    }

    fn handle_app_error(&mut self, e: AppError) {
        match e {
            AppError::ParseStringError(message) => self.show_error(&message),
            AppError::StartNotSet => self.show_error("Start not set"),
            AppError::EndNotSet => self.show_error("End not set"),
            AppError::InvalidPath => {
                self.show_error("Invalid path");
            }
            AppError::FailedRenderFile => {
                self.show_error("Failed render file");
            }
            AppError::InvalidAuthCredentials => {
                self.show_error("Invalid auth credentials");
            }
            AppError::FailedUpdateConfig => {
                self.show_error("Failed update config");
            }
            AppError::FailedTakeScreenshot => {
                self.show_error("Failed to take screenshot");
            }
        }
    }
}

impl App {
    fn take_screenshot(&mut self, ui: &mut Ui) -> Result<(), AppError> {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(UserData::default()));

        let image = ui.ctx().input(|i| {
            i.events
                .iter()
                .filter_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
                .last()
        });

        if let Some(image) = image {
            self.taking_screenshot = false;

            let image = crop_color_image(
                &image,
                self.field.painter_rect(),
                self.field.pixels_per_point(),
            )
            .ok_or(AppError::FailedTakeScreenshot)?;

            save_color_image_to_png("screenshot.png", &image)
        } else {
            Ok(())
        }
    }
}
