use std::collections::HashMap;

pub struct SoundManager {
    effects: HashMap<String, String>,
    music_enabled: bool,
}

impl SoundManager {
    pub fn new() -> Self {
        let mut effects = HashMap::new();
        
        // Register sound effects
        effects.insert("gunshot".to_string(), "assets/sounds/gunshot.wav".to_string());
        effects.insert("footstep".to_string(), "assets/sounds/footstep.wav".to_string());
        effects.insert("impact".to_string(), "assets/sounds/impact.wav".to_string());
        effects.insert("vehicle_engine".to_string(), "assets/sounds/engine.wav".to_string());

        Self {
            effects,
            music_enabled: true,
        }
    }

    pub fn play_effect(&self, effect_name: &str) {
        if let Some(_path) = self.effects.get(effect_name) {
            // Would play sound here
            // For now, just log
            log::info!("Playing effect: {}", effect_name);
        }
    }

    pub fn play_music(&self, music_name: &str) {
        if self.music_enabled {
            log::info!("Playing music: {}", music_name);
        }
    }

    pub fn toggle_music(&mut self) {
        self.music_enabled = !self.music_enabled;
    }

    pub fn stop_all(&self) {
        log::info!("Stopping all audio");
    }
}
