# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Build release
cargo build --workspace --release

# Build a single crate
cargo build -p radioxide-daemon

# Run individual binaries
cargo run -p radioxide-daemon
cargo run -p radioxide-cli
cargo run -p radioxide-gui

# Check (faster than build, no codegen)
cargo check --workspace

# Run tests
cargo test --workspace

# Clippy lint
cargo clippy --workspace
```

## Architecture

Radioxide is a cross-platform ham radio remote control application (hamlib replacement) with a daemon/client architecture. The daemon communicates directly with radio hardware — no hamlib or similar libraries. Five crates in a Cargo workspace:

```
radioxide-proto          (lib)  Core protocol types: RadioCommand enum, RadioxideMessage struct,
    ↓                           Band/Mode/Agc enums, RadioStatus. Serialized as JSON via serde
radioxide-transports     (lib)  Two transport backends:
    │                           - tcp module: async TCP server/client (tokio)
    │                           - dbus module: D-Bus interface via zbus (com.radioxide.Daemon)
    ↓
radioxide-daemon         (bin)  Background service with Radio trait backend:
    │                           - radio/dummy.rs: in-memory mock (default when no --serial)
    │                           - radio/yaesu/ft450d.rs: Yaesu FT-450D via CAT over serial
    │                           Configured via ~/.config/radioxide/config.json
radioxide-cli            (bin)  CLI client using clap v4 derive
radioxide-gui            (bin)  Reference GUI client using iced v0.13.0 (freq, band, mode, AGC, power, volume, PTT, tune)
```

All three binaries depend on both `radioxide-proto` and `radioxide-transports`. The proto crate is the foundation — change message types there and all crates see the update.

## Key Types

- `RadioCommand` (proto): enum — `SetFrequency(u64)`, `SetBand(Band)`, `SetMode(Mode)`, `Tune`, `PttOn`/`PttOff`, `SetPower(u8)`, `SetVolume(u8)`, `SetAgc(Agc)`, and corresponding `Get*` variants plus `GetStatus`
- `Band` (proto): enum — `Band160m` through `Band70cm` (13 HF/VHF/UHF bands)
- `Mode` (proto): enum — `LSB`, `USB`, `CW`, `AM`, `FM`, `Digital`, `CWR`, `DigitalR`
- `Agc` (proto): enum — `Off`, `Fast`, `Medium`, `Slow`
- `RadioStatus` (proto): full radio state snapshot (frequency, band, mode, power, volume, AGC, PTT, tuning)
- `RadioxideMessage` (proto): envelope with `command: RadioCommand`
- `RadioxideResponse` (proto): `success`, `message`, optional `status: RadioStatus`
- `Radio` (daemon::radio): async trait for radio hardware abstraction — `set_frequency`, `get_frequency`, `set_mode`, etc.
- `DummyRadio` (daemon::radio::dummy): in-memory mock implementation
- `Ft450d` (daemon::radio::yaesu::ft450d): Yaesu FT-450D via CAT serial protocol
- `CatPort` (daemon::radio::yaesu::serial): async serial port wrapper for CAT commands
- `RadioxideDBus` (transports::dbus): D-Bus interface struct at path `/com/radioxide/Daemon`

## Cross-Platform Requirements

All three executables (daemon, CLI, GUI) must compile and run on **Windows, macOS, and Linux**. Platform-specific code must be behind `cfg` attributes or feature flags — never a hard dependency that prevents compilation on other platforms.

## Platform-Specific Notes

- **D-Bus transport** (`zbus` crate) is Linux-only — must be gated behind `#[cfg(target_os = "linux")]`
- **Flatpak** packaging: app ID `com.radioxide.GUI`, FreeDesktop Platform 22.08 (Linux only)
- **Windows** packaging: NSIS installer (`installer.nsi`), installs to Program Files with desktop shortcut
