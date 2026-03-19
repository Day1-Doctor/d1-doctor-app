#!/usr/bin/env python3
"""Screenshot comparison utility for visual E2E tests.

Compares two screenshot files and determines if they are within an
acceptable tolerance. On first run (no baseline), copies the actual
screenshot as the new baseline.

Usage:
    python3 lib/diff.py <baseline> <actual> [tolerance_pct]

Exit codes:
    0 = PASS (within tolerance or baseline created)
    1 = FAIL (screenshots differ beyond tolerance)
"""

import sys
import shutil
from pathlib import Path


def compare_by_size(baseline: Path, actual: Path, tolerance: float) -> bool:
    """Compare screenshots by file size as a proxy metric.

    A simple heuristic: if the file sizes differ by more than `tolerance`%,
    the screenshots are considered different. This catches major layout
    changes without requiring PIL/Pillow.
    """
    b_size = baseline.stat().st_size
    a_size = actual.stat().st_size

    if b_size == 0:
        return True  # Empty baseline — treat as first run

    diff_pct = abs(b_size - a_size) / b_size * 100
    return diff_pct < tolerance


def compare_by_pixels(baseline: Path, actual: Path, tolerance: float) -> bool:
    """Compare screenshots pixel-by-pixel using PIL (if available).

    Falls back to size comparison if PIL is not installed.
    """
    try:
        from PIL import Image
        import math

        img_b = Image.open(baseline).convert("RGB")
        img_a = Image.open(actual).convert("RGB")

        # If dimensions differ, screenshots are different
        if img_b.size != img_a.size:
            return False

        pixels_b = list(img_b.getdata())
        pixels_a = list(img_a.getdata())

        total = len(pixels_b)
        diff_count = 0
        for pb, pa in zip(pixels_b, pixels_a):
            # Consider a pixel different if any channel differs by > 10
            if any(abs(b - a) > 10 for b, a in zip(pb, pa)):
                diff_count += 1

        diff_pct = (diff_count / total) * 100
        return diff_pct < tolerance

    except ImportError:
        # PIL not available — fall back to size comparison
        return compare_by_size(baseline, actual, tolerance)


def main():
    if len(sys.argv) < 3:
        print("Usage: diff.py <baseline> <actual> [tolerance_pct]")
        sys.exit(2)

    baseline = Path(sys.argv[1])
    actual = Path(sys.argv[2])
    tolerance = float(sys.argv[3]) if len(sys.argv) > 3 else 5.0

    if not actual.exists():
        print(f"FAIL: actual screenshot not found: {actual}")
        sys.exit(1)

    if not baseline.exists():
        # First run — save actual as baseline
        baseline.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(actual, baseline)
        print(f"BASELINE CREATED: {baseline}")
        sys.exit(0)

    # Compare
    if compare_by_pixels(baseline, actual, tolerance):
        print(f"PASS: within {tolerance}% tolerance")
        sys.exit(0)
    else:
        # Save a diff copy for review
        diff_path = actual.with_suffix(".diff.png")
        shutil.copy2(actual, diff_path)
        print(f"FAIL: screenshot differs beyond {tolerance}% tolerance")
        print(f"  Baseline: {baseline}")
        print(f"  Actual:   {actual}")
        print(f"  Diff:     {diff_path}")
        sys.exit(1)


if __name__ == "__main__":
    main()
