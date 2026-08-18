use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub sensitivity: f32,
    pub invert_x: bool,
    pub invert_y: bool,
    pub shoulder_side: f32,
    pub shoulder_offset: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 3.0, -6.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            up: Vec3::Y,
            fov: 60.0,
            aspect: 1280.0 / 720.0,
            yaw: 0.0,
            pitch: (-15.0_f32).to_radians(),
            distance: 6.0,
            sensitivity: 0.004,
            invert_x: false,
            invert_y: false,
            shoulder_side: 1.0,
            shoulder_offset: 1.35,
        }
    }

    pub fn flat_forward(&self) -> Vec3 { Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()).normalize() }

    pub fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(self.yaw.sin() * cp, self.pitch.sin(), self.yaw.cos() * cp).normalize()
    }

    pub fn right(&self) -> Vec3 { self.flat_forward().cross(self.up).normalize() }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        let x = if self.invert_x { -dx } else { dx };
        let y = if self.invert_y { -dy } else { dy };
        self.yaw -= x;
        self.pitch = (self.pitch + y).clamp(-1.45, 1.45);
    }

    pub fn toggle_shoulder(&mut self) {
        self.shoulder_side = -self.shoulder_side;
    }

    pub fn follow(&mut self, target: Vec3) {
        self.target = target;
        self.position = target - self.forward() * self.distance
            + self.right() * (self.shoulder_side * self.shoulder_offset);
    }

    pub fn zoom(&mut self, amount: f32) { self.distance = (self.distance - amount).clamp(2.5, 15.0); }
    pub fn view_matrix(&self) -> Mat4 { Mat4::look_at_rh(self.position, self.target, self.up) }
    pub fn projection_matrix(&self) -> Mat4 { Mat4::perspective_rh(self.fov.to_radians(), self.aspect.max(0.01), 0.1, 1000.0) }
}
