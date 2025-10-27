struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: array<vec4<f32>, 3>,  // Changed from mat3x3 to match Rust layout [[f32; 4]; 3]
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
    ao_mode: u32,
    ao_debug: u32,
    _padding1: f32,
    _padding2: vec3<f32>,
    _padding3: f32,
    _padding4: vec3<f32>,
    _padding5: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> models: array<ModelUniform>;

@group(1) @binding(1)
var<storage, read> visibility: array<u32>;

@group(2) @binding(0)
var<uniform> lighting: LightingUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
    @location(3) ao: f32,
}

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    if instance_index < arrayLength(&visibility) && visibility[instance_index] == 0u {
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.world_normal = vec3<f32>(0.0);
        out.uv = vec2<f32>(0.0);
        out.color = vec3<f32>(0.0);
        out.ao = 0.0;
        return out;
    }

    let model = models[instance_index];
    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;

    // Multiply normal by mat3 stored as array<vec4<f32>, 3>
    out.world_normal = vec3<f32>(
        dot(model.normal_matrix[0].xyz, in.normal),
        dot(model.normal_matrix[1].xyz, in.normal),
        dot(model.normal_matrix[2].xyz, in.normal)
    );
    out.uv = in.uv;
    out.color = in.color;
    out.ao = in.ao;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);

    // Use vertex AO only (no SSAO)
    let ao = in.ao;

    if lighting.ao_debug == 1u {
        return vec4<f32>(ao, ao, ao, 1.0);
    }

    let ambient = lighting.ambient.color * lighting.ambient.intensity * ao;

    let light_dir = normalize(-lighting.directional.direction);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse = lighting.directional.color * lighting.directional.intensity * diffuse_strength;

    let final_lighting = ambient + diffuse;

    let color = in.color * final_lighting;

    return vec4<f32>(color, 1.0);
}
