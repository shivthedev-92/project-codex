//! Simulated hardware, so the whole pipeline runs without a bench.
//!
//! The washer emits the power, temperature, vibration, and water-level curves
//! a real front-loader produces, and misbehaves the way real ones do when a
//! fault is injected ("cursed").

use std::collections::BTreeSet;

use anyhow::{Result, bail};

use crate::telemetry::LogRecord;

/// Seconds between samples.
pub const STEP_S: f64 = 10.0;

pub const WASHER_FAULTS: &[(&str, &str)] = &[
    (
        "door_lock_failed",
        "The door lock never engages, so no cycle can start.",
    ),
    (
        "inlet_valve_failed",
        "The inlet valve stays shut; the drum never fills.",
    ),
    (
        "heater_dead",
        "The heating element is open; the water stays cold.",
    ),
    (
        "thermistor_failed",
        "The thermistor is stuck; the controller heats blind.",
    ),
    (
        "bearing_worn",
        "Drum bearings are worn; the cabinet shakes.",
    ),
    (
        "drain_pump_failed",
        "The drain pump is dead; water stays in the drum.",
    ),
    (
        "drain_filter_clogged",
        "The drain filter is clogged; the pump strains but water stays.",
    ),
    (
        "unbalanced_load",
        "The load is bunched up; spin is aborted.",
    ),
];

pub const WASHER_PROGRAMS: &[&str] = &["cotton40", "cold"];
pub const WASHER_COMPONENTS: &[&str] = &["valve", "heater", "motor", "pump"];

/// The faults a simulator model supports, or `None` for an unknown model.
pub fn faults_for(target: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match target {
        "washing_machine" => Some(WASHER_FAULTS),
        _ => None,
    }
}

const INLET_TEMP_C: f64 = 20.0;
const FULL_LEVEL_PCT: f64 = 60.0;

pub struct Washer {
    device: String,
    faults: BTreeSet<String>,
    rng: Rng,
}

impl Washer {
    pub fn new(device: &str, faults: &BTreeSet<String>, seed: u64) -> Result<Self> {
        for fault in faults {
            if WASHER_FAULTS.iter().all(|(id, _)| id != fault) {
                bail!("the washer simulator has no fault named {fault:?}");
            }
        }
        Ok(Washer {
            device: device.to_string(),
            faults: faults.clone(),
            rng: Rng::new(seed),
        })
    }

    pub fn is_cursed(&self, fault: &str) -> bool {
        self.faults.contains(fault)
    }

    /// Run a whole program. A fault stops the run with the error code a real
    /// controller would show, at the moment it would show it.
    pub fn run_cycle(&mut self, program: &str) -> Result<Vec<LogRecord>> {
        let heat_to = match program {
            "cotton40" => Some(40.0),
            "cold" => None,
            other => bail!(
                "unknown program {other:?}; choose one of {}",
                WASHER_PROGRAMS.join(", ")
            ),
        };
        let door = self.is_cursed("door_lock_failed");
        let inlet = self.is_cursed("inlet_valve_failed");
        let heater_dead = self.is_cursed("heater_dead");
        let bearing = self.is_cursed("bearing_worn");
        let pump_dead = self.is_cursed("drain_pump_failed");
        let clogged = self.is_cursed("drain_filter_clogged");
        let unbalanced = self.is_cursed("unbalanced_load");
        let mut run = Run::new(
            &self.device,
            &mut self.rng,
            self.faults.contains("thermistor_failed"),
        );

        if door {
            run.sample("start", 2.0, 0.01);
            return Ok(run.fail("dE"));
        }

        // Fill: the valve coil draws a few watts while the level rises.
        if inlet {
            for _ in 0..=30 {
                run.sample("fill", 8.0, 0.01);
            }
            return Ok(run.fail("IE"));
        }
        for step in 1..=12 {
            run.level = FULL_LEVEL_PCT * f64::from(step) / 12.0;
            run.sample("fill", 10.0, 0.01);
        }

        // Heat: about 2 kW until the target temperature.
        if let Some(target) = heat_to {
            if heater_dead {
                for _ in 0..=180 {
                    run.sample("heat", 12.0, 0.02);
                }
                return Ok(run.fail("HE"));
            }
            if run.thermistor_stuck {
                for _ in 0..=180 {
                    run.temp = (run.temp + 0.03).min(90.0);
                    run.sample("heat", 2000.0, 0.02);
                }
                return Ok(run.fail("tE"));
            }
            let start = run.temp;
            for step in 0..90 {
                run.temp = start + (target - start) * f64::from(step) / 89.0;
                run.sample("heat", 2000.0, 0.02);
            }
        }

        // Wash: the motor tumbles back and forth.
        for step in 0..120 {
            run.temp -= (run.temp - INLET_TEMP_C) * 0.001;
            let power =
                250.0 + 60.0 * (f64::from(step) * 0.7).sin() + if bearing { 40.0 } else { 0.0 };
            run.sample("wash", power, if bearing { 0.8 } else { 0.25 });
        }

        // Drain.
        if pump_dead {
            for _ in 0..=30 {
                run.sample("drain", 1.5, 0.01);
            }
            return Ok(run.fail("OE"));
        }
        if clogged {
            for _ in 0..=30 {
                run.level = (run.level - 0.5).max(0.0);
                run.sample("drain", 55.0, 0.02);
            }
            return Ok(run.fail("OE"));
        }
        for step in 1..=12 {
            run.level = FULL_LEVEL_PCT * (1.0 - f64::from(step) / 12.0);
            run.sample("drain", 45.0, 0.02);
        }

        // Spin.
        if unbalanced {
            for _ in 0..6 {
                run.sample("spin", 300.0, 1.6);
            }
            return Ok(run.fail("UE"));
        }
        for _ in 0..60 {
            run.sample("spin", 400.0, if bearing { 1.3 } else { 0.5 });
        }

        run.sample("done", 2.0, 0.01);
        Ok(run.records)
    }

