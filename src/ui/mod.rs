pub mod app;
pub mod dashboard;
pub mod entries;
pub mod projects;
pub mod reports;
pub mod settings_view;
pub mod weekly_grid;

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

/// Renders an eye-catching, highly identifiable form container with a distinct border,
/// elevated background, and styled header pill. This ensures users immediately spot inputs and forms.
pub fn render_form_card<R>(
    ui: &mut egui::Ui,
    dark_theme: bool,
    title: &str,
    icon: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let card_bg = if dark_theme {
        egui::Color32::from_rgb(22, 30, 46) // Elevated Slate-850
    } else {
        egui::Color32::from_rgb(255, 255, 255) // Pure White
    };

    let border_stroke = if dark_theme {
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)) // Crisp Blue border
    } else {
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(96, 165, 250)) // Soft Blue border
    };

    egui::Frame::canvas(ui.style())
        .fill(card_bg)
        .stroke(border_stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(16.0));
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(14.0)
                        .color(if dark_theme {
                            egui::Color32::from_rgb(226, 232, 240)
                        } else {
                            egui::Color32::from_rgb(15, 23, 42)
                        }),
                );
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            add_contents(ui)
        })
        .inner
}
