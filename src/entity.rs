use crate::transform::Transform;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RigidBodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Serialize, Deserialize)]
pub enum ColliderType {
    Sphere(f32),
    Box(glam::Vec3),
    Capsule(f32, f32),
    Cylinder(f32, f32),
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    name: String,
    rigid_body_type: Option<RigidBodyType>,
}

#[derive(Serialize, Deserialize)]
pub struct EntityNode {
    local_transform: Transform,
    mesh: Option<String>,
    material: Option<String>,
    collider: Option<Collider>,
}

#[derive(Serialize, Deserialize)]
pub struct Collider {
    mass: f32,
    collider: ColliderType,
}
