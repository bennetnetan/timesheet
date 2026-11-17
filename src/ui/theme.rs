//! Material 3 design tokens and responsive breakpoints.
//!
//! This is the single source of truth for colours, spacing, shape, and sizing in
//! the UI. Views should take values from [`Tokens`] / the constants below instead
//! of hard-coding hex colours, so the Classic and Material 3 themes stay in sync.
//!
//! Reference: `docs/design/ui-ux-wireframe.md` §4 and §5.

use eframe::egui;
use serde::{Deserialize, Serialize};

// ── Layout constants (§4.4, §5) ─────────────────────────────────────────────

/// Width of the extra-large insights rail.
pub const INSIGHTS_RAIL_WIDTH: f32 = 320.0;
/// Navigation rail width when collapsed to icons.
pub const NAV_RAIL_COLLAPSED: f32 = 96.0;
/// Navigation rail width when expanded.
pub const NAV_RAIL_EXPANDED: f32 = 280.0;
/// Navigation rail width in the compact layout.
pub const NAV_RAIL_COMPACT: f32 = 72.0;
/// Default control height (M3 buttons).
pub const BUTTON_HEIGHT: f32 = 40.0;
pub const FONT_SIZE_LABEL: f32 = 12.0;
pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_TITLE: f32 = 16.0;
pub const FONT_SIZE_HEADLINE: f32 = 22.0;

// ── Breakpoints (§5.1) ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    /// `< 600` – compact phone-sized windows.
    Compact,
    /// `600–839` – medium.
    Medium,
    /// `840–1199` – expanded: collapsed nav rail.
    Expanded,
    /// `1200–1599` – large: expanded nav rail.
    Large,
    /// `1600+` – extra-large: expanded rail + insights rail.
    ExtraLarge,
}

impl Breakpoint {
    pub fn from_width(width: f32) -> Self {
        if width < 600.0 {
            Self::Compact
        } else if width < 840.0 {
            Self::Medium
        } else if width < 1200.0 {
            Self::Expanded
        } else if width < 1600.0 {
            Self::Large
        } else {
            Self::ExtraLarge
        }
    }

    pub fn uses_expanded_rail(self) -> bool {
        matches!(self, Self::Large | Self::ExtraLarge)
    }

    pub fn shows_insights_rail(self) -> bool {
        matches!(self, Self::ExtraLarge)
    }

    pub fn is_narrow(self) -> bool {
        matches!(self, Self::Compact | Self::Medium)
    }

    /// Sidebar width for a docked (non-modal) rail.
    pub fn rail_width(self) -> f32 {
        if self.uses_expanded_rail() {
            NAV_RAIL_EXPANDED
        } else if self.is_narrow() {
            NAV_RAIL_COMPACT
        } else {
            NAV_RAIL_COLLAPSED
        }
    }
}

// ── Appearance options (§10) ────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    #[default]
    Blue,
    Teal,
    Violet,
    Green,
    Amber,
}

impl Accent {
    pub const ALL: [Accent; 5] = [
        Accent::Blue,
        Accent::Teal,
        Accent::Violet,
        Accent::Green,
        Accent::Amber,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Accent::Blue => "Blue",
            Accent::Teal => "Teal",
            Accent::Violet => "Violet",
            Accent::Green => "Green",
            Accent::Amber => "Amber",
        }
    }

    /// `(primary, primary_container, on_primary_container)` for light and dark.
    fn palette(
        self,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ) {
        // light primary, light container, light on-container, dark primary, dark container
        match self {
            Accent::Blue => ("#2563EB", "#DBEAFE", "#1E3A8A", "#60A5FA", "#1E3A8A"),
            Accent::Teal => ("#0D9488", "#CCFBF1", "#134E4A", "#2DD4BF", "#115E59"),
            Accent::Violet => ("#7C3AED", "#EDE9FE", "#4C1D95", "#A78BFA", "#4C1D95"),
            Accent::Green => ("#16A34A", "#DCFCE7", "#14532D", "#4ADE80", "#14532D"),
            Accent::Amber => ("#D97706", "#FEF3C7", "#78350F", "#FBBF24", "#78350F"),
        }
    }
}

/// Overall visual style. `Classic` keeps the original slate/blue look;
/// `Material3` follows the full token palette and the selected accent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeStyle {
    #[default]
    Classic,
    Material3,
}

impl ThemeStyle {
    pub const ALL: [ThemeStyle; 2] = [ThemeStyle::Classic, ThemeStyle::Material3];

    pub fn label(self) -> &'static str {
        match self {
            ThemeStyle::Classic => "Classic",
            ThemeStyle::Material3 => "Material 3 (full)",
        }
    }
}

/// Light/dark preference. `System` follows the Windows app theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark];

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }
}

// ── Colour tokens (§4.2) ────────────────────────────────────────────────────

