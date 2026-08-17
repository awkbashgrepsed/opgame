use glam::Vec3;

pub struct Player {
    pub position: Vec3,
    pub rotation: f32,
    pub health: f32,
    pub velocity: Vec3,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: 0.0,
            health: 100.0,
            velocity: Vec3::ZERO,
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }
}
