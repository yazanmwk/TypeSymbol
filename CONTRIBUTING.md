# Contributing

Thanks for your interest in TypeSymbol. This repository is developed in the open; issues and pull requests are welcome.

## Build from source

You need a recent **stable Rust** toolchain (see [rustup.rs](https://rustup.rs/)).

```bash
cargo build --release -p typesymbol-cli
```

Run the test suite (cross-platform; platform-specific code is `cfg`-gated):

```bash
cargo test
```

### Platform targets

- **macOS:** build on macOS to compile `typesymbol-platform-macos` and the full CLI.
- **Windows:** build on Windows to compile `typesymbol-platform-windows` and the full CLI.
- On Linux, `cargo test` still exercises the core and other crates; the full daemon + CLI binary may require a macOS or Windows target for linking platform crates.

## Packaging scripts and forks

Release automation and local packaging scripts assume a default GitHub org/user. If you publish from a **fork**, set environment variables when generating manifests (see [RELEASING.md](RELEASING.md)):

| Variable | Used by | Purpose |
| --- | --- | --- |
| `TYPESYMBOL_GITHUB_REPO` | `scripts/generate-homebrew-formula.sh`, `scripts/generate-winget-manifests.ps1` | `owner/TypeSymbol` for release URLs (default: `yazanmwk/TypeSymbol`) |
| `HOMEBREW_TAP_REPO` | Homebrew script | `owner/homebrew-tap` (default: `yazanmwk/homebrew-tap`) |
| `WINGET_PUBLISHER` | WinGet PowerShell script | WinGet path/publisher segment (default: `yazanmwk`) |

For **Homebrew** automation in GitHub Actions, the workflow uses `TAP_REPO: ${{ github.repository_owner }}/homebrew-tap`. If your tap’s repository name is not `homebrew-tap`, edit [`.github/workflows/publish-homebrew-tap.yml`](.github/workflows/publish-homebrew-tap.yml) accordingly.

For **WinGet**, paths and the fork to push to are derived from `github.repository_owner` in [`.github/workflows/publish-winget.yml`](.github/workflows/publish-winget.yml), and the manifest step passes `TYPESYMBOL_GITHUB_REPO` and `WINGET_PUBLISHER` via the environment. You still need a `WINGET_PKGS_TOKEN` with access to your `winget-pkgs` fork.

## Product context

A detailed product spec lives in [docs/PRD.md](docs/PRD.md).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
