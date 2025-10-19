struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;

    out.world_normal = model.normal_matrix * in.normal;
    out.uv = in.uv;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(normal, light_dir), 0.0);
    let lighting = ambient + diffuse * 0.7;

    let color = in.color * lighting;

    return vec4<f32>(color, 1.0);
}
