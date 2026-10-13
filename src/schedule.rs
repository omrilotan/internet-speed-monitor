//! Monitoring schedules: a simple minute interval or a 5-field UNIX cron expression.

use std::str::FromStr;

use chrono::{DateTime, Duration, Local, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schedule {
    Interval { minutes: u32 },
    Cron { expression: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleError(pub String);

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScheduleError {}

impl Schedule {
    pub fn interval(minutes: u32) -> Self {
        Self::Interval { minutes }
    }

    pub fn cron(expression: impl Into<String>) -> Self {
        Self::Cron {
            expression: expression.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ScheduleError> {
        match self {
            Schedule::Interval { minutes } => {
                if *minutes < 1 {
                    Err(ScheduleError("Please enter a valid interval".into()))
                } else {
                    Ok(())
                }
            }
            Schedule::Cron { expression } => parse_cron(expression).map(|_| ()),
        }
    }

    /// Instant of the next run after `now`.
    pub fn next_run_after(&self, now: DateTime<Utc>) -> Result<DateTime<Utc>, ScheduleError> {
        match self {
            Schedule::Interval { minutes } => {
                Ok(now + Duration::minutes(i64::from((*minutes).max(1))))
            }
            Schedule::Cron { expression } => {
                let schedule = parse_cron(expression)?;
                let local_now = now.with_timezone(&Local);
                schedule
                    .after(&local_now)
                    .next()
                    .map(|date| date.with_timezone(&Utc))
                    .ok_or_else(|| ScheduleError("Cron expression never fires".into()))
            }
        }
    }
}

impl std::fmt::Display for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schedule::Interval { minutes } => write!(f, "interval ({minutes} min)"),
            Schedule::Cron { expression } => write!(f, "cron ({expression})"),
        }
    }
}

/// Parse a standard 5-field cron expression (`minute hour day month weekday`).
pub fn parse_cron(expression: &str) -> Result<cron::Schedule, ScheduleError> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return Err(ScheduleError(
            "Invalid cron schedule: expression is missing or empty".into(),
        ));
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(ScheduleError(
            "Invalid cron expression: must have exactly 5 parts (minute hour day month weekday)"
                .into(),
        ));
    }
    // The `cron` crate wants a seconds field first.
    let with_seconds = format!("0 {}", parts.join(" "));
    cron::Schedule::from_str(&with_seconds)
        .map_err(|error| ScheduleError(format!("Invalid cron expression: {error}")))
}

pub const CRON_PRESETS: [(&str, &str, &str); 5] = [
    ("5m", "*/5 * * * *", "Every 5 minutes"),
    ("15m", "*/15 * * * *", "Every 15 minutes"),
    ("30m", "*/30 * * * *", "Every 30 minutes"),
    ("1h", "0 * * * *", "Every hour"),
    ("Work", "0 9-17 * * 1-5", "Every hour, 9am-5pm, Mon-Fri"),
];

const KNOWN_PATTERNS: [(&str, &str); 17] = [
    ("* * * * *", "Every minute"),
    ("*/1 * * * *", "Every minute"),
    ("*/5 * * * *", "Every 5 minutes"),
    ("*/10 * * * *", "Every 10 minutes"),
    ("*/15 * * * *", "Every 15 minutes"),
    ("*/30 * * * *", "Every 30 minutes"),
    ("0 * * * *", "Every hour"),
    ("0 */2 * * *", "Every 2 hours"),
    ("0 */4 * * *", "Every 4 hours"),
    ("0 */6 * * *", "Every 6 hours"),
    ("0 */12 * * *", "Every 12 hours"),
    ("0 0 * * *", "Daily at midnight"),
    ("0 9 * * *", "Daily at 9 AM"),
    ("0 17 * * *", "Daily at 5 PM"),
    ("0 9-17 * * 1-5", "Every hour, 9am-5pm, Mon-Fri"),
    ("*/30 9-17 * * 1-5", "Every 30 minutes, 9am-5pm, Mon-Fri"),
    ("0 9,12,15,17 * * 1-5", "At 9am, 12pm, 3pm, 5pm, Mon-Fri"),
];

