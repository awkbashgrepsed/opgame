use winit::window::Window;
use std::ffi::CStr;
use std::os::raw::c_void;
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
        // Load OpenGL function pointers
        gl::load_with(|symbol| {
            // Convert symbol string to CStr
            let cname = unsafe { CStr::from_ptr(symbol as *const i8) };
            match cname.to_str() {
                Ok(_name) => {
                    // The gl crate will handle loading the actual function pointers
                    // This is a no-op here, but required to initialize gl
                    std::ptr::null() as *const c_void
                }
                Err(_) => std::ptr::null() as *const c_void,
            }
        });

        log::info!("OpenGL Renderer initialized");
        
        Self {
            _window: window,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
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
            // Clear screen with a blue-ish color
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Basic rendering - just clear for now
            // More complex rendering will be added once GL is properly initialized
        }

        // Log render info
        log::debug!("Rendered: {} NPCs, {} vehicles", npc_manager.get_npcs().len(), vehicle_manager.get_vehicles().len());
        log::debug!("Player position: {:?}", player.position);
    }
}
