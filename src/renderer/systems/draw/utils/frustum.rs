use crate::renderer::{Camera, components::CachedFrustum};
use crate::renderer::camera::Frustum;
use crate::transform::GlobalTransform;
use bevy_ecs::prelude::*;

pub fn calculate_frustum_with_cache(
    commands: &mut Commands,
    cached_frustum: Option<ResMut<CachedFrustum>>,
    camera: &Camera,
    transform: &GlobalTransform,
) -> (Option<Frustum>, bool) {
    let current_matrix = transform.matrix();

    if let Some(mut cache) = cached_frustum {
        if cache.camera_transform == current_matrix {
            (Some(cache.frustum.clone()), false)
        } else {
            let frustum = camera.frustum(transform);
            cache.frustum = frustum.clone();
            cache.camera_transform = current_matrix;
            (Some(frustum), true)
        }
    } else {
        let frustum = camera.frustum(transform);
        commands.insert_resource(CachedFrustum {
            frustum: frustum.clone(),
            camera_transform: current_matrix,
        });
        (Some(frustum), true)
    }
}
