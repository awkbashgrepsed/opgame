use std::os::raw::{c_char, c_void};
use winit::window::Window;

use crate::camera::Camera;
use crate::npc::NPCManager;
use crate::player::Player;
use crate::ui::UIManager;
use crate::vehicle::VehicleManager;
use crate::world::World;

pub struct Renderer {
    _window: Window,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        // Load OpenGL function pointers.
        gl::load_with(|symbol| {
            let symbol = match std::ffi::CString::new(symbol) {
                Ok(s) => s,
                Err(_) => return std::ptr::null(),
            };

            unsafe {
                let address = winapi::um::wingdi::wglGetProcAddress(symbol.as_ptr());
                if !address.is_null() {
                    return address as *const c_void;
                }

                let module = winapi::um::libloaderapi::GetModuleHandleA(
                    b"opengl32.dll\0".as_ptr() as *const c_char,
                );

                if module.is_null() {
                    return std::ptr::null();
                }

                winapi::um::libloaderapi::GetProcAddress(module, symbol.as_ptr()) as *const c_void
            }
        });

        log::info!("OpenGL Renderer initialized");
        Self { _window: window }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    pub fn render(
        &self,
        camera: &Camera,
        player: &Player,
        world: &World,
        npc_manager: &NPCManager,
        vehicle_manager: &VehicleManager,
        _ui_manager: &UIManager,
    ) {
        // Build the matrices now so the renderer has a real camera pipeline
        // ready for the upcoming shader/mesh implementation.
        let _view = camera.view_matrix();
        let _projection = camera.projection_matrix();

        unsafe {
            gl::ClearColor(0.08, 0.12, 0.18, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        log::debug!(
            "Rendered map: {} roads, {} objects; {} NPCs, {} vehicles",
            world.roads.len(),
            world.objects.len(),
            npc_manager.get_npcs().len(),
            vehicle_manager.get_vehicles().len(),
        );

        log::debug!("Player position: {:?}", player.position);
    }
}
