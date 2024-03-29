use glam::{Quat, Vec3};
use rapier3d::prelude::*;

pub enum ColliderShape {
    Sphere(f32),
    Box(glam::Vec3),
    Capsule(f32, f32),
    Cylinder(f32, f32),
    Mesh,
}

impl ColliderShape {
    fn create_shared_shape(&self) -> SharedShape {
        match self {
            Self::Sphere(radius) => SharedShape::ball(*radius),
            Self::Box(half_extent) => {
                SharedShape::cuboid(half_extent.x, half_extent.y, half_extent.z)
            }
            Self::Capsule(radius, y) => SharedShape::capsule_y(*y, *radius),
            Self::Cylinder(radius, y) => SharedShape::cylinder(*y, *radius),
            _ => unimplemented!(),
        }
    }
}

pub struct PhysicsScene {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsScene {
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let gravity = vector![0.0, 0.0, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        Self {
            rigid_body_set,
            collider_set,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
        }
    }

    pub fn step_physics(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;

        let physics_hooks = ();
        let event_handler = ();

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
    }

    pub fn create_rigid_body(
        &mut self,
        translation: Vec3,
        rotation: Quat,
        body_type: RigidBodyType,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::new(body_type)
            .translation(translation.into())
            .rotation(nalgebra::UnitQuaternion::from(rotation).scaled_axis())
            .build();
        self.rigid_body_set.insert(rigid_body)
    }

    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }

    pub fn get_rigid_body_transform(&self, handle: RigidBodyHandle) -> (Vec3, Quat) {
        let rigid_body = self.rigid_body_set.get(handle).unwrap();
        (
            (*rigid_body.translation()).into(),
            (*rigid_body.rotation()).into(),
        )
    }

    pub fn set_rigid_body_transform(
        &mut self,
        handle: RigidBodyHandle,
        translation: Vec3,
        rotation: Quat,
        wake_up: bool,
    ) {
        if let Some(rigid_body) = self.rigid_body_set.get_mut(handle) {
            rigid_body.set_translation(translation.into(), wake_up);
            rigid_body.set_rotation(rotation.into(), wake_up);
        }
    }

    pub fn create_collider(
        &mut self,
        parent_handle: RigidBodyHandle,
        translation: Vec3,
        rotation: Quat,
        shape: &ColliderShape,
        mass: f32,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::new(shape.create_shared_shape())
            .mass(mass)
            .translation(translation.into())
            .rotation(nalgebra::UnitQuaternion::from(rotation).scaled_axis())
            .build();

        self.collider_set
            .insert_with_parent(collider, parent_handle, &mut self.rigid_body_set)
    }

    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true,
        );
    }

    pub fn get_collider_transform(&self, handle: ColliderHandle) -> (Vec3, Quat) {
        let collider = self.collider_set.get(handle).unwrap();
        (
            (*collider.translation()).into(),
            (*collider.rotation()).into(),
        )
    }

    pub fn set_collider_transform(
        &mut self,
        handle: ColliderHandle,
        translation: Vec3,
        rotation: Quat,
    ) {
        if let Some(collider) = self.collider_set.get_mut(handle) {
            collider.set_translation(translation.into());
            collider.set_rotation(rotation.into());
        }
    }

    pub fn set_rigid_body_angular_velocity(
        &mut self,
        handle: RigidBodyHandle,
        angular_velocity: Vec3,
    ) {
        let rigid_body = self.rigid_body_set.get_mut(handle).unwrap();
        rigid_body.set_angvel(angular_velocity.into(), true);
    }
}
