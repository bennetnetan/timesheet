use crate::db::models::{EntryFilter, NewEntry};
use crate::domain::parse_duration_to_minutes;
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Dashboard");
    ui.add_space(8.0);

    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

    let mut total_today = 0i64;
    let mut total_week = 0i64;
    let mut total_month = 0i64;
    if let Ok(entries) = app.db.list_entries(&EntryFilter::default()) {
        for e in &entries {
            if e.date == today {
                total_today += e.duration_minutes;
            }
            if e.date >= week_start && e.date <= today {
                total_week += e.duration_minutes;
            }
            if e.date >= month_start && e.date <= today {
                total_month += e.duration_minutes;
            }
        }
    }

    ui.horizontal(|ui| {
        ui.group(|ui| {
            ui.label(format!(
                "Today: {}",
                crate::domain::format_minutes(total_today)
            ))
        });
        ui.group(|ui| {
            ui.label(format!(
                "This week: {}",
                crate::domain::format_minutes(total_week)
            ))
        });
        ui.group(|ui| {
            ui.label(format!(
                "This month: {}",
                crate::domain::format_minutes(total_month)
            ))
        });
    });

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Quick Add Entry");

    egui::Grid::new("quick_add")
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
            egui::ComboBox::from_id_salt("dash_project")
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
                    for (i, p) in app
                        .projects_cache
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.active)
                    {
                        if ui
                            .selectable_label(app.entry_project_idx == Some(i), &p.name)
                            .clicked()
                        {
                            app.entry_project_idx = Some(i);
                        }
                    }
                });
            ui.end_row();

            ui.label("Duration (e.g. 1:30, 90, 1.5)");
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

    if ui.button("Add Entry").clicked() {
        match parse_duration_to_minutes(&app.entry_duration) {
            Some(mins) if mins > 0 => {
                let new_entry = NewEntry {
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
                match app.db.create_entry(&new_entry) {
                    Ok(_) => {
                        app.status = "Entry added".to_string();
                        app.clear_entry_form();
                    }
                    Err(e) => app.status = format!("Failed to add entry: {e}"),
                }
            }
            _ => app.status = "Please enter a valid duration (e.g. 1:30, 90, 1h30m)".to_string(),
        }
    }
}
