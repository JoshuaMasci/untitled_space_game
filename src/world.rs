use crate::camera::PerspectiveCamera;
use crate::physics::PhysicsScene;
use crate::renderer::{InstanceHandle, MaterialHandle, MeshHandle, SceneRenderData};
use crate::transform::Transform;
use crate::Renderer;
use glam::Vec3;
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct EntityId;
}

pub struct World {
    pub world_info: WorldInfo,
    pub entities: SlotMap<EntityId, Box<dyn Entity>>,
    pub player_entity: EntityId,
}

impl World {
    pub fn new(renderer: &mut Renderer) -> Self {
        let physics = PhysicsScene::new();
        let rendering = renderer.create_scene();

        Self {
            world_info: WorldInfo {
                physics,
                rendering,
                player_camera: PerspectiveCamera::new(95.0, 0.1),
            },
            entities: SlotMap::with_key(),
            player_entity: Default::default(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.world_info.physics.step_physics(delta_time);

        for (id, entity) in self.entities.iter_mut() {
            entity.update(&mut self.world_info, delta_time);
        }
    }

    pub fn add_entity<T: Entity + 'static>(&mut self, entity: T) -> EntityId {
        let id = self.entities.insert(Box::new(entity));
        let entity = self.entities.get_mut(id).unwrap();
        entity.set_id(id);
        entity.add_to_world(&mut self.world_info);
        id
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(mut entity) = self.entities.remove(entity_id) {
            entity.remove_from_world(&mut self.world_info);
        }

        if self.player_entity == entity_id {
            self.player_entity = EntityId::default();
        }
    }

    pub fn set_player(&mut self, player_id: EntityId) {
        self.player_entity = player_id;
    }

    pub(crate) fn update_player_input(&mut self, linear_input: Vec3, angular_input: Vec3) {
        if let Some(player) = self.entities.get_mut(self.player_entity) {
            player.update_player_input(linear_input, angular_input);
        }
    }

    pub fn get_player_camera(&self) -> (PerspectiveCamera, Transform) {
        let camera_transform: Transform = self
            .entities
            .get(self.player_entity)
            .and_then(|entity| entity.get_camera_transform())
            .unwrap_or_default();

        (self.world_info.player_camera.clone(), camera_transform)
    }
}

pub struct WorldInfo {
    pub physics: PhysicsScene,
    pub rendering: SceneRenderData,

    pub player_camera: PerspectiveCamera,
}

pub trait Entity {
    fn set_id(&mut self, id: EntityId);
    fn add_to_world(&mut self, world: &mut WorldInfo);
    fn remove_from_world(&mut self, world: &mut WorldInfo);

    fn update(&mut self, world: &mut WorldInfo, delta_time: f32);

    fn update_player_input(&mut self, linear_input: Vec3, angular_input: Vec3);
    fn get_camera_transform(&self) -> Option<Transform>;
}

pub struct DynamicEntity {
    id: EntityId,
    transform: Transform,
    model: Option<(MeshHandle, MaterialHandle)>,
    collider: Option<()>,

    model_instance: Option<InstanceHandle>,
    rigid_body_instance: Option<RigidBodyHandle>,
    collider_instance: Option<ColliderHandle>,
}

impl DynamicEntity {
    pub fn new(
        transform: Transform,
        model: Option<(MeshHandle, MaterialHandle)>,
        collider: Option<()>,
    ) -> Self {
        Self {
            id: Default::default(),
            transform,
            model,
            collider,
            model_instance: None,
            rigid_body_instance: None,
            collider_instance: None,
        }
    }
}

impl Entity for DynamicEntity {
    fn set_id(&mut self, id: EntityId) {
        self.id = id;
    }

    fn add_to_world(&mut self, world: &mut WorldInfo) {
        if let Some((mesh, material)) = &self.model {
            self.model_instance =
                world
                    .rendering
                    .create_instance(*mesh, *material, &self.transform);
        }
    }

    fn remove_from_world(&mut self, world: &mut WorldInfo) {
        if let Some(model) = self.model_instance.take() {
            world.rendering.remove_instance(model);
        }

        if let Some(collider) = self.collider_instance.take() {
            world.physics.remove_collider(collider);
        }

        if let Some(rigid_body) = self.rigid_body_instance.take() {
            world.physics.remove_rigid_body(rigid_body);
        }
    }

    fn update(&mut self, world: &mut WorldInfo, delta_time: f32) {
        if let Some(rigid_body) = self.rigid_body_instance {
            let (position, rotation) = world.physics.get_rigid_body_transform(rigid_body);
            self.transform.position = position;
            self.transform.rotation = rotation;
        }

        if let Some(model) = self.model_instance {
            world.rendering.update_instance(model, &self.transform);
        }
    }

    fn update_player_input(&mut self, linear_input: Vec3, angular_input: Vec3) {
        todo!()
    }
    fn get_camera_transform(&self) -> Option<Transform> {
        todo!()
    }
}
