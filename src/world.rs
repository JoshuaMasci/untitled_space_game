use crate::perspective_camera::PerspectiveCamera;
use crate::physics::PhysicsScene;
use crate::renderer::{InstanceHandle, MaterialHandle, MeshHandle, SceneRenderData};
use crate::transform::Transform;
use crate::{Renderer, Vertex};
use glam::{Quat, Vec3};
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use std::path::Path;

struct Entity {
    transform: Transform,
    rigid_body: RigidBodyHandle,
    collider: ColliderHandle,
    instance: InstanceHandle,
}

pub struct World {
    pub camera: PerspectiveCamera,
    pub camera_transform: Transform,

    pub physics: PhysicsScene,
    pub rendering: SceneRenderData,

    cube_material: MaterialHandle,

    ground_entity: Entity,
}

impl Entity {
    pub fn new(
        physics: &mut PhysicsScene,
        rendering: &mut SceneRenderData,
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: Transform,
    ) -> Self {
        let rigid_body = physics.create_rigid_body(transform.position, transform.rotation);
        let collider = physics.create_collider(rigid_body, Vec3::ZERO, Quat::IDENTITY, 1.0);
        let instance = rendering
            .create_instance(mesh, material, &transform)
            .unwrap();

        Self {
            transform,
            rigid_body,
            collider,
            instance,
        }
    }
}

impl World {
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut physics = PhysicsScene::new();
        let mut rendering = renderer.create_scene();

        let obj_mesh = renderer.load_mesh("resource/mesh/Sphere.obj").unwrap();
        let cube_material = renderer.create_material().unwrap();

        let ground_entity = Entity::new(
            &mut physics,
            &mut rendering,
            obj_mesh,
            cube_material,
            Transform {
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        );

        Self {
            camera: PerspectiveCamera::new(95.0, 0.1),
            camera_transform: Transform::new_pos(Vec3::new(0.0, 0.0, -10.0)),
            physics,
            rendering,
            cube_material,
            ground_entity,
        }
    }
}
