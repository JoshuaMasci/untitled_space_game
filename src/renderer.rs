use bytemuck::{Pod, Zeroable};

use std::borrow::Cow;
use std::ops::Range;
use wgpu::util::DeviceExt;
use wgpu::{
    BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
    CompositeAlphaMode, DepthStencilState, Device, IndexFormat, PresentMode,
    ShaderModuleDescriptor, ShaderSource, SurfaceConfiguration, TextureFormat, TextureUsages,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub struct Renderer {
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    cube_mesh: Mesh,
}

impl Renderer {
    pub fn new(device: &Device) -> Self {
        let color_shader = include_str!("shader/color.wgsl");
        let color_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::from(color_shader)),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[[
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &color_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &color_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: None,
                    write_mask: ColorWrites::COLOR,
                })],
            }),
            multiview: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        let cube_mesh = create_cube_mesh(device);

        Self {
            uniform_buffer,
            pipeline,
            bind_group,
            cube_mesh,
        }
    }

    pub fn update_uniforms(&self, device: &wgpu::Device, queue: &wgpu::Queue, mvp: &[f32]) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(mvp));
    }

    pub fn render(
        &mut self,
        size: [u32; 2],
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.4296875,
                            b: 0.19921875,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            self.cube_mesh.draw(&mut render_pass, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: usize,
}

impl Mesh {
    fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len(),
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, instances: Range<u32>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count as u32, 0, instances);
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
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

fn create_cube_mesh(device: &Device) -> Mesh {
    let (vertex_data, index_data) = create_vertices();
    Mesh::new(&device, &vertex_data, &index_data)
}
