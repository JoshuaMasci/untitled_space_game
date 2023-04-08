use bytemuck::{Pod, Zeroable};

use crate::transform::Transform;

use log::error;
use slotmap::SlotMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneData {
    pub(crate) view_projection_matrix: [f32; 16],
    pub(crate) ambient_light_color: [f32; 4],
    pub(crate) sun_light_direction_intensity: [f32; 4],
    pub(crate) sun_light_color: [f32; 4],
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress * 2,
                    shader_location: 2,
                },
            ],
        }
    }
}

pub struct PbrMaterialDefinition {
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

fn create_pbr_material_static_mesh_pipeline(
    device: &Arc<wgpu::Device>,
    pipeline_layout: &wgpu::PipelineLayout,
    depth_stencil_format: Option<wgpu::TextureFormat>,
) -> wgpu::RenderPipeline {
    let code = include_str!("shader/pbr_material_static_mesh.wgsl");
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::from(code)),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        primitive: Default::default(),
        depth_stencil: depth_stencil_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::COLOR,
            })],
        }),
        multiview: None,
    })
}

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    scene_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    instance_set_bind_group_layout: Arc<wgpu::BindGroupLayout>,
    material_bind_group_layout: Arc<wgpu::BindGroupLayout>,

    pbr_material_pipeline_layout: wgpu::PipelineLayout,
    pbr_material_static_mesh_pipeline: wgpu::RenderPipeline,

    scene_data: (wgpu::Buffer, wgpu::BindGroup),

    meshes: SlotMap<MeshHandle, Mesh>,
    materials: SlotMap<MaterialHandle, Material>,
}

impl Renderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let scene_bind_group_layout = Arc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                }],
            },
        ));

        let instance_set_bind_group_layout = Arc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                }],
            },
        ));

        let material_bind_group_layout = Arc::new(device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                }],
            },
        ));

        let pbr_material_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &scene_bind_group_layout,
                    &instance_set_bind_group_layout,
                    &material_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let pbr_material_static_mesh_pipeline = create_pbr_material_static_mesh_pipeline(
            &device,
            &pbr_material_pipeline_layout,
            Some(wgpu::TextureFormat::Depth24Plus),
        );

        let scene_data = {
            let scene_data = SceneData {
                view_projection_matrix: [0.0; 16],
                ambient_light_color: [0.0; 4],
                sun_light_direction_intensity: [0.0; 4],
                sun_light_color: [0.0; 4],
            };

            let scene_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[scene_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let scene_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &scene_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        scene_data_buffer.as_entire_buffer_binding(),
                    ),
                }],
            });

            (scene_data_buffer, scene_data_bind_group)
        };

        Self {
            device,
            queue,
            scene_bind_group_layout,
            instance_set_bind_group_layout,
            material_bind_group_layout,
            pbr_material_pipeline_layout,
            pbr_material_static_mesh_pipeline,
            scene_data,
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

    pub fn create_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) -> Option<MeshHandle> {
        Some(
            self.meshes
                .insert(Mesh::new(&self.device, vertices, indices)),
        )
    }

    pub fn create_material(&mut self, material: PbrMaterialDefinition) -> Option<MaterialHandle> {
        let material_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        material.color[0],
                        material.color[1],
                        material.color[2],
                        material.color[3],
                        material.metallic,
                        material.roughness,
                        0.0,
                        0.0,
                    ]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let material_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.material_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    material_uniform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        Some(self.materials.insert(Material {
            material_uniform_buffer,
            material_bind_group,
        }))
    }

    pub fn load_mesh<P: AsRef<std::path::Path> + Debug>(&mut self, path: P) -> Option<MeshHandle> {
        const LOAD_OPTIONS: tobj::LoadOptions = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };

        let (models, _materials) = match tobj::load_obj(path, &LOAD_OPTIONS) {
            Ok(values) => values,
            Err(e) => {
                error!("Failed to load obj file: {}", e);
                return None;
            }
        };

        //TODO: support more then one model
        let model = &models[0];
        let mesh = &model.mesh;

        let mut vertices = Vec::with_capacity(model.mesh.positions.len());

        for i in 0..(mesh.positions.len() / 3) {
            let i2 = i * 2;
            let i3 = i * 3;

            vertices.push(Vertex::new(
                [
                    mesh.positions[i3],
                    mesh.positions[i3 + 1],
                    mesh.positions[i3 + 2],
                ],
                [mesh.normals[i3], mesh.normals[i3 + 1], mesh.normals[i3 + 2]],
                [mesh.texcoords[i2], mesh.texcoords[i2 + 1]],
            ))
        }

        self.create_mesh(&vertices, &model.mesh.indices)
    }

    pub fn render_scene(
        &mut self,
        size: [u32; 2],
        render_target: &wgpu::TextureView,
        scene_data: &SceneData,
        scene_render_data: &SceneRenderData,
    ) {
        self.queue
            .write_buffer(&self.scene_data.0, 0, bytemuck::cast_slice(&[*scene_data]));

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
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
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

            render_pass.set_pipeline(&self.pbr_material_static_mesh_pipeline);
            render_pass.set_bind_group(0, &self.scene_data.1, &[]);

            for (key, set) in scene_render_data.instance_set_map.iter() {
                if !set.is_empty() {
                    render_pass.set_bind_group(1, &set.bind_group, &[]);

                    render_pass.set_bind_group(
                        2,
                        &self
                            .materials
                            .get(key.material)
                            .unwrap()
                            .material_bind_group,
                        &[],
                    );

                    self.meshes
                        .get(key.mesh)
                        .unwrap()
                        .draw(&mut render_pass, 0..(set.len() as u32));
                }
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
    fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
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
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count as u32, 0, instances);
    }
}

