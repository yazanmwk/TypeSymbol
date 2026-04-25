#!/usr/bin/env bash
set -euo pipefail

# Syncs generated release metadata used by package managers.
# - Regenerates packaging/homebrew/typesymbol.rb from the latest GitHub release.
# - Optionally stages updated files when run from a git hook.

REPO="${TYPESYMBOL_GITHUB_REPO:-yazanmwk/TypeSymbol}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/generate-homebrew-formula.sh"
FORMULA="$ROOT_DIR/packaging/homebrew/typesymbol.rb"
WINDOWS_INSTALLER="$ROOT_DIR/scripts/install-windows-release.ps1"
AUTO_STAGE="${AUTO_STAGE:-0}"
STRICT_RELEASE_SYNC="${STRICT_RELEASE_SYNC:-0}"

if [[ ! -x "$GENERATOR" ]]; then
  chmod +x "$GENERATOR"
fi

if [[ ! -f "$WINDOWS_INSTALLER" ]]; then
  echo "Missing Windows installer script: $WINDOWS_INSTALLER" >&2
  exit 1
fi

api_url="https://api.github.com/repos/${REPO}/releases/latest"
if ! release_json="$(curl -fsSL -H "Accept: application/vnd.github+json" -H "User-Agent: TypeSymbolSyncScript" "$api_url")"; then
  echo "Warning: unable to fetch latest release metadata from GitHub; skipping Homebrew sync." >&2
  if [[ "$STRICT_RELEASE_SYNC" == "1" ]]; then
    exit 1
  fi
  if [[ "$AUTO_STAGE" == "1" ]] && command -v git >/dev/null 2>&1; then
    git -C "$ROOT_DIR" add "$WINDOWS_INSTALLER"
    echo "Staged Windows installer only."
  fi
  exit 0
fi

readarray -t release_info < <(
  RELEASE_JSON="$release_json" python3 - <<'PY'
import json
import os
import sys

data = json.loads(os.environ["RELEASE_JSON"])
tag = data.get("tag_name", "")
if not tag:
    sys.stderr.write("Latest release has no tag_name\n")
    sys.exit(1)

checksums_url = ""
for asset in data.get("assets", []):
    if asset.get("name") == "checksums.txt":
        checksums_url = asset.get("browser_download_url", "")
        break

if not checksums_url:
    sys.stderr.write("Latest release is missing checksums.txt asset\n")
    sys.exit(1)

print(tag)
print(checksums_url)
PY
)

release_tag="${release_info[0]}"
checksums_url="${release_info[1]}"
version="${release_tag#v}"

tmp_checksums="$(mktemp)"
cleanup() {
  rm -f "$tmp_checksums"
}
trap cleanup EXIT

if ! curl -fsSL -H "User-Agent: TypeSymbolSyncScript" "$checksums_url" -o "$tmp_checksums"; then
  echo "Warning: unable to download checksums.txt; skipping Homebrew sync." >&2
  if [[ "$STRICT_RELEASE_SYNC" == "1" ]]; then
    exit 1
  fi
  if [[ "$AUTO_STAGE" == "1" ]] && command -v git >/dev/null 2>&1; then
    git -C "$ROOT_DIR" add "$WINDOWS_INSTALLER"
    echo "Staged Windows installer only."
  fi
  exit 0
fi

"$GENERATOR" "$version" "$tmp_checksums" >/dev/null
echo "Synced Homebrew formula to ${release_tag}: $FORMULA"

if [[ "$AUTO_STAGE" == "1" ]] && command -v git >/dev/null 2>&1; then
  git -C "$ROOT_DIR" add "$FORMULA" "$WINDOWS_INSTALLER"
  echo "Staged metadata files for commit."
fi
