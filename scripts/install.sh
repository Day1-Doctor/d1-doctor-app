#!/bin/bash
# Day1 Doctor CLI Installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Day1-Doctor/d1-doctor-app/main/scripts/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/Day1-Doctor/d1-doctor-app/main/scripts/install.sh | sh -s -- --version v0.2.0
#
# Environment variables:
#   D1_INSTALL_DIR  — Override install directory (default: ~/.local/bin)
#   D1_VERSION      — Install a specific version tag (default: latest)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

REPO="Day1-Doctor/d1-doctor-app"
BINARY_NAME="d1-doctor-cli"
INSTALL_DIR="${D1_INSTALL_DIR:-${HOME}/.local/bin}"

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --version) D1_VERSION="$2"; shift 2 ;;
        --dir)     INSTALL_DIR="$2"; shift 2 ;;
        --help)
            echo "Usage: install.sh [--version VERSION] [--dir INSTALL_DIR]"
            exit 0
            ;;
        *) echo -e "${RED}Unknown option: $1${NC}"; exit 1 ;;
    esac
done

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)  OS_LABEL="linux" ;;
    darwin) OS_LABEL="macos" ;;
    *)      echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH_LABEL="x86_64" ;;
    aarch64|arm64)   ARCH_LABEL="arm64" ;;
    *)               echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

# macOS: prefer universal binary
if [ "$OS_LABEL" = "macos" ]; then
    ARTIFACT="d1-macos-universal"
else
    ARTIFACT="d1-${OS_LABEL}-${ARCH_LABEL}"
fi

echo -e "${CYAN}Day1 Doctor CLI Installer${NC}"
echo "  Platform: ${OS_LABEL}/${ARCH_LABEL}"
echo "  Artifact: ${ARTIFACT}"
echo ""

# Resolve version
if [ -z "$D1_VERSION" ]; then
    echo -e "${YELLOW}Resolving latest release...${NC}"
    D1_VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    if [ -z "$D1_VERSION" ]; then
        echo -e "${RED}Failed to determine latest version.${NC}"
        echo "Try specifying a version: install.sh --version v0.1.0"
        exit 1
    fi
fi
echo "  Version:  ${D1_VERSION}"
echo ""

# Build download URL
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${D1_VERSION}/${ARTIFACT}.tar.gz"

# Download
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo -e "${YELLOW}Downloading ${DOWNLOAD_URL}${NC}"
HTTP_CODE=$(curl -fSL -w '%{http_code}' -o "${TMPDIR}/${ARTIFACT}.tar.gz" "$DOWNLOAD_URL" 2>/dev/null) || true

if [ "$HTTP_CODE" != "200" ] && [ ! -s "${TMPDIR}/${ARTIFACT}.tar.gz" ]; then
    echo -e "${RED}Download failed (HTTP ${HTTP_CODE}).${NC}"
    echo "Check that release ${D1_VERSION} exists and has artifact ${ARTIFACT}.tar.gz"
    exit 1
fi

# Extract
echo -e "${YELLOW}Extracting...${NC}"
tar xzf "${TMPDIR}/${ARTIFACT}.tar.gz" -C "${TMPDIR}"

# Install
mkdir -p "$INSTALL_DIR"
mv "${TMPDIR}/${BINARY_NAME}" "${INSTALL_DIR}/d1"
chmod +x "${INSTALL_DIR}/d1"

# Verify
if "${INSTALL_DIR}/d1" --version >/dev/null 2>&1; then
    INSTALLED_VERSION=$("${INSTALL_DIR}/d1" --version 2>&1 || true)
    echo ""
    echo -e "${GREEN}Installed successfully!${NC}"
    echo "  Binary:  ${INSTALL_DIR}/d1"
    echo "  Version: ${INSTALLED_VERSION}"
else
    echo ""
    echo -e "${GREEN}Binary installed to ${INSTALL_DIR}/d1${NC}"
    echo "  (version check skipped — binary may require runtime dependencies)"
fi

# PATH check
case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        echo -e "${YELLOW}NOTE:${NC} ${INSTALL_DIR} is not in your PATH."
        echo "Add it by appending this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        ;;
esac

echo ""
echo "Get started:"
echo "  d1 --help"
echo "  d1 auth login"
echo "  d1 daemon start"
