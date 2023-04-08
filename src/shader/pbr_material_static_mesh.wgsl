struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal_ws: vec3<f32>,
    @location(1) uv: vec2<f32>,

};

struct SceneData {
    view_projection_matrix: mat4x4<f32>,
    ambient_light_color: vec4<f32>,
    sun_light_direction_intensity: vec4<f32>,
    sun_light_color: vec4<f32>,
}

struct PbrMaterialData {
    color: vec4<f32>,
    metallic_roughness_pad: vec4<f32>,
}

@group(0)
@binding(0)
var<uniform> scene_data: SceneData;

@group(1)
@binding(0)
var<uniform> model_matrices: array<mat4x4<f32>, 1024>;

@group(2)
@binding(0)
var<uniform> material_data: PbrMaterialData;

@vertex
fn vs_main(
    @builtin(instance_index) instanceIdx : u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var mvp_matrix = scene_data.view_projection_matrix * model_matrices[instanceIdx];

    var result: VertexOutput;
    result.position = mvp_matrix * vec4<f32>(position, 1.0);
    result.normal_ws = normalize((model_matrices[instanceIdx] * vec4<f32>(normal, 0.0)).xyz);
    result.uv = uv;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var ambient_color = material_data.color.xyz * scene_data.ambient_light_color.xyz;

    var dot_power = saturate( dot(-vertex.normal_ws, scene_data.sun_light_direction_intensity.xyz));
    var light_color = material_data.color.xyz * (scene_data.sun_light_color.xyz * scene_data.sun_light_direction_intensity.w * dot_power );

    return vec4<f32>(ambient_color + light_color, material_data.color.w);
}