use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub mouse_sensitivity: f32,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            mouse_sensitivity: 0.004,
            invert_x: false,
            invert_y: false,
        }
    }
}
