use crate::db::models::EntryFilter;
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui;

const KPI_CARD_CONTENT_MIN_HEIGHT: f32 = 76.0;

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
    let tokens = app.tokens();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::ui::centered_content(ui, |ui| {
            let greeting = get_greeting(&app.settings.user_name);
            ui.label(
                egui::RichText::new(greeting)
                    .weak()
                    .size(crate::ui::theme::FONT_SIZE_BODY),
            );
            ui.add_space(4.0);

            omnibox_section(app, ui);
            ui.add_space(10.0);
            timer_section(app, ui);
            ui.add_space(10.0);

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
            .fill(tokens.tertiary_container)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎉").size(18.0));
                    ui.label(
                        egui::RichText::new(format!(
                            "Daily goal achieved! You've logged {} against your {:.1}h target today.",
                            crate::domain::format_minutes(total_today),
                            target_hours
                        ))
                        .color(tokens.on_tertiary_container)
                        .strong(),
                    );
                });
            });
        ui.add_space(6.0);
    }

    // Summary KPI cards
    ui.columns(4, |cols| {
        // Today
        cols[0].group(|ui| {
            ui.set_min_height(KPI_CARD_CONTENT_MIN_HEIGHT);
            ui.label(
                egui::RichText::new("TODAY")
                    .weak()
                    .size(crate::ui::theme::FONT_SIZE_LABEL),
            );
            let target_hours = app.settings.daily_target_hours.max(0.5);
            let target_minutes = (target_hours * 60.0) as f32;
            let progress = (total_today as f32 / target_minutes).clamp(0.0, 1.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(crate::domain::format_minutes(total_today))
                        .size(crate::ui::theme::FONT_SIZE_HEADLINE)
                        .monospace()
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.1}h / {:.1}h ({:.0}%)",
                            total_today as f32 / 60.0,
                            target_hours,
                            progress * 100.0
                        ))
                            .weak()
                            .size(crate::ui::theme::FONT_SIZE_LABEL),
                    );
                });
            });
            ui.add_sized(
                [ui.available_width(), 8.0],
                egui::ProgressBar::new(progress),
            );
        });

        // This Week
        cols[1].group(|ui| {
            ui.set_min_height(KPI_CARD_CONTENT_MIN_HEIGHT);
            ui.label(
                egui::RichText::new("THIS WEEK")
                    .weak()
                    .size(crate::ui::theme::FONT_SIZE_LABEL),
            );
            ui.label(
                egui::RichText::new(crate::domain::format_minutes(total_week))
                    .size(22.0)
                    .monospace()
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
            ui.set_min_height(KPI_CARD_CONTENT_MIN_HEIGHT);
            ui.label(
                egui::RichText::new("THIS MONTH")
                    .weak()
                    .size(crate::ui::theme::FONT_SIZE_LABEL),
            );
            ui.label(
                egui::RichText::new(crate::domain::format_minutes(total_month))
                    .size(22.0)
                    .monospace()
                    .strong(),
            );
            ui.weak(format!("{:.1} hours logged", total_month as f32 / 60.0));
        });

        // Est. Value
        cols[3].group(|ui| {
            ui.set_min_height(KPI_CARD_CONTENT_MIN_HEIGHT);
            ui.label(
                egui::RichText::new("EST. VALUE (MONTH)")
                    .weak()
                    .size(crate::ui::theme::FONT_SIZE_LABEL),
            );
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
        // Keep the activity feed and weekly overview side by side on wide layouts.
        let left_w = (avail_w * 0.52).max(440.0);
        let right_w = avail_w - left_w - 16.0;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    today_activity_section(app, ui, &today_entries);
                },
            );

            ui.add_space(16.0);

            ui.allocate_ui_with_layout(
                egui::vec2(right_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    weekly_bar_chart(ui, &all_entries, week_start, today, &app.tokens());
                    ui.add_space(10.0);
                    quick_add_section(app, ui);
                },
            );
        });
    } else {
        today_activity_section(app, ui, &today_entries);
        ui.add_space(10.0);
        ui.separator();
        weekly_bar_chart(ui, &all_entries, week_start, today, &app.tokens());
        ui.add_space(10.0);
        ui.separator();
        quick_add_section(app, ui);
    }

    ui.add_space(16.0);
            }); // end centered_content
    });
}

