# Spellbook

A retro fantasy terminal grimoire that lets humans **and AI agents** talk to,
diagnose, and repair hardware: appliances, robot parts, sensors, actuators.

Robots and appliances already communicate, but over dozens of incompatible
protocols (UART, I2C, SPI, CAN, Modbus, BLE, Zigbee/Matter, USB, IR, and
proprietary ones), and each device's "manual" is a PDF datasheet. Spellbook
gives them:

1. **A machine-readable self-description.** Every device has a manifest: a
   datasheet as code.
2. **One uniform command interface** over every transport.
3. **An AI-native shell** that agents can use to diagnose and repair, with
   safety wards built in.

```text
$ spellbook /curse washer heater_dead      # break the simulated washer
$ spellbook /goblin washer                 # ...and hunt the fault

👹 GOBLIN HUNT: washer: 193 samples
Evidence
  timeout:heat               heat ran 1800 s and hit its 1800 s timeout
  low:power_w@heat           power_w averaged 12.0 W during heat; expected at least 1500 W
  no_rise:water_temp_c@heat  water_temp_c rose only 0.2 °C during heat; expected at least 15 °C
  code:HE                    HE at 1920 s: Heating error: water did not reach temperature

🎯 SUSPECT 1 · 100%: Heating element open circuit
   parts: Heating element, Heater relay
   1. Unplug the machine from mains and wait 5 minutes.
   2. Remove the rear panel and locate the heating element below the drum.
   3. Disconnect the element and measure across its terminals: roughly 20–30 Ω is healthy.
   ...
🎯 SUSPECT 2 · 40%: Thermistor stuck or open
```

## Architecture

| Layer | What it does | Status |
|---|---|---|
| 1. Device manifest | TOML per device: identity, sensors, commands with hazard levels, expected behaviour per phase, error codes, known faults, repair steps | ✅ schema v0.1 |
| 2. Transport adapters | One `Transport` trait over every bus | ✅ simulator · ⏳ serial, MQTT, Modbus, CAN, BLE, I2C |
| 3. Terminal shell | `spellbook` CLI + interactive session, fantasy spells with plain aliases | ✅ |
| 4. Agent bridge | MCP server over stdio, so Claude or any LLM can discover and diagnose devices | ✅ |
| 5. Repair knowledge | Diagnosis engine that compares telemetry with the manifest and ranks known faults | ✅ rule-based |
| 6. Safety & permissions | Reads always allowed; simulation before actuation; agents limited to `safe` commands on real hardware | ✅ |

Spellbook builds on existing standards rather than replacing them: ROS 2 for
robot messaging, Matter for smart-home device models, OPC UA for industrial
semantics, CANopen/Modbus for actuators, and iFixit for repair procedures.
What it adds is the manifest, the AI-native shell, and the repair layer.

## Spells

| Spell | Plain alias | What it does |
|---|---|---|
| 📖 `/spellbook` | `help` | Open the spellbook and show every spell |
| 📍 `/map` | `scan` | Discover the devices Spellbook can reach |
| 📜 `/scroll <device>` | `describe` | Read a device's manifest |
| 🔮 `/scry <device> [sensor] [--at s]` | `read` | Read sensor values |
| ⚡ `/cast <device> <command> [-p k=v] [--confirm]` | `call` | Invoke a command, behind the safety wards |
| 🧪 `/alchemy <device> [--program p]` | `cycle` | Run a test cycle and record its telemetry |
| 👹 `/goblin <device> [--log file]` | `diag` | Hunt the fault behind a misbehaving device |
| 🧿 `/runes <device> [code]` | `codes` | Decode the error codes a device shows |
| 💀 `/curse <device> [fault] [--clear]` | `fault` | Inject a fault into a simulated device |
| 📓 `/chronicle <device> [--tail n]` | `log` | Show the latest recorded telemetry |
| 🔏 `/inscribe <file>` | `validate` | Check a manifest you wrote |
| 🌀 `/portal` | `mcp` | Serve the spells to AI agents over MCP |
| 👋 `/exit` | `quit`, `bye` | Close the spellbook |

The leading slash is optional: `spellbook /goblin washer`, `spellbook goblin washer`,
and `spellbook diag washer` are the same spell.

## Quick start

Install Rust from <https://rustup.rs>, then:

```bash
cargo build --release
./target/release/spellbook                 # interactive session; /exit or Ctrl+D to close
./target/release/spellbook /map
./target/release/spellbook /scroll washer
```

Or install it onto your `PATH` with `cargo install --path .`.

### Bash alias

```bash
./scripts/install-bash-alias
source ~/.bashrc
spellbook /runes washer OE
```

The alias points to `scripts/spellbook`, which rebuilds when the sources
change and then runs the release binary. If you move the repository,
reinstall the alias.

## A diagnosis walkthrough (no hardware needed)

The built-in `washer` is a simulated front-loader. It produces realistic power,
temperature, vibration, and water-level curves. A *curse* makes it fail the way
a real machine fails.

```bash
spellbook /curse washer                    # list the faults you can inject
spellbook /curse washer drain_pump_failed
spellbook /alchemy washer                  # run a cycle; it stops with OE
spellbook /runes washer OE                 # what the display code means
spellbook /goblin washer                   # evidence + ranked suspects + repair steps
spellbook /cast washer test_component -p component=pump   # service-mode test
spellbook /chronicle washer                # the recorded telemetry
spellbook /curse washer --clear
```

The faults you can inject are `door_lock_failed`, `inlet_valve_failed`, `heater_dead`,
`thermistor_failed`, `bearing_worn`, `drain_pump_failed`,
`drain_filter_clogged`, and `unbalanced_load`.

## Getting real data

Diagnosis is only as good as the data. In order of effort:

