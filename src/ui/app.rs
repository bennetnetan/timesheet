use crate::db::Database;
use crate::db::models::{Entry, EntryFilter, Project};
use crate::settings::{self, Settings};
use crate::timer_persist::{self, PersistedTimer};
use anyhow::Result;
use chrono::Timelike;
use eframe::egui;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Tab {
    Dashboard,
    Entries,
    Projects,
    Reports,
    Settings,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SortColumn {
    Date,
    Project,
    Duration,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteEntry(i64),
    DeleteEntries(Vec<i64>),
    DeleteProject(i64),
    RestoreBackup,
}

pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
    pub backup_path: Option<std::path::PathBuf>,
}

#[derive(Clone)]
pub struct TimerState {
    pub running_since: Instant,
    pub accumulated_secs: u64,
    pub is_paused: bool,
    pub pomodoro_target_secs: Option<u64>,
    pub is_pomodoro_break: bool,
    pub date: chrono::NaiveDate,
    pub project_id: Option<i64>,
    pub start_time: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

impl TimerState {
    pub fn total_elapsed_secs(&self) -> u64 {
        if self.is_paused {
            self.accumulated_secs
        } else {
            self.accumulated_secs + self.running_since.elapsed().as_secs()
        }
    }
}

pub struct TimesheetApp {
    pub db: Database,
    pub tab: Tab,
    pub status: String,
    pub projects_cache: Vec<Project>,
    pub filter: EntryFilter,
    pub search_query: String,
    pub filter_tag: Option<String>,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,

    // entry form state
    pub entry_date: chrono::NaiveDate,
    pub entry_project_idx: Option<usize>,
    pub entry_start: String,
    pub entry_end: String,
    pub entry_duration: String,
    pub entry_description: String,
    pub entry_billable: bool,
    pub entry_tags: String,
    pub editing_entry_id: Option<i64>,
    pub show_entry_form: bool,
    pub cloned_from_entry_id: Option<i64>,
    // project form state
    pub proj_name: String,
    pub proj_client: String,
    pub proj_rate: String,
    pub proj_budget: String,
    pub proj_color: Option<String>,
    pub proj_active: bool,
    pub editing_project_id: Option<i64>,
    pub proj_search: String,
    pub proj_active_only: bool,
    // compact mini-timer widget mode
    pub is_compact_mode: bool,
    // bulk/inline helpers
    pub selected_entry_ids: HashSet<i64>,
    pub bulk_tags: String,
    pub bulk_billable: bool,
    pub focus_new_entry: bool,
    pub inline_edit: Option<InlineEditState>,
    // timer & omnibox
    pub timer: Option<TimerState>,
    pub omnibox_input: String,
    pub pomodoro_mode: bool,
    // weekly grid view
    pub weekly_view: bool,
    pub weekly_grid_week_offset: i32,
    pub weekly_cell_editing: Option<(chrono::NaiveDate, Option<i64>)>,
    pub weekly_cell_input: String,
    // system tray & idle detection
    pub tray: Option<crate::tray::TrayHandle>,
    pub window_visible: bool,
    pub last_input_time: std::time::Instant,
    pub idle_prompt_shown: bool,
    // settings, modals & confirmations
    pub settings: Settings,
    pub confirm: Option<ConfirmState>,
    pub show_shortcuts_modal: bool,
}

#[derive(Clone)]
pub struct InlineEditState {
    pub id: i64,
    pub date: chrono::NaiveDate,
    pub project_idx: Option<usize>,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

impl TimesheetApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let db = Database::init()?;
        let projects_cache = db.list_projects(true).unwrap_or_default();
        let today = chrono::Local::now().date_naive();
        let settings = settings::load();
        let mut app = Self {
            db,
            tab: Tab::Dashboard,
            status: "Ready".to_string(),
            projects_cache,
            filter: EntryFilter::default(),
            search_query: String::new(),
            filter_tag: None,
            sort_column: SortColumn::Date,
            sort_ascending: false,
            entry_date: today,
            entry_project_idx: None,
            entry_start: String::new(),
            entry_end: String::new(),
            entry_duration: String::new(),
            entry_description: String::new(),
            entry_billable: true,
            entry_tags: String::new(),
            editing_entry_id: None,
            show_entry_form: true,
            cloned_from_entry_id: None,
            proj_name: String::new(),
            proj_client: String::new(),
            proj_rate: String::new(),
            proj_budget: String::new(),
            proj_color: None,
            proj_active: true,
            editing_project_id: None,
            proj_search: String::new(),
            proj_active_only: false,
            is_compact_mode: false,
            selected_entry_ids: HashSet::new(),
            bulk_tags: String::new(),
            bulk_billable: true,
            focus_new_entry: false,
            inline_edit: None,
            timer: None,
            omnibox_input: String::new(),
            pomodoro_mode: false,
            weekly_view: false,
            weekly_grid_week_offset: 0,
            weekly_cell_editing: None,
            weekly_cell_input: String::new(),
            tray: None,
            window_visible: true,
            last_input_time: std::time::Instant::now(),
            idle_prompt_shown: false,
            settings,
            confirm: None,
            show_shortcuts_modal: false,
        };
        app.load_persisted_timer();
        app.apply_theme(&_cc.egui_ctx);
        // Build tray icon (non-fatal if it fails)
        match crate::tray::TrayHandle::build() {
            Ok(handle) => app.tray = Some(handle),
            Err(e) => eprintln!("Warning: could not create tray icon: {e}"),
        }
        Ok(app)
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        let mut visuals = if self.settings.dark_theme {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        if self.settings.dark_theme {
            // High contrast input background (Slate-950) so inputs clearly pop
            visuals.extreme_bg_color = egui::Color32::from_rgb(15, 23, 42);

            // Inactive inputs / widgets: distinct slate fill, visible border outline, bright legible text
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
            visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(71, 85, 105));
            visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240));
            visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

