use crate::db::Database;
use crate::ui::app::{ConfirmAction, ConfirmState, TimesheetApp};
use eframe::egui;

pub fn show(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            crate::ui::centered_content(ui, |ui| {
                ui.label("Preferences are stored offline in your Windows AppData directory.");
                ui.add_space(6.0);

                crate::ui::material::tabs(
                    ui,
                    "settings_tabs",
                    &mut app.settings_tab,
                    &["Preferences", "Data & Backup", "About"],
                );
                ui.add_space(12.0);

                match app.settings_tab {
                    0 => {
                        // Preferences: profile form + shortcut reference, two columns when wide.
                        let avail_w = ui.available_width();
                        if avail_w > 900.0 {
                            let left_w = (avail_w * 0.56).max(420.0);
                            let right_w = avail_w - left_w - 16.0;
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(left_w, ui.available_height()),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        render_profile_card(app, ui);
                                    },
                                );
                                ui.add_space(16.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(right_w, ui.available_height()),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        render_shortcuts_card(app, ui);
                                    },
                                );
                            });
                        } else {
                            render_profile_card(app, ui);
                            ui.add_space(12.0);
                            render_shortcuts_card(app, ui);
                        }
                    }
                    1 => render_backup_card(app, ui),
                    _ => render_about_card(app, ui),
                }

                ui.add_space(20.0);
            });
        });
}

fn render_profile_card(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(
        ui,
        app.settings.dark_theme,
        "Profile & Tracking Preferences",
        "⚙",
        |ui| {
            egui::Grid::new("settings_profile")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Company name");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.settings.company_name)
                            .hint_text("e.g. Acme Studio, Freelance")
                            .desired_width(ui.available_width().max(200.0)),
                    );
                    ui.end_row();

                    ui.strong("Your name");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.settings.user_name)
                            .hint_text("e.g. Alex Smith")
                            .desired_width(ui.available_width().max(200.0)),
                    );
                    ui.end_row();

                    ui.strong("Theme");
                    ui.checkbox(&mut app.settings.dark_theme, "Dark theme");
                    ui.end_row();

                    ui.strong("Duration rounding");
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
                            ui.selectable_value(
                                &mut app.settings.round_minutes,
                                5,
                                "Nearest 5 min",
                            );
                            ui.selectable_value(
                                &mut app.settings.round_minutes,
                                15,
                                "Nearest 15 min",
                            );
                            ui.selectable_value(
                                &mut app.settings.round_minutes,
                                30,
                                "Nearest 30 min",
                            );
                        });
                    ui.end_row();

                    ui.strong("Daily target hours");
                    ui.horizontal_wrapped(|ui| {
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

                    ui.strong("Pomodoro Focus");
                    ui.horizontal_wrapped(|ui| {
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
                        if ui.small_button("50m").clicked() {
                            app.settings.pomodoro_focus_mins = 50;
                        }
                    });
                    ui.end_row();

                    ui.strong("Pomodoro Break");
                    ui.horizontal_wrapped(|ui| {
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

                    ui.strong("Default export folder");
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.settings.default_export_dir)
                                .hint_text("Choose a folder for reports")
                                .desired_width(180.0),
                        );
                        if ui.small_button("Browse...").clicked() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                app.settings.default_export_dir = dir.display().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.strong("System tray");
                    ui.checkbox(
                        &mut app.settings.minimize_to_tray,
                        "Show icon in tray & minimize to tray on close",
                    );
                    ui.end_row();
                });

            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("💾 Save preferences").strong())
                .clicked()
            {
                app.save_settings();
            }
        },
    );
}

fn render_backup_card(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(ui, app.settings.dark_theme, "Data & Backup", "💾", |ui| {
        if let Ok(path) = Database::default_path() {
            ui.label(format!("Database: {}", path.display()));
        }
        if let Ok(dir) = crate::settings::data_dir() {
            ui.label(format!("App data folder: {}", dir.display()));
        }
        if let Ok((entries, projects)) = app.db.counts() {
            ui.label(format!(
                "Records: {} entries, {} projects",
                entries, projects
            ));
        }
        ui.add_space(6.0);
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
        ui.weak("Tip: backups are a single .db file. Restoring overwrites the live database; restart the app after restoring.");
    });
}

fn render_shortcuts_card(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(
        ui,
        app.settings.dark_theme,
        "Keyboard Shortcuts & Syntax Reference",
        "⌨",
        |ui| {
            egui::Grid::new("settings_shortcuts")
                .num_columns(2)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("Shortcut");
                    ui.strong("Action");
                    ui.end_row();

                    ui.label("Ctrl+1 .. Ctrl+5");
                    ui.label("Switch between tabs");
                    ui.end_row();

                    ui.label("Ctrl+Space");
                    ui.label("Toggle timer (Start or Stop & Save)");
                    ui.end_row();

                    ui.label("Ctrl+M");
                    ui.label("Toggle floating Mini-Timer widget");
                    ui.end_row();

                    ui.label("Ctrl+N");
                    ui.label("Focus new entry creation form");
                    ui.end_row();

                    ui.label("Enter");
                    ui.label("Save current entry form");
                    ui.end_row();

                    ui.label("Esc");
                    ui.label("Cancel editing / clear selection");
                    ui.end_row();

                    ui.label("Delete");
                    ui.label("Delete selected entries");
                    ui.end_row();

                    ui.strong("Duration Syntax");
                    ui.strong("Examples");
                    ui.end_row();

                    ui.label("Supported formats");
                    ui.label("90, 1.5, 1:30, 1h30m, 45m, 2h");
                    ui.end_row();
                });
        },
    );
}

fn render_about_card(app: &mut TimesheetApp, ui: &mut egui::Ui) {
    crate::ui::render_form_card(ui, app.settings.dark_theme, "About Timesheet", "ℹ", |ui| {
        ui.strong(format!("Timesheet v{}", env!("CARGO_PKG_VERSION")));
        ui.label("Offline-first, single-user timesheet app for Windows.");
        ui.label("Local SQLite storage, CSV/Excel/PDF/Markdown exports, timer, and reports.");
    });
}
