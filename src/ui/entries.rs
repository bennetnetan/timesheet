use crate::db::models::{Entry, EntryFilter};
use crate::ui::app::{InlineEditState, SortColumn, TimesheetApp};
use crate::ui::weekly_grid;
use chrono::Datelike;
use eframe::egui;
use egui_extras::DatePickerButton;
use std::collections::{HashMap, HashSet};

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
        app.inline_edit = None;
        app.show_entry_form = true;
        app.focus_new_entry = true;
        app.status = "New entry form ready (Ctrl+N)".to_string();
    }
    if esc && !app.show_entry_form {
        if app.inline_edit.is_some() {
            app.inline_edit = None;
            app.status = "Inline edit cancelled".to_string();
        } else if app.editing_entry_id.is_some() {
            app.clear_entry_form();
            app.status = "Edit cancelled".to_string();
        }
        app.selected_entry_ids.clear();
    }
    // Enter saves the dialog once no field has focus (e.g. after Tab/Enter in a field).
    if enter && app.show_entry_form && ui.memory(|m| m.focused().is_none()) {
        if save_entry_form(app) {
            app.show_entry_form = false;
        }
    }

    ui.label("Shortcuts: Ctrl+N new, Enter save form, Esc cancel/clear, Delete delete selected, ▶ continue in timer, ⧉ clone.");
    ui.add_space(4.0);

    // View mode: Material 3 tabs (list vs weekly matrix)
    let mut view_idx = usize::from(app.weekly_view);
    if crate::ui::material::tabs(
        ui,
        "entries_view_tabs",
        &mut view_idx,
        &["☰ List", "▦ Weekly Grid"],
    ) {
        app.weekly_view = view_idx == 1;
    }
    ui.add_space(6.0);

    if app.weekly_view {
        weekly_grid::show(app, ui);
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::ui::centered_content(ui, |ui| {

    // Search and Quick Presets bar
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text("Search description, tags, project, client...")
                    .desired_width(280.0),
            );
            if !app.search_query.is_empty() && ui.small_button("✖").clicked() {
                app.search_query.clear();
            }

            ui.separator();
            ui.label("Range:");
            if ui.small_button("Today").clicked() {
                let t = chrono::Local::now().date_naive();
                app.filter.from = Some(t);
                app.filter.to = Some(t);
            }
            if ui.small_button("Yesterday").clicked() {
                let y = chrono::Local::now().date_naive() - chrono::Duration::days(1);
                app.filter.from = Some(y);
                app.filter.to = Some(y);
            }
            if ui.small_button("This week").clicked() {
                let t = chrono::Local::now().date_naive();
                let start = t - chrono::Duration::days(t.weekday().num_days_from_monday() as i64);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.small_button("This month").clicked() {
                let t = chrono::Local::now().date_naive();
                let start = chrono::NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap_or(t);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.small_button("All").clicked() {
                app.filter.from = None;
                app.filter.to = None;
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("From:");
            let mut from = app
                .filter
                .from
                .unwrap_or_else(|| chrono::Local::now().date_naive());
            let mut use_from = app.filter.from.is_some();
            if crate::ui::material::checkbox(ui, &mut use_from, "use").changed() {
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
            if crate::ui::material::checkbox(ui, &mut use_to, "use").changed() {
                app.filter.to = if use_to { Some(to) } else { None };
            }
            if app.filter.to.is_some() {
                if ui.add(DatePickerButton::new(&mut to)).changed() {
                    app.filter.to = Some(to);
                }
            }

            ui.separator();
            ui.label("Project:");
            let sel_proj = app
                .filter
                .project_id
                .and_then(|id| app.projects_cache.iter().position(|p| p.id == id))
                .and_then(|idx| app.projects_cache.get(idx).map(|p| (p, idx)));

            if let Some((p, idx)) = sel_proj {
                let dot_color = crate::ui::project_color_32(p.color.as_deref(), idx);
                crate::ui::render_color_dot(ui, dot_color);
            }

            egui::ComboBox::from_id_salt("filter_project")
                .selected_text(
                    sel_proj
                        .map(|(p, _)| p.name.as_str())
                        .unwrap_or("All Projects"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.filter.project_id.is_none(), "All Projects")
                        .clicked()
                    {
                        app.filter.project_id = None;
                    }
                    for (idx, p) in app.projects_cache.iter().enumerate() {
                        let dot_color = crate::ui::project_color_32(p.color.as_deref(), idx);
                        let is_selected = app.filter.project_id == Some(p.id);
                        let clicked = ui
                            .horizontal(|ui| {
                                crate::ui::render_color_dot(ui, dot_color);
                                ui.selectable_label(is_selected, &p.name).clicked()
                            })
                            .inner;
                        if clicked {
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

            if ui.button("Clear filters").clicked() {
                app.filter = crate::db::models::EntryFilter::default();
                app.search_query.clear();
                app.filter_tag = None;
            }
        });

        // Tag cloud chips
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        if let Ok(all_entries) = app
            .db
            .list_entries(&crate::db::models::EntryFilter::default())
        {
            for e in all_entries {
                for raw_t in e.tags.split(|c: char| c == ',' || c == ' ' || c == ';') {
                    let t = raw_t.trim().trim_start_matches('#');
                    if !t.is_empty() {
                        *tag_counts.entry(t.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut sorted_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        if !sorted_tags.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.weak("Tags:");
                let all_active = app.filter_tag.is_none();
                if ui.selectable_label(all_active, "All").clicked() {
                    app.filter_tag = None;
                }
                for (tag, count) in sorted_tags.iter().take(12) {
                    let is_active = app.filter_tag.as_deref() == Some(tag);
                    let label = format!("#{tag} ({count})");
                    if ui.selectable_label(is_active, label).clicked() {
                        if is_active {
                            app.filter_tag = None;
                        } else {
                            app.filter_tag = Some(tag.clone());
                        }
                    }
                }
            });
        }
    });

    ui.separator();

    let is_editing = app.editing_entry_id.is_some();

    ui.horizontal(|ui| {
        if crate::ui::material::primary_button(ui, "➕ New Entry (Ctrl+N)").clicked() {
            app.clear_entry_form();
            app.show_entry_form = true;
            app.focus_new_entry = true;
        }
        if is_editing {
            ui.label(
                egui::RichText::new(format!("Editing Entry #{}", app.editing_entry_id.unwrap()))
                    .color(app.tokens().warning)
                    .strong(),
            );
            if ui.small_button("Cancel Edit (Esc)").clicked() {
                app.clear_entry_form();
            }
        }
    });

    ui.separator();

    let mut entries = app.db.list_entries(&app.filter).unwrap_or_default();

    // Tag filtering
    if let Some(target_tag) = &app.filter_tag {
        let tt = target_tag.to_lowercase();
        entries.retain(|e| {
            e.tags
                .split(|c: char| c == ',' || c == ' ' || c == ';')
                .any(|t| t.trim().trim_start_matches('#').eq_ignore_ascii_case(&tt))
        });
    }

    // Full-text search filtering
    if !app.search_query.trim().is_empty() {
        let q = app.search_query.trim().to_lowercase();
        entries.retain(|e| {
            let proj = e
                .project_id
                .and_then(|id| app.projects_cache.iter().find(|p| p.id == id));
            let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("");
            let proj_client = proj.and_then(|p| p.client.as_deref()).unwrap_or("");
            e.description.to_lowercase().contains(&q)
                || e.tags.to_lowercase().contains(&q)
                || proj_name.to_lowercase().contains(&q)
                || proj_client.to_lowercase().contains(&q)
        });
    }

    // Column sorting
    match app.sort_column {
        SortColumn::Date => {
            if app.sort_ascending {
                entries.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));
            } else {
                entries.sort_by(|a, b| b.date.cmp(&a.date).then(b.id.cmp(&a.id)));
            }
        }
        SortColumn::Duration => {
            if app.sort_ascending {
                entries.sort_by_key(|e| e.duration_minutes);
            } else {
                entries.sort_by(|a, b| b.duration_minutes.cmp(&a.duration_minutes));
            }
        }
        SortColumn::Project => {
            let get_name = |pid: Option<i64>| -> String {
                pid.and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                    .map(|p| p.name.to_lowercase())
                    .unwrap_or_default()
            };
            if app.sort_ascending {
                entries.sort_by(|a, b| get_name(a.project_id).cmp(&get_name(b.project_id)));
            } else {
                entries.sort_by(|a, b| get_name(b.project_id).cmp(&get_name(a.project_id)));
            }
        }
    }

    let valid_ids: HashSet<i64> = entries.iter().map(|e| e.id).collect();
    app.selected_entry_ids.retain(|id| valid_ids.contains(id));

    if del && !app.selected_entry_ids.is_empty() && app.inline_edit.is_none() {
        delete_selected(app);
        entries = app.db.list_entries(&app.filter).unwrap_or_default();
    }

    let total_mins: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    let billable_mins: i64 = entries
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.duration_minutes)
        .sum();
    let non_billable_mins = total_mins.saturating_sub(billable_mins);
    let mut total_est_value = 0.0_f64;
    for e in &entries {
        if e.billable {
            let rate = e
                .project_id
                .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
                .and_then(|p| p.rate)
                .unwrap_or(0.0);
            total_est_value += (e.duration_minutes as f64 / 60.0) * rate;
        }
    }

    crate::ui::theme::bar_frame(&app.tokens()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong(format!("{} entries", entries.len()));
            ui.separator();
            ui.label(
                egui::RichText::new(format!(
                    "Total: {}",
                    crate::domain::format_minutes(total_mins)
                ))
                .strong(),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!(
                    "Billable: {} (${:.2})",
                    crate::domain::format_minutes(billable_mins),
                    total_est_value
                ))
                .color(app.tokens().tertiary),
            );
            if non_billable_mins > 0 {
                ui.separator();
                ui.weak(format!(
                    "Non-billable: {}",
                    crate::domain::format_minutes(non_billable_mins)
                ));
            }

            if !app.selected_entry_ids.is_empty() {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("{} selected", app.selected_entry_ids.len()))
                        .color(app.tokens().primary)
                        .strong(),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("↗ Open in Reports")
                    .on_hover_text("View detailed breakdown and export options in Reports tab")
                    .clicked()
                {
                    app.tab = crate::ui::app::Tab::Reports;
                }
                if ui
                    .small_button("📋 Copy Summary")
                    .on_hover_text("Copy filtered slice summary to clipboard")
                    .clicked()
                {
                    let summary_text = format!(
                        "Timesheet Summary: {} entries, Total: {}, Billable: {} (${:.2}), Non-billable: {}",
                        entries.len(),
                        crate::domain::format_minutes(total_mins),
                        crate::domain::format_minutes(billable_mins),
                        total_est_value,
                        crate::domain::format_minutes(non_billable_mins)
                    );
                    ui.ctx().copy_text(summary_text);
                    app.status = "Summary copied to clipboard".to_string();
                }
            });
        });
    });

    // Bulk actions
    crate::ui::theme::bar_frame(&app.tokens()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Bulk actions on selected:");
            if ui.button("Select all").clicked() {
                app.selected_entry_ids = entries.iter().map(|e| e.id).collect();
            }
            if ui.button("Select none").clicked() {
                app.selected_entry_ids.clear();
            }
            if ui.button("Delete selected").clicked() && !app.selected_entry_ids.is_empty() {
                delete_selected(app);
                entries = app.db.list_entries(&app.filter).unwrap_or_default();
            }
            ui.separator();
            ui.checkbox(&mut app.bulk_billable, "Billable");
            if ui.button("Apply billable").clicked() && !app.selected_entry_ids.is_empty() {
                bulk_update_billable(app, &entries);
                entries = app.db.list_entries(&app.filter).unwrap_or_default();
            }
            ui.label("Tags:");
            ui.add(egui::TextEdit::singleline(&mut app.bulk_tags).desired_width(120.0));
            if ui.button("Apply tags").clicked() && !app.selected_entry_ids.is_empty() {
                bulk_update_tags(app, &entries);
                entries = app.db.list_entries(&app.filter).unwrap_or_default();
            }
        });
    });

    render_table(app, ui, &entries);

            ui.add_space(20.0);
            });
        });
}

