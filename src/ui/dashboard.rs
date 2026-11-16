use crate::db::models::{EntryFilter, NewEntry};
use crate::ui::app::{TimerState, TimesheetApp};
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use egui_extras::DatePickerButton;
use std::time::Instant;

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
    timer_section(app, ui);

    ui.add_space(12.0);
    ui.separator();
    ui.heading("Quick Add Entry");
    ui.label("Tip: enter start/end to auto-calculate duration, or enter duration directly. Ctrl+N in Entries tab for quick new entry workflow.");

    egui::Grid::new("quick_add")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label("Date");
            ui.add(DatePickerButton::new(&mut app.entry_date));
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

            ui.label("Start (HH:MM)");
            let s = ui.text_edit_singleline(&mut app.entry_start);
            if s.changed() {
                if let Some(mins) =
                    crate::domain::duration_minutes_from_times(&app.entry_start, &app.entry_end)
                {
                    app.entry_duration = crate::domain::format_minutes(mins);
                }
            }
            ui.end_row();

            ui.label("End (HH:MM)");
            ui.horizontal(|ui| {
                let e = ui.text_edit_singleline(&mut app.entry_end);
                if e.changed() {
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
        match app.effective_entry_duration_minutes() {
            Some(mins) if mins > 0 => {
                let new_entry = NewEntry {
                    project_id: app.selected_project_id(),
                    date: app.entry_date,
                    start_time: if app.entry_start.trim().is_empty() {
                        None
                    } else {
                        Some(app.entry_start.trim().to_string())
                    },
                    end_time: if app.entry_end.trim().is_empty() {
                        None
                    } else {
                        Some(app.entry_end.trim().to_string())
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
            _ => app.status = "Enter a valid duration or valid start/end times".to_string(),
        }
    }
}

fn timer_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Timer");
    if let Some(timer) = &app.timer {
        let elapsed = timer.running_since.elapsed();
        let mins = (elapsed.as_secs() / 60) as i64;
        let secs = (elapsed.as_secs() % 60) as i64;
        let proj = timer
            .project_idx
            .and_then(|i| app.projects_cache.get(i))
            .map(|p| p.name.as_str())
            .unwrap_or("(none)");
        ui.label(format!(
            "Running: {}m {}s since {} on {} ({})",
            mins, secs, timer.start_time, timer.date, proj
        ));
        ui.label(format!(
            "Description: {}",
            if timer.description.is_empty() {
                "(none)"
            } else {
                &timer.description
            }
        ));
        if ui.button("Stop && Save Entry").clicked() {
            let stop_time = crate::domain::now_hhmm();
            let duration =
                crate::domain::duration_minutes_from_times(&timer.start_time, &stop_time)
                    .unwrap_or(((elapsed.as_secs() + 30) / 60) as i64)
                    .max(1);
            let new_entry = NewEntry {
                project_id: timer
                    .project_idx
                    .and_then(|i| app.projects_cache.get(i))
                    .map(|p| p.id),
                date: timer.date,
                start_time: Some(timer.start_time.clone()),
                end_time: Some(stop_time),
                duration_minutes: duration,
                description: timer.description.clone(),
                billable: timer.billable,
                tags: timer.tags.clone(),
            };
            match app.db.create_entry(&new_entry) {
                Ok(_) => {
                    app.status = format!("Timer entry saved ({} min)", duration);
                    app.timer = None;
                }
                Err(e) => app.status = format!("Failed to save timer entry: {e}"),
            }
        }
        if ui.button("Discard Timer").clicked() {
            app.timer = None;
            app.status = "Timer discarded".to_string();
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.label("Uses current form fields (project/description/billable/tags). Start time is set to now.");
            if ui.button("Start Timer").clicked() {
                app.timer = Some(TimerState {
                    running_since: Instant::now(),
                    date: Local::now().date_naive(),
                    project_idx: app.entry_project_idx,
                    start_time: crate::domain::now_hhmm(),
                    description: app.entry_description.clone(),
                    billable: app.entry_billable,
                    tags: app.entry_tags.clone(),
                });
                if app.entry_start.trim().is_empty() {
                    app.entry_start = crate::domain::now_hhmm();
                }
                app.status = "Timer started".to_string();
            }
        });
    }
}
