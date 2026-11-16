use crate::db::Database;
use crate::ui::app::{ConfirmAction, ConfirmState, TimesheetApp};
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.label("Preferences are stored offline in your Windows AppData directory.");

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.heading("Profile & Tracking Preferences");
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

                ui.label("Duration auto-rounding");
                let rounding_text = match app.settings.round_minutes {
                    0 => "None (exact)",
                    5 => "Nearest 5 min",
                    15 => "Nearest 15 min",
                    30 => "Nearest 30 min",
                    _ => "Custom",
                };
                egui::ComboBox::from_id_salt("settings_rounding")
                    .selected_text(rounding_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.settings.round_minutes, 0, "None (exact)");
                        ui.selectable_value(&mut app.settings.round_minutes, 5, "Nearest 5 min");
                        ui.selectable_value(&mut app.settings.round_minutes, 15, "Nearest 15 min");
                        ui.selectable_value(&mut app.settings.round_minutes, 30, "Nearest 30 min");
                    });
                ui.end_row();

                ui.label("Daily target hours");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut app.settings.daily_target_hours)
                            .speed(0.25)
                            .range(1.0..=16.0)
                            .suffix(" hrs"),
                    );
                    if ui.small_button("4h").clicked() {
                        app.settings.daily_target_hours = 4.0;
                    }
                    if ui.small_button("6h").clicked() {
                        app.settings.daily_target_hours = 6.0;
                    }
                    if ui.small_button("7.5h").clicked() {
                        app.settings.daily_target_hours = 7.5;
                    }
                    if ui.small_button("8h").clicked() {
                        app.settings.daily_target_hours = 8.0;
                    }
                });
                ui.end_row();

                ui.label("Pomodoro Focus duration");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut app.settings.pomodoro_focus_mins)
                            .range(5..=120)
                            .suffix(" min"),
                    );
                    if ui.small_button("20m").clicked() {
                        app.settings.pomodoro_focus_mins = 20;
                    }
                    if ui.small_button("25m").clicked() {
                        app.settings.pomodoro_focus_mins = 25;
                    }
                    if ui.small_button("30m").clicked() {
                        app.settings.pomodoro_focus_mins = 30;
                    }
                    if ui.small_button("45m").clicked() {
                        app.settings.pomodoro_focus_mins = 45;
                    }
                    if ui.small_button("50m").clicked() {
                        app.settings.pomodoro_focus_mins = 50;
                    }
                });
                ui.end_row();

                ui.label("Pomodoro Break duration");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut app.settings.pomodoro_break_mins)
                            .range(1..=60)
                            .suffix(" min"),
                    );
                    if ui.small_button("5m").clicked() {
                        app.settings.pomodoro_break_mins = 5;
                    }
                    if ui.small_button("10m").clicked() {
                        app.settings.pomodoro_break_mins = 10;
                    }
                    if ui.small_button("15m").clicked() {
                        app.settings.pomodoro_break_mins = 15;
                    }
                });
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

                ui.label("System tray");
                ui.checkbox(
                    &mut app.settings.minimize_to_tray,
                    "Show icon in system tray and hide to tray on close",
                );
                ui.end_row();
            });
        ui.add_space(4.0);
        if ui.button("Save preferences").clicked() {
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
        ui.heading("Keyboard Shortcuts & Syntax Reference");
        egui::Grid::new("settings_shortcuts")
            .num_columns(2)
            .spacing([30.0, 6.0])
            .show(ui, |ui| {
                ui.strong("Shortcut");
                ui.strong("Action");
                ui.end_row();

                ui.label("Ctrl+1 .. Ctrl+5");
                ui.label("Switch between tabs (Dashboard, Entries, Projects, Reports, Settings)");
                ui.end_row();

                ui.label("Ctrl+Space");
                ui.label("Toggle timer (Start or Stop & Save)");
                ui.end_row();

                ui.label("Ctrl+N");
                ui.label("Focus new entry creation form");
                ui.end_row();

                ui.label("Enter");
                ui.label("Save current entry form (when editing)");
                ui.end_row();

                ui.label("Esc");
                ui.label("Cancel editing / clear selection");
                ui.end_row();

                ui.label("Delete");
                ui.label("Delete selected entries in Entries view");
                ui.end_row();

                ui.strong("Duration Syntax");
                ui.strong("Examples");
                ui.end_row();

                ui.label("Supported formats");
                ui.label("90, 1.5, 1:30, 1h30m, 45m, 2h");
                ui.end_row();
            });
    });

    ui.add_space(10.0);
    ui.group(|ui| {
        ui.heading("About");
        ui.label(format!("Timesheet v{}", env!("CARGO_PKG_VERSION")));
        ui.label("Offline-first, single-user timesheet app for Windows.");
        ui.label("Local SQLite storage, CSV/Excel/PDF/Markdown exports, timer, and reports.");
    });
}