/// Resolved colour roles for the current mode + accent.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub dark: bool,
    pub primary: egui::Color32,
    pub on_primary: egui::Color32,
    pub primary_container: egui::Color32,
    pub on_primary_container: egui::Color32,
    pub secondary: egui::Color32,
    pub on_secondary: egui::Color32,
    pub secondary_container: egui::Color32,
    pub on_secondary_container: egui::Color32,
    pub tertiary: egui::Color32,
    pub on_tertiary: egui::Color32,
    pub tertiary_container: egui::Color32,
    pub on_tertiary_container: egui::Color32,
    pub error: egui::Color32,
    pub warning: egui::Color32,
    pub on_error: egui::Color32,
    pub error_container: egui::Color32,
    pub on_error_container: egui::Color32,
    pub surface: egui::Color32,
    pub surface_container_lowest: egui::Color32,
    pub surface_container_low: egui::Color32,
    pub surface_container: egui::Color32,
    pub surface_container_high: egui::Color32,
    pub surface_container_highest: egui::Color32,
    pub on_surface: egui::Color32,
    pub on_surface_variant: egui::Color32,
    pub outline: egui::Color32,
    pub outline_variant: egui::Color32,
    pub inverse_surface: egui::Color32,
    pub inverse_on_surface: egui::Color32,
}

impl Tokens {
    pub fn new(dark: bool, accent: Accent) -> Self {
        let (light_primary, light_container, light_on_container, dark_primary, dark_container) =
            accent.palette();
        let hex = |value: &str| {
            egui_material3::theme::MaterialThemeContext::hex_to_color32(value)
                .unwrap_or(egui::Color32::GRAY)
        };

        let primary = hex(if dark { dark_primary } else { light_primary });
        let primary_container = hex(if dark {
            dark_container
        } else {
            light_container
        });
        let on_primary_container = hex(if dark {
            light_container
        } else {
            light_on_container
        });

        // Secondary/tertiary stay stable across accents (indigo + teal), matching §4.2.
        let (secondary, on_secondary, secondary_container, on_secondary_container) = if dark {
            (
                hex("#818CF8"),
                hex("#111827"),
                hex("#312E81"),
                hex("#E0E7FF"),
            )
        } else {
            (
                hex("#4F46E5"),
                hex("#FFFFFF"),
                hex("#E0E7FF"),
                hex("#312E81"),
            )
        };
        let (tertiary, on_tertiary, tertiary_container, on_tertiary_container) = if dark {
            (
                hex("#2DD4BF"),
                hex("#042F2E"),
                hex("#115E59"),
                hex("#CCFBF1"),
            )
        } else {
            (
                hex("#0D9488"),
                hex("#FFFFFF"),
                hex("#CCFBF1"),
                hex("#134E4A"),
            )
        };
        let (error, on_error, error_container, on_error_container) = if dark {
            (
                hex("#F87171"),
                hex("#450A0A"),
                hex("#7F1D1D"),
                hex("#FEE2E2"),
            )
        } else {
            (
                hex("#DC2626"),
                hex("#FFFFFF"),
                hex("#FEE2E2"),
                hex("#7F1D1D"),
            )
        };

        let (
            surface,
            surface_container_lowest,
            surface_container_low,
            surface_container,
            surface_container_high,
            surface_container_highest,
            on_surface,
            on_surface_variant,
            outline,
            outline_variant,
            inverse_surface,
            inverse_on_surface,
        ) = if dark {
            (
                hex("#0F1420"),
                hex("#0B101B"),
                hex("#121A28"),
                hex("#182030"),
                hex("#1E293B"),
                hex("#273449"),
                hex("#E2E8F0"),
                hex("#94A3B8"),
                hex("#64748B"),
                hex("#334155"),
                hex("#E2E8F0"),
                hex("#0F172A"),
            )
        } else {
            (
                hex("#F8FAFC"),
                hex("#FFFFFF"),
                hex("#FFFFFF"),
                hex("#F1F5F9"),
                hex("#E2E8F0"),
                hex("#CBD5E1"),
                hex("#0F172A"),
                hex("#475569"),
                hex("#94A3B8"),
                hex("#E2E8F0"),
                hex("#1E293B"),
                hex("#F8FAFC"),
            )
        };

        Self {
            dark,
            primary,
            on_primary: if dark { hex("#0B1220") } else { hex("#FFFFFF") },
            primary_container,
            on_primary_container,
            secondary,
            on_secondary,
            secondary_container,
            on_secondary_container,
            tertiary,
            on_tertiary,
            tertiary_container,
            on_tertiary_container,
            error,
            warning: if dark { hex("#FBBF24") } else { hex("#D97706") },
            on_error,
            error_container,
            on_error_container,
            surface,
            surface_container_lowest,
            surface_container_low,
            surface_container,
            surface_container_high,
            surface_container_highest,
            on_surface,
            on_surface_variant,
            outline,
            outline_variant,
            inverse_surface,
            inverse_on_surface,
        }
    }

