use crate::db::models::Project;
use crate::export::{csv_export, excel_export, pdf_export};
use crate::ui::app::TimesheetApp;
use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use egui_extras::DatePickerButton;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Reports & Preview");
    ui.label("Configure filters, review summaries and entry preview, then export.");

    // Filters and presets
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Today").clicked() {
                let t = Local::now().date_naive();
                app.filter.from = Some(t);
                app.filter.to = Some(t);
            }
            if ui.button("This week").clicked() {
                let t = Local::now().date_naive();
                let start = t - chrono::Duration::days(t.weekday().num_days_from_monday() as i64);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.button("This month").clicked() {
                let t = Local::now().date_naive();
                let start = NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap_or(t);
                app.filter.from = Some(start);
                app.filter.to = Some(t);
            }
            if ui.button("Last 30 days").clicked() {
                let t = Local::now().date_naive();
                app.filter.from = Some(t - chrono::Duration::days(29));
                app.filter.to = Some(t);
            }
            if ui.button("Clear range").clicked() {
                app.filter.from = None;
                app.filter.to = None;
            }
        });

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
            egui::ComboBox::from_id_salt("report_project")
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
    let mut amount_total = 0.0_f64;

    ui.add_space(6.0);
    ui.group(|ui| {
        ui.heading("Summary");
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

    // Breakdown by project/client
    ui.add_space(6.0);
    ui.collapsing("Breakdown by project/client", |ui| {
        #[derive(Default, Clone)]
        struct Agg {
            name: String,
            client: String,
            minutes: i64,
            billable_minutes: i64,
            amount: f64,
        }
        let mut by_key: HashMap<String, Agg> = HashMap::new();
        for e in &entries {
            let (name, client, rate) = e
                .project_id
                .and_then(|id| projects_by_id.get(&id))
                .map(|p| {
                    (
                        p.name.clone(),
                        p.client.clone().unwrap_or_default(),
                        p.rate.unwrap_or(0.0),
                    )
                })
                .unwrap_or_else(|| ("(No project)".to_string(), String::new(), 0.0));
            let key = format!("{}|{}", name, client);
            let agg = by_key.entry(key).or_insert(Agg {
                name: name.clone(),
                client: client.clone(),
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
        ui.label(format!(
            "Estimated amount (from project rates): {:.2}",
            amount_total
        ));
        egui::Grid::new("breakdown_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Project");
                ui.strong("Client");
                ui.strong("Total");
                ui.strong("Billable");
                ui.strong("Non-billable");
                ui.strong("Amount");
                ui.end_row();
                for r in rows {
                    let nonb = r.minutes - r.billable_minutes;
                    ui.label(&r.name);
                    ui.label(if r.client.is_empty() { "-" } else { &r.client });
                    ui.label(crate::domain::format_minutes(r.minutes));
                    ui.label(crate::domain::format_minutes(r.billable_minutes));
                    ui.label(crate::domain::format_minutes(nonb));
                    ui.label(format!("{:.2}", r.amount));
                    ui.end_row();
                }
            });
    });

    // Export actions (from preview/report)
    ui.add_space(6.0);
    ui.group(|ui| {
        ui.heading("Export (current report)");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Export CSV...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .set_file_name("timesheet_report.csv")
                    .save_file()
                {
                    match csv_export::export_entries(&path, &entries, &projects_by_id) {
                        Ok(_) => app.status = format!("Exported CSV to {}", path.display()),
                        Err(e) => app.status = format!("CSV export failed: {e}"),
                    }
                }
            }
            if ui.button("Export Excel (.xlsx)...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Excel", &["xlsx"])
                    .set_file_name("timesheet_report.xlsx")
                    .save_file()
                {
                    match excel_export::export_entries(&path, &entries, &projects_by_id) {
                        Ok(_) => app.status = format!("Exported Excel to {}", path.display()),
                        Err(e) => app.status = format!("Excel export failed: {e}"),
                    }
                }
            }
            if ui.button("Export PDF...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF", &["pdf"])
                    .set_file_name("timesheet_report.pdf")
                    .save_file()
                {
                    match pdf_export::export_entries(&path, &entries, &projects_by_id) {
                        Ok(_) => app.status = format!("Exported PDF to {}", path.display()),
                        Err(e) => app.status = format!("PDF export failed: {e}"),
                    }
                }
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
                        let (proj, client) = e
                            .project_id
                            .and_then(|id| projects_by_id.get(&id))
                            .map(|p| (p.name.as_str(), p.client.as_deref().unwrap_or("-")))
                            .unwrap_or(("-", "-"));
                        ui.label(e.date.to_string());
                        ui.label(proj);
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
}
