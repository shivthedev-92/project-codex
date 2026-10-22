// ███████╗██████╗ ███████╗██╗     ██╗     ██████╗  ██████╗  ██████╗ ██╗  ██╗
// ██╔════╝██╔══██╗██╔════╝██║     ██║     ██╔══██╗██╔═══██╗██╔═══██╗██║ ██╔╝
// ███████╗██████╔╝█████╗  ██║     ██║     ██████╔╝██║   ██║██║   ██║█████╔╝
// ╚════██║██╔═══╝ ██╔══╝  ██║     ██║     ██╔══██╗██║   ██║██║   ██║██╔═██╗
// ███████║██║     ███████╗███████╗███████╗██████╔╝╚██████╔╝╚██████╔╝██║  ██╗
// ╚══════╝╚═╝     ╚══════╝╚══════╝╚══════╝╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝

//! Spellbook entrypoint: argument parsing, the interactive session, and
//! rendering each spell's result.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, anyhow};
use clap::{CommandFactory, Parser, Subcommand};

use spellbook::book::Spellbook;
use spellbook::diag::{PhaseStats, phase_stats};
use spellbook::manifest::Manifest;
use spellbook::mcp;
use spellbook::safety::{Actor, Hazard};
use spellbook::store::Store;
use spellbook::telemetry::LogRecord;
use spellbook::transport::Availability;
use spellbook::ui::{self, hazard_style, paint, palette, panel, table};

#[derive(Parser, Debug)]
#[command(
    name = "spellbook",
    version,
    about = "A retro fantasy grimoire for talking to, diagnosing, and repairing hardware.",
    disable_help_subcommand = true
)]
struct Cli {
    /// Where Spellbook keeps state, logs, and your own manifests.
    #[arg(long, global = true, env = "SPELLBOOK_HOME")]
    home: Option<PathBuf>,

    /// Seed for the simulators, so runs are reproducible.
    #[arg(long, global = true, default_value_t = 42)]
    seed: u64,

    #[command(subcommand)]
    spell: Option<Spell>,
}

#[derive(Subcommand, Debug)]
enum Spell {
    /// Open the spellbook and show every spell.
    #[command(visible_alias = "spellbook")]
    Help,
    /// Close the spellbook.
    #[command(visible_aliases = ["quit", "bye"])]
    Exit,
    /// Discover the devices Spellbook can reach.
    #[command(visible_alias = "scan")]
    Map,
    /// Read a device's manifest: sensors, commands, codes, and faults.
    #[command(visible_alias = "describe")]
    Scroll { device: String },
    /// Read sensor values.
    #[command(visible_alias = "read")]
    Scry {
        device: String,
        /// One sensor; all of them when omitted.
        sensor: Option<String>,
        /// Simulated devices: seconds into a standard cycle.
        #[arg(long, default_value_t = 0.0)]
        at: f64,
    },
    /// Invoke a device command, behind the safety wards.
    #[command(visible_alias = "call")]
    Cast {
        device: String,
        command: String,
        /// Command parameter as name=value; repeatable.
        #[arg(short, long = "param", value_parser = parse_param)]
        params: Vec<(String, String)>,
        /// Confirm a caution or danger command on real hardware.
        #[arg(long)]
        confirm: bool,
    },
    /// Run a test cycle and record its telemetry.
    #[command(visible_alias = "cycle")]
    Alchemy {
        device: String,
        #[arg(long, default_value = "cotton40")]
        program: String,
    },
    /// Hunt the fault behind a misbehaving device.
    #[command(visible_alias = "diag")]
    Goblin {
        device: String,
        /// Diagnose a recorded JSONL log instead of the device itself.
        #[arg(long)]
        log: Option<PathBuf>,
    },
    /// Decode the error codes a device shows.
    #[command(visible_alias = "codes")]
    Runes {
        device: String,
        code: Option<String>,
    },
    /// Inject a fault into a simulated device, or lift its curses.
    #[command(visible_alias = "fault")]
    Curse {
        device: String,
        fault: Option<String>,
        #[arg(long, conflicts_with = "fault")]
        clear: bool,
    },
    /// Show the latest recorded telemetry.
    #[command(visible_alias = "log")]
    Chronicle {
        device: String,
        /// How many of the last samples to show.
        #[arg(long, default_value_t = 8)]
        tail: usize,
    },
    /// Check a manifest you wrote.
    #[command(visible_alias = "validate")]
    Inscribe { file: PathBuf },
    /// Serve the spells to AI agents over MCP (stdio).
    #[command(visible_alias = "mcp")]
    Portal,
}