fn quick_add_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::theme::card_frame(&app.tokens()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong("Quick add");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::ui::material::primary_button(ui, "New entry").clicked() {
                    app.clear_entry_form();
                    app.show_entry_form = true;
                    app.focus_new_entry = true;
                }
            });
        });
        ui.add_space(4.0);
        ui.weak(
            "Log time in a focused dialog with start/end times, duration, project, tags, and billable tracking.",
        );
    });
}

fn omnibox_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    let tokens = app.tokens();
    crate::ui::render_form_card(
        ui,
        app.tokens(),
        "Quick Task Omnibox (Type natural language & press Enter)",
        "⚡",
        |ui| {
            ui.weak("Natural syntax: Type description, @project, #tags, or duration (e.g. Write report @Acme #research 45m)");
            ui.add_space(4.0);

            let mut submit_mode: Option<bool> = None;
            ui.horizontal_wrapped(|ui| {
                let omnibox_width = (ui.available_width() - 260.0).max(220.0);
                let edit_resp = crate::ui::material::input(
                    ui,
                    &mut app.omnibox_input,
                    "What are you working on? (e.g. Design mockup @Frontend #ui 45m)",
                    omnibox_width,
                );
                if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit_mode = Some(false);
                }

                if crate::ui::material::primary_button(ui, "▶ Start / Log")
                    .on_hover_text(
                        "If duration is specified, logs entry immediately. Otherwise, starts timer.",
                    )
                    .clicked()
                {
                    submit_mode = Some(false);
                }

                if crate::ui::material::tonal_button(ui, "📝 Log as Entry")
                    .on_hover_text("Always log as entry for today directly")
                    .clicked()
                {
                    submit_mode = Some(true);
                }

                if !app.omnibox_input.is_empty()
                    && crate::ui::material::chip(ui, "✕").clicked()
                {
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
                        ui.label(egui::RichText::new(format!("#{tag}")).color(tokens.primary));
                    }

                    if let Some(mins) = parsed.duration_minutes {
                        ui.label(
                            egui::RichText::new(format!(
                                "⏱ {}",
                                crate::domain::format_minutes(mins)
                            ))
                            .color(tokens.tertiary)
                            .strong(),
                        );
                    }
                });
            }
        },
    );
}

