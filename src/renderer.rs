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

    pub fn render(&self,camera:&Camera,player:&Player,world:&World,npc_manager:&NPCManager,vehicle_manager:&VehicleManager,_ui_manager:&UIManager,settings_menu:bool,selected:usize,sensitivity:f32,invert_x:bool,invert_y:bool,aiming:bool){
        unsafe{
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);
            let projection=camera.projection_matrix();let view=camera.view_matrix();glMatrixMode(GL_PROJECTION);glLoadMatrixf(projection.to_cols_array().as_ptr());glMatrixMode(GL_MODELVIEW);glLoadMatrixf(view.to_cols_array().as_ptr());
            draw_ground();for road in &world.roads{draw_road(road.start,road.end,road.width);}for object in world.objects.values(){draw_building(object.position);}draw_player(player.position,player.rotation);
            if settings_menu { draw_settings_menu(&self.font,selected,self.vsync,sensitivity,invert_x,invert_y); }
            if aiming && !settings_menu { draw_crosshair(); }
            gl::Flush();winapi::um::wingdi::SwapBuffers(self.hdc);
        }
        log::debug!("Rendered: {} roads, {} objects, {} NPCs, {} vehicles; player at {:?}",world.roads.len(),world.objects.len(),npc_manager.get_npcs().len(),vehicle_manager.get_vehicles().len(),player.position);
    }
}

impl Drop for Renderer{fn drop(&mut self){unsafe{winapi::um::wingdi::wglMakeCurrent(std::ptr::null_mut(),std::ptr::null_mut());winapi::um::wingdi::wglDeleteContext(self.hglrc);if let RawWindowHandle::Win32(handle)=self.window.raw_window_handle(){winapi::um::winuser::ReleaseDC(handle.hwnd as Hwnd,self.hdc);}}}}

unsafe fn draw_crosshair(){
    glMatrixMode(GL_PROJECTION);glLoadMatrixf(glam::Mat4::orthographic_rh_gl(-1.0,1.0,-1.0,1.0,-1.0,1.0).to_cols_array().as_ptr());glMatrixMode(GL_MODELVIEW);glLoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST); glColor3f(1.0,1.0,1.0); glLineWidth(2.0);
    let gap=0.012; let length=0.035;
    glBegin(GL_LINES);
    glVertex3f(-gap-length,0.0,0.0);glVertex3f(-gap,0.0,0.0);
    glVertex3f(gap,0.0,0.0);glVertex3f(gap+length,0.0,0.0);
    glVertex3f(0.0,-gap-length,0.0);glVertex3f(0.0,-gap,0.0);
    glVertex3f(0.0,gap,0.0);glVertex3f(0.0,gap+length,0.0);
    glEnd(); gl::Enable(gl::DEPTH_TEST);
}

unsafe fn draw_settings_menu(font:&FontRenderer,selected:usize,vsync:bool,sensitivity:f32,invert_x:bool,invert_y:bool){
    glMatrixMode(GL_PROJECTION);glLoadMatrixf(glam::Mat4::orthographic_rh_gl(-1.0,1.0,-1.0,1.0,-1.0,1.0).to_cols_array().as_ptr());glMatrixMode(GL_MODELVIEW);glLoadMatrixf(glam::Mat4::IDENTITY.to_cols_array().as_ptr());
    gl::Disable(gl::DEPTH_TEST); gl::Enable(gl::BLEND); gl::BlendFunc(gl::SRC_ALPHA,gl::ONE_MINUS_SRC_ALPHA);
    glColor4f(0.015,0.02,0.03,0.82);glBegin(GL_QUADS);glVertex3f(-1.0,-1.0,0.0);glVertex3f(1.0,-1.0,0.0);glVertex3f(1.0,1.0,0.0);glVertex3f(-1.0,1.0,0.0);glEnd();
    let labels=["VSync","Mouse Sensitivity","Invert X","Invert Y","Fullscreen","QUIT GAME"];
    let sensitivity_text=format!("{:.3}",sensitivity);
    let values=[if vsync{"ON"}else{"OFF"},sensitivity_text.as_str(),if invert_x{"ON"}else{"OFF"},if invert_y{"ON"}else{"OFF"},"F11",""];
    let rows=[0.36,0.17,-0.02,-0.21,-0.40,-0.62];
    font.draw_text("SETTINGS", -0.22, 0.68, [255,255,255,255]);
    for i in 0..labels.len(){
        let y=rows[i]; let active=i==selected;
        let label_color=if active{[255,255,255,255]}else{[215,215,220,255]};
        let marker=if active{">"}else{" "};
        font.draw_text(marker,-0.62,y,label_color);
        font.draw_text(labels[i],-0.54,y,label_color);
        if !values[i].is_empty(){font.draw_text(values[i],0.34,y,[150,230,170,255]);}
    }
    font.draw_text("ENTER: select",-0.62,-0.84,[170,170,180,255]);
    font.draw_text("ESC: close",0.25,-0.84,[170,170,180,255]);
    gl::Disable(gl::BLEND); gl::Enable(gl::DEPTH_TEST);
}

