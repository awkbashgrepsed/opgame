use crate::player::Player;
use glam::Vec3;

pub struct PhysicsEngine {
    gravity: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self { gravity: 0.1 }
    }

    pub fn update(&mut self, player: &mut Player, _time: f32) {
        // Simple gravity implementation
        if player.position.y > 0.5 {
            player.velocity.y -= self.gravity;
        }

        // Ground collision
        if player.position.y <= 0.5 {
            player.position.y = 0.5;
            player.velocity.y = 0.0;
        }

        // Clamp to world bounds (simple)
        player.position.x = player.position.x.clamp(-500.0, 500.0);
        player.position.z = player.position.z.clamp(-500.0, 500.0);
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<Vec3> {
        // Simple raycast implementation
        Some(origin + direction * max_distance)
    }
}
