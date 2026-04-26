# TypeSymbol Install Guide

Official installs are package-manager based:
- macOS: Homebrew tap
- Windows: WinGet installer package

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

### Build from this repository (developer path)

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

Default CLI interface (Windows): `typesymbol` with no args opens the command shell.

```powershell
typesymbol
# in-shell commands: on, off, daemon status, config show, help, exit
```

If you see a `VCRUNTIME140.dll` error, install the Microsoft VC++ Redistributable (x64), then run `winget upgrade` again:

```powershell
Invoke-WebRequest https://aka.ms/vs/17/release/vc_redist.x64.exe -OutFile vc_redist.x64.exe
Start-Process .\vc_redist.x64.exe -ArgumentList "/install", "/quiet", "/norestart" -Wait
winget upgrade --id yazanmwk.TypeSymbol --exact --source winget
```

### Verify release checksums (manual assets only)

If you manually download release assets from GitHub instead of using package managers, verify integrity with `checksums.txt`.

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

macOS:

```bash
VERSION="x.y.z"
curl -L -o "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz"
curl -L -o "checksums.txt" "https://github.com/yazanmwk/TypeSymbol/releases/download/v${VERSION}/checksums.txt"
grep "typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz" checksums.txt | shasum -a 256 -c -
```

## Windows (from source)

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
