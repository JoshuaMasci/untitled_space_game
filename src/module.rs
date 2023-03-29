use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    mesh: String,
    material: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColliderType {
    Sphere(f32),
    Box([f32; 3]),
    Capsule(f32, f32),
    Cylinder(f32, f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    collider: ColliderType,
    offset: Option<(glam::Vec3, glam::Quat)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Module {
    model: Option<Model>,
    collider: Option<Collider>,
}
