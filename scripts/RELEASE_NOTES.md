# Timesheet v0.17.12 — Adaptive Material 3 UI

This release brings the UI forward from v0.11.2 with a responsive Material 3 design, clearer data tables, and a more focused dashboard. Timesheet remains offline-first; your local database is preserved when updating.

## What's new

- **Material 3 appearance:** centralized theme tokens, Light/Dark/System appearance options, consistent surfaces, dialogs, and navigation.
- **Adaptive desktop layout:** improved navigation and content layouts across window sizes, including an insights rail on extra-wide displays.
- **Dashboard improvements:** clearer task and timer priority, aligned KPI cards, compact goal progress, responsive activity cards, and a weekly overview.
- **More readable tables:** table actions use clear text labels instead of font-dependent icon glyphs; rows remain usable at narrower widths.
- **Improved forms and reports:** refreshed entry, project, settings, and export workflows, with a Reports details pane.
- **Reliability and polish:** restored entry dialogs, fixed collapsed sidebar footer overflow, removed compiler warnings, and improved text sizing and duration alignment.

## Windows downloads

| File | Description |
|------|-------------|
| **Timesheet-Setup.exe** | Recommended single-file installer |
| **Timesheet-v0.17.12-Windows-x64.zip** | Portable app and install/uninstall scripts |
| **timesheet.exe** | Standalone application binary |

The app stores data locally at `%APPDATA%\timesheet\Timesheet\timesheet.db`. Installing this release does not remove your existing database.
