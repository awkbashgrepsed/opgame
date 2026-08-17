use glam::Vec3;
use uuid::Uuid;
use rand::Rng;
use crate::world::World;
use crate::player::Player;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NPCState {
    Idle,
    Walking,
    Running,
    Fighting,
    Dead,
}

pub struct NPC {
    pub id: Uuid,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub state: NPCState,
    pub ai_timer: f32,
    pub target: Option<Uuid>,
    pub name: String,
    pub faction: String,
}

impl NPC {
    pub fn new(position: Vec3, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            velocity: Vec3::ZERO,
            health: 100.0,
            max_health: 100.0,
            state: NPCState::Idle,
            ai_timer: 0.0,
            target: None,
            name,
            faction: "Neutral".to_string(),
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
        if self.health <= 0.0 {
            self.state = NPCState::Dead;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn update(&mut self, player_pos: Vec3, _time: f32) {
        if !self.is_alive() {
            return;
        }

        let distance_to_player = (player_pos - self.position).length();

        self.ai_timer += 0.016;

        match self.state {
            NPCState::Idle => {
                if distance_to_player < 20.0 {
                    self.state = NPCState::Walking;
                }
            }
            NPCState::Walking => {
                if distance_to_player < 10.0 {
                    self.state = NPCState::Fighting;
                    self.target = Some(Uuid::nil()); // Player
                } else if self.ai_timer > 5.0 {
                    self.state = NPCState::Idle;
                    self.ai_timer = 0.0;
                }
                // Random walk
                let mut rng = rand::thread_rng();
                self.velocity = Vec3::new(rng.gen_range(-0.05..0.05), 0.0, rng.gen_range(-0.05..0.05));
            }
            NPCState::Fighting => {
                if distance_to_player < 2.0 {
                    // Melee combat
                } else {
                    // Move towards player
                    let direction = (player_pos - self.position).normalize();
                    self.velocity = direction * 0.1;
                }
            }
            NPCState::Dead => {
                self.velocity = Vec3::ZERO;
            }
        }

        // Apply gravity
        self.position.y -= 0.1;
        if self.position.y < 0.5 {
            self.position.y = 0.5;
        }

        self.position += self.velocity;
    }
}

pub struct NPCManager {
    npcs: HashMap<Uuid, NPC>,
}

impl NPCManager {
    pub fn new() -> Self {
        let mut manager = Self {
            npcs: HashMap::new(),
        };

        // Spawn initial NPCs
        manager.spawn_npc(Vec3::new(10.0, 0.5, 10.0), "Guard_01".to_string());
        manager.spawn_npc(Vec3::new(-10.0, 0.5, -10.0), "Civilian_01".to_string());
        manager.spawn_npc(Vec3::new(20.0, 0.5, 20.0), "Guard_02".to_string());
        manager.spawn_npc(Vec3::new(-20.0, 0.5, -20.0), "Civilian_02".to_string());

        manager
    }

    pub fn spawn_npc(&mut self, position: Vec3, name: String) {
        let npc = NPC::new(position, name);
        self.npcs.insert(npc.id, npc);
    }

    pub fn despawn_npc(&mut self, id: Uuid) {
        self.npcs.remove(&id);
    }

    pub fn update(&mut self, player: &Player, _world: &crate::world::World, _time: f32) {
        for npc in self.npcs.values_mut() {
            npc.update(player.position, _time);
        }
    }

    pub fn get_npcs(&self) -> &HashMap<Uuid, NPC> {
        &self.npcs
    }

    pub fn get_npc_mut(&mut self, id: Uuid) -> Option<&mut NPC> {
        self.npcs.get_mut(&id)
    }

    pub fn npcs_in_range(&self, pos: Vec3, range: f32) -> Vec<Uuid> {
        self.npcs
            .iter()
            .filter(|(_, npc)| (npc.position - pos).length() < range)
            .map(|(id, _)| *id)
            .collect()
    }
}
