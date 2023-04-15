use glam::{IVec3, Quat, Vec3};
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ModuleModel {
    pub offset: Transform,
    pub mesh: String,
    pub material: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ColliderType {
    Mesh(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleCollider {
    pub offset: Transform,
    pub collider_type: ColliderType,
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
pub struct GridDockingPort {
    pub offset: IVec3,
    pub direction: GridDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleHardPoint {
    pub size: u16,
    pub offset: Transform,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ModuleTank {
    /// Offset of the tank from the center of the module, used for mass calculations
    pub offset: Vec3,
    /// Total capacity of the tank in meters^3
    pub capacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub categories: Vec<String>,

    /// Mass in Kg of the module
    pub base_mass: f32,

    /// Health of the module, if none, all damage goes directly to global health
    pub local_max_health: Option<f32>,
    /// When this module takes damage, how much should it take
    /// taken_damage = damage_multiplier * incoming_damage
    /// Armored modules should have damage_multiplier < 1.0
    /// Fragile modules should have damage_multiplier > 1.0
    pub damage_multiplier: f32,

    /// Axis aligned docking port that allow modules to be connected together
    pub connectors: Vec<GridDockingPort>,

    /// Mount point for attachments like turrets, shield generators, etc
    pub hard_points: Vec<ModuleHardPoint>,

    /// Tanks that can contain a liquids or gases
    pub tanks: Vec<ModuleTank>,

    pub exterior_model: Option<ModuleModel>,
    pub exterior_colliders: Vec<ModuleCollider>,

    pub interior: Option<()>,
}

pub fn load_modules_from_directory(
    directory_path: &std::path::Path,
    module_table: &mut HashMap<String, ModuleDefinition>,
) {
    if let Ok(entries) = std::fs::read_dir(directory_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "module") {
                    let contents = match std::fs::read_to_string(&path) {
                        Ok(contents) => contents,
                        Err(e) => {
                            error!("Failed to read file {:?}: {}", path, e);
                            continue;
                        }
                    };
                    let module: ModuleDefinition = match serde_json::from_str(&contents) {
                        Ok(module) => module,
                        Err(e) => {
                            error!("Failed to deserialize file {:?}: {}", path, e);
                            continue;
                        }
                    };
                    if module_table.contains_key(&module.name) {
                        error!("Duplicate module name {:?} in file {:?}", module.name, path);
                    } else {
                        module_table.insert(module.name.clone(), module);
                    }
                } else if path.is_dir() {
                    load_modules_from_directory(&path, module_table);
                }
            } else if let Err(e) = entry {
                error!("Failed to read directory entry: {}", e);
            }
        }
    } else {
        error!("Failed to read directory {:?}", directory_path);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceCraftDefinition {
    pub name: String,
    pub categories: Vec<String>,
    pub modules: HashMap<IVec3, String>,
}
