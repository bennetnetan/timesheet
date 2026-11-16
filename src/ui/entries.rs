use crate::db::models::{Entry, EntryFilter, NewEntry};
use crate::domain::parse_duration_to_minutes;
use crate::ui::app::TimesheetApp;
use chrono::NaiveDate;
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Entries");

    // Filters
    ui.horizontal(|ui| {
        ui.label("From (YYYY-MM-DD):");
        let mut from = app.filter.from.map(|d| d.to_string()).unwrap_or_default();
        if ui.text_edit_singleline(&mut from).changed() {
            app.filter.from = if from.trim().is_empty() {
                None
            } else {
                NaiveDate::parse_from_str(from.trim(), "%Y-%m-%d").ok()
            };
        }
        ui.label("To:");
        let mut to = app.filter.to.map(|d| d.to_string()).unwrap_or_default();
        if ui.text_edit_singleline(&mut to).changed() {
            app.filter.to = if to.trim().is_empty() {
                None
            } else {
                NaiveDate::parse_from_str(to.trim(), "%Y-%m-%d").ok()
            };
        }
        if ui.button("Refresh").clicked() {
            app.status = "Entries refreshed".to_string();
        }
        if ui.button("Clear filters").clicked() {
            app.filter = EntryFilter::default();
        }
    });

    ui.separator();

    // Edit/create form
    ui.collapsing(
        if app.editing_entry_id.is_some() {
            "Edit Entry"
        } else {
            "New Entry"
        },
        |ui| {
            entry_form(app, ui);
        },
    );

    ui.separator();

    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    let total: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    ui.label(format!(
        "{} entries, total {}",
        entries.len(),
        crate::domain::format_minutes(total)
    ));

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("entries_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Date");
                ui.strong("Project");
                ui.strong("Duration");
                ui.strong("Billable");
                ui.strong("Description");
                ui.strong("Tags");
                ui.strong("Actions");
                ui.end_row();

                for e in entries {
                    ui.label(e.date.to_string());
                    let proj = e
                        .project_id
                        .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                        .map(|p| p.name.as_str())
                        .unwrap_or("-");
                    ui.label(proj);
                    ui.label(crate::domain::format_minutes(e.duration_minutes));
                    ui.label(if e.billable { "Yes" } else { "No" });
                    ui.label(&e.description);
                    ui.label(&e.tags);
                    ui.horizontal(|ui| {
                        if ui.small_button("Edit").clicked() {
                            load_entry_into_form(app, &e);
                        }
                        if ui.small_button("Delete").clicked() {
                            if let Err(err) = app.db.delete_entry(e.id) {
                                app.status = format!("Delete failed: {err}");
                            } else {
                                app.status = "Entry deleted".to_string();
                            }
                        }
                    });
                    ui.end_row();
                }
            });
    });
}

fn load_entry_into_form(app: &mut TimesheetApp, e: &Entry) {
    app.editing_entry_id = Some(e.id);
    app.entry_date = e.date;
    app.entry_project_idx = e
        .project_id
        .and_then(|pid| app.projects_cache.iter().position(|p| p.id == pid));
    app.entry_start = e.start_time.clone().unwrap_or_default();
    app.entry_end = e.end_time.clone().unwrap_or_default();
    app.entry_duration = crate::domain::format_minutes(e.duration_minutes);
    app.entry_description = e.description.clone();
    app.entry_billable = e.billable;
    app.entry_tags = e.tags.clone();
}

fn entry_form(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::Grid::new("entry_form")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label("Date (YYYY-MM-DD)");
            let mut date_str = app.entry_date.format("%Y-%m-%d").to_string();
            if ui.text_edit_singleline(&mut date_str).changed() {
                if let Ok(d) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    app.entry_date = d;
                }
            }
            ui.end_row();

            ui.label("Project");
            egui::ComboBox::from_id_salt("entry_project")
                .selected_text(
                    app.entry_project_idx
                        .and_then(|i| app.projects_cache.get(i))
                        .map(|p| p.name.as_str())
                        .unwrap_or("(none)"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.entry_project_idx.is_none(), "(none)")
                        .clicked()
                    {
                        app.entry_project_idx = None;
                    }
                    for (i, p) in app.projects_cache.iter().enumerate() {
                        if ui
                            .selectable_label(app.entry_project_idx == Some(i), &p.name)
                            .clicked()
                        {
                            app.entry_project_idx = Some(i);
                        }
                    }
                });
            ui.end_row();

            ui.label("Start (HH:MM)");
            ui.text_edit_singleline(&mut app.entry_start);
            ui.end_row();

            ui.label("End (HH:MM)");
            ui.text_edit_singleline(&mut app.entry_end);
            ui.end_row();

            ui.label("Duration");
            ui.text_edit_singleline(&mut app.entry_duration);
            ui.end_row();

            ui.label("Description");
            ui.text_edit_singleline(&mut app.entry_description);
            ui.end_row();

            ui.label("Billable");
            ui.checkbox(&mut app.entry_billable, "Billable");
            ui.end_row();

            ui.label("Tags");
            ui.text_edit_singleline(&mut app.entry_tags);
            ui.end_row();
        });

    ui.horizontal(|ui| {
        let label = if app.editing_entry_id.is_some() {
            "Update"
        } else {
            "Create"
        };
        if ui.button(label).clicked() {
            if let Some(mins) = parse_duration_to_minutes(&app.entry_duration) {
                if mins <= 0 {
                    app.status = "Duration must be > 0".to_string();
                    return;
                }
                if let Some(id) = app.editing_entry_id {
                    let e = Entry {
                        id,
                        project_id: app.selected_project_id(),
                        date: app.entry_date,
                        start_time: if app.entry_start.is_empty() {
                            None
                        } else {
                            Some(app.entry_start.clone())
                        },
                        end_time: if app.entry_end.is_empty() {
                            None
                        } else {
                            Some(app.entry_end.clone())
                        },
                        duration_minutes: mins,
                        description: app.entry_description.clone(),
                        billable: app.entry_billable,
                        tags: app.entry_tags.clone(),
                    };
                    match app.db.update_entry(&e) {
                        Ok(_) => {
                            app.status = "Entry updated".to_string();
                            app.clear_entry_form();
                        }
                        Err(err) => app.status = format!("Update failed: {err}"),
                    }
                } else {
                    let e = NewEntry {
                        project_id: app.selected_project_id(),
                        date: app.entry_date,
                        start_time: if app.entry_start.is_empty() {
                            None
                        } else {
                            Some(app.entry_start.clone())
                        },
                        end_time: if app.entry_end.is_empty() {
                            None
                        } else {
                            Some(app.entry_end.clone())
                        },
                        duration_minutes: mins,
                        description: app.entry_description.clone(),
                        billable: app.entry_billable,
                        tags: app.entry_tags.clone(),
                    };
                    match app.db.create_entry(&e) {
                        Ok(_) => {
                            app.status = "Entry created".to_string();
                            app.clear_entry_form();
                        }
                        Err(err) => app.status = format!("Create failed: {err}"),
                    }
                }
            } else {
                app.status = "Invalid duration".to_string();
            }
        }
        if app.editing_entry_id.is_some() && ui.button("Cancel edit").clicked() {
            app.clear_entry_form();
        }
    });
}