    /// Resolve the palette for a theme style, mode, and accent.
    ///
    /// Classic always uses the original blue accent so the legacy look is kept;
    /// Material 3 follows the user's accent choice.
    pub fn for_style(style: ThemeStyle, dark: bool, accent: Accent) -> Self {
        match style {
            ThemeStyle::Classic => Self::new(dark, Accent::Blue),
            ThemeStyle::Material3 => Self::new(dark, accent),
        }
    }

    /// Card background (L2).
    pub fn card_fill(&self) -> egui::Color32 {
        self.surface_container_low
    }

    /// Colour for a 1 dp card/table divider.
    pub fn divider(&self) -> egui::Stroke {
        egui::Stroke::new(1.0_f32, self.outline_variant)
    }
}

// ── Component helpers (§4.4) ────────────────────────────────────────────────

// ── egui theme mapping (§10.2) ──────────────────────────────────────────────

/// Map tokens onto egui's widget visuals (the §10.2 token mapping).
pub fn visuals(tokens: &Tokens, style: ThemeStyle) -> egui::Visuals {
    let mut visuals = if tokens.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let corner = egui::CornerRadius::same(if style == ThemeStyle::Material3 { 8 } else { 6 });

    visuals.panel_fill = tokens.surface;
    visuals.window_fill = tokens.surface_container_high;
    // Striped table rows: a touch stronger for Material 3 so alternating rows
    // stay visible; Classic keeps its original subtle band.
    visuals.faint_bg_color = if style == ThemeStyle::Material3 {
        tokens.surface_container
    } else {
        tokens.surface_container_low
    };
    visuals.extreme_bg_color = tokens.surface_container_lowest;
    visuals.window_stroke = egui::Stroke::new(1.0_f32, tokens.outline_variant);
    visuals.window_corner_radius = egui::CornerRadius::same(if style == ThemeStyle::Material3 {
        12
    } else {
        6
    });
    visuals.menu_corner_radius = egui::CornerRadius::same(12);
    visuals.override_text_color = Some(tokens.on_surface);
    visuals.weak_text_color = Some(tokens.on_surface_variant);
    visuals.hyperlink_color = tokens.primary;
    visuals.error_fg_color = tokens.error;
    visuals.warn_fg_color = tokens.warning;

    visuals.widgets.noninteractive.bg_fill = tokens.surface_container_low;
    visuals.widgets.noninteractive.bg_stroke = tokens.divider();
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, tokens.on_surface);
    visuals.widgets.noninteractive.corner_radius = corner;

    visuals.widgets.inactive.bg_fill = tokens.surface_container_high;
    visuals.widgets.inactive.weak_bg_fill = tokens.surface_container_low;
    visuals.widgets.inactive.bg_stroke = tokens.divider();
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, tokens.on_surface);
    visuals.widgets.inactive.corner_radius = corner;

    visuals.widgets.hovered.bg_fill = tokens.surface_container_highest;
    visuals.widgets.hovered.weak_bg_fill = tokens.surface_container_high;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, tokens.primary);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, tokens.on_surface);
    visuals.widgets.hovered.corner_radius = corner;

    visuals.widgets.active.bg_fill = tokens.primary_container;
    visuals.widgets.active.weak_bg_fill = tokens.primary_container;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, tokens.primary);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, tokens.on_primary_container);
    visuals.widgets.active.corner_radius = corner;

    visuals.widgets.open = visuals.widgets.hovered;

    visuals.selection.bg_fill = tokens.primary_container;
    visuals.selection.stroke = egui::Stroke::new(1.5_f32, tokens.primary);

    visuals
}

/// Complete egui style (visuals + the §4.4 spacing grid) for the app.
pub fn style(tokens: &Tokens, style: ThemeStyle) -> egui::Style {
    let mut style = egui::Style {
        visuals: visuals(tokens, style),
        ..Default::default()
    };
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.interact_size = egui::vec2(BUTTON_HEIGHT, BUTTON_HEIGHT);
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(FONT_SIZE_LABEL),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(FONT_SIZE_BODY),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(FONT_SIZE_BODY),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(FONT_SIZE_HEADLINE),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::monospace(FONT_SIZE_BODY),
    );
    style
}

/// Standard card frame: tonal surface, 1 dp outline, 12 dp radius, 16 dp padding.
pub fn card_frame(tokens: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.card_fill())
        .stroke(tokens.divider())
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
}

/// Framed container for data tables: 12 dp radius, 1 dp `outline-variant` border,
/// and a surface-container fill so the striped rows inside stay legible.
pub fn table_frame(tokens: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.surface_container_lowest)
        .stroke(tokens.divider())
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(10))
}

/// Floating dialog frame: high surface, 28 dp radius, 24 dp padding, soft shadow.
pub fn dialog_frame(tokens: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.surface_container_high)
        .stroke(tokens.divider())
        .corner_radius(egui::CornerRadius::same(28))
        .inner_margin(egui::Margin::same(24))
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(if tokens.dark { 120 } else { 48 }),
        })
}

/// Sticky filter/bulk bar (L3).
pub fn bar_frame(tokens: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.surface_container)
        .stroke(tokens.divider())
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(16, 8))
}
