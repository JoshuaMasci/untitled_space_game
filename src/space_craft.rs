use crate::physics::PhysicsScene;
use crate::renderer::{InstanceHandle, SceneRenderData};
use crate::transform::Transform;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

pub struct SpaceCraftModule {
    pub local_transform: Transform,
    pub model_instance: Option<InstanceHandle>,
    pub collider_instance: Option<ColliderHandle>,
}

pub struct SpaceCraft {
    pub transform: Transform,
    pub rigid_body: RigidBodyHandle,
    pub modules: Vec<SpaceCraftModule>,
}

impl SpaceCraft {
    pub fn update_pre_physics(&mut self, physics_scene: &mut PhysicsScene) {
        physics_scene.set_rigid_body_transform(
            self.rigid_body,
            self.transform.position,
            self.transform.rotation,
            false,
        )
    }

    pub fn update_post_physics(&mut self, physics_scene: &mut PhysicsScene) {
        let new_transform = physics_scene.get_rigid_body_transform(self.rigid_body);
        self.transform.position = new_transform.0;
        self.transform.rotation = new_transform.1;

        for module in self.modules.iter() {
            if let Some(instance) = module.collider_instance {
                physics_scene.set_collider_transform(
                    instance,
                    self.transform.position,
                    self.transform.rotation,
                );
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let _ = delta_time;
    }

    pub fn update_rendering(&mut self, scene_data: &mut SceneRenderData) {
        for module in self.modules.iter() {
            if let Some(instance) = module.model_instance {
                scene_data.update_instance(
                    instance,
                    &self.transform.transform_by(&module.local_transform),
                );
            }
        }
    }
}
