use bytemuck::{Pod, Zeroable};

use crate::transform::Transform;
use slotmap::SlotMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayout, BufferAddress, BufferBindingType, ColorTargetState, ColorWrites,
    CompareFunction, CompositeAlphaMode, DepthStencilState, Device, IndexFormat, PresentMode,
    ShaderModuleDescriptor, ShaderSource, SurfaceConfiguration, TextureFormat, TextureUsages,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
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
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    instance_set_bind_group_layout: Arc<BindGroupLayout>,

    scene_data: (wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout),

    pipeline: wgpu::RenderPipeline,

    meshes: SlotMap<MeshKey, Mesh>,
    materials: SlotMap<MaterialKey, wgpu::RenderPipeline>,
}

impl Renderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let color_shader = include_str!("shader/color.wgsl");
        let color_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::from(color_shader)),
        });

        let scene_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[[
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let scene_data_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let instance_set_bind_group_layout = Arc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                }],
            },
        ));

        let scene_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &scene_data_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    scene_data_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&scene_data_layout, instance_set_bind_group_layout.as_ref()],
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

        Self {
            device,
            queue,
            instance_set_bind_group_layout,
            scene_data: (scene_data_buffer, scene_data_bind_group, scene_data_layout),
            pipeline,
            meshes: SlotMap::with_key(),
            materials: SlotMap::with_key(),
        }
    }

    pub fn create_scene(&self) -> SceneRenderData {
        SceneRenderData::new(
            self.device.clone(),
            self.queue.clone(),
            self.instance_set_bind_group_layout.clone(),
        )
    }

    pub fn create_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Option<MeshKey> {
        Some(
            self.meshes
                .insert(Mesh::new(&self.device, vertices, indices)),
        )
    }

    pub fn create_material(&mut self) -> Option<MaterialKey> {
        Some(MaterialKey::default())
    }

    pub fn render_scene(
        &mut self,
        size: [u32; 2],
        view: &wgpu::TextureView,
        view_projection_matrix: &[f32; 16],
        scene_data: &SceneRenderData,
    ) {
        self.queue.write_buffer(
            &self.scene_data.0,
            0,
            bytemuck::cast_slice(view_projection_matrix),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
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

            render_pass.set_pipeline(&self.pipeline); //TODO: don't use hardcoded pipeline
            render_pass.set_bind_group(0, &self.scene_data.1, &[]);

            for (key, set) in scene_data.instance_set_map.iter() {
                render_pass.set_bind_group(1, &set.bind_group, &[]);
                self.meshes
                    .get(key.mesh)
                    .unwrap()
                    .draw(&mut render_pass, 0..(set.len() as u32));
            }
        }

        self.queue.submit(Some(encoder.finish()));
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

slotmap::new_key_type! {
    pub struct InstanceKey;
    pub struct MeshKey;
    pub struct MaterialKey;
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct InstanceType {
    mesh: MeshKey,
    material: MaterialKey,
}

pub struct SceneRenderData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance_set_bind_group_layout: Arc<BindGroupLayout>,

    instance_map: SlotMap<InstanceKey, InstanceType>,
    instance_set_map: HashMap<InstanceType, InstanceSet<[f32; 16]>>,
}

impl SceneRenderData {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        instance_set_bind_group_layout: Arc<BindGroupLayout>,
    ) -> Self {
        Self {
            device,
            queue,
            instance_set_bind_group_layout,
            instance_map: SlotMap::with_key(),
            instance_set_map: HashMap::new(),
        }
    }

    pub fn create_instance(
        &mut self,
        mesh: MeshKey,
        material: MaterialKey,
        transform: &Transform,
    ) -> Option<InstanceKey> {
        let instance_type = InstanceType { mesh, material };

        let instance_key = self.instance_map.insert(instance_type.clone());

        let set = self
            .instance_set_map
            .entry(instance_type)
            .or_insert_with(|| {
                InstanceSet::new(
                    self.device.clone(),
                    self.queue.clone(),
                    self.instance_set_bind_group_layout.as_ref(),
                    1024,
                )
            });
        set.add(instance_key, transform.as_model_matrix().as_ref());
        Some(instance_key)
    }

    pub fn update_instance(&mut self, key: InstanceKey, transform: &Transform) {
        let instance_type = self.instance_map.get(key).unwrap().clone();
        let set = self.instance_set_map.get_mut(&instance_type).unwrap();
        set.update(key, transform.as_model_matrix().as_ref());
    }

    pub fn remove_instance(&mut self, key: InstanceKey) {
        let instance_type = self.instance_map.get(key).unwrap().clone();
        let set = self.instance_set_map.get_mut(&instance_type).unwrap();
        set.remove(key);
    }
}

pub struct InstanceSet<T: bytemuck::Pod + Clone> {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    count: usize,
    capacity: usize,
    instance_map: HashMap<InstanceKey, (usize, T)>,
}

impl<T: bytemuck::Pod> InstanceSet<T> {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        bind_group_layout: &wgpu::BindGroupLayout,
        capacity: usize,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("InstanceSet Buffer"),
            size: (capacity * std::mem::size_of::<T>()) as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("InstanceSet BindGroup"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        Self {
            device,
            queue,
            buffer,
            bind_group,
            count: 0,
            capacity,
            instance_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: InstanceKey, data: &T) {
        let next_index = self.count;
        self.count += 1;

        if self.count > self.capacity {
            todo!("Resize Instance-Set Buffer");
        }

        let new_entry = (next_index, *data);
        self.write_index(new_entry.0, &new_entry.1);
        self.instance_map.insert(key, new_entry);
    }
    pub fn update(&mut self, key: InstanceKey, data: &T) {
        let index = {
            let instance_entry = self.instance_map.get_mut(&key).unwrap();
            instance_entry.1 = *data;
            instance_entry.0
        };

        self.write_index(index, data);
    }
    pub fn remove(&mut self, key: InstanceKey) {
        let removed_entry = self.instance_map.remove(&key).unwrap();

        let last_index = self.count;

        //If the last entry still exists, move it too the removed slot
        if let Some(entry) = self
            .instance_map
            .iter_mut()
            .find(|entry| entry.1 .0 == last_index)
            .map(|(_id, last_entry)| {
                last_entry.0 = removed_entry.0;
                *last_entry
            })
        {
            self.write_index(entry.0, &entry.1);
        }

        self.count -= 1;
    }

    fn write_index(&mut self, index: usize, data: &T) {
        self.queue.write_buffer(
            &self.buffer,
            (index * std::mem::size_of::<T>()) as BufferAddress,
            bytemuck::cast_slice(&[*data]),
        )
    }

    fn len(&self) -> usize {
        self.count
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}
