use crate::transform::Transform;
use crate::world::{Entity, EntityId, WorldInfo};
use glam::{Quat, Vec3};

pub struct Player {
    id: EntityId,
    transform: Transform,
    linear_input: Vec3,
    angular_input: Vec3,
}

impl Player {
    pub fn new(transform: Transform) -> Self {
        Self {
            id: Default::default(),
            transform,
            linear_input: Vec3::ZERO,
            angular_input: Vec3::ZERO,
        }
    }
}

impl Entity for Player {
    fn set_id(&mut self, id: EntityId) {
        self.id = id;
    }

    fn add_to_world(&mut self, world: &mut WorldInfo) {}

    fn remove_from_world(&mut self, world: &mut WorldInfo) {}

    fn update(&mut self, world: &mut WorldInfo, delta_time: f32) {
        const CAMERA_MOVE_SPEED: f32 = 5.0;

        let input_vector = self.linear_input.normalize_or_zero();
        self.transform.position +=
            self.transform.rotation * (input_vector * CAMERA_MOVE_SPEED * delta_time);

        const CAMERA_ROTATION_SPEED: f32 = 1.0;
        self.transform.rotation *=
            Quat::from_rotation_y(self.angular_input.x * CAMERA_ROTATION_SPEED * delta_time);
        self.transform.rotation *=
            Quat::from_rotation_x(self.angular_input.y * CAMERA_ROTATION_SPEED * delta_time);
        self.transform.rotation *=
            Quat::from_rotation_z(-self.angular_input.z * CAMERA_ROTATION_SPEED * delta_time);

        self.transform.rotation = self.transform.rotation.normalize();
    }

    fn update_player_input(&mut self, linear_input: Vec3, angular_input: Vec3) {
        self.linear_input = linear_input;
        self.angular_input = angular_input;
    }
    fn get_camera_transform(&self) -> Option<Transform> {
        Some(self.transform.clone())
    }
}
