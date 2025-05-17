use eframe::egui::{self, Color32};
use interference_generator::error::AppError;
use interference_generator::neos::api::NeosAPI;
use interference_generator::neos::response::NeosResponse;
use interference_generator::template::Template;
use interference_generator::{
    field::Field,
    toast::{Toast, ToastVariant},
};

fn main() -> eframe::Result {
    dotenvy::dotenv().ok();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    #[allow(clippy::empty_loop)]
    std::thread::spawn(move || rt.block_on(async { loop {} }));

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

struct MyApp {
    field: Field,
    mode: Mode,
    template: Template,
    toast: Option<Toast>,
    neos: NeosAPI,
    neos_output: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            field: Field::new(),
            mode: Mode::Draw,
            template: Template::Disabled,
            toast: None,
            neos: NeosAPI::new(),
            neos_output: String::new(),
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
                    .selected_text(self.template.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.template, Template::Default, "path");
                        ui.selectable_value(&mut self.template, Template::Eight, "path_8");
                        ui.selectable_value(
                            &mut self.template,
                            Template::Disabled,
                            "path_disabled",
                        );
                    });

                if ui.button("Clear path").clicked() {
                    self.field.clear_path();
                }

                if ui.button("Ping NEOS").clicked() {
                    self.neos.ping();
                }

                if ui.button("Send to NEOS").clicked() {
                    match self.template.generate_neos_input_string(&self.field) {
                        Ok(input) => {
                            self.neos.submit_job(input);
                        }
                        Err(e) => self.handle_app_error(e),
                    }
                }
            });

            ui.add_space(20.0);

            ui.columns(2, |columns| {
                self.field.setup(&mut columns[0]);

                if self.neos_output.is_empty() {
                    // columns[1].spinner();
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(self.field.size())
                        .show(&mut columns[1], |ui| {
                            ui.label(&self.neos_output);
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

            ui.label(format!("NEOS response :: {}", self.neos.response));

            match self.mode {
                Mode::Draw => self.field.handle_adding_cells(),
                Mode::Erase => self.field.handle_removing_cells(),
                Mode::StartSelection => self.field.handle_start_cell_selection(),
                Mode::EndSelection => self.field.handle_end_cell_selection(),
            }

            if let Ok(neos_response) = self.neos.rx.try_recv() {
                match neos_response {
                    NeosResponse::Message(msg) => self.neos.response = msg,
                    NeosResponse::JobCredentials(job_number, job_password) => {
                        self.neos.response = format!(
                            "Submitted job: (number = {}, password = {})",
                            job_number, job_password
                        );

                        self.neos.get_final_results(job_number, job_password);
                    }
                    NeosResponse::JobOuput(output) => {
                        match self.field.parse_path(&output) {
                            Ok(_) => {}
                            Err(e) => self.handle_app_error(e),
                        }
                        self.neos_output = output;
                    }
                }
            }
        });
    }
}

impl MyApp {
    fn show_toast(&mut self, message: &str, r#type: ToastVariant) {
        self.toast = Some(Toast::new(message, r#type));
    }

    fn handle_app_error(&mut self, e: AppError) {
        match e {
            AppError::ParseStringError(message) => self.show_toast(&message, ToastVariant::Error),
            AppError::StartNotSet => self.show_toast("Start not set", ToastVariant::Error),
            AppError::EndNotSet => self.show_toast("End not set", ToastVariant::Error),
            AppError::InvalidPath => {
                self.show_toast("Invalid path", ToastVariant::Error);
            }
            AppError::FailedRenderFile => {
                self.show_toast("Failed render file", ToastVariant::Error);
            }
            AppError::InvalidAuthCredentials => {
                self.show_toast("Invalid auth credentials", ToastVariant::Error);
            }
        }
    }
}
