use crate::camera::Camera;
use crate::player::Player;
use crate::renderer::Renderer;
use glam::{Vec3, Quat};
use winit::event::KeyEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Game {
    renderer: Renderer,
    player: Player,
    camera: Camera,
    input_state: InputState,
}

struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
}

impl Game {
    pub async fn new(window: &winit::window::Window) -> Self {
        let renderer = Renderer::new(window).await;
        let player = Player::new(Vec3::ZERO);
        let camera = Camera::new();
        
        let input_state = InputState {
            forward: false,
            backward: false,
            left: false,
            right: false,
            jump: false,
        };

        Self {
            renderer,
            player,
            camera,
            input_state,
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        let pressed = event.state == winit::event::ElementState::Pressed;
        
        if let PhysicalKey::Code(code) = event.physical_key {
            match code {
                KeyCode::KeyW => self.input_state.forward = pressed,
                KeyCode::KeyS => self.input_state.backward = pressed,
                KeyCode::KeyA => self.input_state.left = pressed,
                KeyCode::KeyD => self.input_state.right = pressed,
                KeyCode::Space => self.input_state.jump = pressed,
                KeyCode::Escape => std::process::exit(0),
                _ => {}
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub async fn update_and_render(&mut self) {
        self.update();
        self.renderer.render(&self.camera, &self.player).await;
    }

    fn update(&mut self) {
        let mut velocity = Vec3::ZERO;
        let speed = 0.1;

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
        self.camera.position = self.player.position + Vec3::new(0.0, 2.0, -5.0);
        self.camera.look_at(self.player.position + Vec3::new(0.0, 1.0, 0.0));
    }
}
