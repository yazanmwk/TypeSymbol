# Homebrew Tap Setup

Standard tap for this project:

- `yazanmwk/homebrew-tap`

## 1) Create the tap repository once

Create a new GitHub repository named:

- `homebrew-tap`

Then add a `Formula` directory:

```bash
mkdir -p Formula
```

## 2) Generate formula from a release

From this repo (TypeSymbol), after downloading `checksums.txt` from a release:

```bash
chmod +x scripts/generate-homebrew-formula.sh
./scripts/generate-homebrew-formula.sh 0.1.0 ./checksums.txt
```

This generates:

- `packaging/homebrew/typesymbol.rb`

Copy that file into your tap repo at:

- `Formula/typesymbol.rb`

Commit and push in tap repo.

## 3) Install command for users

Users can then install with:

```bash
brew tap yazanmwk/tap
brew install typesymbol
```

or directly:

```bash
brew install yazanmwk/tap/typesymbol
```

## 4) Enable automatic PRs to tap repo

This repo includes `.github/workflows/publish-homebrew-tap.yml`, which:
- triggers when a GitHub release is published
- downloads `checksums.txt` from that release
- generates `typesymbol.rb`
- opens a PR to `yazanmwk/homebrew-tap`

Required one-time setup in this repo (`TypeSymbol`) GitHub settings:

- Add repository secret: `HOMEBREW_TAP_TOKEN`
- Value: a GitHub Personal Access Token that can write to `yazanmwk/homebrew-tap`
  - Minimum recommended scope: `repo`

After that, each new release tag (`vX.Y.Z`) will automatically produce a tap PR.
