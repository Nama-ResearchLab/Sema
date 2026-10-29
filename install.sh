#!/usr/bin/env bash
# ==============================================================================
# Sema Studio & Engine — Universal Installer
# Nama Research Lab (https://github.com/Nama-ResearchLab/Sema)
# ==============================================================================
set -e

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}"
cat << "EOF"
  ____                        
 / ___|  ___ _ __ ___   __ _  
 \___ \ / _ \ '_ ` _ \ / _` | 
  ___) |  __/ | | | | | (_| | 
 |____/ \___|_| |_| |_|\__,_| 
 Offline Swahili Language Studio & Deterministic Engine
EOF
echo -e "${RESET}"

echo -e "${BOLD}Checking environment...${RESET}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo -e "${YELLOW}Warning: Architecture $ARCH might require building from source.${RESET}" ;;
esac

echo -e "Detected OS: ${GREEN}$OS${RESET}, Arch: ${GREEN}$ARCH${RESET}"

INSTALL_DIR="${SEMA_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

if command -v cargo &> /dev/null; then
  echo -e "\n${BOLD}[1/2] Installing via Cargo (Recommended)...${RESET}"
  cargo install --git https://github.com/Nama-ResearchLab/Sema.git --bin sema --bin sema-gui
  echo -e "\n${GREEN}✓ Sema installed successfully!${RESET}"
else
  echo -e "\n${BOLD}[1/2] Cargo not found. Fetching latest release binary...${RESET}"
  RELEASES_URL="https://github.com/Nama-ResearchLab/Sema/releases/latest"
  echo -e "Please visit ${CYAN}${RELEASES_URL}${RESET} to download prebuilt binaries for $OS-$ARCH,"
  echo -e "or install Rust: ${BOLD}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${RESET}"
fi

echo -e "\n${BOLD}[2/2] Quick Start:${RESET}"
echo -e "  Run CLI:    ${CYAN}sema --help${RESET}"
echo -e "  Run Studio: ${CYAN}sema-gui${RESET}"
echo -e "  Run MCP:    ${CYAN}sema-gui --mcp${RESET}"
echo -e "\nDocumentation & Web Studio: ${CYAN}https://nama-researchlab.github.io/Sema/${RESET}\n"
