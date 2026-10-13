//! Speed test history persisted to `speed_tests.json`.
//!
//! File layout is unchanged from the Electron build:
//! `{ "speedTests": [ { id, timestamp, download, upload, ping, server, isp, networkInterface, created_at }, ... ] }`
//! sorted newest first.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};

pub const CSV_HEADERS: [&str; 7] = [
    "Date",
    "Time",
    "Download Speed (Mbps)",
    "Upload Speed (Mbps)",
    "Ping (ms)",
    "Network Interface",
    "ISP",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeedTest {
    #[serde(default, deserialize_with = "lenient::i64")]
    pub id: i64,
    #[serde(default, deserialize_with = "lenient::string")]
    pub timestamp: String,
    #[serde(default, deserialize_with = "lenient::f64")]
    pub download: f64,
    #[serde(default, deserialize_with = "lenient::f64")]
    pub upload: f64,
    #[serde(default, deserialize_with = "lenient::f64")]
    pub ping: f64,
    #[serde(default, deserialize_with = "lenient::opt_string")]
    pub server: Option<String>,
    #[serde(default, deserialize_with = "lenient::opt_string")]
    pub isp: Option<String>,
    #[serde(
        rename = "networkInterface",
        default,
        deserialize_with = "lenient::opt_string"
    )]
    pub network_interface: Option<String>,
    #[serde(default, deserialize_with = "lenient::string")]
    pub created_at: String,
    /// Fields this version does not know about (written by older or newer versions).
    /// Kept so that saving never drops data.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
    /// Not persisted; set on failed tests so the UI can show the reason.
    #[serde(skip)]
    pub error: Option<String>,
}

impl SpeedTest {
    /// Instant the test ran. Falls back to `created_at`, then to the epoch.
    pub fn time(&self) -> DateTime<Utc> {
        parse_time(&self.timestamp)
            .or_else(|| parse_time(&self.created_at))
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap())
    }

    /// Instant the record was written (used for "clear until" and bounds).
    pub fn created(&self) -> DateTime<Utc> {
        parse_time(&self.created_at)
            .or_else(|| parse_time(&self.timestamp))
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap())
    }

    pub fn local_time(&self) -> DateTime<Local> {
        self.time().with_timezone(&Local)
    }
}

pub fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    if value.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            value.parse::<DateTime<Utc>>().ok().or_else(|| {
                chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|naive| Utc.from_utc_datetime(&naive))
            })
        })
}

/// Top-level key marking a file written by this (2.x) version. Version 1.x ignores and
/// keeps unknown keys, so the marker survives a downgrade round trip.
const FORMAT_MARKER: &str = "writtenBy";

