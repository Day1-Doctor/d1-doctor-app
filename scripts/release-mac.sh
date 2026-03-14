#!/bin/bash
# Day1 Doctor — macOS Release Builder
#
# Builds the Mac DMG (via Tauri) and CLI binary for a given version tag.
#
# Usage:
#   ./scripts/release-mac.sh <version>
#
# Example:
#   ./scripts/release-mac.sh 2.6.0
#
# Outputs (in dist/):
#   - Day1 Doctor_<version>_aarch64.dmg
#   - d1-macos-universal.tar.gz  (CLI binary tarball)

set -euo pipefail

VERSION="${1:?Usage: $0 <version>  (e.g. 2.6.0)}"
TAG="v${VERSION}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"
ARCH="$(uname -m)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Day1 Doctor macOS Release Builder${NC}"
echo "  Version:  ${VERSION}"
echo "  Tag:      ${TAG}"
echo "  Arch:     ${ARCH}"
echo ""

# Validate architecture
if [ "$ARCH" != "arm64" ] && [ "$ARCH" != "aarch64" ]; then
    echo -e "${YELLOW}Warning: Building on ${ARCH}. DMG will be for this architecture.${NC}"
fi

mkdir -p "$DIST_DIR"

# ── Step 1: Build CLI binary ──────────────────────────────────────────────────
echo -e "${YELLOW}[1/3] Building CLI binary (release)...${NC}"
cargo build --release -p d1-doctor-cli

CLI_BIN="${REPO_ROOT}/target/release/d1-doctor-cli"
if [ ! -f "$CLI_BIN" ]; then
    echo -e "${RED}CLI binary not found at ${CLI_BIN}${NC}"
    exit 1
fi

echo -e "${GREEN}  CLI binary: $(du -h "$CLI_BIN" | cut -f1) — ${CLI_BIN}${NC}"

# Package CLI as tarball (matches install.sh expectations)
echo -e "${YELLOW}[2/3] Packaging CLI tarball...${NC}"
TARBALL="${DIST_DIR}/d1-macos-universal.tar.gz"
tar -czf "$TARBALL" -C "$(dirname "$CLI_BIN")" "$(basename "$CLI_BIN")"
echo -e "${GREEN}  Tarball: $(du -h "$TARBALL" | cut -f1) — ${TARBALL}${NC}"

# ── Step 2: Build Tauri DMG ───────────────────────────────────────────────────
echo -e "${YELLOW}[3/3] Building Tauri DMG...${NC}"
cd "${REPO_ROOT}/crates/desktop"

# Install frontend dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "  Installing frontend dependencies..."
    npm ci
fi

# Build via Tauri CLI
npx tauri build --target aarch64-apple-darwin 2>&1 | tail -5

# Locate DMG
DMG_DIR="${REPO_ROOT}/crates/desktop/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg"
DMG_FILE=$(find "$DMG_DIR" -name '*.dmg' -print -quit 2>/dev/null || true)

if [ -n "$DMG_FILE" ]; then
    cp "$DMG_FILE" "$DIST_DIR/"
    echo -e "${GREEN}  DMG: $(du -h "$DMG_FILE" | cut -f1) — copied to dist/${NC}"
else
    echo -e "${YELLOW}  DMG not found in ${DMG_DIR} — Tauri build may have failed.${NC}"
    echo -e "${YELLOW}  The CLI tarball is still available in dist/.${NC}"
fi

cd "$REPO_ROOT"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}Build artifacts in ${DIST_DIR}/:${NC}"
ls -lh "$DIST_DIR/"
echo ""
echo -e "${GREEN}Done!${NC} To create a GitHub release:"
echo "  gh release create ${TAG} dist/* --title 'Day1 Doctor ${TAG}' --draft"