fn timer_section(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    let tokens = app.tokens();
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
                    ("⏸", tokens.warning)
                } else if timer.pomodoro_target_secs.is_some() {
                    ("🍅", tokens.error)
                } else {
                    ("●", tokens.error)
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
                        egui::RichText::new("• Billable").color(tokens.tertiary),
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
                    tokens.warning,
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
    tokens: &crate::ui::theme::Tokens,
) {
    crate::ui::theme::card_frame(tokens).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Weekly Overview")
                    .strong()
                    .size(crate::ui::theme::FONT_SIZE_TITLE),
            );
            ui.weak(format!("· week of {}", week_start.format("%b %d")));
        });
        ui.add_space(10.0);

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

        let bar_max_height = 64.0f32;
        ui.columns(7, |cols| {
            for (idx, (name, date, bill, nonb, total)) in day_data.into_iter().enumerate() {
                let is_today = date == today;
                cols[idx].vertical_centered(|ui| {
                    if total > 0 {
                        ui.label(
                            egui::RichText::new(format!("{:.1}h", total as f32 / 60.0))
                                .size(crate::ui::theme::FONT_SIZE_LABEL)
                                .strong(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("-")
                                .weak()
                                .size(crate::ui::theme::FONT_SIZE_LABEL),
                        );
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

                    let bg_color = tokens.surface_container_highest;
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
                            let nonb_color = tokens.outline;
                            ui.painter().rect_filled(top_rect, 3.0, nonb_color);
                        }

                        // Billable segment (bottom)
                        if bill_h > 0.0 {
                            let bottom_rect = egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, rect.max.y - bill_h),
                                egui::pos2(rect.max.x, rect.max.y),
                            );
                            let bill_color = tokens.tertiary;
                            ui.painter().rect_filled(bottom_rect, 3.0, bill_color);
                        }
                    }

                    if is_today {
                        ui.painter().rect_stroke(
                            rect,
                            4.0_f32,
                            egui::Stroke::new(1.5_f32, tokens.primary),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let day_text = if is_today {
                        egui::RichText::new(name).strong().color(tokens.primary)
                    } else {
                        egui::RichText::new(name).size(crate::ui::theme::FONT_SIZE_LABEL)
                    };
                    ui.label(day_text);
                    ui.weak(date.format("%m/%d").to_string());
                });
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let (r1, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter()
                .circle_filled(r1.center(), 4.0, tokens.tertiary);
            ui.weak("Billable");

            ui.add_space(8.0);
            let (r2, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(r2.center(), 4.0, tokens.outline);
            ui.weak("Non-billable");
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
                    if crate::ui::material::chip(ui, label).clicked() {
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

    if ui.available_width() < 720.0 {
        let tokens = app.tokens();
        let mut continue_entry: Option<crate::db::models::Entry> = None;
        let mut clone_entry: Option<crate::db::models::Entry> = None;

        for e in entries {
            let proj_idx = app.project_idx_for_id(e.project_id);
            let proj = proj_idx.and_then(|i| app.projects_cache.get(i));
            let dot_color = crate::ui::project_color_32(
                proj.and_then(|p| p.color.as_deref()),
                proj_idx.unwrap_or(0),
            );
            let time_range = match (&e.start_time, &e.end_time) {
                (Some(s), Some(end)) => format!("{s} - {end}"),
                (Some(s), None) => format!("{s} - ?"),
                _ => "-".to_string(),
            };

            crate::ui::theme::card_frame(&tokens).show(ui, |ui| {
                ui.strong(if e.description.is_empty() {
                    "(no description)"
                } else {
                    &e.description
                });
                ui.horizontal_wrapped(|ui| {
                    if let Some(project) = proj {
                        crate::ui::render_color_dot(ui, dot_color);
                        ui.label(&project.name);
                    } else {
                        ui.weak("No project");
                    }
                    ui.separator();
                    ui.label(time_range);
                    ui.separator();
                    ui.label(crate::domain::format_minutes(e.duration_minutes));
                    ui.weak(if e.billable {
                        "Billable"
                    } else {
                        "Non-billable"
                    });
                });
                ui.horizontal(|ui| {
                    if ui
                        .small_button("Continue")
                        .on_hover_text("Start a timer with these details")
                        .clicked()
                    {
                        continue_entry = Some(e.clone());
                    }
                    if ui
                        .small_button("Clone")
                        .on_hover_text("Clone entry to form for today")
                        .clicked()
                    {
                        clone_entry = Some(e.clone());
                    }
                });
            });
            ui.add_space(6.0);
        }

        if let Some(e) = continue_entry {
            app.continue_entry_timer(&e);
        }
        if let Some(e) = clone_entry {
            app.clone_entry_to_form(&e);
        }
        return;
    }

    let mut continue_entry: Option<crate::db::models::Entry> = None;
    let mut clone_entry: Option<crate::db::models::Entry> = None;

    crate::ui::theme::table_frame(&app.tokens()).show(ui, |ui| {
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("today_entries_grid")
                    .striped(true)
                    .min_col_width(50.0)
                    .spacing([14.0, 7.0])
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
                                    .small_button("Continue")
                                    .on_hover_text("Start a timer with these details")
                                    .clicked()
                                {
                                    continue_entry = Some(e.clone());
                                }
                                if ui
                                    .small_button("Clone")
                                    .on_hover_text("Clone entry to form for today")
                                    .clicked()
                                {
                                    clone_entry = Some(e.clone());
                                }
                            });
                            ui.end_row();
                        }
                    });
            });
    });

    if let Some(e) = continue_entry {
        app.continue_entry_timer(&e);
    }
    if let Some(e) = clone_entry {
        app.clone_entry_to_form(&e);
    }
}