fn parse_param(text: &str) -> Result<(String, String), String> {
    match text.split_once('=') {
        Some((name, value)) if !name.is_empty() => Ok((name.to_string(), value.to_string())),
        _ => Err(format!("expected name=value, got {text:?}")),
    }
}

/// Accept `/goblin` as well as `goblin`: strip the slash from the first
/// argument that names a spell.
fn normalize(arguments: Vec<String>) -> Vec<String> {
    let command = Cli::command();
    let is_spell = |name: &str| {
        command.get_subcommands().any(|spell| {
            spell.get_name() == name || spell.get_all_aliases().any(|alias| alias == name)
        })
    };
    let mut arguments = arguments;
    if let Some(index) = arguments
        .iter()
        .skip(1)
        .position(|arg| arg.strip_prefix('/').is_some_and(is_spell))
    {
        arguments[index + 1].remove(0);
    }
    arguments
}

fn main() -> ExitCode {
    // Exit quietly when piped into `head` instead of panicking on EPIPE.
    #[cfg(unix)]
    // SAFETY: restoring the default SIGPIPE disposition before any threads start.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let cli = Cli::parse_from(normalize(std::env::args().collect()));
    let home = cli.home.clone().unwrap_or_else(Store::default_root);
    let book = Spellbook::open(home, cli.seed);
    match cli.spell {
        None => interactive(&book),
        Some(Spell::Portal) => match mcp::serve(&book, io::stdin().lock(), io::stdout().lock()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("portal closed: {error:#}");
                ExitCode::FAILURE
            }
        },
        Some(spell) => match cast(&book, spell) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!(
                    "{}",
                    panel("⚠️ SPELL FAILED", &format!("{error:#}"), palette::DANGER)
                );
                ExitCode::FAILURE
            }
        },
    }
}

fn interactive(book: &Spellbook) -> ExitCode {
    ui::intro_animation();
    help();
    let stdin = io::stdin();
    loop {
        print!("\n{} ", paint("spellbook>", palette::ACCENT));
        let _ = io::stdout().flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                println!();
                farewell();
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("could not read input: {error}");
                return ExitCode::FAILURE;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(words) = shlex::split(line) else {
            println!(
                "{}",
                paint("Invalid command: unbalanced quotes.", palette::DANGER)
            );
            continue;
        };
        let arguments = normalize(
            std::iter::once("spellbook".to_string())
                .chain(words)
                .collect(),
        );
        // Errors stay inside the session; only /exit ends it.
        match Cli::try_parse_from(arguments) {
            Err(error) => {
                let _ = error.print();
            }
            Ok(Cli { spell: None, .. }) => {}
            Ok(Cli {
                spell: Some(Spell::Exit),
                ..
            }) => {
                farewell();
                return ExitCode::SUCCESS;
            }
            Ok(Cli {
                spell: Some(Spell::Portal),
                ..
            }) => println!(
                "{}",
                panel(
                    "🌀 PORTAL",
                    "The portal speaks MCP on stdin; run `spellbook /portal` on its own.",
                    palette::GOLD
                )
            ),
            Ok(Cli {
                spell: Some(spell), ..
            }) => {
                if let Err(error) = cast(book, spell) {
                    println!(
                        "{}",
                        panel("⚠️ SPELL FAILED", &format!("{error:#}"), palette::DANGER)
                    );
                }
            }
        }
    }
}

