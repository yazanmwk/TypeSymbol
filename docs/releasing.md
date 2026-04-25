# Releasing TypeSymbol

This is fully automated once set up.

## What I set up

- GitHub Action: `.github/workflows/release.yml`
- Trigger: push a tag like `v0.1.0` (no release on regular branch pushes)
- Outputs:
  - `typesymbol-vX.Y.Z-x86_64-apple-darwin.tar.gz`
  - `typesymbol-vX.Y.Z-aarch64-apple-darwin.tar.gz`
  - `typesymbol-vX.Y.Z-x86_64-pc-windows-msvc.zip`
  - `typesymbol-vX.Y.Z-install-windows.ps1`
  - `checksums.txt`
- Publishes all artifacts to a GitHub Release automatically.

## Release safety model

- External contributors can open issues and PRs, but they cannot publish releases.
- Release publishing is locked to version tags (`v*`) in the canonical repo (`yazanmwk/TypeSymbol`).
- Homebrew tap automation only runs for non-prerelease releases from the canonical repo.
- In practice, only maintainers with tag push rights can cut a release.

## How you cut a release

From your local repo:

```bash
git tag v0.1.0
git push origin v0.1.0
```

That is it. The workflow builds and publishes release binaries.

## Package manager publishing

After a release, download the `checksums.txt` artifact and run:

### Homebrew formula generation

```bash
chmod +x scripts/generate-homebrew-formula.sh
./scripts/generate-homebrew-formula.sh 0.1.0 ./checksums.txt
```

Output:
- `packaging/homebrew/typesymbol.rb`

Then copy it into your tap repository under:
- `Formula/typesymbol.rb`
- Standard tap for this project: `yazanmwk/homebrew-tap`
- Setup guide: [homebrew-tap.md](homebrew-tap.md)

## Implemented automation

- Homebrew tap PR automation is included:
  - `.github/workflows/publish-homebrew-tap.yml`
  - Trigger: release published
  - Action: opens a PR in the `owner/homebrew-tap` repo for the same GitHub user/org as the release (see workflow) updating `Formula/typesymbol.rb`

One-time required secret in this repo:
- `HOMEBREW_TAP_TOKEN` (token with write access to your `homebrew-tap` repo)
