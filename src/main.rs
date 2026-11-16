#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod domain;
mod export;
mod ui;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Timesheet",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            match ui::app::TimesheetApp::new(cc) {
                Ok(app) => Ok(Box::new(app)),
                Err(e) => {
                    // fallback app showing error
                    Ok(Box::new(ErrorApp(format!(
                        "Failed to initialize application: {e:#}"
                    ))))
                }
            }
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(())
}

struct ErrorApp(String);
impl eframe::App for ErrorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Timesheet - Initialization Error");
            ui.colored_label(egui::Color32::RED, &self.0);
        });
    }
}
