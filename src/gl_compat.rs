//! Windows OpenGL 1.x/2.1 compatibility functions used by the fixed-function renderer.
//!
//! The `gl` crate does not expose every legacy fixed-function entry point that
//! this project uses, so these functions are imported directly from opengl32.

#[link(name = "opengl32")]
extern "system" {
    pub fn glPushMatrix();
    pub fn glPopMatrix();
    pub fn glTranslatef(x: f32, y: f32, z: f32);
    pub fn glRotatef(angle: f32, x: f32, y: f32, z: f32);
    pub fn glScalef(x: f32, y: f32, z: f32);
    pub fn glColor4f(red: f32, green: f32, blue: f32, alpha: f32);
    pub fn glBegin(mode: u32);
    pub fn glEnd();
    pub fn glVertex3f(x: f32, y: f32, z: f32);
}

pub const GL_TRIANGLES: u32 = 0x0004;