            // Hovered inputs: glowing blue border with white text
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(51, 65, 85);
            visuals.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(96, 165, 250));
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

            // Focused/Active inputs: prominent blue focus ring with white text
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(37, 99, 235);
            visuals.widgets.active.bg_stroke =
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(96, 165, 250));
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
            visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

            // Text selection / selectable pills: high contrast white on solid vibrant blue
            visuals.selection.stroke = egui::Stroke::new(1.5_f32, egui::Color32::WHITE);
            visuals.selection.bg_fill = egui::Color32::from_rgb(37, 99, 235);

            // Window and panel styling: subtle elevation difference
            visuals.window_fill = egui::Color32::from_rgb(24, 32, 47);
            visuals.panel_fill = egui::Color32::from_rgb(15, 20, 32);
        } else {
            // Light theme: crisp white inputs against soft slate panel
            visuals.extreme_bg_color = egui::Color32::WHITE;

            // Inactive inputs / widgets: clean soft slate border, dark readable text
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
            visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225));
            visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 41, 59));
            visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

            // Hovered inputs
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(241, 245, 249);
            visuals.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246));
            visuals.widgets.hovered.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 23, 42));
            visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

            // Focused/Active inputs: strong blue outline
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(224, 231, 255);
            visuals.widgets.active.bg_stroke =
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(37, 99, 235));
            visuals.widgets.active.fg_stroke =
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(30, 58, 138));
            visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

            // Selection
            visuals.selection.stroke =
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 58, 138));
            visuals.selection.bg_fill = egui::Color32::from_rgb(219, 234, 254);

            visuals.window_fill = egui::Color32::from_rgb(248, 250, 252);
            visuals.panel_fill = egui::Color32::from_rgb(241, 245, 249);
        }

        ctx.set_visuals(visuals);
    }

    pub fn save_settings(&mut self) {
        if let Err(e) = settings::save(&self.settings) {
            self.status = format!("Failed to save settings: {e}");
        } else {
            self.status = "Settings saved".to_string();
        }
    }

    pub fn show_confirm_dialog(&mut self, ctx: &egui::Context) {
        if self.confirm.is_none() {
            return;
        }
        let mut confirmed = false;
        let mut cancelled = false;
        {
            let state = self.confirm.as_ref().unwrap();
            let title = state.title.clone();
            let message = state.message.clone();
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(message);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            cancelled = true;
                        }
                        if ui.button("Confirm").clicked() {
                            confirmed = true;
                        }
                    });
                });
        }
        if cancelled {
            self.confirm = None;
        }
        if confirmed {
            let taken = self.confirm.take();
            let restore_path = taken.as_ref().and_then(|c| c.backup_path.clone());
            let action = taken.map(|c| c.action);
            if let Some(action) = action {
                match action {
                    ConfirmAction::DeleteEntry(id) => match self.db.delete_entry(id) {
                        Ok(_) => {
                            self.selected_entry_ids.remove(&id);
                            if self.inline_edit.as_ref().map(|x| x.id) == Some(id) {
                                self.inline_edit = None;
                            }
                            self.status = "Entry deleted".to_string();
                        }
                        Err(e) => self.status = format!("Delete failed: {e}"),
                    },
                    ConfirmAction::DeleteEntries(ids) => {
                        let mut deleted = 0;
                        for id in ids {
                            if self.db.delete_entry(id).is_ok() {
                                deleted += 1;
                            }
                        }
                        self.selected_entry_ids.clear();
                        self.status = format!(
                            "Deleted {} selected entr{}",
                            deleted,
                            if deleted == 1 { "y" } else { "ies" }
                        );
                    }
                    ConfirmAction::DeleteProject(id) => match self.db.delete_project(id) {
                        Ok(_) => {
                            self.status = "Project deleted".to_string();
                            self.refresh_projects();
                        }
                        Err(e) => self.status = format!("Delete failed: {e}"),
                    },
                    ConfirmAction::RestoreBackup => {
                        if let Some(path) = restore_path {
                            match crate::backup::stage_restore(&path) {
                                Ok(target) => {
                                    self.status = format!(
                                        "Backup restored to {} — please restart the app",
                                        target.display()
                                    );
                                }
                                Err(e) => self.status = format!("Restore failed: {e}"),
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn show_shortcuts_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_shortcuts_modal {
            return;
        }
        let mut is_open = true;
        let mut close_clicked = false;
        egui::Window::new("Keyboard Shortcuts & Help")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Global Hotkeys");
                egui::Grid::new("modal_shortcuts_global")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Ctrl+1 .. Ctrl+5");
                        ui.label("Switch tab (Dashboard, Entries, Projects, Reports, Settings)");
                        ui.end_row();

                        ui.strong("Ctrl+Space");
                        ui.label("Toggle active timer (Start / Stop & Save)");
                        ui.end_row();

                        ui.strong("Ctrl+M");
                        ui.label("Toggle floating always-on-top Mini-Timer widget");
                        ui.end_row();

                        ui.strong("Ctrl+N");
                        ui.label("Jump to and focus New Entry creation");
                        ui.end_row();

                        ui.strong("F1 or ?");
                        ui.label("Open / close this shortcuts guide");
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.heading("Entries View Shortcuts");
                egui::Grid::new("modal_shortcuts_entries")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Enter");
                        ui.label("Save current entry while editing form");
                        ui.end_row();

                        ui.strong("Esc");
                        ui.label("Cancel entry editing / clear selection");
                        ui.end_row();

                        ui.strong("Delete");
                        ui.label("Delete selected entries");
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.heading("Duration Input Syntax");
                egui::Grid::new("modal_shortcuts_duration")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Minutes");
                        ui.label("90  or  45m");
                        ui.end_row();

                        ui.label("Decimal hours");
                        ui.label("1.5  or  0.75");
                        ui.end_row();

                        ui.label("Clock format");
                        ui.label("1:30  or  0:45");
                        ui.end_row();

                        ui.label("Unit strings");
                        ui.label("1h30m,  2h,  45m");
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.heading("Quick Task Omnibox Syntax");
                egui::Grid::new("modal_shortcuts_omnibox")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Format");
                        ui.label("Task description @Project #tags duration");
                        ui.end_row();

                        ui.strong("Examples");
                        ui.label("Design logo @Acme #branding 1.5h\nFix auth crash @Mobile #bug 45m\nClient standup @Acme (starts live timer)");
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Got it").clicked() {
                        close_clicked = true;
                    }
                });
            });
        self.show_shortcuts_modal = is_open && !close_clicked;
    }

    fn load_persisted_timer(&mut self) {
        if let Some(p) = timer_persist::load() {
            let now = timer_persist::now_unix_ms();
            let additional_ms = if p.is_paused {
                0
            } else {
                (now - p.running_since_unix_ms).max(0) as u64
            };
            self.timer = Some(TimerState {
                running_since: Instant::now() - Duration::from_millis(additional_ms),
                accumulated_secs: p.accumulated_secs,
                is_paused: p.is_paused,
                pomodoro_target_secs: p.pomodoro_target_secs,
                is_pomodoro_break: p.is_pomodoro_break,
                date: p.date,
                project_id: p.project_id,
                start_time: p.start_time,
                description: p.description,
                billable: p.billable,
                tags: p.tags,
            });
            self.status = if p.is_paused {
                "Resumed paused timer from previous session".to_string()
            } else {
                "Resumed running timer from previous session".to_string()
            };
        }
    }

    pub fn persist_timer(&self) {
        if let Some(t) = &self.timer {
            let elapsed_ms = if t.is_paused {
                0
            } else {
                t.running_since.elapsed().as_millis() as i64
            };
            let running_since_unix_ms = timer_persist::now_unix_ms() - elapsed_ms;
            let p = PersistedTimer {
                running_since_unix_ms,
                accumulated_secs: t.accumulated_secs,
                is_paused: t.is_paused,
                pomodoro_target_secs: t.pomodoro_target_secs,
                is_pomodoro_break: t.is_pomodoro_break,
                date: t.date,
                project_id: t.project_id,
                start_time: t.start_time.clone(),
                description: t.description.clone(),
                billable: t.billable,
                tags: t.tags.clone(),
            };
            let _ = timer_persist::save(&p);
        }
    }

    pub fn clear_persisted_timer(&self) {
        let _ = timer_persist::clear();
    }

    pub fn export_dialog(
        &self,
        filter_name: &str,
        ext: &str,
        default_name: &str,
    ) -> rfd::FileDialog {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(filter_name, &[ext])
            .set_file_name(default_name);
        if !self.settings.default_export_dir.trim().is_empty() {
            dialog = dialog.set_directory(&self.settings.default_export_dir);
        }
        dialog
    }

    pub fn refresh_projects(&mut self) {
        if let Ok(p) = self.db.list_projects(true) {
            self.projects_cache = p;
        }
    }

    pub fn selected_project_id(&self) -> Option<i64> {
        self.entry_project_idx
            .and_then(|i| self.projects_cache.get(i))
            .map(|p| p.id)
    }

    pub fn clear_entry_form(&mut self) {
        self.entry_date = chrono::Local::now().date_naive();
        self.entry_project_idx = None;
        self.entry_start.clear();
        self.entry_end.clear();
        self.entry_duration.clear();
        self.entry_description.clear();
        self.entry_billable = true;
        self.entry_tags.clear();
        self.editing_entry_id = None;
        self.cloned_from_entry_id = None;
    }

    pub fn effective_entry_duration_minutes(&mut self) -> Option<i64> {
        if let Some(mins) =
            crate::domain::duration_minutes_from_times(&self.entry_start, &self.entry_end)
        {
            self.entry_duration = crate::domain::format_minutes(mins);
            return Some(mins);
        }
        crate::domain::parse_duration_to_minutes(&self.entry_duration)
    }

    pub fn project_idx_for_id(&self, id: Option<i64>) -> Option<usize> {
        id.and_then(|pid| self.projects_cache.iter().position(|p| p.id == pid))
    }

    pub fn find_overlaps(&self, candidate: &Entry, exclude_id: Option<i64>) -> Result<Vec<Entry>> {
        let filter = EntryFilter {
            from: Some(candidate.date),
            to: Some(candidate.date),
            project_id: None,
            billable: None,
        };
        let entries = self.db.list_entries(&filter)?;
        let c_start = candidate
            .start_time
            .as_deref()
            .and_then(crate::domain::parse_time_hhmm);
        let c_end = candidate
            .end_time
            .as_deref()
            .and_then(crate::domain::parse_time_hhmm);
        let (Some(cs), Some(ce)) = (c_start, c_end) else {
            return Ok(vec![]);
        };
        let c0 = cs.hour() as i64 * 60 + cs.minute() as i64;
        let mut c1 = ce.hour() as i64 * 60 + ce.minute() as i64;
        if c1 <= c0 {
            c1 += 24 * 60;
        }
        let mut overlaps = Vec::new();
        for e in entries {
            if Some(e.id) == exclude_id {
                continue;
            }
            let es = e
                .start_time
                .as_deref()
                .and_then(crate::domain::parse_time_hhmm);
            let ee = e
                .end_time
                .as_deref()
                .and_then(crate::domain::parse_time_hhmm);
            let (Some(es), Some(ee)) = (es, ee) else {
                continue;
            };
            let e0 = es.hour() as i64 * 60 + es.minute() as i64;
            let mut e1 = ee.hour() as i64 * 60 + ee.minute() as i64;
            if e1 <= e0 {
                e1 += 24 * 60;
            }
            if c0 < e1 && e0 < c1 {
                overlaps.push(e);
            }
        }
        Ok(overlaps)
    }

    pub fn clear_project_form(&mut self) {
        self.proj_name.clear();
        self.proj_client.clear();
        self.proj_rate.clear();
        self.proj_budget.clear();
        self.proj_color = None;
        self.proj_active = true;
        self.editing_project_id = None;
    }

    pub fn start_timer_with(
        &mut self,
        project_id: Option<i64>,
        description: String,
        billable: bool,
        tags: String,
    ) {
        let pomodoro_target_secs = if self.pomodoro_mode {
            Some(self.settings.pomodoro_focus_mins as u64 * 60)
        } else {
            None
        };
        self.timer = Some(TimerState {
            running_since: Instant::now(),
            accumulated_secs: 0,
            is_paused: false,
            pomodoro_target_secs,
            is_pomodoro_break: false,
            date: chrono::Local::now().date_naive(),
            project_id,
            start_time: crate::domain::now_hhmm(),
            description: description.clone(),
            billable,
            tags: tags.clone(),
        });
        self.persist_timer();
        self.status = if self.pomodoro_mode {
            format!(
                "🍅 Pomodoro focus started ({}m){}",
                self.settings.pomodoro_focus_mins,
                if description.is_empty() {
                    String::new()
                } else {
                    format!(": {description}")
                }
            )
        } else {
            format!(
                "Timer started{}",
                if description.is_empty() {
                    String::new()
                } else {
                    format!(": {description}")
                }
            )
        };
    }

    pub fn start_pomodoro_break(&mut self) {
        let break_mins = self.settings.pomodoro_break_mins;
        let proj_id = self.timer.as_ref().and_then(|t| t.project_id);
        self.timer = Some(TimerState {
            running_since: Instant::now(),
            accumulated_secs: 0,
            is_paused: false,
            pomodoro_target_secs: Some(break_mins as u64 * 60),
            is_pomodoro_break: true,
            date: chrono::Local::now().date_naive(),
            project_id: proj_id,
            start_time: crate::domain::now_hhmm(),
            description: "Break ☕".to_string(),
            billable: false,
            tags: "break".to_string(),
        });
        self.persist_timer();
        self.status = format!("☕ Started {}m break. Relax and stretch!", break_mins);
    }

    pub fn find_project_by_query(&self, query: &str) -> Option<usize> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return None;
        }
        self.projects_cache.iter().position(|p| {
            p.name.to_lowercase().starts_with(&q) || p.name.to_lowercase().contains(&q)
        })
    }

    pub fn submit_omnibox(&mut self, as_entry: bool) {
        let trimmed = self.omnibox_input.trim();
        if trimmed.is_empty() {
            return;
        }
        let parsed = crate::domain::parse_omnibox_input(trimmed);
        let proj_idx = parsed
            .project_query
            .as_deref()
            .and_then(|q| self.find_project_by_query(q));
        let proj_id = proj_idx
            .and_then(|i| self.projects_cache.get(i))
            .map(|p| p.id);
        let tags_str = parsed.tags.join(", ");

        if as_entry || parsed.duration_minutes.is_some() {
            let duration = parsed.duration_minutes.unwrap_or(30);
            let new_entry = crate::db::models::NewEntry {
                project_id: proj_id,
                date: chrono::Local::now().date_naive(),
                start_time: None,
                end_time: None,
                duration_minutes: duration,
                description: parsed.description.clone(),
                billable: true,
                tags: tags_str,
            };
            match self.db.create_entry(&new_entry) {
                Ok(_) => {
                    self.status = format!(
                        "Logged {} entry: {}",
                        crate::domain::format_minutes(duration),
                        if parsed.description.is_empty() {
                            "(no description)"
                        } else {
                            &parsed.description
                        }
                    );
                    self.omnibox_input.clear();
                }
                Err(e) => {
                    self.status = format!("Failed to log entry: {e}");
                }
            }
        } else {
            self.start_timer_with(proj_id, parsed.description, true, tags_str);
            self.omnibox_input.clear();
        }
    }

    pub fn start_timer(&mut self) {
        let pid = self.selected_project_id();
        let desc = self.entry_description.clone();
        let billable = self.entry_billable;
        let tags = self.entry_tags.clone();
        self.start_timer_with(pid, desc, billable, tags);
        if self.entry_start.trim().is_empty() {
            self.entry_start = crate::domain::now_hhmm();
        }
    }

    pub fn pause_timer(&mut self) {
        if let Some(t) = &mut self.timer {
            if !t.is_paused {
                t.accumulated_secs += t.running_since.elapsed().as_secs();
                t.is_paused = true;
                self.persist_timer();
                self.status = "Timer paused".to_string();
            }
        }
    }

    pub fn resume_timer(&mut self) {
        if let Some(t) = &mut self.timer {
            if t.is_paused {
                t.running_since = Instant::now();
                t.is_paused = false;
                self.persist_timer();
                self.status = "Timer resumed".to_string();
            }
        }
    }

    pub fn toggle_pause_timer(&mut self) {
        if let Some(t) = &self.timer {
            if t.is_paused {
                self.resume_timer();
            } else {
                self.pause_timer();
            }
        }
    }

    pub fn stop_and_save_timer(&mut self) -> bool {
        let Some(timer) = self.timer.clone() else {
            return false;
        };
        let stop_time = crate::domain::now_hhmm();
        let mut duration = ((timer.total_elapsed_secs() + 30) / 60).max(1) as i64;
        if self.settings.round_minutes > 1 {
            duration = crate::domain::round_duration_minutes(duration, self.settings.round_minutes)
                .max(self.settings.round_minutes as i64);
        }
        let new_entry = crate::db::models::NewEntry {
            project_id: timer.project_id,
            date: timer.date,
            start_time: Some(timer.start_time.clone()),
            end_time: Some(stop_time.clone()),
            duration_minutes: duration,
            description: timer.description.clone(),
            billable: timer.billable,
            tags: timer.tags.clone(),
        };
        let candidate = crate::db::models::Entry {
            id: -1,
            project_id: new_entry.project_id,
            date: new_entry.date,
            start_time: new_entry.start_time.clone(),
            end_time: new_entry.end_time.clone(),
            duration_minutes: new_entry.duration_minutes,
            description: new_entry.description.clone(),
            billable: new_entry.billable,
            tags: new_entry.tags.clone(),
        };
        match self.find_overlaps(&candidate, None) {
            Ok(overlaps) if !overlaps.is_empty() => {
                self.status = format!(
                    "Cannot save timer: overlaps {} existing entr{}",
                    overlaps.len(),
                    if overlaps.len() == 1 { "y" } else { "ies" }
                );
                false
            }
            _ => match self.db.create_entry(&new_entry) {
                Ok(_) => {
                    self.status = format!("Timer entry saved ({} min)", duration);
                    self.timer = None;
                    self.clear_persisted_timer();
                    true
                }
                Err(e) => {
                    self.status = format!("Failed to save timer entry: {e}");
                    false
                }
            },
        }
    }

    pub fn discard_timer(&mut self) {
        self.timer = None;
        self.clear_persisted_timer();
        self.status = "Timer discarded".to_string();
    }

    pub fn adjust_timer(&mut self, delta_minutes: i64) {
        if let Some(t) = &mut self.timer {
            if delta_minutes > 0 {
                t.accumulated_secs += delta_minutes as u64 * 60;
            } else if delta_minutes < 0 {
                let sub = delta_minutes.unsigned_abs() * 60;
                t.accumulated_secs = t.accumulated_secs.saturating_sub(sub);
            }
            self.persist_timer();
        }
    }

    pub fn recent_descriptions(&self, limit: usize) -> Vec<String> {
        let filter = EntryFilter::default();
        if let Ok(entries) = self.db.list_entries(&filter) {
            let mut seen = HashSet::new();
            let mut list = Vec::new();
            for e in entries {
                let d = e.description.trim().to_string();
                if !d.is_empty() && seen.insert(d.clone()) {
                    list.push(d);
                    if list.len() >= limit {
                        break;
                    }
                }
            }
            list
        } else {
            Vec::new()
        }
    }

    pub fn clone_entry_to_form(&mut self, e: &crate::db::models::Entry) {
        self.editing_entry_id = None;
        self.cloned_from_entry_id = Some(e.id);
        self.show_entry_form = true;
        self.entry_date = chrono::Local::now().date_naive();
        self.entry_project_idx = self.project_idx_for_id(e.project_id);
        self.entry_start.clear();
        self.entry_end.clear();
        self.entry_duration = crate::domain::format_minutes(e.duration_minutes);
        self.entry_description = e.description.clone();
        self.entry_billable = e.billable;
        self.entry_tags = e.tags.clone();
        self.focus_new_entry = true;
        self.tab = Tab::Entries;
        self.status = format!("Cloned entry #{} for today", e.id);
    }

    pub fn continue_entry_timer(&mut self, e: &crate::db::models::Entry) {
        self.start_timer_with(
            e.project_id,
            e.description.clone(),
            e.billable,
            e.tags.clone(),
        );
    }

    pub fn render_compact_mini_timer(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("⏱ Timesheet Mini").strong().size(12.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("🗗 Full")
                    .on_hover_text("Restore full window (1100x720) [Ctrl+M]")
                    .clicked()
                {
                    self.is_compact_mode = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1100.0, 720.0,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                        egui::WindowLevel::Normal,
                    ));
                }
                if self.tray.is_some()
                    && ui
                        .small_button("🔽")
                        .on_hover_text("Minimize to system tray")
                        .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    self.window_visible = false;
                }
            });
        });

        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);

        if let Some(timer) = self.timer.clone() {
            let total_secs = timer.total_elapsed_secs();
            let proj_idx = self.project_idx_for_id(timer.project_id);
            let proj = proj_idx.and_then(|i| self.projects_cache.get(i));
            let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("(no project)");
            let dot_color = crate::ui::project_color_32(
                proj.and_then(|p| p.color.as_deref()),
                proj_idx.unwrap_or(0),
            );

            let time_str = if let Some(target) = timer.pomodoro_target_secs {
                if timer.is_pomodoro_break {
                    let rem = target.saturating_sub(total_secs);
                    format!("☕ Break {:02}:{:02}", rem / 60, rem % 60)
                } else if total_secs < target {
                    let rem = target - total_secs;
                    format!("🍅 {:02}:{:02}", rem / 60, rem % 60)
                } else {
                    let over = total_secs - target;
                    format!("🔔 Done +{:02}:{:02}", over / 60, over % 60)
                }
            } else {
                let h = total_secs / 3600;
                let m = (total_secs % 3600) / 60;
                let s = total_secs % 60;
                if h > 0 {
                    format!("{:02}:{:02}:{:02}", h, m, s)
                } else {
                    format!("{:02}:{:02}", m, s)
                }
            };

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(time_str)
                        .size(22.0)
                        .monospace()
                        .strong()
                        .color(if timer.is_paused {
                            egui::Color32::from_rgb(245, 158, 11)
                        } else {
                            egui::Color32::from_rgb(59, 130, 246)
                        }),
                );

                ui.separator();
                crate::ui::render_color_dot(ui, dot_color);
                ui.label(egui::RichText::new(proj_name).strong());
            });

            ui.add_space(3.0);
            ui.horizontal(|ui| {
                if timer.is_paused {
                    if ui.small_button("▶ Resume").clicked() {
                        self.resume_timer();
                    }
                } else if ui.small_button("⏸ Pause").clicked() {
                    self.pause_timer();
                }

                if ui.small_button("⏹ Stop").clicked() {
                    self.stop_and_save_timer();
                }

                if ui.small_button("+5m").clicked() {
                    self.adjust_timer(5);
                }

                if !timer.description.is_empty() {
                    ui.weak(format!("• {}", timer.description));
                }
            });
        } else {
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("▶ Start").strong()).clicked() {
                    self.start_timer();
                }

                ui.label("Project:");
                let sel_idx = self.entry_project_idx;
                let proj_name = sel_idx
                    .and_then(|i| self.projects_cache.get(i))
                    .map(|p| p.name.as_str())
                    .unwrap_or("(select)");

                egui::ComboBox::from_id_salt("mini_proj_select")
                    .selected_text(proj_name)
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for (i, p) in self
                            .projects_cache
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.active)
                        {
                            if ui
                                .selectable_label(self.entry_project_idx == Some(i), &p.name)
                                .clicked()
                            {
                                self.entry_project_idx = Some(i);
                            }
                        }
                    });
            });
        }
    }
}

