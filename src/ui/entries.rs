use crate::db::models::{Entry, NewEntry};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use egui_extras::DatePickerButton;
use std::collections::HashSet;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    // Keyboard shortcuts (Entries tab)
    let (ctrl_n, esc, del, enter) = ui.input(|i| {
        (
            i.modifiers.command && i.key_pressed(egui::Key::N),
            i.key_pressed(egui::Key::Escape),
            i.key_pressed(egui::Key::Delete),
            i.key_pressed(egui::Key::Enter),
        )
    });
    if ctrl_n {
        app.clear_entry_form();
        app.focus_new_entry = true;
        app.status = "New entry form ready (Ctrl+N)".to_string();
    }
    if esc {
        if app.editing_entry_id.is_some() {
            app.clear_entry_form();
            app.status = "Edit cancelled".to_string();
        }
        app.selected_entry_ids.clear();
    }

    ui.heading("Entries");
    ui.label("Shortcuts: Ctrl+N new entry, Enter save, Esc cancel/clear selection, Delete delete selected");

    // Filters
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("From:");
            let mut from = app
                .filter
                .from
                .unwrap_or_else(|| chrono::Local::now().date_naive());
            let from_enabled = app.filter.from.is_some();
            let mut use_from = from_enabled;
            if ui.checkbox(&mut use_from, "use").changed() {
                app.filter.from = if use_from { Some(from) } else { None };
            }
            if app.filter.from.is_some() {
                if ui.add(DatePickerButton::new(&mut from)).changed() {
                    app.filter.from = Some(from);
                }
            }

            ui.separator();
            ui.label("To:");
            let mut to = app
                .filter
                .to
                .unwrap_or_else(|| chrono::Local::now().date_naive());
            let mut use_to = app.filter.to.is_some();
            if ui.checkbox(&mut use_to, "use").changed() {
                app.filter.to = if use_to { Some(to) } else { None };
            }
            if app.filter.to.is_some() {
                if ui.add(DatePickerButton::new(&mut to)).changed() {
                    app.filter.to = Some(to);
                }
            }

            ui.separator();
            ui.label("Project:");
            egui::ComboBox::from_id_salt("filter_project")
                .selected_text(
                    app.filter
                        .project_id
                        .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                        .map(|p| p.name.as_str())
                        .unwrap_or("All"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.filter.project_id.is_none(), "All")
                        .clicked()
                    {
                        app.filter.project_id = None;
                    }
                    for p in &app.projects_cache {
                        if ui
                            .selectable_label(app.filter.project_id == Some(p.id), &p.name)
                            .clicked()
                        {
                            app.filter.project_id = Some(p.id);
                        }
                    }
                });

            ui.label("Billable:");
            let billable_text = match app.filter.billable {
                None => "All",
                Some(true) => "Billable",
                Some(false) => "Non-billable",
            };
            egui::ComboBox::from_id_salt("filter_billable")
                .selected_text(billable_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.filter.billable.is_none(), "All")
                        .clicked()
                    {
                        app.filter.billable = None;
                    }
                    if ui
                        .selectable_label(app.filter.billable == Some(true), "Billable")
                        .clicked()
                    {
                        app.filter.billable = Some(true);
                    }
                    if ui
                        .selectable_label(app.filter.billable == Some(false), "Non-billable")
                        .clicked()
                    {
                        app.filter.billable = Some(false);
                    }
                });

            if ui.button("Refresh").clicked() {
                app.status = "Entries refreshed".to_string();
            }
            if ui.button("Clear filters").clicked() {
                app.filter = crate::db::models::EntryFilter::default();
            }
        });
    });

    ui.separator();

    let mut save_requested = enter;
    ui.collapsing(
        if app.editing_entry_id.is_some() {
            "Edit Entry"
        } else {
            "New Entry"
        },
        |ui| {
            if entry_form(app, ui) {
                save_requested = true;
            }
        },
    );
    if save_requested {
        save_entry(app);
    }

    ui.separator();

    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    let valid_ids: HashSet<i64> = entries.iter().map(|e| e.id).collect();
    app.selected_entry_ids.retain(|id| valid_ids.contains(id));

    if del && !app.selected_entry_ids.is_empty() {
        let ids: Vec<i64> = app.selected_entry_ids.iter().copied().collect();
        let mut deleted = 0;
        for id in ids {
            if app.db.delete_entry(id).is_ok() {
                deleted += 1;
            }
        }
        app.selected_entry_ids.clear();
        app.status = format!(
            "Deleted {} selected entr{}",
            deleted,
            if deleted == 1 { "y" } else { "ies" }
        );
    }

    let total: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    ui.horizontal(|ui| {
        ui.label(format!(
            "{} entries, total {}",
            entries.len(),
            crate::domain::format_minutes(total)
        ));
        if !app.selected_entry_ids.is_empty() {
            ui.separator();
            ui.label(format!("{} selected", app.selected_entry_ids.len()));
        }
    });

    // Bulk actions
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Bulk actions on selected:");
            if ui.button("Select all").clicked() {
                app.selected_entry_ids = entries.iter().map(|e| e.id).collect();
            }
            if ui.button("Select none").clicked() {
                app.selected_entry_ids.clear();
            }
            if ui.button("Delete selected").clicked() && !app.selected_entry_ids.is_empty() {
                let ids: Vec<i64> = app.selected_entry_ids.iter().copied().collect();
                let mut deleted = 0;
                for id in ids {
                    if app.db.delete_entry(id).is_ok() {
                        deleted += 1;
                    }
                }
                app.selected_entry_ids.clear();
                app.status = format!(
                    "Deleted {} selected entr{}",
                    deleted,
                    if deleted == 1 { "y" } else { "ies" }
                );
            }
            ui.separator();
            ui.checkbox(&mut app.bulk_billable, "Billable");
            if ui.button("Apply billable").clicked() && !app.selected_entry_ids.is_empty() {
                bulk_update_billable(app, &entries);
            }
            ui.label("Tags:");
            ui.text_edit_singleline(&mut app.bulk_tags);
            if ui.button("Apply tags").clicked() && !app.selected_entry_ids.is_empty() {
                bulk_update_tags(app, &entries);
            }
        });
    });

    // Table
    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("entries_grid")
                .striped(true)
                .min_col_width(40.0)
                .show(ui, |ui| {
                    ui.strong("Sel");
                    ui.strong("Date");
                    ui.strong("Project");
                    ui.strong("Start");
                    ui.strong("End");
                    ui.strong("Duration");
                    ui.strong("Billable");
                    ui.strong("Tags");
                    ui.strong("Description");
                    ui.strong("Actions");
                    ui.end_row();

                    for e in entries {
                        let mut selected = app.selected_entry_ids.contains(&e.id);
                        if ui.checkbox(&mut selected, "").changed() {
                            if selected {
                                app.selected_entry_ids.insert(e.id);
                            } else {
                                app.selected_entry_ids.remove(&e.id);
                            }
                        }
                        ui.label(e.date.to_string());
                        let proj = e
                            .project_id
                            .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                            .map(|p| p.name.as_str())
                            .unwrap_or("-");
                        ui.label(proj);
                        ui.label(e.start_time.as_deref().unwrap_or("-"));
                        ui.label(e.end_time.as_deref().unwrap_or("-"));
                        ui.label(crate::domain::format_minutes(e.duration_minutes));
                        ui.label(if e.billable { "Yes" } else { "No" });
                        ui.label(&e.tags);
                        ui.label(&e.description);
                        ui.horizontal(|ui| {
                            if ui.small_button("Edit").clicked() {
                                load_entry_into_form(app, &e);
                            }
                            if ui.small_button("Delete").clicked() {
                                if let Err(err) = app.db.delete_entry(e.id) {
                                    app.status = format!("Delete failed: {err}");
                                } else {
                                    app.selected_entry_ids.remove(&e.id);
                                    app.status = "Entry deleted".to_string();
                                }
                            }
                        });
                        ui.end_row();
                    }
                });
        });
}

