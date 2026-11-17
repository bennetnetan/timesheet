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

/// Maximum width of centred page content on very wide windows.
pub const CONTENT_MAX_WIDTH: f32 = 1440.0;
/// Maximum width of a single reading/form column in the large layout.
pub const CONTENT_READING_WIDTH: f32 = 1120.0;
/// Width of the extra-large insights rail.
pub const INSIGHTS_RAIL_WIDTH: f32 = 320.0;
/// Height of the top app bar.
pub const TOP_APP_BAR_HEIGHT: f32 = 64.0;
/// Navigation rail width when collapsed to icons.
pub const NAV_RAIL_COLLAPSED: f32 = 96.0;
/// Navigation rail width when expanded.
pub const NAV_RAIL_EXPANDED: f32 = 280.0;
/// Navigation rail width in the compact layout.
pub const NAV_RAIL_COMPACT: f32 = 72.0;
/// Default control height (M3 buttons).
pub const BUTTON_HEIGHT: f32 = 40.0;
/// Minimum interactive target (M3).
pub const MIN_HIT_TARGET: f32 = 48.0;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

impl Density {
    pub fn label(self) -> &'static str {
        match self {
            Density::Comfortable => "Comfortable",
            Density::Compact => "Compact",
        }
    }

    pub fn row_height(self) -> f32 {
        match self {
            Density::Comfortable => 52.0,
            Density::Compact => 44.0,
        }
    }

    pub fn item_spacing(self) -> egui::Vec2 {
        match self {
            Density::Comfortable => egui::vec2(8.0, 8.0),
            Density::Compact => egui::vec2(6.0, 6.0),
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

/// Standard card frame: tonal surface, 1 dp outline, 12 dp radius, 16 dp padding.
pub fn card_frame(tokens: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.card_fill())
        .stroke(tokens.divider())
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
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