impl eframe::App for TimesheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global shortcuts
        let (ctrl_1, ctrl_2, ctrl_3, ctrl_4, ctrl_5, ctrl_space, ctrl_m, f1_or_help) =
            ctx.input(|i| {
                (
                    i.modifiers.command && i.key_pressed(egui::Key::Num1),
                    i.modifiers.command && i.key_pressed(egui::Key::Num2),
                    i.modifiers.command && i.key_pressed(egui::Key::Num3),
                    i.modifiers.command && i.key_pressed(egui::Key::Num4),
                    i.modifiers.command && i.key_pressed(egui::Key::Num5),
                    i.modifiers.command && i.key_pressed(egui::Key::Space),
                    i.modifiers.command && i.key_pressed(egui::Key::M),
                    i.key_pressed(egui::Key::F1)
                        || (i.modifiers.shift && i.key_pressed(egui::Key::Slash)),
                )
            });
        if f1_or_help {
            self.show_shortcuts_modal = !self.show_shortcuts_modal;
        }
        if ctrl_m {
            self.is_compact_mode = !self.is_compact_mode;
            if self.is_compact_mode {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(380.0, 120.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100.0, 720.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::Normal,
                ));
            }
        }
        if ctrl_1 {
            self.tab = Tab::Dashboard;
        }
        if ctrl_2 {
            self.tab = Tab::Entries;
        }
        if ctrl_3 {
            self.tab = Tab::Projects;
        }
        if ctrl_4 {
            self.tab = Tab::Reports;
        }
        if ctrl_5 {
            self.tab = Tab::Settings;
        }
        if ctrl_space {
            if self.timer.is_some() {
                self.stop_and_save_timer();
            } else {
                self.start_timer();
            }
        }

        if self.timer.as_ref().map(|t| !t.is_paused).unwrap_or(false) {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        // ── Tray icon event polling ───────────────────────────────────────────
        // We only process tray events when the tray is initialised.
        if self.tray.is_some() {
            let mut do_show = false;
            let mut do_dashboard = false;
            let mut do_stop_timer = false;
            let mut do_quit = false;

            // Poll tray icon click (left-click → show window)
            while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    do_show = true;
                }
            }

            // Poll tray menu events
            if let Some(tray) = &self.tray {
                while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    if event.id == tray.show_item_id {
                        do_show = true;
                    } else if event.id == tray.dashboard_item_id {
                        do_dashboard = true;
                        do_show = true;
                    } else if event.id == tray.stop_timer_item_id {
                        do_stop_timer = true;
                    } else if event.id == tray.quit_item_id {
                        do_quit = true;
                    }
                }
            }

            if do_stop_timer {
                self.stop_and_save_timer();
            }
            if do_dashboard {
                self.tab = Tab::Dashboard;
            }
            if do_show {
                // Toggle window visibility
                if self.window_visible {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    self.window_visible = false;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.window_visible = true;
                }
            }
            if do_quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // Update tray tooltip with current timer state
            if let Some(tray) = &self.tray {
                let tip = if let Some(ref t) = self.timer {
                    let secs = t.total_elapsed_secs();
                    let h = secs / 3600;
                    let m = (secs % 3600) / 60;
                    let s = secs % 60;
                    if h > 0 {
                        format!("⏱ {:02}:{:02}:{:02} running — Timesheet", h, m, s)
                    } else {
                        format!("⏱ {:02}:{:02} running — Timesheet", m, s)
                    }
                } else {
                    "Timesheet — no timer running".to_string()
                };
                tray.update_tooltip(&tip);
            }
        }

        // ── Idle detection ────────────────────────────────────────────────────
        // Track last time the user gave any pointer/keyboard input.
        let had_input = ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key { .. }
                        | egui::Event::PointerButton { .. }
                        | egui::Event::MouseWheel { .. }
                        | egui::Event::Text(_)
                )
            })
        });
        if had_input {
            self.last_input_time = std::time::Instant::now();
            self.idle_prompt_shown = false;
        }

        const IDLE_THRESHOLD_SECS: u64 = 15 * 60; // 15 minutes
        let timer_running = self.timer.as_ref().map(|t| !t.is_paused).unwrap_or(false);
        let idle_secs = self.last_input_time.elapsed().as_secs();
        if timer_running && idle_secs >= IDLE_THRESHOLD_SECS && !self.idle_prompt_shown {
            self.idle_prompt_shown = true;
            // Show a non-blocking idle alert window
        }
        // Idle alert dialog
        if self.idle_prompt_shown {
            let mut discard_idle_time = false;
            let mut continue_timer = false;
            let idle_mins = idle_secs / 60;
            egui::Window::new("⏳ Are you still working?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Your timer has been running for {} minutes without any input.",
                        idle_mins
                    ));
                    ui.label("Were you still working during this time?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button("✅ Yes, keep all time")
                            .on_hover_text("Dismiss this alert and keep the timer running")
                            .clicked()
                        {
                            continue_timer = true;
                        }
                        if ui
                            .button("⏹ No, stop & save now")
                            .on_hover_text("Stop the timer and save the entry")
                            .clicked()
                        {
                            continue_timer = true;
                            discard_idle_time = true;
                        }
                    });
                });
            if continue_timer && !discard_idle_time {
                self.idle_prompt_shown = false;
                self.last_input_time = std::time::Instant::now();
            }
            if discard_idle_time {
                self.stop_and_save_timer();
                self.idle_prompt_shown = false;
            }
        }

        self.apply_theme(ctx);

        if self.is_compact_mode {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_compact_mini_timer(ctx, ui);
            });
            return;
        }

        let dark = self.settings.dark_theme;
        egui::SidePanel::left("app_sidebar")
            .exact_width(225.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // 1. Workspace / App Brand
                ui.horizontal(|ui| {
                    let badge_fill = if dark {
                        egui::Color32::from_rgb(37, 99, 235)
                    } else {
                        egui::Color32::from_rgb(224, 231, 255)
                    };
                    egui::Frame::new()
                        .fill(badge_fill)
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("⏱").size(15.0));
                        });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Timesheet").strong().size(14.0));
                        let user = if self.settings.user_name.trim().is_empty() {
                            "My Workspace".to_string()
                        } else {
                            self.settings.user_name.trim().to_string()
                        };
                        ui.label(egui::RichText::new(user).weak().size(11.0));
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // 2. Active Timer Widget in Sidebar
                if let Some(timer) = self.timer.clone() {
                    let total_secs = timer.total_elapsed_secs();
                    let (icon, badge_color, time_str) =
                        if let Some(target) = timer.pomodoro_target_secs {
                            if timer.is_pomodoro_break {
                                let rem = target.saturating_sub(total_secs);
                                (
                                    "☕",
                                    egui::Color32::from_rgb(16, 185, 129),
                                    format!("Break {:02}:{:02}", rem / 60, rem % 60),
                                )
                            } else if total_secs < target {
                                let rem = target - total_secs;
                                (
                                    "🍅",
                                    egui::Color32::from_rgb(239, 68, 68),
                                    format!("{:02}:{:02}", rem / 60, rem % 60),
                                )
                            } else {
                                let over = total_secs - target;
                                (
                                    "🔔",
                                    egui::Color32::from_rgb(245, 158, 11),
                                    format!("+{:02}:{:02}", over / 60, over % 60),
                                )
                            }
                        } else {
                            let hrs = total_secs / 3600;
                            let mins = (total_secs % 3600) / 60;
                            let secs = total_secs % 60;
                            let s = if hrs > 0 {
                                format!("{:02}:{:02}:{:02}", hrs, mins, secs)
                            } else {
                                format!("{:02}:{:02}", mins, secs)
                            };
                            let col = if timer.is_paused {
                                egui::Color32::from_rgb(245, 158, 11)
                            } else {
                                egui::Color32::from_rgb(59, 130, 246)
                            };
                            ("⏱", col, s)
                        };

                    let proj_idx = self.project_idx_for_id(timer.project_id);
                    let (proj_name, dot_color) = if let Some(i) = proj_idx {
                        if let Some(p) = self.projects_cache.get(i) {
                            (
                                p.name.clone(),
                                crate::ui::project_color_32(p.color.as_deref(), i),
                            )
                        } else {
                            (
                                "(no project)".to_string(),
                                crate::ui::project_color_32(None, 0),
                            )
                        }
                    } else {
                        (
                            "(no project)".to_string(),
                            crate::ui::project_color_32(None, 0),
                        )
                    };

                    let card_bg = if dark {
                        egui::Color32::from_rgb(30, 41, 59)
                    } else {
                        egui::Color32::from_rgb(241, 245, 249)
                    };

                    egui::Frame::canvas(ui.style())
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, badge_color))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                crate::ui::render_color_dot(ui, dot_color);
                                ui.label(egui::RichText::new(&proj_name).strong().size(12.0));
                            });
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{} {}", icon, time_str))
                                        .size(16.0)
                                        .strong()
                                        .color(badge_color),
                                );
                                if timer.is_paused {
                                    ui.weak("[Paused]");
                                }
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if timer.is_paused {
                                    if ui.small_button("▶ Resume").clicked() {
                                        self.resume_timer();
                                    }
                                } else if ui.small_button("⏸ Pause").clicked() {
                                    self.pause_timer();
                                }
                                if ui.small_button("⏹ Stop").clicked() {
                                    self.stop_and_save_timer();
                                }
                            });
                        });
                    ui.add_space(8.0);
                } else {
                    let start_btn = egui::Button::new(
                        egui::RichText::new("▶ Start Live Timer")
                            .color(egui::Color32::WHITE)
                            .strong()
                            .size(12.0),
                    )
                    .fill(egui::Color32::from_rgb(37, 99, 235))
                    .corner_radius(egui::CornerRadius::same(6));

                    if ui
                        .add_sized([ui.available_width(), 28.0], start_btn)
                        .clicked()
                    {
                        self.tab = Tab::Dashboard;
                        self.start_timer();
                    }
                    ui.add_space(8.0);
                }

                // 3. Navigation List (AppFlowy style full-width items)
                ui.weak("NAVIGATION");
                ui.add_space(4.0);

                let (entries_count, projects_count) = self.db.counts().unwrap_or((0, 0));
                let nav_items = [
                    (Tab::Dashboard, "📊 Dashboard", None),
                    (Tab::Entries, "📝 Entries", Some(entries_count)),
                    (Tab::Projects, "📁 Projects", Some(projects_count)),
                    (Tab::Reports, "📈 Reports & Export", None),
                    (Tab::Settings, "⚙ Settings", None),
                ];

                for (tab_val, label, badge) in nav_items {
                    let is_active = self.tab == tab_val;
                    let (fill, text_col, stroke) = if is_active {
                        if dark {
                            (
                                egui::Color32::from_rgb(37, 99, 235),
                                egui::Color32::WHITE,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(96, 165, 250)),
                            )
                        } else {
                            (
                                egui::Color32::from_rgb(224, 231, 255),
                                egui::Color32::from_rgb(30, 58, 138),
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(99, 102, 241)),
                            )
                        }
                    } else {
                        (
                            egui::Color32::TRANSPARENT,
                            if dark {
                                egui::Color32::from_rgb(203, 213, 225)
                            } else {
                                egui::Color32::from_rgb(51, 65, 85)
                            },
                            egui::Stroke::NONE,
                        )
                    };

                    let w = ui.available_width();
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .color(text_col)
                            .strong()
                            .size(13.0),
                    )
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(egui::CornerRadius::same(6));

                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([w - if badge.is_some() { 36.0 } else { 0.0 }, 28.0], btn)
                            .clicked()
                        {
                            self.tab = tab_val;
                        }
                        if let Some(cnt) = badge {
                            let badge_bg = if is_active {
                                egui::Color32::from_rgba_premultiplied(255, 255, 255, 40)
                            } else if dark {
                                egui::Color32::from_rgb(51, 65, 85)
                            } else {
                                egui::Color32::from_rgb(226, 232, 240)
                            };
                            egui::Frame::new()
                                .fill(badge_bg)
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::symmetric(6, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(cnt.to_string())
                                            .size(10.5)
                                            .color(text_col),
                                    );
                                });
                        }
                    });
                    ui.add_space(2.0);
                }

                // 4. Spacer pushing footer to bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.weak(format!("v{}", env!("CARGO_PKG_VERSION")));
                        ui.separator();
                        let theme_icon = if dark { "☀ Light" } else { "🌙 Dark" };
                        if ui.small_button(theme_icon).clicked() {
                            self.settings.dark_theme = !self.settings.dark_theme;
                            self.save_settings();
                            self.apply_theme(ctx);
                        }
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .small_button("🗖 Mini")
                            .on_hover_text("Mini floating timer (Ctrl+M)")
                            .clicked()
                        {
                            self.is_compact_mode = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                                380.0, 120.0,
                            )));
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::AlwaysOnTop,
                            ));
                        }
                        if self.tray.is_some() && self.settings.minimize_to_tray {
                            if ui
                                .small_button("🔽 Tray")
                                .on_hover_text("Minimize to tray")
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                                self.window_visible = false;
                            }
                        }
                        if ui.small_button("⌨ Shortcuts").clicked() {
                            self.show_shortcuts_modal = true;
                        }
                    });
                    ui.add_space(4.0);
                    ui.separator();
                });
            });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let is_err = self.status.starts_with("Failed")
                    || self.status.starts_with("Cannot")
                    || self.status.starts_with("Invalid");
                let is_ok = self.status.contains("saved")
                    || self.status.contains("created")
                    || self.status.contains("updated")
                    || self.status.contains("deleted");

                let (status_icon, status_color) = if is_err {
                    ("⚠ ", egui::Color32::from_rgb(239, 68, 68))
                } else if is_ok {
                    ("✓ ", egui::Color32::from_rgb(16, 185, 129))
                } else {
                    (
                        "ℹ ",
                        if self.settings.dark_theme {
                            egui::Color32::from_rgb(148, 163, 184)
                        } else {
                            egui::Color32::from_rgb(71, 85, 105)
                        },
                    )
                };

                ui.label(
                    egui::RichText::new(format!("{}{}", status_icon, self.status))
                        .color(status_color),
                );

                if !self.status.is_empty() && self.status != "Ready" {
                    if ui
                        .small_button("✕")
                        .on_hover_text("Clear status message")
                        .clicked()
                    {
                        self.status = "Ready".to_string();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("❓ Help (F1)")
                        .on_hover_text("Keyboard shortcuts reference")
                        .clicked()
                    {
                        self.show_shortcuts_modal = true;
                    }
                    ui.weak("Ctrl+1..5: Tabs | Ctrl+N: New | Ctrl+Space: Timer | Ctrl+M: Mini | Esc: Clear");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (icon, title, subtitle) = match self.tab {
                Tab::Dashboard => ("📊", "Dashboard", "Daily tracking summary & activity feed"),
                Tab::Entries => (
                    "📝",
                    "Entries",
                    "Manage time entries, tags, and timesheet matrix",
                ),
                Tab::Projects => (
                    "📁",
                    "Projects",
                    "Client projects, budgets, and hourly rates",
                ),
                Tab::Reports => (
                    "📈",
                    "Reports & Export",
                    "Timesheet analytics and multi-format exports",
                ),
                Tab::Settings => (
                    "⚙",
                    "Settings",
                    "Preferences, backup, and desktop configuration",
                ),
            };

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(16.0));
                ui.heading(title);
                ui.separator();
                ui.label(egui::RichText::new(subtitle).weak().size(12.0));
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            match self.tab {
                Tab::Dashboard => crate::ui::dashboard::show(self, ui),
                Tab::Entries => crate::ui::entries::show(self, ui),
                Tab::Projects => crate::ui::projects::show(self, ui),
                Tab::Reports => crate::ui::reports::show(self, ui),
                Tab::Settings => crate::ui::settings_view::show(self, ui),
            }
        });

        self.show_confirm_dialog(ctx);
        self.show_shortcuts_dialog(ctx);

        // ── Hide-to-tray on window close ─────────────────────────────────────
        // Intercept the close request: hide instead of quitting when tray is active.
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.tray.is_some() && self.settings.minimize_to_tray {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.window_visible = false;
            self.status = "Minimized to tray — right-click the tray icon to restore".to_string();
        }
    }
}
