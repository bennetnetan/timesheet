//! Material Design 3 integration helpers.
//!
//! The app uses [`egui_material3`] for a handful of interactive components
//! (tabs, buttons, switches, chips).  That crate keeps its palette in a global
//! theme, so we push the resolved [`Tokens`] whenever the appearance changes.

use crate::ui::theme::Tokens;
use eframe::egui;
use egui_material3::{
    ContrastLevel, MaterialButton, MaterialCheckbox, MaterialChip, MaterialSwitch, MaterialTabs,
    ThemeMode, get_global_theme,
};

/// Push the resolved colour tokens into egui-material3's global theme.
///
/// Material components read colours from this theme on every frame, so this
/// must be called from `TimesheetApp::apply_theme`.
pub fn apply_theme(tokens: &Tokens) {
    let global_theme = get_global_theme();
    let Ok(mut theme) = global_theme.lock() else {
        return;
    };

    theme.theme_mode = if tokens.dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    theme.contrast_level = ContrastLevel::Normal;
    theme.selected_colors.clear();

    let palette: &[(&str, egui::Color32)] = &[
        ("primary", tokens.primary),
        ("onPrimary", tokens.on_primary),
        ("primaryContainer", tokens.primary_container),
        ("onPrimaryContainer", tokens.on_primary_container),
        ("secondary", tokens.secondary),
        ("onSecondary", tokens.on_secondary),
        ("secondaryContainer", tokens.secondary_container),
        ("onSecondaryContainer", tokens.on_secondary_container),
        ("tertiary", tokens.tertiary),
        ("onTertiary", tokens.on_tertiary),
        ("tertiaryContainer", tokens.tertiary_container),
        ("onTertiaryContainer", tokens.on_tertiary_container),
        ("error", tokens.error),
        ("onError", tokens.on_error),
        ("errorContainer", tokens.error_container),
        ("onErrorContainer", tokens.on_error_container),
        ("surface", tokens.surface),
        ("onSurface", tokens.on_surface),
        ("surfaceVariant", tokens.surface_container),
        ("onSurfaceVariant", tokens.on_surface_variant),
        ("surfaceContainerLowest", tokens.surface_container_lowest),
        ("surfaceContainerLow", tokens.surface_container_low),
        ("surfaceContainer", tokens.surface_container),
        ("surfaceContainerHigh", tokens.surface_container_high),
        ("surfaceContainerHighest", tokens.surface_container_highest),
        ("surfaceTint", tokens.primary),
        ("background", tokens.surface),
        ("onBackground", tokens.on_surface),
        ("outline", tokens.outline),
        ("outlineVariant", tokens.outline_variant),
        ("inverseSurface", tokens.inverse_surface),
        ("inverseOnSurface", tokens.inverse_on_surface),
        ("shadow", egui::Color32::BLACK),
        ("scrim", egui::Color32::BLACK),
    ];

    for (name, color) in palette {
        theme.selected_colors.insert((*name).to_string(), *color);
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
    // Add horizontal padding so tabs don't get clipped by adjacent panels
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let changed = ui.add(widget).changed();
        ui.add_space(16.0);
        changed
    })
    .inner
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
///
/// Colours come from the active theme visuals, which in turn are built from
/// the design tokens, so this field follows both appearance styles.
pub fn input(ui: &mut egui::Ui, buffer: &mut String, hint: &str, width: f32) -> egui::Response {
    let fill = ui.visuals().extreme_bg_color;
    let stroke = egui::Stroke::new(1.0_f32, ui.visuals().widgets.inactive.bg_stroke.color);

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