fn cast(book: &Spellbook, spell: Spell) -> Result<()> {
    match spell {
        Spell::Help => {
            ui::intro_animation();
            help();
        }
        Spell::Exit => farewell(),
        Spell::Map => map(book),
        Spell::Scroll { device } => scroll(book.manifest(&device)?),
        Spell::Scry { device, sensor, at } => scry(book, &device, sensor.as_deref(), at)?,
        Spell::Cast {
            device,
            command,
            params,
            confirm,
        } => {
            let params: BTreeMap<String, String> = params.into_iter().collect();
            let actor = Actor::Human { confirmed: confirm };
            let report =
                ui::magic_wand("Casting", || book.call(&device, &command, &params, actor))?;
            println!("{}", panel("⚡ CAST", &report.message, palette::SUCCESS));
            if report.simulated && report.hazard != Hazard::Safe {
                println!(
                    "{}",
                    paint(
                        &format!(
                            "🔮 Simulated. On real hardware this {} command needs --confirm.",
                            report.hazard.label()
                        ),
                        palette::MUTED
                    )
                );
            }
            if let Some(path) = &report.log {
                println!(
                    "Telemetry recorded to {}",
                    paint(&path.display().to_string(), palette::GOLD)
                );
            }
        }
        Spell::Alchemy { device, program } => {
            let params = BTreeMap::from([("program".to_string(), program)]);
            let actor = Actor::Human { confirmed: false };
            let report = ui::magic_wand("Brewing a test cycle", || {
                book.call(&device, "start_cycle", &params, actor)
            })?;
            let style = if report.records.iter().any(|r| r.error.is_some()) {
                palette::DANGER
            } else {
                palette::SUCCESS
            };
            println!("{}", panel("🧪 ALCHEMY", &report.message, style));
            println!(
                "{}",
                phase_table(book.manifest(&device)?, &phase_stats(&report.records))
            );
            if let Some(path) = &report.log {
                println!(
                    "Telemetry recorded to {}",
                    paint(&path.display().to_string(), palette::GOLD)
                );
            }
            println!(
                "{}",
                paint(&format!("Next: spellbook /goblin {device}"), palette::MUTED)
            );
        }
        Spell::Goblin { device, log } => goblin(book, &device, log.as_deref())?,
        Spell::Runes { device, code } => runes(book, &device, code.as_deref())?,
        Spell::Curse {
            device,
            fault,
            clear,
        } => curse(book, &device, fault.as_deref(), clear)?,
        Spell::Chronicle { device, tail } => chronicle(book, &device, tail)?,
        Spell::Inscribe { file } => inscribe(&file)?,
        Spell::Portal => unreachable!("the portal is served from main"),
    }
    Ok(())
}

const GRIMOIRE: &[(&str, &str, &str)] = &[
    (
        "📖 /spellbook",
        "help",
        "Open the spellbook and show every spell.",
    ),
    (
        "📍 /map",
        "scan",
        "Discover the devices Spellbook can reach.",
    ),
    ("📜 /scroll", "describe", "Read a device's manifest."),
    ("🔮 /scry", "read", "Read sensor values."),
    (
        "⚡ /cast",
        "call",
        "Invoke a device command, behind the safety wards.",
    ),
    (
        "🧪 /alchemy",
        "cycle",
        "Run a test cycle and record its telemetry.",
    ),
    (
        "👹 /goblin",
        "diag",
        "Hunt the fault behind a misbehaving device.",
    ),
    (
        "🧿 /runes",
        "codes",
        "Decode the error codes a device shows.",
    ),
    (
        "💀 /curse",
        "fault",
        "Inject a fault into a simulated device.",
    ),
    (
        "📓 /chronicle",
        "log",
        "Show the latest recorded telemetry.",
    ),
    ("🔏 /inscribe", "validate", "Check a manifest you wrote."),
    (
        "🌀 /portal",
        "mcp",
        "Let AI agents cast these spells over MCP.",
    ),
    ("👋 /exit", "quit, bye", "Close the spellbook."),
];

fn help() {
    println!("{}", ui::banner());
    let rows: Vec<Vec<String>> = GRIMOIRE
        .iter()
        .map(|(spell, alias, meaning)| {
            vec![spell.to_string(), alias.to_string(), meaning.to_string()]
        })
        .collect();
    println!(
        "{}",
        table(
            "Command Grimoire",
            &["Spell", "Plain Alias", "Meaning"],
            &rows,
            &[palette::ACCENT, palette::GOLD, palette::LABEL]
        )
    );
    println!(
        "{}",
        panel(
            "💡 TIP",
            "Try: /curse washer heater_dead, then /goblin washer",
            palette::MUTED
        )
    );
}