1. **Error codes on the display.** Every modern machine has them. `/runes`
   maps a code to its meaning, causes, and faults.
2. **Service/diagnostic mode.** Most machines can run their motor, pump,
   valve, and heater one at a time. The manifest models this as `test_component`.
3. **The vendor app or cloud** (SmartThings, ThinQ). Some of them have APIs.
4. **External sensors, which work on any appliance:** a Tasmota/ESPHome
   energy-monitoring plug for `power_w`, and an ESP32 with an accelerometer
   for `vibration_g`. Whatever the manufacturer does or doesn't expose, you
   build the log yourself.

Any of these can feed `spellbook /goblin <device> --log <file.jsonl>`. The log
format is JSON Lines, one sample per line:

```json
{"t":0,"device":"washer","phase":"fill","readings":{"power_w":10.2,"water_level_pct":5.0}}
{"t":300,"device":"washer","phase":"drain","readings":{"power_w":1.5},"error":"OE"}
```

`phase` and `error` are optional. Without phases, Spellbook still checks
every reading against its rated range and decodes error codes.

## Writing a manifest

Put your own devices in `~/.spellbook/manifests/*.toml`. A manifest with the
same `id` as a built-in one replaces it. See
[`manifests/washer.toml`](manifests/washer.toml) for a complete example. The
short version:

```toml
schema = "spellbook.manifest/0.1"

[device]
id = "kettle"
name = "Kitchen kettle"
kind = "appliance.kettle"

[transport]
kind = "mqtt"                       # sim | serial | mqtt | modbus | can | ble | i2c
target = "tele/kettle-plug/SENSOR"

[[sensors]]
id = "power_w"
unit = "W"
min = 0.0
max = 3200.0

[[commands]]
id = "power_off"
hazard = "safe"                     # safe | caution | danger

[[phases]]
id = "boil"
timeout_s = 600
expect = [{ sensor = "power_w", mean_min = 1800.0 }]

[[error_codes]]
code = "E1"
meaning = "Boil-dry protection tripped"
faults = ["element_dead"]

[[faults]]
id = "element_dead"
name = "Heating element open"
symptoms = ["low:power_w@boil", "timeout:boil", "code:E1"]
hazard = "danger"
repair = ["Unplug the kettle.", "Measure the element resistance."]
```

Symptoms describe the evidence a fault leaves behind:

| Symptom | Meaning |
|---|---|
| `low:<sensor>@<phase>` / `high:…` | Phase mean below `mean_min` / above `mean_max` |
| `no_rise:<sensor>@<phase>` | Rose less than `rise_min` during the phase |
| `peak:<sensor>@<phase>` | Peaked above `peak_max` |
| `timeout:<phase>` | The phase ran until `timeout_s` |
| `code:<CODE>` | The device reported this error code |
| `range:<sensor>` | A reading left the sensor's rated `min`/`max` |

Each known fault is scored by how closely its symptoms match what was observed
(Jaccard similarity), so evidence that is missing *or* unexplained lowers its
score. Check your file with `spellbook /inscribe kettle.toml`.

## Connecting an AI agent

`spellbook /portal` speaks the Model Context Protocol over stdio. To add it
to Claude Code:

```bash
claude mcp add spellbook -- /home/shivthedev/Documents/projects/project-codex/scripts/spellbook /portal
```

Tools: `list_devices`, `describe_device`, `read_sensors`, `lookup_error_code`,
`diagnose`, `call_command`. Then ask: *"The washer shows OE. What's wrong and
how do I fix it?"*

## Safety wards

- **Reads are always allowed.**
- **Simulators allow everything.** Simulate before you actuate.
- **On real hardware**, every command carries a hazard level in its manifest:
  - `safe`: anyone, including agents.
  - `caution` / `danger`: only a human at the terminal, with `--confirm`.
    AI agents are refused.
- Repair steps for `danger` faults start with isolating mains power, and
  Spellbook tells you to call a technician if you're unsure.

## Project layout

```text
manifests/washer.toml   built-in manifest (embedded in the binary)
src/
  main.rs               CLI, interactive session, rendering
  book.rs               operations shared by the CLI and MCP
  manifest.rs           manifest schema, symptoms, validation
  registry.rs           built-in + user manifests
  transport.rs          Transport trait, adapters
  sim.rs                simulated washing machine with injectable faults
  telemetry.rs          JSONL log format
  diag.rs               diagnosis engine
  safety.rs             hazard levels and authorization
  store.rs              ~/.spellbook state, logs
  mcp.rs                MCP server
  ui.rs, art.rs         palette, panels, tables, wand and intro animations
tests/cli.rs            end-to-end tests of the binary
```

State lives in `~/.spellbook`. Override it with `SPELLBOOK_HOME` or `--home`.

## Roadmap

- **Adapters:** serial (ESP32 boards), MQTT (Tasmota/ESPHome plugs), Modbus
  (industrial drives).
- **Manifests for cheap real hardware:** an energy-monitoring smart plug, an
  ESP32 accelerometer board, a servo/stepper driver.
- **A real bench:** a smart plug plus an ESP32 accelerometer on an actual
  washing machine.
- **An error-code library** for common Indian-market models (LG, Samsung,
  IFB, Whirlpool).
- **Phase detection from raw power curves,** so unlabelled smart-plug logs get
  the full per-phase diagnosis.
- **Robot-to-robot:** devices discover and query each other through the same
  manifests.

## Contributing

```bash
cargo test
cargo clippy --all-targets
cargo fmt
```

Good first contributions:

- A manifest for a device you own.
- A new simulator model, such as a fridge compressor or a robot arm joint.
- A transport adapter.
- More fault signatures for the washer.

Keep the fantasy theme playful, but never let it make a command's behaviour
unclear. Keep changes small, tested, and easy to understand.
