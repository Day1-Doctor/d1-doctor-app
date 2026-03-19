#!/bin/bash
# P1 Journey: Multi-Task
#
# Creates multiple tasks and verifies the task list renders correctly
# with multiple entries. Tests the Linear-style TaskView with
# task/sub-task hierarchy.
#
# Screenshots:
#   1. Empty or initial task list
#   2. After creating first task
#   3. After creating second task (multiple visible)
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p1-multi-task"
RESULTS_DIR="${1:?Usage: p1-multi-task.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Navigate to task list area
sleep 2
# Click on Tasks section in sidebar
click_at 150 250
sleep 1

# Screenshot 1: Initial task list state
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Initial task list screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Open chat to create a task
click_at 800 700
sleep 0.5
type_text "Analyze the performance metrics for Q1"
sleep 0.5
press_key
sleep 3

# Screenshot 2: After first task creation
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: First task screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Create a second task
click_at 800 700
sleep 0.5
type_text "Write a summary report of findings"
sleep 0.5
press_key
sleep 3

# Screenshot 3: After second task (multi-task view)
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Multi-task screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
