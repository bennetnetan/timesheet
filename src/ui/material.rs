//! Material Design 3 integration helpers.
//!
//! The app uses [`egui_material3`] for a handful of interactive components
//! (tabs, buttons, switches, chips).  That crate keeps its palette in a global
//! theme, so we push a blue/slate scheme matching the rest of the app whenever
//! the light/dark preference changes.

use eframe::egui;
use egui_material3::{
    ContrastLevel, MaterialButton, MaterialCheckbox, MaterialChip, MaterialSwitch, MaterialTabs,
    ThemeMode, get_global_theme,
};

/// Push the app colour scheme into egui-material3's global theme.
///
/// Material components read colours from this theme on every frame, so this
/// must be called from `TimesheetApp::apply_theme`.
pub fn apply_theme(dark: bool) {
    let global_theme = get_global_theme();
    let Ok(mut theme) = global_theme.lock() else {
        return;
    };

    theme.theme_mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    theme.contrast_level = ContrastLevel::Normal;
    theme.selected_colors.clear();

    let palette: &[(&str, &str)] = if dark {
        const DARK: &[(&str, &str)] = &[
            ("primary", "#60A5FA"),
            ("onPrimary", "#0B1220"),
            ("primaryContainer", "#1E3A8A"),
            ("onPrimaryContainer", "#DBEAFE"),
            ("secondary", "#818CF8"),
            ("onSecondary", "#111827"),
            ("secondaryContainer", "#312E81"),
            ("onSecondaryContainer", "#E0E7FF"),
            ("tertiary", "#2DD4BF"),
            ("onTertiary", "#042F2E"),
            ("tertiaryContainer", "#115E59"),
            ("onTertiaryContainer", "#CCFBF1"),
            ("error", "#F87171"),
            ("onError", "#450A0A"),
            ("errorContainer", "#7F1D1D"),
            ("onErrorContainer", "#FEE2E2"),
            ("surface", "#0F1420"),
            ("onSurface", "#E2E8F0"),
            ("surfaceVariant", "#1E293B"),
            ("onSurfaceVariant", "#94A3B8"),
            ("surfaceContainerLowest", "#0B101B"),
            ("surfaceContainerLow", "#121A28"),
            ("surfaceContainer", "#182030"),
            ("surfaceContainerHigh", "#1E293B"),
            ("surfaceContainerHighest", "#273449"),
            ("surfaceTint", "#60A5FA"),
            ("background", "#0F1420"),
            ("onBackground", "#E2E8F0"),
            ("outline", "#64748B"),
            ("outlineVariant", "#334155"),
            ("inverseSurface", "#E2E8F0"),
            ("inverseOnSurface", "#0F172A"),
            ("shadow", "#000000"),
            ("scrim", "#000000"),
        ];
        DARK
    } else {
        const LIGHT: &[(&str, &str)] = &[
            ("primary", "#2563EB"),
            ("onPrimary", "#FFFFFF"),
            ("primaryContainer", "#DBEAFE"),
            ("onPrimaryContainer", "#1E3A8A"),
            ("secondary", "#4F46E5"),
            ("onSecondary", "#FFFFFF"),
            ("secondaryContainer", "#E0E7FF"),
            ("onSecondaryContainer", "#312E81"),
            ("tertiary", "#0D9488"),
            ("onTertiary", "#FFFFFF"),
            ("tertiaryContainer", "#CCFBF1"),
            ("onTertiaryContainer", "#134E4A"),
            ("error", "#DC2626"),
            ("onError", "#FFFFFF"),
            ("errorContainer", "#FEE2E2"),
            ("onErrorContainer", "#7F1D1D"),
            ("surface", "#FFFFFF"),
            ("onSurface", "#0F172A"),
            ("surfaceVariant", "#F1F5F9"),
            ("onSurfaceVariant", "#475569"),
            ("surfaceContainerLowest", "#FFFFFF"),
            ("surfaceContainerLow", "#F8FAFC"),
            ("surfaceContainer", "#F8FAFC"),
            ("surfaceContainerHigh", "#F1F5F9"),
            ("surfaceContainerHighest", "#E2E8F0"),
            ("surfaceTint", "#2563EB"),
            ("background", "#F8FAFC"),
            ("onBackground", "#0F172A"),
            ("outline", "#94A3B8"),
            ("outlineVariant", "#E2E8F0"),
            ("inverseSurface", "#1E293B"),
            ("inverseOnSurface", "#F8FAFC"),
            ("shadow", "#000000"),
            ("scrim", "#000000"),
        ];
        LIGHT
    };

    for (name, hex) in palette {
        if let Some(color) = egui_material3::theme::MaterialThemeContext::hex_to_color32(hex) {
            theme.selected_colors.insert((*name).to_string(), color);
        }
    }
}

/// Material 3 secondary tab bar. Returns `true` when the selection changed.
pub fn tabs(ui: &mut egui::Ui, id_salt: &str, selected: &mut usize, labels: &[&str]) -> bool {
    let mut widget = MaterialTabs::secondary(selected)
        .id_salt(id_salt.to_string())
        .height(44.0);
    for label in labels {
        widget = widget.tab(*label);
    }
    ui.add(widget).changed()
}

/// Material filled tonal button (medium emphasis, e.g. "update" actions).
pub fn tonal_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(MaterialButton::filled_tonal(label.into()).min_size(egui::vec2(0.0, 36.0)))
}

/// Small Material "chip" used for quick-fill suggestions and presets.
pub fn chip(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(MaterialChip::suggestion(label).small())
}

/// Material filled button for the primary action of a form.
pub fn primary_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(MaterialButton::filled(label.into()).min_size(egui::vec2(0.0, 36.0)))
}

/// Material outlined button for secondary actions.
pub fn secondary_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(MaterialButton::outlined(label.into()).min_size(egui::vec2(0.0, 36.0)))
}

/// Material text button for low-emphasis actions.
pub fn text_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    ui.add(MaterialButton::text(label.into()).min_size(egui::vec2(0.0, 32.0)))
}

/// Material switch (toggle).
pub fn switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    ui.add(MaterialSwitch::new(value))
}

/// Material checkbox with an inline label.
pub fn checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str) -> egui::Response {
    ui.add(MaterialCheckbox::new(value, label))
}

/// Outlined Material single-line text field without an external label.
pub fn input(ui: &mut egui::Ui, buffer: &mut String, hint: &str, width: f32) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let (fill, stroke) = if dark {
        (
            egui::Color32::from_rgb(11, 18, 32),
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(51, 65, 85)),
        )
    } else {
        (
            egui::Color32::WHITE,
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225)),
        )
    };

    egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(buffer)
                    .frame(false)
                    .hint_text(hint)
                    .desired_width(width),
            )
        })
        .inner
}
