use crate::db::models::{EntryFilter, NewEntry};
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui;
use egui_extras::DatePickerButton;

fn get_greeting(user_name: &str) -> String {
    let now = Local::now();
    let hour = now.hour();
    let name_part = if user_name.trim().is_empty() {
        String::new()
    } else {
        format!(", {}", user_name.trim())
    };

    if (5..12).contains(&hour) {
        format!("Good morning{}! Ready for a focused day? ☀️", name_part)
    } else if (12..18).contains(&hour) {
        format!("Good afternoon{}! Making steady progress ⛅", name_part)
    } else if (18..23).contains(&hour) {
        format!("Good evening{}! Wrapping up your day? 🌙", name_part)
    } else {
        format!(
            "Burning the midnight oil{}? Take care and rest soon ☕",
            name_part
        )
    }
}

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Dashboard");
                ui.separator();
                let greeting = get_greeting(&app.settings.user_name);
                ui.label(egui::RichText::new(greeting).weak().size(13.0));
            });
            ui.add_space(4.0);

    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

    let mut total_today = 0i64;
    let mut total_week = 0i64;
    let mut billable_week = 0i64;
    let mut total_month = 0i64;
    let mut earnings_month = 0.0f64;
    let mut today_entries = Vec::new();
    let all_entries = app
        .db
        .list_entries(&EntryFilter::default())
        .unwrap_or_default();

    for e in &all_entries {
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

    let mut active_days_this_week = std::collections::HashSet::new();
    for e in &all_entries {
        if e.date >= week_start && e.date <= today && e.duration_minutes > 0 {
            active_days_this_week.insert(e.date);
        }
    }
    let days_count = active_days_this_week.len();

    let target_hours = app.settings.daily_target_hours.max(0.5);
    let target_minutes = (target_hours * 60.0) as i64;
    if total_today >= target_minutes && total_today > 0 {
        egui::Frame::group(ui.style())
            .fill(if app.settings.dark_theme {
                egui::Color32::from_rgb(16, 50, 30)
            } else {
                egui::Color32::from_rgb(236, 253, 245)
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎉").size(18.0));
                    ui.label(
                        egui::RichText::new(format!(
                            "Daily goal achieved! You've logged {} against your {:.1}h target today.",
                            crate::domain::format_minutes(total_today),
                            target_hours
                        ))
                        .color(if app.settings.dark_theme {
                            egui::Color32::from_rgb(52, 211, 153)
                        } else {
                            egui::Color32::from_rgb(5, 150, 105)
                        })
                        .strong(),
                    );
                });
            });
        ui.add_space(6.0);
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
            let target_hours = app.settings.daily_target_hours.max(0.5);
            let target_minutes = (target_hours * 60.0) as f32;
            let progress = (total_today as f32 / target_minutes).clamp(0.0, 1.0);
            let is_goal_met = total_today as f32 >= target_minutes;
            let bar_text = if is_goal_met {
                format!(
                    "{:.1}h / {:.1}h 🎉",
                    total_today as f32 / 60.0,
                    target_hours
                )
            } else {
                format!("{:.1}h / {:.1}h", total_today as f32 / 60.0, target_hours)
            };
            ui.add(egui::ProgressBar::new(progress).text(bar_text));
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
                "Billable: {} • {} active day{}",
                crate::domain::format_minutes(billable_week),
                days_count,
                if days_count == 1 { "" } else { "s" }
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

    let avail_w = ui.available_width();
    if avail_w > 900.0 {
        // AppFlowy-style balanced 2-column widescreen hub:
        // Left Column (52%): Omnibox + Active Timer + Today's Activity Feed
        // Right Column (48%): Weekly Overview Chart + Quick Add Form
        let left_w = (avail_w * 0.52).max(440.0);
        let right_w = avail_w - left_w - 16.0;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    omnibox_section(app, ui);
                    ui.add_space(10.0);
                    timer_section(app, ui);
                    ui.add_space(10.0);
                    today_activity_section(app, ui, &today_entries);
                },
            );

            ui.add_space(16.0);

            ui.allocate_ui_with_layout(
                egui::vec2(right_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    weekly_bar_chart(ui, &all_entries, week_start, today, app.settings.dark_theme);
                    ui.add_space(10.0);
                    quick_add_section(app, ui);
                },
            );
        });
    } else {
        omnibox_section(app, ui);
        ui.add_space(10.0);
        ui.separator();
        timer_section(app, ui);
        ui.add_space(10.0);
        ui.separator();
        weekly_bar_chart(ui, &all_entries, week_start, today, app.settings.dark_theme);
        ui.add_space(10.0);
        ui.separator();
        today_activity_section(app, ui, &today_entries);
        ui.add_space(10.0);
        ui.separator();
        quick_add_section(app, ui);
    }

    ui.add_space(16.0);
    });
}

