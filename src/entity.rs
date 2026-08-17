use glam::Vec3;

pub trait Entity {
    fn update(&mut self);
    fn position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
}

pub struct GameObject {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec3,
}

impl GameObject {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec3::ONE,
        }
    }
}

impl Entity for GameObject {
    fn update(&mut self) {}

    fn position(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }
}
