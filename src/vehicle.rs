use glam::Vec3;
use uuid::Uuid;
use crate::player::Player;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VehicleType {
    Car,
    Bike,
    Truck,
    Helicopter,
    Boat,
}

pub struct Vehicle {
    pub id: Uuid,
    pub vehicle_type: VehicleType,
    pub position: Vec3,
    pub rotation: f32,
    pub velocity: Vec3,
    pub speed: f32,
    pub max_speed: f32,
    pub health: f32,
    pub max_health: f32,
    pub fuel: f32,
    pub max_fuel: f32,
    pub occupants: Vec<Uuid>,
    pub name: String,
}

impl Vehicle {
    pub fn new(vehicle_type: VehicleType, position: Vec3, name: String) -> Self {
        let (max_speed, max_health) = match vehicle_type {
            VehicleType::Car => (0.5, 100.0),
            VehicleType::Bike => (0.7, 50.0),
            VehicleType::Truck => (0.3, 150.0),
            VehicleType::Helicopter => (0.8, 120.0),
            VehicleType::Boat => (0.4, 100.0),
        };

        Self {
            id: Uuid::new_v4(),
            vehicle_type,
            position,
            rotation: 0.0,
            velocity: Vec3::ZERO,
            speed: 0.0,
            max_speed,
            health: max_health,
            max_health,
            fuel: 100.0,
            max_fuel: 100.0,
            occupants: Vec::new(),
            name,
        }
    }

    pub fn add_occupant(&mut self, player_id: Uuid) {
        if !self.occupants.contains(&player_id) {
            self.occupants.push(player_id);
        }
    }

    pub fn remove_occupant(&mut self, player_id: Uuid) {
        self.occupants.retain(|id| id != &player_id);
    }

    pub fn accelerate(&mut self) {
        if self.fuel > 0.0 {
            self.speed = (self.speed + 0.02).min(self.max_speed);
            self.fuel -= 0.01;
        } else {
            self.speed = 0.0;
        }
    }

    pub fn brake(&mut self) {
        self.speed = (self.speed - 0.05).max(0.0);
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    pub fn refuel(&mut self) {
        self.fuel = self.max_fuel;
    }

    pub fn is_destroyed(&self) -> bool {
        self.health <= 0.0
    }

    pub fn update(&mut self, _time: f32) {
        self.position += self.velocity * self.speed;
    }
}

pub struct VehicleManager {
    vehicles: Vec<Vehicle>,
}

impl VehicleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            vehicles: Vec::new(),
        };

        // Spawn initial vehicles
        manager.spawn_vehicle(VehicleType::Car, Vec3::new(0.0, 0.5, 30.0), "Car_01".to_string());
        manager.spawn_vehicle(VehicleType::Car, Vec3::new(30.0, 0.5, 0.0), "Car_02".to_string());
        manager.spawn_vehicle(VehicleType::Bike, Vec3::new(-30.0, 0.5, 0.0), "Bike_01".to_string());

        manager
    }

    pub fn spawn_vehicle(&mut self, vehicle_type: VehicleType, position: Vec3, name: String) {
        self.vehicles.push(Vehicle::new(vehicle_type, position, name));
    }

    pub fn update(&mut self, _time: f32) {
        for vehicle in &mut self.vehicles {
            vehicle.update(_time);
        }
    }

    pub fn get_vehicles(&self) -> &[Vehicle] {
        &self.vehicles
    }

    pub fn get_vehicle_mut(&mut self, index: usize) -> Option<&mut Vehicle> {
        self.vehicles.get_mut(index)
    }

    pub fn vehicles_in_range(&self, pos: Vec3, range: f32) -> Vec<usize> {
        self.vehicles
            .iter()
            .enumerate()
            .filter(|(_, v)| (v.position - pos).length() < range)
            .map(|(idx, _)| idx)
            .collect()
    }
}
