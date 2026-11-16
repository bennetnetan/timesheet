use crate::db::Database;
use crate::ui::app::{ConfirmAction, ConfirmState, TimesheetApp};
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.label("Preferences are stored offline in your Windows AppData directory.");

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.heading("Profile / Report Metadata");
        egui::Grid::new("settings_profile")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                ui.label("Company name");
                ui.text_edit_singleline(&mut app.settings.company_name);
                ui.end_row();
                ui.label("Your name");
                ui.text_edit_singleline(&mut app.settings.user_name);
                ui.end_row();
                ui.label("Theme");
                ui.checkbox(&mut app.settings.dark_theme, "Dark theme");
                ui.end_row();
                ui.label("Default export folder");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut app.settings.default_export_dir);
                    if ui.small_button("Browse...").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            app.settings.default_export_dir = dir.display().to_string();
                        }
                    }
                });
                ui.end_row();
            });
        if ui.button("Save settings").clicked() {
            app.save_settings();
        }
    });

    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("Data & Backup");
        if let Ok(path) = Database::default_path() {
            ui.label(format!("Database: {}", path.display()));
        }
        if let Ok(dir) = crate::settings::data_dir() {
            ui.label(format!("App data folder: {}", dir.display()));
        }
        if let Ok((entries, projects)) = app.db.counts() {
            ui.label(format!("Records: {} entries, {} projects", entries, projects));
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Backup database...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("SQLite database", &["db"])
                    .set_file_name(format!("timesheet-backup-{}.db", chrono::Local::now().format("%Y%m%d-%H%M%S")))
                    .save_file()
                {
                    match crate::backup::backup_database(&app.db, &path) {
                        Ok(_) => app.status = format!("Backup created at {}", path.display()),
                        Err(e) => app.status = format!("Backup failed: {e}"),
                    }
                }
            }
            if ui.button("Restore from backup...").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("SQLite database", &["db"]).pick_file() {
                    app.confirm = Some(ConfirmState {
                        title: "Restore Backup".to_string(),
                        message: format!("Replace the current database with {}? The app should be restarted afterwards.", path.display()),
                        action: ConfirmAction::RestoreBackup,
                        backup_path: Some(path),
                    });
                }
            }
            if ui.button("Open data folder").clicked() {
                if let Ok(dir) = crate::settings::data_dir() {
                    let _ = std::process::Command::new("explorer").arg(dir).spawn();
                }
            }
        });
        ui.add_space(4.0);
        ui.label("Tip: backups are a single .db file. Restoring overwrites the live database; restart the app after restoring.");
    });

    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("About");
        ui.label(format!("Timesheet v{}", env!("CARGO_PKG_VERSION")));
        ui.label("Offline-first, single-user timesheet app for Windows.");
        ui.label("Local SQLite storage, CSV/Excel/PDF/Markdown exports, timer, and reports.");
    });
}
