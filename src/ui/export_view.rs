use crate::db::models::Project;
use crate::export::{csv_export, excel_export, pdf_export};
use crate::ui::app::TimesheetApp;
use eframe::egui;
use std::collections::HashMap;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Export");
    ui.label(
        "Exports use the current filters from the Entries tab (date range, project, billable).",
    );
    ui.add_space(8.0);

    let entries = app.db.list_entries(&app.filter).unwrap_or_default();
    ui.label(format!("Entries to export: {}", entries.len()));
    let projects: HashMap<i64, Project> = app
        .projects_cache
        .iter()
        .cloned()
        .map(|p| (p.id, p))
        .collect();

    if ui.button("Export CSV...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name("timesheet.csv")
            .save_file()
        {
            match csv_export::export_entries(&path, &entries, &projects) {
                Ok(_) => app.status = format!("Exported CSV to {}", path.display()),
                Err(e) => app.status = format!("CSV export failed: {e}"),
            }
        }
    }
    if ui.button("Export Excel (.xlsx)...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel", &["xlsx"])
            .set_file_name("timesheet.xlsx")
            .save_file()
        {
            match excel_export::export_entries(&path, &entries, &projects) {
                Ok(_) => app.status = format!("Exported Excel to {}", path.display()),
                Err(e) => app.status = format!("Excel export failed: {e}"),
            }
        }
    }
    if ui.button("Export PDF...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name("timesheet.pdf")
            .save_file()
        {
            match pdf_export::export_entries(&path, &entries, &projects) {
                Ok(_) => app.status = format!("Exported PDF to {}", path.display()),
                Err(e) => app.status = format!("PDF export failed: {e}"),
            }
        }
    }

    ui.add_space(16.0);
    ui.separator();
    ui.label(
        "Tip: set date/project filters in the Entries tab before exporting to limit the report.",
    );
}
