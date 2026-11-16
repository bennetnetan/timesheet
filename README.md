# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), PDF, and Markdown via native save dialogs (`rfd`)

## Features
- Local SQLite database with automatic schema creation (WAL mode enabled)
- Projects CRUD with visual color badges, client, hourly rate, and lifetime tracked time/count
- Time entries CRUD (date, project, start/end, duration, description, billable, tags)
- **Global active timer pill** in top header across all tabs showing elapsed time and project with quick Pause/Resume and Stop & Save
- **Dashboard hub:**
  - Hero KPI cards (Today with 8h progress bar, This Week with billable ratio, This Month, and Estimated Value from rates)
  - **Weekly Overview Bar Chart:** Interactive Mon–Sun daily stacked bars displaying billable vs non-billable hours, today highlight, and hover breakdown tooltips
  - Live digital stopwatch with Pause / Resume support and quick time adjustments (`-5m`, `+5m`, `+15m`)
  - "Today's Activity" list with one-click **Continue** (`▶`) and **Clone** (`📋`)
  - Quick Add entry form with project color badges and recent task suggestions
- **Entries view:**
  - Real-time instant full-text search across description, tags, project, and client
  - **Tag Cloud filtering:** interactive clickable tag chips with entry counts
  - Quick date presets (Today, Yesterday, This week, This month, All)
  - Sortable table columns (Date, Project, Duration)
  - Visual project color badges on every entry
  - Quick row actions: **Continue** in timer, **Clone** into form for today, **Inline** table editing, form edit, delete
  - Multi-select and bulk actions (delete, set billable, apply tags)
  - Recent description suggestion chips in entry form
  - Overlap validation prevents saving conflicting time entries on the same date
- **Global shortcuts:** **Ctrl+1..5** for tab switching, **Ctrl+Space** toggle timer, **Ctrl+N** new entry, **Enter** save form, **Esc** cancel/clear, **Delete** delete selected
- **Reports & Export** consolidated view:
  - Custom date range presets, project/billable filters
  - Summary metrics and project/client breakdown with estimated totals
  - **Quick Clipboard actions:** one-click copy summary or formatted Markdown report to clipboard
  - Export filtered entries to **CSV**, **Excel (.xlsx)**, **PDF**, and **Markdown** with grouping and summary options
  - Entry preview table with project color tags
- **Settings & Theme:**
  - One-click auto-saving Light/Dark theme toggle in the header
  - Company and user profile metadata for exports
  - Default export folder persistence
  - Database backup and restore (`.db` snapshot) with confirmation dialogs
- Unit tests for duration parsing/formatting, hex color decoding, and export metadata (`cargo test`)

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
- `src/ui/` – egui app and views (dashboard, entries, projects, reports, settings)
- `src/main.rs` – application entry point