/// Copy of the 1.x history taken before 2.x writes to the file for the first time.
pub const LEGACY_BACKUP_NAME: &str = "speed_tests.backup-1.x.json";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateRangeBounds {
    pub earliest: Option<DateTime<Utc>>,
    pub latest: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearOutcome {
    pub removed: usize,
    pub remaining: usize,
}

#[derive(Debug)]
pub struct DataStore {
    path: PathBuf,
    tests: Vec<SpeedTest>,
    /// Records that could not be read as speed tests. Written back unchanged.
    unreadable: Vec<serde_json::Value>,
    /// Other top-level keys in the file. Written back unchanged.
    extra: serde_json::Map<String, serde_json::Value>,
    /// Problem found while loading, for the UI to report once.
    load_warning: Option<String>,
}

impl DataStore {
    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            tests: Vec::new(),
            unreadable: Vec::new(),
            extra: serde_json::Map::new(),
            load_warning: None,
        }
    }

    /// Load the history file.
    ///
    /// This never loses data: unknown fields and unreadable records are kept and written
    /// back, a file that cannot be parsed at all is copied aside before anything else can
    /// overwrite it, and a file last written by 1.x is backed up once before the first
    /// 2.x write.
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        let mut store = Self::empty(path.clone());
        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                log::info!("No existing data file found, starting fresh");
                return Ok(store);
            }
            Err(error) => return Err(error),
        };

        let root = match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(serde_json::Value::Object(root)) => root,
            Ok(_) | Err(_) => {
                let backup = backup_path(&path, "unreadable");
                std::fs::copy(&path, &backup)?;
                let message = format!(
                    "The speed test history file could not be read, so the app started with an empty history. \
                     The original file was kept at {}.",
                    backup.display()
                );
                log::error!("{message}");
                store.load_warning = Some(message);
                return Ok(store);
            }
        };

        if !root.contains_key(FORMAT_MARKER) {
            backup_legacy_file(&path, &contents);
        }

        let mut root = root;
        let records = root.remove("speedTests");
        root.remove(FORMAT_MARKER);
        store.extra = root;

        match records {
            Some(serde_json::Value::Array(records)) => {
                for record in records {
                    match serde_json::from_value::<SpeedTest>(record.clone()) {
                        Ok(test) if record.is_object() => store.tests.push(test),
                        _ => store.unreadable.push(record),
                    }
                }
            }
            Some(serde_json::Value::Null) | None => {}
            Some(other) => store.unreadable.push(other),
        }

        if !store.unreadable.is_empty() {
            log::error!(
                "{} history records could not be read; they are kept in the file unchanged",
                store.unreadable.len()
            );
        }
        log::info!("Loaded existing data from: {}", path.display());
        store.sort();
        log::info!(
            "Data store initialized with {} speed tests",
            store.tests.len()
        );
        Ok(store)
    }

    /// A problem found while loading, reported once.
    pub fn take_load_warning(&mut self) -> Option<String> {
        self.load_warning.take()
    }

    fn sort(&mut self) {
        self.tests
            .sort_by_key(|test| std::cmp::Reverse(test.created()));
    }

    fn persist(&self) -> std::io::Result<()> {
        let mut root = self.extra.clone();
        let mut records: Vec<serde_json::Value> = Vec::with_capacity(self.tests.len());
        for test in &self.tests {
            records.push(serde_json::to_value(test)?);
        }
        records.extend(self.unreadable.iter().cloned());
        root.insert("speedTests".into(), serde_json::Value::Array(records));
        root.insert(
            FORMAT_MARKER.into(),
            serde_json::Value::String(format!("{} {}", crate::APP_NAME, crate::CURRENT_VERSION)),
        );
        let json = serde_json::to_string_pretty(&serde_json::Value::Object(root))?;
        write_atomically(&self.path, json.as_bytes())
    }

    pub fn len(&self) -> usize {
        self.tests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tests.is_empty()
    }

    /// Append a result, assign it an id and persist. Returns the stored record.
    pub fn save(&mut self, mut test: SpeedTest) -> std::io::Result<SpeedTest> {
        let now = Utc::now();
        test.id = now.timestamp_millis();
        test.created_at = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        // Insert at the front so equal timestamps keep insertion order (stable sort).
        self.tests.insert(0, test.clone());
        self.sort();
        self.persist()?;
        log::info!("Speed test saved with ID: {}", test.id);
        Ok(test)
    }

    /// Newest-first, at most `limit` records.
    pub fn recent(&self, limit: usize) -> Vec<SpeedTest> {
        self.tests.iter().take(limit).cloned().collect()
    }

    pub fn all(&self) -> &[SpeedTest] {
        &self.tests
    }

    /// Newest-first records whose test time falls inside `[start, end]`.
    pub fn by_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<SpeedTest> {
        self.tests
            .iter()
            .filter(|test| {
                let time = test.time();
                time >= start && time <= end
            })
            .cloned()
            .collect()
    }

    pub fn bounds(&self) -> DateRangeBounds {
        DateRangeBounds {
            latest: self.tests.first().map(SpeedTest::time),
            earliest: self.tests.last().map(SpeedTest::time),
        }
    }

    pub fn clear_all(&mut self) -> std::io::Result<ClearOutcome> {
        let removed = self.tests.len();
        self.tests.clear();
        self.persist()?;
        log::info!("Cleared all {removed} speed tests");
        Ok(ClearOutcome {
            removed,
            remaining: 0,
        })
    }

    /// Remove every record created at or before `cutoff`.
    pub fn clear_until(&mut self, cutoff: DateTime<Utc>) -> std::io::Result<ClearOutcome> {
        let before = self.tests.len();
        self.tests.retain(|test| test.created() > cutoff);
        self.persist()?;
        let outcome = ClearOutcome {
            removed: before - self.tests.len(),
            remaining: self.tests.len(),
        };
        log::info!(
            "Cleared {} speed tests before {}. Remaining tests: {}",
            outcome.removed,
            cutoff.to_rfc3339(),
            outcome.remaining
        );
        Ok(outcome)
    }

    /// Count of records that `clear_until(cutoff)` would remove.
    pub fn count_until(&self, cutoff: DateTime<Utc>) -> usize {
        self.tests
            .iter()
            .filter(|test| test.created() <= cutoff)
            .count()
    }

    /// CSV for every record between the start of `start` and the end of `end` (local days).
    pub fn export_csv_for_days(&self, start: NaiveDate, end: NaiveDate) -> (String, usize) {
        let (start, end) = local_day_span(start, end);
        let rows: Vec<&SpeedTest> = self
            .tests
            .iter()
            .filter(|test| {
                let time = test.time();
                time >= start && time <= end
            })
            .collect();
        (
            render_csv(
                rows.iter().copied(),
                "No data available for selected date range",
            ),
            rows.len(),
        )
    }

    #[cfg(test)]
    pub fn export_csv_all(&self) -> (String, usize) {
        (
            render_csv(self.tests.iter(), "No data available"),
            self.tests.len(),
        )
    }
}

