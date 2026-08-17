mod game;
mod camera;
mod player;
mod entity;
mod renderer;
mod world;
mod npc;
mod vehicle;
mod combat;
mod physics;
mod ui;
mod mission;
mod sound;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use game::Game;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("OPGAME - GTA SA-like (OpenGL 2.1+)")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .expect("Failed to create window");

    let mut game = Game::new(window);

    let _ = event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CloseRequested,
            } => target.exit(),
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::Resized(size),
            } => {
                game.resize(size.width, size.height);
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::KeyboardInput { event, .. },
            } => {
                game.handle_key(event);
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CursorMoved { position, .. },
            } => {
                game.handle_cursor_moved(position.x, position.y);
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::MouseInput { state, button, .. },
            } => {
                game.handle_mouse_click(state, button);
            }
            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::MouseWheel { delta, .. },
            } => {
                game.handle_mouse_wheel(delta);
            }
            Event::AboutToWait => {
                game.update_and_render();
            }
            _ => {}
        }
        target.set_control_flow(ControlFlow::Poll);
    });
}
