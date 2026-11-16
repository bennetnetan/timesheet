# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), PDF, and Markdown via native save dialogs (`rfd`)

## Features
- Local SQLite database with automatic schema creation (WAL mode enabled)
- **Projects management:**
  - Real-time search filter and **Active only** toggle
  - Visual color badge palette picker (8 modern colors)
  - Inline **Archive** / **Activate** toggle button with live status indicator
  - Client name, hourly rate, and lifetime tracked time/count
- Time entries CRUD (date, project, start/end, duration, description, billable, tags)
- **Global active timer pill** in top header across all tabs showing elapsed time and project with quick Pause/Resume, Stop & Save, and **long-running alert (> 3 hours)**
- **Dashboard hub:**
  - **Quick Task Omnibox:** Single-line natural language parser with live project and tag autocomplete (`Task description @Project #tags duration`). Type `Design logo @Acme #ui 45m` and press Enter to log instantly, or omit duration to start a live timer!
  - **Pomodoro Focus Timer Mode:** Dual-mode timer switching between standard Stopwatch and Pomodoro Focus sessions (default 25 min focus / 5 min break) with countdown readouts, animated progress bar, and 1-click break transitions.
  - **Dynamic Time-of-Day Greeting:** Personalized greeting based on local time and profile name (e.g. `Good morning, Netan! Ready for a focused day? ☀️`)
  - **Daily Goal Achievement Banner:** Celebratory banner displayed upon reaching your configured daily target hours
  - Hero KPI cards:
    - **Today** with configurable daily target progress bar and celebration badge (`🎉 Goal Met!`)
    - **This Week** with billable hours and active days counter (e.g. `4 active days`)
    - **This Month** total hours and **Estimated Value** from project rates
  - **Weekly Overview Bar Chart:** Interactive Mon–Sun daily stacked bars displaying billable vs non-billable hours, today highlight, and hover breakdown tooltips
  - Live digital stopwatch with Pause / Resume support, quick adjustments (`-5m`, `+5m`, `+15m`), and session safety warnings
  - "Today's Activity" list with one-click **Continue** (`▶`) and **Clone** (`📋`)
  - **Supportive Empty States & Starter Chips:** One-click quick-start chips (`💻 Deep Work`, `☕ Quick Sync`, `📋 Planning & Admin`, `🛠 Bug Fixing`, `📖 Research & Notes`) to start tracking with zero friction
  - Quick Add entry form with project color badges and recent task suggestions
- **Entries view:**
  - Real-time instant full-text search across description, tags, project, and client
  - **☰ List / ▦ Weekly Grid toggle:** Switch between the classic chronological list view and a compact weekly timesheet matrix
  - **Weekly Grid view:** Rows = active projects, columns = Mon–Sun + Total row/column; click any cell to enter or edit hours; supports all duration formats (`1.5`, `1:30`, `1h30m`, `45m`); prev/next week navigation; today highlighted in blue; weekend columns dimmed
  - **Filtered Summary Bar:** Live total hours, billable amount, non-billable time, selected count, one-click `📋 Copy Summary`, and `↗ Open in Reports`
  - **Supportive Empty States:** Context-aware guidance with 1-click `↺ Reset all filters & search` or quick-start buttons
  - **Tag Cloud filtering:** interactive clickable tag chips with entry counts
  - Quick date presets (Today, Yesterday, This week, This month, All)
  - Sortable table columns (Date, Project, Duration)
  - Visual project color badges on every entry and in project selector dropdowns
  - Quick row actions: **Continue** in timer, **Clone** into form for today, **Inline** table editing, form edit, delete
  - Multi-select and bulk actions (delete, set billable, apply tags)
  - Recent description suggestion chips in entry form
  - Overlap validation prevents saving conflicting time entries on the same date
