use winit::window::Window;
use std::os::raw::{c_char, c_void};

use crate::camera::Camera;
use crate::player::Player;
use crate::world::World;
use crate::npc::NPCManager;
use crate::vehicle::VehicleManager;
use crate::ui::UIManager;

pub struct Renderer {
    _window: Window,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        // Load OpenGL function pointers.
        //
        // gl::load_with() gives us the OpenGL function name as a &str.
        // On Windows, we need to ask WGL for the actual function address.
        gl::load_with(|symbol| {
            let symbol = match std::ffi::CString::new(symbol) {
                Ok(s) => s,
                Err(_) => return std::ptr::null(),
            };

            unsafe {
                // First try WGL.
                let address =
                    winapi::um::wingdi::wglGetProcAddress(symbol.as_ptr());

                if !address.is_null() {
                    return address as *const c_void;
                }

                // Some basic OpenGL functions are exported directly
                // by opengl32.dll, so fall back to GetProcAddress.
                let module =
                    winapi::um::libloaderapi::GetModuleHandleA(
                        b"opengl32.dll\0".as_ptr() as *const c_char
                    );

                if module.is_null() {
                    return std::ptr::null();
                }

                winapi::um::libloaderapi::GetProcAddress(
                    module,
                    symbol.as_ptr(),
                ) as *const c_void
            }
        });

        log::info!("OpenGL Renderer initialized");

        Self {
            _window: window,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(
                0,
                0,
                width as i32,
                height as i32,
            );
        }
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
        unsafe {
            // Clear screen with a blue-ish color.
            gl::ClearColor(
                0.1,
                0.2,
                0.3,
                1.0,
            );

            gl::Clear(
                gl::COLOR_BUFFER_BIT |
                gl::DEPTH_BUFFER_BIT,
            );

            // Basic rendering - just clear for now.
            // More complex rendering will be added once
            // OpenGL is properly initialized.
        }

        log::debug!(
            "Rendered: {} NPCs, {} vehicles",
            npc_manager.get_npcs().len(),
            vehicle_manager.get_vehicles().len()
        );

        log::debug!(
            "Player position: {:?}",
            player.position
        );
    }
}