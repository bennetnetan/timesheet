# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## Project overview

- **Name:** `timesheet`
- **Type:** Offline-first Windows desktop timesheet app
- **Stack:** Rust (`edition = "2024"`), `eframe`/`egui`, `rusqlite` (bundled SQLite), `rfd`, `csv`, `rust_xlsxwriter`, `printpdf`
- **Data location:** `%APPDATA%\timesheet\Timesheet\timesheet.db` (created automatically)

## Repository structure

- `src/main.rs` – application entry point
- `src/db/` – database initialization, schema, models, queries
- `src/domain/` – domain helpers (duration parsing/formatting, etc.)
- `src/export/` – CSV/Excel/PDF export implementations
- `src/ui/` – egui app state and views (dashboard, entries, projects, export)
- `Cargo.toml` – crate metadata and dependencies
- `README.md` – user/developer overview and run instructions

## Development practices

- Keep the app **offline-first** and local-only unless a change explicitly requires otherwise.
- Prefer simple, readable Rust and follow existing module/style patterns.
- Use `anyhow::Result` for fallible operations where appropriate and surface user-facing errors in the UI status/messages.
- When changing behavior or adding features, update `README.md` and this file as needed.

## Validation

Before finishing a change, run the relevant checks when possible:

```powershell
cargo fmt
cargo check
```

Run `cargo test` if/when tests are added or affected.

## Versioning and commits (required going forward)

For **every** change set:

1. **Update the version** in `Cargo.toml` (`[package].version`) appropriately:
   - Patch bump (e.g. `0.1.1` -> `0.1.2`) for fixes/small changes
   - Minor bump for backward-compatible features
   - Major bump for breaking changes
2. **Commit the changes** with a clear, descriptive commit message that reflects the work done.
3. Include related updates (code, docs, config, lockfile as applicable) in the same commit unless there is a good reason to split them.

Do not leave completed work uncommitted.
