# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), and PDF via native save dialogs (`rfd`)

## Features
- Local SQLite database with automatic schema creation (WAL mode enabled)
- Projects CRUD (name, client, rate, active flag)
- Time entries CRUD (date, project, start/end, duration, description, billable, tags)
- Dashboard with quick-add entry, today/week/month totals, and a start/stop timer that saves entries (running timer persists across app restarts)
- Easy input improvements: date pickers, project selectors, start/end with auto-calculated duration (manual duration fallback), "Now" helper for end time
- Entries view with date/project/billable filters, editable table, inline row editing, form editing, multi-select, and bulk actions (delete, set billable, apply tags)
- Validation: prevents saving entries with overlapping start/end time ranges on the same date
- Keyboard-friendly shortcuts in Entries: **Ctrl+N** new entry, **Enter** save form, **Esc** cancel/clear selection, **Delete** delete selected
- Export filtered entries to **CSV**, **Excel**, and **PDF**

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
- Set filters in the **Entries** tab before exporting to limit the report range.
- Data is stored offline in your Windows AppData directory (database and running timer state).

## Project structure
- `src/db/` – database init, models, queries
- `src/domain/` – duration parsing/formatting helpers
- `src/export/` – CSV, Excel, PDF exporters
- `src/ui/` – egui app and views (dashboard, entries, projects, export)
- `src/main.rs` – application entry point
