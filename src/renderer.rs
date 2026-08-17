use glium::{Display, Surface};
use winit::window::Window;
use crate::camera::Camera;
use crate::player::Player;
use crate::world::World;
use crate::npc::NPCManager;
use crate::vehicle::VehicleManager;
use crate::ui::UIManager;
use glam::Vec3;

pub struct Renderer {
    display: Display,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let display = glium::Display::new(
            window,
            glium::glutin::context::ContextAttributeFlags::empty(),
            Default::default(),
        ).expect("Failed to create OpenGL context");

        // Set OpenGL to compatibility mode for older versions
        unsafe {
            let gl = display.get_context();
            gl.make_current().expect("Failed to make context current");
        }

        log::info!("OpenGL version: {:?}", display.get_opengl_version());
        log::info!("Renderer: {}", display.get_opengl_renderer_string());

        Self { display }
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        // Glium handles resizing automatically
    }

    pub fn render(
        &self,
        _camera: &Camera,
        player: &Player,
        _world: &World,
        npc_manager: &NPCManager,
        vehicle_manager: &VehicleManager,
        _ui_manager: &UIManager,
    ) {
        let mut frame = self.display.draw();
        frame.clear_color(0.1, 0.2, 0.3, 1.0);

        // Render simple debug grid
        self.render_grid(&mut frame);

        frame.finish().expect("Failed to finish frame");

        // Log render info
        log::debug!("Rendered: {} NPCs, {} vehicles", npc_manager.get_npcs().len(), vehicle_manager.get_vehicles().len());
        log::debug!("Player position: {:?}", player.position);
    }

    fn render_grid(&self, _frame: &mut glium::Frame) {
        // TODO: Render a simple grid using OpenGL 2.1 immediate mode
    }
}
