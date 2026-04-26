# TypeSymbol Install Guide

Official install paths:
- macOS: Homebrew
- Windows: MSI installer from GitHub Releases

## macOS (Homebrew)

```bash
brew tap yazanmwk/homebrew-tap
brew install typesymbol
```

Or:

```bash
brew install yazanmwk/homebrew-tap/typesymbol
```

Update:

```bash
brew update
brew upgrade typesymbol
```

## Windows installer (official path)

### Install

1. Open [latest release](https://github.com/yazanmwk/TypeSymbol/releases/latest).
2. Download `typesymbol-vX.Y.Z-x86_64-pc-windows-msvc.msi`.
3. Run the installer.
4. Open a new PowerShell window.

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

If you see a `VCRUNTIME140.dll` error, install the Microsoft VC++ Redistributable (x64), then rerun the latest MSI installer:

```powershell
Invoke-WebRequest https://aka.ms/vs/17/release/vc_redist.x64.exe -OutFile vc_redist.x64.exe
Start-Process .\vc_redist.x64.exe -ArgumentList "/install", "/quiet", "/norestart" -Wait
```

### Verify release checksums (recommended)

Before running the MSI, verify integrity with `checksums.txt`.

Windows PowerShell:

```powershell
$ErrorActionPreference = "Stop"

$version = "x.y.z"
$asset = "typesymbol-v$version-x86_64-pc-windows-msvc.msi"
$base = "https://github.com/yazanmwk/TypeSymbol/releases/download/v$version"

Invoke-WebRequest "$base/$asset" -OutFile $asset
Invoke-WebRequest "$base/checksums.txt" -OutFile "checksums.txt"

$line = Select-String -Path .\checksums.txt -Pattern ([regex]::Escape($asset)) | Select-Object -First 1
if (-not $line) { throw "No checksum entry found for $asset" }

$expected = ($line.ToString() -split '\s+')[0].ToLower()
$actual = (Get-FileHash ".\$asset" -Algorithm SHA256).Hash.ToLower()
if ($expected -ne $actual) { throw "Checksum mismatch. expected=$expected actual=$actual" }
"Checksum OK"
```

## Build from source (developer path)

Build on a Windows machine with Rust installed:

```powershell
cargo build --release -p typesymbol-cli
```

Then run the binary directly:

```powershell
.\target\release\typesymbol.exe
```

## Notes

- The daemon is currently blocked inside virtual machines by design.
- CLI/TUI commands still work on host machines.
