use crate::physics::ColliderShape;
use crate::player::Player;
use crate::renderer::PbrMaterialDefinition;
use crate::transform::Transform;
use crate::world::{DynamicEntity, World};
use crate::Renderer;
use glam::Vec3;
use log::{error, info, warn};
use nalgebra::Point;
use rapier3d::prelude::SharedShape;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

pub struct App {
    pub input: WinitInputHelper,
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

        let info: wgpu::AdapterInfo = adapter.get_info();

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

        let mut renderer = Renderer::new(device.clone(), queue);

        let mut world = World::new(&mut renderer);

        let camera_id = world.add_entity(Player::new(Transform::default()));
        world.set_player(camera_id);

        world.add_entity(DynamicEntity::new(
            Transform::new_pos(Vec3::new(0.0, 0.0, 15.0)),
            Some((
                renderer.load_mesh("resource/mesh/Cube.obj").unwrap(),
                renderer
                    .create_material(PbrMaterialDefinition {
                        color: [0.1, 0.75, 0.55, 1.0],
                        metallic: 0.5,
                        roughness: 0.5,
                    })
                    .unwrap(),
            )),
            Some(ColliderShape::Box(glam::Vec3::splat(0.5))),
        ));

        let mut module_table = HashMap::new();
        crate::space_craft::load_modules_from_directory(
            Path::new("resource/module/"),
            &mut module_table,
        );

        Self {
            input: WinitInputHelper::new(),
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

    pub fn update(&mut self, delta_time: f32) {
        let linear_input = Vec3::new(
            keys_to_axis(&self.input, VirtualKeyCode::D, VirtualKeyCode::A),
            keys_to_axis(&self.input, VirtualKeyCode::Space, VirtualKeyCode::LShift),
            keys_to_axis(&self.input, VirtualKeyCode::W, VirtualKeyCode::S),
        );

        let angular_input = Vec3::new(
            keys_to_axis(&self.input, VirtualKeyCode::Right, VirtualKeyCode::Left),
            keys_to_axis(&self.input, VirtualKeyCode::Up, VirtualKeyCode::Down),
            keys_to_axis(&self.input, VirtualKeyCode::E, VirtualKeyCode::Q),
        );

        self.world.update_player_input(linear_input, angular_input);
        self.world.update(delta_time);
    }

    pub fn render(&mut self) {
        let output_texture = self.surface.get_current_texture().unwrap();

        let output_view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (camera, camera_transform) = self.world.get_player_camera();

        let light_dir = glam::Vec3::new(0.5, -2.0, 1.0).normalize();

        let scene_data = crate::renderer::SceneData {
            view_projection_matrix: *(camera
                .as_infinite_reverse_perspective_matrix(self.surface_size)
                * camera_transform.as_view_matrix())
            .as_ref(),
            ambient_light_color: [0.1; 4],
            sun_light_direction_intensity: [light_dir.x, light_dir.y, light_dir.z, 0.5],
            sun_light_color: [1.0; 4],
        };

        self.renderer.render_scene(
            self.surface_size,
            &output_view,
            &scene_data,
            &self.world.world_info.rendering,
        );

        output_texture.present();
    }
}

fn load_convex_hull_from_obj<P: AsRef<std::path::Path> + Debug>(path: P) -> Option<SharedShape> {
    const LOAD_OPTIONS: tobj::LoadOptions = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ignore_points: false,
        ignore_lines: true,
    };

    let (models, _materials) = match tobj::load_obj(path, &LOAD_OPTIONS) {
        Ok(values) => values,
        Err(e) => {
            error!("Failed to load obj file: {}", e);
            return None;
        }
    };
    let model = &models[0];
    let mesh = &model.mesh;

    let mut points = Vec::new();

    for i in 0..(mesh.positions.len() / 3) {
        let i3 = i * 3;
        points.push(Point::from_slice(&mesh.positions[i3..(i3 + 3)]));
    }

    SharedShape::convex_hull(&points)
}

fn keys_to_axis(
    input: &WinitInputHelper,
    positive_key: VirtualKeyCode,
    negative_key: VirtualKeyCode,
) -> f32 {
    if input.key_held(positive_key) && !input.key_held(negative_key) {
        1.0
    } else if !input.key_held(positive_key) && input.key_held(negative_key) {
        -1.0
    } else {
        0.0
    }
}
