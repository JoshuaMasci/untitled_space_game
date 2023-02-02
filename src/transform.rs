use glam::{Quat, Vec3};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
