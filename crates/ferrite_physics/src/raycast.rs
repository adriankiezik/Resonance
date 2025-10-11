//! Raycasting system for physics queries.
//!
//! Provides ray-based collision detection for:
//! - Ground detection (character controllers)
//! - Line of sight checks (NPCs, targeting)
//! - Projectile hit detection
//! - Click-to-move ground positioning

use crate::collision::{Aabb, Collider, ColliderShape};
use bevy_ecs::prelude::*;
use ferrite_core::math::*;
use ferrite_transform::Transform;

/// A ray for raycasting (origin + direction)
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin point of the ray
    pub origin: Vec3,
    /// Direction of the ray (should be normalized)
    pub direction: Vec3,
    /// Maximum distance to check (use f32::INFINITY for unlimited)
    pub max_distance: f32,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            max_distance,
        }
    }

    /// Get a point along the ray at distance t
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Result of a raycast hit
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// Entity that was hit
    pub entity: Entity,
    /// Point where the ray hit
    pub point: Vec3,
    /// Normal at the hit point
    pub normal: Vec3,
    /// Distance from ray origin to hit point
    pub distance: f32,
}

impl RaycastHit {
    /// Create a new raycast hit
    pub fn new(entity: Entity, point: Vec3, normal: Vec3, distance: f32) -> Self {
        Self {
            entity,
            point,
            normal,
            distance,
        }
    }
}

/// Raycast against an AABB
pub fn raycast_aabb(ray: &Ray, aabb: &Aabb) -> Option<f32> {
    let mut tmin: f32 = 0.0;
    let mut tmax: f32 = ray.max_distance;

    // Check each axis
    for i in 0..3 {
        let origin = ray.origin.to_array()[i];
        let dir = ray.direction.to_array()[i];
        let min = aabb.min.to_array()[i];
        let max = aabb.max.to_array()[i];

        if dir.abs() < 1e-8 {
            // Ray is parallel to slab, check if origin is within bounds
            if origin < min || origin > max {
                return None;
            }
        } else {
            // Compute intersection t values with near and far planes
            let inv_d = 1.0 / dir;
            let mut t1 = (min - origin) * inv_d;
            let mut t2 = (max - origin) * inv_d;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            if tmin > tmax {
                return None;
            }
        }
    }

    if tmin > 0.0 {
        Some(tmin)
    } else if tmax > 0.0 {
        Some(0.0) // Ray starts inside AABB
    } else {
        None
    }
}

/// Raycast against a sphere
pub fn raycast_sphere(ray: &Ray, center: Vec3, radius: f32) -> Option<f32> {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);

    // Return closest positive hit
    if t1 > 0.0 && t1 <= ray.max_distance {
        Some(t1)
    } else if t2 > 0.0 && t2 <= ray.max_distance {
        Some(t2)
    } else {
        None
    }
}

/// Raycast against a box collider
pub fn raycast_box(ray: &Ray, center: Vec3, half_extents: Vec3) -> Option<(f32, Vec3)> {
    let aabb = Aabb::from_center_half_extents(center, half_extents);
    let t = raycast_aabb(ray, &aabb)?;
    let hit_point = ray.point_at(t);

    // Calculate normal based on which face was hit
    let local = hit_point - center;
    let mut normal = Vec3::ZERO;
    let mut min_dist = f32::INFINITY;

    // Check each axis to find which face we hit
    for i in 0..3 {
        let extent = half_extents.to_array()[i];
        let dist_to_max = (extent - local.to_array()[i]).abs();
        let dist_to_min = (extent + local.to_array()[i]).abs();

        if dist_to_max < min_dist {
            min_dist = dist_to_max;
            normal = Vec3::ZERO;
            normal.as_mut_array()[i] = 1.0;
        }
        if dist_to_min < min_dist {
            min_dist = dist_to_min;
            normal = Vec3::ZERO;
            normal.as_mut_array()[i] = -1.0;
        }
    }

    Some((t, normal))
}

