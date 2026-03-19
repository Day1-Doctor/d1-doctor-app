#!/bin/bash
# P0 Journey: Cold Start
#
# Launches the application from scratch and verifies it renders correctly.
# Captures the initial state screenshot and compares against baseline.
#
# Screenshots:
#   1. Initial window after launch (loading/splash)
#   2. Main view after load completes
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-cold-start"
RESULTS_DIR="${1:?Usage: p0-cold-start.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

# Ensure app is running (or skip)
ensure_app_running || exit $EXIT_SKIP

# Wait for the window to appear
wait_for_window 8

# Screenshot 1: Initial render
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"

# Verify screenshot has content (non-zero file size)
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Screenshot is empty" >&2
  exit 1
fi

# Compare against baseline (creates baseline on first run)
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Screenshot 2: After full load (wait a bit more)
sleep 3
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"

if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Second screenshot is empty" >&2
  exit 1
fi

compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