    /// Service mode: exercise one component for 60 s.
    pub fn test_component(&mut self, component: &str) -> Result<Vec<LogRecord>> {
        if !WASHER_COMPONENTS.contains(&component) {
            bail!(
                "unknown component {component:?}; choose one of {}",
                WASHER_COMPONENTS.join(", ")
            );
        }
        let inlet = self.is_cursed("inlet_valve_failed");
        let heater_dead = self.is_cursed("heater_dead");
        let bearing = self.is_cursed("bearing_worn");
        let pump_dead = self.is_cursed("drain_pump_failed");
        let clogged = self.is_cursed("drain_filter_clogged");
        let mut run = Run::new(
            &self.device,
            &mut self.rng,
            self.faults.contains("thermistor_failed"),
        );
        let phase = format!("test:{component}");
        for _ in 0..6 {
            let (power, vibration) = match component {
                "valve" => {
                    if !inlet {
                        run.level += 5.0;
                    }
                    (if inlet { 8.0 } else { 10.0 }, 0.01)
                }
                "heater" => {
                    if !heater_dead {
                        run.temp += 0.2;
                    }
                    (if heater_dead { 12.0 } else { 2000.0 }, 0.02)
                }
                "motor" => (
                    if bearing { 290.0 } else { 250.0 },
                    if bearing { 0.8 } else { 0.25 },
                ),
                _ => (
                    if pump_dead {
                        1.5
                    } else if clogged {
                        55.0
                    } else {
                        45.0
                    },
                    0.02,
                ),
            };
            run.sample(&phase, power, vibration);
        }
        Ok(run.records)
    }
}

struct Run<'a> {
    device: &'a str,
    rng: &'a mut Rng,
    thermistor_stuck: bool,
    t: f64,
    level: f64,
    temp: f64,
    records: Vec<LogRecord>,
}

impl<'a> Run<'a> {
    fn new(device: &'a str, rng: &'a mut Rng, thermistor_stuck: bool) -> Self {
        Run {
            device,
            rng,
            thermistor_stuck,
            t: 0.0,
            level: 0.0,
            temp: INLET_TEMP_C,
            records: Vec::new(),
        }
    }

    /// Record one sample with sensor noise, then advance the clock.
    fn sample(&mut self, phase: &str, power: f64, vibration: f64) {
        let power = power * (1.0 + self.rng.jitter(0.015));
        let vibration = vibration * (1.0 + self.rng.jitter(0.1));
        let temp = if self.thermistor_stuck {
            INLET_TEMP_C
        } else {
            self.temp + self.rng.jitter(0.1)
        };
        let readings = [
            ("power_w", power),
            ("water_temp_c", temp),
            ("vibration_g", vibration),
            ("water_level_pct", self.level),
        ]
        .into_iter()
        .map(|(id, value)| (id.to_string(), (value * 100.0).round() / 100.0))
        .collect();
        self.records.push(LogRecord {
            t: self.t,
            device: self.device.to_string(),
            phase: Some(phase.to_string()),
            readings,
            error: None,
        });
        self.t += STEP_S;
    }

    fn fail(mut self, code: &str) -> Vec<LogRecord> {
        if let Some(last) = self.records.last_mut() {
            last.error = Some(code.to_string());
        }
        self.records
    }
}

/// xorshift64*: small, fast, and reproducible for a given seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }

    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let value = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (value >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform noise in `[-amplitude, amplitude]`.
    fn jitter(&mut self, amplitude: f64) -> f64 {
        (self.next_f64() * 2.0 - 1.0) * amplitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn washer(faults: &[&str]) -> Washer {
        let faults = faults.iter().map(|fault| fault.to_string()).collect();
        Washer::new("washer", &faults, 7).unwrap()
    }

    #[test]
    fn healthy_cycle_runs_every_phase_without_errors() {
        let records = washer(&[]).run_cycle("cotton40").unwrap();
        let phases: Vec<&str> = records.iter().filter_map(|r| r.phase.as_deref()).collect();
        for phase in ["fill", "heat", "wash", "drain", "spin", "done"] {
            assert!(phases.contains(&phase), "missing {phase}");
        }
        assert!(records.iter().all(|r| r.error.is_none()));
    }

    #[test]
    fn cold_program_skips_heating() {
        let records = washer(&[]).run_cycle("cold").unwrap();
        assert!(records.iter().all(|r| r.phase.as_deref() != Some("heat")));
    }

    #[test]
    fn faults_stop_with_their_error_code() {
        for (fault, code) in [
            ("heater_dead", "HE"),
            ("drain_pump_failed", "OE"),
            ("door_lock_failed", "dE"),
        ] {
            let records = washer(&[fault]).run_cycle("cotton40").unwrap();
            assert_eq!(
                records.last().unwrap().error.as_deref(),
                Some(code),
                "{fault}"
            );
        }
    }

    #[test]
    fn same_seed_gives_same_run() {
        assert_eq!(
            washer(&[]).run_cycle("cotton40").unwrap(),
            washer(&[]).run_cycle("cotton40").unwrap()
        );
    }

    #[test]
    fn unknown_faults_and_programs_are_rejected() {
        let faults = BTreeSet::from(["melted".to_string()]);
        assert!(Washer::new("washer", &faults, 1).is_err());
        assert!(washer(&[]).run_cycle("turbo").is_err());
        assert!(washer(&[]).test_component("flux_capacitor").is_err());
    }
}
