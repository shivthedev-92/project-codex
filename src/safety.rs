//! Safety wards: who may actuate what, and where.
//!
//! Reads are always allowed. Anything a simulator does is allowed, because
//! simulation comes before actuation. On real hardware, AI agents may only run
//! `safe` commands; `caution` and `danger` commands need a human with `--confirm`.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hazard {
    #[default]
    Safe,
    Caution,
    Danger,
}

impl Hazard {
    pub fn label(self) -> &'static str {
        match self {
            Hazard::Safe => "safe",
            Hazard::Caution => "caution",
            Hazard::Danger => "danger",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A person at the terminal. `confirmed` is the `--confirm` flag.
    Human { confirmed: bool },
    /// An AI agent connected through the MCP portal.
    Agent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denied(pub String);

impl fmt::Display for Denied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Denied {}

pub fn authorize(hazard: Hazard, simulated: bool, actor: Actor) -> Result<(), Denied> {
    if simulated || hazard == Hazard::Safe {
        return Ok(());
    }
    match actor {
        Actor::Human { confirmed: true } => Ok(()),
        Actor::Human { confirmed: false } => Err(Denied(format!(
            "this is a {} command on real hardware; make the area safe, then re-run with --confirm",
            hazard.label()
        ))),
        Actor::Agent => Err(Denied(format!(
            "AI agents may only run safe commands on real hardware; a human must cast this {} command with --confirm",
            hazard.label()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulators_allow_everything() {
        for hazard in [Hazard::Safe, Hazard::Caution, Hazard::Danger] {
            assert!(authorize(hazard, true, Actor::Agent).is_ok());
            assert!(authorize(hazard, true, Actor::Human { confirmed: false }).is_ok());
        }
    }

    #[test]
    fn real_hardware_needs_a_confirming_human_above_safe() {
        assert!(authorize(Hazard::Safe, false, Actor::Agent).is_ok());
        for hazard in [Hazard::Caution, Hazard::Danger] {
            assert!(authorize(hazard, false, Actor::Agent).is_err());
            assert!(authorize(hazard, false, Actor::Human { confirmed: false }).is_err());
            assert!(authorize(hazard, false, Actor::Human { confirmed: true }).is_ok());
        }
    }
}
