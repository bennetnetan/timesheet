use crate::db::models::{Entry, EntryFilter, NewEntry, Project};
use crate::domain::{format_minutes, parse_duration_to_minutes};
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use eframe::egui;
use std::collections::HashMap;

/// Compute the Monday of the ISO week containing `date`.
fn week_monday(date: NaiveDate) -> NaiveDate {
    let days_from_mon = date.weekday().num_days_from_monday() as i64;
    date - Duration::days(days_from_mon)
}

/// Render the weekly grid view on the Entries tab.
pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    let tokens = app.tokens();
    let today = chrono::Local::now().date_naive();
    let base_monday = week_monday(today);
    let offset = app.weekly_grid_week_offset;
    let monday = base_monday + Duration::weeks(offset as i64);
    let sunday = monday + Duration::days(6);

    // ── Week navigation header ──────────────────────────────────────────────
    ui.horizontal(|ui| {
        if ui.button("◀ Prev").clicked() {
            app.weekly_grid_week_offset -= 1;
            app.weekly_cell_editing = None;
        }
        let label = if offset == 0 {
            format!(
                "This week  ({} – {})",
                monday.format("%d %b"),
                sunday.format("%d %b %Y")
            )
        } else if offset == -1 {
            format!(
                "Last week  ({} – {})",
                monday.format("%d %b"),
                sunday.format("%d %b %Y")
            )
        } else {
            format!("{} – {}", monday.format("%d %b"), sunday.format("%d %b %Y"))
        };
        ui.strong(label);
        if ui.button("Next ▶").clicked() {
            app.weekly_grid_week_offset += 1;
            app.weekly_cell_editing = None;
        }
        if offset != 0 && ui.small_button("Go to current week").clicked() {
            app.weekly_grid_week_offset = 0;
            app.weekly_cell_editing = None;
        }
    });
    ui.add_space(4.0);
    ui.label("Click any cell to enter or edit hours. Supports: 90, 1.5, 1:30, 1h30m");
    ui.add_space(6.0);

    // ── Load entries for this week ──────────────────────────────────────────
    let filter = EntryFilter {
        from: Some(monday),
        to: Some(sunday),
        project_id: None,
        billable: None,
    };
    let entries: Vec<Entry> = app.db.list_entries(&filter).unwrap_or_default();

    // Build a map: (project_id_or_none, date) → total_minutes
    let mut cell_minutes: HashMap<(Option<i64>, NaiveDate), i64> = HashMap::new();
    // Also track which projects appear so we can populate the row list
    let mut seen_project_ids: Vec<Option<i64>> = Vec::new();

    for e in &entries {
        let key = (e.project_id, e.date);
        *cell_minutes.entry(key).or_insert(0) += e.duration_minutes;
        if !seen_project_ids.contains(&e.project_id) {
            seen_project_ids.push(e.project_id);
        }
    }

    // Build row list: active projects first, then any "No Project" if it has entries
    let active_projects: Vec<&Project> = app.projects_cache.iter().filter(|p| p.active).collect();

    let show_no_project = seen_project_ids.contains(&None);

    // Days of the week (Mon–Sun)
    let days: Vec<NaiveDate> = (0..7).map(|d| monday + Duration::days(d)).collect();
    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    // ── Collect actions to perform after rendering (avoid borrow conflicts) ─
    let mut action_save: Option<(Option<i64>, NaiveDate, i64)> = None; // (project_id, date, minutes)
    let mut action_cancel = false;

    // ── Grid ───────────────────────────────────────────────────────────────
    let col_project_w = 160.0;
    let col_day_w = 68.0;
    let _col_total_w = 60.0;

    // Header row
    egui::Grid::new("weekly_grid_header")
        .num_columns(9)
        .min_col_width(col_day_w)
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            // Project column header
            ui.allocate_exact_size(egui::vec2(col_project_w, 20.0), egui::Sense::hover());
            ui.label(egui::RichText::new("Project").strong());
            // Day headers
            for (i, day) in days.iter().enumerate() {
                let day_name = day_names[i];
                let is_today = *day == today;
                let is_weekend = day.weekday() == Weekday::Sat || day.weekday() == Weekday::Sun;
                let label = format!("{}\n{}", day_name, day.format("%d"));
                let text = if is_today {
                    egui::RichText::new(label).strong().color(tokens.primary)
                } else if is_weekend {
                    egui::RichText::new(label).color(tokens.on_surface_variant)
                } else {
                    egui::RichText::new(label)
                };
                ui.label(text);
            }
            ui.label(egui::RichText::new("Total").strong());
            ui.end_row();
        });

    ui.separator();

    // ── Scrollable body ─────────────────────────────────────────────────────
    egui::ScrollArea::vertical()
        .id_salt("weekly_grid_scroll")
        .max_height(400.0)
        .show(ui, |ui| {
            egui::Grid::new("weekly_grid_body")
                .num_columns(9)
                .min_col_width(col_day_w)
                .spacing([2.0, 2.0])
                .striped(true)
                .show(ui, |ui| {
                    // Render project rows
                    let render_project_row =
                        |ui: &mut egui::Ui,
                         project_id: Option<i64>,
                         proj_name: &str,
                         proj_color: Option<&str>,
                         proj_idx: usize,
                         cell_minutes: &HashMap<(Option<i64>, NaiveDate), i64>,
                         editing: &Option<(NaiveDate, Option<i64>)>,
                         cell_input: &mut String,
                         action_save: &mut Option<(Option<i64>, NaiveDate, i64)>,
                         action_cancel: &mut bool| {
                            // Project label
                            ui.allocate_exact_size(
                                egui::vec2(col_project_w, 24.0),
                                egui::Sense::hover(),
                            );
                            ui.horizontal(|ui| {
                                // Color dot
                                let color = crate::ui::project_color_32(proj_color, proj_idx);
                                crate::ui::render_color_dot(ui, color);
                                ui.label(egui::RichText::new(proj_name).color(
                                    if project_id.is_none() {
                                        tokens.on_surface_variant
                                    } else {
                                        egui::Color32::default()
                                    },
                                ));
                            });

                            let mut row_total: i64 = 0;

                            // Day cells
                            for day in &days {
                                let key = (project_id, *day);
                                let mins = cell_minutes.get(&key).copied().unwrap_or(0);
                                row_total += mins;

                                let is_editing = editing
                                    .as_ref()
                                    .map(|(d, pid)| d == day && pid == &project_id)
                                    .unwrap_or(false);

                                let is_today = *day == today;
                                let is_weekend =
                                    day.weekday() == Weekday::Sat || day.weekday() == Weekday::Sun;

                                if is_editing {
                                    // Active text input
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(cell_input)
                                            .desired_width(col_day_w - 4.0)
                                            .hint_text("1:30"),
                                    );
                                    resp.request_focus();

                                    let enter_pressed =
                                        ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    let esc_pressed =
                                        ui.input(|i| i.key_pressed(egui::Key::Escape));

                                    if enter_pressed {
                                        if let Some(parsed_mins) =
                                            parse_duration_to_minutes(cell_input)
                                        {
                                            if parsed_mins >= 0 {
                                                *action_save =
                                                    Some((project_id, *day, parsed_mins));
                                            }
                                        } else if cell_input.trim().is_empty() {
                                            // Blank = set to 0 (no-op / clear)
                                            *action_save = Some((project_id, *day, 0));
                                        }
                                    }
                                    if esc_pressed {
                                        *action_cancel = true;
                                    }
                                    // Also close if focus is lost
                                    if resp.lost_focus() && !enter_pressed && !esc_pressed {
                                        *action_cancel = true;
                                    }
                                } else {
                                    // Display cell
                                    let display = if mins == 0 {
                                        "–".to_string()
                                    } else {
                                        format_minutes(mins)
                                    };
                                    let text = egui::RichText::new(&display).monospace();
                                    let text = if is_today {
                                        text.color(tokens.primary)
                                    } else if is_weekend && mins == 0 {
                                        text.color(tokens.on_surface_variant)
                                    } else {
                                        text
                                    };

                                    let cell_resp = ui
                                        .add(egui::Label::new(text).sense(egui::Sense::click()))
                                        .on_hover_text("Click to edit");

                                    if cell_resp.clicked() {
                                        // We signal that the caller should start editing this cell.
                                        // We store the value via action_save with a sentinel of -1
                                        // to mean "open editor". Actually we use a separate mechanism:
                                        // we pack it in action_save as None-minutes sentinel.
                                        // Instead: We'll use a flag field — but we can't mutate app here.
                                        // The workaround: use action_save with mins = -999 as "open" signal.
                                        *action_save = Some((project_id, *day, -999));
                                    }
                                }
                            }

                            // Row total
                            if row_total > 0 {
                                ui.label(
                                    egui::RichText::new(format_minutes(row_total))
                                        .strong()
                                        .monospace(),
                                );
                            } else {
                                ui.label(egui::RichText::new("–").color(tokens.on_surface_variant));
                            }
                            ui.end_row();
                        };

                    // Clone what we need to avoid borrow issues
                    let editing_clone = app.weekly_cell_editing.clone();
                    let mut cell_input_clone = app.weekly_cell_input.clone();

                    // Active-project rows
                    for (idx, proj) in active_projects.iter().enumerate() {
                        render_project_row(
                            ui,
                            Some(proj.id),
                            &proj.name,
                            proj.color.as_deref(),
                            idx,
                            &cell_minutes,
                            &editing_clone,
                            &mut cell_input_clone,
                            &mut action_save,
                            &mut action_cancel,
                        );
                    }

                    // "No Project" row if any unassigned entries exist
                    if show_no_project {
                        render_project_row(
                            ui,
                            None,
                            "No Project",
                            None,
                            active_projects.len(),
                            &cell_minutes,
                            &editing_clone,
                            &mut cell_input_clone,
                            &mut action_save,
                            &mut action_cancel,
                        );
                    }

                    app.weekly_cell_input = cell_input_clone;
                });
        });

    // ── Totals footer ───────────────────────────────────────────────────────
    ui.separator();
    egui::Grid::new("weekly_grid_footer")
        .num_columns(9)
        .min_col_width(col_day_w)
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            ui.allocate_exact_size(egui::vec2(col_project_w, 20.0), egui::Sense::hover());
            ui.label(egui::RichText::new("Total").strong());
            let mut grand_total: i64 = 0;
            for day in &days {
                let day_total: i64 = cell_minutes
                    .iter()
                    .filter(|((_, d), _)| d == day)
                    .map(|(_, m)| m)
                    .sum();
                grand_total += day_total;
                let is_today = *day == today;
                let text = if day_total > 0 {
                    let t = egui::RichText::new(format_minutes(day_total))
                        .strong()
                        .monospace();
                    if is_today { t.color(tokens.primary) } else { t }
                } else {
                    egui::RichText::new("–").color(tokens.on_surface_variant)
                };
                ui.label(text);
            }
            ui.label(
                egui::RichText::new(format_minutes(grand_total))
                    .strong()
                    .monospace()
                    .color(tokens.tertiary),
            );
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "ℹ Editing a cell creates/updates an aggregated entry for that project & day.",
        )
        .small()
        .color(tokens.on_surface_variant),
    );

    // ── Apply deferred actions ───────────────────────────────────────────────
    if action_cancel {
        app.weekly_cell_editing = None;
        app.weekly_cell_input.clear();
    } else if let Some((project_id, date, mins)) = action_save {
        if mins == -999 {
            // Open editor for this cell
            // Pre-fill with existing value if any
            let existing = cell_minutes.get(&(project_id, date)).copied().unwrap_or(0);
            app.weekly_cell_editing = Some((date, project_id));
            app.weekly_cell_input = if existing > 0 {
                format_minutes(existing)
            } else {
                String::new()
            };
        } else {
            // Commit: upsert entry for (project_id, date)
            app.weekly_cell_editing = None;
            app.weekly_cell_input.clear();

            if mins > 0 {
                // Upsert: find existing entry for (project_id, date) or create new
                let existing_entry = entries
                    .iter()
                    .find(|e| e.project_id == project_id && e.date == date)
                    .cloned();

                if let Some(mut e) = existing_entry {
                    e.duration_minutes = mins;
                    if let Err(err) = app.db.update_entry(&e) {
                        app.status = format!("Failed to update entry: {err}");
                    } else {
                        app.status = format!(
                            "Updated {} on {}",
                            format_minutes(mins),
                            date.format("%d %b")
                        );
                    }
                } else {
                    let new_entry = NewEntry {
                        project_id,
                        date,
                        start_time: None,
                        end_time: None,
                        duration_minutes: mins,
                        description: String::new(),
                        billable: true,
                        tags: String::new(),
                    };
                    if let Err(err) = app.db.create_entry(&new_entry) {
                        app.status = format!("Failed to create entry: {err}");
                    } else {
                        app.status = format!(
                            "Logged {} on {}",
                            format_minutes(mins),
                            date.format("%d %b")
                        );
                    }
                }
            } else {
                // mins == 0: delete any existing entry for this cell
                let existing_entry = entries
                    .iter()
                    .find(|e| e.project_id == project_id && e.date == date)
                    .cloned();
                if let Some(e) = existing_entry {
                    if let Err(err) = app.db.delete_entry(e.id) {
                        app.status = format!("Failed to clear entry: {err}");
                    } else {
                        app.status = format!("Cleared entry on {}", date.format("%d %b"));
                    }
                }
            }
        }
    }
}