fn delete_selected(app: &mut TimesheetApp) {
    let ids: Vec<i64> = app.selected_entry_ids.iter().copied().collect();
    if ids.is_empty() {
        return;
    }
    app.confirm = Some(crate::ui::app::ConfirmState {
        title: "Delete Entries".to_string(),
        message: format!(
            "Delete {} selected entr{}? This cannot be undone.",
            ids.len(),
            if ids.len() == 1 { "y" } else { "ies" }
        ),
        action: crate::ui::app::ConfirmAction::DeleteEntries(ids),
        backup_path: None,
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

fn render_table(app: &mut TimesheetApp, ui: &mut egui::Ui, entries: &[Entry]) {
    if entries.is_empty() {
        ui.add_space(16.0);
        let has_filter = !app.search_query.trim().is_empty()
            || app.filter.from.is_some()
            || app.filter.to.is_some()
            || app.filter.project_id.is_some()
            || app.filter.billable.is_some()
            || app.filter_tag.is_some();

        if has_filter {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍").size(24.0));
                    ui.vertical(|ui| {
                        ui.strong("No entries match your search or filter.");
                        ui.label(
                            egui::RichText::new(
                                "Try broadening your date range or clearing your search query.",
                            )
                            .weak(),
                        );
                        ui.add_space(4.0);
                        if ui.button("↺ Reset all filters & search").clicked() {
                            app.search_query.clear();
                            app.filter = EntryFilter::default();
                            app.filter_tag = None;
                        }
                    });
                });
            });
        } else {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📝").size(24.0));
                    ui.vertical(|ui| {
                        ui.strong("No time entries recorded yet.");
                        ui.label(
                            egui::RichText::new(
                                "Start tracking your time using the live timer or create your first entry above.",
                            )
                            .weak(),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("▶ Start Timer (Ctrl+Space)").clicked() {
                                app.start_timer();
                            }
                            if ui.button("➕ Open New Entry Form (Ctrl+N)").clicked() {
                                app.focus_new_entry = true;
                            }
                        });
                    });
                });
            });
        }
        return;
    }

    let mut save_inline: Option<Entry> = None;
    let mut cancel_inline = false;
    let mut start_inline: Option<Entry> = None;
    let mut delete_id: Option<i64> = None;
    let mut load_form: Option<Entry> = None;
    let mut continue_entry: Option<Entry> = None;
    let mut clone_entry: Option<Entry> = None;

    let avail_w = ui.available_width();
    let desc_w = (avail_w - 740.0).max(220.0);

    crate::ui::theme::table_frame(&app.tokens()).show(ui, |ui| {
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("entries_grid")
                    .striped(true)
                    .min_col_width(30.0)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Sel");
                        ui.strong("#ID");

                        let date_label = format!(
                            "Date{}",
                            if app.sort_column == SortColumn::Date {
                                if app.sort_ascending { " ▲" } else { " ▼" }
                            } else {
                                ""
                            }
                        );
                        if ui
                            .button(egui::RichText::new(date_label).strong())
                            .clicked()
                        {
                            if app.sort_column == SortColumn::Date {
                                app.sort_ascending = !app.sort_ascending;
                            } else {
                                app.sort_column = SortColumn::Date;
                                app.sort_ascending = false;
                            }
                        }

                        let proj_label = format!(
                            "Project{}",
                            if app.sort_column == SortColumn::Project {
                                if app.sort_ascending { " ▲" } else { " ▼" }
                            } else {
                                ""
                            }
                        );
                        if ui
                            .button(egui::RichText::new(proj_label).strong())
                            .clicked()
                        {
                            if app.sort_column == SortColumn::Project {
                                app.sort_ascending = !app.sort_ascending;
                            } else {
                                app.sort_column = SortColumn::Project;
                                app.sort_ascending = true;
                            }
                        }

                        ui.strong("Start");
                        ui.strong("End");

                        let dur_label = format!(
                            "Dur{}",
                            if app.sort_column == SortColumn::Duration {
                                if app.sort_ascending { " ▲" } else { " ▼" }
                            } else {
                                ""
                            }
                        );
                        if ui.button(egui::RichText::new(dur_label).strong()).clicked() {
                            if app.sort_column == SortColumn::Duration {
                                app.sort_ascending = !app.sort_ascending;
                            } else {
                                app.sort_column = SortColumn::Duration;
                                app.sort_ascending = false;
                            }
                        }

                        ui.strong("Bill");
                        ui.strong("Tags");
                        ui.add_sized(
                            [desc_w, 20.0],
                            egui::Label::new(egui::RichText::new("Description").strong()),
                        );
                        ui.strong("Actions");
                        ui.end_row();

                        for e in entries {
                            let is_inline = app.inline_edit.as_ref().map(|x| x.id) == Some(e.id);
                            if is_inline {
                                let proj_names: Vec<(Option<usize>, String)> = {
                                    let mut v = vec![(None, "(none)".to_string())];
                                    v.extend(
                                        app.projects_cache
                                            .iter()
                                            .enumerate()
                                            .map(|(i, p)| (Some(i), p.name.clone())),
                                    );
                                    v
                                };
                                let mut built: Option<Entry> = None;
                                if let Some(edit) = app.inline_edit.as_mut() {
                                    let mut selected = app.selected_entry_ids.contains(&e.id);
                                    if ui.checkbox(&mut selected, "").changed() {
                                        if selected {
                                            app.selected_entry_ids.insert(e.id);
                                        } else {
                                            app.selected_entry_ids.remove(&e.id);
                                        }
                                    }
                                    ui.weak(format!("#{}", e.id));
                                    let salt = format!("in_date_{}", e.id);
                                    ui.add(DatePickerButton::new(&mut edit.date).id_salt(&salt));
                                    egui::ComboBox::from_id_salt(("in_proj", e.id))
                                        .selected_text(
                                            edit.project_idx
                                                .and_then(|i| app.projects_cache.get(i))
                                                .map(|p| p.name.as_str())
                                                .unwrap_or("(none)"),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (idx, name) in &proj_names {
                                                if ui
                                                    .selectable_label(
                                                        edit.project_idx == *idx,
                                                        name.as_str(),
                                                    )
                                                    .clicked()
                                                {
                                                    edit.project_idx = *idx;
                                                }
                                            }
                                        });
                                    let s = ui.add(
                                        egui::TextEdit::singleline(&mut edit.start)
                                            .desired_width(55.0),
                                    );
                                    if s.changed() {
                                        if let Some(m) = crate::domain::duration_minutes_from_times(
                                            &edit.start,
                                            &edit.end,
                                        ) {
                                            edit.duration = crate::domain::format_minutes(m);
                                        }
                                    }
                                    let en = ui.add(
                                        egui::TextEdit::singleline(&mut edit.end)
                                            .desired_width(55.0),
                                    );
                                    if en.changed() {
                                        if let Some(m) = crate::domain::duration_minutes_from_times(
                                            &edit.start,
                                            &edit.end,
                                        ) {
                                            edit.duration = crate::domain::format_minutes(m);
                                        }
                                    }
                                    ui.add(
                                        egui::TextEdit::singleline(&mut edit.duration)
                                            .desired_width(55.0),
                                    );
                                    ui.checkbox(&mut edit.billable, "");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut edit.tags)
                                            .desired_width(90.0),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut edit.description)
                                            .desired_width(desc_w),
                                    );
                                    ui.horizontal(|ui| {
                                        if ui.small_button("Save").clicked() {
                                            let mins = crate::domain::duration_minutes_from_times(
                                                &edit.start,
                                                &edit.end,
                                            )
                                            .or_else(|| {
                                                crate::domain::parse_duration_to_minutes(
                                                    &edit.duration,
                                                )
                                            });
                                            if let Some(mins) = mins.filter(|m| *m > 0) {
                                                built = Some(Entry {
                                                    id: e.id,
                                                    project_id: edit
                                                        .project_idx
                                                        .and_then(|i| app.projects_cache.get(i))
                                                        .map(|p| p.id),
                                                    date: edit.date,
                                                    start_time: if edit.start.trim().is_empty() {
                                                        None
                                                    } else {
                                                        Some(edit.start.trim().to_string())
                                                    },
                                                    end_time: if edit.end.trim().is_empty() {
                                                        None
                                                    } else {
                                                        Some(edit.end.trim().to_string())
                                                    },
                                                    duration_minutes: mins,
                                                    description: edit.description.clone(),
                                                    billable: edit.billable,
                                                    tags: edit.tags.clone(),
                                                });
                                            } else {
                                                app.status =
                                                "Inline save failed: invalid duration/start-end"
                                                    .to_string();
                                            }
                                        }
                                        if ui.small_button("Cancel").clicked() {
                                            cancel_inline = true;
                                        }
                                    });
                                    if let Some(b) = built {
                                        save_inline = Some(b);
                                    }
                                }
                                ui.end_row();
                                continue;
                            }

                            let mut selected = app.selected_entry_ids.contains(&e.id);
                            if ui.checkbox(&mut selected, "").changed() {
                                if selected {
                                    app.selected_entry_ids.insert(e.id);
                                } else {
                                    app.selected_entry_ids.remove(&e.id);
                                }
                            }

                            let is_editing_this = app.editing_entry_id == Some(e.id);
                            if is_editing_this {
                                ui.label(
                                    egui::RichText::new(format!("✏ #{}", e.id))
                                        .strong()
                                        .color(app.tokens().warning),
                                );
                            } else {
                                ui.weak(format!("#{}", e.id));
                            }

                            ui.label(e.date.to_string());
                            let proj_idx = app.project_idx_for_id(e.project_id);
                            let proj = proj_idx.and_then(|i| app.projects_cache.get(i));
                            ui.horizontal(|ui| {
                                if let Some(p) = proj {
                                    let dot_color = crate::ui::project_color_32(
                                        p.color.as_deref(),
                                        proj_idx.unwrap_or(0),
                                    );
                                    crate::ui::render_color_dot(ui, dot_color);
                                    if let Some(client) = &p.client {
                                        ui.label(format!("{} ({})", p.name, client));
                                    } else {
                                        ui.label(&p.name);
                                    }
                                } else {
                                    ui.weak("(none)");
                                }
                            });
                            ui.label(e.start_time.as_deref().unwrap_or("-"));
                            ui.label(e.end_time.as_deref().unwrap_or("-"));
                            ui.label(crate::domain::format_minutes(e.duration_minutes));
                            if e.billable {
                                ui.label(egui::RichText::new("Yes").color(app.tokens().tertiary));
                            } else {
                                ui.weak("No");
                            }
                            ui.label(&e.tags);
                            ui.add_sized([desc_w, 18.0], egui::Label::new(&e.description));
                            ui.horizontal(|ui| {
                                let action = |ui: &mut egui::Ui, icon: &str, tip: &str| {
                                    ui.add_sized(
                                        [24.0, 22.0],
                                        egui::Button::new(egui::RichText::new(icon).size(12.0)),
                                    )
                                    .on_hover_text(tip)
                                    .clicked()
                                };
                                if action(
                                    ui,
                                    "▶",
                                    "Continue: start timer with this entry's details",
                                ) {
                                    continue_entry = Some(e.clone());
                                }
                                if action(ui, "⧉", "Clone entry into form for today") {
                                    clone_entry = Some(e.clone());
                                }
                                if action(ui, "✎", "Edit this entry in the dialog") {
                                    load_form = Some(e.clone());
                                }
                                if action(ui, "⇄", "Quick edit this row inline") {
                                    start_inline = Some(e.clone());
                                }
                                if action(ui, "✕", "Delete this entry") {
                                    delete_id = Some(e.id);
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

    if let Some(e) = start_inline {
        app.inline_edit = Some(InlineEditState {
            id: e.id,
            date: e.date,
            project_idx: app.project_idx_for_id(e.project_id),
            start: e.start_time.unwrap_or_default(),
            end: e.end_time.unwrap_or_default(),
            duration: crate::domain::format_minutes(e.duration_minutes),
            description: e.description,
            billable: e.billable,
            tags: e.tags,
        });
        app.status = format!("Inline editing entry #{}", e.id);
    }
    if cancel_inline {
        app.inline_edit = None;
    }
    if let Some(e) = save_inline {
        match app.find_overlaps(&e, Some(e.id)) {
            Ok(ov) if !ov.is_empty() => {
                app.status = format!(
                    "Cannot save: overlaps {} existing entr{}",
                    ov.len(),
                    if ov.len() == 1 { "y" } else { "ies" }
                )
            }
            _ => match app.db.update_entry(&e) {
                Ok(_) => {
                    app.inline_edit = None;
                    app.status = "Entry updated (inline)".to_string();
                }
                Err(err) => app.status = format!("Inline update failed: {err}"),
            },
        }
    }
    if let Some(id) = delete_id {
        app.confirm = Some(crate::ui::app::ConfirmState {
            title: "Delete Entry".to_string(),
            message: format!("Delete entry #{}? This cannot be undone.", id),
            action: crate::ui::app::ConfirmAction::DeleteEntry(id),
            backup_path: None,
        });
    }
    if let Some(e) = load_form {
        load_entry_into_form(app, &e);
    }
}

fn load_entry_into_form(app: &mut TimesheetApp, e: &Entry) {
    app.editing_entry_id = Some(e.id);
    app.cloned_from_entry_id = None;
    app.show_entry_form = true;
    app.entry_date = e.date;
    app.entry_project_idx = app.project_idx_for_id(e.project_id);
    app.entry_start = e.start_time.clone().unwrap_or_default();
    app.entry_end = e.end_time.clone().unwrap_or_default();
    app.entry_duration = crate::domain::format_minutes(e.duration_minutes);
    app.entry_description = e.description.clone();
    app.entry_billable = e.billable;
    app.entry_tags = e.tags.clone();
    app.status = format!("Editing entry #{} in form", e.id);
}

pub(crate) fn entry_form(app: &mut TimesheetApp, ui: &mut egui::Ui) -> bool {
    let mut save_clicked = false;

    // Context banner if editing or cloned
    if let Some(id) = app.editing_entry_id {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "✏ Currently editing Entry #{} — changes will overwrite this record",
                    id
                ))
                .color(app.tokens().warning)
                .strong(),
            );
            if ui.small_button("✖ Cancel Edit (Esc)").clicked() {
                app.clear_entry_form();
                app.show_entry_form = false;
            }
        });
        ui.add_space(4.0);
    } else if let Some(clone_id) = app.cloned_from_entry_id {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "📋 Cloned from Entry #{} for today. Edit and save as a new record.",
                    clone_id
                ))
                .color(app.tokens().primary),
            );
            if ui.small_button("↺ Clear").clicked() {
                app.clear_entry_form();
            }
        });
        ui.add_space(4.0);
    }

    // Row 1: Date & Project
    ui.horizontal(|ui| {
        ui.strong("📅 Date:");
        ui.add(DatePickerButton::new(&mut app.entry_date));

        ui.add_space(16.0);
        ui.strong("📁 Project:");
        let sel_proj_info = app
            .entry_project_idx
            .and_then(|i| app.projects_cache.get(i).map(|p| (p, i)));

        if let Some((p, idx)) = sel_proj_info {
            let dot_color = crate::ui::project_color_32(p.color.as_deref(), idx);
            crate::ui::render_color_dot(ui, dot_color);
        }

        let proj_title = sel_proj_info
            .map(|(p, _)| {
                if let Some(c) = &p.client {
                    format!("{} ({})", p.name, c)
                } else {
                    p.name.clone()
                }
            })
            .unwrap_or_else(|| "(none)".to_string());

        egui::ComboBox::from_id_salt("entry_project")
            .selected_text(proj_title)
            .width(220.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(app.entry_project_idx.is_none(), "(none)")
                    .clicked()
                {
                    app.entry_project_idx = None;
                }
                for (i, p) in app.projects_cache.iter().enumerate() {
                    let dot = crate::ui::project_color_32(p.color.as_deref(), i);
                    let label = if let Some(c) = &p.client {
                        if p.active {
                            format!("{} ({})", p.name, c)
                        } else {
                            format!("{} ({}) [archived]", p.name, c)
                        }
                    } else if p.active {
                        p.name.clone()
                    } else {
                        format!("{} (archived)", p.name)
                    };
                    let is_sel = app.entry_project_idx == Some(i);
                    let clicked = ui
                        .horizontal(|ui| {
                            crate::ui::render_color_dot(ui, dot);
                            ui.selectable_label(is_sel, label).clicked()
                        })
                        .inner;
                    if clicked {
                        app.entry_project_idx = Some(i);
                    }
                }
            });
    });

    ui.add_space(6.0);

    // Row 2: Time tracking (Start, End, Duration)
    ui.horizontal_wrapped(|ui| {
        ui.strong("Start:");
        let start_resp = crate::ui::material::input(ui, &mut app.entry_start, "09:00", 78.0);
        if start_resp.changed() {
            let _ = app.effective_entry_duration_minutes();
        }

        ui.add_space(10.0);
        ui.strong("End:");
        let end_resp = crate::ui::material::input(ui, &mut app.entry_end, "10:30", 78.0);
        if end_resp.changed() {
            let _ = app.effective_entry_duration_minutes();
        }
        if crate::ui::material::chip(ui, "Now").clicked() {
            app.entry_end = crate::domain::now_hhmm();
            let _ = app.effective_entry_duration_minutes();
        }

        ui.add_space(12.0);
        ui.strong("Duration *:");
        crate::ui::material::input(ui, &mut app.entry_duration, "1h 30m", 90.0);
        ui.weak("(e.g. 1.5, 90, 1:30, 45m)");
    });

    ui.add_space(6.0);

    // Row 3: Description
    ui.strong("Description:");
    let desc_width = ui.available_width().max(380.0) - 20.0;
    let desc_resp = crate::ui::material::input(
        ui,
        &mut app.entry_description,
        "What did you work on? (e.g. Research Sharegate migration API)",
        desc_width,
    );
    if app.focus_new_entry {
        desc_resp.request_focus();
        app.focus_new_entry = false;
    }

    // Row 4: Clean Recent Tasks Quick-Fill Chips (Truncated, nicely styled)
    let recents = app.recent_descriptions(4);
    if !recents.is_empty() {
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.weak("⚡ Quick fill:");
            for d in recents {
                let chip_label = if d.chars().count() > 24 {
                    let truncated: String = d.chars().take(22).collect();
                    format!("⚡ {}...", truncated)
                } else {
                    format!("⚡ {}", d)
                };
                if crate::ui::material::chip(ui, chip_label)
                    .on_hover_text(format!("Click to fill description:\n{}", d))
                    .clicked()
                {
                    app.entry_description = d;
                }
            }
        });
    }

    ui.add_space(6.0);

    // Row 5: Billable & Tags
    ui.horizontal_wrapped(|ui| {
        crate::ui::material::switch(ui, &mut app.entry_billable);
        ui.label("Track as billable hours");
        ui.add_space(20.0);
        ui.strong("Tags:");
        crate::ui::material::input(
            ui,
            &mut app.entry_tags,
            "e.g. design, backend, client-review",
            220.0,
        );
    });

    ui.add_space(8.0);

    // Row 6: Action buttons
    ui.horizontal_wrapped(|ui| {
        let label = if app.editing_entry_id.is_some() {
            format!("💾 Update Entry #{} (Enter)", app.editing_entry_id.unwrap())
        } else {
            "➕ Save Entry (Enter)".to_string()
        };
        let save_clicked_now = if app.editing_entry_id.is_some() {
            crate::ui::material::tonal_button(ui, label).clicked()
        } else {
            crate::ui::material::primary_button(ui, label).clicked()
        };
        if save_clicked_now {
            save_clicked = true;
        }

        if crate::ui::material::text_button(ui, "↺ Clear Form (Ctrl+N)").clicked() {
            app.clear_entry_form();
            app.focus_new_entry = true;
        }

        if app.editing_entry_id.is_some()
            && crate::ui::material::text_button(ui, "✖ Cancel Edit (Esc)").clicked()
        {
            app.clear_entry_form();
            app.show_entry_form = false;
        }
    });

    save_clicked
}

