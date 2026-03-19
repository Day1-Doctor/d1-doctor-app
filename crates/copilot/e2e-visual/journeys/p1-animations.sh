#!/bin/bash
# P1 Journey: Agent Animations
#
# Captures agents in different animation states to verify pixel-art
# sprites render correctly. The 6 agents have 9 animation states
# with mood bubbles.
#
# Screenshots:
#   1. Agents in idle state (valley view)
#   2. Zoomed into agent area
#   3. After triggering activity (agents may animate)
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-animations"
RESULTS_DIR="${1:?Usage: p1-animations.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Valley view showing agents in idle state
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Idle agents screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Click on the agent area to zoom in / select an agent
click_at 400 350
sleep 1

# Screenshot 2: Focused agent area
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Agent focus screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Wait for potential animation cycles
sleep 3

# Screenshot 3: After animation cycle
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Animation cycle screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
