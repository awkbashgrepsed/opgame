use crate::assets::path;
use serde::{Deserialize, Serialize};
use std::fs;

const SETTINGS_RELATIVE_PATH: &str = "config/settings.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub vsync: bool,
    #[serde(default = "default_true")]
    pub fullscreen: bool,
    #[serde(default = "default_sensitivity")]
    pub mouse_sensitivity: f32,
    #[serde(default)]
    pub invert_x: bool,
    #[serde(default)]
    pub invert_y: bool,
}

fn default_true() -> bool { true }
fn default_sensitivity() -> f32 { 0.004 }

impl Default for Settings {
    fn default() -> Self {
        Self { vsync: true, fullscreen: true, mouse_sensitivity: 0.004, invert_x: false, invert_y: false }
    }
}

impl Settings {
    fn file_path() -> std::path::PathBuf { path(SETTINGS_RELATIVE_PATH) }

    pub fn load() -> Self {
        let file_path = Self::file_path();
        match fs::read_to_string(&file_path) {
            Ok(contents) => match toml::from_str::<Settings>(&contents) {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!("Could not parse {}: {}. Using defaults.", file_path.display(), error);
                    let settings = Self::default();
                    settings.save();
                    settings
                }
            },
            Err(_) => {
                let settings = Self::default();
                settings.save();
                settings
            }
        }
    }

    pub fn save(&self) {
        let file_path = Self::file_path();
        if let Some(parent) = file_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                log::warn!("Could not create settings directory: {}", error);
                return;
            }
        }
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(error) = fs::write(&file_path, contents) {
                    log::warn!("Could not save {}: {}", file_path.display(), error);
                }
            }
            Err(error) => log::warn!("Could not serialize settings: {}", error),
        }
    }
}
