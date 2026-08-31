mod assets;
mod settings;
mod game;
mod camera;
mod player;
mod entity;
mod collision;
mod renderer;
mod model;
mod gl;
mod world;
mod npc;
mod vehicle;
mod combat;
mod physics;
mod ui;
mod mission;
mod sound;
mod font;

use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin_winit::DisplayBuilder;
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use game::Game;
use settings::Settings;

fn main() {
    env_logger::init();
    let settings = Settings::load();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window_builder = WindowBuilder::new()
        .with_title("OPGAME - OpenGL 2.1+")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
    let requested_msaa = settings.graphics.msaa_samples;
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_depth_size(24)
        .with_stencil_size(8)
        .with_multisampling(requested_msaa);
    let (window, gl_config) = DisplayBuilder::new().with_window_builder(Some(window_builder)).build(&event_loop, template, |configs| {
        configs
            .max_by_key(|config| if requested_msaa == 0 && config.num_samples() == 0 { 10_000 } else if config.num_samples() == requested_msaa { 20_000 } else { config.num_samples() as i32 })
            .expect("No suitable OpenGL configuration found")
    }).expect("Failed to create window");
    let window = window.expect("Window builder was provided, so a window must exist");
    log::info!("Requested MSAA {}x, selected OpenGL config with {} samples", requested_msaa, gl_config.num_samples());
    let mut game = Game::new(window, gl_config, settings);
    let _ = event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { window_id: _, event: WindowEvent::CloseRequested } => target.exit(),
            Event::WindowEvent { window_id: _, event: WindowEvent::Resized(size) } => game.resize(size.width, size.height),
            Event::WindowEvent { window_id: _, event: WindowEvent::KeyboardInput { event, .. } } => game.handle_key(event),
            Event::WindowEvent { window_id: _, event: WindowEvent::CursorMoved { position, .. } } => game.handle_cursor_moved(position.x, position.y),
            Event::WindowEvent { window_id: _, event: WindowEvent::MouseInput { state, button, .. } } => game.handle_mouse_click(state, button),
            Event::AboutToWait => game.update_and_render(),
            _ => {}
        }
        target.set_control_flow(ControlFlow::Poll);
    });
}
