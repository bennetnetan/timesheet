# Timesheet (Rust, Offline Windows Desktop)

An offline-first, single-user timesheet desktop app built with Rust.

## Stack
- **GUI:** `eframe` / `egui` 0.33 with Material Design 3 components from `egui-material3`
- **Database:** SQLite via `rusqlite` (bundled), stored locally in `%APPDATA%/timesheet/Timesheet/timesheet.db`
- **Exports:** CSV, Excel (`.xlsx`), PDF, and Markdown via native save dialogs (`rfd`)

## Features
- Local SQLite database with automatic schema creation (WAL mode enabled)
- **Projects management:**
  - Real-time search filter and **Active only** toggle
  - **Harvest-style Project Budgets & Burn-Down:** Set budget hours per project with live visual progress bars, percentage indicators, and color-coded alerts (<80% green, 80–100% amber, >100% red over-budget warning)
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
- **Windows System Tray & Compact Floating Mini-Timer:**
  - **Always-on-Top Floating Mini-Timer (`Ctrl+M` / `🗖 Mini`):** Shrinks window into a compact 380×120 floating widget that stays above IDEs and browser windows while you work, with direct pause/resume, +5m nudge, stop & save, and 1-click restore to full window
  - Native Windows system tray integration with custom generated RGBA icon (no external image assets required)
  - Tray context menu: Show/Hide Window, Open Dashboard, Stop & Save Timer, Quit
  - Left-click tray icon to toggle/focus window
  - Dynamic tray tooltip continuously reflecting active timer elapsed time
  - Close button minimization: closing window hides to tray instead of terminating when enabled
  - Header quick-action **🔽 Tray** button to minimize immediately
- **AppFlowy-Style Left Navigation Sidebar & Multi-Column Widescreen Layouts (v0.11.0):**
  - **Left Navigation Sidebar (225px width):** Persistent modern left sidebar featuring workspace branding, active live timer card with pause/resume/stop controls, full-width navigation buttons with real-time record count badges (`Dashboard`, `Entries`, `Projects`, `Reports`, `Settings`), and bottom utility footer (theme toggle, floating mini-timer, system tray minimize, and keyboard shortcuts).
  - **Breadcrumb Header Banner:** Clean contextual breadcrumb header at the top of the canvas showing view icon, title, and descriptive subtitle.
  - **Widescreen Multi-Column Canvas (eliminates empty space):**
    - **Projects:** Balanced 2-column layout pairing the Project Directory and status table on the left (~62%) with the New/Edit Project Form on the right (~38%).
    - **Entries:** Table columns dynamically stretch; Description column expands to fill the entire remaining screen width, gracefully anchoring Actions to the far right.
    - **Dashboard:** 2-column executive hub pairing Omnibox, active Stopwatch/Pomodoro timer, and Today's Activity timeline on the left (~52%) with dynamically scaled Weekly Overview bar charts and Quick Add form on the right (~48%).
    - **Reports & Export:** 2-column analytics split pairing Project Distribution charts and multi-format Export options on the left with Report Summary KPIs and Project Breakdown table on the right, followed by a full-width expanding Entry Preview table.
    - **Settings:** 2-column balanced grid pairing Profile & Tracking Preferences with Data & Backup and Keyboard Shortcuts reference.
- **Ultra-Wide Content Framing & Material 3 Page Tabs (v0.13.0):**
  - **Centered max-width canvas:** Every page caps its content at 1400px and centers it, so ultra-wide and multi-monitor windows no longer stretch forms, filters, and tables from edge to edge
  - **Entries tabs:** `☰ List` and `▦ Weekly Grid` are now a Material 3 tab bar instead of a small toggle
  - **Reports tabs:** `Overview` (KPIs + distribution), `Breakdown` (project/client table), `Export` (format options), and `Preview` (filtered entries) — each tab uses the wide layout instead of one long scroll
  - **Settings tabs:** `Preferences` (profile + shortcuts in two columns on wide screens), `Data & Backup`, and `About`
  - **Material 3 components:** Tabs, buttons, switches, checkboxes, chips, and outlined text fields come from `egui-material3`, themed with the app's blue/slate palette in both light and dark mode
- **Material 3 Form Redesign (v0.14.0):**
  - **Entry forms** (Entries + Dashboard Quick Add): outlined text fields for start/end, duration, description, and tags; toggle switch for billable; quick-fill suggestion chips for recent tasks
  - **Project form:** outlined name/client/rate/budget fields, budget preset chips (20h/40h/80h/160h), and an Active switch
  - **Settings:** switches for dark theme and tray behaviour, preset chips for daily target and Pomodoro durations, and Material actions for save/backup/restore
  - **Reports:** preset date chips, export toggles, and primary/secondary Material buttons for CSV, Excel, PDF, and Markdown exports
  - Primary actions use filled buttons, update actions use tonal buttons, and low-emphasis actions use text buttons, keeping one visual hierarchy across the app
