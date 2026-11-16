use crate::db::models::{EntryFilter, NewEntry};
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use egui_extras::DatePickerButton;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Dashboard");
    ui.add_space(6.0);

    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

    let mut total_today = 0i64;
    let mut total_week = 0i64;
    let mut billable_week = 0i64;
    let mut total_month = 0i64;
    let mut earnings_month = 0.0f64;
    let mut today_entries = Vec::new();

    if let Ok(entries) = app.db.list_entries(&EntryFilter::default()) {
        for e in &entries {
            let rate = e
                .project_id
                .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                .and_then(|p| p.rate)
                .unwrap_or(0.0);

            if e.date == today {
                total_today += e.duration_minutes;
                today_entries.push(e.clone());
            }
            if e.date >= week_start && e.date <= today {
                total_week += e.duration_minutes;
                if e.billable {
                    billable_week += e.duration_minutes;
                }
            }
            if e.date >= month_start && e.date <= today {
                total_month += e.duration_minutes;
                if e.billable {
                    earnings_month += (e.duration_minutes as f64 / 60.0) * rate;
                }
            }
        }
    }

    // Hero KPI Cards
    ui.columns(4, |cols| {
        // Today
        cols[0].group(|ui| {
            ui.label(egui::RichText::new("TODAY").weak().size(11.0));
            ui.label(
                egui::RichText::new(crate::domain::format_minutes(total_today))
                    .size(22.0)
                    .strong(),
            );
            let progress = (total_today as f32 / 480.0).clamp(0.0, 1.0);
            ui.add(
                egui::ProgressBar::new(progress)
                    .text(format!("{:.1}h / 8.0h", total_today as f32 / 60.0)),
            );
        });

        // This Week
        cols[1].group(|ui| {
            ui.label(egui::RichText::new("THIS WEEK").weak().size(11.0));
            ui.label(
                egui::RichText::new(crate::domain::format_minutes(total_week))
                    .size(22.0)
                    .strong(),
            );
            ui.weak(format!(
                "Billable: {}",
                crate::domain::format_minutes(billable_week)
            ));
        });

        // This Month
        cols[2].group(|ui| {
            ui.label(egui::RichText::new("THIS MONTH").weak().size(11.0));
            ui.label(
                egui::RichText::new(crate::domain::format_minutes(total_month))
                    .size(22.0)
                    .strong(),
            );
            ui.weak(format!("{:.1} hours logged", total_month as f32 / 60.0));
        });

        // Est. Value
        cols[3].group(|ui| {
            ui.label(egui::RichText::new("EST. VALUE (MONTH)").weak().size(11.0));
            ui.label(
                egui::RichText::new(format!("${:.2}", earnings_month))
                    .size(22.0)
                    .strong(),
            );
            ui.weak("Based on project rates");
        });
    });

    ui.add_space(10.0);
    ui.separator();

    // Timer section
    timer_section(app, ui);

    ui.add_space(10.0);
    ui.separator();

    // Today's Activity section
    today_activity_section(app, ui, &today_entries);

    ui.add_space(10.0);
    ui.separator();

    // Quick Add Entry form
    ui.collapsing("Quick Add Entry", |ui| {
        ui.label("Tip: enter start/end to auto-calculate duration, or enter duration directly.");
        ui.add_space(4.0);

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
                            let color = crate::ui::project_color_32(p.color.as_deref(), i);
                            ui.horizontal(|ui| {
                                crate::ui::render_color_dot(ui, color);
                                if ui
                                    .selectable_label(app.entry_project_idx == Some(i), &p.name)
                                    .clicked()
                                {
                                    app.entry_project_idx = Some(i);
                                }
                            });
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
                        if let Some(mins) = crate::domain::duration_minutes_from_times(
                            &app.entry_start,
                            &app.entry_end,
                        ) {
                            app.entry_duration = crate::domain::format_minutes(mins);
                        }
                    }
                    if ui.small_button("Now").clicked() {
                        app.entry_end = crate::domain::now_hhmm();
                        if let Some(mins) = crate::domain::duration_minutes_from_times(
                            &app.entry_start,
                            &app.entry_end,
                        ) {
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

        ui.add_space(4.0);
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
    });
}

