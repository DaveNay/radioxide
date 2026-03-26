# Development Guide

## Prerequisites

- Rust 1.85+ and Cargo (install via [rustup](https://rustup.rs))
- Linux: `pkg-config`, `libudev-dev`, `libxkbcommon-dev`, `libwayland-dev`, `libegl-dev`
- Windows: MSVC build tools
- macOS: Xcode command line tools

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Build release binaries
cargo build --workspace --release

# Build a single crate
cargo build -p radioxide-daemon

# Run individual binaries
cargo run -p radioxide-daemon
cargo run -p radioxide-cli
cargo run -p radioxide-gui

# Check without codegen (faster)
cargo check --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace

# Format
cargo fmt --all
```

## Architecture

Radioxide uses a daemon/client architecture. Five crates in a Cargo workspace:

```
radioxide-proto          (lib)  Core protocol types: RadioCommand, RadioxideMessage,
    |                           RadioStatus, Band, Mode, Agc, Vfo. Serialized as JSON.
    |
radioxide-transports     (lib)  Transport backends:
    |                           - tcp: async TCP server/client (tokio)
    |                           - dbus: D-Bus interface (Linux only, com.radioxide.Daemon)
    |
    +-- radioxide-daemon  (bin)  Background service. Radio trait backends:
    |                           - radio/dummy.rs: in-memory mock (default, no --serial)
    |                           - radio/yaesu/ft450d.rs: Yaesu FT-450D via CAT over serial
    |
    +-- radioxide-cli     (bin)  CLI client (clap v4 derive)
    +-- radioxide-gui     (bin)  GUI client (iced v0.13)
```

All three binaries depend on `radioxide-proto` and `radioxide-transports`. Change a type in proto and all crates see it immediately.

## Key Types

| Type | Crate | Description |
|------|-------|-------------|
| `RadioCommand` | proto | Enum of all commands: `SetFrequency`, `SetBand`, `SetMode`, `SetVfo`, `Tune`, `PttOn/Off`, `SetPower`, `SetVolume`, `SetAgc`, and `Get*` variants |
| `RadioStatus` | proto | Full radio state snapshot |
| `RadioxideMessage` | proto | Request envelope wrapping a `RadioCommand` |
| `RadioxideResponse` | proto | Response with `success`, `message`, optional `RadioStatus` |
| `Radio` | daemon | Async trait for hardware backends |
| `DummyRadio` | daemon | In-memory mock implementing `Radio` |
| `Ft450d` | daemon | Yaesu FT-450D CAT implementation |

## Cross-Platform Requirements

All three binaries must compile and run on **Windows, macOS, and Linux**. Platform-specific code must be gated with `cfg` attributes — never a hard dependency that breaks other platforms.

- **D-Bus** (`zbus`) is Linux-only: gated behind `#[cfg(target_os = "linux")]`
- **Flatpak** packaging: `packaging/flatpak-manifest.json`, app ID `com.radioxide.GUI`
- **Windows** packaging: `packaging/installer.nsi`, run `makensis packaging\installer.nsi` from repo root

## Adding a New Radio

1. Create a module under `radioxide-daemon/src/radio/` (e.g., `icom/ic7300.rs`)
2. Implement the `Radio` trait — async methods for each command
3. Wire it into `main.rs` as a backend option via config

## Configuration

The daemon reads `~/.config/radioxide/config.json` (created on first run):

```json
{
  "addr": "127.0.0.1:7600",
  "radio": {
    "serial": "/dev/ttyUSB0",
    "baud": 9600
  }
}
```

Omit the `radio` key entirely to use the dummy backend.
