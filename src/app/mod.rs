//! Application state, event handling and actions. Drawing lives in `view.rs` and
//! `dialogs.rs`.

mod dialogs;
mod format;
mod view;

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use tokio::runtime::Handle;

use crate::data_store::{DataStore, DateRangeBounds, SpeedTest};
use crate::launch;
use crate::logger;
use crate::monitor::{Event, SpeedMonitor};
use crate::paths::AppPaths;
use crate::schedule::{self, Schedule};
use crate::settings::Settings;
use crate::updates::{self, UpdateInfo};
use crate::{CURRENT_VERSION, RELEASES_URL};

pub const HISTORY_WINDOW: usize = 100;
pub const TABLE_ROWS: usize = 20;
pub const CHART_POINTS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleType {
    Interval,
    Cron,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Running,
    Sleeping,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartMode {
    Last20,
    LastDay,
    Last7,
    Range,
}

impl ChartMode {
    pub fn subtitle(self) -> &'static str {
        match self {
            ChartMode::Last20 => "(last twenty data points)",
            ChartMode::LastDay => "(last day)",
            ChartMode::Last7 => "(last 7 days)",
            ChartMode::Range => "(date range)",
        }
    }
}

/// A simple message box. `download_url` adds a "Download Update" button.
#[derive(Debug, Clone)]
pub struct Alert {
    pub title: String,
    pub message: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClearDataDialog {
    pub date: jiff::civil::Date,
    pub hour: u32,
    pub minute: u32,
    pub earliest: DateTime<Local>,
    pub latest: DateTime<Local>,
    pub preview: Option<ClearPreview>,
}

#[derive(Debug, Clone)]
pub struct ClearPreview {
    pub lines: Vec<(String, bool)>,
    pub can_clear: bool,
}

#[derive(Debug, Clone)]
pub struct ExportDialog {
    pub start: jiff::civil::Date,
    pub end: jiff::civil::Date,
    pub min: jiff::civil::Date,
    pub max: jiff::civil::Date,
    pub record_count: usize,
    pub has_data: bool,
}

pub struct App {
    pub(crate) handle: Handle,
    pub(crate) paths: AppPaths,
    pub(crate) store: Arc<Mutex<DataStore>>,
    pub(crate) monitor: SpeedMonitor,
    pub(crate) events: Receiver<Event>,
    pub(crate) event_tx: Sender<Event>,
    pub(crate) settings: Settings,

    // Schedule inputs
    pub(crate) schedule_type: ScheduleType,
    pub(crate) interval_minutes: u32,
    pub(crate) interval_text: String,
    pub(crate) cron_expression: String,
    pub(crate) auto_start: bool,
    pub(crate) launch_at_startup: bool,

    // Monitoring state
    pub(crate) is_monitoring: bool,
    pub(crate) current_schedule: Option<Schedule>,
    pub(crate) status_text: String,
    pub(crate) status_kind: StatusKind,
    pub(crate) test_running: bool,
    pub(crate) test_now_busy: bool,
    pub(crate) start_pending: bool,

    // Data shown in the UI
    pub(crate) recent: Vec<SpeedTest>,
    pub(crate) latest: Option<SpeedTest>,

    // Chart
    pub(crate) chart_mode: ChartMode,
    pub(crate) chart_data: Vec<SpeedTest>,
    pub(crate) chart_subtitle: String,
    pub(crate) chart_start: jiff::civil::Date,
    pub(crate) chart_end: jiff::civil::Date,
    pub(crate) chart_min: jiff::civil::Date,
    pub(crate) chart_max: jiff::civil::Date,

    // Updates
    pub(crate) update_info: Option<UpdateInfo>,
    pub(crate) update_dismissed: bool,
    pub(crate) update_check_busy: bool,