- **Windows System Tray & Mini-Timer Support:**
  - Native Windows system tray integration with custom generated RGBA icon (no external image assets required)
  - Tray context menu: Show/Hide Window, Open Dashboard, Stop & Save Timer, Quit
  - Left-click tray icon to toggle/focus window
  - Dynamic tray tooltip continuously reflecting active timer elapsed time
  - Close button minimization: closing window hides to tray instead of terminating when enabled
  - Header quick-action **🔽 Tray** button to minimize immediately
- **Idle Detection:**
  - Non-intrusive alert when the active timer has been running for > 15 minutes without user input (keyboard/mouse/scroll)
  - Options to keep all elapsed time or stop & save immediately
- **Global shortcuts, Status & Help:**
  - **Ctrl+1..5** for tab switching, **Ctrl+Space** toggle timer, **Ctrl+N** new entry, **Enter** save form, **Esc** cancel/clear, **Delete** delete selected
  - **F1** or **?** opens the **Keyboard Shortcuts & Duration Syntax Help** modal
  - **Semantic status indicator:** Green success, red error, and informational states with dismiss button (`✕`)
- **Reports & Export** consolidated view:
  - Custom date range presets, project/billable filters with visual project color dots
  - **Visual Project Distribution Bar:** Segmented color-coded proportion bar chart with hover percentages
  - Summary metrics and project/client breakdown with estimated totals
  - **Quick Clipboard actions:** one-click copy summary or formatted Markdown report to clipboard
  - Export filtered entries to **CSV**, **Excel (.xlsx)**, **PDF**, and **Markdown** with grouping and summary options
  - Entry preview table with project color tags
- **Settings & Theme:**
  - One-click auto-saving Light/Dark theme toggle in the header
  - **Configurable Daily Target Hours** (drag slider & quick presets: 4h, 6h, 7.5h, 8h)
  - **Pomodoro Timer Preferences** (Customizable focus duration & break duration)
  - **Duration auto-rounding preference** (Exact, Nearest 5 min, Nearest 15 min, Nearest 30 min)
  - Company and user profile metadata for exports
  - Default export folder persistence
  - Database backup and restore (`.db` snapshot) with confirmation dialogs
- Unit tests for duration parsing/formatting, auto-rounding intervals, hex color decoding, and export metadata (`cargo test`)

## Run
```powershell
cargo run
```

## Build release
```powershell
cargo build --release
```
The binary will be at `target\release\timesheet.exe`.

## Notes
- Duration accepts formats like `90`, `1.5`, `1:30`, `1h30m`, `45m`.
- Timer entries automatically apply the duration auto-rounding setting (e.g. nearest 15 min) configured in **Settings**.
- Press **F1** or **?** anywhere to view the floating keyboard shortcuts and syntax help guide.
- If start/end times are provided (`HH:MM`), duration is auto-calculated from them (end before start is treated as crossing midnight).
- Entries with overlapping start/end times on the same date are rejected to prevent double-counting.
- Use the **Reports** tab for custom ranges, summaries, breakdowns, preview, and export; filters are shared with Entries/Export.
- Export options in Reports/Export let you group by project and include summary/metadata.
- Data is stored offline in your Windows AppData directory (`timesheet.db`, `timer.json`, and `settings.json`).
- Use **Settings → Backup database** to save a snapshot, and **Restore from backup** to replace the live database. Restart the app after restoring.
- Theme and profile/export preferences are stored in `settings.json`; the default export folder is applied to every CSV/Excel/PDF/Markdown save dialog.

## Project structure
- `src/db/` – database init, schema, models, queries
- `src/domain/` – duration parsing/formatting helpers
- `src/export/` – CSV, Excel, PDF, Markdown exporters
- `src/settings.rs` – persisted user preferences (`settings.json`)
- `src/backup.rs` – database backup and restore helpers
- `src/tray.rs` – Windows system tray integration and runtime icon generation
- `src/ui/` – egui app and views (dashboard, entries, weekly_grid, projects, reports, settings)
- `src/main.rs` – application entry point
