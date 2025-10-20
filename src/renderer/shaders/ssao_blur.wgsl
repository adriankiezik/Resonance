struct BlurParams {
    texel_size: vec2<f32>,
    blur_radius: f32,
    depth_threshold: f32,
}

@group(0) @binding(0)
var<uniform> params: BlurParams;

@group(0) @binding(1)
var ssao_texture: texture_2d<f32>;

@group(0) @binding(2)
var ssao_sampler: sampler;

@group(0) @binding(3)
var depth_texture: texture_depth_2d;

@group(0) @binding(4)
var depth_sampler: sampler;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center_depth = textureSample(depth_texture, depth_sampler, in.uv);

    if center_depth >= 0.9999 {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    let center_ao = textureSample(ssao_texture, ssao_sampler, in.uv).r;

    var total_weight = 1.0;
    var total_ao = center_ao;

    let kernel_size = i32(params.blur_radius);

    for (var x = -kernel_size; x <= kernel_size; x++) {
        for (var y = -kernel_size; y <= kernel_size; y++) {
            if x == 0 && y == 0 {
                continue;
            }

            let offset = vec2<f32>(f32(x), f32(y)) * params.texel_size;
            let sample_uv = in.uv + offset;

            if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 {
                continue;
            }

            let sample_depth = textureSample(depth_texture, depth_sampler, sample_uv);
            let sample_ao = textureSample(ssao_texture, ssao_sampler, sample_uv).r;

            let depth_diff = abs(center_depth - sample_depth);

            if depth_diff < params.depth_threshold {
                let spatial_weight = exp(-f32(x * x + y * y) / (2.0 * params.blur_radius * params.blur_radius));
                let depth_weight = exp(-depth_diff * depth_diff / (2.0 * params.depth_threshold * params.depth_threshold));
                let weight = spatial_weight * depth_weight;

                total_ao += sample_ao * weight;
                total_weight += weight;
            }
        }
    }

    let blurred_ao = total_ao / total_weight;

    return vec4<f32>(blurred_ao, blurred_ao, blurred_ao, 1.0);
}
