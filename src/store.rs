//! Local state: injected simulator faults, recorded logs, and user manifests.
//!
//! Everything lives under one root, `~/.spellbook` unless `SPELLBOOK_HOME`
//! or `--home` says otherwise.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::telemetry::{LogRecord, write_log};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// Faults injected into simulated devices, by device id.
    #[serde(default)]
    pub faults: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Store { root: root.into() }
    }

    pub fn default_root() -> PathBuf {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".spellbook")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifests_dir(&self) -> PathBuf {
        self.root.join("manifests")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }

    /// A missing or unreadable state file means a fresh start, not a crash.
    pub fn load_state(&self) -> State {
        fs::read_to_string(self.state_path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save_state(&self, state: &State) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("could not create {}", self.root.display()))?;
        fs::write(self.state_path(), serde_json::to_string_pretty(state)?)
            .with_context(|| format!("could not write {}", self.state_path().display()))
    }

    /// Write records to `logs/<device>-<unix millis>.jsonl`.
    pub fn record_log(&self, device: &str, records: &[LogRecord]) -> Result<PathBuf> {
        let dir = self.logs_dir();
        fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;
        let mut stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or_default();
        let path = loop {
            let candidate = dir.join(format!("{device}-{stamp}.jsonl"));
            if !candidate.exists() {
                break candidate;
            }
            stamp += 1;
        };
        write_log(&path, records)?;
        Ok(path)
    }

    pub fn latest_log(&self, device: &str) -> Result<Option<PathBuf>> {
        let entries = match fs::read_dir(self.logs_dir()) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let prefix = format!("{device}-");
        let mut latest: Option<(u128, PathBuf)> = None;
        for entry in entries {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            // Parse the stamp so device "smart" never picks up "smart-plug-…" logs.
            let stamp = name
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix(".jsonl"))
                .and_then(|stamp| stamp.parse::<u128>().ok());
            if let Some(stamp) = stamp
                && latest.as_ref().is_none_or(|(best, _)| stamp > *best)
            {
                latest = Some((stamp, path));
            }
        }
        Ok(latest.map(|(_, path)| path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_state_starts_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        fs::write(dir.path().join("state.json"), "{ not json").unwrap();
        assert_eq!(store.load_state(), State::default());
    }

    #[test]
    fn latest_log_ignores_devices_sharing_a_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let first = store.record_log("smart", &[]).unwrap();
        let second = store.record_log("smart", &[]).unwrap();
        store.record_log("smart-plug", &[]).unwrap();
        assert_ne!(first, second);
        assert_eq!(store.latest_log("smart").unwrap(), Some(second));
        assert_eq!(store.latest_log("missing").unwrap(), None);
    }
}
