use crate::db::models::Project;
use crate::export::{ExportOptions, csv_export, excel_export, markdown_export, pdf_export};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Export");
    ui.label("Exports use the current shared filters (Entries/Reports). For preview and breakdowns, use the Reports tab.");
    ui.add_space(8.0);

    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    ui.label(format!("Entries to export: {}", entries.len()));
    let projects: HashMap<i64, Project> = app
        .projects_cache
        .iter()
        .cloned()
        .map(|p| (p.id, p))
        .collect();

    let mut group_by_project = false;
    let mut include_summary = true;
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut group_by_project, "Group by project");
        ui.checkbox(&mut include_summary, "Include summary/metadata");
    });

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
    let options = ExportOptions {
        title: "Timesheet Report".to_string(),
        company_name: app.settings.company_name.clone(),
        user_name: app.settings.user_name.clone(),
        period_text,
        filter_text: format!("Project: {}; Billable: {}", project_text, billable_text),
        group_by_project,
        include_summary,
    };

    if ui.button("Export CSV...").clicked() {
        if let Some(path) = app.export_dialog("CSV", "csv", "timesheet.csv").save_file() {
            match csv_export::export_entries(&path, &entries, &projects, &options) {
                Ok(_) => app.status = format!("Exported CSV to {}", path.display()),
                Err(e) => app.status = format!("CSV export failed: {e}"),
            }
        }
    }
    if ui.button("Export Excel (.xlsx)...").clicked() {
        if let Some(path) = app
            .export_dialog("Excel", "xlsx", "timesheet.xlsx")
            .save_file()
        {
            match excel_export::export_entries(&path, &entries, &projects, &options) {
                Ok(_) => app.status = format!("Exported Excel to {}", path.display()),
                Err(e) => app.status = format!("Excel export failed: {e}"),
            }
        }
    }
    if ui.button("Export PDF...").clicked() {
        if let Some(path) = app.export_dialog("PDF", "pdf", "timesheet.pdf").save_file() {
            match pdf_export::export_entries(&path, &entries, &projects, &options) {
                Ok(_) => app.status = format!("Exported PDF to {}", path.display()),
                Err(e) => app.status = format!("PDF export failed: {e}"),
            }
        }
    }
    if ui.button("Export Markdown...").clicked() {
        if let Some(path) = app
            .export_dialog("Markdown", "md", "timesheet.md")
            .save_file()
        {
            match markdown_export::export_entries(&path, &entries, &projects, &options) {
                Ok(_) => app.status = format!("Exported Markdown to {}", path.display()),
                Err(e) => app.status = format!("Markdown export failed: {e}"),
            }
        }
    }
}
