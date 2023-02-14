use crate::perspective_camera::PerspectiveCamera;
use crate::physics::PhysicsScene;
use crate::renderer::{InstanceHandle, MaterialHandle, MeshHandle, SceneRenderData};
use crate::transform::Transform;
use crate::{Renderer, Vertex};
use glam::{Quat, Vec3};
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

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

    cube_mesh: MeshHandle,
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

        let cube_mesh = create_cube_mesh(renderer);
        let cube_material = renderer.create_material().unwrap();

        let ground_entity = Entity::new(
            &mut physics,
            &mut rendering,
            cube_mesh,
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
            cube_mesh,
            cube_material,
            ground_entity,
        }
    }
}

fn create_cube_mesh(renderer: &mut Renderer) -> MeshHandle {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0, 0.0]),
        // bottom (0.0, 0.0, -1.0)
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, 0.0, 0.0]),
        // right (1.0, 0.0, 0.0)
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 1.0, 0.0]),
        // left (-1.0, 0.0, 0.0)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, 0.0, 0.0]),
        // front (0.0, 1.0, 0.0)
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 0.0]),
        // back (0.0, -1.0, 0.0)
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 1.0, 0.0]),
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, 0.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0, 0.0]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    renderer.create_mesh(&vertex_data, &index_data).unwrap()
}
