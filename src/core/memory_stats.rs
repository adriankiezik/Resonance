use bevy_ecs::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMemoryStats {
    pub process_bytes: u64,
    pub system_used_bytes: u64,
    pub system_total_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuMemoryStats {
    pub depth_textures: u64,
    pub ssao_textures: u64,
    pub msaa_textures: u64,
    pub camera_buffer: u64,
    pub mesh_vertex_buffers: u64,
    pub mesh_index_buffers: u64,
    pub other_buffers: u64,
}

impl GpuMemoryStats {
    pub fn total(&self) -> u64 {
        self.depth_textures
            + self.ssao_textures
            + self.msaa_textures
            + self.camera_buffer
            + self.mesh_vertex_buffers
            + self.mesh_index_buffers
            + self.other_buffers
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AssetMemoryStats {
    pub textures: u64,
    pub meshes: u64,
    pub audio: u64,
    pub shaders: u64,
    pub fonts: u64,
    pub other: u64,
}

impl AssetMemoryStats {
    pub fn total(&self) -> u64 {
        self.textures + self.meshes + self.audio + self.shaders + self.fonts + self.other
    }
}

#[derive(Resource)]
pub struct MemoryTracker {
    pub gpu: GpuMemoryStats,
    pub assets: AssetMemoryStats,
    pub process: ProcessMemoryStats,
    mesh_sizes: Arc<DashMap<crate::assets::AssetId, (u64, u64)>>,
    system: sysinfo::System,
    last_process_update: Instant,
    update_interval: Duration,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self {
            gpu: Default::default(),
            assets: Default::default(),
            process: Default::default(),
            mesh_sizes: Arc::new(DashMap::new()),
            system: sysinfo::System::new(),
            last_process_update: Instant::now(),
            update_interval: Duration::from_millis(500),
        }
    }
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_depth_texture(&mut self, size: u64) {
        self.gpu.depth_textures = size;
    }

    pub fn track_ssao_textures(&mut self, size: u64) {
        self.gpu.ssao_textures = size;
    }

    pub fn track_msaa_textures(&mut self, size: u64) {
        self.gpu.msaa_textures = size;
    }

    pub fn track_camera_buffer(&mut self, size: u64) {
        self.gpu.camera_buffer = size;
    }

    pub fn track_other_buffer(&mut self, size: u64) {
        self.gpu.other_buffers += size;
    }

    pub fn track_mesh_gpu(&mut self, id: crate::assets::AssetId, vertex_size: u64, index_size: u64) {
        let old = self.mesh_sizes.insert(id, (vertex_size, index_size));

        if let Some((old_v, old_i)) = old {
            self.gpu.mesh_vertex_buffers = self.gpu.mesh_vertex_buffers.saturating_sub(old_v);
            self.gpu.mesh_index_buffers = self.gpu.mesh_index_buffers.saturating_sub(old_i);
        }

        self.gpu.mesh_vertex_buffers += vertex_size;
        self.gpu.mesh_index_buffers += index_size;
    }

    pub fn untrack_mesh_gpu(&mut self, id: &crate::assets::AssetId) {
        if let Some((_, (vertex_size, index_size))) = self.mesh_sizes.remove(id) {
            self.gpu.mesh_vertex_buffers = self.gpu.mesh_vertex_buffers.saturating_sub(vertex_size);
            self.gpu.mesh_index_buffers = self.gpu.mesh_index_buffers.saturating_sub(index_size);
        }
    }

    pub fn track_texture_asset(&mut self, size: u64) {
        self.assets.textures += size;
    }

    pub fn track_mesh_asset(&mut self, size: u64) {
        self.assets.meshes += size;
    }

    pub fn track_audio_asset(&mut self, size: u64) {
        self.assets.audio += size;
    }

    pub fn track_shader_asset(&mut self, size: u64) {
        self.assets.shaders += size;
    }

    pub fn track_font_asset(&mut self, size: u64) {
        self.assets.fonts += size;
    }

    pub fn total_memory(&self) -> u64 {
        self.gpu.total() + self.assets.total()
    }

    pub fn gpu_mesh_count(&self) -> usize {
        self.mesh_sizes.len()
    }

    pub fn update_process_memory(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_process_update) < self.update_interval {
            return;
        }

        self.system.refresh_memory();

        if let Ok(pid) = sysinfo::get_current_pid() {
            self.system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);

            if let Some(process) = self.system.process(pid) {
                self.process.process_bytes = process.memory();
                self.process.system_used_bytes = self.system.used_memory();
                self.process.system_total_bytes = self.system.total_memory();
            }
        }

        self.last_process_update = now;
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
