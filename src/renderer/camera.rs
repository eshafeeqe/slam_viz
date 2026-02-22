use glam::{Mat4, Vec3, Quat};

pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 8.0,
            yaw: 0.5,
            pitch: 0.5,  // positive = above target
            fov_y: 60_f32.to_radians(),
        }
    }

    pub fn position(&self) -> Vec3 {
        // Negate pitch so positive pitch = camera above target (intuitive convention)
        let rot = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, -self.pitch, 0.0);
        self.target + rot * Vec3::new(0.0, 0.0, self.distance)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, aspect, 0.01, 1000.0)
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.proj_matrix(aspect) * self.view_matrix()
    }

    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * 0.005;
        self.pitch = (self.pitch - delta_y * 0.005).clamp(-1.5, 1.5);
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let rot = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        let right = rot * Vec3::X;
        let up = rot * Vec3::Y;
        let scale = self.distance * 0.001;
        self.target -= right * delta_x * scale;
        self.target += up * delta_y * scale;
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (1.0 - delta * 0.1)).clamp(0.1, 500.0);
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Numpad 7 — top view (looking straight down)
    pub fn set_top_view(&mut self) {
        self.yaw = 0.0;
        self.pitch = std::f32::consts::FRAC_PI_2 - 0.001;
    }

    /// Numpad 1 — front view
    pub fn set_front_view(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
    }

    /// Numpad 3 — right side view
    pub fn set_right_view(&mut self) {
        self.yaw = -std::f32::consts::FRAC_PI_2;
        self.pitch = 0.0;
    }

    pub fn fit_to_scene(&mut self, poses: &[crate::data::CameraPose]) {
        if poses.is_empty() {
            return;
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for p in poses {
            let pos = Vec3::from(p.position);
            min = min.min(pos);
            max = max.max(pos);
        }
        self.target = (min + max) * 0.5;
        let extent = (max - min).length();
        self.distance = (extent * 1.5).max(1.0);
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &OrbitCamera, aspect: f32) -> Self {
        Self {
            view_proj: camera.view_proj(aspect).to_cols_array_2d(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_matrix_no_nan() {
        let cam = OrbitCamera::new();
        let v = cam.view_matrix();
        for col in v.to_cols_array() {
            assert!(!col.is_nan());
        }
    }

    #[test]
    fn test_proj_matrix_no_nan() {
        let cam = OrbitCamera::new();
        let p = cam.proj_matrix(16.0 / 9.0);
        for col in p.to_cols_array() {
            assert!(!col.is_nan());
        }
    }
}
