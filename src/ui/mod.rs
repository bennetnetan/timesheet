pub mod app;
pub mod dashboard;
pub mod entries;
pub mod material;
pub mod projects;
pub mod reports;
pub mod settings_view;
pub mod theme;
pub mod weekly_grid;

use crate::ui::theme::Tokens;
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
/// Maximum content width used across all pages to prevent extreme stretching on
/// ultra-wide / multi-monitor setups.  Content is centered within this cap.
pub const MAX_CONTENT_WIDTH: f32 = 1400.0;

/// Wraps `add_contents` inside a centered, max-width-capped sub-UI.
/// Call this at the top level of a page's scroll area.
pub fn centered_content<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let avail = ui.available_width();
    let content_w = avail.min(MAX_CONTENT_WIDTH);
    let margin = ((avail - content_w) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        if margin > 0.0 {
            ui.add_space(margin);
        }
        ui.allocate_ui_with_layout(
            egui::vec2(content_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            add_contents,
        )
        .inner
    })
    .inner
}

pub fn render_form_card<R>(
    ui: &mut egui::Ui,
    tokens: Tokens,
    title: &str,
    icon: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    crate::ui::theme::card_frame(&tokens)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icon pill badge
                egui::Frame::new()
                    .fill(tokens.primary_container)
                    .stroke(egui::Stroke::new(1.0_f32, tokens.primary))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(6, 3))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(icon)
                                .size(13.0)
                                .color(tokens.on_primary_container),
                        );
                    });

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(13.5)
                        .color(tokens.on_surface),
                );
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(8.0);
            add_contents(ui)
        })
        .inner
}
