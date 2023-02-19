use crate::perspective_camera::PerspectiveCamera;
use crate::physics::PhysicsScene;
use crate::renderer::SceneRenderData;
use crate::space_craft::SpaceCraft;
use crate::transform::Transform;
use crate::Renderer;
use glam::Vec3;

pub struct World {
    pub camera: PerspectiveCamera,
    pub camera_transform: Transform,

    pub physics: PhysicsScene,
    pub rendering: SceneRenderData,

    pub space_crafts: Vec<SpaceCraft>,
}

impl World {
    pub fn new(renderer: &mut Renderer) -> Self {
        let physics = PhysicsScene::new();
        let rendering = renderer.create_scene();

        Self {
            camera: PerspectiveCamera::new(95.0, 0.1),
            camera_transform: Transform::new_pos(Vec3::new(0.0, 0.0, -10.0)),
            physics,
            rendering,
            space_crafts: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for craft in self.space_crafts.iter_mut() {
            craft.update_pre_physics(&mut self.physics);
        }

        self.physics.step_physics();

        for craft in self.space_crafts.iter_mut() {
            craft.update_post_physics(&mut self.physics);

            craft.update(delta_time);

            craft.update_rendering(&mut self.rendering);
        }

        let _ = delta_time;
    }
}
