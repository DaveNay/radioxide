# Radioxide

A cross-platform ham radio remote control application written in Rust. Radioxide communicates directly with radio hardware via serial CAT protocol — no hamlib or external libraries required.

![Rust](https://img.shields.io/badge/Rust-1.85+-orange)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)
![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)
![CI](https://github.com/your-username/radioxide/actions/workflows/ci.yml/badge.svg)

## Features

- **Direct hardware control** — Talks CAT protocol over serial, no hamlib dependency
- **Daemon/client architecture** — One daemon controls the radio, multiple clients can connect
- **Three interfaces** — GUI, CLI, and D-Bus (Linux) for scripting and desktop integration
- **Skeuomorphic GUI** — Dark radio-panel aesthetic with VFD-style frequency display, rotary tuning knob, and toggle button groups
- **Localizable** — All GUI strings use fluent-rs for i18n (English included, extensible)
- **Cross-platform** — Builds and runs on Linux, macOS, and Windows
- **HiDPI ready** — Scales cleanly on 4K and high-DPI displays

## Supported Radios

| Radio | Status |
|-------|--------|
| Yaesu FT-450D | Implemented (CAT over serial) |
| Dummy backend | Built-in mock for testing without hardware |

Adding support for additional radios involves implementing the `Radio` trait in the daemon.

## Architecture

```
radioxide-proto          Shared protocol types (commands, status, enums)
    |
radioxide-transports     TCP and D-Bus transport layers
    |
    +-- radioxide-daemon     Background service controlling the radio hardware
    +-- radioxide-cli        Command-line client
    +-- radioxide-gui        iced-based graphical client
```

All communication between clients and the daemon uses JSON-serialized messages over TCP (port 7600 by default). On Linux, D-Bus is also available at `com.radioxide.Daemon`.

## Building

Requires Rust 1.75+ and Cargo.

```bash
# Build the entire workspace
cargo build --workspace

# Build release binaries
cargo build --workspace --release

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace
```

## Usage

### 1. Start the daemon

```bash
# With the dummy (mock) radio backend
cargo run -p radioxide-daemon

# With a real radio on a serial port
# First edit ~/.config/radioxide/config.json:
# {
#   "addr": "127.0.0.1:7600",
#   "radio": {
#     "serial": "/dev/ttyUSB0",
#     "baud": 9600
#   }
# }
cargo run -p radioxide-daemon
```

The daemon creates a default config at `~/.config/radioxide/config.json` on first run. Remove the `radio` section to use the dummy backend.

### 2. Connect with the GUI

```bash
cargo run -p radioxide-gui
```

The GUI provides:
- Large frequency display with comma-separated readout
- Rotary tuning knob with speed-proportional frequency steps (grab and rotate with the mouse)
- Toggle buttons for band, mode, and AGC
- Power and volume sliders
- PTT and tune controls
- Connection status indicator

The tuning knob placement can be toggled between left and right sides for left/right-hand preference.

### 3. Or use the CLI

```bash
cargo run -p radioxide-cli -- status
cargo run -p radioxide-cli -- freq 14074000
cargo run -p radioxide-cli -- band 20m
cargo run -p radioxide-cli -- mode USB
cargo run -p radioxide-cli -- power 50
cargo run -p radioxide-cli -- volume 30
cargo run -p radioxide-cli -- tune
cargo run -p radioxide-cli -- ptt on
cargo run -p radioxide-cli -- ptt off
```

### 4. D-Bus (Linux only)

With the daemon running, you can control the radio via D-Bus:

```bash
# Get current frequency
busctl --user call com.radioxide.Daemon /com/radioxide/Daemon com.radioxide.Daemon GetFrequency

# Set frequency to 7.074 MHz
busctl --user call com.radioxide.Daemon /com/radioxide/Daemon com.radioxide.Daemon SetFrequency t 7074000
```

## Configuration

The daemon reads `~/.config/radioxide/config.json`:

```json
{
  "addr": "127.0.0.1:7600",
  "radio": {
    "serial": "/dev/ttyUSB0",
    "baud": 9600
  }
}
```

| Field | Default | Description |
|-------|---------|-------------|
| `addr` | `127.0.0.1:7600` | TCP listen address for the daemon |
| `radio.serial` | *(none)* | Serial port path. Omit `radio` entirely for the dummy backend |
| `radio.baud` | `9600` | Baud rate for serial communication |

## Packaging

### Linux (Flatpak)

A Flatpak manifest is included (`flatpak-manifest.json`) with app ID `com.radioxide.GUI`.

### Windows (NSIS)

An NSIS installer script is included (`installer.nsi`). Build release binaries first, then compile with `makensis installer.nsi`.

## Project Structure

```
radioxide/
  Cargo.toml                        Workspace root
  radioxide-proto/                  Shared types: RadioCommand, RadioStatus, Band, Mode, Agc
  radioxide-transports/             TCP client/server + D-Bus interface (Linux)
  radioxide-daemon/
    src/
      main.rs                       Daemon entry point, TCP + D-Bus listeners
      radio/
        mod.rs                      Radio trait definition
        dummy.rs                    In-memory mock backend
        yaesu/
          ft450d.rs                 Yaesu FT-450D CAT implementation
          serial.rs                 Async serial port with auto-reconnection
          cat.rs                    CAT command encoding/decoding
  radioxide-cli/                    CLI client (clap v4)
  radioxide-gui/
    src/
      main.rs                       iced application, layout, message handling
      knob.rs                       Canvas-drawn rotary tuning knob widget
      theme.rs                      Color palette and custom theme
      styles.rs                     Per-widget style functions
      widgets.rs                    Toggle button group helper
      i18n.rs                       fluent-rs localization loader
    resources/
      fonts/                        JetBrains Mono Bold (OFL licensed)
      locales/en-US/radio.ftl       English UI strings
  flatpak-manifest.json             Linux Flatpak packaging
  installer.nsi                     Windows NSIS installer
```

## Adding a New Radio

1. Create a new module under `radioxide-daemon/src/radio/` (e.g., `icom/ic7300.rs`)
2. Implement the `Radio` trait — async methods for `set_frequency`, `get_frequency`, `set_mode`, etc.
3. Wire it into `main.rs` as a new backend option in the config

## Contributing

Contributions are welcome. The codebase uses standard Rust tooling:

- `cargo fmt` for formatting
- `cargo clippy` for linting
- `cargo test --workspace` to run all tests

## Acknowledgments

- [iced](https://iced.rs/) — Cross-platform GUI framework
- [tokio](https://tokio.rs/) — Async runtime
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/) — Monospace font (OFL license)
- [fluent-rs](https://github.com/projectfluent/fluent-rs) — Localization system
- [zbus](https://docs.rs/zbus/) — D-Bus library for Linux integration
