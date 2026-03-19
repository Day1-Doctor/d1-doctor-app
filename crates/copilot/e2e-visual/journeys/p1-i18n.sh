#!/bin/bash
# P1 Journey: Internationalization (i18n)
#
# Toggles between EN and CN locales and captures screenshots
# to verify translations render correctly.
#
# Screenshots:
#   1. English locale (default)
#   2. Chinese locale (after toggle)
#   3. Back to English
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-i18n"
RESULTS_DIR="${1:?Usage: p1-i18n.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: English locale (default)
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: EN locale screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Open settings/language toggle (Developer section in sidebar)
# Navigate to sidebar bottom area where locale toggle would be
click_at 150 650
sleep 1

# Look for the language toggle button and click it
click_at 150 700
sleep 2

# Screenshot 2: After language toggle (potentially CN)
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: CN locale screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Toggle back to English
click_at 150 700
sleep 2

# Screenshot 3: Back to English
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Restored EN screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
