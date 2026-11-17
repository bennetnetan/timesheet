//! Reads the Windows app theme (light/dark) for the `System` theme mode.
//!
//! Windows stores the preference in
//! `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
//! (`AppsUseLightTheme`).  The value is cached briefly so the UI can poll it
//! every frame without spawning a process on each repaint.

use std::sync::Mutex;
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(2);

static CACHE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

/// Whether Windows is currently using the dark app theme.
///
/// Falls back to dark when the preference cannot be read.
pub fn is_dark() -> bool {
    if let Ok(mut cache) = CACHE.lock() {
        if let Some((read_at, dark)) = *cache {
            if read_at.elapsed() < CACHE_TTL {
                return dark;
            }
        }
        let dark = read_apps_use_light_theme().unwrap_or(true);
        *cache = Some((Instant::now(), dark));
        return dark;
    }
    read_apps_use_light_theme().unwrap_or(true)
}

#[cfg(windows)]
fn read_apps_use_light_theme() -> Option<bool> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.split_whitespace().last()?;
    let value = value.trim_start_matches("0x").trim_start_matches("0X");
    let light = u32::from_str_radix(value, 16).ok()?;
    Some(light == 0)
}

#[cfg(not(windows))]
fn read_apps_use_light_theme() -> Option<bool> {
    None
}
