use glium::{Display, Surface};
use glium::backend::Facade;
use winit::window::Window;
use winit::event_loop::EventLoop;
use crate::camera::Camera;
use crate::player::Player;
use crate::world::World;
use crate::npc::NPCManager;
use crate::vehicle::VehicleManager;
use crate::ui::UIManager;

pub struct Renderer {
    display: glium::Display<glium::glutin::surface::WindowSurface>,
}

impl Renderer {
    pub fn new(window: Window, event_loop: &EventLoop<()>) -> Self {
        use glium::glutin::config::ConfigTemplateBuilder;
        use glium::glutin::display::GlDisplay;
        use glium::glutin::prelude::*;
        use glium::glutin::surface::WindowSurface;

        let (width, height) = {
            let size = window.inner_size();
            (size.width, size.height)
        };

        let config_template = ConfigTemplateBuilder::new()
            .with_transparency(false)
            .build();

        let (window, gl_config) = unsafe {
            glium::glutin::display::Display::new()
                .unwrap()
                .find_configurations(config_template)
                .unwrap()
                .next()
                .unwrap()
        };

        let attrs = glium::glutin::surface::WindowAttributes::default()
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(width as f64, height as f64));

        let surface_attrs = glium::glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(window.raw_window_handle(), std::num::NonZeroU32::new(width).unwrap(), std::num::NonZeroU32::new(height).unwrap());

        // Simple fallback: just create display with default settings
        let display = unsafe {
            glium::Display::new(
                glium::glutin::display::Display::new().unwrap(),
                Default::default(),
            ).expect("Failed to create OpenGL context")
        };

        log::info!("OpenGL context created successfully");

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
