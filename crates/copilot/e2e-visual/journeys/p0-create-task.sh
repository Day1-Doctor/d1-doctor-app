#!/bin/bash
# P0 Journey: Create Task
#
# Types a message in the chat input and confirms task creation.
# Verifies the chat panel renders and accepts input.
#
# Screenshots:
#   1. Chat panel visible with empty input
#   2. Text typed in chat input
#   3. After pressing Enter/submit
#
# Exit: 0 = pass, 1 = fail, 77 = skip

set -euo pipefail

JOURNEY="p0-create-task"
RESULTS_DIR="${1:?Usage: p0-create-task.sh <results-dir>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/helpers.sh"

mkdir -p "$RESULTS_DIR"

ensure_app_running || exit $EXIT_SKIP
wait_for_window 8

# Open the chat/command center (typically on the right side)
# Click on the chat toggle area
sleep 2
click_at 950 400
sleep 1

# Screenshot 1: Chat panel visible
take_screenshot "$RESULTS_DIR" "${JOURNEY}-01"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-01.png" ]; then
  echo "FAIL: Chat panel screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "01" "$RESULTS_DIR"

# Click on the chat input field (bottom of chat panel)
click_at 800 700
sleep 0.5

# Type a task description
type_text "Research the latest AI agent frameworks"
sleep 1

# Screenshot 2: Text typed in input
take_screenshot "$RESULTS_DIR" "${JOURNEY}-02"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-02.png" ]; then
  echo "FAIL: Typed text screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "02" "$RESULTS_DIR"

# Press Enter to submit
press_key
sleep 2

# Screenshot 3: After submission (task should appear)
take_screenshot "$RESULTS_DIR" "${JOURNEY}-03"
if [ ! -s "$RESULTS_DIR/${JOURNEY}-03.png" ]; then
  echo "FAIL: Post-submit screenshot empty" >&2
  exit 1
fi
compare_baseline "$JOURNEY" "03" "$RESULTS_DIR"

echo "Journey $JOURNEY completed successfully"
exit 0
