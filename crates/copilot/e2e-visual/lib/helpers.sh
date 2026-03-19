#!/bin/bash
# Common helper functions for visual journey scripts.
#
# Source this file at the top of each journey:
#   source "$(dirname "$0")/../lib/helpers.sh"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR"
BASELINES_DIR="$(cd "$SCRIPT_DIR/../baselines" && pwd)"

# Exit code 77 = SKIP (conventionally used by test frameworks)
EXIT_SKIP=77

# Check if the app is running or can be launched
ensure_app_running() {
  if pgrep -x "d1-copilot" >/dev/null 2>&1 || pgrep -x "d1_copilot" >/dev/null 2>&1; then
    return 0
  fi

  # Try to launch
  local result
  result=$(osascript "$LIB_DIR/launch.applescript" 2>/dev/null) || true
  if [[ "$result" == "launched" ]]; then
    sleep 3
    return 0
  fi

  echo "App not available — skipping" >&2
  return $EXIT_SKIP
}

# Take a screenshot and save to results directory
# Usage: take_screenshot <results_dir> <screenshot_name>
take_screenshot() {
  local results_dir="$1"
  local name="$2"
  bash "$LIB_DIR/screenshot.sh" "$results_dir/${name}.png"
}

# Compare screenshot against baseline
# Usage: compare_baseline <journey_name> <screenshot_name> <results_dir> [tolerance]
compare_baseline() {
  local journey="$1"
  local name="$2"
  local results_dir="$3"
  local tolerance="${4:-5.0}"

  python3 "$LIB_DIR/diff.py" \
    "$BASELINES_DIR/${journey}-${name}.png" \
    "$results_dir/${journey}-${name}.png" \
    "$tolerance"
}

# Click at coordinates
# Usage: click_at <x> <y>
click_at() {
  osascript "$LIB_DIR/click.applescript" "$1" "$2" 2>/dev/null || true
}

# Type text
# Usage: type_text "hello world"
type_text() {
  osascript "$LIB_DIR/type-text.applescript" "$1" 2>/dev/null || true
}

# Press a key using System Events
# Usage: press_key "return"
press_key() {
  osascript -e "tell application \"System Events\" to keystroke return" 2>/dev/null || true
}

# Wait for the app window to be frontmost
wait_for_window() {
  local max_wait="${1:-5}"
  local elapsed=0
  while [ $elapsed -lt $max_wait ]; do
    local front
    front=$(osascript -e 'tell application "System Events" to name of first application process whose frontmost is true' 2>/dev/null) || true
    if [[ "$front" == *"copilot"* ]] || [[ "$front" == *"Copilot"* ]] || [[ "$front" == *"Day1"* ]]; then
      return 0
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done
  echo "Warning: app window not frontmost after ${max_wait}s" >&2
  return 0  # Non-fatal
}
