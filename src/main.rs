use crate::renderer::{Renderer, Vertex};
use crate::transform::Transform;
use glam::{Mat4, Vec3};
use std::sync::Arc;

mod module;
mod physics_scene;
mod renderer;
mod space_craft;
mod transform;

fn main() {
    pretty_env_logger::init_timed();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("untitled_space_game")
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    window.set_maximized(true);

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
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        },
    );

    let mut renderer = Renderer::new(device.clone(), queue.clone());

    let cube_mesh = {
        let (vertices, indices) = create_cube_mesh();
        renderer.create_mesh(&vertices, &indices).unwrap()
    };

    let cube_material = renderer.create_material().unwrap();

    let mut scene_data = renderer.create_scene();

    scene_data.create_instance(cube_mesh, cube_material, &Transform::default());

    let mut window_size = [window_size.width, window_size.height];

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                window_id,
            } if window_id == window.id() => {
                window_size = [new_size.width, new_size.height];

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: surface.get_capabilities(&adapter).formats[0],
                        width: window_size[0],
                        height: window_size[1],
                        present_mode: wgpu::PresentMode::Mailbox,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: Vec::new(),
                    },
                );
            }
            winit::event::Event::MainEventsCleared => {
                let output_texture = surface.get_current_texture().unwrap();

                let output_view = output_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                renderer.render_scene(window_size, &output_view, &[0.0; 16], &scene_data);

                output_texture.present();
            }
            _ => (),
        }
    });
}

// fn get_mvp_matrix(fov_x: f32, window_size: [f32; 2]) -> glam::Mat4 {
//     let aspect_ratio = window_size[0] / window_size[1];
//     let fov_y = 2.0 * f32::atan(f32::tan(fov_x.to_radians() / 2.0) / aspect_ratio);
//
//     let model = glam::Mat4::IDENTITY;
//
//     let view = Mat4::look_to_lh(Vec3::new(0.0, 0.0, -5.0), Vec3::Z, Vec3::Y);
//     let projection = Mat4::perspective_infinite_reverse_lh(fov_y, aspect_ratio, 0.1);
//
//     projection * view * model
// }

fn create_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
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

    (vertex_data.to_vec(), index_data.to_vec())
}
