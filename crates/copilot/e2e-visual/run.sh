#!/bin/bash
# Visual E2E Test Runner for Day1 Copilot.
#
# Runs P0 (release-blocking) and P1 (advisory) visual journey tests.
# Each journey captures screenshots and optionally compares against baselines.
#
# Usage:
#   bash run.sh                    # Run all journeys
#   bash run.sh --p0-only          # Run only P0 journeys
#   bash run.sh --results-dir DIR  # Custom results directory

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="${1:-$SCRIPT_DIR/results/$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$RESULTS_DIR"

# Parse arguments
P0_ONLY=false
for arg in "$@"; do
  case "$arg" in
    --p0-only) P0_ONLY=true ;;
    --results-dir)
      shift
      RESULTS_DIR="$1"
      mkdir -p "$RESULTS_DIR"
      ;;
  esac
done

P0_PASS=0
P0_FAIL=0
P1_PASS=0
P1_FAIL=0
P0_SKIP=0
P1_SKIP=0

echo "========================================"
echo "  Day1 Copilot Visual E2E Tests"
echo "========================================"
echo "Results: $RESULTS_DIR"
echo ""

# Check if the app is available
APP_RUNNING=false
if pgrep -x "d1-copilot" >/dev/null 2>&1 || pgrep -x "d1_copilot" >/dev/null 2>&1; then
  APP_RUNNING=true
  echo "Application detected as running."
elif [ -f "$SCRIPT_DIR/../../target/debug/d1-copilot" ] || [ -f "$SCRIPT_DIR/../../target/release/d1-copilot" ]; then
  echo "Binary found but not running. Journeys will attempt to launch."
else
  echo "No binary found. Journeys that require the app will be SKIPPED."
fi
echo ""

# --- Run P0 journeys (block release) ---
echo "--- P0 Journeys (Release-Blocking) ---"
P0_COUNT=0
for journey in "$SCRIPT_DIR"/journeys/p0-*.sh; do
  [ -f "$journey" ] || continue
  P0_COUNT=$((P0_COUNT + 1))
  name=$(basename "$journey" .sh)
  echo -n "  $name... "
  if bash "$journey" "$RESULTS_DIR" 2>"$RESULTS_DIR/${name}.stderr"; then
    echo "PASS"
    P0_PASS=$((P0_PASS + 1))
  else
    exit_code=$?
    if [ $exit_code -eq 77 ]; then
      echo "SKIP (app not available)"
      P0_SKIP=$((P0_SKIP + 1))
    else
      echo "FAIL (see $RESULTS_DIR/${name}.stderr)"
      P0_FAIL=$((P0_FAIL + 1))
    fi
  fi
done

if [ $P0_COUNT -eq 0 ]; then
  echo "  (no P0 journeys found)"
fi
echo ""

# --- Run P1 journeys (advisory) ---
if [ "$P0_ONLY" = false ]; then
  echo "--- P1 Journeys (Advisory) ---"
  P1_COUNT=0
  for journey in "$SCRIPT_DIR"/journeys/p1-*.sh; do
    [ -f "$journey" ] || continue
    P1_COUNT=$((P1_COUNT + 1))
    name=$(basename "$journey" .sh)
    echo -n "  $name... "
    if bash "$journey" "$RESULTS_DIR" 2>"$RESULTS_DIR/${name}.stderr"; then
      echo "PASS"
      P1_PASS=$((P1_PASS + 1))
    else
      exit_code=$?
      if [ $exit_code -eq 77 ]; then
        echo "SKIP (app not available)"
        P1_SKIP=$((P1_SKIP + 1))
      else
        echo "ADVISORY FAIL (see $RESULTS_DIR/${name}.stderr)"
        P1_FAIL=$((P1_FAIL + 1))
      fi
    fi
  done

  if [ $P1_COUNT -eq 0 ]; then
    echo "  (no P1 journeys found)"
  fi
  echo ""
fi

# --- Summary ---
echo "========================================"
echo "  Summary"
echo "========================================"
echo "  P0: $P0_PASS pass, $P0_FAIL fail, $P0_SKIP skip (BLOCKS RELEASE)"
if [ "$P0_ONLY" = false ]; then
  echo "  P1: $P1_PASS pass, $P1_FAIL fail, $P1_SKIP skip (advisory)"
fi
TOTAL=$((P0_PASS + P0_FAIL + P0_SKIP + P1_PASS + P1_FAIL + P1_SKIP))
echo "  TOTAL: $TOTAL journeys"
echo "  Screenshots: $RESULTS_DIR/"
echo "========================================"

# Exit code: 0 if no P0 failures, 1 if any P0 failed
[ $P0_FAIL -eq 0 ] && exit 0 || exit 1
