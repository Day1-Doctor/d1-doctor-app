#!/bin/bash
# P0 Journey: Auth Flow
#
# Opens and closes the authentication panel. Verifies the auth wall
# renders properly with Google OAuth and Email options.
#
# Screenshots:
#   1. Before auth panel opens
#   2. Auth panel visible
#   3. After closing auth panel
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-auth-flow"
RESULTS_DIR="${1:?Usage: p0-auth-flow.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Initial state (before opening auth)
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Pre-auth screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Click on the auth/login area (typically top-right corner)
# The auth button is usually in the header/sidebar area
click_at 950 50
sleep 2

# Screenshot 2: Auth panel visible
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Auth panel screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Close the auth panel — press Escape or click outside
osascript -e 'tell application "System Events" to key code 53' 2>/dev/null || true
sleep 1

# Screenshot 3: After closing auth
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Post-auth screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
