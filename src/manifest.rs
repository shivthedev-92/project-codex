//! Device manifests: a datasheet as code.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::safety::Hazard;

pub const SCHEMA: &str = "spellbook.manifest/0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub device: Device,
    pub transport: TransportSpec,
    #[serde(default)]
    pub sensors: Vec<Sensor>,
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    #[serde(default)]
    pub error_codes: Vec<ErrorCode>,
    #[serde(default)]
    pub faults: Vec<Fault>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportSpec {
    /// The adapter: `sim`, `serial`, `mqtt`, `modbus`, ...
    pub kind: String,
    /// Adapter-specific address: a simulator model, a serial port, an MQTT topic.
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sensor {
    pub id: String,
    pub unit: String,
    #[serde(default)]
    pub description: String,
    /// Rated range; readings outside it are always reported.
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub hazard: Hazard,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// A parameter without a default is required.
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase {
    pub id: String,
    #[serde(default)]
    pub nominal_s: Option<f64>,
    #[serde(default)]
    pub timeout_s: Option<f64>,
    #[serde(default)]
    pub expect: Vec<Expectation>,
}

/// What a sensor should do during a phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    pub sensor: String,
    #[serde(default)]
    pub mean_min: Option<f64>,
    #[serde(default)]
    pub mean_max: Option<f64>,
    /// Minimum rise from the first reading of the phase to its peak.
    #[serde(default)]
    pub rise_min: Option<f64>,
    #[serde(default)]
    pub peak_max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorCode {
    pub code: String,
    pub meaning: String,
    #[serde(default)]
    pub causes: Vec<String>,
    #[serde(default)]
    pub faults: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fault {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Symptoms in [`Symptom`] notation, such as `low:power_w@heat`.
    pub symptoms: Vec<String>,
    #[serde(default)]
    pub parts: Vec<String>,
    #[serde(default)]
    pub hazard: Hazard,
    #[serde(default)]
    pub repair: Vec<String>,
}

/// Evidence the diagnosis engine can observe, written as a short string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symptom {
    /// `low:<sensor>@<phase>`: the phase mean fell below `mean_min`.
    Low { sensor: String, phase: String },
    /// `high:<sensor>@<phase>`: the phase mean exceeded `mean_max`.
    High { sensor: String, phase: String },
    /// `no_rise:<sensor>@<phase>`: the sensor rose less than `rise_min`.
    NoRise { sensor: String, phase: String },
    /// `peak:<sensor>@<phase>`: the sensor peaked above `peak_max`.
    Peak { sensor: String, phase: String },
    /// `timeout:<phase>`: the phase ran until its timeout.
    Timeout { phase: String },
    /// `code:<CODE>`: the device reported an error code.
    Code(String),
    /// `range:<sensor>`: a reading fell outside the sensor's rated range.
    OutOfRange { sensor: String },
}

impl Symptom {
    fn sensor(&self) -> Option<&str> {
        match self {
            Symptom::Low { sensor, .. }
            | Symptom::High { sensor, .. }
            | Symptom::NoRise { sensor, .. }
            | Symptom::Peak { sensor, .. }
            | Symptom::OutOfRange { sensor } => Some(sensor),
            Symptom::Timeout { .. } | Symptom::Code(_) => None,
        }
    }

    fn phase(&self) -> Option<&str> {
        match self {
            Symptom::Low { phase, .. }
            | Symptom::High { phase, .. }
            | Symptom::NoRise { phase, .. }
            | Symptom::Peak { phase, .. }
            | Symptom::Timeout { phase } => Some(phase),
            Symptom::Code(_) | Symptom::OutOfRange { .. } => None,
        }
    }
}

impl fmt::Display for Symptom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symptom::Low { sensor, phase } => write!(f, "low:{sensor}@{phase}"),
            Symptom::High { sensor, phase } => write!(f, "high:{sensor}@{phase}"),
            Symptom::NoRise { sensor, phase } => write!(f, "no_rise:{sensor}@{phase}"),
            Symptom::Peak { sensor, phase } => write!(f, "peak:{sensor}@{phase}"),
            Symptom::Timeout { phase } => write!(f, "timeout:{phase}"),
            Symptom::Code(code) => write!(f, "code:{code}"),
            Symptom::OutOfRange { sensor } => write!(f, "range:{sensor}"),
        }
    }
}

