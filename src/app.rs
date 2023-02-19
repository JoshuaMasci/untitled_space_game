use crate::physics::ColliderType;
use crate::space_craft::{SpaceCraft, SpaceCraftModule};
use crate::transform::Transform;
use crate::world::World;
use crate::Renderer;
use glam::Vec3;
use log::info;
use rapier3d::dynamics::RigidBodyType;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode::T;
use winit::window::Window;

pub struct App {
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,

    surface_size: [u32; 2],
    surface_config: wgpu::SurfaceConfiguration,

    renderer: Renderer,

    world: World,
}

impl App {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let window_size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        };
        surface.configure(&device, &surface_config);

        let mut renderer = Renderer::new(device.clone(), queue.clone());

        let mut world = World::new(&mut renderer);

        {
            let space_craft_transform = Transform::default();

            let rigid_body = world.physics.create_rigid_body(
                space_craft_transform.position,
                space_craft_transform.rotation,
                RigidBodyType::Dynamic,
            );

            let sphere_module = {
                let module_transform = Transform::new_pos(Vec3::new(0.0, -2.0, 0.0));
                let sphere_mesh = renderer.load_mesh("resource/mesh/Sphere.obj").unwrap();
                let sphere_material = renderer.create_material().unwrap();

                SpaceCraftModule {
                    local_transform: module_transform.clone(),
                    model_instance: world.rendering.create_instance(
                        sphere_mesh,
                        sphere_material,
                        &module_transform,
                    ),
                    collider_instance: Some(world.physics.create_collider(
                        rigid_body,
                        module_transform.position,
                        module_transform.rotation,
                        ColliderType::Sphere(0.5),
                        1.0,
                    )),
                }
            };

            let cube_module = {
                let module_transform = Transform::new_pos(Vec3::new(0.0, 2.0, 0.0));
                let cube_mesh = renderer.load_mesh("resource/mesh/Cube.obj").unwrap();
                let cube_material = renderer.create_material().unwrap();

                SpaceCraftModule {
                    local_transform: module_transform.clone(),
                    model_instance: world.rendering.create_instance(
                        cube_mesh,
                        cube_material,
                        &module_transform,
                    ),
                    collider_instance: Some(world.physics.create_collider(
                        rigid_body,
                        module_transform.position,
                        module_transform.rotation,
                        ColliderType::Box(Vec3::splat(0.5)),
                        1.0,
                    )),
                }
            };

            let space_craft = SpaceCraft {
                transform: space_craft_transform,
                rigid_body,
                modules: vec![sphere_module, cube_module],
            };

            world
                .physics
                .set_rigid_body_angular_velocity(space_craft.rigid_body, Vec3::new(0.0, 0.0, 2.0));

            world.space_crafts.push(space_craft);
        }

        Self {
            surface,
            device,
            surface_size: [window_size.width, window_size.height],
            surface_config,
            renderer,
            world,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_size = [new_size.width, new_size.height];
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn keyboard_event(
        &mut self,
        scancode: &winit::event::ScanCode,
        state: &winit::event::ElementState,
    ) {
        info!("Keyboard Event: {:?}:{:?}", scancode, state);
    }

    pub fn update(&mut self, delta_time: f32) {
        let _ = delta_time;

        self.world.update(delta_time);
    }

    pub fn render(&mut self) {
        let output_texture = self.surface.get_current_texture().unwrap();

        let output_view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render_scene(
            self.surface_size,
            &output_view,
            (self
                .world
                .camera
                .as_infinite_reverse_perspective_matrix(self.surface_size)
                * self.world.camera_transform.as_view_matrix())
            .as_ref(),
            &self.world.rendering,
        );

        output_texture.present();
    }
}