    // Dialogs
    pub(crate) alerts: VecDeque<Alert>,
    pub(crate) confirm_clear_history: bool,
    pub(crate) clear_dialog: Option<ClearDataDialog>,
    pub(crate) export_dialog: Option<ExportDialog>,
    pub(crate) about_open: bool,
    pub(crate) debug_log: Option<String>,
}

impl App {
    pub fn new(
        ctx: &egui::Context,
        handle: Handle,
        paths: AppPaths,
        store: Arc<Mutex<DataStore>>,
    ) -> Self {
        view::install_style(ctx);

        let (event_tx, events) = std::sync::mpsc::channel();
        let monitor = SpeedMonitor::new(
            handle.clone(),
            Arc::clone(&store),
            event_tx.clone(),
            ctx.clone(),
        );

        let mut settings = Settings::load(&paths.settings_file);
        log::info!("Settings loaded: {settings:?}");
        if !settings.legacy_cleanup_done {
            crate::legacy::remove_electron_leftovers(&paths.data_dir);
            settings.legacy_cleanup_done = true;
            settings.save(&paths.settings_file);
        }
        let schedule_type = if settings.schedule_type == "cron" {
            ScheduleType::Cron
        } else {
            ScheduleType::Interval
        };
        let interval_minutes = settings.interval.clamp(1, 60);
        let cron_expression = if settings.cron_expression.trim().is_empty() {
            crate::settings::DEFAULT_CRON_EXPRESSION.to_string()
        } else {
            settings.cron_expression.clone()
        };
        let launch_at_startup = launch::get_launch_at_startup();
        let today = format::today();

        let mut app = Self {
            handle,
            paths,
            store,
            monitor,
            events,
            event_tx,
            auto_start: settings.auto_start,
            settings,
            schedule_type,
            interval_minutes,
            interval_text: interval_minutes.to_string(),
            cron_expression,
            launch_at_startup,
            is_monitoring: false,
            current_schedule: None,
            status_text: "Stopped".to_string(),
            status_kind: StatusKind::Stopped,
            test_running: false,
            test_now_busy: false,
            start_pending: false,
            recent: Vec::new(),
            latest: None,
            chart_mode: ChartMode::Last20,
            chart_data: Vec::new(),
            chart_subtitle: ChartMode::Last20.subtitle().to_string(),
            chart_start: today,
            chart_end: today,
            chart_min: today,
            chart_max: today,
            update_info: None,
            update_dismissed: false,
            update_check_busy: false,
            alerts: VecDeque::new(),
            confirm_clear_history: false,
            clear_dialog: None,
            export_dialog: None,
            about_open: false,
            debug_log: None,
        };

        let load_warning = app.with_store_mut(|store| store.take_load_warning());
        if let Some(warning) = load_warning {
            app.alert("Speed test history", warning);
        }
        app.load_historical_data();
        app.maybe_check_for_updates();

        if app.auto_start {
            log::info!("Auto-start is enabled, starting monitoring...");
            app.start_pending = true;
        }

        app
    }

    // ----- Data -----

    fn with_store<T>(&self, f: impl FnOnce(&DataStore) -> T) -> T {
        match self.store.lock() {
            Ok(store) => f(&store),
            Err(poisoned) => f(&poisoned.into_inner()),
        }
    }

    fn with_store_mut<T>(&self, f: impl FnOnce(&mut DataStore) -> T) -> T {
        match self.store.lock() {
            Ok(mut store) => f(&mut store),
            Err(poisoned) => f(&mut poisoned.into_inner()),
        }
    }

    pub(crate) fn load_historical_data(&mut self) {
        self.recent = self.with_store(|store| store.recent(HISTORY_WINDOW));
        self.latest = self.recent.first().cloned();
        log::info!("Loaded {} recent speed tests", self.recent.len());
        self.update_chart_bounds();
        self.refresh_chart();
    }

    pub(crate) fn bounds(&self) -> DateRangeBounds {
        self.with_store(|store| store.bounds())
    }

