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
    pub date: chrono::NaiveDate,
    pub project_id: Option<i64>,
    pub start_time: String,
    pub description: String,
    pub billable: bool,
    pub tags: String,
}

pub struct TimesheetApp {
    pub db: Database,
    pub tab: Tab,
    pub status: String,
    pub projects_cache: Vec<Project>,
    pub filter: EntryFilter,
    pub search_query: String,
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
    // project form state
    pub proj_name: String,
    pub proj_client: String,
    pub proj_rate: String,
    pub proj_color: Option<String>,
    pub proj_active: bool,
    pub editing_project_id: Option<i64>,
    // bulk/inline helpers
    pub selected_entry_ids: HashSet<i64>,
    pub bulk_tags: String,
    pub bulk_billable: bool,
    pub focus_new_entry: bool,
    pub inline_edit: Option<InlineEditState>,
    // timer
    pub timer: Option<TimerState>,
    // settings & confirmations
    pub settings: Settings,
    pub confirm: Option<ConfirmState>,
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
            proj_name: String::new(),
            proj_client: String::new(),
            proj_rate: String::new(),
            proj_color: None,
            proj_active: true,
            editing_project_id: None,
            selected_entry_ids: HashSet::new(),
            bulk_tags: String::new(),
            bulk_billable: true,
            focus_new_entry: false,
            inline_edit: None,
            timer: None,
            settings,
            confirm: None,
        };
        app.load_persisted_timer();
        app.apply_theme(&_cc.egui_ctx);
        Ok(app)
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        if self.settings.dark_theme {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
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

    fn load_persisted_timer(&mut self) {
        if let Some(p) = timer_persist::load() {
            let now = timer_persist::now_unix_ms();
            let elapsed_ms = (now - p.running_since_unix_ms).max(0) as u64;
            self.timer = Some(TimerState {
                running_since: Instant::now() - Duration::from_millis(elapsed_ms),
                date: p.date,
                project_id: p.project_id,
                start_time: p.start_time,
                description: p.description,
                billable: p.billable,
                tags: p.tags,
            });
            self.status = "Resumed running timer from previous session".to_string();
        }
    }

    pub fn persist_timer(&self) {
        if let Some(t) = &self.timer {
            let elapsed_ms = t.running_since.elapsed().as_millis() as i64;
            let running_since_unix_ms = timer_persist::now_unix_ms() - elapsed_ms;
            let p = PersistedTimer {
                running_since_unix_ms,
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
        self.timer = Some(TimerState {
            running_since: Instant::now(),
            date: chrono::Local::now().date_naive(),
            project_id,
            start_time: crate::domain::now_hhmm(),
            description: description.clone(),
            billable,
            tags: tags.clone(),
        });
        self.persist_timer();
        self.status = format!(
            "Timer started{}",
            if description.is_empty() {
                String::new()
            } else {
                format!(": {description}")
            }
        );
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

    pub fn stop_and_save_timer(&mut self) -> bool {
        let Some(timer) = self.timer.clone() else {
            return false;
        };
        let stop_time = crate::domain::now_hhmm();
        let duration = crate::domain::duration_minutes_from_times(&timer.start_time, &stop_time)
            .unwrap_or(((timer.running_since.elapsed().as_secs() + 30) / 60) as i64)
            .max(1);
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
                let d = Duration::from_secs(delta_minutes as u64 * 60);
                t.running_since = t.running_since.checked_sub(d).unwrap_or(t.running_since);
            } else if delta_minutes < 0 {
                let d = Duration::from_secs(delta_minutes.unsigned_abs() * 60);
                let max_advance = Instant::now();
                t.running_since = (t.running_since + d).min(max_advance);
            }
            self.persist_timer();
        }
    }

    pub fn clone_entry_to_form(&mut self, e: &crate::db::models::Entry) {
        self.editing_entry_id = None;
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
}

impl eframe::App for TimesheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global shortcuts
        let (ctrl_1, ctrl_2, ctrl_3, ctrl_4, ctrl_5, ctrl_space) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::Num1),
                i.modifiers.command && i.key_pressed(egui::Key::Num2),
                i.modifiers.command && i.key_pressed(egui::Key::Num3),
                i.modifiers.command && i.key_pressed(egui::Key::Num4),
                i.modifiers.command && i.key_pressed(egui::Key::Num5),
                i.modifiers.command && i.key_pressed(egui::Key::Space),
            )
        });
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

        if self.timer.is_some() {
            ctx.request_repaint_after(Duration::from_millis(1000));
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.tab, Tab::Entries, "📝 Entries");
                ui.selectable_value(&mut self.tab, Tab::Projects, "📁 Projects");
                ui.selectable_value(&mut self.tab, Tab::Reports, "📈 Reports & Export");
                ui.selectable_value(&mut self.tab, Tab::Settings, "⚙ Settings");

                ui.separator();

                // Running timer pill in top bar
                if let Some(timer) = &self.timer {
                    let elapsed = timer.running_since.elapsed();
                    let total_secs = elapsed.as_secs();
                    let hrs = total_secs / 3600;
                    let mins = (total_secs % 3600) / 60;
                    let secs = total_secs % 60;
                    let time_str = if hrs > 0 {
                        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
                    } else {
                        format!("{:02}:{:02}", mins, secs)
                    };

                    let proj_idx = self.project_idx_for_id(timer.project_id);
                    let proj = proj_idx.and_then(|i| self.projects_cache.get(i));
                    let proj_name = proj.map(|p| p.name.as_str()).unwrap_or("(none)");
                    let dot_color = crate::ui::project_color_32(
                        proj.and_then(|p| p.color.as_deref()),
                        proj_idx.unwrap_or(0),
                    );

                    crate::ui::render_color_dot(ui, dot_color);
                    let clicked = ui
                        .button(
                            egui::RichText::new(format!("⏱ {} ({})", time_str, proj_name))
                                .color(egui::Color32::from_rgb(239, 68, 68))
                                .strong(),
                        )
                        .on_hover_text("Timer is active. Click to open Dashboard.")
                        .clicked();
                    if clicked {
                        self.tab = Tab::Dashboard;
                    }

                    if ui
                        .small_button("Stop")
                        .on_hover_text("Stop and save timer entry")
                        .clicked()
                    {
                        self.stop_and_save_timer();
                    }

                    ui.separator();
                }

                // Push theme toggle to right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_icon = if self.settings.dark_theme {
                        "☀ Light"
                    } else {
                        "🌙 Dark"
                    };
                    if ui.small_button(theme_icon).clicked() {
                        self.settings.dark_theme = !self.settings.dark_theme;
                        self.save_settings();
                        self.apply_theme(ctx);
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status_icon = if self.status.starts_with("Failed")
                    || self.status.starts_with("Cannot")
                    || self.status.starts_with("Invalid")
                {
                    "⚠ "
                } else if self.status.contains("saved")
                    || self.status.contains("created")
                    || self.status.contains("updated")
                    || self.status.contains("deleted")
                {
                    "✓ "
                } else {
                    "ℹ "
                };
                ui.label(format!("{}{}", status_icon, self.status));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak("Ctrl+1..5: Tabs | Ctrl+N: New | Ctrl+Space: Timer | Esc: Clear");
                });
            });
        });

        self.apply_theme(ctx);

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Dashboard => crate::ui::dashboard::show(self, ui),
            Tab::Entries => crate::ui::entries::show(self, ui),
            Tab::Projects => crate::ui::projects::show(self, ui),
            Tab::Reports => crate::ui::reports::show(self, ui),
            Tab::Settings => crate::ui::settings_view::show(self, ui),
        });

        self.show_confirm_dialog(ctx);
    }
}
