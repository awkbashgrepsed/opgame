use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SETTINGS_PATH: &str = "data/config/settings.toml";

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
    pub fn load() -> Self {
        let path = Path::new(SETTINGS_PATH);
        match fs::read_to_string(path) {
            Ok(contents) => match toml::from_str::<Settings>(&contents) {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!("Could not parse {}: {}. Using defaults.", SETTINGS_PATH, error);
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
        let path = Path::new(SETTINGS_PATH);
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                log::warn!("Could not create settings directory: {}", error);
                return;
            }
        }
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(error) = fs::write(path, contents) {
                    log::warn!("Could not save {}: {}", SETTINGS_PATH, error);
                }
            }
            Err(error) => log::warn!("Could not serialize settings: {}", error),
        }
    }
}
