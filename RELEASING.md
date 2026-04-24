# Releasing TypeSymbol

This is fully automated once set up.

## What I set up

- GitHub Action: `.github/workflows/release.yml`
- Trigger: push a tag like `v0.1.0`
- Outputs:
  - `typesymbol-vX.Y.Z-x86_64-apple-darwin.tar.gz`
  - `typesymbol-vX.Y.Z-aarch64-apple-darwin.tar.gz`
  - `typesymbol-vX.Y.Z-x86_64-pc-windows-msvc.zip`
  - `checksums.txt`
- Publishes all artifacts to a GitHub Release automatically.

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
- Setup guide: `HOMEBREW_TAP_SETUP.md`

### Winget manifest generation

```powershell
.\scripts\generate-winget-manifests.ps1 -Version 0.1.0 -ChecksumsFile .\checksums.txt
```

Output:
- `packaging/winget/manifests/y/yazanmwk/TypeSymbol/0.1.0/*.yaml`

Then submit those generated files to:
- `microsoft/winget-pkgs`

## Next step (optional)

- Add CI job that auto-opens PRs to:
  - your Homebrew tap repo
  - `microsoft/winget-pkgs`

This gives users one-command installs with no Rust/Git prerequisites.

## Implemented automation

- Homebrew tap PR automation is included:
  - `.github/workflows/publish-homebrew-tap.yml`
  - Trigger: release published
  - Action: opens PR in `yazanmwk/homebrew-tap` updating `Formula/typesymbol.rb`

One-time required secret in this repo:
- `HOMEBREW_TAP_TOKEN` (token with write access to `yazanmwk/homebrew-tap`)
