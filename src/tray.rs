/// System tray icon integration for Windows.
///
/// We generate a simple coloured square icon at runtime (no external file needed)
/// so the binary stays fully self-contained.
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

pub struct TrayHandle {
    /// Keep alive — dropping this removes the tray icon.
    pub _icon: TrayIcon,
    pub show_item_id: tray_icon::menu::MenuId,
    pub dashboard_item_id: tray_icon::menu::MenuId,
    pub stop_timer_item_id: tray_icon::menu::MenuId,
    pub quit_item_id: tray_icon::menu::MenuId,
}

impl TrayHandle {
    /// Build the tray icon and context menu. Call once from `TimesheetApp::new`.
    pub fn build() -> anyhow::Result<Self> {
        let tray_menu = Menu::new();

        let show_item = MenuItem::new("Show / Hide Window", true, None);
        let dashboard_item = MenuItem::new("Open Dashboard", true, None);
        let stop_timer_item = MenuItem::new("Stop & Save Timer", true, None);
        let quit_item = MenuItem::new("Quit Timesheet", true, None);

        tray_menu
            .append(&show_item)
            .map_err(|e| anyhow::anyhow!("tray menu append: {e:?}"))?;
        tray_menu
            .append(&dashboard_item)
            .map_err(|e| anyhow::anyhow!("tray menu append: {e:?}"))?;
        tray_menu
            .append(&stop_timer_item)
            .map_err(|e| anyhow::anyhow!("tray menu append: {e:?}"))?;
        tray_menu
            .append(&quit_item)
            .map_err(|e| anyhow::anyhow!("tray menu append: {e:?}"))?;

        let icon = make_icon();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Timesheet — click to show")
            .with_icon(icon)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create tray icon: {e:?}"))?;

        Ok(Self {
            show_item_id: show_item.id().clone(),
            dashboard_item_id: dashboard_item.id().clone(),
            stop_timer_item_id: stop_timer_item.id().clone(),
            quit_item_id: quit_item.id().clone(),
            _icon: tray,
        })
    }

    /// Update tray tooltip to reflect active timer state.
    pub fn update_tooltip(&self, text: &str) {
        let _ = self._icon.set_tooltip(Some(text));
    }
}

/// Generate a 32×32 RGBA icon: a blue-tinted clock-face look (solid teal square
/// with a white inner ring and a small clock hand) — all done with simple pixel math.
fn make_icon() -> Icon {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    let cx = SIZE as f32 / 2.0;
    let cy = SIZE as f32 / 2.0;
    let outer_r = 15.0_f32;
    let ring_r = 11.0_f32;
    let inner_r = 9.5_f32;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * SIZE + x) * 4) as usize;

            if dist <= outer_r {
                if dist <= inner_r {
                    // Inner face — white/light
                    rgba[idx] = 240;
                    rgba[idx + 1] = 248;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 255;
                } else if dist <= ring_r {
                    // Teal ring
                    rgba[idx] = 20;
                    rgba[idx + 1] = 184;
                    rgba[idx + 2] = 166;
                    rgba[idx + 3] = 255;
                } else {
                    // Outer ring — slightly darker teal
                    rgba[idx] = 15;
                    rgba[idx + 1] = 118;
                    rgba[idx + 2] = 110;
                    rgba[idx + 3] = 255;
                }
            }
            // else transparent
        }
    }

    // Draw 12 o'clock hand: a thin vertical line from centre up
    for y in 4..15u32 {
        let x = SIZE / 2;
        let idx = ((y * SIZE + x) * 4) as usize;
        rgba[idx] = 30;
        rgba[idx + 1] = 64;
        rgba[idx + 2] = 175;
        rgba[idx + 3] = 255;
        // 1px wide — also do x+1 to make it visible
        if x + 1 < SIZE {
            let idx2 = ((y * SIZE + x + 1) * 4) as usize;
            rgba[idx2] = 30;
            rgba[idx2 + 1] = 64;
            rgba[idx2 + 2] = 175;
            rgba[idx2 + 3] = 255;
        }
    }
    // 3 o'clock hand: horizontal from centre right
    for x in 16..22u32 {
        let y = SIZE / 2;
        let idx = ((y * SIZE + x) * 4) as usize;
        rgba[idx] = 30;
        rgba[idx + 1] = 64;
        rgba[idx + 2] = 175;
        rgba[idx + 3] = 255;
    }

    Icon::from_rgba(rgba, SIZE, SIZE).expect("valid icon")
}
