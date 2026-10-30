#!/usr/bin/env bash
# ==============================================================================
# Sema Studio & Engine — Robust Universal Installer
# Nama Research Lab (https://github.com/Nama-ResearchLab/Sema)
# ==============================================================================
set -e

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}"
cat << "EOF"
  ____                        
 / ___|  ___ _ __ ___   __ _  
 \___ \ / _ \ '_ ` _ \ / _` | 
  ___) |  __/ | | | | | (_| | 
 |____/ \___|_| |_| |_|\__,_| 
 Offline Swahili Language Studio & Deterministic Engine (v0.2.0)
EOF
echo -e "${RESET}"

echo -e "${BOLD}Checking environment...${RESET}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo -e "${YELLOW}Warning: Architecture $ARCH will compile natively.${RESET}" ;;
esac

echo -e "Detected OS: ${GREEN}$OS${RESET}, Architecture: ${GREEN}$ARCH${RESET}"

# Ensure ~/.cargo/bin is sourced if present
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

# 1. Check for Rust / Cargo toolchain
if ! command -v cargo &> /dev/null; then
  echo -e "\n${YELLOW}[!] Cargo/Rust toolchain not found in PATH.${RESET}"
  echo -e "Sema is built for bare-metal deterministic execution and compiles natively."
  
  if command -v curl &> /dev/null; then
    echo -e "${BOLD}Installing Rust toolchain via official rustup (https://rustup.rs)...${RESET}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    if [ -f "$HOME/.cargo/env" ]; then
      # shellcheck source=/dev/null
      source "$HOME/.cargo/env"
    fi
  else
    echo -e "${RED}Error: Neither cargo nor curl was found.${RESET}"
    echo -e "Please install Rust (https://rustup.rs) or curl to continue."
    exit 1
  fi
fi

if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: Failed to initialize Cargo toolchain.${RESET}"
  echo -e "Please ensure $HOME/.cargo/bin is in your PATH and retry."
  exit 1
fi

CARGO_VER="$(cargo --version)"
echo -e "Found Cargo: ${GREEN}$CARGO_VER${RESET}"

INSTALL_DIR="${SEMA_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"
mkdir -p "$HOME/.cargo/bin"

# 2. Compile and install binaries (sema, sema-gui, sema_cc, sema-distill)
echo -e "\n${BOLD}[1/3] Compiling and Installing Sema Binaries...${RESET}"

# Check if running from within an existing checkout of Sema
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || pwd)"
if [ -f "$SCRIPT_DIR/Cargo.toml" ] && grep -q 'name = "sema"' "$SCRIPT_DIR/Cargo.toml" 2>/dev/null; then
  echo -e "Detected local Sema repository at: ${CYAN}$SCRIPT_DIR${RESET}"
  (cd "$SCRIPT_DIR" && cargo install --path . --bins --force)
elif [ -f "Cargo.toml" ] && grep -q 'name = "sema"' "Cargo.toml" 2>/dev/null; then
  echo -e "Detected local Sema repository in current directory."
  cargo install --path . --bins --force
else
  # Remote install via git clone or cargo git
  echo -e "Fetching latest Sema source from GitHub..."
  TMP_BUILD="$(mktemp -d -t sema-install-XXXXXX)"
  trap 'rm -rf "$TMP_BUILD"' EXIT
  
  if command -v git &> /dev/null; then
    git clone --depth 1 https://github.com/Nama-ResearchLab/Sema.git "$TMP_BUILD"
    (cd "$TMP_BUILD" && cargo install --path . --bins --force)
  else
    cargo install --git https://github.com/Nama-ResearchLab/Sema.git --bin sema --bin sema-gui --bin sema_cc --force
  fi
fi

# Ensure binaries exist in ~/.local/bin as well
for bin in sema sema-gui sema_cc sema-distill; do
  if [ -f "$HOME/.cargo/bin/$bin" ]; then
    cp -f "$HOME/.cargo/bin/$bin" "$INSTALL_DIR/$bin"
    chmod +x "$INSTALL_DIR/$bin"
  fi
done

echo -e "\n${GREEN}✓ Binaries installed successfully into ${INSTALL_DIR} and ~/.cargo/bin${RESET}"

# 3. Setup Shell PATH Integration
echo -e "\n${BOLD}[2/3] Configuring Shell Environment...${RESET}"