- **Adaptive Shell & System Theme (v0.16.0):**
  - **Breakpoint-aware navigation rail:** labelled 280 dp rail at 1200 dp+, icon-only 96 dp rail at 840–1199 dp, and a compact 72 dp rail below 840 dp, all driven by `Breakpoint::rail_width()`
  - **Insights rail at 1600 dp+:** a 320 dp right pane with today's target progress, billable hours and estimated value, recent sessions, recent projects, and quick actions
  - **System theme mode:** Settings → Appearance offers Light, Dark, and System; System reads the Windows app theme (`AppsUseLightTheme`) and picks up changes within a couple of seconds
  - Form cards, filter bars, and bulk-action bars now render with the shared `card_frame` / `bar_frame` tokens
- **Themeable Design Tokens & Appearance Settings (v0.15.0):**
  - **Design tokens:** `src/ui/theme.rs` is the single source for Material 3 colour roles, accent presets, breakpoints, layout constants, and card/dialog/bar frames; egui visuals, spacing, and the `egui-material3` palette are built from these tokens
  - **Appearance settings:** a new Settings → Appearance card switches between **Classic** (original blue/slate look) and **Material 3 (full)**, picks an accent preset (Blue, Teal, Violet, Green, Amber), and shows a live preview
  - Hard-coded colours in the views, form cards, charts, and navigation rail now resolve through token lookups, so both styles stay consistent
- **High-Visibility Forms, Input Styling & Full Page Scrolling (v0.10.0):**
  - **Full-page vertical scrolling:** Settings, Dashboard, Entries, Projects, and Reports wrap seamlessly in vertical scroll areas, guaranteeing all settings, data/backup options, and lower controls are easily accessible regardless of window size
  - **Flawless Tab Readability:** Custom-rendered navigation tabs in both Dark and Light modes featuring vibrant accent pills with bold white/navy text and smooth hover illumination
  - **Compact, Ergonomic Form Layout:** Streamlined 5-row entry form placing Date & Project side-by-side, Start/End/Duration inline, and full-width task descriptions
  - **Collapsible Form Toggle:** One-click `▼ Hide Form` / `➕ Show Form (Ctrl+N)` toggle to maximize table view space when reviewing long entry lists
  - **Clear Data Provenance & Context Banners:** Explicit banner indicating whether you are editing an existing record (`✏ Currently editing Entry #ID`), working with a clone (`📋 Cloned from Entry #ID`), or logging a fresh entry
  - **Entry Table Identifiers:** New `#ID` column, active row highlight, and detailed Project labels (`Project (Client)`) so users always know exactly which record is which and where the data came from
  - **Neat Quick-Fill Chips:** Recent task suggestions formatted as compact `⚡` chips truncated to 22 characters with full hover tooltips, avoiding haphazard wrapping
  - Dedicated elevated Form Cards (`render_form_card`) with subtle Slate borders, rounded corners, and icon badges across Entries, Projects, Omnibox, and Settings
  - High-contrast input styling: deep Slate-950 (dark theme) / pure white (light theme) backgrounds, defined inactive borders, and glowing active focus rings
  - Input hint text and clear sizing across all form fields (start/end times, durations, descriptions, tags, project rates, and budgets)
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
  - Auto-saving Light/Dark toggle in the navigation rail footer, plus Light/Dark/System in Settings → Appearance
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

## Build Release & Windows Installation

### Build release binary
```powershell
cargo build --release
```
The optimized executable is created at `target\release\timesheet.exe`.

### 1-Click Windows Installer & Distribution Packages
Run the packaging script to generate the distribution files:
```powershell
powershell -ExecutionPolicy Bypass -File scripts\package.ps1
```
This produces:
- **Single-File Setup:** `dist\Timesheet-Setup.exe` (all-in-one standalone installer containing all files inside it)
- **Extracted Folder:** `dist\Timesheet-v0.11.1-Windows-x64\`
- **Zip Archive:** `dist\Timesheet-v0.11.1-Windows-x64.zip`

**How to Install on Windows:**
1. **Single-file Installer (Easiest):** Simply double-click **`dist\Timesheet-Setup.exe`**.
   - It contains `timesheet.exe`, uninstaller, and documentation embedded inside it.
   - Extracts all files into your local install directory (`%LOCALAPPDATA%\Programs\Timesheet\`).
   - Automatically generates Windows Start Menu and Desktop shortcuts.
   - Registers Timesheet in Windows Settings under "Installed Apps" / "Add or Remove Programs".
   - Launches the app immediately!
2. **Script Install:** Inside the distribution folder or zip, double-click `install.bat` (or run `powershell -ExecutionPolicy Bypass -File scripts\install.ps1`).
3. **Portable Mode:** You can also run `timesheet.exe` directly from any folder or USB drive without installing.
4. **Uninstall:** Double-click `uninstall.bat` in the install directory or uninstall from Windows Settings. Your database at `%APPDATA%\timesheet\` is preserved so you never lose your logged hours.

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
- `src/ui/` – egui app and views (dashboard, entries, weekly_grid, projects, reports, settings, Material 3 helpers, and `theme.rs` design tokens)
- `docs/design/ui-ux-wireframe.md` – Material 3 UI/UX design proposal, screen wireframes, and adaptive layout reference
- `src/main.rs` – application entry point