fn quick_add_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(
        ui,
        app.settings.dark_theme,
        "Quick Add Entry Form",
        "➕",
        |ui| {
            ui.label(
                "Tip: enter start/end to auto-calculate duration, or enter duration directly.",
            );
            ui.add_space(4.0);

            egui::Grid::new("quick_add")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Date");
                    ui.add(DatePickerButton::new(&mut app.entry_date));
                    ui.end_row();

                    ui.strong("Project");
                    let sel_proj_info = app
                        .entry_project_idx
                        .and_then(|i| app.projects_cache.get(i).map(|p| (p, i)));

                    ui.horizontal(|ui| {
                        if let Some((p, idx)) = sel_proj_info {
                            let dot_color = crate::ui::project_color_32(p.color.as_deref(), idx);
                            crate::ui::render_color_dot(ui, dot_color);
                        }
                        egui::ComboBox::from_id_salt("dash_project")
                            .selected_text(
                                sel_proj_info
                                    .map(|(p, _)| p.name.as_str())
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
                                            .selectable_label(
                                                app.entry_project_idx == Some(i),
                                                &p.name,
                                            )
                                            .clicked()
                                        {
                                            app.entry_project_idx = Some(i);
                                        }
                                    });
                                }
                            });
                    });
                    ui.end_row();

                    ui.strong("Start (HH:MM)");
                    let s = ui.add(
                        egui::TextEdit::singleline(&mut app.entry_start)
                            .hint_text("09:00")
                            .desired_width(80.0),
                    );
                    if s.changed() {
                        if let Some(mins) = crate::domain::duration_minutes_from_times(
                            &app.entry_start,
                            &app.entry_end,
                        ) {
                            app.entry_duration = crate::domain::format_minutes(mins);
                        }
                    }
                    ui.end_row();

                    ui.strong("End (HH:MM)");
                    ui.horizontal(|ui| {
                        let e = ui.add(
                            egui::TextEdit::singleline(&mut app.entry_end)
                                .hint_text("10:30")
                                .desired_width(80.0),
                        );
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

                    ui.strong("Duration *");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.entry_duration)
                                .hint_text("1.5, 90, 1:30")
                                .desired_width(100.0),
                        );
                        ui.weak("(supports 90, 1.5, 1:30, 1h30m, 45m)");
                    });
                    ui.end_row();

                    ui.strong("Description");
                    ui.vertical(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.entry_description)
                                .hint_text("What did you work on?")
                                .desired_width(340.0),
                        );
                        let recents = app.recent_descriptions(4);
                        if !recents.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.weak("Recent:");
                                for d in recents {
                                    if ui.small_button(&d).clicked() {
                                        app.entry_description = d;
                                    }
                                }
                            });
                        }
                    });
                    ui.end_row();

                    ui.strong("Billable");
                    ui.checkbox(&mut app.entry_billable, "Track as billable hours");
                    ui.end_row();

                    ui.strong("Tags");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.entry_tags)
                            .hint_text("e.g. dev, meeting, bug")
                            .desired_width(240.0),
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("➕ Add Entry").strong())
                .clicked()
            {
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
                                app.status = "Entry added successfully".to_string();
                                app.clear_entry_form();
                            }
                            Err(e) => app.status = format!("Failed to add entry: {e}"),
                        }
                    }
                    _ => app.status = "Enter a valid duration or valid start/end times".to_string(),
                }
            }
        },
    );
}

