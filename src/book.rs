//! The operations behind every spell, shared by the CLI and the MCP portal.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::diag::{Diagnosis, diagnose};
use crate::manifest::{ErrorCode, Fault, Manifest};
use crate::registry::Registry;
use crate::safety::{Actor, Hazard, authorize};
use crate::sim;
use crate::store::Store;
use crate::telemetry::{LogRecord, read_log};
use crate::transport::{self, Availability, Transport};

pub struct Spellbook {
    store: Store,
    registry: Registry,
    warnings: Vec<String>,
    seed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub transport: String,
    pub availability: Availability,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallReport {
    pub device: String,
    pub command: String,
    pub hazard: Hazard,
    pub simulated: bool,
    pub message: String,
    pub log: Option<PathBuf>,
    #[serde(skip)]
    pub records: Vec<LogRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosisReport {
    /// Where the telemetry came from.
    pub source: String,
    #[serde(flatten)]
    pub diagnosis: Diagnosis,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeReport {
    pub device: String,
    pub code: ErrorCode,
    pub faults: Vec<Fault>,
}

impl Spellbook {
    pub fn open(root: impl Into<PathBuf>, seed: u64) -> Self {
        let store = Store::new(root);
        let (registry, warnings) = Registry::load(&store.manifests_dir());
        Spellbook {
            store,
            registry,
            warnings,
            seed,
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Problems found while loading user manifests.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn manifest(&self, device: &str) -> Result<&Manifest> {
        self.registry.get(device)
    }

    pub fn devices(&self) -> Vec<DeviceStatus> {
        self.registry
            .all()
            .iter()
            .map(|manifest| DeviceStatus {
                id: manifest.device.id.clone(),
                name: manifest.device.name.clone(),
                kind: manifest.device.kind.clone(),
                transport: match &manifest.transport.target {
                    Some(target) => format!("{}:{target}", manifest.transport.kind),
                    None => manifest.transport.kind.clone(),
                },
                availability: transport::availability(&manifest.transport),
            })
            .collect()
    }

    fn connect(&self, manifest: &Manifest) -> Result<Box<dyn Transport>> {
        let faults = self.curses(&manifest.device.id)?;
        transport::connect(manifest, &faults, self.seed)
    }

    pub fn read(&self, device: &str, at_s: f64) -> Result<LogRecord> {
        let manifest = self.manifest(device)?;
        self.connect(manifest)?.snapshot(at_s)
    }

    pub fn call(
        &self,
        device: &str,
        command: &str,
        params: &BTreeMap<String, String>,
        actor: Actor,
    ) -> Result<CallReport> {
        let manifest = self.manifest(device)?;
        let Some(spec) = manifest.command(command) else {
            let known: Vec<&str> = manifest.commands.iter().map(|c| c.id.as_str()).collect();
            bail!(
                "{device} has no command {command:?}; it knows {}",
                known.join(", ")
            );
        };

        let mut resolved = BTreeMap::new();
        for name in params.keys() {
            if spec.params.iter().all(|param| &param.name != name) {
                bail!("{command} takes no parameter {name:?}");
            }
        }
        for param in &spec.params {
            let value = match params.get(&param.name).or(param.default.as_ref()) {
                Some(value) => value.clone(),
                None => bail!("{command} needs --param {}=<value>", param.name),
            };
            if !param.choices.is_empty() && !param.choices.contains(&value) {
                bail!(
                    "{} must be one of {}, not {value:?}",
                    param.name,
                    param.choices.join(", ")
                );
            }
            resolved.insert(param.name.clone(), value);
        }

        let simulated = manifest.is_simulated();
        authorize(spec.hazard, simulated, actor)?;
        let outcome = self.connect(manifest)?.call(command, &resolved)?;
        let log = if outcome.records.is_empty() {
            None
        } else {
            Some(self.store.record_log(device, &outcome.records)?)
        };
        Ok(CallReport {
            device: device.to_string(),
            command: command.to_string(),
            hazard: spec.hazard,
            simulated,
            message: outcome.message,
            log,
            records: outcome.records,
        })
    }

    /// Diagnose from `log` if given. Otherwise a simulated device runs a fresh
    /// cycle (so injected faults show up immediately) and a real device uses
    /// its latest recorded log.
    pub fn diagnose(&self, device: &str, log: Option<&Path>) -> Result<DiagnosisReport> {
        let manifest = self.manifest(device)?;
        let (source, records) = match log {
            Some(path) => (path.display().to_string(), read_log(path)?),
            None if manifest.is_simulated() => {
                let report = self.call(device, "start_cycle", &BTreeMap::new(), Actor::Agent)?;
                let source = match &report.log {
                    Some(path) => format!("fresh simulated cycle, recorded to {}", path.display()),
                    None => "fresh simulated cycle".to_string(),
                };
                (source, report.records)
            }
            None => match self.store.latest_log(device)? {
                Some(path) => (path.display().to_string(), read_log(&path)?),
                None => bail!("no telemetry recorded for {device} yet; pass --log <file.jsonl>"),
            },
        };
        Ok(DiagnosisReport {
            source,
            diagnosis: diagnose(manifest, &records),
        })
    }

    pub fn lookup_code(&self, device: &str, code: &str) -> Result<CodeReport> {
        let manifest = self.manifest(device)?;
        let Some(entry) = manifest.error_code(code) else {
            let known: Vec<&str> = manifest
                .error_codes
                .iter()
                .map(|e| e.code.as_str())
                .collect();
            bail!(
                "{device} has no error code {code:?}; it knows {}",
                known.join(", ")
            );
        };
        Ok(CodeReport {
            device: device.to_string(),
            faults: entry
                .faults
                .iter()
                .filter_map(|id| manifest.fault(id))
                .cloned()
                .collect(),
            code: entry.clone(),
        })
    }

    pub fn latest_log(&self, device: &str) -> Result<Option<(PathBuf, Vec<LogRecord>)>> {
        self.manifest(device)?;
        match self.store.latest_log(device)? {
            Some(path) => {
                let records = read_log(&path)?;
                Ok(Some((path, records)))
            }
            None => Ok(None),
        }
    }

    /// The faults a simulated device supports, with descriptions.
    pub fn available_curses(
        &self,
        device: &str,
    ) -> Result<&'static [(&'static str, &'static str)]> {
        let manifest = self.manifest(device)?;
        let target = manifest.transport.target.as_deref().unwrap_or("");
        match sim::faults_for(target) {
            Some(faults) if manifest.is_simulated() => Ok(faults),
            _ => bail!("{device} is real hardware; only simulated devices can be cursed"),
        }
    }

    pub fn curses(&self, device: &str) -> Result<BTreeSet<String>> {
        Ok(self
            .store
            .load_state()
            .faults
            .get(device)
            .cloned()
            .unwrap_or_default())
    }

    pub fn curse(&self, device: &str, fault: &str) -> Result<BTreeSet<String>> {
        let available = self.available_curses(device)?;
        if available.iter().all(|(id, _)| *id != fault) {
            let names: Vec<&str> = available.iter().map(|(id, _)| *id).collect();
            bail!(
                "{device} has no fault {fault:?}; choose one of {}",
                names.join(", ")
            );
        }
        let mut state = self.store.load_state();
        state
            .faults
            .entry(device.to_string())
            .or_default()
            .insert(fault.to_string());
        self.store.save_state(&state)?;
        self.curses(device)
    }

    pub fn lift_curses(&self, device: &str) -> Result<()> {
        self.available_curses(device)?;
        let mut state = self.store.load_state();
        state.faults.remove(device);
        self.store.save_state(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> (tempfile::TempDir, Spellbook) {
        let dir = tempfile::tempdir().unwrap();
        let book = Spellbook::open(dir.path(), 42);
        (dir, book)
    }

    #[test]
    fn curses_persist_and_drive_diagnosis() {
        let (_dir, book) = book();
        book.curse("washer", "heater_dead").unwrap();
        let report = book.diagnose("washer", None).unwrap();
        assert_eq!(report.diagnosis.suspects[0].fault, "heater_dead");
        assert!(book.latest_log("washer").unwrap().is_some());

        book.lift_curses("washer").unwrap();
        assert!(book.diagnose("washer", None).unwrap().diagnosis.healthy());
        assert!(book.curse("washer", "gremlins").is_err());
    }

    #[test]
    fn calls_validate_params_against_the_manifest() {
        let (_dir, book) = book();
        let human = Actor::Human { confirmed: false };
        let params = |pairs: &[(&str, &str)]| -> BTreeMap<String, String> {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        assert!(
            book.call("washer", "levitate", &params(&[]), human)
                .is_err()
        );
        assert!(
            book.call("washer", "start_cycle", &params(&[("speed", "9")]), human)
                .is_err()
        );
        assert!(
            book.call(
                "washer",
                "start_cycle",
                &params(&[("program", "turbo")]),
                human
            )
            .is_err()
        );
        assert!(
            book.call("washer", "test_component", &params(&[]), human)
                .is_err()
        );

        let report = book
            .call(
                "washer",
                "test_component",
                &params(&[("component", "heater")]),
                human,
            )
            .unwrap();
        assert!(report.simulated && report.log.is_some());
        assert!(report.message.contains("heater"), "{}", report.message);
    }

    #[test]
    fn error_codes_link_to_their_faults() {
        let (_dir, book) = book();
        let report = book.lookup_code("washer", "OE").unwrap();
        let faults: Vec<&str> = report.faults.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(faults, ["drain_filter_clogged", "drain_pump_failed"]);
        assert!(book.lookup_code("washer", "ZZ").is_err());
    }
}
