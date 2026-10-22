//! One interface over every bus.
//!
//! Each adapter (serial, MQTT, Modbus, CAN, BLE, ...) implements [`Transport`].
//! Spellbook ships the simulator today; the others are planned and plug in here.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::manifest::{Manifest, TransportSpec};
use crate::sim::{STEP_S, Washer};
use crate::telemetry::LogRecord;

pub const PLANNED_ADAPTERS: &[&str] = &["serial", "mqtt", "modbus", "can", "ble", "i2c"];

pub struct CallOutcome {
    pub message: String,
    /// Telemetry produced by the call, recorded to a log when non-empty.
    pub records: Vec<LogRecord>,
}

pub trait Transport {
    fn simulated(&self) -> bool;

    /// Read every sensor. `at_s` places simulated devices that many seconds
    /// into a standard cycle; live adapters read the present value.
    fn snapshot(&mut self, at_s: f64) -> Result<LogRecord>;

    /// Invoke a command already validated against the manifest and authorized.
    fn call(&mut self, command: &str, params: &BTreeMap<String, String>) -> Result<CallOutcome>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Simulated,
    Planned,
    Unknown,
}

pub fn availability(spec: &TransportSpec) -> Availability {
    match spec.kind.as_str() {
        "sim" => Availability::Simulated,
        kind if PLANNED_ADAPTERS.contains(&kind) => Availability::Planned,
        _ => Availability::Unknown,
    }
}

pub fn connect(
    manifest: &Manifest,
    faults: &BTreeSet<String>,
    seed: u64,
) -> Result<Box<dyn Transport>> {
    let spec = &manifest.transport;
    match availability(spec) {
        Availability::Simulated => match spec.target.as_deref() {
            Some("washing_machine") => Ok(Box::new(SimWasher {
                sim: Washer::new(&manifest.device.id, faults, seed)?,
            })),
            other => bail!(
                "there is no simulator model named {:?}",
                other.unwrap_or("")
            ),
        },
        Availability::Planned => bail!(
            "the {} adapter is not built yet; record a log from the device and run /goblin --log <file>",
            spec.kind
        ),
        Availability::Unknown => bail!("unknown transport {:?}", spec.kind),
    }
}

struct SimWasher {
    sim: Washer,
}

impl Transport for SimWasher {
    fn simulated(&self) -> bool {
        true
    }

    fn snapshot(&mut self, at_s: f64) -> Result<LogRecord> {
        let records = self.sim.run_cycle("cotton40")?;
        let index = records
            .partition_point(|record| record.t <= at_s)
            .saturating_sub(1);
        Ok(records[index].clone())
    }

    fn call(&mut self, command: &str, params: &BTreeMap<String, String>) -> Result<CallOutcome> {
        let quiet = |message: &str| CallOutcome {
            message: message.to_string(),
            records: Vec::new(),
        };
        match command {
            "start_cycle" => {
                let program = params.get("program").map_or("cotton40", String::as_str);
                let records = self.sim.run_cycle(program)?;
                let last = records.last().expect("a cycle always records a sample");
                let message = match &last.error {
                    Some(code) => format!(
                        "The {program} cycle stopped at {:.0} s with error {code}.",
                        last.t
                    ),
                    None => format!(
                        "The {program} cycle finished in {:.0} min.",
                        (last.t + STEP_S) / 60.0
                    ),
                };
                Ok(CallOutcome { message, records })
            }
            "test_component" => {
                let component = params.get("component").map_or("", String::as_str);
                let records = self.sim.test_component(component)?;
                let mean = records
                    .iter()
                    .filter_map(|r| r.readings.get("power_w"))
                    .sum::<f64>()
                    / records.len() as f64;
                let message = format!(
                    "The {component} drew {mean:.0} W on average over {:.0} s.",
                    records.len() as f64 * STEP_S
                );
                Ok(CallOutcome { message, records })
            }
            "unlock_door" if self.sim.is_cursed("door_lock_failed") => {
                Ok(quiet("The door lock did not respond."))
            }
            "unlock_door" => Ok(quiet("Door unlocked.")),
            "stop" => Ok(quiet("Machine stopped.")),
            other => bail!("the washer simulator does not implement {other:?}"),
        }
    }
}
