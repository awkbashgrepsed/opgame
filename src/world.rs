use glam::Vec3;
use crate::entity::GameObject;
use std::collections::HashMap;
use uuid::Uuid;

pub struct World {
    pub objects: HashMap<Uuid, GameObject>,
    pub width: f32,
    pub height: f32,
    pub time_of_day: f32, // 0-24
    pub weather: Weather,
}

#[derive(Clone, Copy, Debug)]
pub enum Weather {
    Clear,
    Rainy,
    Foggy,
    Stormy,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            objects: HashMap::new(),
            width: 1000.0,
            height: 1000.0,
            time_of_day: 12.0,
            weather: Weather::Clear,
        };

        // Spawn initial buildings/structures
        world.spawn_building(Vec3::new(-50.0, 0.0, -50.0), "Building_A");
        world.spawn_building(Vec3::new(50.0, 0.0, -50.0), "Building_B");
        world.spawn_building(Vec3::new(-50.0, 0.0, 50.0), "Building_C");
        world.spawn_building(Vec3::new(50.0, 0.0, 50.0), "Building_D");

        world
    }

    pub fn spawn_building(&mut self, position: Vec3, name: &str) {
        let obj = GameObject::new(position, name.to_string());
        self.objects.insert(obj.id, obj);
    }

    pub fn spawn_object(&mut self, position: Vec3, name: &str) {
        let obj = GameObject::new(position, name.to_string());
        self.objects.insert(obj.id, obj);
    }

    pub fn remove_object(&mut self, id: Uuid) {
        self.objects.remove(&id);
    }

    pub fn update_time(&mut self, delta: f32) {
        self.time_of_day += delta * 0.001; // Slow time progression
        if self.time_of_day >= 24.0 {
            self.time_of_day = 0.0;
        }
    }

    pub fn set_weather(&mut self, weather: Weather) {
        self.weather = weather;
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day < 6.0 || self.time_of_day >= 18.0
    }

    pub fn is_day(&self) -> bool {
        !self.is_night()
    }
}
