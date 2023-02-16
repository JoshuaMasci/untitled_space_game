use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn new_pos(position: Vec3) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }

    pub fn as_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn as_view_matrix(&self) -> Mat4 {
        glam::Mat4::look_at_lh(
            self.position,
            (self.rotation * glam::Vec3::Z) + self.position,
            self.rotation * glam::Vec3::Y,
        )
    }
}
