use crate::db::models::{EntryFilter, NewProject, Project};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Projects");
    ui.label("Organize projects, assign visual color codes, and view tracked time per project.");
    ui.add_space(4.0);

    ui.collapsing(
        if app.editing_project_id.is_some() {
            "Edit Project"
        } else {
            "New Project"
        },
        |ui| {
            egui::Grid::new("proj_form")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut app.proj_name);
                    ui.end_row();

                    ui.label("Client");
                    ui.text_edit_singleline(&mut app.proj_client);
                    ui.end_row();

                    ui.label("Hourly rate");
                    ui.text_edit_singleline(&mut app.proj_rate);
                    ui.end_row();

                    ui.label("Color badge");
                    ui.horizontal_wrapped(|ui| {
                        for &(color_name, hex) in crate::domain::PROJECT_PALETTE {
                            let (r, g, b) = crate::domain::parse_hex_color(hex).unwrap();
                            let color_32 = egui::Color32::from_rgb(r, g, b);
                            let is_selected = app.proj_color.as_deref() == Some(hex);

                            let (rect, resp) = ui
                                .allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                            ui.painter().circle_filled(rect.center(), 8.0, color_32);
                            if is_selected {
                                ui.painter().circle_stroke(
                                    rect.center(),
                                    10.0,
                                    egui::Stroke::new(
                                        2.0_f32,
                                        if app.settings.dark_theme {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::BLACK
                                        },
                                    ),
                                );
                            }
                            if resp.on_hover_text(color_name).clicked() {
                                if is_selected {
                                    app.proj_color = None;
                                } else {
                                    app.proj_color = Some(hex.to_string());
                                }
                            }
                        }
                        if app.proj_color.is_some() && ui.small_button("Default").clicked() {
                            app.proj_color = None;
                        }
                    });
                    ui.end_row();

                    ui.label("Active");
                    ui.checkbox(&mut app.proj_active, "Active project");
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let label = if app.editing_project_id.is_some() {
                    "Update Project"
                } else {
                    "Create Project"
                };
                if ui.button(label).clicked() {
                    if app.proj_name.trim().is_empty() {
                        app.status = "Project name is required".to_string();
                        return;
                    }
                    let rate = if app.proj_rate.trim().is_empty() {
                        None
                    } else {
                        app.proj_rate.trim().parse::<f64>().ok()
                    };
                    let client = if app.proj_client.trim().is_empty() {
                        None
                    } else {
                        Some(app.proj_client.trim().to_string())
                    };
                    let color = app.proj_color.clone();

                    if let Some(id) = app.editing_project_id {
                        let p = Project {
                            id,
                            name: app.proj_name.trim().to_string(),
                            client,
                            rate,
                            color,
                            active: app.proj_active,
                        };
                        match app.db.update_project(&p) {
                            Ok(_) => {
                                app.status = "Project updated".to_string();
                                app.clear_project_form();
                                app.refresh_projects();
                            }
                            Err(e) => app.status = format!("Update failed: {e}"),
                        }
                    } else {
                        let p = NewProject {
                            name: app.proj_name.trim().to_string(),
                            client,
                            rate,
                            color,
                            active: app.proj_active,
                        };
                        match app.db.create_project(&p) {
                            Ok(_) => {
                                app.status = "Project created".to_string();
                                app.clear_project_form();
                                app.refresh_projects();
                            }
                            Err(e) => app.status = format!("Create failed: {e}"),
                        }
                    }
                }
                if app.editing_project_id.is_some() && ui.button("Cancel edit").clicked() {
                    app.clear_project_form();
                }
            });
        },
    );

    ui.add_space(8.0);
    ui.separator();

    // Compute statistics per project
    let mut project_minutes: HashMap<i64, (i64, usize)> = HashMap::new();
    if let Ok(entries) = app.db.list_entries(&EntryFilter::default()) {
        for e in entries {
            if let Some(pid) = e.project_id {
                let stat = project_minutes.entry(pid).or_insert((0, 0));
                stat.0 += e.duration_minutes;
                stat.1 += 1;
            }
        }
    }

    let total_projects = app.projects_cache.len();
    let archived_count = app.projects_cache.iter().filter(|p| !p.active).count();

    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.add(
            egui::TextEdit::singleline(&mut app.proj_search)
                .hint_text("Filter by name or client...")
                .desired_width(180.0),
        );
        if !app.proj_search.is_empty() && ui.small_button("✕").clicked() {
            app.proj_search.clear();
        }
        ui.separator();
        ui.checkbox(&mut app.proj_active_only, "Active only");
        ui.separator();
        if ui.button("↻ Refresh").clicked() {
            app.refresh_projects();
        }
        ui.weak(format!(
            "{} projects ({} archived)",
            total_projects, archived_count
        ));
    });

    let search_lower = app.proj_search.trim().to_lowercase();
    let filtered_projects: Vec<(usize, Project)> = app
        .projects_cache
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            if app.proj_active_only && !p.active {
                return false;
            }
            if !search_lower.is_empty() {
                let matches_name = p.name.to_lowercase().contains(&search_lower);
                let matches_client = p
                    .client
                    .as_deref()
                    .map(|c| c.to_lowercase().contains(&search_lower))
                    .unwrap_or(false);
                if !matches_name && !matches_client {
                    return false;
                }
            }
            true
        })
        .map(|(idx, p)| (idx, p.clone()))
        .collect();

    if filtered_projects.is_empty() {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("No projects match the current filter.").italics());
    } else {
        let mut toggle_active_project: Option<Project> = None;

        egui::Grid::new("projects_grid")
            .striped(true)
            .min_col_width(50.0)
            .show(ui, |ui| {
                ui.strong("Project Name");
                ui.strong("Client");
                ui.strong("Rate");
                ui.strong("Total Logged");
                ui.strong("Entries");
                ui.strong("Status");
                ui.strong("Actions");
                ui.end_row();

                for (idx, p) in &filtered_projects {
                    let dot_color = crate::ui::project_color_32(p.color.as_deref(), *idx);

                    ui.horizontal(|ui| {
                        crate::ui::render_color_dot(ui, dot_color);
                        if p.active {
                            ui.label(&p.name);
                        } else {
                            ui.label(egui::RichText::new(&p.name).weak());
                        }
                    });

                    ui.label(p.client.as_deref().unwrap_or("-"));
                    ui.label(
                        p.rate
                            .map(|r| format!("${r:.2}/h"))
                            .unwrap_or_else(|| "-".into()),
                    );

                    let (mins, count) = project_minutes.get(&p.id).copied().unwrap_or((0, 0));
                    ui.label(crate::domain::format_minutes(mins));
                    ui.label(count.to_string());

                    if p.active {
                        ui.label(
                            egui::RichText::new("● Active")
                                .color(egui::Color32::from_rgb(16, 185, 129)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("○ Archived")
                                .color(egui::Color32::from_rgb(148, 163, 184)),
                        );
                    }

                    ui.horizontal(|ui| {
                        if ui.small_button("Edit").clicked() {
                            app.editing_project_id = Some(p.id);
                            app.proj_name = p.name.clone();
                            app.proj_client = p.client.clone().unwrap_or_default();
                            app.proj_rate = p.rate.map(|r| r.to_string()).unwrap_or_default();
                            app.proj_color = p.color.clone();
                            app.proj_active = p.active;
                        }

                        let toggle_label = if p.active { "Archive" } else { "Activate" };
                        let toggle_tip = if p.active {
                            "Archive project (hide from active picks)"
                        } else {
                            "Reactivate project"
                        };
                        if ui
                            .small_button(toggle_label)
                            .on_hover_text(toggle_tip)
                            .clicked()
                        {
                            let mut updated = p.clone();
                            updated.active = !p.active;
                            toggle_active_project = Some(updated);
                        }

                        if ui.small_button("Delete").clicked() {
                            app.confirm = Some(crate::ui::app::ConfirmState {
                                title: "Delete Project".to_string(),
                                message: format!(
                                    "Delete project \"{}\"? Entries will keep their data but lose the project link.",
                                    p.name
                                ),
                                action: crate::ui::app::ConfirmAction::DeleteProject(p.id),
                                backup_path: None,
                            });
                        }
                    });
                    ui.end_row();
                }
            });

        if let Some(updated) = toggle_active_project {
            let act_name = if updated.active {
                "activated"
            } else {
                "archived"
            };
            match app.db.update_project(&updated) {
                Ok(_) => {
                    app.status = format!("Project \"{}\" {}", updated.name, act_name);
                    app.refresh_projects();
                }
                Err(e) => app.status = format!("Failed to update project status: {e}"),
            }
        }
    }
}
