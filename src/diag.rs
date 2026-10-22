//! The diagnosis engine: telemetry in, ranked suspects out.
//!
//! It turns a log into symptoms by comparing each phase with the manifest's
//! expectations, then scores every known fault by how well its symptoms match
//! what was observed (Jaccard similarity, so extra or missing evidence counts
//! against a suspect).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::manifest::{Manifest, Symptom};
use crate::safety::Hazard;
use crate::telemetry::LogRecord;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SensorStats {
    pub first: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhaseStats {
    pub phase: String,
    pub start_s: f64,
    pub duration_s: f64,
    pub samples: usize,
    pub sensors: BTreeMap<String, SensorStats>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub symptom: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Suspect {
    pub fault: String,
    pub name: String,
    /// 0.0–1.0: how closely the observed symptoms match this fault.
    pub score: f64,
    pub matched: Vec<String>,
    pub missing: Vec<String>,
    pub hazard: Hazard,
    pub parts: Vec<String>,
    pub repair: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnosis {
    pub device: String,
    pub samples: usize,
    pub error_codes: Vec<String>,
    pub findings: Vec<Finding>,
    pub suspects: Vec<Suspect>,
    pub phases: Vec<PhaseStats>,
}

impl Diagnosis {
    pub fn healthy(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Split a log into contiguous phase segments and summarise each sensor.
pub fn phase_stats(records: &[LogRecord]) -> Vec<PhaseStats> {
    let mut stats = Vec::new();
    let mut segment: Vec<&LogRecord> = Vec::new();
    for record in records {
        let continues = segment
            .last()
            .is_none_or(|previous| previous.phase == record.phase);
        if !continues {
            stats.extend(summarize(&segment));
            segment.clear();
        }
        segment.push(record);
    }
    stats.extend(summarize(&segment));
    stats
}

fn summarize(segment: &[&LogRecord]) -> Option<PhaseStats> {
    let first = segment.first()?;
    let phase = first.phase.clone()?;
    let last = segment.last()?;
    let mut values: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for record in segment {
        for (sensor, value) in &record.readings {
            values.entry(sensor).or_default().push(*value);
        }
    }
    let sensors = values
        .into_iter()
        .map(|(sensor, values)| {
            let stats = SensorStats {
                first: values[0],
                mean: values.iter().sum::<f64>() / values.len() as f64,
                min: values.iter().copied().fold(f64::INFINITY, f64::min),
                max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            };
            (sensor.to_string(), stats)
        })
        .collect();
    Some(PhaseStats {
        phase,
        start_s: first.t,
        duration_s: last.t - first.t,
        samples: segment.len(),
        sensors,
    })
}

pub fn diagnose(manifest: &Manifest, records: &[LogRecord]) -> Diagnosis {
    let phases = phase_stats(records);
    let mut findings: Vec<Finding> = Vec::new();
    let mut note = |symptom: Symptom, detail: String| {
        let symptom = symptom.to_string();
        if findings.iter().all(|finding| finding.symptom != symptom) {
            findings.push(Finding { symptom, detail });
        }
    };
    let unit = |sensor: &str| manifest.sensor(sensor).map_or("", |s| s.unit.as_str());

    for stats in &phases {
        let Some(spec) = manifest.phase(&stats.phase) else {
            continue;
        };
        let phase = stats.phase.clone();
        if let Some(timeout) = spec.timeout_s
            && stats.duration_s >= timeout
        {
            note(
                Symptom::Timeout {
                    phase: phase.clone(),
                },
                format!(
                    "{phase} ran {:.0} s and hit its {timeout:.0} s timeout",
                    stats.duration_s
                ),
            );
        }
        for expect in &spec.expect {
            let Some(seen) = stats.sensors.get(&expect.sensor) else {
                continue;
            };
            let (sensor, unit) = (expect.sensor.clone(), unit(&expect.sensor));
            if let Some(min) = expect.mean_min
                && seen.mean < min
            {
                note(
                    Symptom::Low {
                        sensor: sensor.clone(),
                        phase: phase.clone(),
                    },
                    format!(
                        "{sensor} averaged {:.1} {unit} during {phase}; expected at least {min} {unit}",
                        seen.mean
                    ),
                );
            }
            if let Some(max) = expect.mean_max
                && seen.mean > max
            {
                note(
                    Symptom::High {
                        sensor: sensor.clone(),
                        phase: phase.clone(),
                    },
                    format!(
                        "{sensor} averaged {:.1} {unit} during {phase}; expected at most {max} {unit}",
                        seen.mean
                    ),
                );
            }
            if let Some(rise) = expect.rise_min
                && seen.max - seen.first < rise
            {
                note(
                    Symptom::NoRise {
                        sensor: sensor.clone(),
                        phase: phase.clone(),
                    },
                    format!(
                        "{sensor} rose only {:.1} {unit} during {phase}; expected at least {rise} {unit}",
                        seen.max - seen.first
                    ),
                );
            }
            if let Some(peak) = expect.peak_max
                && seen.max > peak
            {
                note(
                    Symptom::Peak {
                        sensor: sensor.clone(),
                        phase: phase.clone(),
                    },
                    format!(
                        "{sensor} peaked at {:.2} {unit} during {phase}; expected at most {peak} {unit}",
                        seen.max
                    ),
                );
            }
        }
    }

    for record in records {
        for (sensor, value) in &record.readings {
            let Some(spec) = manifest.sensor(sensor) else {
                continue;
            };
            if spec.min.is_some_and(|min| *value < min) || spec.max.is_some_and(|max| *value > max)
            {
                note(
                    Symptom::OutOfRange {
                        sensor: sensor.clone(),
                    },
                    format!(
                        "{sensor} read {value} {} at {:.0} s, outside its rated range",
                        spec.unit, record.t
                    ),
                );
            }
        }
    }

    let mut error_codes: Vec<String> = Vec::new();
    for record in records {
        let Some(code) = &record.error else {
            continue;
        };
        if error_codes.contains(code) {
            continue;
        }
        error_codes.push(code.clone());
        let meaning = manifest
            .error_code(code)
            .map_or("not listed in the manifest", |entry| entry.meaning.as_str());
        note(
            Symptom::Code(code.clone()),
            format!("{code} at {:.0} s: {meaning}", record.t),
        );
    }

    let observed: BTreeSet<&str> = findings
        .iter()
        .map(|finding| finding.symptom.as_str())
        .collect();
    let mut suspects: Vec<Suspect> = manifest
        .faults
        .iter()
        .filter_map(|fault| {
            // Canonicalise so `code:OE` written by hand matches what we emit.
            let expected: BTreeSet<String> = fault
                .symptoms
                .iter()
                .filter_map(|text| text.parse::<Symptom>().ok())
                .map(|symptom| symptom.to_string())
                .collect();
            let expected: BTreeSet<&str> = expected.iter().map(String::as_str).collect();
            let matched: Vec<String> = expected
                .intersection(&observed)
                .map(|s| s.to_string())
                .collect();
            if matched.is_empty() {
                return None;
            }
            let union = expected.union(&observed).count();
            Some(Suspect {
                fault: fault.id.clone(),
                name: fault.name.clone(),
                score: matched.len() as f64 / union as f64,
                missing: expected
                    .difference(&observed)
                    .map(|s| s.to_string())
                    .collect(),
                matched,
                hazard: fault.hazard,
                parts: fault.parts.clone(),
                repair: fault.repair.clone(),
            })
        })
        .collect();
    suspects.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.fault.cmp(&b.fault))
    });

    Diagnosis {
        device: manifest.device.id.clone(),
        samples: records.len(),
        error_codes,
        findings,
        suspects,
        phases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;
    use crate::sim::{WASHER_FAULTS, Washer};

    fn cycle(faults: &[&str]) -> Vec<LogRecord> {
        let faults = faults.iter().map(|fault| fault.to_string()).collect();
        Washer::new("washer", &faults, 42)
            .unwrap()
            .run_cycle("cotton40")
            .unwrap()
    }

    #[test]
    fn a_healthy_cycle_has_no_findings() {
        let registry = Registry::builtin();
        let diagnosis = diagnose(registry.get("washer").unwrap(), &cycle(&[]));
        assert!(diagnosis.healthy(), "{:?}", diagnosis.findings);
        assert!(diagnosis.suspects.is_empty());
    }

    #[test]
    fn every_simulated_fault_is_the_top_suspect_with_full_confidence() {
        let registry = Registry::builtin();
        let washer = registry.get("washer").unwrap();
        for (fault, _) in WASHER_FAULTS {
            let diagnosis = diagnose(washer, &cycle(&[fault]));
            let top = diagnosis
                .suspects
                .first()
                .unwrap_or_else(|| panic!("{fault}: no suspects"));
            assert_eq!(top.fault, *fault, "{fault}: {:?}", diagnosis.suspects);
            assert_eq!(top.score, 1.0, "{fault}: {:?}", diagnosis.findings);
        }
    }

    #[test]
    fn related_faults_rank_below_the_real_one() {
        let registry = Registry::builtin();
        let diagnosis = diagnose(
            registry.get("washer").unwrap(),
            &cycle(&["drain_filter_clogged"]),
        );
        let ranked: Vec<&str> = diagnosis
            .suspects
            .iter()
            .map(|s| s.fault.as_str())
            .collect();
        assert_eq!(ranked, ["drain_filter_clogged", "drain_pump_failed"]);
        assert!(diagnosis.suspects[1].score < 1.0);
    }

    #[test]
    fn phaseless_logs_still_check_ranges_and_codes() {
        let registry = Registry::builtin();
        let record = LogRecord {
            t: 5.0,
            device: "washer".into(),
            phase: None,
            readings: BTreeMap::from([("water_temp_c".into(), 120.0)]),
            error: Some("tE".into()),
        };
        let diagnosis = diagnose(registry.get("washer").unwrap(), &[record]);
        let symptoms: Vec<&str> = diagnosis
            .findings
            .iter()
            .map(|f| f.symptom.as_str())
            .collect();
        assert_eq!(symptoms, ["range:water_temp_c", "code:tE"]);
        assert_eq!(diagnosis.suspects[0].fault, "thermistor_failed");
        assert!(diagnosis.phases.is_empty());
    }
}
