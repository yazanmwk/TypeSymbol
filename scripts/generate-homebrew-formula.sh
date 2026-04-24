#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <version-without-v> <checksums-file>"
  echo "Example: $0 0.1.0 ./checksums.txt"
  exit 1
fi

VERSION="$1"
CHECKSUMS_FILE="$2"
REPO="yazanmwk/TypeSymbol"
HOMEBREW_TAP_REPO="${HOMEBREW_TAP_REPO:-yazanmwk/homebrew-tap}"
TEMPLATE="packaging/homebrew/typesymbol.rb.template"
OUTPUT="packaging/homebrew/typesymbol.rb"

if [[ ! -f "$CHECKSUMS_FILE" ]]; then
  echo "checksums file not found: $CHECKSUMS_FILE"
  exit 1
fi

if [[ ! -f "$TEMPLATE" ]]; then
  echo "template file not found: $TEMPLATE"
  exit 1
fi

extract_sha() {
  local artifact="$1"
  awk -v name="$artifact" '$2 ~ name { print $1 }' "$CHECKSUMS_FILE" | tail -n 1
}

ART_X64="typesymbol-v${VERSION}-x86_64-apple-darwin.tar.gz"
ART_ARM="typesymbol-v${VERSION}-aarch64-apple-darwin.tar.gz"

SHA_X64="$(extract_sha "$ART_X64")"
SHA_ARM="$(extract_sha "$ART_ARM")"

if [[ -z "$SHA_X64" || -z "$SHA_ARM" ]]; then
  echo "could not find expected macOS artifact checksums in $CHECKSUMS_FILE"
  echo "expected: $ART_X64 and $ART_ARM"
  exit 1
fi

URL_X64="https://github.com/${REPO}/releases/download/v${VERSION}/${ART_X64}"
URL_ARM="https://github.com/${REPO}/releases/download/v${VERSION}/${ART_ARM}"

sed \
  -e "s|__VERSION__|${VERSION}|g" \
  -e "s|__URL_MACOS_X64__|${URL_X64}|g" \
  -e "s|__SHA_MACOS_X64__|${SHA_X64}|g" \
  -e "s|__URL_MACOS_ARM__|${URL_ARM}|g" \
  -e "s|__SHA_MACOS_ARM__|${SHA_ARM}|g" \
  "$TEMPLATE" > "$OUTPUT"

echo "Generated Homebrew formula: $OUTPUT"
echo "Next: copy this file to your tap repo: https://github.com/${HOMEBREW_TAP_REPO}"
echo "Target path in tap repo: Formula/typesymbol.rb"
echo "Tip: override default tap with HOMEBREW_TAP_REPO=<owner/homebrew-tap>"
