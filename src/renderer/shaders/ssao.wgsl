struct SSAOParams {
    projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
    radius: f32,
    bias: f32,
    sample_count: f32,
    intensity: f32,
}

@group(0) @binding(0)
var<uniform> params: SSAOParams;

@group(0) @binding(1)
var depth_texture: texture_depth_2d;

@group(0) @binding(2)
var depth_sampler: sampler;

@group(0) @binding(3)
var noise_texture: texture_2d<f32>;

@group(0) @binding(4)
var noise_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32((vertex_index & 1u) << 1u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

fn reconstruct_view_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(
        uv.x * 2.0 - 1.0,
        1.0 - uv.y * 2.0,
        depth,
        1.0
    );

    var view_pos = params.inv_projection * ndc;
    view_pos = view_pos / view_pos.w;

    return view_pos.xyz;
}

fn reconstruct_view_normal(uv: vec2<f32>, depth: f32, view_pos: vec3<f32>) -> vec3<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(depth_texture));

    let depth_right = textureSample(depth_texture, depth_sampler, uv + vec2<f32>(texel_size.x, 0.0));
    let depth_top = textureSample(depth_texture, depth_sampler, uv + vec2<f32>(0.0, texel_size.y));

    let pos_right = reconstruct_view_position(uv + vec2<f32>(texel_size.x, 0.0), depth_right);
    let pos_top = reconstruct_view_position(uv + vec2<f32>(0.0, texel_size.y), depth_top);

    let ddx = pos_right - view_pos;
    let ddy = pos_top - view_pos;

    return normalize(cross(ddy, ddx));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center_depth = textureSample(depth_texture, depth_sampler, in.uv);

    if center_depth >= 0.9999 {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    let view_pos = reconstruct_view_position(in.uv, center_depth);
    let normal = reconstruct_view_normal(in.uv, center_depth, view_pos);

    let screen_size = vec2<f32>(textureDimensions(depth_texture));
    let noise_scale = screen_size / 4.0;
    let random_vec = textureSample(noise_texture, noise_sampler, in.uv * noise_scale).xyz * 2.0 - 1.0;

    let sample_kernel = array<vec3<f32>, 16>(
        vec3<f32>(0.0366, 0.0131, 0.0208),
        vec3<f32>(-0.0261, -0.0371, 0.0234),
        vec3<f32>(-0.0562, 0.0396, 0.0486),
        vec3<f32>(0.0734, -0.0289, 0.0613),
        vec3<f32>(0.0213, 0.0856, 0.0701),
        vec3<f32>(-0.0895, 0.0134, 0.0902),
        vec3<f32>(-0.0623, -0.0748, 0.1123),
        vec3<f32>(0.1089, -0.0512, 0.1189),
        vec3<f32>(0.0698, 0.1234, 0.1456),
        vec3<f32>(-0.1567, 0.0623, 0.1701),
        vec3<f32>(-0.1134, -0.1389, 0.1923),
        vec3<f32>(0.1823, -0.0967, 0.2134),
        vec3<f32>(0.1289, 0.1923, 0.2456),
        vec3<f32>(-0.2345, 0.1234, 0.2701),
        vec3<f32>(-0.1789, -0.2145, 0.2923),
        vec3<f32>(0.2567, -0.1678, 0.3134)
    );

    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var occlusion = 0.0;
    var sample_count = 0.0;

    for (var i = 0u; i < 16u; i++) {
        var sample_offset = tbn * sample_kernel[i];
        sample_offset = sample_offset * params.radius;
        let sample_pos = view_pos + sample_offset;

        var clip_space = params.projection * vec4<f32>(sample_pos, 1.0);
        clip_space = clip_space / clip_space.w;

        let sample_uv = vec2<f32>(
            clip_space.x * 0.5 + 0.5,
            0.5 - clip_space.y * 0.5
        );

        if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 {
            continue;
        }

        let sample_depth = textureSample(depth_texture, depth_sampler, sample_uv);
        let scene_view_pos = reconstruct_view_position(sample_uv, sample_depth);

        let depth_diff = scene_view_pos.z - sample_pos.z;

        let range_check = smoothstep(0.0, 1.0, params.radius / abs(depth_diff));

        if depth_diff >= params.bias {
            occlusion += range_check;
            sample_count += 1.0;
        }
    }

    if sample_count > 0.0 {
        occlusion = occlusion / sample_count;
    }

    occlusion = 1.0 - occlusion * params.intensity;
    occlusion = clamp(occlusion, 0.0, 1.0);

    return vec4<f32>(occlusion, occlusion, occlusion, 1.0);
}
