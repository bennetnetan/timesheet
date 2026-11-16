use crate::db::models::Project;
use crate::export::{ExportOptions, csv_export, excel_export, markdown_export, pdf_export};
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use egui_extras::DatePickerButton;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading("Reports & Export");
            ui.label("Configure report filters, review project summaries and entry previews, then export to any format.");

    // Filters and presets
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Presets:");
            if ui.small_button("Today").clicked() {
                let t = Local::now().date_naive();
                app.filter.from = Some(t);
                app.filter.to = Some(t);
            }
            if ui.small_button("Yesterday").clicked() {
                let y = Local::now().date_naive() - chrono::Duration::days(1);
                app.filter.from = Some(y);
                app.filter.to = Some(y);
            }
            if ui.small_button("This week").clicked() {
                let t = Local::now().date_naive();
                let start = t - chrono::Duration::days(t.weekday().num_days_from_monday() as i64);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.small_button("This month").clicked() {
                let t = Local::now().date_naive();
                let start = NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap_or(t);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.small_button("Last 30 days").clicked() {
                let t = Local::now().date_naive();
                app.filter.from = Some(t - chrono::Duration::days(29));
                app.filter.to = Some(t);
            }
            if ui.small_button("Clear range").clicked() {
                app.filter.from = None;
                app.filter.to = None;
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("From:");
            let mut from = app.filter.from.unwrap_or_else(|| Local::now().date_naive());
            let mut use_from = app.filter.from.is_some();
            if ui.checkbox(&mut use_from, "use").changed() {
                app.filter.from = if use_from { Some(from) } else { None };
            }
            if app.filter.from.is_some() && ui.add(DatePickerButton::new(&mut from)).changed() {
                app.filter.from = Some(from);
            }

            ui.separator();
            ui.label("To:");
            let mut to = app.filter.to.unwrap_or_else(|| Local::now().date_naive());
            let mut use_to = app.filter.to.is_some();
            if ui.checkbox(&mut use_to, "use").changed() {
                app.filter.to = if use_to { Some(to) } else { None };
            }
            if app.filter.to.is_some() && ui.add(DatePickerButton::new(&mut to)).changed() {
                app.filter.to = Some(to);
            }

            ui.separator();
            ui.label("Project:");
            let sel_proj_report = app
                .filter
                .project_id
                .and_then(|id| app.projects_cache.iter().position(|p| p.id == id))
                .and_then(|idx| app.projects_cache.get(idx).map(|p| (p, idx)));

            if let Some((p, idx)) = sel_proj_report {
                let dot_color = crate::ui::project_color_32(p.color.as_deref(), idx);
                crate::ui::render_color_dot(ui, dot_color);
            }

            egui::ComboBox::from_id_salt("report_project")
                .selected_text(
                    sel_proj_report
                        .map(|(p, _)| p.name.as_str())
                        .unwrap_or("All"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(app.filter.project_id.is_none(), "All")
                        .clicked()
                    {
                        app.filter.project_id = None;
                    }
                    for (i, p) in app.projects_cache.iter().enumerate() {
                        let dot_color = crate::ui::project_color_32(p.color.as_deref(), i);
                        ui.horizontal(|ui| {
                            crate::ui::render_color_dot(ui, dot_color);
                            if ui
                                .selectable_label(app.filter.project_id == Some(p.id), &p.name)
                                .clicked()
                            {
                                app.filter.project_id = Some(p.id);
                            }
                        });
                    }
                });

            ui.label("Billable:");
            let billable_text = match app.filter.billable {
                None => "All",
                Some(true) => "Billable",
                Some(false) => "Non-billable",
            };
            egui::ComboBox::from_id_salt("report_billable")
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
        });
    });

    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    let projects_by_id: HashMap<i64, Project> = app
        .projects_cache
        .iter()
        .cloned()
        .map(|p| (p.id, p))
        .collect();

    // Summary
    let total_minutes: i64 = entries.iter().map(|e| e.duration_minutes).sum();
    let billable_minutes: i64 = entries
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.duration_minutes)
        .sum();
    let non_billable_minutes = total_minutes - billable_minutes;

    ui.add_space(6.0);
    ui.group(|ui| {
        ui.heading("Report Summary");
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Entries: {}", entries.len()));
            ui.separator();
            ui.label(format!(
                "Total: {} ({:.2} h)",
                crate::domain::format_minutes(total_minutes),
                total_minutes as f64 / 60.0
            ));
            ui.separator();
            ui.label(format!(
                "Billable: {} ({:.2} h)",
                crate::domain::format_minutes(billable_minutes),
                billable_minutes as f64 / 60.0
            ));
            ui.separator();
            ui.label(format!(
                "Non-billable: {} ({:.2} h)",
                crate::domain::format_minutes(non_billable_minutes),
                non_billable_minutes as f64 / 60.0
            ));
        });
    });

    // Aggregate by project/client
    #[derive(Default, Clone)]
    struct Agg {
        name: String,
        client: String,
        color: Option<String>,
        minutes: i64,
        billable_minutes: i64,
        amount: f64,
        budget_hours: Option<f64>,
    }
    let mut by_key: HashMap<String, Agg> = HashMap::new();
    let mut amount_total = 0.0_f64;
    for e in &entries {
        let (name, client, rate, color, budget_hours) = e
            .project_id
            .and_then(|id| projects_by_id.get(&id))
            .map(|p| {
                (
                    p.name.clone(),
                    p.client.clone().unwrap_or_default(),
                    p.rate.unwrap_or(0.0),
                    p.color.clone(),
                    p.budget_hours,
                )
            })
            .unwrap_or_else(|| ("(No project)".to_string(), String::new(), 0.0, None, None));
        let key = format!("{}|{}", name, client);
        let agg = by_key.entry(key).or_insert(Agg {
            name: name.clone(),
            client: client.clone(),
            color,
            budget_hours,
            ..Default::default()
        });
        agg.minutes += e.duration_minutes;
        if e.billable {
            agg.billable_minutes += e.duration_minutes;
        }
        let amt = (e.duration_minutes as f64 / 60.0) * rate;
        agg.amount += amt;
        amount_total += amt;
    }
    let mut rows: Vec<Agg> = by_key.into_values().collect();
    rows.sort_by(|a, b| b.minutes.cmp(&a.minutes).then(a.name.cmp(&b.name)));

    // Project distribution bar
    if total_minutes > 0 {
        ui.add_space(6.0);
        ui.group(|ui| {
            ui.heading("Project Distribution");
            ui.add_space(4.0);

            let (bar_rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::hover());
            ui.painter().rect_filled(
                bar_rect,
                4.0,
                if app.settings.dark_theme {
                    egui::Color32::from_gray(40)
                } else {
                    egui::Color32::from_gray(220)
                },
            );

            let mut current_x = bar_rect.min.x;
            for (idx, r) in rows.iter().enumerate() {
                if r.minutes == 0 {
                    continue;
                }
                let ratio = r.minutes as f32 / total_minutes as f32;
                let seg_width = (bar_rect.width() * ratio).max(2.0);
                let seg_rect = egui::Rect::from_min_max(
                    egui::pos2(current_x, bar_rect.min.y),
                    egui::pos2((current_x + seg_width).min(bar_rect.max.x), bar_rect.max.y),
                );
                let dot_color = crate::ui::project_color_32(r.color.as_deref(), idx);
                ui.painter().rect_filled(seg_rect, 2.0, dot_color);
                current_x += seg_width;
            }

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for (idx, r) in rows.iter().enumerate() {
                    if r.minutes == 0 {
                        continue;
                    }
                    let ratio = (r.minutes as f64 / total_minutes as f64) * 100.0;
                    let dot_color = crate::ui::project_color_32(r.color.as_deref(), idx);
                    crate::ui::render_color_dot(ui, dot_color);
                    ui.label(format!(
                        "{} {:.1}% ({})",
                        r.name,
                        ratio,
                        crate::domain::format_minutes(r.minutes)
                    ));
                    ui.add_space(8.0);
                }
            });
        });
    }

    // Breakdown by project/client
    ui.add_space(6.0);
    ui.collapsing("Breakdown by project/client", |ui| {
        ui.label(format!(
            "Estimated total amount (from project rates): ${:.2}",
            amount_total
        ));
        egui::Grid::new("breakdown_grid")
            .striped(true)
            .min_col_width(50.0)
            .show(ui, |ui| {
                ui.strong("Project");
                ui.strong("Client");
                ui.strong("Total");
                ui.strong("Billable");
                ui.strong("Non-billable");
                ui.strong("Budget Progress");
                ui.strong("Amount");
                ui.end_row();
                for (idx, r) in rows.iter().enumerate() {
                    let nonb = r.minutes - r.billable_minutes;
                    let dot_color = crate::ui::project_color_32(r.color.as_deref(), idx);
                    ui.horizontal(|ui| {
                        crate::ui::render_color_dot(ui, dot_color);
                        ui.label(&r.name);
                    });
                    ui.label(if r.client.is_empty() { "-" } else { &r.client });
                    ui.label(crate::domain::format_minutes(r.minutes));
                    ui.label(crate::domain::format_minutes(r.billable_minutes));
                    ui.label(crate::domain::format_minutes(nonb));

                    if let Some(bh) = r.budget_hours {
                        let spent_h = r.minutes as f64 / 60.0;
                        let pct = if bh > 0.0 {
                            (spent_h / bh) * 100.0
                        } else {
                            0.0
                        };
                        let color = if pct > 100.0 {
                            egui::Color32::from_rgb(239, 68, 68)
                        } else if pct >= 80.0 {
                            egui::Color32::from_rgb(245, 158, 11)
                        } else {
                            egui::Color32::from_rgb(16, 185, 129)
                        };
                        ui.label(
                            egui::RichText::new(format!("{:.1}/{:.0}h ({:.0}%)", spent_h, bh, pct))
                                .color(color),
                        );
                    } else {
                        ui.label(egui::RichText::new("-").weak());
                    }

                    ui.label(format!("${:.2}", r.amount));
                    ui.end_row();
                }
            });
    });

    // Export actions
    ui.add_space(6.0);
    ui.group(|ui| {
        ui.heading("Export Options");
        let mut group_by_project = true;
        let mut include_summary = true;
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut group_by_project, "Group by project");
            ui.checkbox(&mut include_summary, "Include summary & metadata");
        });
        let mut options = build_export_options(app, group_by_project);
        options.include_summary = include_summary;

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(egui::RichText::new("Export CSV...").strong())
                .clicked()
            {
                if let Some(path) = app
                    .export_dialog("CSV", "csv", "timesheet_report.csv")
                    .save_file()
                {
                    match csv_export::export_entries(&path, &entries, &projects_by_id, &options) {
                        Ok(_) => app.status = format!("Exported CSV to {}", path.display()),
                        Err(e) => app.status = format!("CSV export failed: {e}"),
                    }
                }
            }
            if ui
                .button(egui::RichText::new("Export Excel (.xlsx)...").strong())
                .clicked()
            {
                if let Some(path) = app
                    .export_dialog("Excel", "xlsx", "timesheet_report.xlsx")
                    .save_file()
                {
                    match excel_export::export_entries(&path, &entries, &projects_by_id, &options) {
                        Ok(_) => app.status = format!("Exported Excel to {}", path.display()),
                        Err(e) => app.status = format!("Excel export failed: {e}"),
                    }
                }
            }
            if ui
                .button(egui::RichText::new("Export PDF...").strong())
                .clicked()
            {
                if let Some(path) = app
                    .export_dialog("PDF", "pdf", "timesheet_report.pdf")
                    .save_file()
                {
                    match pdf_export::export_entries(&path, &entries, &projects_by_id, &options) {
                        Ok(_) => app.status = format!("Exported PDF to {}", path.display()),
                        Err(e) => app.status = format!("PDF export failed: {e}"),
                    }
                }
            }
            if ui
                .button(egui::RichText::new("Export Markdown...").strong())
                .clicked()
            {
                if let Some(path) = app
                    .export_dialog("Markdown", "md", "timesheet_report.md")
                    .save_file()
                {
                    match markdown_export::export_entries(
                        &path,
                        &entries,
                        &projects_by_id,
                        &options,
                    ) {
                        Ok(_) => app.status = format!("Exported Markdown to {}", path.display()),
                        Err(e) => app.status = format!("Markdown export failed: {e}"),
                    }
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick Clipboard:");
            if ui
                .button("📋 Copy Summary")
                .on_hover_text("Copy report summary metrics to clipboard")
                .clicked()
            {
                let summary_text = format!(
                    "Timesheet Report Summary\nPeriod: {}\nTotal Time: {} ({:.2}h)\nBillable Time: {} ({:.2}h)\nNon-billable: {} ({:.2}h)\nEntries: {}",
                    options.period_text,
                    crate::domain::format_minutes(total_minutes),
                    total_minutes as f64 / 60.0,
                    crate::domain::format_minutes(billable_minutes),
                    billable_minutes as f64 / 60.0,
                    crate::domain::format_minutes(non_billable_minutes),
                    non_billable_minutes as f64 / 60.0,
                    entries.len()
                );
                ui.ctx().copy_text(summary_text);
                app.status = "Report summary copied to clipboard".to_string();
            }

            if ui
                .button("📋 Copy Markdown")
                .on_hover_text("Copy formatted Markdown report to clipboard")
                .clicked()
            {
                let mut md = String::new();
                md.push_str(&format!("# {}\n\n", options.title));
                md.push_str(&format!("- **Period:** {}\n", options.period_text));
                md.push_str(&format!(
                    "- **Total:** {} ({:.2}h)\n",
                    crate::domain::format_minutes(total_minutes),
                    total_minutes as f64 / 60.0
                ));
                md.push_str(&format!(
                    "- **Billable:** {} ({:.2}h)\n\n",
                    crate::domain::format_minutes(billable_minutes),
                    billable_minutes as f64 / 60.0
                ));
                md.push_str("| Date | Project | Duration | Billable | Description |\n");
                md.push_str("| --- | --- | --- | --- | --- |\n");
                for e in &entries {
                    let proj_name = e
                        .project_id
                        .and_then(|id| projects_by_id.get(&id))
                        .map(|p| p.name.as_str())
                        .unwrap_or("-");
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        e.date,
                        proj_name,
                        crate::domain::format_minutes(e.duration_minutes),
                        if e.billable { "Yes" } else { "No" },
                        e.description.replace('|', "\\|")
                    ));
                }
                ui.ctx().copy_text(md);
                app.status = "Markdown report copied to clipboard".to_string();
            }
        });
    });

    // Preview
    ui.add_space(6.0);
    ui.heading("Entry Preview");
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .max_height(260.0)
        .show(ui, |ui| {
            egui::Grid::new("report_preview")
                .striped(true)
                .min_col_width(50.0)
                .show(ui, |ui| {
                    ui.strong("Date");
                    ui.strong("Project");
                    ui.strong("Client");
                    ui.strong("Start");
                    ui.strong("End");
                    ui.strong("Duration");
                    ui.strong("Billable");
                    ui.strong("Description");
                    ui.end_row();
                    for e in &entries {
                        let proj = e.project_id.and_then(|id| projects_by_id.get(&id));
                        let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("-");
                        let client = proj.and_then(|p| p.client.as_deref()).unwrap_or("-");
                        let dot_color =
                            crate::ui::project_color_32(proj.and_then(|p| p.color.as_deref()), 0);

                        ui.label(e.date.to_string());
                        ui.horizontal(|ui| {
                            if e.project_id.is_some() {
                                crate::ui::render_color_dot(ui, dot_color);
                            }
                            ui.label(proj_name);
                        });
                        ui.label(client);
                        ui.label(e.start_time.as_deref().unwrap_or("-"));
                        ui.label(e.end_time.as_deref().unwrap_or("-"));
                        ui.label(crate::domain::format_minutes(e.duration_minutes));
                        ui.label(if e.billable { "Yes" } else { "No" });
                        ui.label(&e.description);
                        ui.end_row();
                    }
                });
        });

            ui.add_space(16.0);
        });
}

fn build_export_options(app: &TimesheetApp, group_by_project: bool) -> ExportOptions {
    let period_text = match (app.filter.from, app.filter.to) {
        (Some(f), Some(t)) if f == t => f.to_string(),
        (Some(f), Some(t)) => format!("{} to {}", f, t),
        (Some(f), None) => format!("From {}", f),
        (None, Some(t)) => format!("Up to {}", t),
        (None, None) => "All dates".to_string(),
    };
    let project_text = app
        .filter
        .project_id
        .and_then(|id| app.projects_cache.iter().find(|p| p.id == id))
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "All projects".to_string());
    let billable_text = match app.filter.billable {
        None => "All",
        Some(true) => "Billable only",
        Some(false) => "Non-billable only",
    };
    ExportOptions {
        title: "Timesheet Report".to_string(),
        company_name: app.settings.company_name.clone(),
        user_name: app.settings.user_name.clone(),
        period_text,
        filter_text: format!("Project: {}; Billable: {}", project_text, billable_text),
        group_by_project,
        include_summary: true,
    }
}