fn timer_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        let timer_opt = app.timer.clone();
        if let Some(timer) = timer_opt {
            let elapsed = timer.running_since.elapsed();
            let total_secs = elapsed.as_secs();
            let hrs = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            let time_str = format!("{:02}:{:02}:{:02}", hrs, mins, secs);

            let proj_idx = app.project_idx_for_id(timer.project_id);
            let proj = proj_idx.and_then(|i| app.projects_cache.get(i));
            let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("(No project)");
            let dot_color = crate::ui::project_color_32(
                proj.and_then(|p| p.color.as_deref()),
                proj_idx.unwrap_or(0),
            );

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("●")
                        .color(egui::Color32::from_rgb(239, 68, 68))
                        .size(16.0),
                );
                ui.heading("Active Timer");
                ui.separator();
                crate::ui::render_color_dot(ui, dot_color);
                ui.strong(proj_name);
                if timer.billable {
                    ui.label(
                        egui::RichText::new("• Billable")
                            .color(egui::Color32::from_rgb(16, 185, 129)),
                    );
                }
            });

            ui.add_space(4.0);

            let mut adjust: Option<i64> = None;
            let mut stop = false;
            let mut discard = false;

            ui.horizontal(|ui| {
                // Large stopwatch readout
                ui.label(
                    egui::RichText::new(time_str)
                        .size(32.0)
                        .monospace()
                        .strong(),
                );

                ui.add_space(14.0);

                // Quick adjust buttons
                if ui
                    .button("-5m")
                    .on_hover_text("Subtract 5 minutes from elapsed")
                    .clicked()
                {
                    adjust = Some(-5);
                }
                if ui
                    .button("+5m")
                    .on_hover_text("Add 5 minutes to elapsed")
                    .clicked()
                {
                    adjust = Some(5);
                }
                if ui
                    .button("+15m")
                    .on_hover_text("Add 15 minutes to elapsed")
                    .clicked()
                {
                    adjust = Some(15);
                }

                ui.add_space(14.0);

                if ui
                    .button(egui::RichText::new("⏹ Stop & Save Entry").strong())
                    .clicked()
                {
                    stop = true;
                }
                if ui.button("✖ Discard").clicked() {
                    discard = true;
                }
            });

            if let Some(delta) = adjust {
                app.adjust_timer(delta);
            }
            if stop {
                app.stop_and_save_timer();
            }
            if discard {
                app.discard_timer();
            }

            ui.add_space(2.0);
            let desc_display = if timer.description.is_empty() {
                "(no description)"
            } else {
                &timer.description
            };
            ui.label(format!("Description: {}", desc_display));
            ui.weak(format!("Started at {} on {}", timer.start_time, timer.date));

            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        } else {
            ui.horizontal(|ui| {
                ui.heading("Timer");
                ui.label("• Ready to track");
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(egui::RichText::new("▶ Start Timer (Ctrl+Space)").strong())
                    .clicked()
                {
                    app.start_timer();
                }
                ui.weak("Uses current form fields (project, description, billable, tags). Persists across restarts.");
            });
        }
    });
}

fn today_activity_section(
    app: &mut TimesheetApp,
    ui: &mut egui::Ui,
    entries: &[crate::db::models::Entry],
) {
    ui.heading("Today's Activity");

    if entries.is_empty() {
        ui.label(
            egui::RichText::new(
                "No entries logged yet today. Start the timer above or use Quick Add.",
            )
            .weak(),
        );
        return;
    }

    let mut continue_entry: Option<crate::db::models::Entry> = None;
    let mut clone_entry: Option<crate::db::models::Entry> = None;

    egui::Grid::new("today_entries_grid")
        .striped(true)
        .min_col_width(50.0)
        .show(ui, |ui| {
            ui.strong("Project");
            ui.strong("Time");
            ui.strong("Duration");
            ui.strong("Billable");
            ui.strong("Description");
            ui.strong("Actions");
            ui.end_row();

            for e in entries {
                let proj_idx = app.project_idx_for_id(e.project_id);
                let proj = proj_idx.and_then(|i| app.projects_cache.get(i));
                let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("-");
                let dot_color = crate::ui::project_color_32(
                    proj.and_then(|p| p.color.as_deref()),
                    proj_idx.unwrap_or(0),
                );

                ui.horizontal(|ui| {
                    if e.project_id.is_some() {
                        crate::ui::render_color_dot(ui, dot_color);
                    }
                    ui.label(proj_name);
                });

                let time_range = match (&e.start_time, &e.end_time) {
                    (Some(s), Some(end)) => format!("{s} - {end}"),
                    (Some(s), None) => format!("{s} - ?"),
                    _ => "-".to_string(),
                };
                ui.label(time_range);
                ui.label(crate::domain::format_minutes(e.duration_minutes));
                ui.label(if e.billable { "Yes" } else { "No" });
                ui.label(&e.description);

                ui.horizontal(|ui| {
                    if ui
                        .small_button("▶")
                        .on_hover_text("Continue: start timer with these details")
                        .clicked()
                    {
                        continue_entry = Some(e.clone());
                    }
                    if ui
                        .small_button("📋")
                        .on_hover_text("Clone entry to form for today")
                        .clicked()
                    {
                        clone_entry = Some(e.clone());
                    }
                });
                ui.end_row();
            }
        });

    if let Some(e) = continue_entry {
        app.continue_entry_timer(&e);
    }
    if let Some(e) = clone_entry {
        app.clone_entry_to_form(&e);
    }
}
