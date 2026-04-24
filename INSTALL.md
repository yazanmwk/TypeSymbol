# TypeSymbol Install Guide

These installers bootstrap both dependencies and the `typesymbol` binary.

## macOS

From repository root:

```bash
chmod +x scripts/install-macos.sh
./scripts/install-macos.sh
```

What it installs/checks:
- Xcode Command Line Tools
- Homebrew
- Git
- rustup + Rust stable toolchain
- Builds TypeSymbol and installs `typesymbol` to `~/.local/bin/typesymbol`

After install, run:

```bash
typesymbol
```

If command is not found, add `~/.local/bin` to your shell PATH.

## Windows (PowerShell)

Run PowerShell as Administrator in repository root:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\install-windows.ps1
```

What it installs/checks:
- `winget` availability
- Git
- Visual Studio Build Tools (C++ workload)
- rustup + Rust stable toolchain
- Builds TypeSymbol and installs `typesymbol.exe` to `%USERPROFILE%\bin`
- Adds `%USERPROFILE%\bin` to user PATH

After install, open a new PowerShell window and run:

```powershell
typesymbol
```

## Notes

- The daemon is currently blocked inside virtual machines by design.
- CLI/TUI commands still work on host machines.
