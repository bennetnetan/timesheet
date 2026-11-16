# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), and PDF via native save dialogs (`rfd`)

## Features (MVP)
- Local SQLite database with automatic schema creation (WAL mode enabled)
- Projects CRUD (name, client, rate, active flag)
- Time entries CRUD (date, project, start/end, duration, description, billable, tags)
- Dashboard with quick-add entry and today/week/month totals
- Entries view with date-range filters, editable table, and inline edit/delete
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
- Set filters in the **Entries** tab before exporting to limit the report range.
- Data is stored offline in your Windows AppData directory.

## Project structure
- `src/db/` – database init, models, queries
- `src/domain/` – duration parsing/formatting helpers
- `src/export/` – CSV, Excel, PDF exporters
- `src/ui/` – egui app and views (dashboard, entries, projects, export)
- `src/main.rs` – application entry point
