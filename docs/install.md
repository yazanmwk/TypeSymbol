# TypeSymbol Install Guide

These installers bootstrap both dependencies and the `typesymbol` binary.

## macOS

### Homebrew (recommended)

If you use [Homebrew](https://brew.sh/):

```bash
brew tap yazanmwk/homebrew-tap
brew install typesymbol
```

or `brew install yazanmwk/homebrew-tap/typesymbol`. For tap setup and version bumps, see [homebrew-tap.md](homebrew-tap.md).

To keep your machine up to date after install:

```bash
typesymbol update check
typesymbol update
```

### Build from this repository

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

## Windows (recommended)

### WinGet (recommended)

```powershell
winget install --id yazanmwk.TypeSymbol --exact --source winget
```

Then run:

```powershell
typesymbol test "alpha -> beta"
typesymbol daemon status
```

Manage with WinGet:

```powershell
# Upgrade to latest published version
winget upgrade --id yazanmwk.TypeSymbol --exact --source winget

# Uninstall
winget uninstall --id yazanmwk.TypeSymbol --exact
```

### GitHub installer (fallback)

If you just want to install TypeSymbol on Windows:

```powershell
irm https://raw.githubusercontent.com/yazanmwk/TypeSymbol/main/scripts/install-windows-release.ps1 | iex
```

Then run:

```powershell
typesymbol test "alpha -> beta"
typesymbol daemon status
```

Default CLI interface (Windows): `typesymbol` with no args opens the command shell.

```powershell
typesymbol
# in-shell commands: on, off, daemon status, config show, help, exit
```

If you see a `VCRUNTIME140.dll` error, install the Microsoft VC++ Redistributable (x64), then rerun:

```powershell
Invoke-WebRequest https://aka.ms/vs/17/release/vc_redist.x64.exe -OutFile vc_redist.x64.exe
Start-Process .\vc_redist.x64.exe -ArgumentList "/install", "/quiet", "/norestart" -Wait
```

Optional version-pinned install:

```powershell
# Replace x.y.z with a real tag version shown on:
# https://github.com/yazanmwk/TypeSymbol/releases
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/yazanmwk/TypeSymbol/main/scripts/install-windows-release.ps1))) -Version x.y.z
```

### Verify release checksums (recommended)

Before manual install from GitHub release assets, verify integrity with `checksums.txt`.

Windows PowerShell:

```powershell
$ErrorActionPreference = "Stop"

# Resolve latest release tag automatically.
$latest = Invoke-RestMethod -Uri "https://api.github.com/repos/yazanmwk/TypeSymbol/releases/latest" -Headers @{ "User-Agent" = "TypeSymbolInstallDocs" }
$version = $latest.tag_name.TrimStart("v")
$asset = "typesymbol-v$version-x86_64-pc-windows-msvc.zip"
$base = "https://github.com/yazanmwk/TypeSymbol/releases/download/v$version"

Invoke-WebRequest "$base/$asset" -OutFile $asset
Invoke-WebRequest "$base/checksums.txt" -OutFile "checksums.txt"

if (!(Test-Path ".\$asset")) { throw "Missing file: $asset" }
if (!(Test-Path ".\checksums.txt")) { throw "Missing file: checksums.txt" }

$line = Select-String -Path .\checksums.txt -Pattern ([regex]::Escape($asset)) | Select-Object -First 1
if (-not $line) { throw "No checksum entry found for $asset" }

$expected = ($line.ToString() -split '\s+')[0].ToLower()
$actual = (Get-FileHash ".\$asset" -Algorithm SHA256).Hash.ToLower()

if ($expected -ne $actual) { throw "Checksum mismatch. expected=$expected actual=$actual" }
"Checksum OK"
```

macOS:

```bash
VERSION="x.y.z"
curl -L -o "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz"
curl -L -o "checksums.txt" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/checksums.txt"
grep "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" checksums.txt | shasum -a 256 -c -
```

If Windows autostart setup logs `Access is denied` during install, TypeSymbol still installs and starts the daemon immediately. Re-run from an elevated PowerShell only if you specifically need Task Scheduler registration.

## Windows (PowerShell from source)

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
