#!/bin/bash
# Datamorph — Universal Data Format Transformer
# One-command installer for Linux, macOS, and Windows (via Git Bash/WSL)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO="aksaayyy/Datamorph"
LATEST_RELEASE="v0.1.0"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="datamorph"

# Ensure install directory exists
mkdir -p "$INSTALL_DIR"

# Detect OS and architecture
detect_platform() {
  local os arch

  # OS detection
  case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
    linux*)   os="linux" ;;
    darwin*)  os="darwin" ;;
    mingw*|msys*|cygwin*) os="windows" ;;
    *) echo -e "${RED}❌ Unsupported OS: $(uname -s)${NC}"; exit 1 ;;
  esac

  # Architecture detection
  case "$(uname -m)" in
    x86_64|amd64)   arch="amd64" ;;
    arm64|aarch64)  arch="arm64" ;;
    i386|i686)      arch="386" ;;
    *) echo -e "${RED}❌ Unsupported architecture: $(uname -m)${NC}"; exit 1 ;;
  esac

  # For Windows, binary extension differs
  local ext=""
  if [ "$os" = "windows" ]; then
    ext=".exe"
  fi

  # Construct binary name
  local binary="datamorph-${os}-${arch}${ext}"

  echo "$binary"
}

# Download and install
install() {
  local binary="$1"
  local download_url="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/${binary}"
  local install_path="${INSTALL_DIR}/${BINARY_NAME}"

  echo -e "${CYAN}📦 Installing Datamorph v0.1.0...${NC}"
  echo -e "   Platform: ${GREEN}$binary${NC}"
  echo -e "   Installing to: ${YELLOW}${install_path}${NC}"

  # Check if curl/wget available
  if command -v curl &>/dev/null; then
    echo "⬇️  Downloading..."
    if ! curl -fsSL "$download_url" -o "$install_path"; then
      echo -e "${RED}❌ Download failed!${NC}"
      echo "   URL: $download_url"
      echo "   Make sure the release asset exists for your platform."
      exit 1
    fi
  elif command -v wget &>/dev/null; then
    echo "⬇️  Downloading..."
    if ! wget -qO "$install_path" "$download_url"; then
      echo -e "${RED}❌ Download failed!${NC}"
      exit 1
    fi
  else
    echo -e "${RED}❌ Neither curl nor wget found. Please install one and retry.${NC}"
    exit 1
  fi

  # Make executable
  chmod +x "$install_path"

  # Verify installation
  if [ -x "$install_path" ]; then
    echo -e "${GREEN}✅ Installed successfully!${NC}"
    echo ""
    echo "📋 Next steps:"

    # Check if INSTALL_DIR is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
      echo -e "   ${YELLOW}⚠️  ${INSTALL_DIR} is not in your PATH.${NC}"
      echo ""
      echo "   Add it to your shell profile:"
      case "$SHELL" in
        *bash)  echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc" ;;
        *zsh)   echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc" ;;
        *fish)  echo "   set -U fish_user_paths \$HOME/.local/bin \$fish_user_paths" ;;
        *)      echo "   export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
      esac
      echo ""
      echo "   Then reload your shell or run:"
      echo "   source ~/.bashrc  # or ~/.zshrc"
    else
      echo "   ${GREEN}✓ ${INSTALL_DIR} is already in your PATH${NC}"
    fi

    echo ""
    echo "   Test it:"
    echo "   ${CYAN}datamorph --version${NC}"
    echo "   ${CYAN}datamorph --help${NC}"
  else
    echo -e "${RED}❌ Installation failed — cannot execute binary${NC}"
    exit 1
  fi
}

# Main
main() {
  echo -e "${CYAN}"
  echo "   ___    _     _             _       "
  echo "  / _ \  | |   | |           (_)      "
  echo " / /_\ \ | |__ | |__    ___   _  __ _ "
  echo "|  _  | | '_ \| '_ \  / _ \ | |/ _` |"
  echo "| | | | | |_) | |_) || (_) || | (_| |"
  echo "\_| |_/ |_.__/|_.__/  \___/ | |\__,_|"
  echo "                            _/ |      "
  echo "                           |__/       "
  echo -e "${NC}"

  binary=$(detect_platform)
  install "$binary"
}

main "$@"