    pub(crate) fn update_chart_bounds(&mut self) {
        let bounds = self.bounds();
        let today = format::today();
        let (min, max) = match (bounds.earliest, bounds.latest) {
            (Some(earliest), Some(latest)) => (
                format::to_jiff(earliest.with_timezone(&Local).date_naive()),
                format::to_jiff(latest.with_timezone(&Local).date_naive()),
            ),
            _ => (today, today),
        };
        let first_time = self.chart_min == self.chart_max && self.chart_start == self.chart_min;
        self.chart_min = min;
        self.chart_max = max;
        if first_time {
            self.chart_start = min;
            self.chart_end = max;
        }
        self.chart_start = self.chart_start.clamp(min, max);
        self.chart_end = self.chart_end.clamp(min, max);
    }

    pub(crate) fn refresh_chart(&mut self) {
        let now = Utc::now();
        let data: Vec<SpeedTest> = match self.chart_mode {
            ChartMode::Last20 => {
                let mut items: Vec<SpeedTest> =
                    self.recent.iter().take(CHART_POINTS).cloned().collect();
                items.reverse();
                items
            }
            ChartMode::LastDay => self.query_range(now - Duration::hours(24), now),
            ChartMode::Last7 => self.query_range(now - Duration::days(7), now),
            ChartMode::Range => {
                let start = format::from_jiff(self.chart_start);
                let end = format::from_jiff(self.chart_end);
                let (start, end) = crate::data_store::local_day_span(start, end);
                self.query_range(start, end)
            }
        };
        self.chart_data = data;
        self.chart_subtitle = self.chart_mode.subtitle().to_string();
    }

    fn query_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<SpeedTest> {
        let mut items = self.with_store(|store| store.by_range(start, end));
        items.reverse();
        items
    }

    pub(crate) fn set_chart_mode(&mut self, mode: ChartMode) {
        if self.chart_mode == mode {
            return;
        }
        self.chart_mode = mode;
        if mode == ChartMode::Range {
            self.update_chart_bounds();
            // Like the previous build: wait for "Apply" before loading a range.
            self.chart_subtitle = mode.subtitle().to_string();
        } else {
            self.refresh_chart();
        }
    }

    // ----- Schedule / settings -----

    pub(crate) fn current_schedule_from_inputs(&self) -> Result<Schedule, String> {
        let schedule = match self.schedule_type {
            ScheduleType::Interval => Schedule::interval(self.interval_minutes),
            ScheduleType::Cron => {
                let expression = self.cron_expression.trim();
                if expression.is_empty() {
                    return Err("Please enter a cron expression".to_string());
                }
                Schedule::cron(expression)
            }
        };
        schedule.validate().map_err(|error| error.to_string())?;
        Ok(schedule)
    }

    pub(crate) fn set_interval(&mut self, minutes: u32) {
        let minutes = minutes.clamp(1, 60);
        if minutes != self.interval_minutes {
            self.interval_minutes = minutes;
            self.save_schedule_settings();
        }
        self.interval_text = minutes.to_string();
    }

    pub(crate) fn step_interval(&mut self, increment: bool) {
        let next = schedule::next_interval_step(self.interval_minutes, increment);
        self.set_interval(next);
    }

    pub(crate) fn commit_interval_text(&mut self) {
        match self.interval_text.trim().parse::<u32>() {
            Ok(value) if value >= 1 => self.set_interval(value),
            _ => self.interval_text = self.interval_minutes.to_string(),
        }
    }

    pub(crate) fn save_schedule_settings(&mut self) {
        self.settings.schedule_type = match self.schedule_type {
            ScheduleType::Interval => "interval".to_string(),
            ScheduleType::Cron => "cron".to_string(),
        };
        self.settings.interval = self.interval_minutes;
        self.settings.cron_expression = if self.cron_expression.trim().is_empty() {
            crate::settings::DEFAULT_CRON_EXPRESSION.to_string()
        } else {
            self.cron_expression.trim().to_string()
        };
        self.settings.auto_start = self.auto_start;
        self.settings.launch_at_startup = self.launch_at_startup;
        self.settings.save(&self.paths.settings_file);
    }

    pub(crate) fn toggle_auto_start(&mut self) {
        self.settings.auto_start = self.auto_start;
        if !self.settings.save(&self.paths.settings_file) {
            self.auto_start = !self.auto_start;
            self.settings.auto_start = self.auto_start;
        }
        log::info!("Auto-start setting saved: {}", self.auto_start);
    }

