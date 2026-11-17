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

/// Wraps `add_contents` in a modal dialog: scrim, focus trap, 28 dp radius
/// surface, 24 dp padding, a title header with a close button, and a scrolling
/// body capped at ~62% of the window height.
///
/// Returns the closure's value plus whether the dialog asked to close (✕, Esc,
/// or a backdrop click).
pub fn show_dialog<R>(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    icon: &str,
    tokens: Tokens,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> (R, bool) {
    let mut close_requested = false;
    let modal = egui::Modal::new(egui::Id::new(id))
        .frame(theme::dialog_frame(&tokens))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(20.0));
                ui.label(
                    egui::RichText::new(title)
                        .strong()
                        .size(18.0)
                        .color(tokens.on_surface),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("✕").frame(false))
                        .on_hover_text("Close (Esc)")
                        .clicked()
                    {
                        close_requested = true;
                    }
                });
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let max_height = (ctx.content_rect().height() * 0.62).max(220.0);
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .auto_shrink([false, true])
                .show(ui, add_contents)
                .inner
        });

    let should_close = modal.should_close();
    (modal.inner, close_requested || should_close)
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
