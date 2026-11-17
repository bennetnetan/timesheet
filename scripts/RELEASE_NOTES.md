## Timesheet v0.11.1 (Windows Desktop)

Offline-first, single-user timesheet and productivity app built with Rust, `eframe`/`egui`, and SQLite.

### 🚀 Downloads & Installation

| Asset | Description | Size |
| :--- | :--- | :--- |
| **`Timesheet-Setup.exe`** | **Recommended:** All-in-one standalone Windows installer. Installs to `%LOCALAPPDATA%\Programs\Timesheet`, creates Start Menu & Desktop shortcuts, registers in Windows Settings (*Apps & Features*), and launches immediately (requires zero admin rights / no UAC). | ~5.0 MB |
| **`Timesheet-v0.11.1-Windows-x64.zip`** | Portable ZIP package containing `timesheet.exe`, `install.bat`, and `uninstall.bat`. Extract anywhere or run from a USB drive. | ~5.0 MB |
| **`timesheet.exe`** | Direct raw standalone executable. Zero dependencies, statically bundled SQLite. | ~10.8 MB |

---

### ✨ What's New in v0.11.1

- **Single-File Setup Installer (`Timesheet-Setup.exe`):**
  - Standalone setup executable containing the application and scripts embedded inside it.
  - Automatically manages installation, shortcuts, and registration in Windows Apps & Features.
  - Built-in clean uninstallation that keeps your offline SQLite database intact.
- **AppFlowy-Style Left Navigation Sidebar (225px):**
  - Workspace branding with live active timer card.
  - Full-width navigation buttons with real-time record count badges (`Dashboard`, `Entries`, `Projects`, `Reports`, `Settings`).
  - Quick-access utility footer: Light/Dark theme toggle, floating mini-timer, system tray minimization, and shortcuts modal.
- **Balanced Multi-Column Widescreen Layouts:**
  - **Projects:** 2-column split (Projects directory and status table on the left, New/Edit Project Form on the right).
  - **Entries:** Dynamically expanding Description column filling wide monitors and anchoring row actions to the right boundary.
  - **Dashboard:** Executive 2-column hub pairing Omnibox, active Stopwatch/Pomodoro timer, and Today's Activity timeline with dynamically scaled Weekly Overview bar charts and Quick Add form.
  - **Reports & Export:** 2-column analytics split pairing Project Distribution charts and multi-format Export options (CSV, Excel `.xlsx`, PDF, Markdown) with Report Summary KPIs and Project Breakdown table.
  - **Settings:** 2-column balanced grid pairing Profile & Tracking Preferences with Data & Backup and Keyboard Shortcuts reference.
- **Project Budget Burn-Down Tracking:**
  - Configurable budget hours with live color-coded progress bars (<80% green, 80–100% amber, >100% red alert).
- **Floating Always-on-Top Mini-Timer (`Ctrl+M`):**
  - Compact 380×120 floating widget that stays above IDEs and browser windows while working.
- **Windows System Tray Integration:**
  - Minimize to tray on close, live elapsed time in tooltip, right-click context menu.
- **Quick Task Omnibox:**
  - Single-line natural language parser with live project and tag autocomplete (`Task description @Project #tags 45m`).
- **Pomodoro Focus Timer Mode:**
  - Dual-mode timer switching between Stopwatch and Pomodoro Focus sessions (25 min focus / 5 min break) with countdown readouts and animated progress bar.
- **Weekly Matrix Grid View:**
  - Switch between chronological list view and a compact weekly timesheet matrix with inline cell editing.
