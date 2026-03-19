#!/bin/bash
# P1 Journey: Workspace View
#
# Opens and explores the workspace section. Verifies parcel grid
# renders (11 parcels, 3 states: running/empty/locked) and the
# 3D isometric office with work zone + rest zone.
#
# Screenshots:
#   1. Workspace section in sidebar
#   2. Parcel grid view
#   3. Office interior view
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-workspace"
RESULTS_DIR="${1:?Usage: p1-workspace.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Navigate to workspace section in sidebar
sleep 2
# Click on Workspace accordion (3rd section)
click_at 150 450
sleep 1

# Screenshot 1: Workspace section expanded
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Workspace section screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Click on a parcel in the main canvas area
click_at 500 350
sleep 2

# Screenshot 2: Parcel grid / campus view
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Parcel grid screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Click on a specific parcel to enter office view
click_at 500 300
sleep 2

# Screenshot 3: Office interior
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Office view screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
