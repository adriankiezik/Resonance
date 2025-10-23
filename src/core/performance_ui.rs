use crate::addons::DebugUiState;
use crate::core::PerformanceAnalytics;
use bevy_ecs::prelude::*;

pub fn render_performance_panel(world: &mut World, ctx: &egui::Context) {
    let state = world.get_resource::<DebugUiState>();
    if state.map_or(false, |s| !s.show_performance) {
        log::trace!("Performance panel hidden");
        return;
    }

    log::trace!("Rendering performance panel");

    let analytics = world.get_resource::<PerformanceAnalytics>();
    let Some(analytics) = analytics else {
        log::warn!("PerformanceAnalytics not found!");
        return;
    };

    egui::Window::new("Performance Metrics")
        .default_pos([10.0, 10.0])
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            let fps = analytics.fps();
            let min_fps = analytics.min_fps();
            let max_fps = analytics.max_fps();
            let avg_frame_time = analytics.avg_frame_time();
            let min_frame_time = analytics.min_frame_time();
            let max_frame_time = analytics.max_frame_time();
            let total_frames = analytics.total_frames();

            ui.heading("Frame Rate");
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", fps));
                ui.separator();
                ui.label(format!("Min: {:.1}", min_fps));
                ui.separator();
                ui.label(format!("Max: {:.1}", max_fps));
            });

            ui.add_space(5.0);

            let frame_times: Vec<f64> = analytics
                .frame_times()
                .iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();

            if !frame_times.is_empty() {
                use egui_plot::{HLine, Line, Plot, PlotPoints};

                let fps_points: PlotPoints = frame_times
                    .iter()
                    .enumerate()
                    .map(|(i, &ft)| {
                        let fps = if ft > 0.0 { 1000.0 / ft } else { 0.0 };
                        [i as f64, fps]
                    })
                    .collect();

                let fps_line =
                    Line::new("FPS", fps_points).color(egui::Color32::from_rgb(100, 200, 100));

                Plot::new("fps_plot")
                    .height(120.0)
                    .show_axes([false, true])
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(fps_line);
                        plot_ui.hline(HLine::new("60 FPS", 60.0).color(egui::Color32::YELLOW));
                        plot_ui.hline(HLine::new("30 FPS", 30.0).color(egui::Color32::RED));
                    });
            }

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui.heading("Frame Time");
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Avg: {:.2}ms",
                    avg_frame_time.as_secs_f64() * 1000.0
                ));
                ui.separator();
                ui.label(format!(
                    "Min: {:.2}ms",
                    min_frame_time.as_secs_f64() * 1000.0
                ));
                ui.separator();
                ui.label(format!(
                    "Max: {:.2}ms",
                    max_frame_time.as_secs_f64() * 1000.0
                ));
            });

            let target_frame_time = 16.67;
            let avg_ms = avg_frame_time.as_secs_f64() * 1000.0;
            let budget_color = if avg_ms > target_frame_time {
                egui::Color32::RED
            } else {
                egui::Color32::GREEN
            };

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Frame Budget (16.67ms @ 60fps):");
                ui.colored_label(
                    budget_color,
                    format!("{:.1}%", (avg_ms / target_frame_time) * 100.0),
                );
            });

            ui.add_space(5.0);

            if !frame_times.is_empty() {
                use egui_plot::{HLine, Line, Plot, PlotPoints};

                let frame_time_points: PlotPoints = frame_times
                    .iter()
                    .enumerate()
                    .map(|(i, &ft)| [i as f64, ft])
                    .collect();

                let frame_time_line = Line::new("Frame Time (ms)", frame_time_points)
                    .color(egui::Color32::from_rgb(100, 150, 250));

                Plot::new("frame_time_plot")
                    .height(120.0)
                    .show_axes([false, true])
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(frame_time_line);
                        plot_ui.hline(
                            HLine::new("60 FPS (16.67ms)", 16.67).color(egui::Color32::YELLOW),
                        );
                        plot_ui
                            .hline(HLine::new("30 FPS (33.33ms)", 33.33).color(egui::Color32::RED));
                    });
            }

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui.heading("Statistics");
            ui.label(format!("Total Frames: {}", total_frames));

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui.heading("Memory Usage");

            let mut system = sysinfo::System::new();
            system.refresh_memory();

            if let Ok(pid) = sysinfo::get_current_pid() {
                system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);

                if let Some(process) = system.process(pid) {
                    let memory_bytes = process.memory();
                    let memory_mb = memory_bytes as f64 / 1024.0 / 1024.0;

                    let (memory_value, memory_unit) = if memory_mb >= 1024.0 {
                        (memory_mb / 1024.0, "GB")
                    } else {
                        (memory_mb, "MB")
                    };

                    ui.horizontal(|ui| {
                        ui.label("Process Memory:");
                        ui.colored_label(
                            egui::Color32::from_rgb(150, 200, 255),
                            format!("{:.1} {}", memory_value, memory_unit)
                        );
                    });

                    let total_memory = system.total_memory() as f64 / 1024.0 / 1024.0;
                    let memory_percent = (memory_mb / total_memory) * 100.0;

                    ui.horizontal(|ui| {
                        ui.label("System Memory:");
                        ui.label(format!(
                            "{:.1} GB / {:.1} GB ({:.1}%)",
                            system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
                            total_memory / 1024.0,
                            memory_percent
                        ));
                    });
                } else {
                    ui.label("Memory info unavailable");
                }
            } else {
                ui.label("Memory info unavailable");
            }

            if let Some(memory_tracker) = world.get_resource::<crate::core::MemoryTracker>() {
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.heading("GPU Memory");

                ui.horizontal(|ui| {
                    ui.label("Total GPU:");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        crate::core::format_bytes(memory_tracker.gpu.total()),
                    );
                });

                ui.indent("gpu_breakdown", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Depth Textures:");
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.depth_textures));
                    });
                    ui.horizontal(|ui| {
                        ui.label("SSAO Textures:");
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.ssao_textures));
                    });
                    ui.horizontal(|ui| {
                        ui.label("MSAA Textures:");
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.msaa_textures));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Camera Buffer:");
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.camera_buffer));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Mesh Vertex Buffers ({}):", memory_tracker.gpu_mesh_count()));
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.mesh_vertex_buffers));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mesh Index Buffers:");
                        ui.label(crate::core::format_bytes(memory_tracker.gpu.mesh_index_buffers));
                    });
                    if memory_tracker.gpu.other_buffers > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Other Buffers:");
                            ui.label(crate::core::format_bytes(memory_tracker.gpu.other_buffers));
                        });
                    }
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                ui.heading("Asset Memory");

                ui.horizontal(|ui| {
                    ui.label("Total Assets:");
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 255, 150),
                        crate::core::format_bytes(memory_tracker.assets.total()),
                    );
                });

                ui.indent("asset_breakdown", |ui| {
                    if memory_tracker.assets.textures > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Textures:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.textures));
                        });
                    }
                    if memory_tracker.assets.meshes > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Meshes:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.meshes));
                        });
                    }
                    if memory_tracker.assets.audio > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Audio:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.audio));
                        });
                    }
                    if memory_tracker.assets.shaders > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Shaders:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.shaders));
                        });
                    }
                    if memory_tracker.assets.fonts > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Fonts:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.fonts));
                        });
                    }
                    if memory_tracker.assets.other > 0 {
                        ui.horizontal(|ui| {
                            ui.label("Other:");
                            ui.label(crate::core::format_bytes(memory_tracker.assets.other));
                        });
                    }
                });
            }
        });
}
