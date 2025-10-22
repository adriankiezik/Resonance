use crate::addons::DebugUiState;
use crate::core::{Profiler, TimingEntry};
use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
enum SortColumn {
    Name,
    AvgTime,
    MinTime,
    MaxTime,
    Percent,
    Calls,
}

#[derive(Resource)]
pub struct ProfilerUiState {
    sort_column: SortColumn,
    sort_ascending: bool,
    filter: String,
}

impl Default for ProfilerUiState {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::AvgTime,
            sort_ascending: false,
            filter: String::new(),
        }
    }
}

pub fn render_profiler_panel(world: &mut World, ctx: &egui::Context) {
    let state = world.get_resource::<DebugUiState>();
    if state.map_or(false, |s| !s.show_profiler) {
        return;
    }

    if !world.contains_resource::<Profiler>() {
        return;
    }

    if !world.contains_resource::<ProfilerUiState>() {
        world.insert_resource(ProfilerUiState::default());
    }

    let (timings, time_since_log) = {
        let profiler = world.get_resource::<Profiler>().unwrap();
        (profiler.timings().clone(), profiler.time_since_last_log())
    };

    let mut ui_state = world.get_resource_mut::<ProfilerUiState>().unwrap();

    egui::Window::new("Profiler")
        .default_pos([420.0, 10.0])
        .default_size([700.0, 500.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Window: {:.1}s / 5.0s",
                    time_since_log.as_secs_f64()
                ));
                ui.separator();

                ui.label("Filter:");
                ui.text_edit_singleline(&mut ui_state.filter);
            });

            ui.add_space(5.0);
            ui.separator();

            let mut timing_vec: Vec<(&String, &TimingEntry)> = timings.iter().collect();

            if !ui_state.filter.is_empty() {
                timing_vec.retain(|(name, _)| {
                    name.to_lowercase()
                        .contains(&ui_state.filter.to_lowercase())
                });
            }

            match ui_state.sort_column {
                SortColumn::Name => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.0.cmp(b.0));
                    } else {
                        timing_vec.sort_by(|a, b| b.0.cmp(a.0));
                    }
                }
                SortColumn::AvgTime => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.1.avg_time.cmp(&b.1.avg_time));
                    } else {
                        timing_vec.sort_by(|a, b| b.1.avg_time.cmp(&a.1.avg_time));
                    }
                }
                SortColumn::MinTime => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.1.min_time.cmp(&b.1.min_time));
                    } else {
                        timing_vec.sort_by(|a, b| b.1.min_time.cmp(&a.1.min_time));
                    }
                }
                SortColumn::MaxTime => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.1.max_time.cmp(&b.1.max_time));
                    } else {
                        timing_vec.sort_by(|a, b| b.1.max_time.cmp(&a.1.max_time));
                    }
                }
                SortColumn::Percent => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.1.total_time.cmp(&b.1.total_time));
                    } else {
                        timing_vec.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));
                    }
                }
                SortColumn::Calls => {
                    if ui_state.sort_ascending {
                        timing_vec.sort_by(|a, b| a.1.call_count.cmp(&b.1.call_count));
                    } else {
                        timing_vec.sort_by(|a, b| b.1.call_count.cmp(&a.1.call_count));
                    }
                }
            }

            use egui_extras::{Column, TableBuilder};

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        if ui.button("Name").clicked() {
                            if ui_state.sort_column == SortColumn::Name {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::Name;
                                ui_state.sort_ascending = true;
                            }
                        }
                    });
                    header.col(|ui| {
                        if ui.button("Avg Time").clicked() {
                            if ui_state.sort_column == SortColumn::AvgTime {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::AvgTime;
                                ui_state.sort_ascending = false;
                            }
                        }
                    });
                    header.col(|ui| {
                        if ui.button("Min Time").clicked() {
                            if ui_state.sort_column == SortColumn::MinTime {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::MinTime;
                                ui_state.sort_ascending = true;
                            }
                        }
                    });
                    header.col(|ui| {
                        if ui.button("Max Time").clicked() {
                            if ui_state.sort_column == SortColumn::MaxTime {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::MaxTime;
                                ui_state.sort_ascending = false;
                            }
                        }
                    });
                    header.col(|ui| {
                        if ui.button("% Window").clicked() {
                            if ui_state.sort_column == SortColumn::Percent {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::Percent;
                                ui_state.sort_ascending = false;
                            }
                        }
                    });
                    header.col(|ui| {
                        if ui.button("Calls").clicked() {
                            if ui_state.sort_column == SortColumn::Calls {
                                ui_state.sort_ascending = !ui_state.sort_ascending;
                            } else {
                                ui_state.sort_column = SortColumn::Calls;
                                ui_state.sort_ascending = false;
                            }
                        }
                    });
                })
                .body(|mut body| {
                    for (name, timing) in timing_vec.iter() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(*name);
                            });

                            let avg_ms = timing.avg_time.as_secs_f64() * 1000.0;
                            let color = if avg_ms > 5.0 {
                                egui::Color32::RED
                            } else if avg_ms > 1.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GREEN
                            };

                            row.col(|ui| {
                                ui.colored_label(color, format!("{:.2}ms", avg_ms));
                            });

                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.2}ms",
                                    timing.min_time.as_secs_f64() * 1000.0
                                ));
                            });

                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.2}ms",
                                    timing.max_time.as_secs_f64() * 1000.0
                                ));
                            });

                            let window_duration = time_since_log.as_secs_f64().max(0.001);
                            let percent =
                                (timing.total_time.as_secs_f64() / window_duration) * 100.0;

                            row.col(|ui| {
                                ui.label(format!("{:.2}%", percent));
                            });

                            row.col(|ui| {
                                ui.label(format!("{}", timing.call_count));
                            });
                        });
                    }
                });
        });
}
