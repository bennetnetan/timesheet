# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), PDF, and Markdown via native save dialogs (`rfd`)

## Features
- Local SQLite database with automatic schema creation (WAL mode enabled)
- Projects CRUD (name, client, rate, active flag)
- Time entries CRUD (date, project, start/end, duration, description, billable, tags)
- Dashboard with quick-add entry, today/week/month totals, and a start/stop timer that saves entries (running timer persists across app restarts)
- Easy input improvements: date pickers, project selectors, start/end with auto-calculated duration (manual duration fallback), "Now" helper for end time
- Entries view with date/project/billable filters, editable table, inline row editing, form editing, multi-select, and bulk actions (delete, set billable, apply tags)
- Validation: prevents saving entries with overlapping start/end time ranges on the same date
- Keyboard-friendly shortcuts in Entries: **Ctrl+N** new entry, **Enter** save form, **Esc** cancel/clear selection, **Delete** delete selected
- Reports view with custom date range presets, project/billable filters, totals (including billable vs non-billable), project/client breakdown with estimated amounts, entry preview, and export actions
- Export filtered entries to **CSV**, **Excel (.xlsx)**, **PDF**, and **Markdown**
  - CSV: optional summary/metadata rows and project grouping
  - Excel: styled headers, freeze panes, autofilter, number formats, project sections, subtotals, and grand total
  - PDF: metadata header (period/filters/summary), repeating page headers, project grouping/subtotals, pagination, and signature line
  - Markdown: summary, project sections/subtotals, and formatted tables
- **Settings** tab with company/user profile metadata (shown in export headers as "Prepared by"), light/dark theme toggle, and a default export folder used by all save dialogs
- Confirmation prompts before deleting entries, deleting selected entries, deleting projects, and restoring a backup
- **Backup & restore:** export a self-contained `.db` snapshot (WAL checkpointed) and restore from one; the app should be restarted after restoring
- Unit tests for duration parsing/formatting and export metadata (`cargo test`)

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
- `src/ui/` – egui app and views (dashboard, entries, projects, reports, export, settings)
- `src/main.rs` – application entry point