impl FromStr for Symptom {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self> {
        let (kind, rest) = text.split_once(':').ok_or_else(|| {
            anyhow!("symptom {text:?} is missing its kind, as in low:power_w@heat")
        })?;
        let sensor_at_phase = || -> Result<(String, String)> {
            match rest.split_once('@') {
                Some((sensor, phase)) if !sensor.is_empty() && !phase.is_empty() => {
                    Ok((sensor.to_string(), phase.to_string()))
                }
                _ => bail!("symptom {text:?} needs <sensor>@<phase>"),
            }
        };
        if rest.is_empty() {
            bail!("symptom {text:?} is empty after {kind:?}");
        }
        Ok(match kind {
            "low" => {
                let (sensor, phase) = sensor_at_phase()?;
                Symptom::Low { sensor, phase }
            }
            "high" => {
                let (sensor, phase) = sensor_at_phase()?;
                Symptom::High { sensor, phase }
            }
            "no_rise" => {
                let (sensor, phase) = sensor_at_phase()?;
                Symptom::NoRise { sensor, phase }
            }
            "peak" => {
                let (sensor, phase) = sensor_at_phase()?;
                Symptom::Peak { sensor, phase }
            }
            "timeout" => Symptom::Timeout {
                phase: rest.to_string(),
            },
            "code" => Symptom::Code(rest.to_string()),
            "range" => Symptom::OutOfRange {
                sensor: rest.to_string(),
            },
            other => bail!(
                "symptom kind {other:?} is not one of low, high, no_rise, peak, timeout, code, range"
            ),
        })
    }
}

impl Manifest {
    /// Parse and validate a manifest.
    pub fn from_toml(source: &str) -> Result<Self> {
        let manifest: Manifest = toml::from_str(source).map_err(|error| anyhow!("{error}"))?;
        let problems = manifest.problems();
        if !problems.is_empty() {
            bail!(
                "manifest {:?} has problems:\n  - {}",
                manifest.device.id,
                problems.join("\n  - ")
            );
        }
        Ok(manifest)
    }

    pub fn is_simulated(&self) -> bool {
        self.transport.kind == "sim"
    }

    pub fn sensor(&self, id: &str) -> Option<&Sensor> {
        self.sensors.iter().find(|sensor| sensor.id == id)
    }

