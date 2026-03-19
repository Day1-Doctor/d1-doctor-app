#!/bin/bash
# P1 Journey: Metrics Panel
#
# Opens the metrics/costs panel in the sidebar and verifies
# it displays DD credit information and usage data.
#
# Screenshots:
#   1. Sidebar with metrics section collapsed
#   2. Metrics section expanded
#   3. Cost details visible
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-metrics"
RESULTS_DIR="${1:?Usage: p1-metrics.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Screenshot 1: Sidebar visible (metrics section collapsed)
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Sidebar screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Click on the Metrics accordion section in the sidebar
# Metrics is the 4th section (Valley, Tasks, Workspace, Metrics)
click_at 150 550
sleep 1

# Screenshot 2: Metrics section expanded
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Metrics expanded screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Click on a cost detail area to expand further
click_at 150 600
sleep 1

# Screenshot 3: Cost details
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Cost details screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
