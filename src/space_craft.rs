use glam::{IVec3, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Model {
    position: Vec3,
    orientation: Quat,
    mesh: String,
    material: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ColliderType {
    Mesh(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collider {
    position: Vec3,
    orientation: Quat,
    collider_type: ColliderType,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Tank {
    /// Offset of the tank from the center of the module, used for mass calculations
    offset: Vec3,
    /// Total capacity of the tank in meters^3
    capacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GridDirection {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridConnector {
    offset: IVec3,
    direction: GridDirection,
}

pub struct ModuleDefinition {
    grid_size: (),
    name: String,
    categories: Vec<String>,
    base_mass: f32,

    /// Health of the module, if none, all damage goes directly to global health
    local_max_health: Option<f32>,
    /// When this module takes damage, how much should it take
    /// taken_damage = damage_multiplier * incoming_damage;
    /// Armored modules should have damage_multiplier < 1.0
    /// Fragile modules should have damage_multiplier > 1.0
    damage_multiplier: f32,

    tanks: Vec<Tank>,

    connectors: Vec<GridConnector>,

    exterior_model: Option<Model>,
    exterior_colliders: Vec<Collider>,

    interior: Option<()>,
}

pub struct SpaceCraftDefinition {
    name: String,
    categories: Vec<String>,

    modules: HashMap<IVec3, String>,
}
