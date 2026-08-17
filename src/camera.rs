use glam::{Mat4, Vec3};

/// Third-person orbit camera used by the game.
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
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
        }
    }

    pub fn look_at(&mut self, target: Vec3) {
        self.target = target;
    }

    /// Horizontal direction used for player movement.
    pub fn flat_forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos()).normalize()
    }

    pub fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(
            self.yaw.sin() * cp,
            self.pitch.sin(),
            self.yaw.cos() * cp,
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.flat_forward().cross(self.up).normalize()
    }

    /// Rotate the camera from raw mouse movement.
    ///
    /// The signs intentionally make the camera feel natural for the current
    /// third-person view: moving the mouse right looks right, and moving it
    /// up looks up.
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx;
        self.pitch = (self.pitch + dy).clamp(-1.45, 1.45);
    }

    /// Rebuild the camera position around a world-space target.
    pub fn follow(&mut self, target: Vec3) {
        self.target = target;
        self.position = target - self.forward() * self.distance;
    }

    pub fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance - amount).clamp(2.5, 15.0);
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect.max(0.01), 0.1, 1000.0)
    }
}
