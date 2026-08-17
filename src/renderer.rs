use std::ffi::CString;
use std::os::raw::{c_char, c_void};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::dpi::PhysicalPosition;
use winit::window::{Fullscreen, Window};

use crate::camera::Camera;
use crate::npc::NPCManager;
use crate::player::Player;
use crate::ui::UIManager;
use crate::vehicle::VehicleManager;
use crate::world::World;

#[link(name = "opengl32")]
extern "system" {
    fn glMatrixMode(mode: u32);
    fn glLoadMatrixf(matrix: *const f32);
    fn glColor3f(red: f32, green: f32, blue: f32);
    fn glBegin(mode: u32);
    fn glEnd();
    fn glVertex3f(x: f32, y: f32, z: f32);
    fn glLineWidth(width: f32);
    fn glPushMatrix();
    fn glPopMatrix();
    fn glTranslatef(x: f32, y: f32, z: f32);
    fn glRotatef(angle: f32, x: f32, y: f32, z: f32);
}

type WglSwapInterval = unsafe extern "system" fn(interval: i32) -> i32;

const GL_PROJECTION: u32 = 0x1701;
const GL_MODELVIEW: u32 = 0x1700;
const GL_QUADS: u32 = 0x0007;
const GL_LINES: u32 = 0x0001;

type Hwnd = winapi::shared::windef::HWND;

pub struct Renderer {
    pub(crate) window: Window,
    hdc: winapi::shared::windef::HDC,
    hglrc: winapi::shared::windef::HGLRC,
    wgl_swap_interval: Option<WglSwapInterval>,
    vsync: bool,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let hwnd = match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => handle.hwnd as Hwnd,
            _ => panic!("OPGAME currently requires a Windows Win32 window"),
        };

        unsafe {
            let hdc = winapi::um::winuser::GetDC(hwnd);
            if hdc.is_null() { panic!("GetDC failed"); }
            let pixel_format = winapi::um::wingdi::PIXELFORMATDESCRIPTOR {
                nSize: std::mem::size_of::<winapi::um::wingdi::PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: winapi::um::wingdi::PFD_DRAW_TO_WINDOW | winapi::um::wingdi::PFD_SUPPORT_OPENGL | winapi::um::wingdi::PFD_DOUBLEBUFFER,
                iPixelType: winapi::um::wingdi::PFD_TYPE_RGBA,
                cColorBits: 32, cDepthBits: 24, cStencilBits: 8,
                iLayerType: winapi::um::wingdi::PFD_MAIN_PLANE,
                ..std::mem::zeroed()
            };
            let format = winapi::um::wingdi::ChoosePixelFormat(hdc, &pixel_format);
            if format == 0 || winapi::um::wingdi::SetPixelFormat(hdc, format, &pixel_format) == 0 { panic!("Could not configure the OpenGL pixel format"); }
            let hglrc = winapi::um::wingdi::wglCreateContext(hdc);
            if hglrc.is_null() || winapi::um::wingdi::wglMakeCurrent(hdc, hglrc) == 0 { panic!("Could not create the OpenGL context"); }
            gl::load_with(|symbol| {
                let symbol = match CString::new(symbol) { Ok(s) => s, Err(_) => return std::ptr::null() };
                let address = winapi::um::wingdi::wglGetProcAddress(symbol.as_ptr());
                if !address.is_null() { return address as *const c_void; }
                let module = winapi::um::libloaderapi::GetModuleHandleA(b"opengl32.dll\0".as_ptr() as *const c_char);
                if module.is_null() { return std::ptr::null(); }
                winapi::um::libloaderapi::GetProcAddress(module, symbol.as_ptr()) as *const c_void
            });

            let wgl_swap_interval = {
                let name = CString::new("wglSwapIntervalEXT").unwrap();
                let address = winapi::um::wingdi::wglGetProcAddress(name.as_ptr());
                if address.is_null() {
                    None
                } else {
                    Some(std::mem::transmute::<*const c_void, WglSwapInterval>(address as *const c_void))
                }
            };

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::ClearColor(0.38, 0.58, 0.82, 1.0);
            gl::Viewport(0, 0, 1280, 720);

            let renderer = Self { window, hdc, hglrc, wgl_swap_interval, vsync: true };
            renderer.apply_vsync();
            log::info!("OpenGL context initialized (VSync enabled)");
            renderer
        }
    }

    fn apply_vsync(&self) {
        if let Some(swap_interval) = self.wgl_swap_interval {
            unsafe { swap_interval(if self.vsync { 1 } else { 0 }); }
        }
    }

    pub fn toggle_vsync(&mut self) {
        self.vsync = !self.vsync;
        self.apply_vsync();
        log::info!("VSync {}", if self.vsync { "enabled" } else { "disabled" });
    }

    pub fn toggle_fullscreen(&self) {
        if self.window.fullscreen().is_some() {
            self.window.set_fullscreen(None);
        } else {
            self.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        self.window.request_redraw();
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    pub fn center_cursor(&self) {
        let size = self.window.inner_size();
        let center = PhysicalPosition::new(size.width as f64 / 2.0, size.height as f64 / 2.0);
        let _ = self.window.set_cursor_position(center);
    }

    pub fn window_center(&self) -> (f64, f64) {
        let size = self.window.inner_size();
        (size.width as f64 / 2.0, size.height as f64 / 2.0)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe { gl::Viewport(0, 0, width as i32, height as i32); }
    }

    pub fn render(&self, camera: &Camera, player: &Player, world: &World, npc_manager: &NPCManager, vehicle_manager: &VehicleManager, _ui_manager: &UIManager) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let projection = camera.projection_matrix();
            let view = camera.view_matrix();
            glMatrixMode(GL_PROJECTION); glLoadMatrixf(projection.to_cols_array().as_ptr());
            glMatrixMode(GL_MODELVIEW); glLoadMatrixf(view.to_cols_array().as_ptr());
            draw_ground();
            for road in &world.roads { draw_road(road.start, road.end, road.width); }
            for object in world.objects.values() { draw_building(object.position); }
            draw_player(player.position, player.rotation);
            gl::Flush();
            winapi::um::wingdi::SwapBuffers(self.hdc);
        }
        log::debug!("Rendered: {} roads, {} objects, {} NPCs, {} vehicles; player at {:?}", world.roads.len(), world.objects.len(), npc_manager.get_npcs().len(), vehicle_manager.get_vehicles().len(), player.position);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            winapi::um::wingdi::wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut());
            winapi::um::wingdi::wglDeleteContext(self.hglrc);
            if let RawWindowHandle::Win32(handle) = self.window.raw_window_handle() {
                let hwnd = handle.hwnd as Hwnd;
                winapi::um::winuser::ReleaseDC(hwnd, self.hdc);
            }
        }
    }
}