    pub(crate) fn toggle_launch_at_startup(&mut self) {
        let enable = self.launch_at_startup;
        match launch::set_launch_at_startup(enable) {
            Ok(()) => {
                self.settings.launch_at_startup = enable;
                self.settings.save(&self.paths.settings_file);
            }
            Err(error) => {
                self.launch_at_startup = !enable;
                self.alert(
                    "Startup setting",
                    format!("Failed to change startup setting: {error}"),
                );
            }
        }
    }

    // ----- Monitoring -----

    pub(crate) fn toggle_monitoring(&mut self) {
        if self.is_monitoring {
            self.stop_monitoring();
        } else {
            self.start_monitoring();
        }
    }

    pub(crate) fn start_monitoring(&mut self) {
        log::info!("=== START MONITORING ===");
        let schedule = match self.current_schedule_from_inputs() {
            Ok(schedule) => schedule,
            Err(error) => {
                self.set_status("Stopped", StatusKind::Stopped);
                self.alert("Cannot start monitoring", error);
                return;
            }
        };
        match self.monitor.start(schedule.clone()) {
            Ok(()) => {
                self.is_monitoring = true;
                self.current_schedule = Some(schedule);
                self.set_status("Sleeping", StatusKind::Sleeping);
            }
            Err(error) => {
                self.set_status("Stopped", StatusKind::Stopped);
                self.alert(
                    "Cannot start monitoring",
                    format!("Failed to start monitoring: {error}"),
                );
            }
        }
    }

    pub(crate) fn stop_monitoring(&mut self) {
        self.monitor.stop();
        self.is_monitoring = false;
        self.current_schedule = None;
        self.set_status("Stopped", StatusKind::Stopped);
    }

    pub(crate) fn test_once_now(&mut self) {
        log::info!("=== TEST ONCE NOW CLICKED ===");
        if self.test_now_busy || self.monitor.has_current_test() {
            return;
        }
        self.test_now_busy = true;
        if !self.is_monitoring {
            self.set_status("Testing...", StatusKind::Running);
        }
        self.monitor.test_once_now();
    }

    pub(crate) fn set_status(&mut self, text: &str, kind: StatusKind) {
        self.status_text = text.to_string();
        self.status_kind = kind;
    }

    pub(crate) fn next_test_time(&self) -> Option<DateTime<Utc>> {
        if !self.is_monitoring {
            return None;
        }
        self.monitor.next_run()
    }

    // ----- Events -----