fn omnibox_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(
        ui,
        app.settings.dark_theme,
        "Quick Task Omnibox (Type natural language & press Enter)",
        "⚡",
        |ui| {
            ui.weak("Natural syntax: Type description, @project, #tags, or duration (e.g. Write report @Acme #research 45m)");
            ui.add_space(4.0);

            let mut submit_mode: Option<bool> = None;
            ui.horizontal(|ui| {
                let edit_resp = ui.add(
                    egui::TextEdit::singleline(&mut app.omnibox_input)
                        .hint_text("What are you working on? (e.g. Design mockup @Frontend #ui 45m)")
                        .desired_width(450.0),
                );
                if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit_mode = Some(false);
                }

                if ui
                    .button(egui::RichText::new("▶ Start / Log").strong())
                    .on_hover_text(
                        "If duration is specified, logs entry immediately. Otherwise, starts timer.",
                    )
                    .clicked()
                {
                    submit_mode = Some(false);
                }

                if ui
                    .button("📝 Log as Entry")
                    .on_hover_text("Always log as entry for today directly")
                    .clicked()
                {
                    submit_mode = Some(true);
                }

                if !app.omnibox_input.is_empty() && ui.small_button("✕").clicked() {
                    app.omnibox_input.clear();
                }
            });

            if let Some(as_entry) = submit_mode {
                app.submit_omnibox(as_entry);
            }

            // Live parse preview
            let trimmed = app.omnibox_input.trim();
            if !trimmed.is_empty() {
                let parsed = crate::domain::parse_omnibox_input(trimmed);
                let proj_idx = parsed
                    .project_query
                    .as_deref()
                    .and_then(|q| app.find_project_by_query(q));
                let proj = proj_idx.and_then(|i| app.projects_cache.get(i));

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.weak("Parsed:");

                    if !parsed.description.is_empty() {
                        ui.label(format!("\"{}\"", parsed.description));
                    }

                    if let Some(p) = proj {
                        let dot_color =
                            crate::ui::project_color_32(p.color.as_deref(), proj_idx.unwrap_or(0));
                        ui.horizontal(|ui| {
                            crate::ui::render_color_dot(ui, dot_color);
                            ui.label(egui::RichText::new(&p.name).strong());
                        });
                    } else if let Some(query) = &parsed.project_query {
                        ui.weak(format!("@{} (no match)", query));
                    }

                    for tag in &parsed.tags {
                        ui.label(
                            egui::RichText::new(format!("#{tag}"))
                                .color(egui::Color32::from_rgb(59, 130, 246)),
                        );
                    }

                    if let Some(mins) = parsed.duration_minutes {
                        ui.label(
                            egui::RichText::new(format!(
                                "⏱ {}",
                                crate::domain::format_minutes(mins)
                            ))
                            .color(egui::Color32::from_rgb(16, 185, 129))
                            .strong(),
                        );
                    }
                });
            }
        },
    );
}

