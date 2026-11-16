use anyhow::{Context, Result};
use chrono::NaiveDate;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTimer {
    pub running_since_unix_ms: i64,
    #[serde(default)]
    pub accumulated_secs: u64,
    #[serde(default)]
    pub is_paused: bool,
    pub date: NaiveDate,
    pub project_id: Option<i64>,
    pub start_time: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

fn path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("com", "timesheet", "Timesheet")
        .context("could not determine application data directory")?;
    Ok(proj.data_dir().join("timer.json"))
}

pub fn load() -> Option<PersistedTimer> {
    let p = path().ok()?;
    let data = fs::read_to_string(p).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save(t: &PersistedTimer) -> Result<()> {
    let p = path()?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(t)?)?;
    Ok(())
}

pub fn clear() -> Result<()> {
    let p = path()?;
    if p.exists() {
        fs::remove_file(p)?;
    }
    Ok(())
}

pub fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as i64
}