    fn handle_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::TestStarted => {
                    self.test_running = true;
                    self.set_status("Running", StatusKind::Running);
                }
                Event::TestResult(result) => {
                    self.test_running = false;
                    if self.is_monitoring {
                        self.set_status("Sleeping", StatusKind::Sleeping);
                    } else {
                        self.set_status("Stopped", StatusKind::Stopped);
                    }
                    self.recent.insert(0, result.clone());
                    self.recent.truncate(HISTORY_WINDOW);
                    self.latest = Some(result);
                    self.update_chart_bounds();
                    if self.chart_mode == ChartMode::Last20 {
                        self.refresh_chart();
                    }
                }
                Event::TestSaveFailed(error) => {
                    self.alert(
                        "Storage error",
                        format!("Failed to save test result: {error}"),
                    );
                }
                Event::ManualTestFinished => {
                    self.test_now_busy = false;
                    self.test_running = false;
                    if !self.is_monitoring {
                        self.set_status("Stopped", StatusKind::Stopped);
                    }
                }
                Event::UpdateCheckFinished { manual, result } => {
                    self.update_check_busy = false;
                    self.settings.last_check_date = Some(Utc::now().to_rfc3339());
                    self.settings.last_checked_version = Some(CURRENT_VERSION.to_string());
                    self.settings.save(&self.paths.settings_file);
                    match result {
                        Ok(Some(info)) => {
                            if manual {
                                self.alerts.push_back(Alert {
                                    title: "Update Available".to_string(),
                                    message: format!(
                                        "A new version is available!\n\nCurrent version: {}\nLatest version: {}\n\nWould you like to download the update?",
                                        info.current_version, info.latest_version
                                    ),
                                    download_url: Some(RELEASES_URL.to_string()),
                                });
                            }
                            self.update_info = Some(info);
                            self.update_dismissed = false;
                        }
                        Ok(None) => {
                            if manual {
                                self.alert(
                                    "No Updates Available",
                                    format!("You have the latest version!\n\nCurrent version: {CURRENT_VERSION}"),
                                );
                            }
                        }
                        Err(error) => {
                            log::error!("Error checking for updates: {error}");
                            if manual {
                                self.alert(
                                    "Update Check Failed",
                                    format!("Unable to check for updates\n\nError: {error}"),
                                );
                            }
                        }
                    }
                }
                Event::ScheduleStopped(error) => {
                    self.is_monitoring = false;
                    self.current_schedule = None;
                    self.set_status("Stopped", StatusKind::Stopped);
                    self.alert("Monitoring stopped", error);
                }
            }
        }
    }

    // ----- Updates -----

    fn maybe_check_for_updates(&mut self) {
        if self.settings.should_check_for_updates(Utc::now()) {
            self.check_for_updates(false);
        } else {
            log::info!("Skipping update check (checked within last 24 hours)");
        }
    }

    pub(crate) fn check_for_updates(&mut self, manual: bool) {
        if self.update_check_busy {
            return;
        }
        self.update_check_busy = true;
        let client = self.monitor.client();
        let tx = self.event_tx.clone();
        self.handle.spawn(async move {
            let result = updates::check_for_updates(&client, CURRENT_VERSION).await;
            let _ = tx.send(Event::UpdateCheckFinished { manual, result });
        });
    }

    // ----- History management -----

    pub(crate) fn clear_history(&mut self) {
        match self.with_store_mut(|store| store.clear_all()) {
            Ok(_) => {
                self.recent.clear();
                self.latest = None;
                self.chart_data.clear();
                self.load_historical_data();
                self.alert("History cleared", "History cleared successfully!");
            }
            Err(error) => self.alert("Clear history", format!("Failed to clear history: {error}")),
        }
    }

    pub(crate) fn open_clear_dialog(&mut self) {
        let bounds = self.with_store(|store| {
            let all = store.all();
            all.first()
                .zip(all.last())
                .map(|(newest, oldest)| (oldest.created(), newest.created()))
        });
        let Some((earliest, latest)) = bounds else {
            self.alert("Clear data", "No data available to clear.");
            return;
        };
        let earliest = earliest.with_timezone(&Local);
        let latest = latest.with_timezone(&Local);
        let thirty_days_ago = Local::now() - Duration::days(30);
        let default = if thirty_days_ago < earliest {
            // Round up to the next whole minute so the cutoff is not before the earliest test.
            let secs = i64::from(earliest.format("%S").to_string().parse::<u8>().unwrap_or(0));
            if secs > 0 {
                earliest + Duration::seconds(60 - secs)
            } else {
                earliest
            }
        } else if thirty_days_ago > latest {
            latest
        } else {
            thirty_days_ago
        };
        self.clear_dialog = Some(ClearDataDialog {
            date: format::to_jiff(default.date_naive()),
            hour: default.format("%H").to_string().parse().unwrap_or(0),
            minute: default.format("%M").to_string().parse().unwrap_or(0),
            earliest,
            latest,
            preview: None,
        });
    }

    pub(crate) fn clear_dialog_cutoff(dialog: &ClearDataDialog) -> Option<DateTime<Utc>> {
        let date: NaiveDate = format::from_jiff(dialog.date);
        let naive = date.and_hms_opt(dialog.hour, dialog.minute, 0)?;
        Local
            .from_local_datetime(&naive)
            .earliest()
            .map(|local| local.with_timezone(&Utc))
    }

    pub(crate) fn preview_clear(&mut self) {
        let Some(dialog) = self.clear_dialog.as_ref() else {
            return;
        };
        let Some(cutoff) = Self::clear_dialog_cutoff(dialog) else {
            return;
        };
        let earliest = dialog.earliest.with_timezone(&Utc);
        let latest = dialog.latest.with_timezone(&Utc);
        let total = self.with_store(|store| store.len());
        let to_remove = self.with_store(|store| store.count_until(cutoff));
        let cutoff_local = format::datetime_long(cutoff.with_timezone(&Local));

        let preview = if total == 0 {
            ClearPreview {
                lines: vec![("No data available.".to_string(), false)],
                can_clear: false,
            }
        } else if cutoff < earliest {
            ClearPreview {
                lines: vec![
                    ("Invalid Date Range".to_string(), true),
                    (
                        "Selected date is before your earliest test.".to_string(),
                        false,
                    ),
                    (
                        format!("Earliest test: {}", format::datetime_long(dialog.earliest)),
                        false,
                    ),
                    (format!("Selected cutoff: {cutoff_local}"), false),
                ],
                can_clear: false,
            }
        } else if cutoff > latest {
            ClearPreview {
                lines: vec![
                    ("Invalid Date Range".to_string(), true),
                    (
                        "Selected date is after your latest test.".to_string(),
                        false,
                    ),
                    (
                        format!("Latest test: {}", format::datetime_long(dialog.latest)),
                        false,
                    ),
                    (format!("Selected cutoff: {cutoff_local}"), false),
                    ("Note: This would clear ALL your data.".to_string(), false),
                ],
                can_clear: false,
            }
        } else {
            let mut lines = vec![
                ("Preview:".to_string(), false),
                (format!("• Tests to be removed: {to_remove}"), false),
                (format!("• Tests to be kept: {}", total - to_remove), false),
                (format!("• Cutoff date: {cutoff_local}"), false),
                ("This action cannot be undone.".to_string(), false),
            ];
            if to_remove == 0 {
                lines.push(("No data to remove before this date.".to_string(), false));
            } else if to_remove == total {
                lines.push(("⚠ This will remove ALL your data!".to_string(), true));
            }
            ClearPreview {
                lines,
                can_clear: to_remove > 0,
            }
        };
        if let Some(dialog) = self.clear_dialog.as_mut() {
            dialog.preview = Some(preview);
        }
    }

    pub(crate) fn confirm_clear(&mut self) {
        let Some(dialog) = self.clear_dialog.as_ref() else {
            return;
        };
        let Some(cutoff) = Self::clear_dialog_cutoff(dialog) else {
            return;
        };
        match self.with_store_mut(|store| store.clear_until(cutoff)) {
            Ok(outcome) => {
                self.clear_dialog = None;
                self.load_historical_data();
                self.alert(
                    "Data cleared",
                    format!(
                        "Successfully cleared {} speed tests before {}.\n\nRemaining tests: {}",
                        outcome.removed,
                        format::datetime_long(cutoff.with_timezone(&Local)),
                        outcome.remaining
                    ),
                );
            }
            Err(error) => self.alert("Clear data", format!("Failed to clear data: {error}")),
        }
    }

    // ----- Export -----

    pub(crate) fn open_export_dialog(&mut self) {
        let bounds = self.bounds();
        let today = format::today();
        let (min, max, has_data) = match (bounds.earliest, bounds.latest) {
            (Some(earliest), Some(latest)) => (
                format::to_jiff(earliest.with_timezone(&Local).date_naive()),
                format::to_jiff(latest.with_timezone(&Local).date_naive()),
                true,
            ),
            _ => (today, today, false),
        };
        let mut dialog = ExportDialog {
            start: min,
            end: max,
            min,
            max,
            record_count: 0,
            has_data,
        };
        dialog.record_count = self.count_export_records(&dialog);
        self.export_dialog = Some(dialog);
    }

    pub(crate) fn count_export_records(&self, dialog: &ExportDialog) -> usize {
        let (start, end) = crate::data_store::local_day_span(
            format::from_jiff(dialog.start),
            format::from_jiff(dialog.end),
        );
        self.with_store(|store| store.by_range(start, end).len())
    }

    pub(crate) fn export_range(&mut self, start: jiff::civil::Date, end: jiff::civil::Date) {
        let (start, end) = if start > end {
            (end, start)
        } else {
            (start, end)
        };
        let start_naive = format::from_jiff(start);
        let end_naive = format::from_jiff(end);
        let (csv, count) =
            self.with_store(|store| store.export_csv_for_days(start_naive, end_naive));
        let default_name = format!("speed-test-data-{start_naive}_to_{end_naive}.csv");
        self.save_csv(csv, count, &default_name);
    }

    pub(crate) fn export_all(&mut self) {
        let bounds = self.bounds();
        let (Some(earliest), Some(latest)) = (bounds.earliest, bounds.latest) else {
            self.alert("Export", "No data available to export");
            return;
        };
        self.export_range(
            format::to_jiff(earliest.with_timezone(&Local).date_naive()),
            format::to_jiff(latest.with_timezone(&Local).date_naive()),
        );
    }

    fn save_csv(&mut self, csv: String, count: usize, default_name: &str) {
        let chosen = rfd::FileDialog::new()
            .set_title("Export Speed Test Data")
            .set_file_name(default_name)
            .add_filter("CSV Files", &["csv"])
            .add_filter("All Files", &["*"])
            .save_file();
        let Some(path) = chosen else {
            log::info!("Export cancelled");
            return;
        };
        match std::fs::write(&path, csv) {
            Ok(()) => {
                let message = if count == 0 {
                    "Empty CSV file exported (no speed tests recorded yet)".to_string()
                } else {
                    format!("Exported {count} speed test records")
                };
                self.export_dialog = None;
                self.alert(
                    "Export complete",
                    format!("{message}\nFile saved to: {}", path.display()),
                );
            }
            Err(error) => self.alert("Export failed", format!("Failed to export data: {error}")),
        }
    }

    // ----- Misc actions -----

    pub(crate) fn alert(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.alerts.push_back(Alert {
            title: title.into(),
            message: message.into(),
            download_url: None,
        });
    }

    pub(crate) fn open_external(&mut self, url: &str) {
        if let Err(error) = open::that_detached(url) {
            log::error!("Error opening external link: {error}");
            self.alert("Open link", format!("Could not open {url}: {error}"));
        }
    }

    pub(crate) fn open_storage_folder(&mut self) {
        if let Err(error) = open::that_detached(&self.paths.data_dir) {
            log::error!("Error opening app storage folder: {error}");
            self.alert("Open folder", format!("Could not open folder: {error}"));
        }
    }

    pub(crate) fn show_debug_log(&mut self) {
        match logger::read(&self.paths.debug_log) {
            Ok(content) => self.debug_log = Some(content),
            Err(error) => self.alert("Debug log", format!("Failed to get debug log: {error}")),
        }
    }

    pub(crate) fn clear_debug_log(&mut self) {
        match logger::clear(&self.paths.debug_log) {
            Ok(()) => {
                if self.debug_log.is_some() {
                    self.debug_log = Some(String::new());
                }
                self.alert("Debug log", "Debug log cleared successfully");
            }
            Err(error) => self.alert("Debug log", format!("Failed to clear debug log: {error}")),
        }
    }

    pub(crate) fn median(values: impl Iterator<Item = f64>) -> Option<f64> {
        let mut sorted: Vec<f64> = values.filter(|v| v.is_finite()).collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        Some(if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        })
    }
}

impl App {
    /// One frame of the application: process background events, then draw.
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        self.handle_events();

        if self.start_pending {
            self.start_pending = false;
            self.start_monitoring();
        }

        let ctx = ui.ctx().clone();
        view::draw(self, ui);
        dialogs::draw(self, &ctx);

        // Keep clocks and countdowns fresh.
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.monitor.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn median_matches_previous_build() {
        assert_eq!(App::median([3.0, 1.0, 2.0].into_iter()), Some(2.0));
        assert_eq!(App::median([4.0, 1.0, 2.0, 3.0].into_iter()), Some(2.5));
        assert_eq!(App::median(std::iter::empty()), None);
    }
}
