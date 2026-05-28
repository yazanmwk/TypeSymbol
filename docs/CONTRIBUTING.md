<!-- Custom Table Styling for local markdown previewers (VS Code, Obsidian, etc.) to match the Mermaid chart's purple theme -->
<style>
  table {
    border-collapse: collapse;
    width: 100%;
    margin: 24px 0;
    color: #e8e0f0 !important;
    background-color: #1a1020 !important;
    border: 1px solid #ba53e6 !important;
  }
  th {
    background-color: #1f1630 !important;
    color: #e8e0f0 !important;
    border: 1px solid #ba53e6 !important;
    padding: 12px;
    font-weight: bold;
  }
  td {
    border: 1px solid #ba53e6 !important;
    padding: 12px;
  }
  tr:nth-child(even) {
    background-color: #1f1630 !important;
  }
  /* Style inline code blocks inside tables to match */
  table code {
    background-color: #2a1635 !important;
    color: #e8d0ff !important;
    border: 1px solid #ba53e6 !important;
    padding: 2px 6px !important;
  }
</style>

# Contributing

Thanks for your interest in TypeSymbol. This repository is developed in the open; issues and pull requests are welcome.

## Contribution boundaries

- Community contributions are for features, bug fixes, docs, and tests via pull requests.
- Release publishing is maintainer-only: contributors do not cut releases or publish artifacts.
- Version tags (`v*`) are the release gate and are pushed by maintainers in the canonical repository.

## Contribution rules

- **Discuss first for non-trivial changes:** open an issue before large features, behavioral changes, or refactors.
- **Keep PRs focused:** one logical change per PR (bug fix, feature slice, docs update, or refactor), not mixed bundles.
- **Preserve default behavior unless intentional:** if behavior changes, explain the reason and migration impact in the PR.
- **Add or update tests for code changes:** no test regressions; new parsing/formatting rules should include coverage.
- **Do not include release/process edits unless requested:** avoid changing release workflows, tagging logic, packaging, or distribution metadata.
- **No secrets or local artifacts:** never commit credentials, local config secrets, or machine-specific generated files.
- **Keep docs in sync:** update user-facing docs when commands, behavior, or supported syntax changes.

## Pull request checklist

Before opening a PR, ensure:

- `cargo build --release -p typesymbol-cli` succeeds on your target platform.
- `cargo test` passes.
- You validated key behavior with `typesymbol test "..."` for the rules you touched.
- The PR description includes: **what changed**, **why**, and **how it was verified**.
- If applicable, include before/after examples for transformed syntax.

## Build from source

You need a recent **stable Rust** toolchain (see [rustup.rs](https://rustup.rs/)).

```bash
cargo build --release -p typesymbol-cli
```

Run the test suite:

```bash
cargo test
```

### Platform targets

- **macOS:** TypeSymbol is macOS-only. You must build and test on macOS.

## Packaging scripts and forks

Release automation and local packaging scripts assume a default GitHub org/user. If you publish from a **fork**, set environment variables when generating manifests (see [releasing.md](releasing.md)):

| Variable | Used by | Purpose |
| --- | --- | --- |
| `TYPESYMBOL_GITHUB_REPO` | `scripts/generate-homebrew-formula.sh` | `owner/TypeSymbol` for release URLs (default: `yazanmwk/TypeSymbol`) |
| `HOMEBREW_TAP_REPO` | Homebrew script | `owner/homebrew-tap` (default: `yazanmwk/homebrew-tap`) |

For **Homebrew** automation in GitHub Actions, the workflow uses `TAP_REPO: ${{ github.repository_owner }}/homebrew-tap`. If your tap’s repository name is not `homebrew-tap`, edit [`.github/workflows/publish-homebrew-tap.yml`](.github/workflows/publish-homebrew-tap.yml) accordingly.

## License

By contributing, you agree that your contributions are licensed under the [MIT License](../LICENSE).
