use glam::Vec3;
use uuid::Uuid;

pub trait Entity {
    fn update(&mut self, time: f32);
    fn position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
    fn id(&self) -> Uuid;
}

pub struct GameObject {
    pub id: Uuid,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec3,
    pub name: String,
}

impl GameObject {
    pub fn new(position: Vec3, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            rotation: 0.0,
            scale: Vec3::ONE,
            name,
        }
    }
}

impl Entity for GameObject {
    fn update(&mut self, _time: f32) {}

    fn position(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    fn id(&self) -> Uuid {
        self.id
    }
}