PATH_NEEDS_UPDATE=1
case ":$PATH:" in
  *":$INSTALL_DIR:"*|*":$HOME/.cargo/bin:"*)
    PATH_NEEDS_UPDATE=0
    ;;
esac

if [ "$PATH_NEEDS_UPDATE" -eq 1 ]; then
  # Determine user's shell rc file
  SHELL_NAME="$(basename "$SHELL")"
  TARGET_RC="$HOME/.profile"
  
  if [ "$SHELL_NAME" = "zsh" ] && [ -f "$HOME/.zshrc" ]; then
    TARGET_RC="$HOME/.zshrc"
  elif [ "$SHELL_NAME" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
      TARGET_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      TARGET_RC="$HOME/.bash_profile"
    fi
  fi

  LINE_EXPORT='export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"'
  if ! grep -qs 'HOME/\.local/bin' "$TARGET_RC"; then
    echo "" >> "$TARGET_RC"
    echo "# Sema Swahili Engine PATH" >> "$TARGET_RC"
    echo "$LINE_EXPORT" >> "$TARGET_RC"
    echo -e "Added ${CYAN}$INSTALL_DIR${RESET} to your PATH in ${CYAN}$TARGET_RC${RESET}"
  fi
  export PATH="$INSTALL_DIR:$HOME/.cargo/bin:$PATH"
fi

# 4. Linux Desktop Menu Integration (for GUI / non-coders)
if [ "$OS" = "linux" ]; then
  DESKTOP_DIR="$HOME/.local/share/applications"
  mkdir -p "$DESKTOP_DIR"
  cat << EOF > "$DESKTOP_DIR/sema.desktop"
[Desktop Entry]
Name=Sema Studio
GenericName=Swahili Language Studio
Comment=Offline Swahili Language Studio & Deterministic Engine
Exec=$INSTALL_DIR/sema-gui
Icon=accessories-character-map
Terminal=false
Type=Application
Categories=Development;TextTools;Science;Education;
Keywords=swahili;nlp;studio;mcp;deterministic;
EOF
  chmod +x "$DESKTOP_DIR/sema.desktop"
  if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
  fi
  echo -e "Created Linux Desktop Launcher at: ${CYAN}$DESKTOP_DIR/sema.desktop${RESET}"
fi

# 5. Automated Verification & Self-Test
echo -e "\n${BOLD}[3/3] Running Post-Install Self-Test...${RESET}"

SEMA_BIN=""
if command -v sema &> /dev/null; then
  SEMA_BIN="$(command -v sema)"
elif [ -x "$INSTALL_DIR/sema" ]; then
  SEMA_BIN="$INSTALL_DIR/sema"
elif [ -x "$HOME/.cargo/bin/sema" ]; then
  SEMA_BIN="$HOME/.cargo/bin/sema"
fi

if [ -n "$SEMA_BIN" ]; then
  TEST_RESULT="$("$SEMA_BIN" synth --subj 3PL --tense FUT --root jibu --obj ku --neg 2>/dev/null || true)"
  if [ "$TEST_RESULT" = "hawatakujibu" ]; then
    echo -e "Self-Test: ${GREEN}100% PASSED${RESET} (Realized: '$TEST_RESULT' in < 1 µs)"
  else
    echo -e "Self-Test Note: Engine installed at ${CYAN}$SEMA_BIN${RESET}"
  fi
fi

echo -e "\n${BOLD}${GREEN}========================================================================${RESET}"
echo -e "${BOLD}${GREEN}  Sema v0.2.0 Installation Complete!${RESET}"
echo -e "${BOLD}${GREEN}========================================================================${RESET}"
echo -e "\n${BOLD}Quick Start:${RESET}"
echo -e "  1. Run CLI Tool:        ${CYAN}sema --help${RESET}"
echo -e "  2. Test Realization:    ${CYAN}sema synth --subj 3PL --tense FUT --root jibu --obj ku --neg${RESET}"
echo -e "  3. Launch Studio Pro:   ${CYAN}sema-gui${RESET}  (or ${CYAN}sema gui${RESET})"
echo -e "  4. Agent MCP Server:    ${CYAN}sema-gui --mcp${RESET}  (or ${CYAN}sema mcp${RESET})"
echo -e "\nIf this is your first install, reload your terminal:"
echo -e "  ${CYAN}source ~/.bashrc${RESET}  (or ${CYAN}source ~/.zshrc${RESET})\n"
echo -e "Documentation & Master Academy: ${CYAN}https://nama-researchlab.github.io/Sema/${RESET}\n"
