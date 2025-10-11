// Textured vertex/fragment shader for rendering textured meshes

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(2) @binding(0)
var texture_sampler: sampler;

@group(2) @binding(1)
var texture_data: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);
    out.clip_position = camera.view_projection * world_position;
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture_data, texture_sampler, in.uv);
    return tex_color * in.color;
}
