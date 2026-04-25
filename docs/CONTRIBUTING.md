# Contributing

Thanks for your interest in TypeSymbol. This repository is developed in the open; issues and pull requests are welcome.

## Contribution boundaries

- Community contributions are for features, bug fixes, docs, and tests via pull requests.
- Release publishing is maintainer-only: contributors do not cut releases or publish artifacts.
- Version tags (`v*`) are the release gate and are pushed by maintainers in the canonical repository.

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

Release automation and local packaging scripts assume a default GitHub org/user. If you publish from a **fork**, set environment variables when generating manifests (see [releasing.md](releasing.md)):

| Variable | Used by | Purpose |
| --- | --- | --- |
| `TYPESYMBOL_GITHUB_REPO` | `scripts/generate-homebrew-formula.sh` | `owner/TypeSymbol` for release URLs (default: `yazanmwk/TypeSymbol`) |
| `HOMEBREW_TAP_REPO` | Homebrew script | `owner/homebrew-tap` (default: `yazanmwk/homebrew-tap`) |

For **Homebrew** automation in GitHub Actions, the workflow uses `TAP_REPO: ${{ github.repository_owner }}/homebrew-tap`. If your tap’s repository name is not `homebrew-tap`, edit [`.github/workflows/publish-homebrew-tap.yml`](.github/workflows/publish-homebrew-tap.yml) accordingly.

## Product context

A detailed product spec lives in [PRD.md](PRD.md).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](../LICENSE).
