use crate::ui::theme::{Accent, ThemeMode, ThemeStyle};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub user_name: String,
    #[serde(default = "default_true")]
    pub dark_theme: bool,
    /// Explicit light/dark preference; `None` keeps the legacy `dark_theme` flag.
    #[serde(default)]
    pub theme_mode: Option<ThemeMode>,
    #[serde(default)]
    pub theme_style: ThemeStyle,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default)]
    pub default_export_dir: String,
    #[serde(default)]
    pub round_minutes: u32,
    #[serde(default = "default_target_hours")]
    pub daily_target_hours: f64,
    #[serde(default = "default_pomodoro_focus")]
    pub pomodoro_focus_mins: u32,
    #[serde(default = "default_pomodoro_break")]
    pub pomodoro_break_mins: u32,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
}

fn default_true() -> bool {
    true
}

fn default_target_hours() -> f64 {
    8.0
}

fn default_pomodoro_focus() -> u32 {
    25
}

fn default_pomodoro_break() -> u32 {
    5
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            company_name: String::new(),
            user_name: String::new(),
            dark_theme: true,
            theme_mode: Some(ThemeMode::Dark),
            theme_style: ThemeStyle::default(),
            accent: Accent::default(),
            default_export_dir: String::new(),
            round_minutes: 0,
            daily_target_hours: 8.0,
            pomodoro_focus_mins: 25,
            pomodoro_break_mins: 5,
            minimize_to_tray: true,
        }
    }
}

impl Settings {
    /// Resolve the effective dark-mode flag, following the Windows app theme
    /// when the user picked `System`.
    pub fn resolved_dark_theme(&self) -> bool {
        match self.theme_mode {
            Some(ThemeMode::System) => crate::system_theme::is_dark(),
            Some(ThemeMode::Light) => false,
            Some(ThemeMode::Dark) => true,
            None => self.dark_theme,
        }
    }

    /// Set an explicit light/dark preference (used by the sidebar toggle).
    pub fn set_dark_theme(&mut self, dark: bool) {
        self.dark_theme = dark;
        self.theme_mode = Some(if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        });
    }
}

pub fn data_dir() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "timesheet", "Timesheet")
        .context("could not determine application data directory")?;
    Ok(proj.data_dir().to_path_buf())
}

fn path() -> Result<PathBuf> {
    Ok(data_dir()?.join("settings.json"))
}

pub fn load() -> Settings {
    let mut settings: Settings = path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    // Migrate settings written before the explicit theme mode existed.
    if settings.theme_mode.is_none() {
        settings.theme_mode = Some(if settings.dark_theme {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        });
    }
    settings
}

pub fn save(s: &Settings) -> Result<()> {
    let p = path()?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(s)?)?;
    Ok(())
}
