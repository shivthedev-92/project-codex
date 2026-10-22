// ███████╗██████╗ ███████╗██╗     ██╗     ██████╗  ██████╗  ██████╗ ██╗  ██╗
// ██╔════╝██╔══██╗██╔════╝██║     ██║     ██╔══██╗██╔═══██╗██╔═══██╗██║ ██╔╝
// ███████╗██████╔╝█████╗  ██║     ██║     ██████╔╝██║   ██║██║   ██║█████╔╝
// ╚════██║██╔═══╝ ██╔══╝  ██║     ██║     ██╔══██╗██║   ██║██║   ██║██╔═██╗
// ███████║██║     ███████╗███████╗███████╗██████╔╝╚██████╔╝╚██████╔╝██║  ██╗
// ╚══════╝╚═╝     ╚══════╝╚══════╝╚══════╝╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝

//! Spellbook: a universal terminal interface between AI agents and hardware.
//!
//! The layers, bottom up:
//!
//! - [`manifest`]: every device describes itself as code (sensors, commands,
//!   expected behaviour, error codes, faults, and repair steps).
//! - [`transport`]: one interface over every bus. The simulator ships today;
//!   serial, MQTT, and Modbus adapters plug into the same trait.
//! - [`telemetry`]: a JSON Lines log format any data source can produce.
//! - [`diag`]: compares telemetry against the manifest and ranks known faults.
//! - [`safety`]: decides who may actuate what, and where.
//! - [`book`]: the operations shared by the CLI and the MCP server.
//! - [`mcp`]: exposes those operations to AI agents as MCP tools.

pub mod art;
pub mod book;
pub mod diag;
pub mod manifest;
pub mod mcp;
pub mod registry;
pub mod safety;
pub mod sim;
pub mod store;
pub mod telemetry;
pub mod transport;
pub mod ui;
