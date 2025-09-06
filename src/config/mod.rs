pub mod editor;

use serde::{Deserialize, Serialize};

use crate::{consts::CONFIG_PATH, error::AppError};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub email: String,
}

impl Config {
    pub fn load() -> Self {
        let content = std::fs::read_to_string(CONFIG_PATH).unwrap_or_default();
        let config: Self = toml::from_str(&content).unwrap_or_default();

        config
    }

    pub fn save(&self) -> Result<(), AppError> {
        let toml_str = toml::to_string_pretty(self).unwrap();
        std::fs::write(CONFIG_PATH, toml_str).map_err(|_| AppError::FailedUpdateConfig)
    }
}
