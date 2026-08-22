use glam::Vec3;
use crate::entity::GameObject;
use crate::collision::Aabb;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Road {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
}

pub struct World {
    pub objects: HashMap<Uuid, GameObject>,
    pub roads: Vec<Road>,
    pub width: f32,
    pub height: f32,
    pub time_of_day: f32,
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
            roads: Vec::new(),
            width: 1000.0,
            height: 1000.0,
            time_of_day: 12.0,
            weather: Weather::Clear,
        };

        world.add_road(Vec3::new(-250.0, 0.0, 0.0), Vec3::new(250.0, 0.0, 0.0), 12.0);
        world.add_road(Vec3::new(0.0, 0.0, -250.0), Vec3::new(0.0, 0.0, 250.0), 12.0);
        world.add_road(Vec3::new(-250.0, 0.0, -100.0), Vec3::new(250.0, 0.0, -100.0), 8.0);
        world.add_road(Vec3::new(-250.0, 0.0, 100.0), Vec3::new(250.0, 0.0, 100.0), 8.0);
        world.add_road(Vec3::new(-100.0, 0.0, -250.0), Vec3::new(-100.0, 0.0, 250.0), 8.0);
        world.add_road(Vec3::new(100.0, 0.0, -250.0), Vec3::new(100.0, 0.0, 250.0), 8.0);

        world.spawn_building(Vec3::new(-50.0, 0.0, -50.0), "Building_A");
        world.spawn_building(Vec3::new(50.0, 0.0, -50.0), "Building_B");
        world.spawn_building(Vec3::new(-50.0, 0.0, 50.0), "Building_C");
        world.spawn_building(Vec3::new(50.0, 0.0, 50.0), "Building_D");

        world
    }

    pub fn add_road(&mut self, start: Vec3, end: Vec3, width: f32) {
        self.roads.push(Road { start, end, width });
    }

    pub fn spawn_building(&mut self, position: Vec3, name: &str) {
        let obj = GameObject::new(position, name.to_string())
            .with_box_collider(Vec3::new(5.0, 5.0, 5.0));
        self.objects.insert(obj.id, obj);
    }

    pub fn spawn_object(&mut self, position: Vec3, name: &str) {
        let obj = GameObject::new(position, name.to_string());
        self.objects.insert(obj.id, obj);
    }

    pub fn remove_object(&mut self, id: Uuid) {
        self.objects.remove(&id);
    }

    pub fn collision_boxes(&self) -> impl Iterator<Item = Aabb> + '_ {
        self.objects.values().filter_map(|object| object.collision_aabb())
    }

    pub fn update_time(&mut self, delta: f32) {
        self.time_of_day += delta * 0.001;
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
