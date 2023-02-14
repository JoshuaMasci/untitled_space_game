#[derive(Clone, Debug)]
pub struct PerspectiveCamera {
    x_fov_deg: f32,
    z_near: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            x_fov_deg: 75.0,
            z_near: 0.1,
        }
    }
}

impl PerspectiveCamera {
    pub fn new(x_fov_deg: f32, z_near: f32) -> Self {
        Self { x_fov_deg, z_near }
    }

    pub fn get_fov_y_rad(&self, aspect_ratio: f32) -> f32 {
        f32::atan(f32::tan(self.x_fov_deg.to_radians() / 2.0) / aspect_ratio) * 2.0
    }

    pub fn as_infinite_perspective_matrix(&self, size: [u32; 2]) -> glam::Mat4 {
        let aspect_ratio = size[0] as f32 / size[1] as f32;
        glam::Mat4::perspective_infinite_lh(
            self.get_fov_y_rad(aspect_ratio),
            aspect_ratio,
            self.z_near,
        )
    }

    pub fn as_infinite_reverse_perspective_matrix(&self, size: [u32; 2]) -> glam::Mat4 {
        let aspect_ratio = size[0] as f32 / size[1] as f32;
        glam::Mat4::perspective_infinite_reverse_lh(
            self.get_fov_y_rad(aspect_ratio),
            aspect_ratio,
            self.z_near,
        )
    }
}
