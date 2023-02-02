use crate::module::Module;
use crate::transform::Transform;
use std::collections::HashMap;

pub struct SpaceCraft {
    //TODO: RIGID BODIES + COLLIDERS
    //TODO: MESH HANDLES
    transform: Transform,
    modules: HashMap<glam::IVec3, Module>,
}
