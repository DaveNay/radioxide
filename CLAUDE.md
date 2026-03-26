# CLAUDE.md

This file provides guidance to Claude Code when working in this repository.
See [DEVELOPMENT.md](DEVELOPMENT.md) for build commands, architecture, key types, and cross-platform requirements.

## Cross-Platform Requirements

All three executables (daemon, CLI, GUI) must compile and run on **Windows, macOS, and Linux**. Platform-specific code must be behind `cfg` attributes or feature flags — never a hard dependency that prevents compilation on other platforms.

## Platform-Specific Notes

- **D-Bus transport** (`zbus` crate) is Linux-only — must be gated behind `#[cfg(target_os = "linux")]`
- **Flatpak** packaging: app ID `com.radioxide.GUI`, FreeDesktop Platform 22.08 (Linux only)
- **Windows** packaging: NSIS installer (`packaging/installer.nsi`), installs to Program Files with desktop shortcut
