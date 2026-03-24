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

# Run tests (none exist yet)
cargo test --workspace

# Clippy lint
cargo clippy --workspace
```

## Architecture

Radioxide is a cross-platform radio application with a daemon/client architecture. Five crates in a Cargo workspace:

```
radioxide-proto          (lib)  Core protocol types: RadioxideCommand enum, RadioxideMessage struct
    ↓                           Serialized as JSON via serde
radioxide-transports     (lib)  Two transport backends:
    │                           - tcp module: async TCP server/client (tokio)
    │                           - dbus module: D-Bus interface via zbus (com.radioxide.Daemon)
    ↓
radioxide-daemon         (bin)  Background service (stub)
radioxide-cli            (bin)  CLI client using clap v4 derive
radioxide-gui            (bin)  GUI client using iced v0.13.0
```

All three binaries depend on both `radioxide-proto` and `radioxide-transports`. The proto crate is the foundation — change message types there and all crates see the update.

## Key Types

- `RadioxideCommand` (proto): enum with `Play`, `Pause`, `Stop`, `SetVolume(u8)`
- `RadioxideMessage` (proto): struct with `command: RadioxideCommand` and `payload: Option<String>`
- `RadioxideDBus` (transports::dbus): D-Bus interface struct at path `/com/radioxide/Daemon`

## Platform-Specific Notes

- **D-Bus transport** (`zbus` crate) is Linux-specific
- **Flatpak** packaging: app ID `com.radioxide.GUI`, FreeDesktop Platform 22.08
- **Windows** packaging: NSIS installer (`installer.nsi`), installs to Program Files with desktop shortcut
