use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::dpi::PhysicalPosition;
use winit::window::{Fullscreen, Window};
use crate::camera::Camera;
use crate::font::FontRenderer;
use crate::npc::NPCManager;
use crate::player::Player;
use crate::ui::UIManager;
use crate::vehicle::VehicleManager;
use crate::world::World;

#[link(name = "opengl32")]
extern "system" {
    fn glMatrixMode(mode:u32); fn glLoadMatrixf(matrix:*const f32); fn glColor3f(red:f32,green:f32,blue:f32); fn glColor4f(red:f32,green:f32,blue:f32,alpha:f32);
    fn glBegin(mode:u32); fn glEnd(); fn glVertex3f(x:f32,y:f32,z:f32); fn glLineWidth(width:f32);
    fn glPushMatrix(); fn glPopMatrix(); fn glTranslatef(x:f32,y:f32,z:f32); fn glRotatef(angle:f32,x:f32,y:f32,z:f32);
}

type WglSwapInterval = unsafe extern "system" fn(interval:i32)->i32;
const GL_PROJECTION:u32=0x1701; const GL_MODELVIEW:u32=0x1700; const GL_QUADS:u32=0x0007; const GL_LINES:u32=0x0001;
type Hwnd=winapi::shared::windef::HWND;

pub struct Renderer { pub(crate) window:Window, hdc:winapi::shared::windef::HDC, hglrc:winapi::shared::windef::HGLRC, wgl_swap_interval:Option<WglSwapInterval>, vsync:bool, font:FontRenderer }

impl Renderer {
    pub fn new(window:Window)->Self {
        let hwnd=match window.raw_window_handle(){RawWindowHandle::Win32(handle)=>handle.hwnd as Hwnd,_=>panic!("OPGAME currently requires a Windows Win32 window")};
        unsafe {
            let hdc=winapi::um::winuser::GetDC(hwnd); if hdc.is_null(){panic!("GetDC failed");}
            let pfd=winapi::um::wingdi::PIXELFORMATDESCRIPTOR{nSize:std::mem::size_of::<winapi::um::wingdi::PIXELFORMATDESCRIPTOR>() as u16,nVersion:1,dwFlags:winapi::um::wingdi::PFD_DRAW_TO_WINDOW|winapi::um::wingdi::PFD_SUPPORT_OPENGL|winapi::um::wingdi::PFD_DOUBLEBUFFER,iPixelType:winapi::um::wingdi::PFD_TYPE_RGBA,cColorBits:32,cDepthBits:24,cStencilBits:8,iLayerType:winapi::um::wingdi::PFD_MAIN_PLANE,..std::mem::zeroed()};
            let format=winapi::um::wingdi::ChoosePixelFormat(hdc,&pfd); if format==0||winapi::um::wingdi::SetPixelFormat(hdc,format,&pfd)==0{panic!("Could not configure the OpenGL pixel format");}
            let hglrc=winapi::um::wingdi::wglCreateContext(hdc); if hglrc.is_null()||winapi::um::wingdi::wglMakeCurrent(hdc,hglrc)==0{panic!("Could not create the OpenGL context");}
            gl::load_with(|symbol|{let symbol=match CString::new(symbol){Ok(s)=>s,Err(_)=>return std::ptr::null()};let address=winapi::um::wingdi::wglGetProcAddress(symbol.as_ptr());if !address.is_null(){return address as *const c_void;}let module=winapi::um::libloaderapi::GetModuleHandleA(b"opengl32.dll\0".as_ptr() as *const c_char);if module.is_null(){return std::ptr::null();}winapi::um::libloaderapi::GetProcAddress(module,symbol.as_ptr()) as *const c_void});
            let wgl_swap_interval={let name=CString::new("wglSwapIntervalEXT").unwrap();let address=winapi::um::wingdi::wglGetProcAddress(name.as_ptr());if address.is_null(){None}else{Some(std::mem::transmute::<*const c_void,WglSwapInterval>(address as *const c_void))}};
            gl::Enable(gl::DEPTH_TEST); gl::DepthFunc(gl::LEQUAL); gl::ClearColor(0.38,0.58,0.82,1.0); gl::Viewport(0,0,1280,720);
            let font=FontRenderer::new(26.0).unwrap_or_else(|e|panic!("Font initialization failed: {}",e));
            let renderer=Self{window,hdc,hglrc,wgl_swap_interval,vsync:true,font}; renderer.apply_vsync(); renderer
        }
    }
    fn apply_vsync(&self){if let Some(f)=self.wgl_swap_interval{unsafe{f(if self.vsync{1}else{0});}}}
    pub fn toggle_vsync(&mut self){self.vsync=!self.vsync;self.apply_vsync();log::info!("VSync {}",if self.vsync{"enabled"}else{"disabled"});}
    pub fn toggle_fullscreen(&self){if self.window.fullscreen().is_some(){self.window.set_fullscreen(None);}else{self.window.set_fullscreen(Some(Fullscreen::Borderless(None)));}self.window.request_redraw();}
    pub fn set_cursor_visible(&self,visible:bool){self.window.set_cursor_visible(visible);}
    pub fn center_cursor(&self){let s=self.window.inner_size();let p=PhysicalPosition::new(s.width as f64/2.0,s.height as f64/2.0);let _=self.window.set_cursor_position(p);}
    pub fn window_center(&self)->(f64,f64){let s=self.window.inner_size();(s.width as f64/2.0,s.height as f64/2.0)}
    pub fn resize(&mut self,width:u32,height:u32){unsafe{gl::Viewport(0,0,width as i32,height as i32);}}

