//! Background scheduler that runs speed tests on an interval or cron schedule and
//! reports back to the UI through a channel.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

use crate::data_store::{DataStore, SpeedTest};
use crate::schedule::{Schedule, ScheduleError};
use crate::speedtest::{self, FastConfig};
use crate::updates::UpdateInfo;

/// Messages from background work to the UI thread.
#[derive(Debug)]
pub enum Event {
    TestStarted,
    TestResult(SpeedTest),
    /// A test finished but was not saved (store write failed).
    TestSaveFailed(String),
    /// Manual "test once" finished.
    ManualTestFinished,
    UpdateCheckFinished {
        manual: bool,
        result: Result<Option<UpdateInfo>, String>,
    },
    ScheduleStopped(String),
}

pub type EventSender = std::sync::mpsc::Sender<Event>;

struct Shared {
    store: Arc<Mutex<DataStore>>,
    events: EventSender,
    repaint: egui::Context,
    client: reqwest::Client,
    config: FastConfig,
    test_in_progress: AtomicBool,
    next_run: Mutex<Option<DateTime<Utc>>>,
}

impl Shared {
    fn notify(&self, event: Event) {
        let _ = self.events.send(event);
        self.repaint.request_repaint();
    }

    /// Run one test unless one is already running. Returns false if skipped.
    async fn run_test(&self) -> bool {
        if self
            .test_in_progress
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            log::info!("Speed test already in progress, skipping...");
            return false;
        }
        self.notify(Event::TestStarted);
        let result = speedtest::run_speed_test(&self.client, &self.config).await;
        let saved = match self.store.lock() {
            Ok(mut store) => store.save(result.clone()),
            Err(poisoned) => poisoned.into_inner().save(result.clone()),
        };
        match saved {
            Ok(mut record) => {
                record.error = result.error;
                self.notify(Event::TestResult(record));
            }
            Err(error) => {
                log::error!("Failed to save speed test result: {error}");
                self.notify(Event::TestSaveFailed(error.to_string()));
                self.notify(Event::TestResult(result));
            }
        }
        self.test_in_progress.store(false, Ordering::Release);
        true
    }
}

pub struct SpeedMonitor {
    handle: Handle,
    shared: Arc<Shared>,
    schedule: Option<Schedule>,
    cancel: Option<CancellationToken>,
}

impl SpeedMonitor {
    pub fn new(
        handle: Handle,
        store: Arc<Mutex<DataStore>>,
        events: EventSender,
        repaint: egui::Context,
    ) -> Self {
        Self {
            handle,
            shared: Arc::new(Shared {
                store,
                events,
                repaint,
                client: speedtest::build_client(),
                config: FastConfig::default(),
                test_in_progress: AtomicBool::new(false),
                next_run: Mutex::new(None),
            }),
            schedule: None,
            cancel: None,
        }
    }

    pub fn client(&self) -> reqwest::Client {
        self.shared.client.clone()
    }

    pub fn is_running(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|token| !token.is_cancelled())
    }

    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        *self
            .shared
            .next_run
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }

    pub fn start(&mut self, schedule: Schedule) -> Result<(), ScheduleError> {
        if self.is_running() {
            log::info!("Speed monitor is already running");
            return Ok(());
        }
        schedule.validate()?;
        log::info!("Starting speed monitor with schedule: {schedule}");

        // Publish the first due time right away so the UI can show it this frame.
        let first_run = schedule.next_run_after(Utc::now())?;
        *self
            .shared
            .next_run
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(first_run);

        let token = CancellationToken::new();
        let shared = Arc::clone(&self.shared);
        let task_schedule = schedule.clone();
        let task_token = token.clone();

        self.handle.spawn(async move {
            // Interval mode runs a test immediately; cron mode waits for the boundary.
            if matches!(task_schedule, Schedule::Interval { .. }) {
                tokio::select! {
                    _ = task_token.cancelled() => return,
                    _ = shared.run_test() => {}
                }
            }
            loop {
                let next = match task_schedule.next_run_after(Utc::now()) {
                    Ok(next) => next,
                    Err(error) => {
                        log::error!("Error calculating next run time: {error}");
                        shared.notify(Event::ScheduleStopped(error.to_string()));
                        return;
                    }
                };
                *shared.next_run.lock().unwrap_or_else(|p| p.into_inner()) = Some(next);
                let wait = (next - Utc::now())
                    .to_std()
                    .unwrap_or(std::time::Duration::ZERO);
                log::info!("Next scheduled test: {}", next.to_rfc3339());
                tokio::select! {
                    _ = task_token.cancelled() => return,
                    _ = tokio::time::sleep(wait) => {}
                }
                log::info!("Running scheduled test at {}", Utc::now().to_rfc3339());
                tokio::select! {
                    _ = task_token.cancelled() => return,
                    _ = shared.run_test() => {}
                }
            }
        });

        self.schedule = Some(schedule);
        self.cancel = Some(token);
        Ok(())
    }

    pub fn stop(&mut self) {
        match self.cancel.take() {
            Some(token) => {
                token.cancel();
                *self
                    .shared
                    .next_run
                    .lock()
                    .unwrap_or_else(|p| p.into_inner()) = None;
                log::info!("Speed monitor stopped");
            }
            None => log::info!("Speed monitor is not running"),
        }
    }

    /// Run a single test now, outside of the schedule.
    pub fn test_once_now(&self) {
        log::info!("Manual test requested");
        let shared = Arc::clone(&self.shared);
        self.handle.spawn(async move {
            shared.run_test().await;
            shared.notify(Event::ManualTestFinished);
        });
    }

    pub fn has_current_test(&self) -> bool {
        self.shared.test_in_progress.load(Ordering::Acquire)
    }
}

impl Drop for SpeedMonitor {
    fn drop(&mut self) {
        if let Some(token) = self.cancel.take() {
            token.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Needs network access: starts an interval schedule and waits for the first result.
    #[test]
    #[ignore = "requires internet access"]
    fn interval_schedule_runs_first_test_immediately() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let dir = std::env::temp_dir().join(format!("ism-monitor-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(Mutex::new(DataStore::empty(dir.join("speed_tests.json"))));
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = egui::Context::default();
        let mut monitor = SpeedMonitor::new(runtime.handle().clone(), Arc::clone(&store), tx, ctx);

        monitor.start(Schedule::interval(1)).unwrap();
        assert!(monitor.is_running());
        assert!(monitor.next_run().is_some());

        let deadline = Instant::now() + Duration::from_secs(60);
        let mut started = false;
        let mut result = None;
        while Instant::now() < deadline && result.is_none() {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Event::TestStarted) => started = true,
                Ok(Event::TestResult(test)) => result = Some(test),
                Ok(_) => {}
                Err(_) => {}
            }
        }
        monitor.stop();
        assert!(!monitor.is_running());

        assert!(started, "TestStarted event never arrived");
        let result = result.expect("TestResult event never arrived");
        assert!(result.id > 0, "result was not saved");
        assert_eq!(store.lock().unwrap().len(), 1);
        assert!(result.download >= 0.0);
    }
}
