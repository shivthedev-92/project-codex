//! The telemetry log: one JSON object per line.
//!
//! Anything can produce it: the simulator, a smart plug's MQTT feed, or a
//! script that converts a CSV export. `phase` is optional; without it the
//! diagnosis engine can still check rated ranges and error codes.
//!
//! ```text
//! {"t":0.0,"device":"washer","phase":"fill","readings":{"power_w":10.2,"water_level_pct":5.0}}
//! {"t":300.0,"device":"washer","phase":"drain","readings":{"power_w":1.5},"error":"OE"}
//! ```

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRecord {
    /// Seconds since the start of the recording.
    pub t: f64,
    pub device: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub readings: BTreeMap<String, f64>,
    /// An error code the device reported at this moment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn write_log(path: &Path, records: &[LogRecord]) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("could not create {}", path.display()))?;
    let mut out = BufWriter::new(file);
    for record in records {
        serde_json::to_writer(&mut out, record)?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}

pub fn read_log(path: &Path) -> Result<Vec<LogRecord>> {
    let file = File::open(path).with_context(|| format!("could not open {}", path.display()))?;
    let mut records = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str(&line).with_context(|| {
            format!(
                "{}:{} is not a Spellbook log record",
                path.display(),
                index + 1
            )
        })?;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_round_trip_and_tolerate_minimal_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plug.jsonl");
        let records = vec![LogRecord {
            t: 1.5,
            device: "plug".into(),
            phase: Some("heat".into()),
            readings: BTreeMap::from([("power_w".into(), 1999.5)]),
            error: Some("HE".into()),
        }];
        write_log(&path, &records).unwrap();
        assert_eq!(read_log(&path).unwrap(), records);

        std::fs::write(&path, "{\"t\":0,\"device\":\"plug\"}\n\n").unwrap();
        let minimal = read_log(&path).unwrap();
        assert_eq!(minimal.len(), 1);
        assert!(minimal[0].phase.is_none() && minimal[0].readings.is_empty());
    }

    #[test]
    fn bad_lines_report_their_line_number() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.jsonl");
        std::fs::write(&path, "{\"t\":0,\"device\":\"plug\"}\nnot json\n").unwrap();
        let error = format!("{:#}", read_log(&path).unwrap_err());
        assert!(error.contains("broken.jsonl:2"), "{error}");
    }
}
