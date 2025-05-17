use eframe::egui::{Color32, Id, Modal, RichText, Ui};
use email_address::EmailAddress;

use super::Config;

pub struct ConfigEditor {
    pub config: Config,
    open: bool,
    email_buffer: String,
    error_msg: String,
}

impl ConfigEditor {
    pub fn new() -> Self {
        let config = Config::load();

        Self {
            email_buffer: config.email.clone(),
            config,
            open: false,
            error_msg: String::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.error_msg = String::new();
    }

    pub fn show(&mut self, ui: &mut Ui) {
        Modal::new(Id::new("CONFIG MODAL")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);

            ui.heading("Config");

            ui.label("Email:");
            ui.text_edit_singleline(&mut self.email_buffer);

            ui.add_space(10.0);

            if !self.error_msg.is_empty() {
                ui.label(RichText::new(&self.error_msg).color(Color32::RED).strong());
                ui.add_space(10.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if EmailAddress::is_valid(&self.email_buffer) {
                        self.config.email = self.email_buffer.clone();
                        let _ = self.config.save();
                        self.close();
                    } else {
                        self.error_msg = "Invalid email".to_string();
                    }
                }

                ui.add_space(10.0);

                if ui.button("Cancel").clicked() {
                    self.close();
                }
            });
        });
    }
}

impl Default for ConfigEditor {
    fn default() -> Self {
        Self::new()
    }
}
