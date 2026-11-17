use crate::db::models::{EntryFilter, NewProject, Project};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::ui::centered_content(ui, |ui| {
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

                let avail_w = ui.available_width();
                if avail_w > 880.0 {
                    // Widescreen 2-column split (AppFlowy style):
                    // Left = Directory & List (fills available space ~62%)
                    // Right = New / Edit Project Form Card (~38%)
                    let right_w = 380.0_f32.min(avail_w * 0.40);
                    let left_w = (avail_w - right_w - 20.0).max(450.0);

                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(left_w, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_project_directory(app, ui, &project_minutes);
                            },
                        );

                        ui.add_space(12.0);

                        ui.allocate_ui_with_layout(
                            egui::vec2(right_w, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                render_project_form(app, ui);
                            },
                        );
                    });
                } else {
                    // Compact / narrow single-column layout
                    render_project_form(app, ui);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    render_project_directory(app, ui, &project_minutes);
                }

                ui.add_space(20.0);
            });
        });
}

fn render_project_form(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    let form_title = if app.editing_project_id.is_some() {
        format!("Edit Project #{}", app.editing_project_id.unwrap())
    } else {
        "New Project Form".to_string()
    };
    let form_icon = if app.editing_project_id.is_some() {
        "✏"
    } else {
        "📁"
    };

    crate::ui::render_form_card(ui, app.tokens(), &form_title, form_icon, |ui| {
        egui::Grid::new("proj_form")
            .num_columns(2)
            .spacing([16.0, 8.0])
            .show(ui, |ui| {
                ui.strong("Project Name *");
                let field_w = (ui.available_width() - 24.0).max(200.0);
                crate::ui::material::input(
                    ui,
                    &mut app.proj_name,
                    "e.g. Mobile App Redesign, Internal Tools...",
                    field_w,
                );
                ui.end_row();

                ui.strong("Client");
                let field_w = (ui.available_width() - 24.0).max(200.0);
                crate::ui::material::input(
                    ui,
                    &mut app.proj_client,
                    "e.g. Acme Corp, Internal",
                    field_w,
                );
                ui.end_row();

                ui.strong("Hourly rate ($/h)");
                crate::ui::material::input(ui, &mut app.proj_rate, "e.g. 75.00", 120.0);
                ui.end_row();

                ui.strong("Budget hours");
                ui.horizontal_wrapped(|ui| {
                    crate::ui::material::input(ui, &mut app.proj_budget, "e.g. 40.0", 70.0);
                    for preset in ["20h", "40h", "80h", "160h"] {
                        if crate::ui::material::chip(ui, preset).clicked() {
                            app.proj_budget = preset.trim_end_matches('h').to_string() + ".0";
                        }
                    }
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
                                egui::Stroke::new(2.0_f32, app.tokens().on_surface),
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
                ui.horizontal(|ui| {
                    crate::ui::material::switch(ui, &mut app.proj_active);
                    ui.label("Active (shown in pickers)");
                });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let label = if app.editing_project_id.is_some() {
                "💾 Update Project"
            } else {
                "➕ Create Project"
            };
            let clicked = if app.editing_project_id.is_some() {
                crate::ui::material::tonal_button(ui, label).clicked()
            } else {
                crate::ui::material::primary_button(ui, label).clicked()
            };
            if clicked {
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
            if app.editing_project_id.is_some()
                && crate::ui::material::text_button(ui, "✖ Cancel edit").clicked()
            {
                app.clear_project_form();
            }
        });
    });
}

fn render_project_directory(
    app: &mut TimesheetApp,
    ui: &mut egui::Ui,
    project_minutes: &HashMap<i64, (i64, usize)>,
) {
    if app.projects_cache.is_empty() {
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
                            "Fill out the New Project form to create your first project!",
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

    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut app.proj_search)
                    .hint_text("Search name or client...")
                    .desired_width(180.0),
            );
            if !app.proj_search.is_empty() && ui.small_button("✕").clicked() {
                app.proj_search.clear();
            }
            ui.separator();
            ui.checkbox(&mut app.proj_active_only, "Active only");
            ui.separator();
            if ui.small_button("↻ Refresh").clicked() {
                app.refresh_projects();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(format!(
                    "{} projects ({} archived)",
                    total_projects, archived_count
                ));
            });
        });
    });

    ui.add_space(6.0);

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

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("projects_grid")
                    .striped(true)
                    .min_col_width(45.0)
                    .spacing([14.0, 7.0])
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
                            let tokens = app.tokens();
                            if let Some(bh) = p.budget_hours {
                                let spent_h = mins as f64 / 60.0;
                                let pct = if bh > 0.0 { (spent_h / bh) * 100.0 } else { 0.0 };
                                let (stat_color, stat_label) = if pct > 100.0 {
                                    (
                                        tokens.error,
                                        format!("{:.1}/{:.0}h (OVER {:.0}%)", spent_h, bh, pct),
                                    )
                                } else if pct >= 80.0 {
                                    (
                                        tokens.warning,
                                        format!("{:.1}/{:.0}h ({:.0}%)", spent_h, bh, pct),
                                    )
                                } else {
                                    (
                                        tokens.tertiary,
                                        format!("{:.1}/{:.0}h ({:.0}%)", spent_h, bh, pct),
                                    )
                                };
                                ui.label(egui::RichText::new(stat_label).color(stat_color));
                            } else {
                                ui.label(egui::RichText::new("-").weak());
                            }

                            ui.label(count.to_string());

                            if p.active {
                                ui.label(
                                    egui::RichText::new("● Active").color(tokens.tertiary),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("○ Archived")
                                        .color(tokens.on_surface_variant),
                                );
                            }

                            ui.horizontal(|ui| {
                                if ui.small_button("✏ Edit").clicked() {
                                    app.editing_project_id = Some(p.id);
                                    app.proj_name = p.name.clone();
                                    app.proj_client = p.client.clone().unwrap_or_default();
                                    app.proj_rate =
                                        p.rate.map(|r| r.to_string()).unwrap_or_default();
                                    app.proj_budget =
                                        p.budget_hours.map(|b| b.to_string()).unwrap_or_default();
                                    app.proj_color = p.color.clone();
                                    app.proj_active = p.active;
                                }

                                let toggle_label = if p.active { "Archive" } else { "Activate" };
                                let toggle_tip = if p.active {
                                    "Archive project (hide from active pickers)"
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

                                if ui.small_button("🗑 Delete").clicked() {
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