fn farewell() {
    println!(
        "{}",
        panel("👋 EXIT", "Closing the spellbook.", palette::SUCCESS)
    );
}

fn map(book: &Spellbook) {
    let rows: Vec<Vec<String>> = book
        .devices()
        .into_iter()
        .map(|device| {
            let status = match device.availability {
                Availability::Simulated => "🔮 simulated",
                Availability::Planned => "⏳ adapter planned",
                Availability::Unknown => "⚠️ unknown transport",
            };
            vec![device.id, device.name, device.transport, status.to_string()]
        })
        .collect();
    println!(
        "{}",
        panel(
            "📍 MAP",
            "Devices within reach of the spellbook.",
            palette::ACCENT
        )
    );
    println!(
        "{}",
        table(
            "",
            &["Device", "Name", "Transport", "Status"],
            &rows,
            &[
                palette::ACCENT,
                palette::LABEL,
                palette::GOLD,
                palette::MUTED
            ]
        )
    );
    for warning in book.warnings() {
        println!("{}", paint(&format!("⚠️ {warning}"), palette::DANGER));
    }
    println!(
        "{}",
        paint(
            &format!(
                "Add your own devices as TOML manifests in {}",
                book.store().manifests_dir().display()
            ),
            palette::MUTED
        )
    );
}

fn scroll(manifest: &Manifest) {
    let device = &manifest.device;
    let mut summary = vec![device.name.clone(), format!("kind: {}", device.kind)];
    if let Some(model) = &device.model {
        summary.push(
            format!("model: {} {model}", device.vendor.as_deref().unwrap_or("")).replace("  ", " "),
        );
    }
    summary.push(format!(
        "transport: {}{}",
        manifest.transport.kind,
        manifest
            .transport
            .target
            .as_deref()
            .map(|t| format!(" ({t})"))
            .unwrap_or_default()
    ));
    println!(
        "{}",
        panel(
            &format!("📜 {}", device.id),
            &summary.join("\n"),
            palette::ACCENT
        )
    );

    let sensors: Vec<Vec<String>> = manifest
        .sensors
        .iter()
        .map(|sensor| {
            vec![
                sensor.id.clone(),
                sensor.unit.clone(),
                range(sensor.min, sensor.max),
                sensor.description.clone(),
            ]
        })
        .collect();
    println!(
        "{}",
        table(
            "Sensors",
            &["Sensor", "Unit", "Rated", "Description"],
            &sensors,
            &[palette::ACCENT, palette::GOLD]
        )
    );

    let commands: Vec<Vec<String>> = manifest
        .commands
        .iter()
        .map(|command| {
            let params: Vec<String> = command
                .params
                .iter()
                .map(|param| match (&param.default, param.choices.is_empty()) {
                    (_, false) => format!("{}={}", param.name, param.choices.join("|")),
                    (Some(default), true) => format!("{}={default}", param.name),
                    (None, true) => format!("{}=…", param.name),
                })
                .collect();
            vec![
                command.id.clone(),
                command.hazard.label().to_string(),
                params.join(" "),
                command.description.clone(),
            ]
        })
        .collect();
    println!(
        "{}",
        table(
            "Commands",
            &["Command", "Hazard", "Params", "Description"],
            &commands,
            &[palette::ACCENT, palette::GOLD]
        )
    );

    let codes: Vec<Vec<String>> = manifest
        .error_codes
        .iter()
        .map(|entry| vec![entry.code.clone(), entry.meaning.clone()])
        .collect();
    if !codes.is_empty() {
        println!(
            "{}",
            table(
                "Error Codes",
                &["Code", "Meaning"],
                &codes,
                &[palette::DANGER]
            )
        );
    }

    let faults: Vec<Vec<String>> = manifest
        .faults
        .iter()
        .map(|fault| {
            vec![
                fault.id.clone(),
                fault.name.clone(),
                fault.hazard.label().to_string(),
            ]
        })
        .collect();
    if !faults.is_empty() {
        println!(
            "{}",
            table(
                "Known Faults",
                &["Fault", "Name", "Repair hazard"],
                &faults,
                &[palette::GOLD]
            )
        );
    }
}

