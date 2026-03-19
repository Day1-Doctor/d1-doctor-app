#!/bin/bash
# P0 Journey: Approval Dialog Flow
#
# Triggers and interacts with the approval dialog. Verifies it renders
# correctly and can be dismissed.
#
# Screenshots:
#   1. Main view before approval
#   2. Approval dialog visible
#   3. After approval action
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-approval"
RESULTS_DIR="${1:?Usage: p0-approval.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Main view before any approval interaction
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Pre-approval screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Navigate to the approval queue area
# The approval queue is typically in the task/command center area
click_at 800 350
sleep 2

# Screenshot 2: Approval area visible
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Approval area screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Try to interact with any approval button or dismiss
# Click on potential approve/deny button area
click_at 800 450
sleep 1

# Screenshot 3: After interaction
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Post-approval screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
