use crate::assets::AssetManager;
use crate::camera::Camera;
use crate::combat::CombatSystem;
use crate::mission::MissionManager;
use crate::npc::NPCManager;
use crate::physics::PhysicsEngine;
use crate::player::Player;
use crate::renderer::Renderer;
use crate::settings::Settings;
use crate::sound::SoundManager;
use crate::ui::UIManager;
use crate::vehicle::VehicleManager;
use crate::world::World;
use glam::Vec3;
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const WALK_SPEED: f32 = 3.0;
const SPRINT_SPEED: f32 = 6.0;
const MAX_FRAME_DT: f32 = 0.1;
const PLAYER_TURN_SPEED: f32 = 10.0;
const AIM_TURN_SPEED: f32 = 18.0;
const NORMAL_CAMERA_DISTANCE: f32 = 6.0;
const AIM_CAMERA_DISTANCE: f32 = 3.8;
const AIM_CAMERA_SMOOTHNESS: f32 = 10.0;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Menu {
    Pause,
    Settings,
    Controls,
    Display,
}

const PAUSE_COUNT: usize = 3;
const SETTINGS_COUNT: usize = 3;
const CONTROLS_COUNT: usize = 4;
const DISPLAY_COUNT: usize = 7;

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
    _asset_manager: AssetManager,
    settings: Settings,
    game_time: f32,
    paused: bool,
    mouse_captured: bool,
    aiming: bool,
    menu: Option<Menu>,
    menu_selected: usize,
    last_frame: Instant,
    display_initialized: bool,
}

struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
    sprint: bool,
}

impl Game {
    pub fn new(window: Window, gl_config: glutin::config::Config, settings: Settings) -> Self {
        let renderer = Renderer::new(window, gl_config, &settings);
        let mut camera = Camera::new();

        camera.sensitivity = settings.camera.mouse_sensitivity;
        camera.invert_x = settings.camera.invert_x;
        camera.invert_y = settings.camera.invert_y;

        let asset_manager = AssetManager::load();
        let player = Player::new(Vec3::ZERO);
        let world = World::new(&asset_manager);
        let physics_engine = PhysicsEngine::new(&asset_manager);
        let npc_manager = NPCManager::new();
        let vehicle_manager = VehicleManager::new();
        let combat_system = CombatSystem::new();
        let ui_manager = UIManager::new();
        let mut mission_manager = MissionManager::new();
        let sound_manager = SoundManager::new();

        mission_manager.create_tutorial_mission();
        camera.follow(player.position + Vec3::new(0.0, 1.0, 0.0));

        let mut game = Self {
            renderer,
            player,
            camera,
            input_state: InputState {
                forward: false,
                backward: false,
                left: false,
                right: false,
                jump: false,
                sprint: false,
            },
            world,
            npc_manager,
            vehicle_manager,
            combat_system,
            physics_engine,
            ui_manager,
            mission_manager,
            sound_manager,
            _asset_manager: asset_manager,
            settings,
            game_time: 0.0,
            paused: false,
            mouse_captured: false,
            aiming: false,
            menu: None,
            menu_selected: 0,
            last_frame: Instant::now(),
            display_initialized: false,
        };

        game.player.position = game.world.player_spawn();
        game.camera.follow(game.player.position + Vec3::new(0.0, 1.0, 0.0));
        game
    }
