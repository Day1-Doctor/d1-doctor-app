#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> Day 1 Doctor — Local Dev Setup"
echo ""

# Check prerequisites
echo "==> Checking prerequisites..."
command -v cargo >/dev/null 2>&1 || { echo "ERROR: Rust/cargo not found. Install via rustup.rs"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "ERROR: Node.js not found. Install via nvm or nodejs.org"; exit 1; }
command -v uv >/dev/null 2>&1 || pip3 install uv 2>/dev/null || { echo "ERROR: uv not found. Run: pip3 install uv"; exit 1; }

# Create runtime dirs
echo "==> Creating runtime directories..."
mkdir -p ~/.d1doctor/logs

# Build Rust workspace
echo "==> Building Rust workspace (this may take a few minutes)..."
cd "$REPO_ROOT"
cargo build 2>&1 | tail -5

# Install TypeScript deps
echo "==> Installing TypeScript dependencies..."
cd "$REPO_ROOT/crates/desktop"
npm install --silent

echo ""
echo "✓ Setup complete! Run ./scripts/dev.sh to start the local stack."
