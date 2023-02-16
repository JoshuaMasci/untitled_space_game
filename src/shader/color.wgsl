struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> view_projection_matrix: mat4x4<f32>;

@group(1)
@binding(0)
var<uniform> model_matrices: array<mat4x4<f32>, 1024>;

@vertex
fn vs_main(
    @builtin(instance_index) instanceIdx : u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.color = abs(normal);
    result.position =  view_projection_matrix * ( model_matrices[instanceIdx] * vec4<f32>(position, 1.0));
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(vertex.color, 1.0);
}