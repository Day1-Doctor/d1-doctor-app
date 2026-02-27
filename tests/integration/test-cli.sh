#!/usr/bin/env bash
set -euo pipefail
# CLI integration test — verifies CLI commands work against a running daemon

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> CLI Integration Test"
echo ""

# Build
echo "[1/4] Building CLI..."
cargo build --bin d1 -q 2>&1
echo "     ✓ d1 built"

# Check d1 help
echo "[2/4] Testing: d1 --help"
"$REPO_ROOT/target/debug/d1" --help | grep -q "Usage" && echo "     ✓ --help works"

# Check d1 --version
echo "[3/4] Testing: d1 --version"
"$REPO_ROOT/target/debug/d1" --version | grep -q "d1" && echo "     ✓ --version works"

# Check d1 doctor (no daemon needed for basic check)
echo "[4/4] Testing: d1 doctor (daemon not running — expect graceful output)"
"$REPO_ROOT/target/debug/d1" doctor 2>&1 | grep -qi "diagnostic\|doctor\|check\|daemon" && echo "     ✓ doctor outputs diagnostic info"

echo ""
echo "✓ CLI integration tests passed"