fn timer_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.group(|ui| {
        let timer_opt = app.timer.clone();
        if let Some(timer) = timer_opt {
            let total_secs = timer.total_elapsed_secs();
            let proj_idx = app.project_idx_for_id(timer.project_id);
            let proj = proj_idx.and_then(|i| app.projects_cache.get(i));
            let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("(No project)");
            let dot_color = crate::ui::project_color_32(
                proj.and_then(|p| p.color.as_deref()),
                proj_idx.unwrap_or(0),
            );

            let (heading_title, time_str, progress_opt, is_focus_complete) =
                if let Some(target) = timer.pomodoro_target_secs {
                    if timer.is_pomodoro_break {
                        let remaining = target.saturating_sub(total_secs);
                        let title = if timer.is_paused {
                            "Pomodoro Break (Paused)"
                        } else {
                            "Pomodoro Break ☕"
                        };
                        let s = format!("{:02}:{:02}", remaining / 60, remaining % 60);
                        let p = (total_secs as f32 / target as f32).clamp(0.0, 1.0);
                        (title, s, Some(p), false)
                    } else if total_secs < target {
                        let remaining = target - total_secs;
                        let title = if timer.is_paused {
                            "Pomodoro Focus (Paused)"
                        } else {
                            "Pomodoro Focus 🍅"
                        };
                        let s = format!("{:02}:{:02}", remaining / 60, remaining % 60);
                        let p = (total_secs as f32 / target as f32).clamp(0.0, 1.0);
                        (title, s, Some(p), false)
                    } else {
                        let overtime = total_secs - target;
                        let title = "Focus Finished! 🎉";
                        let s = format!("Complete! (+{:02}:{:02})", overtime / 60, overtime % 60);
                        (title, s, Some(1.0), true)
                    }
                } else {
                    let hrs = total_secs / 3600;
                    let mins = (total_secs % 3600) / 60;
                    let secs = total_secs % 60;
                    let title = if timer.is_paused {
                        "Timer (Paused)"
                    } else {
                        "Active Timer"
                    };
                    let s = format!("{:02}:{:02}:{:02}", hrs, mins, secs);
                    (title, s, None, false)
                };

            ui.horizontal(|ui| {
                let (dot_indicator, dot_color_val) = if timer.is_paused {
                    ("⏸", egui::Color32::from_rgb(245, 158, 11))
                } else if timer.pomodoro_target_secs.is_some() {
                    ("🍅", egui::Color32::from_rgb(239, 68, 68))
                } else {
                    ("●", egui::Color32::from_rgb(239, 68, 68))
                };
                ui.label(
                    egui::RichText::new(dot_indicator)
                        .color(dot_color_val)
                        .size(16.0),
                );
                ui.heading(heading_title);
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

            if let Some(p) = progress_opt {
                ui.add_space(2.0);
                ui.add(egui::ProgressBar::new(p).animate(!timer.is_paused));
            }

            ui.add_space(4.0);

            let mut adjust: Option<i64> = None;
            let mut toggle_pause = false;
            let mut start_break = false;
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

                // Pause / Resume button
                if timer.is_paused {
                    if ui.button(egui::RichText::new("▶ Resume").strong()).clicked() {
                        toggle_pause = true;
                    }
                } else if ui.button(egui::RichText::new("⏸ Pause").strong()).clicked() {
                    toggle_pause = true;
                }

                if is_focus_complete {
                    ui.add_space(6.0);
                    if ui
                        .button(egui::RichText::new("☕ Start 5m Break").strong())
                        .clicked()
                    {
                        start_break = true;
                    }
                }

                ui.add_space(8.0);

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

            if toggle_pause {
                app.toggle_pause_timer();
            }
            if start_break {
                app.stop_and_save_timer();
                app.start_pomodoro_break();
            }
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
            if total_secs >= 10800 {
                ui.add_space(2.0);
                ui.colored_label(
                    egui::Color32::from_rgb(245, 158, 11),
                    "⚠ Long-running session (> 3 hours). Remember to stop or adjust when your work is finished.",
                );
            }

            if !timer.is_paused {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(500));
            }
        } else {
            ui.horizontal(|ui| {
                ui.heading("Timer");
                ui.separator();
                ui.selectable_value(&mut app.pomodoro_mode, false, "⏱ Stopwatch");
                ui.selectable_value(
                    &mut app.pomodoro_mode,
                    true,
                    format!("🍅 Pomodoro ({}m)", app.settings.pomodoro_focus_mins),
                );
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let start_label = if app.pomodoro_mode {
                    format!(
                        "🍅 Start Focus Session ({}m)",
                        app.settings.pomodoro_focus_mins
                    )
                } else {
                    "▶ Start Timer (Ctrl+Space)".to_string()
                };
                if ui
                    .button(egui::RichText::new(start_label).strong())
                    .clicked()
                {
                    app.start_timer();
                }
                ui.weak(if app.pomodoro_mode {
                    "Focus timer counts down from 25 min and alerts you when it's time for a break."
                } else {
                    "Standard stopwatch counts elapsed time. Persists across restarts."
                });
            });
        }
    });
}