fn range(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{min}–{max}"),
        (Some(min), None) => format!("≥ {min}"),
        (None, Some(max)) => format!("≤ {max}"),
        (None, None) => "—".to_string(),
    }
}

fn number(value: f64) -> String {
    if value.abs() >= 100.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn scry(book: &Spellbook, device: &str, sensor: Option<&str>, at: f64) -> Result<()> {
    let manifest = book.manifest(device)?;
    if let Some(sensor) = sensor
        && manifest.sensor(sensor).is_none()
    {
        let known: Vec<&str> = manifest.sensors.iter().map(|s| s.id.as_str()).collect();
        return Err(anyhow!(
            "{device} has no sensor {sensor:?}; it has {}",
            known.join(", ")
        ));
    }
    let record = book.read(device, at)?;
    let mut headline = format!("{device} at {:.0} s", record.t);
    if let Some(phase) = &record.phase {
        headline.push_str(&format!(" · phase {phase}"));
    }
    if let Some(code) = &record.error {
        headline.push_str(&format!(" · showing {code}"));
    }
    println!("{}", panel("🔮 SCRY", &headline, palette::ACCENT));
    let rows: Vec<Vec<String>> = manifest
        .sensors
        .iter()
        .filter(|spec| sensor.is_none_or(|wanted| wanted == spec.id))
        .filter_map(|spec| {
            let value = *record.readings.get(&spec.id)?;
            let rated =
                spec.min.is_none_or(|min| value >= min) && spec.max.is_none_or(|max| value <= max);
            Some(vec![
                spec.id.clone(),
                format!("{} {}", number(value), spec.unit),
                range(spec.min, spec.max),
                if rated {
                    "✓".into()
                } else {
                    "⚠ out of range".into()
                },
            ])
        })
        .collect();
    println!(
        "{}",
        table(
            "",
            &["Sensor", "Value", "Rated", "Status"],
            &rows,
            &[palette::ACCENT, palette::GOLD, palette::MUTED]
        )
    );
    Ok(())
}

fn phase_table(manifest: &Manifest, phases: &[PhaseStats]) -> String {
    let mut headers = vec![
        "Phase".to_string(),
        "Start".to_string(),
        "Duration".to_string(),
    ];
    headers.extend(
        manifest
            .sensors
            .iter()
            .map(|s| format!("{} ({})", s.id, s.unit)),
    );
    let rows: Vec<Vec<String>> =
        phases
            .iter()
            .map(|stats| {
                let mut row = vec![
                    stats.phase.clone(),
                    format!("{:.0} s", stats.start_s),
                    format!("{:.0} s", stats.duration_s),
                ];
                row.extend(manifest.sensors.iter().map(
                    |sensor| match stats.sensors.get(&sensor.id) {
                        Some(seen) => format!("{} ↑{}", number(seen.mean), number(seen.max)),
                        None => "—".to_string(),
                    },
                ));
                row
            })
            .collect();
    let headers: Vec<&str> = headers.iter().map(String::as_str).collect();
    table(
        "Phases (mean ↑peak)",
        &headers,
        &rows,
        &[palette::ACCENT, palette::MUTED, palette::MUTED],
    )
}

fn goblin(book: &Spellbook, device: &str, log: Option<&Path>) -> Result<()> {
    let report = ui::magic_wand("Hunting goblins", || book.diagnose(device, log))?;
    let diagnosis = &report.diagnosis;
    println!(
        "{}",
        panel(
            "👹 GOBLIN HUNT",
            &format!(
                "{device}: {} samples\nfrom {}",
                diagnosis.samples, report.source
            ),
            palette::GOLD
        )
    );
    if diagnosis.healthy() {
        println!(
            "{}",
            panel(
                "✨ ALL CLEAR",
                "Every phase behaved as its manifest expects. No goblins here.",
                palette::SUCCESS
            )
        );
        return Ok(());
    }

    let evidence: Vec<Vec<String>> = diagnosis
        .findings
        .iter()
        .map(|finding| vec![finding.symptom.clone(), finding.detail.clone()])
        .collect();
    println!(
        "{}",
        table(
            "Evidence",
            &["Symptom", "Detail"],
            &evidence,
            &[palette::DANGER]
        )
    );

    if diagnosis.suspects.is_empty() {
        println!(
            "{}",
            paint(
                "No known fault explains this evidence. Once you find the cause, add it to the manifest's faults.",
                palette::MUTED
            )
        );
        return Ok(());
    }
    for (rank, suspect) in diagnosis.suspects.iter().take(3).enumerate() {
        let mut lines = vec![
            suspect.name.clone(),
            format!("matches: {}", suspect.matched.join(", ")),
        ];
        if !suspect.missing.is_empty() {
            lines.push(format!("not seen: {}", suspect.missing.join(", ")));
        }
        if !suspect.parts.is_empty() {
            lines.push(format!("parts: {}", suspect.parts.join(", ")));
        }
        lines.push(format!("repair hazard: {}", suspect.hazard.label()));
        // Repair steps only for the lead suspect; the rest are alternatives.
        if rank == 0 {
            for (step, text) in suspect.repair.iter().enumerate() {
                lines.push(format!("{}. {text}", step + 1));
            }
        }
        let label = format!("🎯 SUSPECT {} · {:.0}%", rank + 1, suspect.score * 100.0);
        let style = if rank == 0 {
            hazard_style(suspect.hazard)
        } else {
            palette::MUTED
        };
        println!("{}", panel(&label, &lines.join("\n"), style));
    }
    if diagnosis
        .suspects
        .first()
        .is_some_and(|top| top.hazard == Hazard::Danger)
    {
        println!(
            "{}",
            paint(
                "⚠️ Danger: unplug from mains before opening any panel. If unsure, call a technician.",
                palette::DANGER
            )
        );
    }
    Ok(())
}

fn runes(book: &Spellbook, device: &str, code: Option<&str>) -> Result<()> {
    let Some(code) = code else {
        let manifest = book.manifest(device)?;
        let rows: Vec<Vec<String>> = manifest
            .error_codes
            .iter()
            .map(|entry| vec![entry.code.clone(), entry.meaning.clone()])
            .collect();
        println!(
            "{}",
            table(
                &format!("🧿 Runes of {device}"),
                &["Code", "Meaning"],
                &rows,
                &[palette::DANGER]
            )
        );
        return Ok(());
    };
    let report = book.lookup_code(device, code)?;
    let mut lines = vec![report.code.meaning.clone()];
    if !report.code.causes.is_empty() {
        lines.push(String::new());
        lines.push("Usual causes:".to_string());
        lines.extend(report.code.causes.iter().map(|cause| format!("• {cause}")));
    }
    if !report.faults.is_empty() {
        lines.push(String::new());
        lines.push("Known faults:".to_string());
        lines.extend(
            report
                .faults
                .iter()
                .map(|fault| format!("• {} ({})", fault.name, fault.id)),
        );
    }
    println!(
        "{}",
        panel(
            &format!("🧿 {}", report.code.code),
            &lines.join("\n"),
            palette::GOLD
        )
    );
    println!(
        "{}",
        paint(
            &format!("Gather evidence with: spellbook /goblin {device}"),
            palette::MUTED
        )
    );
    Ok(())
}

fn curse(book: &Spellbook, device: &str, fault: Option<&str>, clear: bool) -> Result<()> {
    if clear {
        book.lift_curses(device)?;
        println!(
            "{}",
            panel(
                "✨ CURSES LIFTED",
                &format!("{device} is healthy again."),
                palette::SUCCESS
            )
        );
        return Ok(());
    }
    if let Some(fault) = fault {
        let active = book.curse(device, fault)?;
        let active: Vec<&str> = active.iter().map(String::as_str).collect();
        println!(
            "{}",
            panel(
                "💀 CURSED",
                &format!(
                    "{device} now suffers: {}\nHunt it with: spellbook /goblin {device}",
                    active.join(", ")
                ),
                palette::DANGER
            )
        );
        return Ok(());
    }
    let available = book.available_curses(device)?;
    let active = book.curses(device)?;
    let rows: Vec<Vec<String>> = available
        .iter()
        .map(|(id, description)| {
            let mark = if active.contains(*id) { "💀" } else { "" };
            vec![mark.to_string(), id.to_string(), description.to_string()]
        })
        .collect();
    println!(
        "{}",
        table(
            &format!("💀 Curses for {device}"),
            &["", "Fault", "Effect"],
            &rows,
            &["", palette::GOLD]
        )
    );
    println!(
        "{}",
        paint(
            &format!("Cast one with: spellbook /curse {device} <fault>   Lift all with: --clear"),
            palette::MUTED
        )
    );
    Ok(())
}

fn chronicle(book: &Spellbook, device: &str, tail: usize) -> Result<()> {
    let manifest = book.manifest(device)?;
    let Some((path, records)) = book.latest_log(device)? else {
        println!(
            "{}",
            panel(
                "📓 CHRONICLE",
                &format!("No telemetry yet. Run: spellbook /alchemy {device}"),
                palette::MUTED
            )
        );
        return Ok(());
    };
    println!(
        "{}",
        panel(
            "📓 CHRONICLE",
            &format!("{} samples in {}", records.len(), path.display()),
            palette::ACCENT
        )
    );
    println!("{}", phase_table(manifest, &phase_stats(&records)));
    println!(
        "{}",
        sample_table(manifest, &records[records.len().saturating_sub(tail)..])
    );
    Ok(())
}

fn sample_table(manifest: &Manifest, records: &[LogRecord]) -> String {
    let mut headers = vec!["t", "Phase"];
    headers.extend(manifest.sensors.iter().map(|s| s.id.as_str()));
    headers.push("Error");
    let rows: Vec<Vec<String>> = records
        .iter()
        .map(|record| {
            let mut row = vec![
                format!("{:.0}", record.t),
                record.phase.clone().unwrap_or_default(),
            ];
            row.extend(manifest.sensors.iter().map(|sensor| {
                record
                    .readings
                    .get(&sensor.id)
                    .map_or("—".to_string(), |v| number(*v))
            }));
            row.push(record.error.clone().unwrap_or_default());
            row
        })
        .collect();
    let mut styles = vec![palette::MUTED, palette::ACCENT];
    styles.extend(manifest.sensors.iter().map(|_| ""));
    styles.push(palette::DANGER);
    table("Latest samples", &headers, &rows, &styles)
}

fn inscribe(file: &Path) -> Result<()> {
    let source = std::fs::read_to_string(file)
        .map_err(|error| anyhow!("could not read {}: {error}", file.display()))?;
    let manifest = Manifest::from_toml(&source)?;
    println!(
        "{}",
        panel(
            "🔏 SEALED",
            &format!(
                "{} is a valid {} manifest\n{} sensors · {} commands · {} phases · {} error codes · {} faults",
                manifest.device.id,
                manifest.schema,
                manifest.sensors.len(),
                manifest.commands.len(),
                manifest.phases.len(),
                manifest.error_codes.len(),
                manifest.faults.len()
            ),
            palette::SUCCESS
        )
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn slashes_are_stripped_only_from_spell_names() {
        assert_eq!(
            normalize(args("spellbook /goblin washer")),
            args("spellbook goblin washer")
        );
        assert_eq!(
            normalize(args("spellbook /diag washer")),
            args("spellbook diag washer")
        );
        assert_eq!(
            normalize(args("spellbook --home /tmp/x /scry washer")),
            args("spellbook --home /tmp/x scry washer")
        );
        assert_eq!(
            normalize(args("spellbook /tmp/file")),
            args("spellbook /tmp/file")
        );
    }

    #[test]
    fn plain_aliases_reach_the_same_spell() {
        let parsed = Cli::try_parse_from(args("spellbook diag washer")).unwrap();
        assert!(matches!(parsed.spell, Some(Spell::Goblin { .. })));
        let parsed = Cli::try_parse_from(args("spellbook call washer stop -p a=b")).unwrap();
        let Some(Spell::Cast { params, .. }) = parsed.spell else {
            panic!("not a cast")
        };
        assert_eq!(params, [("a".to_string(), "b".to_string())]);
    }

    #[test]
    fn the_cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }
}
