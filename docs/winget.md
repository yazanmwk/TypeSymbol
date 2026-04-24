# WinGet Publishing Setup (TypeSymbol)

This guide sets up automatic WinGet PRs so each GitHub release opens a PR to `microsoft/winget-pkgs`.

## Prerequisites

- GitHub repo: `yazanmwk/TypeSymbol`
- Release workflow already publishing:
  - `typesymbol-vX.Y.Z-x86_64-pc-windows-msvc.zip`
  - `checksums.txt`
- Workflow file present: `.github/workflows/publish-winget.yml`

## Step 1: Fork winget-pkgs

1. Open [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).
2. Click **Fork**.
3. Create your fork under your account (for example `yazanmwk/winget-pkgs`).

## Step 2: Create a GitHub token

Create a fine-grained personal access token.

Recommended access:

- **Repository access**: your fork `winget-pkgs`
- **Permissions**:
  - Contents: Read and write
  - Pull requests: Read and write
  - Metadata: Read-only (default)

If you use a classic token, `repo` scope is sufficient.

## Step 3: Add repository secret

In `yazanmwk/TypeSymbol`:

1. Go to **Settings** -> **Secrets and variables** -> **Actions**.
2. Click **New repository secret**.
3. Name: `WINGET_PKGS_TOKEN`
4. Value: paste token from Step 2.
5. Save.

## Step 4: Verify workflow (fork target is automatic)

Open `.github/workflows/publish-winget.yml` and confirm `WINGET_PKGS_REPO` is `microsoft/winget-pkgs`.

The job derives `WINGET_PKGS_FORK` and manifest paths from `github.repository_owner` (your `winget-pkgs` fork must be named `winget-pkgs` under the same account as this repo). If you use a nonstandard fork name, adjust the **Derive WinGet paths** step in that workflow.

## Step 5: Ensure package identifier stays stable

Your package identifier is:

- `yazanmwk.TypeSymbol`

Do not rename this unless you intentionally want a different WinGet package identity.

## Step 6: Cut a release

From your local repo:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Or publish a GitHub release from the UI.

## Step 7: Confirm release artifacts exist

In the release assets, confirm both are present:

- `typesymbol-vX.Y.Z-x86_64-pc-windows-msvc.zip`
- `checksums.txt`

Without these, manifest generation will fail.

## Step 8: Confirm WinGet automation ran

In GitHub Actions:

1. Open workflow run **Publish WinGet Manifests**.
2. Check successful steps:
   - download checksums
   - parse version
   - generate manifests
   - create PR in `microsoft/winget-pkgs`

## Step 9: Review and merge the PR

The workflow opens a PR in `microsoft/winget-pkgs`.

1. Open the PR.
2. Wait for WinGet validation checks.
3. Merge when checks pass.

## Step 10: User install command

After merge and index update, Windows users can run:

```powershell
winget install --id yazanmwk.TypeSymbol
```

## Troubleshooting

- **No PR created**
  - Verify `WINGET_PKGS_TOKEN` exists and is valid.
  - Ensure your `winget-pkgs` fork exists as `github.repository_owner/winget-pkgs` (see workflow).
- **Checksum error**
  - Ensure release has `checksums.txt` and includes the Windows zip line.
- **Branch push failure**
  - Token likely lacks write access to your fork.
- **PR validation fails in winget-pkgs**
  - Check manifest formatting and metadata fields in the generated YAML files.