/// Raycast against a capsule collider
pub fn raycast_capsule(
    ray: &Ray,
    center: Vec3,
    half_height: f32,
    radius: f32,
) -> Option<(f32, Vec3)> {
    // Simplified capsule raycast: treat as cylinder + two spheres
    // Top and bottom sphere centers (capsule aligned with Y axis)
    let top_center = center + Vec3::new(0.0, half_height, 0.0);
    let bottom_center = center - Vec3::new(0.0, half_height, 0.0);

    let mut closest_t = f32::INFINITY;
    let mut closest_normal = Vec3::ZERO;

    // Check top sphere
    if let Some(t) = raycast_sphere(ray, top_center, radius) {
        if t < closest_t {
            closest_t = t;
            let hit_point = ray.point_at(t);
            closest_normal = (hit_point - top_center).normalize();
        }
    }

    // Check bottom sphere
    if let Some(t) = raycast_sphere(ray, bottom_center, radius) {
        if t < closest_t {
            closest_t = t;
            let hit_point = ray.point_at(t);
            closest_normal = (hit_point - bottom_center).normalize();
        }
    }

    // Check cylindrical body (simplified)
    // For a full implementation, we'd need cylinder-ray intersection

    if closest_t < f32::INFINITY {
        Some((closest_t, closest_normal))
    } else {
        None
    }
}

/// Raycast against a collider at a specific position
pub fn raycast_collider(
    ray: &Ray,
    collider: &Collider,
    position: Vec3,
) -> Option<(f32, Vec3)> {
    match &collider.shape {
        ColliderShape::Box { half_extents } => raycast_box(ray, position, *half_extents),
        ColliderShape::Sphere { radius } => {
            raycast_sphere(ray, position, *radius).map(|t| {
                let hit_point = ray.point_at(t);
                let normal = (hit_point - position).normalize();
                (t, normal)
            })
        }
        ColliderShape::Capsule {
            half_height,
            radius,
        } => raycast_capsule(ray, position, *half_height, *radius),
    }
}

/// Perform a raycast against all entities with colliders.
/// Returns the closest hit, if any.
pub fn raycast_world(
    ray: &Ray,
    query: &Query<(Entity, &Collider, &Transform)>,
) -> Option<RaycastHit> {
    let mut closest_hit: Option<RaycastHit> = None;
    let mut closest_distance = ray.max_distance;

    for (entity, collider, transform) in query.iter() {
        if let Some((distance, normal)) = raycast_collider(ray, collider, transform.position) {
            if distance < closest_distance {
                closest_distance = distance;
                let point = ray.point_at(distance);
                closest_hit = Some(RaycastHit::new(entity, point, normal, distance));
            }
        }
    }

    closest_hit
}

/// Helper trait to add as_mut_array to Vec3
trait Vec3Ext {
    fn as_mut_array(&mut self) -> &mut [f32; 3];
}

impl Vec3Ext for Vec3 {
    fn as_mut_array(&mut self) -> &mut [f32; 3] {
        unsafe { std::mem::transmute(self) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_creation() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 10.0);
        assert_eq!(ray.origin, Vec3::ZERO);
        assert_eq!(ray.direction, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(ray.max_distance, 10.0);
    }

    #[test]
    fn test_ray_point_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 10.0);
        let point = ray.point_at(5.0);
        assert_eq!(point, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_raycast_sphere_hit() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 20.0);
        let hit = raycast_sphere(&ray, Vec3::ZERO, 1.0);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 4.0).abs() < 0.01); // Should hit at distance 4.0
    }

    #[test]
    fn test_raycast_sphere_miss() {
        let ray = Ray::new(Vec3::new(-5.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 20.0);
        let hit = raycast_sphere(&ray, Vec3::ZERO, 1.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_raycast_aabb_hit() {
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 20.0);
        let aabb = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
        let hit = raycast_aabb(&ray, &aabb);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 4.0).abs() < 0.01); // Should hit at distance 4.0
    }
}