    pub fn render(&self,camera:&Camera,player:&Player,world:&World,npc_manager:&NPCManager,vehicle_manager:&VehicleManager,_ui_manager:&UIManager,settings_menu:bool,selected:usize,sensitivity:f32,invert_x:bool,invert_y:bool){
        unsafe{
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);
            let projection=camera.projection_matrix();let view=camera.view_matrix();glMatrixMode(GL_PROJECTION);glLoadMatrixf(projection.to_cols_array().as_ptr());glMatrixMode(GL_MODELVIEW);glLoadMatrixf(view.to_cols_array().as_ptr());
            draw_ground();for road in &world.roads{draw_road(road.start,road.end,road.width);}for object in world.objects.values(){draw_building(object.position);}draw_player(player.position,player.rotation);
            if settings_menu { draw_settings_menu(&self.font,selected,self.vsync,sensitivity,invert_x,invert_y); }
            gl::Flush();winapi::um::wingdi::SwapBuffers(self.hdc);
        }
        log::debug!("Rendered: {} roads, {} objects, {} NPCs, {} vehicles; player at {:?}",world.roads.len(),world.objects.len(),npc_manager.get_npcs().len(),vehicle_manager.get_vehicles().len(),player.position);
    }
}

impl Drop for Renderer{fn drop(&mut self){unsafe{winapi::um::wingdi::wglMakeCurrent(std::ptr::null_mut(),std::ptr::null_mut());winapi::um::wingdi::wglDeleteContext(self.hglrc);if let RawWindowHandle::Win32(handle)=self.window.raw_window_handle(){winapi::um::winuser::ReleaseDC(handle.hwnd as Hwnd,self.hdc);}}}}

unsafe fn draw_settings_menu(font:&FontRenderer,selected:usize,vsync:bool,sensitivity:f32,invert_x:bool,invert_y:bool){
    glMatrixMode(GL_PROJECTION);glLoadMatrixf(glam::Mat4::orthographic_rh_gl(-1.0,1.0,-1.0,1.0,-1.0,1.0).to_cols_array().as_ptr());glMatrixMode(GL_MODELVIEW);glLoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST); gl::Enable(gl::BLEND); gl::BlendFunc(gl::SRC_ALPHA,gl::ONE_MINUS_SRC_ALPHA);

    // A simple translucent tint keeps the game visible underneath the menu.
    glColor4f(0.015,0.02,0.03,0.82);glBegin(GL_QUADS);glVertex3f(-1.0,-1.0,0.0);glVertex3f(1.0,-1.0,0.0);glVertex3f(1.0,1.0,0.0);glVertex3f(-1.0,1.0,0.0);glEnd();

    let labels=["VSync","Mouse Sensitivity","Invert X","Invert Y","Fullscreen","QUIT GAME"];
    let values=[if vsync{"ON"}else{"OFF"},"","", "","F11",""];
    let rows=[0.36,0.17,-0.02,-0.21,-0.40,-0.62];

    font.draw_text("SETTINGS", -0.22, 0.68, [255,255,255,255]);
    for i in 0..labels.len(){
        let y=rows[i];
        let active=i==selected;
        let label_color=if active{[255,255,255,255]}else{[215,215,220,255]};
        font.draw_text(if active {">"} else {" \