use crate::player::Player;

pub struct UIManager {
    show_map: bool,
    show_menu: bool,
    show_objectives: bool,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            show_map: true,
            show_menu: false,
            show_objectives: true,
        }
    }

    pub fn toggle_map(&mut self) {
        self.show_map = !self.show_map;
    }

    pub fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
    }

    pub fn update(&mut self, _player: &Player, _mission_manager: &crate::mission::MissionManager, _time: f32) {
        // Update UI based on game state
    }

    pub fn render_hud(&self, player: &Player) -> String {
        format!(
            "Health: {:.1}/{:.1} | Money: ${} | Wanted: {}",
            player.health, player.max_health, player.money, player.wanted_level
        )
    }

    pub fn should_show_map(&self) -> bool {
        self.show_map
    }

    pub fn should_show_menu(&self) -> bool {
        self.show_menu
    }
}