unsafe fn draw_ground(){glColor3f(0.18,0.38,0.18);glBegin(GL_QUADS);glVertex3f(-500.0,0.0,-500.0);glVertex3f(500.0,0.0,-500.0);glVertex3f(500.0,0.0,500.0);glVertex3f(-500.0,0.0,500.0);glEnd();}
unsafe fn draw_road(start:glam::Vec3,end:glam::Vec3,width:f32){let d=end-start;if d.length_squared()<f32::EPSILON{return;}let dir=d.normalize();let side=glam::Vec3::new(-dir.z,0.0,dir.x)*(width*0.5);glColor3f(0.12,0.12,0.13);glBegin(GL_QUADS);glVertex3f((start-side).x,0.01,(start-side).z);glVertex3f((start+side).x,0.01,(start+side).z);glVertex3f((end+side).x,0.01,(end+side).z);glVertex3f((end-side).x,0.01,(end-side).z);glEnd();glColor3f(0.85,0.78,0.18);glLineWidth(2.0);glBegin(GL_LINES);glVertex3f(start.x,0.025,start.z);glVertex3f(end.x,0.025,end.z);glEnd();}
unsafe fn draw_building(position:glam::Vec3){let w=14.0;let d=14.0;let h=8.0+((position.x.abs()+position.z.abs())%3.0)*4.0;let x0=position.x-w*0.5;let x1=position.x+w*0.5;let z0=position.z-d*0.5;let z1=position.z+d*0.5;glColor3f(0.55,0.50,0.43);glBegin(GL_QUADS);glVertex3f(x0,0.0,z1);glVertex3f(x1,0.0,z1);glVertex3f(x1,h,z1);glVertex3f(x0,h,z1);glVertex3f(x1,0.0,z0);glVertex3f(x0,0.0,z0);glVertex3f(x0,h,z0);glVertex3f(x1,h,z0);glVertex3f(x0,0.0,z0);glVertex3f(x0,0.0,z1);glVertex3f(x0,h,z1);glVertex3f(x0,h,z0);glVertex3f(x1,0.0,z1);glVertex3f(x1,0.0,z0);glVertex3f(x1,h,z0);glVertex3f(x1,h,z1);glVertex3f(x0,h,z0);glVertex3f(x0,h,z1);glVertex3f(x1,h,z1);glVertex3f(x1,h,z0);glEnd();}
unsafe fn draw_player(position:glam::Vec3,rotation:f32){let s=0.8;glPushMatrix();glTranslatef(position.x,position.y,position.z);glRotatef(rotation.to_degrees(),0.0,1.0,0.0);glColor3f(0.15,0.35,1.0);glBegin(GL_QUADS);glVertex3f(-s,0.0,s);glVertex3f(s,0.0,s);glVertex3f(s,s*2.25,s);glVertex3f(-s,s*2.25,s);glEnd();glColor3f(0.05,0.85,0.9);glBegin(GL_QUADS);glVertex3f(s,0.0,-s);glVertex3f(-s,0.0,-s);glVertex3f(-s,s*2.25,-s);glVertex3f(s,s*2.25,-s);glEnd();glColor3f(1.0,0.15,0.12);glBegin(GL_QUADS);glVertex3f(s,0.0,s);glVertex3f(s,0.0,-s);glVertex3f(s,s*2.25,-s);glVertex3f(s,s*2.25,s);glEnd();glColor3f(1.0,0.55,0.08);glBegin(GL_QUADS);glVertex3f(-s,0.0,-s);glVertex3f(-s,0.0,s);glVertex3f(-s,s*2.25,s);glVertex3f(-s,s*2.25,-s);glEnd();glColor3f(0.15,0.9,0.2);glBegin(GL_QUADS);glVertex3f(-s,s*2.25,s);glVertex3f(s,s*2.25,s);glVertex3f(s,s*2.25,-s);glVertex3f(-s,s*2.25,-s);glEnd();glColor3f(0.8,0.1,0.75);glBegin(GL_QUADS);glVertex3f(-s,0.0,-s);glVertex3f(s,0.0,-s);glVertex3f(s,0.0,s);glVertex3f(-s,0.0,s);glEnd();glPopMatrix();}