fn weekly_bar_chart(
    ui: &mut egui::Ui,
    entries: &[crate::db::models::Entry],
    week_start: NaiveDate,
    today: NaiveDate,
    dark_theme: bool,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Weekly Overview");
            ui.weak(format!("(Week of {})", week_start.format("%b %d")));
        });
        ui.add_space(6.0);

        let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let mut day_data = Vec::with_capacity(7);
        let mut max_minutes = 480i64; // minimum 8h scale for aesthetics

        for day_offset in 0..7 {
            let day_date = week_start + chrono::Duration::days(day_offset);
            let mut billable_m = 0i64;
            let mut non_billable_m = 0i64;

            for e in entries {
                if e.date == day_date {
                    if e.billable {
                        billable_m += e.duration_minutes;
                    } else {
                        non_billable_m += e.duration_minutes;
                    }
                }
            }
            let total = billable_m + non_billable_m;
            if total > max_minutes {
                max_minutes = total;
            }
            day_data.push((
                day_names[day_offset as usize],
                day_date,
                billable_m,
                non_billable_m,
                total,
            ));
        }

        let bar_max_height = 80.0f32;
        ui.columns(7, |cols| {
            for (idx, (name, date, bill, nonb, total)) in day_data.into_iter().enumerate() {
                let is_today = date == today;
                cols[idx].vertical_centered(|ui| {
                    if total > 0 {
                        ui.label(
                            egui::RichText::new(format!("{:.1}h", total as f32 / 60.0))
                                .size(11.0)
                                .strong(),
                        );
                    } else {
                        ui.label(egui::RichText::new("-").weak().size(11.0));
                    }

                    let bar_w = (ui.available_width() * 0.72).clamp(24.0, 52.0);
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(bar_w, bar_max_height),
                        egui::Sense::hover(),
                    );
                    resp.on_hover_text(format!(
                        "{} {}\nTotal: {:.1}h\nBillable: {:.1}h\nNon-billable: {:.1}h",
                        name,
                        date.format("%b %d"),
                        total as f32 / 60.0,
                        bill as f32 / 60.0,
                        nonb as f32 / 60.0
                    ));

                    let bg_color = if dark_theme {
                        egui::Color32::from_gray(40)
                    } else {
                        egui::Color32::from_gray(230)
                    };
                    ui.painter().rect_filled(rect, 4.0, bg_color);

                    if total > 0 {
                        let total_h = (total as f32 / max_minutes as f32) * bar_max_height;
                        let bill_h = (bill as f32 / total as f32) * total_h;
                        let nonb_h = total_h - bill_h;

                        // Non-billable segment (top)
                        if nonb_h > 0.0 {
                            let top_rect = egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, rect.max.y - total_h),
                                egui::pos2(rect.max.x, rect.max.y - bill_h),
                            );
                            let nonb_color = if dark_theme {
                                egui::Color32::from_rgb(100, 116, 139)
                            } else {
                                egui::Color32::from_rgb(148, 163, 184)
                            };
                            ui.painter().rect_filled(top_rect, 3.0, nonb_color);
                        }

                        // Billable segment (bottom)
                        if bill_h > 0.0 {
                            let bottom_rect = egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, rect.max.y - bill_h),
                                egui::pos2(rect.max.x, rect.max.y),
                            );
                            let bill_color = egui::Color32::from_rgb(16, 185, 129);
                            ui.painter().rect_filled(bottom_rect, 3.0, bill_color);
                        }
                    }

                    if is_today {
                        ui.painter().rect_stroke(
                            rect,
                            4.0_f32,
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let day_text = if is_today {
                        egui::RichText::new(name)
                            .strong()
                            .color(egui::Color32::from_rgb(59, 130, 246))
                    } else {
                        egui::RichText::new(name).size(12.0)
                    };
                    ui.label(day_text);
                    ui.weak(date.format("%m/%d").to_string());
                });
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let (r1, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(r1.center(), 4.0, egui::Color32::from_rgb(16, 185, 129));
            ui.weak("Billable");

            ui.add_space(8.0);
            let (r2, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(r2.center(), 4.0, egui::Color32::from_rgb(100, 116, 139));
            ui.weak("Non-billable");

            ui.add_space(8.0);
            ui.weak("• Blue border highlights Today");
        });
    });
}

fn today_activity_section(
    app: &mut TimesheetApp,
    ui: &mut egui::Ui,
    entries: &[crate::db::models::Entry],
) {
    ui.heading("Today's Activity");

    if entries.is_empty() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✨").size(20.0));
                ui.vertical(|ui| {
                    ui.strong("Your timeline is a blank slate today.");
                    ui.label(
                        egui::RichText::new(
                            "Start the timer above, or kick off instantly with a quick starter activity:",
                        )
                        .weak(),
                    );
                });
            });

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let starter_actions = [
                    ("💻 Deep Work", "Deep work session", true),
                    ("☕ Quick Sync (30m)", "Sync / Meeting", true),
                    ("📋 Planning & Admin", "Planning & administrative tasks", false),
                    ("🛠 Bug Fixing", "Bug investigation and fixing", true),
                    ("📖 Research & Notes", "Research and documentation", false),
                ];

                for (label, desc, billable) in starter_actions {
                    if ui.button(label).clicked() {
                        app.start_timer_with(
                            app.selected_project_id(),
                            desc.to_string(),
                            billable,
                            String::new(),
                        );
                    }
                }
            });
        });
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
