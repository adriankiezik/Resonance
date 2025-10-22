struct Plane {
    normal: vec3<f32>,
    distance: f32,
}

struct Frustum {
    planes: array<Plane, 6>,
}

struct AABB {
    min: vec3<f32>,
    _padding1: f32,
    max: vec3<f32>,
    _padding2: f32,
}

struct EntityData {
    model_matrix: mat4x4<f32>,
    aabb: AABB,
    mesh_id: u32,
    entity_index: u32,
    _padding1: u32,
    _padding2: u32,
}

@group(0) @binding(0)
var<uniform> frustum: Frustum;

@group(0) @binding(1)
var<storage, read> entities: array<EntityData>;

@group(0) @binding(2)
var<storage, read_write> visibility: array<u32>;

fn transform_aabb(aabb: AABB, transform: mat4x4<f32>) -> AABB {
    let corners = array<vec3<f32>, 8>(
        (transform * vec4<f32>(aabb.min.x, aabb.min.y, aabb.min.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.min.x, aabb.min.y, aabb.max.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.min.x, aabb.max.y, aabb.min.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.min.x, aabb.max.y, aabb.max.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.max.x, aabb.min.y, aabb.min.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.max.x, aabb.min.y, aabb.max.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.max.x, aabb.max.y, aabb.min.z, 1.0)).xyz,
        (transform * vec4<f32>(aabb.max.x, aabb.max.y, aabb.max.z, 1.0)).xyz,
    );

    var min_corner = corners[0];
    var max_corner = corners[0];

    for (var i = 1u; i < 8u; i++) {
        min_corner = min(min_corner, corners[i]);
        max_corner = max(max_corner, corners[i]);
    }

    var result: AABB;
    result.min = min_corner;
    result.max = max_corner;
    return result;
}

fn frustum_contains_aabb(f: Frustum, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = f.planes[i];

        let p_vertex = vec3<f32>(
            select(aabb_min.x, aabb_max.x, plane.normal.x >= 0.0),
            select(aabb_min.y, aabb_max.y, plane.normal.y >= 0.0),
            select(aabb_min.z, aabb_max.z, plane.normal.z >= 0.0),
        );

        let distance = dot(plane.normal, p_vertex) + plane.distance;
        if distance < 0.0 {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let entity_idx = global_id.x;

    if entity_idx >= arrayLength(&entities) {
        return;
    }

    let entity = entities[entity_idx];

    let world_aabb = transform_aabb(entity.aabb, entity.model_matrix);

    let visible = frustum_contains_aabb(frustum, world_aabb.min, world_aabb.max);

    visibility[entity_idx] = select(0u, 1u, visible);
}
