//! The portal: Spellbook as an MCP server over stdio.
//!
//! Any MCP client (Claude Code, Claude Desktop, or another robot's agent) can
//! discover devices, read them, look up codes, diagnose faults, and call
//! commands. Agents act as [`Actor::Agent`], so on real hardware the safety
//! wards only let them run `safe` commands.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};

use crate::book::Spellbook;
use crate::safety::Actor;

const PROTOCOL_VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

/// Serve newline-delimited JSON-RPC until `input` closes.
pub fn serve(book: &Spellbook, input: impl BufRead, mut output: impl Write) -> Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(book, &message),
            Err(error) => Some(error_response(
                Value::Null,
                -32700,
                &format!("parse error: {error}"),
            )),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut output, &response)?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }
    Ok(())
}

fn handle(book: &Spellbook, message: &Value) -> Option<Value> {
    // Notifications (no id) and responses get no reply.
    let id = message.get("id")?.clone();
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return Some(error_response(
            id,
            -32600,
            "invalid request: missing method",
        ));
    };
    let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
    let result = match method {
        "initialize" => Ok(initialize(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => call_tool(book, &params),
        other => Err((-32601, format!("method not found: {other}"))),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, text)) => error_response(id, code, &text),
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = requested
        .filter(|version| PROTOCOL_VERSIONS.contains(version))
        .unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "spellbook", "version": env!("CARGO_PKG_VERSION") },
        "instructions": "Spellbook connects you to hardware. Start with list_devices, read a device's \
            manifest with describe_device, then diagnose it. Actuating real hardware beyond safe \
            commands requires a human at the terminal.",
    })
}

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": properties, "required": required },
    })
}

fn tools() -> Vec<Value> {
    let device = json!({ "type": "string", "description": "Device id from list_devices" });
    vec![
        tool(
            "list_devices",
            "List every device Spellbook knows and whether it is reachable.",
            json!({}),
            &[],
        ),
        tool(
            "describe_device",
            "Return a device's manifest: sensors, commands with hazard levels, expected behaviour, error codes, known faults and repair steps.",
            json!({ "device": device }),
            &["device"],
        ),
        tool(
            "read_sensors",
            "Read every sensor on a device.",
            json!({
                "device": device,
                "at_s": { "type": "number", "description": "Simulated devices only: seconds into a standard cycle" },
            }),
            &["device"],
        ),
        tool(
            "lookup_error_code",
            "Explain an error code shown on a device's display and the faults that cause it.",
            json!({ "device": device, "code": { "type": "string" } }),
            &["device", "code"],
        ),
        tool(
            "diagnose",
            "Analyse a device's telemetry against its manifest and rank the likely faults, with repair steps.",
            json!({ "device": device }),
            &["device"],
        ),
        tool(
            "call_command",
            "Invoke a device command. On real hardware, agents may only run commands whose hazard is safe.",
            json!({
                "device": device,
                "command": { "type": "string" },
                "params": { "type": "object", "additionalProperties": { "type": "string" } },
            }),
            &["device", "command"],
        ),
    ]
}

fn call_tool(book: &Spellbook, params: &Value) -> Result<Value, (i64, String)> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or((-32602, "tools/call needs a tool name".to_string()))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = match name {
        "list_devices" => Ok(json!(book.devices())),
        "describe_device" => device(&arguments).and_then(|d| Ok(json!(book.manifest(d)?))),
        "read_sensors" => device(&arguments).and_then(|d| {
            let at_s = arguments.get("at_s").and_then(Value::as_f64).unwrap_or(0.0);
            Ok(json!(book.read(d, at_s)?))
        }),
        "lookup_error_code" => device(&arguments).and_then(|d| {
            let code = string(&arguments, "code")?;
            Ok(json!(book.lookup_code(d, code)?))
        }),
        "diagnose" => device(&arguments).and_then(|d| Ok(json!(book.diagnose(d, None)?))),
        "call_command" => device(&arguments).and_then(|d| {
            let command = string(&arguments, "command")?;
            let params: BTreeMap<String, String> = match arguments.get("params") {
                Some(value) => serde_json::from_value(value.clone())
                    .context("params must map names to strings")?,
                None => BTreeMap::new(),
            };
            Ok(json!(book.call(d, command, &params, Actor::Agent)?))
        }),
        other => return Err((-32602, format!("unknown tool: {other}"))),
    };
    Ok(match outcome {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }],
            "isError": false,
        }),
        Err(error) => json!({
            "content": [{ "type": "text", "text": format!("{error:#}") }],
            "isError": true,
        }),
    })
}

fn device(arguments: &Value) -> Result<&str> {
    string(arguments, "device")
}

fn string<'a>(arguments: &'a Value, key: &str) -> Result<&'a str> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string argument {key:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exchange(requests: &[Value]) -> Vec<Value> {
        let dir = tempfile::tempdir().unwrap();
        let book = Spellbook::open(dir.path(), 42);
        let input: String = requests.iter().map(|r| format!("{r}\n")).collect();
        let mut output = Vec::new();
        serve(&book, input.as_bytes(), &mut output).unwrap();
        String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn handshake_lists_tools_and_skips_notifications() {
        let responses = exchange(&[
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "teleport"}),
        ]);
        assert_eq!(responses.len(), 3);
        assert_eq!(responses[0]["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 6);
        assert_eq!(responses[2]["error"]["code"], -32601);
    }

    #[test]
    fn tools_return_text_and_flag_errors() {
        let call = |id: i64, name: &str, arguments: Value| json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": name, "arguments": arguments}});
        let responses = exchange(&[
            call(1, "diagnose", json!({"device": "washer"})),
            call(
                2,
                "lookup_error_code",
                json!({"device": "washer", "code": "HE"}),
            ),
            call(3, "read_sensors", json!({"device": "toaster"})),
            call(4, "summon", json!({})),
        ]);
        let text = |i: usize| {
            responses[i]["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(responses[0]["result"]["isError"], false);
        assert!(text(0).contains("\"suspects\""));
        assert!(text(1).contains("heater_dead"));
        assert_eq!(responses[2]["result"]["isError"], true);
        assert!(text(2).contains("no device named"));
        assert_eq!(responses[3]["error"]["code"], -32602);
    }

    #[test]
    fn garbage_gets_a_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let book = Spellbook::open(dir.path(), 42);
        let mut output = Vec::new();
        serve(&book, "not json\n".as_bytes(), &mut output).unwrap();
        let response: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(response["error"]["code"], -32700);
    }
}
