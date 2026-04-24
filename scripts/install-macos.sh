#!/usr/bin/env bash
set -euo pipefail

log() {
  printf "\n[TypeSymbol installer] %s\n" "$1"
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

install_xcode_tools() {
  if xcode-select -p >/dev/null 2>&1; then
    log "Xcode Command Line Tools already installed."
    return
  fi

  log "Installing Xcode Command Line Tools..."
  xcode-select --install || true
  log "Follow Apple's prompt, then rerun this installer."
  exit 1
}

install_homebrew() {
  if have_cmd brew; then
    log "Homebrew already installed."
    return
  fi

  log "Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
}

install_git() {
  if have_cmd git; then
    log "Git already installed: $(git --version)"
    return
  fi

  log "Installing git with Homebrew..."
  brew install git
}

install_rustup() {
  if have_cmd rustup; then
    log "rustup already installed: $(rustup --version | head -n 1)"
    return
  fi

  log "Installing rustup + Rust stable toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

ensure_rust_on_path() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi

  if ! have_cmd cargo; then
    log "cargo not found on PATH after rustup install."
    log "Open a new terminal or run: source \$HOME/.cargo/env"
    exit 1
  fi
}

install_typesymbol() {
  local repo_dir
  repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

  log "Building TypeSymbol from: $repo_dir"
  (cd "$repo_dir" && cargo build --release)

  mkdir -p "$HOME/.local/bin"
  cp "$repo_dir/target/release/typesymbol" "$HOME/.local/bin/typesymbol"
  chmod +x "$HOME/.local/bin/typesymbol"

  log "Installed binary to $HOME/.local/bin/typesymbol"
  log "If needed, add to PATH:"
  echo '  echo '\''export PATH="$HOME/.local/bin:$PATH"'\'' >> ~/.zshrc && source ~/.zshrc'
}

main() {
  log "Starting macOS dependency bootstrap..."
  install_xcode_tools
  install_homebrew
  install_git
  install_rustup
  ensure_rust_on_path
  rustup toolchain install stable >/dev/null
  rustup default stable >/dev/null
  install_typesymbol
  log "Done."
}

main "$@"
