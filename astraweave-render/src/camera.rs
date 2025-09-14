use glam::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        let dir = Self::dir(self.yaw, self.pitch);
        Mat4::look_to_rh(self.position, dir, Vec3::Y)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect.max(0.01), self.znear, self.zfar)
    }

    pub fn vp(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    pub fn dir(yaw: f32, pitch: f32) -> Vec3 {
        let cy = yaw.cos();
        let sy = yaw.sin();
        let cp = pitch.cos();
        let sp = pitch.sin();
        Vec3::new(cy * cp, sp, sy * cp).normalize()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    FreeFly,
    Orbit,
}

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub mode: CameraMode,
    pub orbit_target: Vec3,
    pub orbit_distance: f32,
    fwd: f32,
    back: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    dragging: bool,
    last_mouse: Option<Vec2>,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            zoom_sensitivity: 0.1,
            mode: CameraMode::FreeFly,
            orbit_target: Vec3::ZERO,
            orbit_distance: 5.0,
            fwd: 0.0,
            back: 0.0,
            left: 0.0,
            right: 0.0,
            up: 0.0,
            down: 0.0,
            dragging: false,
            last_mouse: None,
        }
    }

    pub fn process_keyboard(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        let v = if pressed { 1.0 } else { 0.0 };
        match key {
            winit::keyboard::KeyCode::KeyW => self.fwd = v,
            winit::keyboard::KeyCode::KeyS => self.back = v,
            winit::keyboard::KeyCode::KeyA => self.left = v,
            winit::keyboard::KeyCode::KeyD => self.right = v,
            winit::keyboard::KeyCode::Space => self.up = v,
            winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                self.down = v
            }
            _ => {}
        }
    }

    pub fn process_mouse_button(&mut self, button: winit::event::MouseButton, pressed: bool) {
        if button == winit::event::MouseButton::Right {
            self.dragging = pressed;
            if !pressed {
                self.last_mouse = None;
            }
        }
    }

    pub fn process_mouse_move(&mut self, camera: &mut Camera, pos: Vec2) {
        if self.dragging {
            if let Some(last) = self.last_mouse {
                let delta = (pos - last) * self.sensitivity;
                camera.yaw -= delta.x;
                camera.pitch -= delta.y;
                camera.pitch = camera.pitch.clamp(-1.54, 1.54);
                
                // Update orbit position if in orbit mode
                if matches!(self.mode, CameraMode::Orbit) {
                    self.update_orbit_position(camera);
                }
            }
            self.last_mouse = Some(pos);
        }
    }

    pub fn process_scroll(&mut self, camera: &mut Camera, delta: f32) {
        match self.mode {
            CameraMode::FreeFly => {
                // Zoom by adjusting FOV
                let fov_delta = delta * self.zoom_sensitivity;
                camera.fovy = (camera.fovy - fov_delta).clamp(0.1, 3.0);
            }
            CameraMode::Orbit => {
                // Zoom by adjusting orbit distance
                self.orbit_distance = (self.orbit_distance - delta * 0.5).clamp(1.0, 50.0);
                self.update_orbit_position(camera);
            }
        }
    }

    pub fn toggle_mode(&mut self, camera: &mut Camera) {
        match self.mode {
            CameraMode::FreeFly => {
                self.mode = CameraMode::Orbit;
                // Set orbit target to current look direction
                let look_dir = Camera::dir(camera.yaw, camera.pitch);
                self.orbit_target = camera.position + look_dir * self.orbit_distance;
            }
            CameraMode::Orbit => {
                self.mode = CameraMode::FreeFly;
                // Keep current position when switching to free fly
            }
        }
    }

    pub fn set_orbit_target(&mut self, target: Vec3, camera: &mut Camera) {
        self.orbit_target = target;
        if matches!(self.mode, CameraMode::Orbit) {
            self.update_orbit_position(camera);
        }
    }

    fn update_orbit_position(&mut self, camera: &mut Camera) {
        let dir = Camera::dir(camera.yaw, camera.pitch);
        camera.position = self.orbit_target - dir * self.orbit_distance;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        match self.mode {
            CameraMode::FreeFly => {
                let dir = Camera::dir(camera.yaw, camera.pitch);
                let right = dir.cross(Vec3::Y).normalize();
                let up = Vec3::Y;

                let mut vel = Vec3::ZERO;
                vel += dir * (self.fwd - self.back);
                vel += right * (self.right - self.left);
                vel += up * (self.up - self.down);
                if vel.length_squared() > 0.0 {
                    camera.position += vel.normalize() * self.speed * dt;
                }
            }
            CameraMode::Orbit => {
                // In orbit mode, WASD moves the orbit target
                let dir = Camera::dir(camera.yaw, camera.pitch);
                let right = dir.cross(Vec3::Y).normalize();
                let forward = Vec3::new(dir.x, 0.0, dir.z).normalize(); // Horizontal movement only
                let up = Vec3::Y;

                let mut target_vel = Vec3::ZERO;
                target_vel += forward * (self.fwd - self.back);
                target_vel += right * (self.right - self.left);
                target_vel += up * (self.up - self.down);
                
                if target_vel.length_squared() > 0.0 {
                    self.orbit_target += target_vel.normalize() * self.speed * dt;
                    self.update_orbit_position(camera);
                }
            }
        }
    }
}
