//! Persisted application state, stored as JSON in the app config directory.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::baseline::stevenblack::Extension;
use crate::domain::hostname::Hostname;
use crate::domain::mode::BlockingMode;
use crate::error::{AppError, AppResult};

/// Everything the UI shows and every apply needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub mode: BlockingMode,
    pub sites: Vec<Hostname>,
    pub baseline: Option<BaselineMeta>,
}

/// Metadata for the imported StevenBlack baseline; the host list itself is
/// cached separately as a plain text file next to the state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineMeta {
    pub extensions: Vec<Extension>,
    pub fetched_at: DateTime<Utc>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: BlockingMode::Active,
            sites: Vec::new(),
            baseline: None,
        }
    }
}

const STATE_FILE: &str = "state.json";
/// File name of the cached, parsed baseline host list.
pub const BASELINE_CACHE_FILE: &str = "baseline-hosts.txt";

/// Loads and saves [`AppState`] from `<state_dir>/state.json` and keeps the
/// parsed baseline host list in `<cache_dir>/baseline-hosts.txt`.
#[derive(Debug, Clone)]
pub struct Store {
    state_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Store {
    /// Single-directory convenience constructor (used by tests).
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self {
            state_dir: dir.clone(),
            cache_dir: dir,
        }
    }

    pub fn with_dirs(state_dir: impl Into<PathBuf>, cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub fn state_path(&self) -> PathBuf {
        self.state_dir.join(STATE_FILE)
    }

    pub fn baseline_cache_path(&self) -> PathBuf {
        self.cache_dir.join(BASELINE_CACHE_FILE)
    }

    /// Missing file yields the defaults; a corrupt file is an error rather
    /// than a silent reset.
    pub fn load(&self) -> AppResult<AppState> {
        match fs::read_to_string(self.state_path()) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| {
                AppError::Data(format!(
                    "could not read {}: {e}; fix or remove the file to reset",
                    self.state_path().display()
                ))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppState::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Atomic save: write a sibling temp file, then rename over the target.
    pub fn save(&self, state: &AppState) -> AppResult<()> {
        save_json(&self.state_path(), state)
    }

    pub fn load_baseline_hosts(&self) -> AppResult<Vec<String>> {
        match fs::read_to_string(self.baseline_cache_path()) {
            Ok(text) => Ok(text
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_baseline_hosts(&self, hosts: &[String]) -> AppResult<()> {
        let path = self.baseline_cache_path();
        let tmp = write_tmp(&path, hosts.join("\n").as_bytes())?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn delete_baseline_hosts(&self) -> AppResult<()> {
        match fs::remove_file(self.baseline_cache_path()) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Write `value` as pretty JSON to `path` atomically.
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    let tmp = write_tmp(path, text.as_bytes())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn write_tmp(path: &Path, bytes: &[u8]) -> AppResult<PathBuf> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    Ok(tmp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let state = AppState {
            mode: BlockingMode::CustomPaused,
            sites: vec![Hostname::parse("https://x.com/a").unwrap()],
            baseline: Some(BaselineMeta {
                extensions: vec![Extension::Porn, Extension::Gambling],
                fetched_at: DateTime::from_str("2026-01-02T03:04:05Z").unwrap(),
            }),
        };
        store.save(&state).unwrap();
        assert_eq!(store.load().unwrap(), state);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("nested"));
        let state = store.load().unwrap();
        assert_eq!(state, AppState::default());
        assert_eq!(state.mode, BlockingMode::Active);
    }

    #[test]
    fn corrupt_file_is_an_error_not_a_reset() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        fs::write(store.state_path(), "{ not json").unwrap();
        let err = store.load().unwrap_err();
        assert!(matches!(err, AppError::Data(_)));
    }

    #[test]
    fn baseline_hosts_cache_round_trip_and_delete() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        assert_eq!(store.load_baseline_hosts().unwrap(), Vec::<String>::new());
        let hosts = vec!["a.example.com".to_string(), "b.example.com".to_string()];
        store.save_baseline_hosts(&hosts).unwrap();
        assert_eq!(store.load_baseline_hosts().unwrap(), hosts);
        store.delete_baseline_hosts().unwrap();
        assert_eq!(store.load_baseline_hosts().unwrap(), Vec::<String>::new());
        store.delete_baseline_hosts().unwrap(); // deleting twice is fine
    }
}
