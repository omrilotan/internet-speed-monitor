//! Small formatting and date-conversion helpers shared by the views.

use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone};

pub fn today() -> jiff::civil::Date {
    to_jiff(Local::now().date_naive())
}

pub fn to_jiff(date: NaiveDate) -> jiff::civil::Date {
    jiff::civil::Date::new(date.year() as i16, date.month() as i8, date.day() as i8)
        .unwrap_or_default()
}

pub fn from_jiff(date: jiff::civil::Date) -> NaiveDate {
    NaiveDate::from_ymd_opt(
        i32::from(date.year()),
        u32::from(date.month() as u8),
        u32::from(date.day() as u8),
    )
    .unwrap_or_else(|| Local::now().date_naive())
}

/// `14:05:09`
pub fn time_hms(date: DateTime<Local>) -> String {
    date.format("%H:%M:%S").to_string()
}

/// `14:05`
pub fn time_hm(date: DateTime<Local>) -> String {
    date.format("%H:%M").to_string()
}

/// `10/13/2026, 2:05:09 PM`
pub fn datetime_long(date: DateTime<Local>) -> String {
    date.format("%-m/%-d/%Y, %-I:%M:%S %p").to_string()
}

/// `Oct 13 14:05`
pub fn chart_label(date: DateTime<Local>) -> String {
    date.format("%b %-d %H:%M").to_string()
}

pub fn chart_axis_label(seconds: f64, span_seconds: f64) -> String {
    let Some(date) = Local.timestamp_opt(seconds.round() as i64, 0).single() else {
        return String::new();
    };
    if span_seconds > 36.0 * 3600.0 {
        date.format("%b %-d\n%H:%M").to_string()
    } else {
        date.format("%H:%M").to_string()
    }
}

pub fn mbps(value: f64) -> String {
    format!("{value:.2} Mbps")
}

pub fn ms(value: f64) -> String {
    format!("{value:.2} ms")
}
