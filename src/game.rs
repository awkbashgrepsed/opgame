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
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const WALK_SPEED:f32=3.0;
const SPRINT_SPEED:f32=6.0;
const MAX_FRAME_DT:f32=0.1;
const PLAYER_TURN_SPEED:f32=10.0;
const AIM_TURN_SPEED:f32=18.0;
const NORMAL_CAMERA_DISTANCE:f32=6.0;
const AIM_CAMERA_DISTANCE:f32=3.8;
const AIM_CAMERA_SMOOTHNESS:f32=10.0;

pub struct Game {
    renderer:Renderer, player:Player, camera:Camera, input_state:InputState, world:World,
    npc_manager:NPCManager, vehicle_manager:VehicleManager, combat_system:CombatSystem,
    physics_engine:PhysicsEngine, ui_manager:UIManager, mission_manager:MissionManager,
    sound_manager:SoundManager, game_time:f32, paused:bool, mouse_captured:bool,
    aiming:bool, settings_menu:bool, settings_selected:usize, last_frame:Instant,
}
struct InputState{forward:bool,backward:bool,left:bool,right:bool,jump:bool,sprint:bool}

impl Game {
    pub fn new(window:Window,gl_config:glutin::config::Config)->Self{
        let renderer=Renderer::new(window,gl_config); let player=Player::new(Vec3::new(0.0,0.5,0.0)); let mut camera=Camera::new();
        let world=World::new(); let npc_manager=NPCManager::new(); let vehicle_manager=VehicleManager::new();
        let combat_system=CombatSystem::new(); let physics_engine=PhysicsEngine::new(); let ui_manager=UIManager::new();
        let mut mission_manager=MissionManager::new(); let sound_manager=SoundManager::new(); mission_manager.create_tutorial_mission();
        camera.follow(player.position+Vec3::new(0.0,1.0,0.0));
        let mut game=Self{renderer,player,camera,input_state:InputState{forward:false,backward:false,left:false,right:false,jump:false,sprint:false},world,npc_manager,vehicle_manager,combat_system,physics_engine,ui_manager,mission_manager,sound_manager,game_time:0.0,paused:false,mouse_captured:false,aiming:false,settings_menu:false,settings_selected:0,last_frame:Instant::now()};
        game.set_mouse_capture(true); game
    }
    pub fn toggle_settings_menu(&mut self){self.settings_menu=!self.settings_menu;self.paused=self.settings_menu;self.clear_input_state();self.aiming=false;self.set_mouse_capture(!self.settings_menu);self.last_frame=Instant::now();}
    fn clear_input_state(&mut self){self.input_state=InputState{forward:false,backward:false,left:false,right:false,jump:false,sprint:false};}
    fn change_setting(&mut self){match self.settings_selected{0=>self.renderer.toggle_vsync(),1=>self.camera.sensitivity=(self.camera.sensitivity+0.001).min(0.02),2=>self.camera.invert_x=!self.camera.invert_x,3=>self.camera.invert_y=!self.camera.invert_y,4=>self.renderer.toggle_fullscreen(),5=>std::process::exit(0),_=>{}}}
    pub fn handle_key(&mut self,event:KeyEvent){
        let pressed=event.state==ElementState::Pressed;
        if let PhysicalKey::Code(code)=event.physical_key{
            if self.settings_menu{if pressed{match code{KeyCode::Escape=>self.toggle_settings_menu(),KeyCode::ArrowUp=>self.settings_selected=self.settings_selected.checked_sub(1).unwrap_or(5),KeyCode::ArrowDown=>self.settings_selected=(self.settings_selected+1)%6,KeyCode::Enter|KeyCode::Space=>self.change_setting(),KeyCode::ArrowLeft if self.settings_selected==1=>self.camera.sensitivity=(self.camera.sensitivity-0.001).max(0.001),KeyCode::ArrowRight if self.settings_selected==1=>self.camera.sensitivity=(self.camera.sensitivity+0.001).min(0.02),_=>{}}}return;}
            match code{
                KeyCode::Escape if pressed=>self.toggle_settings_menu(),
                KeyCode::KeyW=>self.input_state.forward=pressed,KeyCode::KeyS=>self.input_state.backward=pressed,
                KeyCode::KeyA=>self.input_state.left=pressed,KeyCode::KeyD=>self.input_state.right=pressed,
                KeyCode::Space=>self.input_state.jump=pressed,KeyCode::ShiftLeft=>self.input_state.sprint=pressed,
                KeyCode::KeyP if pressed=>{self.paused=!self.paused;self.clear_input_state();self.aiming=false;self.last_frame=Instant::now();},
                KeyCode::F9 if pressed=>self.renderer.toggle_vsync(),KeyCode::F11 if pressed=>self.renderer.toggle_fullscreen(),
                KeyCode::KeyF if pressed=>{self.player.fire_weapon();},KeyCode::KeyR if pressed=>{self.player.reload_weapon();},
                KeyCode::Digit1 if pressed=>{self.player.select_weapon(0);},KeyCode::Digit2 if pressed&&self.player.weapons.len()>1=>{self.player.select_weapon(1);},KeyCode::Digit3 if pressed&&self.player.weapons.len()>2=>{self.player.select_weapon(2);},
                KeyCode::PageUp if pressed=>self.camera.zoom(0.5),KeyCode::PageDown if pressed=>self.camera.zoom(-0.5),
                KeyCode::Tab if pressed=>self.camera.toggle_shoulder(),
                KeyCode::F10 if pressed=>self.set_mouse_capture(!self.mouse_captured),_=>{}
            }
        }
    }
    pub fn handle_cursor_moved(&mut self,x:f64,y:f64){if !self.mouse_captured||self.settings_menu{return;}let(cx,cy)=self.renderer.window_center();let dx=x-cx;let dy=y-cy;if dx.abs()<0.5&&dy.abs()<0.5{return;}self.camera.rotate(dx as f32*self.camera.sensitivity,dy as f32*self.camera.sensitivity);self.renderer.center_cursor();}
    pub fn handle_mouse_click(&mut self,state:ElementState,button:MouseButton){if self.settings_menu{return;}match button{MouseButton::Right=>{self.aiming=state==ElementState::Pressed;if self.aiming&&!self.mouse_captured{self.set_mouse_capture(true);}},MouseButton::Left=>{if state==ElementState::Pressed{if !self.mouse_captured{self.set_mouse_capture(true);return;}self.player.fire_weapon();}},_=>{}}}
    fn set_mouse_capture(&mut self,captured:bool){self.mouse_captured=captured;if !captured{self.aiming=false;}self.renderer.set_cursor_visible(!captured);if captured{self.renderer.center_cursor();}}
    pub fn resize(&mut self,width:u32,height:u32){self.renderer.resize(width,height);if height!=0{self.camera.aspect=width as f32/height as f32;}if self.mouse_captured{self.renderer.center_cursor();}}
    pub fn update_and_render(&mut self){let now=Instant::now();let mut dt=(now-self.last_frame).as_secs_f32();self.last_frame=now;dt=dt.min(MAX_FRAME_DT);if !self.paused{self.update(dt);}self.renderer.render(&self.camera,&self.player,&self.world,&self.npc_manager,&self.vehicle_manager,&self.ui_manager,self.settings_menu,self.settings_selected,self.camera.sensitivity,self.camera.invert_x,self.camera.invert_y,self.aiming);}
    fn update(&mut self,dt:f32){
        self.game_time+=dt;
        let mut movement=Vec3::ZERO;
        let speed=if self.input_state.sprint{SPRINT_SPEED}else{WALK_SPEED};
        let forward=self.camera.flat_forward();
        let right=self.camera.right();
        if self.input_state.forward{movement+=forward;}
        if self.input_state.backward{movement-=forward;}
        if self.input_state.right{movement+=right;}
        if self.input_state.left{movement-=right;}
        if movement.length_squared()>0.0001{
            let direction=movement.normalize();
            self.player.position+=direction*speed*dt;
            let target_rotation=if self.aiming{forward.x.atan2(forward.z)}else{direction.x.atan2(direction.z)};
            let mut angle_delta=target_rotation-self.player.rotation;
            while angle_delta>std::f32::consts::PI{angle_delta-=std::f32::consts::TAU;}
            while angle_delta < -std::f32::consts::PI{angle_delta+=std::f32::consts::TAU;}
            let max_turn=(if self.aiming{AIM_TURN_SPEED}else{PLAYER_TURN_SPEED})*dt;
            self.player.rotation+=angle_delta.clamp(-max_turn,max_turn);
        }else if self.aiming{
            let target_rotation=forward.x.atan2(forward.z);
            let mut angle_delta=target_rotation-self.player.rotation;
            while angle_delta>std::f32::consts::PI{angle_delta-=std::f32::consts::TAU;}
            while angle_delta < -std::f32::consts::PI{angle_delta+=std::f32::consts::TAU;}
            self.player.rotation+=angle_delta.clamp(-AIM_TURN_SPEED*dt,AIM_TURN_SPEED*dt);
        }
        let desired_camera_distance=if self.aiming{AIM_CAMERA_DISTANCE}else{NORMAL_CAMERA_DISTANCE};
        let blend=(AIM_CAMERA_SMOOTHNESS*dt).min(1.0);
        self.camera.distance+=(desired_camera_distance-self.camera.distance)*blend;
        self.player.update(self.game_time);
        self.physics_engine.update(&mut self.player,self.game_time);
        self.npc_manager.update(&self.player,&self.world,self.game_time);
        self.vehicle_manager.update(self.game_time);
        self.combat_system.update(&mut self.player,&mut self.npc_manager,self.game_time);
        self.mission_manager.update(&self.player,&self.npc_manager,self.game_time);
        self.world.update_time(dt);
        self.camera.follow(self.player.position+Vec3::new(0.0,1.0,0.0));
        self.ui_manager.update(&self.player,&self.mission_manager,self.game_time);
    }
}
