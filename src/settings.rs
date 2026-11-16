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
            default_export_dir: String::new(),
            round_minutes: 0,
            daily_target_hours: 8.0,
            pomodoro_focus_mins: 25,
            pomodoro_break_mins: 5,
        }
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
    path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(s: &Settings) -> Result<()> {
    let p = path()?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(s)?)?;
    Ok(())
}
