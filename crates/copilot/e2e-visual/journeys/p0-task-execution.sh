#!/bin/bash
# P0 Journey: Task Execution
#
# Watches task progress after creation. Verifies agents animate
# and the task view updates.
#
# Screenshots:
#   1. Task list with running task
#   2. Agent in working state (mid-execution)
#   3. Task progress/timeline view
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-task-execution"
RESULTS_DIR="${1:?Usage: p0-task-execution.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Navigate to the task view / sidebar tasks section
sleep 2
# Click on Tasks section in sidebar (left side)
click_at 150 300
sleep 1

# Screenshot 1: Task list view
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Task list screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Wait for agents to potentially animate (if tasks are running)
sleep 3

# Screenshot 2: Agent working state (canvas area)
# Click on the main canvas/valley to see agents
click_at 500 400
sleep 1
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Agent state screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Screenshot 3: Capture task progress after a pause
sleep 2
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Task progress screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
