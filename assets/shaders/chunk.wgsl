#import bevy_pbr::mesh_functions

@group(1) @binding(0) var array_texture: texture_2d_array<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct CustomVertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) index: u32,
}

struct CustomVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) index: u32,
}

@vertex
fn vertex(vertex: CustomVertex) -> CustomVertexOutput {
    var out: CustomVertexOutput;
    var model = mesh_functions::get_model_matrix(vertex.instance_index);
    out.clip_position = mesh_functions::mesh_position_local_to_clip(
        model,
        vec4<f32>(vertex.position, 1.0),
    );
    out.uv = vertex.uv;
    out.index = vertex.index;
    return out;
}

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: CustomVertexOutput,
) -> @location(0) vec4<f32> {
    let color = textureSample(array_texture, texture_sampler, mesh.uv, mesh.index);
    return color;
}