/// UTC instants for local midnight at `start` through the last millisecond of `end`.
pub fn local_day_span(start: NaiveDate, end: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = Local
        .from_local_datetime(&start.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .map(|date| date.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let end = Local
        .from_local_datetime(&end.and_hms_milli_opt(23, 59, 59, 999).unwrap())
        .latest()
        .map(|date| date.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    (start, end)
}

fn render_csv<'a>(tests: impl Iterator<Item = &'a SpeedTest>, empty_message: &str) -> String {
    let mut lines = vec![CSV_HEADERS.join(",")];
    let mut count = 0;
    for test in tests {
        count += 1;
        let local = test.local_time();
        lines.push(
            [
                csv_quote(&local.format("%-m/%-d/%Y").to_string()),
                csv_quote(&local.format("%-I:%M:%S %p").to_string()),
                format!("{:.2}", test.download),
                format!("{:.2}", test.upload),
                format!("{:.0}", test.ping),
                csv_quote(test.network_interface.as_deref().unwrap_or("Unknown")),
                csv_quote(test.isp.as_deref().unwrap_or("Unknown")),
            ]
            .join(","),
        );
    }
    if count == 0 {
        lines.push(format!(
            "{},\"\",\"\",\"\",\"\",\"\",\"\"",
            csv_quote(empty_message)
        ));
    }
    lines.join("\n")
}

fn csv_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

/// `speed_tests.<label>-<timestamp>.json` next to `path`.
fn backup_path(path: &Path, label: &str) -> PathBuf {
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    path.with_file_name(format!("speed_tests.{label}-{stamp}.json"))
}

/// Keep one copy of the history as 1.x left it, before 2.x changes the file.
fn backup_legacy_file(path: &Path, contents: &str) {
    let backup = path.with_file_name(LEGACY_BACKUP_NAME);
    if backup.exists() {
        return;
    }
    match std::fs::write(&backup, contents) {
        Ok(()) => log::info!("Backed up 1.x history to {}", backup.display()),
        Err(error) => log::error!("Could not back up 1.x history: {error}"),
    }
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Deserializers that accept the loose JSON an older version, a hand edit or a failed
/// test may have written (null, numbers as strings, strings as numbers) instead of failing
/// the whole file.
pub(crate) mod lenient {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    fn as_f64(value: &Value) -> Option<f64> {
        match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.trim().parse().ok(),
            Value::Bool(flag) => Some(if *flag { 1.0 } else { 0.0 }),
            _ => None,
        }
        .filter(|number| number.is_finite())
    }

    pub fn f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
        Ok(as_f64(&Value::deserialize(deserializer)?).unwrap_or(0.0))
    }

    pub fn i64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Ok(match &value {
            Value::Number(number) => number
                .as_i64()
                .or_else(|| number.as_f64().map(|f| f as i64))
                .unwrap_or(0),
            other => as_f64(other).map(|f| f as i64).unwrap_or(0),
        })
    }

    pub fn u32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u32>, D::Error> {
        Ok(as_f64(&Value::deserialize(deserializer)?)
            .filter(|n| *n >= 0.0 && *n <= f64::from(u32::MAX))
            .map(|n| n as u32))
    }

    pub fn bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        Ok(match Value::deserialize(deserializer)? {
            Value::Bool(flag) => flag,
            Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0),
            Value::String(text) => matches!(text.trim(), "true" | "1" | "yes"),
            _ => false,
        })
    }

    fn as_string(value: Value) -> Option<String> {
        match value {
            Value::String(text) => Some(text),
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(flag) => Some(flag.to_string()),
            _ => None,
        }
    }

    pub fn string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
        Ok(as_string(Value::deserialize(deserializer)?).unwrap_or_default())
    }

    pub fn opt_string<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<String>, D::Error> {
        Ok(as_string(Value::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(timestamp: &str, download: f64) -> SpeedTest {
        SpeedTest {
            id: 0,
            timestamp: timestamp.to_string(),
            download,
            upload: download / 10.0,
            ping: 20.0,
            server: Some("Netflix Fast.com".into()),
            isp: Some("ISP".into()),
            network_interface: Some("WiFi".into()),
            created_at: timestamp.to_string(),
            extra: Default::default(),
            error: None,
        }
    }

    fn temp_store() -> DataStore {
        let dir = std::env::temp_dir().join(format!(
            "ism-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        DataStore::empty(dir.join("speed_tests.json"))
    }

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ism-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A history file as 1.x writes it, including the awkward cases seen in the wild:
    /// a failed test with nulls, a record from before ISP/interface existed, a value that
    /// JSON.stringify turned into null, numbers stored as strings and a field 2.x does not
    /// know about.
    const LEGACY_FILE: &str = r#"{
  "speedTests": [
    {
      "id": 1760383548000,
      "timestamp": "2026-10-13T19:25:48.000Z",
      "download": 100.5,
      "upload": 10.05,
      "ping": 12.4,
      "server": "Netflix Fast.com",
      "isp": "Acme",
      "networkInterface": "WiFi",
      "created_at": "2026-10-13T19:25:48.120Z"
    },
    {
      "id": 1760379948000,
      "timestamp": "2026-10-13T18:25:48.000Z",
      "download": 0,
      "upload": 0,
      "ping": 0,
      "server": "Error",
      "isp": null,
      "networkInterface": null,
      "created_at": "2026-10-13T18:25:48.120Z"
    },
    {
      "id": 1700000000000,
      "timestamp": "2023-11-14T22:13:20.000Z",
      "download": null,
      "upload": "9.5",
      "ping": 20,
      "server": "Netflix Fast.com",
      "created_at": "2023-11-14T22:13:20.000Z",
      "note": "kept by 2.x"
    },
    "not a record"
  ],
  "somethingElse": {"kept": true}
}"#;

    #[test]
    fn reads_every_legacy_record() {
        let dir = temp_dir();
        let path = dir.join("speed_tests.json");
        std::fs::write(&path, LEGACY_FILE).unwrap();
        let mut store = DataStore::open(path.clone()).unwrap();
        assert_eq!(store.len(), 3);
        assert!(store.take_load_warning().is_none());
        let tests = store.all();
        assert_eq!(tests[0].download, 100.5);
        assert_eq!(tests[0].network_interface.as_deref(), Some("WiFi"));
        assert_eq!(tests[1].server.as_deref(), Some("Error"));
        assert_eq!(tests[1].isp, None);
        assert_eq!(tests[2].download, 0.0);
        assert_eq!(tests[2].upload, 9.5);
        assert_eq!(tests[2].isp, None);
        assert_eq!(
            tests[2].extra.get("note").and_then(|v| v.as_str()),
            Some("kept by 2.x")
        );
        // The untouched 1.x file is backed up once.
        let backup = dir.join(LEGACY_BACKUP_NAME);
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), LEGACY_FILE);
    }

    #[test]
    fn saving_keeps_unknown_fields_and_unreadable_records() {
        let dir = temp_dir();
        let path = dir.join("speed_tests.json");
        std::fs::write(&path, LEGACY_FILE).unwrap();
        let mut store = DataStore::open(path.clone()).unwrap();
        store.save(sample("2026-10-13T20:00:00Z", 50.0)).unwrap();

        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let records = written["speedTests"].as_array().unwrap();
        assert_eq!(records.len(), 5);
        assert!(records.iter().any(|r| r == "not a record"));
        assert!(records.iter().any(|r| r["note"] == "kept by 2.x"));
        assert_eq!(written["somethingElse"]["kept"], true);
        assert!(written.get(FORMAT_MARKER).is_some());
        // Field names stay exactly as 1.x wrote them.
        let first = &records[0];
        for key in [
            "id",
            "timestamp",
            "download",
            "upload",
            "ping",
            "server",
            "isp",
            "networkInterface",
            "created_at",
        ] {
            assert!(first.get(key).is_some(), "missing {key}");
        }
        assert!(first.get("error").is_none());

        // Re-opening a 2.x file does not overwrite the 1.x backup.
        std::fs::write(dir.join(LEGACY_BACKUP_NAME), "original").unwrap();
        let reopened = DataStore::open(path).unwrap();
        assert_eq!(reopened.len(), 4);
        assert_eq!(
            std::fs::read_to_string(dir.join(LEGACY_BACKUP_NAME)).unwrap(),
            "original"
        );
    }

    #[test]
    fn unreadable_file_is_copied_aside_not_overwritten() {
        let dir = temp_dir();
        let path = dir.join("speed_tests.json");
        std::fs::write(&path, "{ this is not json").unwrap();
        let mut store = DataStore::open(path.clone()).unwrap();
        assert_eq!(store.len(), 0);
        assert!(store.take_load_warning().is_some());
        let copies: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("speed_tests.unreadable-")
            })
            .collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(
            std::fs::read_to_string(copies[0].path()).unwrap(),
            "{ this is not json"
        );
    }

    #[test]
    fn save_sorts_newest_first_and_persists() {
        let mut store = temp_store();
        store.save(sample("2026-10-10T10:00:00Z", 1.0)).unwrap();
        store.save(sample("2026-10-12T10:00:00Z", 3.0)).unwrap();
        store.save(sample("2026-10-11T10:00:00Z", 2.0)).unwrap();
        // created_at is assigned at save time, so insertion order wins.
        assert_eq!(store.recent(10)[0].download, 2.0);
        let reopened = DataStore::open(store.path.clone()).unwrap();
        assert_eq!(reopened.len(), 3);
    }

    #[test]
    fn range_queries_and_bounds() {
        let mut store = temp_store();
        store.tests = vec![
            sample("2026-10-12T10:00:00Z", 3.0),
            sample("2026-10-11T10:00:00Z", 2.0),
            sample("2026-10-10T10:00:00Z", 1.0),
        ];
        let start = parse_time("2026-10-11T00:00:00Z").unwrap();
        let end = parse_time("2026-10-11T23:59:59Z").unwrap();
        let rows = store.by_range(start, end);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].download, 2.0);
        let bounds = store.bounds();
        assert_eq!(bounds.latest, parse_time("2026-10-12T10:00:00Z"));
        assert_eq!(bounds.earliest, parse_time("2026-10-10T10:00:00Z"));
    }

    #[test]
    fn clear_until_keeps_newer_records() {
        let mut store = temp_store();
        store.tests = vec![
            sample("2026-10-12T10:00:00Z", 3.0),
            sample("2026-10-11T10:00:00Z", 2.0),
            sample("2026-10-10T10:00:00Z", 1.0),
        ];
        let cutoff = parse_time("2026-10-11T10:00:00Z").unwrap();
        assert_eq!(store.count_until(cutoff), 2);
        let outcome = store.clear_until(cutoff).unwrap();
        assert_eq!(
            outcome,
            ClearOutcome {
                removed: 2,
                remaining: 1
            }
        );
        assert_eq!(store.all()[0].download, 3.0);
    }

    #[test]
    fn csv_has_header_and_quotes() {
        let mut store = temp_store();
        store.tests = vec![sample("2026-10-12T10:00:00Z", 3.456)];
        let (csv, count) = store.export_csv_all();
        assert_eq!(count, 1);
        let mut lines = csv.lines();
        assert_eq!(lines.next().unwrap(), CSV_HEADERS.join(","));
        let row = lines.next().unwrap();
        assert!(row.contains(",3.46,0.35,20,\"WiFi\",\"ISP\""));
        let (empty, count) = DataStore::empty(store.path.clone()).export_csv_all();
        assert_eq!(count, 0);
        assert!(empty.contains("No data available"));
    }
}
