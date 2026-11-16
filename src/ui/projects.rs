use crate::db::models::{EntryFilter, NewProject, Project};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading("Projects");
            ui.label("Organize projects, assign visual color codes, and view tracked time per project.");
            ui.add_space(4.0);

    let form_title = if app.editing_project_id.is_some() {
        format!("Edit Project #{}", app.editing_project_id.unwrap())
    } else {
        "New Project Form".to_string()
    };

    crate::ui::render_form_card(ui, app.settings.dark_theme, &form_title, "📁", |ui| {
        egui::Grid::new("proj_form")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.strong("Project Name *");
                ui.add(
                    egui::TextEdit::singleline(&mut app.proj_name)
                        .hint_text("e.g. Mobile App Redesign, Internal Tools...")
                        .desired_width(280.0),
                );
                ui.end_row();

                ui.strong("Client");
                ui.add(
                    egui::TextEdit::singleline(&mut app.proj_client)
                        .hint_text("e.g. Acme Corp, Internal")
                        .desired_width(280.0),
                );
                ui.end_row();

                ui.strong("Hourly rate ($/h)");
                ui.add(
                    egui::TextEdit::singleline(&mut app.proj_rate)
                        .hint_text("e.g. 75.00")
                        .desired_width(120.0),
                );
                ui.end_row();

                ui.strong("Budget hours");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.proj_budget)
                            .hint_text("e.g. 40.0")
                            .desired_width(80.0),
                    );
                    if ui.small_button("20h").clicked() {
                        app.proj_budget = "20.0".to_string();
                    }
                    if ui.small_button("40h").clicked() {
                        app.proj_budget = "40.0".to_string();
                    }
                    if ui.small_button("80h").clicked() {
                        app.proj_budget = "80.0".to_string();
                    }
                    if ui.small_button("160h").clicked() {
                        app.proj_budget = "160.0".to_string();
                    }
                    ui.weak("(optional target for burn-down tracking)");
                });
                ui.end_row();

                ui.strong("Color badge");
                ui.horizontal_wrapped(|ui| {
                    for &(color_name, hex) in crate::domain::PROJECT_PALETTE {
                        let (r, g, b) = crate::domain::parse_hex_color(hex).unwrap();
                        let color_32 = egui::Color32::from_rgb(r, g, b);
                        let is_selected = app.proj_color.as_deref() == Some(hex);

                        let (rect, resp) =
                            ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
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

                ui.strong("Status");
                ui.checkbox(&mut app.proj_active, "Active (shown in entry pickers)");
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let label = if app.editing_project_id.is_some() {
                "💾 Update Project"
            } else {
                "➕ Create Project"
            };
            if ui.button(egui::RichText::new(label).strong()).clicked() {
                if app.proj_name.trim().is_empty() {
                    app.status = "Project name is required".to_string();
                    return;
                }
                let rate = if app.proj_rate.trim().is_empty() {
                    None
                } else {
                    app.proj_rate.trim().parse::<f64>().ok()
                };
                let budget_hours = if app.proj_budget.trim().is_empty() {
                    None
                } else {
                    app.proj_budget.trim().parse::<f64>().ok()
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
                        budget_hours,
                    };
                    match app.db.update_project(&p) {
                        Ok(_) => {
                            app.status = "Project updated successfully".to_string();
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
                        budget_hours,
                    };
                    match app.db.create_project(&p) {
                        Ok(_) => {
                            app.status = "Project created successfully".to_string();
                            app.clear_project_form();
                            app.refresh_projects();
                        }
                        Err(e) => app.status = format!("Create failed: {e}"),
                    }
                }
            }
            if app.editing_project_id.is_some() && ui.button("✖ Cancel edit").clicked() {
                app.clear_project_form();
            }
        });
    });

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

    if app.projects_cache.is_empty() {
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📁").size(24.0));
                ui.vertical(|ui| {
                    ui.strong("No projects yet.");
                    ui.label(
                        egui::RichText::new(
                            "Projects help you organize entries by client, calculate hourly rates, and color-code work.",
                        )
                        .weak(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Fill out the New Project form above to create your first project!",
                        )
                        .italics(),
                    );
                });
            });
        });
        return;
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
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(20.0));
                ui.vertical(|ui| {
                    ui.strong("No projects match your filter.");
                    ui.add_space(2.0);
                    if ui
                        .small_button("↺ Clear project search & filters")
                        .clicked()
                    {
                        app.proj_search.clear();
                        app.proj_active_only = false;
                    }
                });
            });
        });
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
                ui.strong("Budget Progress");
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

                    // Budget Progress display
                    if let Some(bh) = p.budget_hours {
                        let spent_h = mins as f64 / 60.0;
                        let pct = if bh > 0.0 { (spent_h / bh) * 100.0 } else { 0.0 };
                        let (stat_color, stat_label) = if pct > 100.0 {
                            (egui::Color32::from_rgb(239, 68, 68), format!("{:.1}/{:.0}h (OVER {:.0}%)", spent_h, bh, pct))
                        } else if pct >= 80.0 {
                            (egui::Color32::from_rgb(245, 158, 11), format!("{:.1}/{:.0}h ({:.0}%)", spent_h, bh, pct))
                        } else {
                            (egui::Color32::from_rgb(16, 185, 129), format!("{:.1}/{:.0}h ({:.0}%)", spent_h, bh, pct))
                        };
                        ui.label(egui::RichText::new(stat_label).color(stat_color));
                    } else {
                        ui.label(egui::RichText::new("-").weak());
                    }

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
                            app.proj_budget = p.budget_hours.map(|b| b.to_string()).unwrap_or_default();
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

            ui.add_space(16.0);
        });
}
