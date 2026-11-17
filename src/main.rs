#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod db;
mod domain;
mod export;
mod settings;
mod system_theme;
mod timer_persist;
mod tray;
mod ui;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(make_app_icon()),
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

/// Generate a 32×32 RGBA icon matching the tray icon (clock face with teal ring and blue hands).
fn make_app_icon() -> egui::IconData {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    let cx = SIZE as f32 / 2.0;
    let cy = SIZE as f32 / 2.0;
    let outer_r = 15.0_f32;
    let ring_r = 11.0_f32;
    let inner_r = 9.5_f32;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * SIZE + x) * 4) as usize;

            if dist <= outer_r {
                if dist <= inner_r {
                    // Inner face – white/light
                    rgba[idx] = 240;
                    rgba[idx + 1] = 248;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 255;
                } else if dist <= ring_r {
                    // Teal ring
                    rgba[idx] = 20;
                    rgba[idx + 1] = 184;
                    rgba[idx + 2] = 166;
                    rgba[idx + 3] = 255;
                } else {
                    // Outer ring – slightly darker teal
                    rgba[idx] = 15;
                    rgba[idx + 1] = 118;
                    rgba[idx + 2] = 110;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    // 12 o'clock hand
    for y in 4..15u32 {
        let x = SIZE / 2;
        let idx = ((y * SIZE + x) * 4) as usize;
        rgba[idx] = 30;
        rgba[idx + 1] = 64;
        rgba[idx + 2] = 175;
        rgba[idx + 3] = 255;
        if x + 1 < SIZE {
            let idx2 = ((y * SIZE + x + 1) * 4) as usize;
            rgba[idx2] = 30;
            rgba[idx2 + 1] = 64;
            rgba[idx2 + 2] = 175;
            rgba[idx2 + 3] = 255;
        }
    }
    // 3 o'clock hand
    for x in 16..22u32 {
        let y = SIZE / 2;
        let idx = ((y * SIZE + x) * 4) as usize;
        rgba[idx] = 30;
        rgba[idx + 1] = 64;
        rgba[idx + 2] = 175;
        rgba[idx + 3] = 255;
    }

    egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
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