    pub fn command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.id == id)
    }

    pub fn phase(&self, id: &str) -> Option<&Phase> {
        self.phases.iter().find(|phase| phase.id == id)
    }

    pub fn fault(&self, id: &str) -> Option<&Fault> {
        self.faults.iter().find(|fault| fault.id == id)
    }

    /// Codes match exactly first, then case-insensitively (`te` finds `tE`).
    pub fn error_code(&self, code: &str) -> Option<&ErrorCode> {
        self.error_codes
            .iter()
            .find(|entry| entry.code == code)
            .or_else(|| {
                self.error_codes
                    .iter()
                    .find(|entry| entry.code.eq_ignore_ascii_case(code))
            })
    }

    /// Everything wrong with the manifest, as human-readable sentences.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema != SCHEMA {
            problems.push(format!("schema is {:?}; expected {SCHEMA:?}", self.schema));
        }
        let id = &self.device.id;
        if id.is_empty() || id.contains(|c: char| c.is_whitespace() || c == '/') {
            problems.push(format!(
                "device id {id:?} must be non-empty, without spaces or slashes"
            ));
        }

        duplicates(
            &mut problems,
            "sensor",
            self.sensors.iter().map(|s| s.id.as_str()),
        );
        duplicates(
            &mut problems,
            "command",
            self.commands.iter().map(|c| c.id.as_str()),
        );
        duplicates(
            &mut problems,
            "phase",
            self.phases.iter().map(|p| p.id.as_str()),
        );
        duplicates(
            &mut problems,
            "error code",
            self.error_codes.iter().map(|e| e.code.as_str()),
        );
        duplicates(
            &mut problems,
            "fault",
            self.faults.iter().map(|f| f.id.as_str()),
        );

        for command in &self.commands {
            for param in &command.params {
                if let Some(default) = &param.default
                    && !param.choices.is_empty()
                    && !param.choices.contains(default)
                {
                    problems.push(format!(
                        "command {} param {} defaults to {default:?}, which is not one of its choices",
                        command.id, param.name
                    ));
                }
            }
        }

        for phase in &self.phases {
            for expectation in &phase.expect {
                if self.sensor(&expectation.sensor).is_none() {
                    problems.push(format!(
                        "phase {} expects unknown sensor {:?}",
                        phase.id, expectation.sensor
                    ));
                }
            }
        }

        for entry in &self.error_codes {
            for fault in &entry.faults {
                if self.fault(fault).is_none() {
                    problems.push(format!(
                        "error code {} links unknown fault {fault:?}",
                        entry.code
                    ));
                }
            }
        }

        for fault in &self.faults {
            for text in &fault.symptoms {
                let symptom = match text.parse::<Symptom>() {
                    Ok(symptom) => symptom,
                    Err(error) => {
                        problems.push(format!("fault {}: {error}", fault.id));
                        continue;
                    }
                };
                if let Some(sensor) = symptom.sensor()
                    && self.sensor(sensor).is_none()
                {
                    problems.push(format!(
                        "fault {} symptom {text} names unknown sensor",
                        fault.id
                    ));
                }
                if let Some(phase) = symptom.phase()
                    && self.phase(phase).is_none()
                {
                    problems.push(format!(
                        "fault {} symptom {text} names unknown phase",
                        fault.id
                    ));
                }
                if let Symptom::Code(code) = &symptom
                    && self.error_codes.iter().all(|entry| entry.code != *code)
                {
                    problems.push(format!(
                        "fault {} symptom {text} names unknown error code",
                        fault.id
                    ));
                }
            }
        }
        problems
    }
}

fn duplicates<'a>(problems: &mut Vec<String>, what: &str, ids: impl Iterator<Item = &'a str>) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            problems.push(format!("{what} {id:?} is defined more than once"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symptoms_round_trip() {
        for text in [
            "low:power_w@heat",
            "high:power_w@spin",
            "no_rise:water_temp_c@heat",
            "peak:vibration_g@spin",
            "timeout:drain",
            "code:OE",
            "range:power_w",
        ] {
            assert_eq!(text.parse::<Symptom>().unwrap().to_string(), text);
        }
    }

    #[test]
    fn malformed_symptoms_are_rejected() {
        for text in [
            "power_w",
            "low:power_w",
            "low:@heat",
            "melt:power_w@heat",
            "code:",
        ] {
            assert!(text.parse::<Symptom>().is_err(), "{text} should not parse");
        }
    }

    #[test]
    fn problems_name_dangling_references() {
        let source = r#"
            schema = "spellbook.manifest/0.1"
            [device]
            id = "kettle"
            name = "Kettle"
            kind = "appliance.kettle"
            [transport]
            kind = "mqtt"
            [[sensors]]
            id = "power_w"
            unit = "W"
            [[phases]]
            id = "boil"
            expect = [{ sensor = "temp_c", rise_min = 50.0 }]
            [[faults]]
            id = "element_dead"
            name = "Element dead"
            symptoms = ["low:power_w@heat", "code:E1"]
        "#;
        let error = Manifest::from_toml(source).unwrap_err().to_string();
        assert!(error.contains("unknown sensor \"temp_c\""), "{error}");
        assert!(error.contains("names unknown phase"), "{error}");
        assert!(error.contains("names unknown error code"), "{error}");
    }

    #[test]
    fn error_codes_fall_back_to_case_insensitive_matches() {
        let manifest = crate::registry::Registry::builtin();
        let washer = manifest.get("washer").unwrap();
        assert_eq!(washer.error_code("te").unwrap().code, "tE");
        assert_eq!(washer.error_code("OE").unwrap().code, "OE");
        assert!(washer.error_code("ZZ").is_none());
    }
}