unsafe fn draw_ground() {
    glColor3f(0.18, 0.38, 0.18); glBegin(GL_QUADS);
    glVertex3f(-500.0,0.0,-500.0); glVertex3f(500.0,0.0,-500.0); glVertex3f(500.0,0.0,500.0); glVertex3f(-500.0,0.0,500.0); glEnd();
}
unsafe fn draw_road(start: glam::Vec3, end: glam::Vec3, width: f32) {
    let delta=end-start; if delta.length_squared()<f32::EPSILON{return;} let direction=delta.normalize(); let side=glam::Vec3::new(-direction.z,0.0,direction.x)*(width*0.5);
    glColor3f(0.12,0.12,0.13); glBegin(GL_QUADS);
    glVertex3f((start-side).x,0.01,(start-side).z); glVertex3f((start+side).x,0.01,(start+side).z); glVertex3f((end+side).x,0.01,(end+side).z); glVertex3f((end-side).x,0.01,(end-side).z); glEnd();
    glColor3f(0.85,0.78,0.18); glLineWidth(2.0); glBegin(GL_LINES); glVertex3f(start.x,0.025,start.z); glVertex3f(end.x,0.025,end.z); glEnd();
}
unsafe fn draw_building(position: glam::Vec3) {
    let width=14.0; let depth=14.0; let height=8.0+((position.x.abs()+position.z.abs())%3.0)*4.0; let x0=position.x-width*0.5; let x1=position.x+width*0.5; let z0=position.z-depth*0.5; let z1=position.z+depth*0.5;
    glColor3f(0.55,0.50,0.43); glBegin(GL_QUADS);
    glVertex3f(x0,0.0,z1);glVertex3f(x1,0.0,z1);glVertex3f(x1,height,z1);glVertex3f(x0,height,z1); glVertex3f(x1,0.0,z0);glVertex3f(x0,0.0,z0);glVertex3f(x0,height,z0);glVertex3f(x1,height,z0); glVertex3f(x0,0.0,z0);glVertex3f(x0,0.0,z1);glVertex3f(x0,height,z1);glVertex3f(x0,height,z0); glVertex3f(x1,0.0,z1);glVertex3f(x1,0.0,z0);glVertex3f(x1,height,z0);glVertex3f(x1,height,z1); glVertex3f(x0,height,z0);glVertex3f(x0,height,z1);glVertex3f(x1,height,z1);glVertex3f(x1,height,z0); glEnd();
}
unsafe fn draw_player(position: glam::Vec3, rotation: f32) {
    let size=0.8;
    glPushMatrix(); glTranslatef(position.x,position.y,position.z); glRotatef(rotation.to_degrees(),0.0,1.0,0.0);
    glColor3f(0.15,0.35,1.0); glBegin(GL_QUADS); glVertex3f(-size,0.0,size);glVertex3f(size,0.0,size);glVertex3f(size,size*2.25,size);glVertex3f(-size,size*2.25,size);glEnd();
    glColor3f(0.05,0.85,0.9); glBegin(GL_QUADS); glVertex3f(size,0.0,-size);glVertex3f(-size,0.0,-size);glVertex3f(-size,size*2.25,-size);glVertex3f(size,size*2.25,-size);glEnd();
    glColor3f(1.0,0.15,0.12); glBegin(GL_QUADS); glVertex3f(size,0.0,size);glVertex3f(size,0.0,-size);glVertex3f(size,size*2.25,-size);glVertex3f(size,size*2.25,size);glEnd();
    glColor3f(1.0,0.55,0.08); glBegin(GL_QUADS); glVertex3f(-size,0.0,-size);glVertex3f(-size,0.0,size);glVertex3f(-size,size*2.25,size);glVertex3f(-size,size*2.25,-size);glEnd();
    glColor3f(0.15,0.9,0.2); glBegin(GL_QUADS); glVertex3f(-size,size*2.25,size);glVertex3f(size,size*2.25,size);glVertex3f(size,size*2.25,-size);glVertex3f(-size,size*2.25,-size);glEnd();
    glColor3f(0.8,0.1,0.75); glBegin(GL_QUADS); glVertex3f(-size,0.0,-size);glVertex3f(size,0.0,-size);glVertex3f(size,0.0,size);glVertex3f(-size,0.0,size);glEnd();
    glPopMatrix();
}
