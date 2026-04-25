# TypeSymbol Install Guide

These installers bootstrap both dependencies and the `typesymbol` binary.

## macOS

### Homebrew (recommended)

If you use [Homebrew](https://brew.sh/):

```bash
brew tap yazanmwk/tap
brew install typesymbol
```

or `brew install yazanmwk/tap/typesymbol`. For tap setup and version bumps, see [homebrew-tap.md](homebrew-tap.md).

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

## Windows (GitHub installer - recommended)

If you just want to install TypeSymbol on Windows:

```powershell
irm https://raw.githubusercontent.com/yazanmwk/TypeSymbol/main/scripts/install-windows-release.ps1 | iex
```

Then run:

```powershell
typesymbol
```

If you see a `VCRUNTIME140.dll` error, install the Microsoft VC++ Redistributable (x64), then rerun:

```powershell
Invoke-WebRequest https://aka.ms/vs/17/release/vc_redist.x64.exe -OutFile vc_redist.x64.exe
Start-Process .\vc_redist.x64.exe -ArgumentList "/install", "/quiet", "/norestart" -Wait
```

Optional version-pinned install:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/yazanmwk/TypeSymbol/main/scripts/install-windows-release.ps1))) -Version 0.1.0
```

### Verify release checksums (recommended)

Before manual install from GitHub release assets, verify integrity with `checksums.txt`.

Windows PowerShell:

```powershell
$version = "0.1.0"
Invoke-WebRequest "https://github.com/yazanmwk/TypeSymbol/releases/download/v$version/typesymbol-v$version-x86_64-pc-windows-msvc.zip" -OutFile "typesymbol-v$version-x86_64-pc-windows-msvc.zip"
Invoke-WebRequest "https://github.com/yazanmwk/TypeSymbol/releases/download/v$version/checksums.txt" -OutFile "checksums.txt"
$expected = (Select-String -Path .\checksums.txt -Pattern "typesymbol-v$version-x86_64-pc-windows-msvc.zip").ToString().Split(" ")[0]
$actual = (Get-FileHash ".\typesymbol-v$version-x86_64-pc-windows-msvc.zip" -Algorithm SHA256).Hash.ToLower()
if ($expected -eq $actual) { "Checksum OK" } else { throw "Checksum mismatch" }
```

macOS:

```bash
VERSION="0.1.0"
curl -L -o "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz"
curl -L -o "checksums.txt" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/checksums.txt"
grep "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" checksums.txt | shasum -a 256 -c -
```

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
