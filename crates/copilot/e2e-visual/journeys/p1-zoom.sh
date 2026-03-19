#!/bin/bash
# P1 Journey: Display Zoom
#
# Tests the display zoom feature (80-150%, 16px base font).
# Uses keyboard shortcuts or UI controls to change zoom level.
#
# Screenshots:
#   1. Default zoom (100%)
#   2. Zoomed in (125% or 150%)
#   3. Zoomed out (80%)
#   4. Back to default
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-zoom"
RESULTS_DIR="${1:?Usage: p1-zoom.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Default zoom level
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Default zoom screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Zoom in using Cmd+= (standard zoom shortcut)
osascript -e 'tell application "System Events" to keystroke "=" using command down' 2>/dev/null || true
sleep 0.5
osascript -e 'tell application "System Events" to keystroke "=" using command down' 2>/dev/null || true
sleep 1

# Screenshot 2: Zoomed in
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Zoomed in screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Zoom out past default using Cmd+-
osascript -e 'tell application "System Events" to keystroke "-" using command down' 2>/dev/null || true
sleep 0.3
osascript -e 'tell application "System Events" to keystroke "-" using command down' 2>/dev/null || true
sleep 0.3
osascript -e 'tell application "System Events" to keystroke "-" using command down' 2>/dev/null || true
sleep 0.3
osascript -e 'tell application "System Events" to keystroke "-" using command down' 2>/dev/null || true
sleep 1

# Screenshot 3: Zoomed out
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Zoomed out screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

# Reset to default zoom (Cmd+0)
osascript -e 'tell application "System Events" to keystroke "0" using command down' 2>/dev/null || true
sleep 1

# Screenshot 4: Back to default
take_screenshot "$RESULTS_DIR" "${JOURNEY}-04"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-04.png" ]; then
  echo "FAIL: Reset zoom screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "04" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
