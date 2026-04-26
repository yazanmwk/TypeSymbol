# TypeSymbol Install Guide

TypeSymbol is distributed as a macOS app via Homebrew.

## Install on macOS

```bash
brew tap yazanmwk/homebrew-tap
brew install typesymbol
```

Or install directly:

```bash
brew install yazanmwk/homebrew-tap/typesymbol
```

## Verify installation

```bash
typesymbol test "alpha -> beta"
typesymbol daemon status
```

## Update

```bash
brew update
brew upgrade typesymbol
```

## Build from source (developer path)

```bash
cargo build --release -p typesymbol-cli
./target/release/typesymbol
```

## Notes

- The daemon is currently blocked inside virtual machines by design.
- CLI/TUI commands still work on host machines.
