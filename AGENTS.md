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
- `src/ui/` – egui app state and views (dashboard, entries, projects, reports, settings, Material 3 helpers); `src/ui/theme.rs` holds the design tokens for colours, spacing, breakpoints, and component frames
- `docs/design/ui-ux-wireframe.md` – **primary UI/UX reference**: layered Material 3 design system, adaptive breakpoints, screen wireframes, dialog specs, timer UX, tray behaviour, and the full Material 3 theme plan
- `docs/design/wireframe.html` – self-contained interactive mockup of the design doc (open directly in a browser; no build step)
- `Cargo.toml` – crate metadata and dependencies
- `README.md` – user/developer overview and run instructions

## Development practices

- Keep the app **offline-first** and local-only unless a change explicitly requires otherwise.
- Prefer simple, readable Rust and follow existing module/style patterns.
- Use `anyhow::Result` for fallible operations where appropriate and surface user-facing errors in the UI status/messages.
- When changing behavior or adding features, update `README.md` and this file as needed.

## Design & UX

`docs/design/ui-ux-wireframe.md` is the source of truth for UI direction; `docs/design/wireframe.html`
is its clickable reference. Read them before redesigning a view, and keep code aligned with them.

- **Layered, not flat:** depth comes from Material 3 surface-container roles + 1 dp `outline-variant`
  borders. Shadows are reserved for floating layers (dialogs, menus, snackbars), never ordinary cards.
- **Forms are dialogs:** every create/edit form opens in a basic desktop dialog (280–560 dp wide,
  28 dp radius, 24 dp padding, scrim + focus trap). No long inline forms in page bodies.
- **Wide screens earn their space:** use the breakpoint table in the design doc (840 / 1200 / 1600 dp).
  At 1600+ reveal the right insights rail instead of stretching cards; keep text columns ≤ ~75 chars.
- **Industry sizing:** 40 dp buttons, 56 dp text fields, 48 dp minimum hit targets, 12 dp cards,
  64 dp top app bar, tabular numerals for all clocks/durations.
- **Timer rules:** persist the start timestamp, show state with icon + text + colour (never colour
  alone), review on stop, and prompt calmly for idle/long sessions.
- **Tray rules:** follow Microsoft notification-area guidance — left click opens the window, right
  click shows the menu, `Open` first/default, minimize-to-tray is opt-in, never flash the icon.
- **Theming:** new colours, radii, and type sizes must come from theme tokens so the
  Classic / full Material 3 switch keeps working for every view.

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
