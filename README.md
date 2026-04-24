# TypeSymbol

TypeSymbol is a system-wide math shorthand tool.  
It lets you type expressions like `alpha -> beta` and convert them into Unicode math symbols like `α → β`.

## Features

- Cross-platform core engine in Rust
- Global daemon flow for system-wide replacement
- CLI + terminal UI for config, testing, and daemon control
- Formula/release automation for macOS and Windows distribution

## Quick Start

### macOS

```bash
chmod +x scripts/install-macos.sh
./scripts/install-macos.sh
typesymbol
```

### Windows (PowerShell)

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\install-windows.ps1
typesymbol
```

## Basic Usage

```bash
typesymbol test "alpha -> beta"
typesymbol config init
typesymbol config show
typesymbol daemon status
```

## Repository Layout

- `typesymbol-core` – parser/formatter/rule engine
- `typesymbol-config` – config model/loading/defaults
- `typesymbol-daemon` – runtime and event pipeline
- `typesymbol-platform-macos` – macOS input/replacement adapter
- `typesymbol-platform-windows` – Windows input/replacement adapter
- `typesymbol-cli` – CLI and TUI entrypoint
- `scripts/` – installer + packaging helper scripts
- `.github/workflows/` – release and package automation

## Releases and Packaging

- Release process: `RELEASING.md`
- Installer details: `INSTALL.md`
- Homebrew tap setup: `HOMEBREW_TAP_SETUP.md`

## Security

Please do not report vulnerabilities via public issues first.  
See `SECURITY.md` for responsible disclosure guidance.
