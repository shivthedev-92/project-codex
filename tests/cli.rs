//! End-to-end: drive the real binary the way a person would.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn spellbook(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spellbook"))
        .args(args)
        .env("SPELLBOOK_HOME", home)
        .env("NO_COLOR", "1")
        .output()
        .expect("spellbook runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn curse_then_hunt_the_goblin() {
    let home = tempfile::tempdir().unwrap();

    let healthy = spellbook(home.path(), &["/goblin", "washer"]);
    assert!(healthy.status.success());
    assert!(
        stdout(&healthy).contains("ALL CLEAR"),
        "{}",
        stdout(&healthy)
    );

    assert!(
        spellbook(home.path(), &["/curse", "washer", "drain_pump_failed"])
            .status
            .success()
    );
    let hunt = spellbook(home.path(), &["diag", "washer"]);
    let text = stdout(&hunt);
    assert!(text.contains("SUSPECT 1 · 100%"), "{text}");
    assert!(text.contains("Drain pump not running"), "{text}");

    let chronicle = spellbook(home.path(), &["/chronicle", "washer"]);
    assert!(stdout(&chronicle).contains("OE"), "{}", stdout(&chronicle));
}

#[test]
fn unknown_devices_fail_with_a_helpful_message() {
    let home = tempfile::tempdir().unwrap();
    let output = spellbook(home.path(), &["/scry", "toaster"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("known devices: washer"));
}

#[test]
fn interactive_session_survives_errors_until_exit() {
    let home = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_spellbook"))
        .env("SPELLBOOK_HOME", home.path())
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"/scry toaster\n/runes washer OE\n/exit\n/map\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let text = stdout(&output);
    assert!(output.status.success());
    assert!(text.contains("SPELL FAILED"), "{text}");
    assert!(text.contains("Drain error"), "{text}");
    assert!(text.contains("Closing the spellbook"), "{text}");
    assert!(
        !text.contains("📍 MAP"),
        "commands after /exit must not run: {text}"
    );
}

#[test]
fn portal_speaks_mcp_on_stdio() {
    let home = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_spellbook"))
        .arg("/portal")
        .env("SPELLBOOK_HOME", home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["result"]["serverInfo"]["name"], "spellbook");
}