struct Material {
    material_uniform_buffer: wgpu::Buffer,
    material_bind_group: wgpu::BindGroup,
}

slotmap::new_key_type! {
    pub struct InstanceHandle;
    pub struct MeshHandle;
    pub struct MaterialHandle;
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct InstanceType {
    mesh: MeshHandle,
    material: MaterialHandle,
}

pub struct SceneRenderData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance_set_bind_group_layout: Arc<wgpu::BindGroupLayout>,

    instance_map: SlotMap<InstanceHandle, InstanceType>,
    instance_set_map: HashMap<InstanceType, InstanceSet<[f32; 16]>>,
}

impl SceneRenderData {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        instance_set_bind_group_layout: Arc<wgpu::BindGroupLayout>,
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
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: &Transform,
    ) -> Option<InstanceHandle> {
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

    pub fn update_instance(&mut self, key: InstanceHandle, transform: &Transform) {
        let instance_type = self.instance_map.get(key).unwrap().clone();
        let set = self.instance_set_map.get_mut(&instance_type).unwrap();
        set.update(key, transform.as_model_matrix().as_ref());
    }

    pub fn remove_instance(&mut self, key: InstanceHandle) {
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
    instance_map: HashMap<InstanceHandle, (usize, T)>,
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
            size: (capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress,
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

    pub fn add(&mut self, key: InstanceHandle, data: &T) {
        let next_index = self.count;
        self.count += 1;

        if self.count > self.capacity {
            todo!("Resize Instance-Set Buffer");
        }

        let new_entry = (next_index, *data);
        self.write_index(new_entry.0, &new_entry.1);
        self.instance_map.insert(key, new_entry);
    }
    pub fn update(&mut self, key: InstanceHandle, data: &T) {
        let index = {
            let instance_entry = self.instance_map.get_mut(&key).unwrap();
            instance_entry.1 = *data;
            instance_entry.0
        };

        self.write_index(index, data);
    }
    pub fn remove(&mut self, key: InstanceHandle) {
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
            (index * std::mem::size_of::<T>()) as wgpu::BufferAddress,
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
