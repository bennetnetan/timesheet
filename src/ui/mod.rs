pub mod app;
pub mod dashboard;
pub mod entries;
pub mod projects;
pub mod reports;
pub mod settings_view;

use eframe::egui;

pub fn project_color_32(color_hex: Option<&str>, default_idx: usize) -> egui::Color32 {
    if let Some(hex) = color_hex {
        if let Some((r, g, b)) = crate::domain::parse_hex_color(hex) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }
    let palette = [
        egui::Color32::from_rgb(59, 130, 246),  // Blue
        egui::Color32::from_rgb(16, 185, 129),  // Emerald
        egui::Color32::from_rgb(139, 92, 246),  // Violet
        egui::Color32::from_rgb(245, 158, 11),  // Amber
        egui::Color32::from_rgb(244, 63, 94),   // Rose
        egui::Color32::from_rgb(6, 182, 212),   // Cyan
        egui::Color32::from_rgb(249, 115, 22),  // Orange
        egui::Color32::from_rgb(100, 116, 139), // Slate
    ];
    palette[default_idx % palette.len()]
}

pub fn render_color_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}
