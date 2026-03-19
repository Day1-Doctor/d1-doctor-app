#!/bin/bash
# Capture a screenshot of the entire screen (or frontmost window).
#
# Usage: bash lib/screenshot.sh <output-path>
#
# Uses macOS screencapture. The -x flag suppresses the shutter sound.
# The -o flag captures the frontmost window without shadow.

set -euo pipefail

OUTPUT="${1:?Usage: screenshot.sh <output-path>}"
OUTPUT_DIR="$(dirname "$OUTPUT")"
mkdir -p "$OUTPUT_DIR"

# Capture the screen (full screen mode for reliability)
# -x = no sound, -C = include cursor (optional)
screencapture -x "$OUTPUT" 2>/dev/null

if [ -s "$OUTPUT" ]; then
  echo "Screenshot saved: $OUTPUT"
  exit 0
else
  echo "Screenshot failed: $OUTPUT" >&2
  exit 1
fi
