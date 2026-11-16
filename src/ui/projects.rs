use crate::db::models::{NewProject, Project};
use crate::ui::app::TimesheetApp;
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Projects");

    ui.collapsing(
        if app.editing_project_id.is_some() {
            "Edit Project"
        } else {
            "New Project"
        },
        |ui| {
            egui::Grid::new("proj_form").num_columns(2).show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut app.proj_name);
                ui.end_row();
                ui.label("Client");
                ui.text_edit_singleline(&mut app.proj_client);
                ui.end_row();
                ui.label("Rate");
                ui.text_edit_singleline(&mut app.proj_rate);
                ui.end_row();
                ui.label("Active");
                ui.checkbox(&mut app.proj_active, "Active");
                ui.end_row();
            });
            ui.horizontal(|ui| {
                let label = if app.editing_project_id.is_some() {
                    "Update"
                } else {
                    "Create"
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
                    if let Some(id) = app.editing_project_id {
                        let p = Project {
                            id,
                            name: app.proj_name.trim().to_string(),
                            client,
                            rate,
                            color: None,
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
                            color: None,
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

    ui.separator();
    if ui.button("Refresh list").clicked() {
        app.refresh_projects();
    }

    let projects = app.projects_cache.clone();
    egui::Grid::new("projects_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Name");
            ui.strong("Client");
            ui.strong("Rate");
            ui.strong("Active");
            ui.strong("Actions");
            ui.end_row();
            for p in projects {
                ui.label(&p.name);
                ui.label(p.client.as_deref().unwrap_or("-"));
                ui.label(
                    p.rate
                        .map(|r| format!("{r:.2}"))
                        .unwrap_or_else(|| "-".into()),
                );
                ui.label(if p.active { "Yes" } else { "No" });
                ui.horizontal(|ui| {
                    if ui.small_button("Edit").clicked() {
                        app.editing_project_id = Some(p.id);
                        app.proj_name = p.name.clone();
                        app.proj_client = p.client.clone().unwrap_or_default();
                        app.proj_rate = p.rate.map(|r| r.to_string()).unwrap_or_default();
                        app.proj_active = p.active;
                    }
                    if ui.small_button("Delete").clicked() {
                        app.confirm = Some(crate::ui::app::ConfirmState {
                            title: "Delete Project".to_string(),
                            message: format!("Delete project \"{}\"? Entries will keep their data but lose the project link.", p.name),
                            action: crate::ui::app::ConfirmAction::DeleteProject(p.id),
                            backup_path: None,
                        });
                    }
                });
                ui.end_row();
            }
        });
}
