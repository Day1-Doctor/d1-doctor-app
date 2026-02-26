#!/bin/bash
# Day 1 Doctor Installer Script
# 
# Usage: curl -fsSL https://install.day1doctor.com | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${HOME}/.local/bin"
DAEMON_SERVICE_DIR="${HOME}/.config/systemd/user"
RELEASE_API="https://api.github.com/repos/day1doctor/d1-doctor-app/releases/latest"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

case "$OS" in
    linux) OS="linux" ;;
    darwin) OS="macos" ;;
    *) echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

echo -e "${GREEN}Day 1 Doctor Installer${NC}"
echo "OS: $OS, Architecture: $ARCH"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Fetch latest release
echo -e "${YELLOW}Fetching latest release...${NC}"
RELEASE_INFO=$(curl -s "$RELEASE_API")
DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep "browser_download_url.*d1-doctor-${OS}-${ARCH}" | cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Failed to find release for ${OS}-${ARCH}${NC}"
    exit 1
fi

echo -e "${YELLOW}Downloading from: $DOWNLOAD_URL${NC}"
curl -fL "$DOWNLOAD_URL" -o "$INSTALL_DIR/d1-doctor"
chmod +x "$INSTALL_DIR/d1-doctor"

# Verify installation
if ! "$INSTALL_DIR/d1-doctor" --version >/dev/null 2>&1; then
    echo -e "${RED}Installation failed - binary check failed${NC}"
    exit 1
fi

echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Ensure $INSTALL_DIR is in your PATH"
echo "  2. Run: d1-doctor daemon start"
echo "  3. Run: d1-doctor auth login"
echo ""
