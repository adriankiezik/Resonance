/// Debug rendering utilities for visualizing game state
///
/// Provides simple debug rendering capabilities for AABBs, lines, and other
/// debug visualizations. Useful for debugging physics, culling, and spatial issues.
///
/// # Example
/// ```no_run
/// use resonance::prelude::*;
/// use resonance::addons::debug_render::*;
///
/// fn debug_system(mut debug: ResMut<DebugRenderer>) {
///     // Draw a red AABB
///     debug.draw_aabb(
///         Vec3::ZERO,
///         Vec3::new(1.0, 1.0, 1.0),
///         Vec3::new(1.0, 0.0, 0.0)
///     );
/// }
/// ```

use bevy_ecs::prelude::*;
use glam::Vec3;

/// Debug line to be rendered
#[derive(Clone, Debug)]
pub struct DebugLine {
    pub from: Vec3,
    pub to: Vec3,
    pub color: Vec3,
}

/// Resource for managing debug rendering
///
/// Collects debug primitives each frame and renders them as wireframes.
/// All debug primitives are cleared at the end of each frame.
#[derive(Resource, Default)]
pub struct DebugRenderer {
    lines: Vec<DebugLine>,
    enabled: bool,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            enabled: true,
        }
    }

    /// Enables or disables debug rendering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Draws a line between two points
    pub fn draw_line(&mut self, from: Vec3, to: Vec3, color: Vec3) {
        if self.enabled {
            self.lines.push(DebugLine { from, to, color });
        }
    }

    /// Draws an axis-aligned bounding box
    pub fn draw_aabb(&mut self, min: Vec3, max: Vec3, color: Vec3) {
        if !self.enabled {
            return;
        }

        // Bottom face
        self.draw_line(Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), color);
        self.draw_line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), color);
        self.draw_line(Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z), color);
        self.draw_line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, min.y, min.z), color);

        // Top face
        self.draw_line(Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        self.draw_line(Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z), color);
        self.draw_line(Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z), color);
        self.draw_line(Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z), color);

        // Vertical edges
        self.draw_line(Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), color);
        self.draw_line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        self.draw_line(Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), color);
        self.draw_line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), color);
    }

    /// Draws a camera frustum for visualization
    pub fn draw_frustum(&mut self, _frustum: &crate::renderer::camera::Frustum, _color: Vec3) {
        // Simplified frustum visualization
        // In a full implementation, would extract frustum corners and draw all edges
        if self.enabled {
            // This is a stub - full implementation would require frustum corner extraction
            log::warn!("Frustum visualization not yet fully implemented");
        }
    }

    /// Gets all debug lines for rendering
    pub fn lines(&self) -> &[DebugLine] {
        &self.lines
    }

    /// Clears all debug primitives (called automatically each frame)
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// System that clears debug rendering each frame
fn clear_debug_renderer(mut debug: ResMut<DebugRenderer>) {
    debug.clear();
}

/// Plugin that adds debug rendering capabilities
#[derive(Default)]
pub struct DebugRenderPlugin;

impl crate::app::Plugin for DebugRenderPlugin {
    fn build(&self, engine: &mut crate::app::Resonance) {
        engine.world.insert_resource(DebugRenderer::new());

        // Clear debug primitives at the end of each frame
        use crate::app::Stage;
        if let Some(schedule) = engine.schedules.get_mut(Stage::Last) {
            schedule.add_systems(clear_debug_renderer);
        }
    }

    fn name(&self) -> &'static str {
        "DebugRenderPlugin"
    }
}
