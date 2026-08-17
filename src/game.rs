use crate::camera::Camera;
use crate::player::Player;
use crate::renderer::Renderer;
use crate::world::World;
use crate::npc::NPCManager;
use crate::vehicle::VehicleManager;
use crate::combat::CombatSystem;
use crate::physics::PhysicsEngine;
use crate::ui::UIManager;
use crate::mission::MissionManager;
use crate::sound::SoundManager;
use glam::Vec3;
use winit::event::{KeyEvent, ElementState, MouseButton};
use winit::event::MouseScrollDelta;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Game {
    renderer: Renderer,
    player: Player,
    camera: Camera,
    input_state: InputState,
    world: World,
    npc_manager: NPCManager,
    vehicle_manager: VehicleManager,
    combat_system: CombatSystem,
    physics_engine: PhysicsEngine,
    ui_manager: UIManager,
    mission_manager: MissionManager,
    sound_manager: SoundManager,
    game_time: f32,
    paused: bool,
}

struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
    sprint: bool,
    in_vehicle: bool,
}

impl Game {
    pub async fn new(window: &winit::window::Window) -> Self {
        let renderer = Renderer::new(window).await;
        let player = Player::new(Vec3::new(0.0, 1.0, 0.0));
        let mut camera = Camera::new();
        
        let world = World::new();
        let npc_manager = NPCManager::new();
        let vehicle_manager = VehicleManager::new();
        let combat_system = CombatSystem::new();
        let physics_engine = PhysicsEngine::new();
        let ui_manager = UIManager::new();
        let mut mission_manager = MissionManager::new();
        let sound_manager = SoundManager::new();

        // Initialize first mission
        mission_manager.create_tutorial_mission();

        let input_state = InputState {
            forward: false,
            backward: false,
            left: false,
            right: false,
            jump: false,
            sprint: false,
            in_vehicle: false,
        };

        camera.position = player.position + Vec3::new(0.0, 2.0, -5.0);
        camera.look_at(player.position + Vec3::new(0.0, 1.0, 0.0));

        Self {
            renderer,
            player,
            camera,
            input_state,
            world,
            npc_manager,
            vehicle_manager,
            combat_system,
            physics_engine,
            ui_manager,
            mission_manager,
            sound_manager,
            game_time: 0.0,
            paused: false,
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        let pressed = event.state == ElementState::Pressed;
        
        if let PhysicalKey::Code(code) = event.physical_key {
            match code {
                KeyCode::KeyW => self.input_state.forward = pressed,
                KeyCode::KeyS => self.input_state.backward = pressed,
                KeyCode::KeyA => self.input_state.left = pressed,
                KeyCode::KeyD => self.input_state.right = pressed,
                KeyCode::Space => self.input_state.jump = pressed,
                KeyCode::ShiftLeft => self.input_state.sprint = pressed,
                KeyCode::KeyP => {
                    if pressed {
                        self.paused = !self.paused;
                    }
                }
                KeyCode::KeyF => {
                    if pressed {
                        self.player.fire_weapon();
                    }
                }
                KeyCode::KeyR => {
                    if pressed {
                        self.player.reload_weapon();
                    }
                }
                KeyCode::Digit1 => {
                    if pressed {
                        self.player.select_weapon(0);
                    }
                }
                KeyCode::Digit2 => {
                    if pressed && self.player.weapons.len() > 1 {
                        self.player.select_weapon(1);
                    }
                }
                KeyCode::Digit3 => {
                    if pressed && self.player.weapons.len() > 2 {
                        self.player.select_weapon(2);
                    }
                }
                KeyCode::Escape => std::process::exit(0),
                _ => {}
            }
        }
    }

    pub fn handle_mouse_click(&mut self, state: ElementState, button: MouseButton) {
        if state == ElementState::Pressed {
            match button {
                MouseButton::Left => {
                    self.player.fire_weapon();
                }
                MouseButton::Right => {
                    // Aim
                }
                _ => {}
            }
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let scroll_amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 10.0,
        };
        self.player.change_weapon(if scroll_amount > 0.0 { 1 } else { -1 });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.camera.aspect = width as f32 / height as f32;
    }

    pub async fn update_and_render(&mut self) {
        if !self.paused {
            self.update();
        }
        self.renderer.render(&self.camera, &self.player, &self.world, &self.npc_manager, &self.vehicle_manager, &self.ui_manager).await;
    }

    fn update(&mut self) {
        self.game_time += 0.016; // ~60 FPS

        // Update player
        let mut velocity = Vec3::ZERO;
        let speed = if self.input_state.sprint { 0.2 } else { 0.1 };

        if self.input_state.forward {
            velocity += self.camera.forward() * speed;
        }
        if self.input_state.backward {
            velocity -= self.camera.forward() * speed;
        }
        if self.input_state.right {
            velocity += self.camera.right() * speed;
        }
        if self.input_state.left {
            velocity -= self.camera.right() * speed;
        }

        self.player.position += velocity;
        self.player.update(self.game_time);

        // Physics
        self.physics_engine.update(&mut self.player, self.game_time);

        // NPCs
        self.npc_manager.update(&self.player, &self.world, self.game_time);

        // Vehicles
        self.vehicle_manager.update(self.game_time);

        // Combat
        self.combat_system.update(&mut self.player, &mut self.npc_manager, self.game_time);

        // Missions
        self.mission_manager.update(&self.player, &self.npc_manager, self.game_time);

        // Camera follow
        self.camera.position = self.player.position + Vec3::new(0.0, 2.0, -5.0);
        self.camera.look_at(self.player.position + Vec3::new(0.0, 1.0, 0.0));

        // UI
        self.ui_manager.update(&self.player, &self.mission_manager, self.game_time);
    }
}