pub(crate) fn save_entry_form(app: &mut TimesheetApp) -> bool {
    let Some(mins) = app.effective_entry_duration_minutes() else {
        app.status = "Invalid duration (enter duration or valid start/end times)".to_string();
        return false;
    };
    if mins <= 0 {
        app.status = "Duration must be > 0".to_string();
        return false;
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
        match app.find_overlaps(&e, Some(id)) {
            Ok(ov) if !ov.is_empty() => {
                app.status = format!(
                    "Cannot save: overlaps {} existing entr{} (adjust times)",
                    ov.len(),
                    if ov.len() == 1 { "y" } else { "ies" }
                );
                return false;
            }
            Err(err) => {
                app.status = format!("Overlap check failed: {err}");
                return false;
            }
            _ => {}
        }
        match app.db.update_entry(&e) {
            Ok(_) => {
                app.status = "Entry updated".to_string();
                app.clear_entry_form();
                return true;
            }
            Err(err) => app.status = format!("Update failed: {err}"),
        }
    } else {
        let e = crate::db::models::NewEntry {
            project_id: app.selected_project_id(),
            date: app.entry_date,
            start_time: start_time.clone(),
            end_time: end_time.clone(),
            duration_minutes: mins,
            description: app.entry_description.clone(),
            billable: app.entry_billable,
            tags: app.entry_tags.clone(),
        };
        let candidate = Entry {
            id: -1,
            project_id: e.project_id,
            date: e.date,
            start_time,
            end_time,
            duration_minutes: mins,
            description: e.description.clone(),
            billable: e.billable,
            tags: e.tags.clone(),
        };
        match app.find_overlaps(&candidate, None) {
            Ok(ov) if !ov.is_empty() => {
                app.status = format!(
                    "Cannot save: overlaps {} existing entr{} (adjust times)",
                    ov.len(),
                    if ov.len() == 1 { "y" } else { "ies" }
                );
                return false;
            }
            Err(err) => {
                app.status = format!("Overlap check failed: {err}");
                return false;
            }
            _ => {}
        }
        match app.db.create_entry(&e) {
            Ok(_) => {
                app.status = "Entry created".to_string();
                app.clear_entry_form();
                return true;
            }
            Err(err) => app.status = format!("Create failed: {err}"),
        }
    }
    false
}
