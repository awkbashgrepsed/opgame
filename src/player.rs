use glam::Vec3;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Weapon {
    Fist,
    Pistol,
    Rifle,
    Shotgun,
    Knife,
}

impl Weapon {
    pub fn damage(&self) -> f32 {
        match self {
            Weapon::Fist => 5.0,
            Weapon::Pistol => 15.0,
            Weapon::Rifle => 25.0,
            Weapon::Shotgun => 40.0,
            Weapon::Knife => 20.0,
        }
    }

    pub fn fire_rate(&self) -> f32 {
        match self {
            Weapon::Fist => 0.5,
            Weapon::Pistol => 0.1,
            Weapon::Rifle => 0.08,
            Weapon::Shotgun => 0.3,
            Weapon::Knife => 0.4,
        }
    }
}

pub struct Player {
    pub id: Uuid,
    pub position: Vec3,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub max_armor: f32,
    pub velocity: Vec3,
    pub money: u32,
    pub experience: u32,
    pub weapons: Vec<Weapon>,
    pub current_weapon_index: usize,
    pub ammo: u32,
    pub max_ammo: u32,
    pub wanted_level: u32,
    pub last_fire_time: f32,
    pub is_jumping: bool,
    pub is_falling: bool,
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            rotation: 0.0,
            health: 100.0,
            max_health: 100.0,
            armor: 0.0,
            max_armor: 100.0,
            velocity: Vec3::ZERO,
            money: 0,
            experience: 0,
            weapons: vec![Weapon::Fist, Weapon::Pistol],
            current_weapon_index: 0,
            ammo: 100,
            max_ammo: 100,
            wanted_level: 0,
            last_fire_time: -1.0,
            is_jumping: false,
            is_falling: false,
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        let armor_reduction = (self.armor / self.max_armor) * damage * 0.5;
        let remaining_damage = damage - armor_reduction;
        self.armor = (self.armor - armor_reduction).max(0.0);
        self.health = (self.health - remaining_damage).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn add_armor(&mut self, amount: f32) {
        self.armor = (self.armor + amount).min(self.max_armor);
    }

    pub fn add_money(&mut self, amount: u32) {
        self.money += amount;
    }

    pub fn add_experience(&mut self, amount: u32) {
        self.experience += amount;
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn current_weapon(&self) -> Weapon {
        self.weapons.get(self.current_weapon_index).copied().unwrap_or(Weapon::Fist)
    }

    pub fn select_weapon(&mut self, index: usize) {
        if index < self.weapons.len() {
            self.current_weapon_index = index;
            self.ammo = self.max_ammo;
        }
    }

    pub fn change_weapon(&mut self, direction: i32) {
        let new_index = (self.current_weapon_index as i32 + direction).clamp(0, self.weapons.len() as i32 - 1);
        self.current_weapon_index = new_index as usize;
        self.ammo = self.max_ammo;
    }

    pub fn fire_weapon(&mut self) -> bool {
        if self.ammo > 0 && self.current_weapon() != Weapon::Fist {
            self.ammo -= 1;
            return true;
        }
        false
    }

    pub fn reload_weapon(&mut self) {
        self.ammo = self.max_ammo;
    }

    pub fn add_weapon(&mut self, weapon: Weapon) {
        if !self.weapons.contains(&weapon) {
            self.weapons.push(weapon);
        }
    }

    pub fn increase_wanted_level(&mut self) {
        self.wanted_level = (self.wanted_level + 1).min(6);
    }

    pub fn decrease_wanted_level(&mut self) {
        self.wanted_level = self.wanted_level.saturating_sub(1);
    }

    pub fn update(&mut self, _time: f32) {
        // Player state is updated by the physics system. Keeping movement and
        // collision out of Player prevents multiple systems from moving it.
    }
}
