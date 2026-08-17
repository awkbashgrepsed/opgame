use crate::player::Player;
use crate::npc::{NPCManager, NPCState};
use uuid::Uuid;

pub struct CombatSystem {
    damage_cooldown: f32,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            damage_cooldown: 0.0,
        }
    }

    pub fn update(&mut self, player: &mut Player, npc_manager: &mut NPCManager, time: f32) {
        self.damage_cooldown = (self.damage_cooldown - 0.016).max(0.0);

        // Check for NPC hits
        for npc_id in npc_manager.npcs_in_range(player.position, 2.0) {
            if let Some(npc) = npc_manager.get_npc_mut(npc_id) {
                if player.fire_weapon() && self.damage_cooldown <= 0.0 {
                    let weapon = player.current_weapon();
                    npc.take_damage(weapon.damage());
                    self.damage_cooldown = weapon.fire_rate();
                    
                    if npc.state != NPCState::Dead {
                        npc.state = NPCState::Fighting;
                        player.increase_wanted_level();
                    }
                }
            }
        }

        // Check for NPC attacks on player
        for npc_id in npc_manager.npcs_in_range(player.position, 2.0) {
            if let Some(npc) = npc_manager.get_npc_mut(npc_id) {
                if npc.state == NPCState::Fighting && npc.is_alive() {
                    player.take_damage(5.0); // Simple damage
                }
            }
        }
    }

    pub fn deal_damage(&mut self, attacker_id: Uuid, target_id: Uuid, damage: f32) {
        // Logic for dealing damage between entities
    }
}
