struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
}

struct DirectionalLight {
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    _padding: f32,
}

struct AmbientLight {
    color: vec3<f32>,
    intensity: f32,
}

struct LightingUniform {
    directional: DirectionalLight,
    ambient: AmbientLight,
    point_light_count: u32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(2) @binding(0)
var<uniform> lighting: LightingUniform;

@group(3) @binding(0)
var ssao_texture: texture_2d<f32>;

@group(3) @binding(1)
var ssao_sampler: sampler;

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
    @location(3) screen_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.screen_position = out.clip_position;

    out.world_normal = model.normal_matrix * in.normal;
    out.uv = in.uv;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);

    // Convert from clip space [-1, 1] to UV space [0, 1]
    let ndc = in.screen_position.xy / in.screen_position.w;
    let screen_uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let ao = textureSample(ssao_texture, ssao_sampler, screen_uv).r;

    // Ambient lighting with SSAO
    let ambient = lighting.ambient.color * lighting.ambient.intensity * ao;

    // Directional light (sun/moon)
    let light_dir = normalize(-lighting.directional.direction);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse = lighting.directional.color * lighting.directional.intensity * diffuse_strength;

    // Combine lighting
    let final_lighting = ambient + diffuse;

    // Apply final color
    let color = in.color * final_lighting;

    return vec4<f32>(color, 1.0);
}