/// Human description of a cron expression, as shown under the input field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronDescription {
    Empty,
    Known(&'static str),
    Custom,
    Invalid,
}

pub fn classify_cron(expression: &str) -> CronDescription {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return CronDescription::Empty;
    }
    let normalised = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some((_, description)) = KNOWN_PATTERNS
        .iter()
        .find(|(pattern, _)| *pattern == normalised)
    {
        return CronDescription::Known(description);
    }
    match parse_cron(trimmed) {
        Ok(_) => CronDescription::Custom,
        Err(_) => CronDescription::Invalid,
    }
}

#[cfg(test)]
pub fn describe_cron(expression: &str) -> String {
    match classify_cron(expression) {
        CronDescription::Empty => "Enter a cron expression".to_string(),
        CronDescription::Known(description) => description.to_string(),
        CronDescription::Custom => "Custom schedule".to_string(),
        CronDescription::Invalid => "Invalid cron expression".to_string(),
    }
}

/// Smart stepping for the interval input: 1, 5, 10, 15, 30, 60.
pub const INTERVAL_STEPS: [u32; 6] = [1, 5, 10, 15, 30, 60];

pub fn next_interval_step(current: u32, increment: bool) -> u32 {
    if let Some(index) = INTERVAL_STEPS.iter().position(|step| *step == current) {
        return if increment {
            INTERVAL_STEPS[(index + 1).min(INTERVAL_STEPS.len() - 1)]
        } else {
            INTERVAL_STEPS[index.saturating_sub(1)]
        };
    }
    if increment {
        INTERVAL_STEPS
            .iter()
            .copied()
            .find(|step| *step > current)
            .unwrap_or(INTERVAL_STEPS[INTERVAL_STEPS.len() - 1])
    } else {
        INTERVAL_STEPS
            .iter()
            .rev()
            .copied()
            .find(|step| *step < current)
            .unwrap_or(INTERVAL_STEPS[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn interval_next_run() {
        let now = Utc.with_ymd_and_hms(2026, 10, 13, 12, 0, 0).unwrap();
        let next = Schedule::interval(15).next_run_after(now).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 13, 12, 15, 0).unwrap());
    }

    #[test]
    fn cron_next_run_is_on_boundary() {
        let now = Utc.with_ymd_and_hms(2026, 10, 13, 12, 2, 30).unwrap();
        let next = Schedule::cron("*/5 * * * *").next_run_after(now).unwrap();
        assert_eq!(next.timestamp() % 300, 0);
        assert!(next > now);
        assert!(next - now <= Duration::minutes(5));
    }

    #[test]
    fn cron_validation() {
        assert!(Schedule::cron("").validate().is_err());
        assert!(Schedule::cron("* * * *").validate().is_err());
        assert!(Schedule::cron("0 0 * * * *").validate().is_err());
        assert!(Schedule::cron("99 * * * *").validate().is_err());
        assert!(Schedule::cron("0 9-17 * * 1-5").validate().is_ok());
        assert!(Schedule::interval(0).validate().is_err());
        assert!(Schedule::interval(1).validate().is_ok());
    }

    #[test]
    fn cron_descriptions() {
        assert_eq!(classify_cron(""), CronDescription::Empty);
        assert_eq!(
            classify_cron("*/5 * * * *"),
            CronDescription::Known("Every 5 minutes")
        );
        assert_eq!(
            classify_cron(" 0   * * * * "),
            CronDescription::Known("Every hour")
        );
        assert_eq!(classify_cron("7 3 * * *"), CronDescription::Custom);
        assert_eq!(classify_cron("bogus"), CronDescription::Invalid);
        assert_eq!(describe_cron("0 9 * * *"), "Daily at 9 AM");
    }

    #[test]
    fn interval_stepping() {
        assert_eq!(next_interval_step(5, true), 10);
        assert_eq!(next_interval_step(5, false), 1);
        assert_eq!(next_interval_step(60, true), 60);
        assert_eq!(next_interval_step(1, false), 1);
        assert_eq!(next_interval_step(7, true), 10);
        assert_eq!(next_interval_step(7, false), 5);
        assert_eq!(next_interval_step(100, false), 60);
        assert_eq!(next_interval_step(100, true), 60);
    }
}