fn bulk_update_billable(app: &mut TimesheetApp, entries: &[Entry]) {
    let selected = app.selected_entry_ids.clone();
    let mut updated = 0;
    for e in entries.iter().filter(|e| selected.contains(&e.id)) {
        let mut ne = e.clone();
        ne.billable = app.bulk_billable;
        if app.db.update_entry(&ne).is_ok() {
            updated += 1;
        }
    }
    app.status = format!(
        "Updated billable on {} entr{}",
        updated,
        if updated == 1 { "y" } else { "ies" }
    );
}

fn bulk_update_tags(app: &mut TimesheetApp, entries: &[Entry]) {
    let selected = app.selected_entry_ids.clone();
    let mut updated = 0;
    for e in entries.iter().filter(|e| selected.contains(&e.id)) {
        let mut ne = e.clone();
        ne.tags = app.bulk_tags.clone();
        if app.db.update_entry(&ne).is_ok() {
            updated += 1;
        }
    }
    app.status = format!(
        "Updated tags on {} entr{}",
        updated,
        if updated == 1 { "y" } else { "ies" }
    );
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
    app.status = format!("Editing entry #{}", e.id);
}

fn entry_form(app: &mut TimesheetApp, ui: &mut egui::Ui) -> bool {
    let mut save_clicked = false;
    let desc_id = egui::Id::new("entry_desc_field");

    egui::Grid::new("entry_form")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label("Date");
            ui.add(DatePickerButton::new(&mut app.entry_date));
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
                        let label = if p.active {
                            p.name.clone()
                        } else {
                            format!("{} (inactive)", p.name)
                        };
                        if ui
                            .selectable_label(app.entry_project_idx == Some(i), label)
                            .clicked()
                        {
                            app.entry_project_idx = Some(i);
                        }
                    }
                });
            ui.end_row();

            ui.label("Start (HH:MM)");
            let start_resp = ui.text_edit_singleline(&mut app.entry_start);
            if start_resp.changed() {
                if let Some(mins) =
                    crate::domain::duration_minutes_from_times(&app.entry_start, &app.entry_end)
                {
                    app.entry_duration = crate::domain::format_minutes(mins);
                }
            }
            ui.end_row();

            ui.label("End (HH:MM)");
            ui.horizontal(|ui| {
                let end_resp = ui.text_edit_singleline(&mut app.entry_end);
                if end_resp.changed() {
                    if let Some(mins) =
                        crate::domain::duration_minutes_from_times(&app.entry_start, &app.entry_end)
                    {
                        app.entry_duration = crate::domain::format_minutes(mins);
                    }
                }
                if ui.small_button("Now").clicked() {
                    app.entry_end = crate::domain::now_hhmm();
                    if let Some(mins) =
                        crate::domain::duration_minutes_from_times(&app.entry_start, &app.entry_end)
                    {
                        app.entry_duration = crate::domain::format_minutes(mins);
                    }
                }
            });
            ui.end_row();

            ui.label("Duration");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.entry_duration);
                ui.label("(auto-fills from start/end)");
            });
            ui.end_row();

            ui.label("Description");
            let desc_resp =
                ui.add(egui::TextEdit::singleline(&mut app.entry_description).id(desc_id));
            if app.focus_new_entry {
                desc_resp.request_focus();
                app.focus_new_entry = false;
            }
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
            "Update (Enter)"
        } else {
            "Create (Enter)"
        };
        if ui.button(label).clicked() {
            save_clicked = true;
        }
        if ui.button("New / Clear (Ctrl+N)").clicked() {
            app.clear_entry_form();
            app.focus_new_entry = true;
        }
        if app.editing_entry_id.is_some() && ui.button("Cancel edit (Esc)").clicked() {
            app.clear_entry_form();
        }
    });
    save_clicked
}

fn save_entry(app: &mut TimesheetApp) {
    let Some(mins) = app.effective_entry_duration_minutes() else {
        app.status = "Invalid duration (enter duration or valid start/end times)".to_string();
        return;
    };
    if mins <= 0 {
        app.status = "Duration must be > 0".to_string();
        return;
    }
    let start_time = if app.entry_start.trim().is_empty() {
        None
    } else {
        Some(app.entry_start.trim().to_string())
    };
    let end_time = if app.entry_end.trim().is_empty() {
        None
    } else {
        Some(app.entry_end.trim().to_string())
    };
    if let Some(id) = app.editing_entry_id {
        let e = Entry {
            id,
            project_id: app.selected_project_id(),
            date: app.entry_date,
            start_time,
            end_time,
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
            start_time,
            end_time,
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
}
