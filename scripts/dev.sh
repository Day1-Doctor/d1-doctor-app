#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}==> Day 1 Doctor — Local Dev Stack${NC}"
echo ""

# Trap to clean up background processes on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}==> Stopping services...${NC}"
    kill $(jobs -p) 2>/dev/null || true
    wait 2>/dev/null || true
    echo "==> Done"
}
trap cleanup EXIT INT TERM

# Start daemon in background
echo -e "${GREEN}[1/2] Starting daemon (d1d) on port 9876...${NC}"
cd "$REPO_ROOT"
cargo run --bin d1-daemon -- --port 9876 &
DAEMON_PID=$!
sleep 2

# Check daemon is up
if ! nc -z localhost 9876 2>/dev/null; then
    echo "ERROR: daemon failed to start on port 9876"
    exit 1
fi
echo "     ✓ Daemon running (PID $DAEMON_PID)"
echo ""

# Start Mac client (Tauri dev)
echo -e "${GREEN}[2/2] Starting Mac client (Tauri dev)...${NC}"
cd "$REPO_ROOT/crates/desktop"
npm run tauri dev &

echo ""
echo -e "${GREEN}✓ Local stack running. Press Ctrl+C to stop all services.${NC}"

# Wait for all background jobs
wait
