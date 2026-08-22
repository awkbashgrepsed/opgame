use glam::Vec3;
use uuid::Uuid;
use crate::collision::Aabb;

pub trait Entity {
    fn update(&mut self, time: f32);
    fn position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
    fn id(&self) -> Uuid;
}

pub struct GameObject {
    pub id: Uuid,
    pub asset: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub name: String,
    pub collider: Option<Aabb>,
}

impl GameObject {
    pub fn new(position: Vec3, name: String) -> Self {
        Self { id: Uuid::new_v4(), asset: String::new(), position, rotation: Vec3::ZERO, scale: Vec3::ONE, name, collider: None }
    }

    pub fn from_asset(asset: &str, position: Vec3, rotation: Vec3, scale: Vec3, collider: Aabb) -> Self {
        Self { id: Uuid::new_v4(), asset: asset.to_string(), position, rotation, scale, name: asset.to_string(), collider: Some(collider) }
    }

    pub fn with_box_collider(mut self, half_extents: Vec3) -> Self {
        self.collider = Some(Aabb::new(Vec3::ZERO, half_extents));
        self
    }

    pub fn collision_aabb(&self) -> Option<Aabb> {
        self.collider.map(|local| {
            let half_extents = local.half_extents * self.scale.abs();
            Aabb::new(self.position + local.center * self.scale, half_extents)
        })
    }
}

impl Entity for GameObject {
    fn update(&mut self, _time: f32) {}
    fn position(&self) -> Vec3 { self.position }
    fn set_position(&mut self, pos: Vec3) { self.position = pos; }
    fn id(&self) -> Uuid { self.id }
}
