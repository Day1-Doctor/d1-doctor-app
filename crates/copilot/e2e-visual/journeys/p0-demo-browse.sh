#!/bin/bash
# P0 Journey: Demo Browse
#
# Navigates through all major views without authentication.
# Verifies that the Cowork Valley, Office, and sidebar all render.
#
# Screenshots:
#   1. Main valley view
#   2. Sidebar expanded
#   3. Office/workspace view
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-demo-browse"
RESULTS_DIR="${1:?Usage: p0-demo-browse.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Main valley view (initial state after load)
sleep 3
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Valley view screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Navigate to sidebar — click on the left side area
# Approximate coordinates for sidebar toggle (top-left area)
click_at 30 400
sleep 1

# Screenshot 2: Sidebar state
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Sidebar screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Navigate to office/workspace — click on a parcel or workspace area
# Approximate coordinates for center workspace area
click_at 600 400
sleep 2

# Screenshot 3: Workspace/office view
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Workspace screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
