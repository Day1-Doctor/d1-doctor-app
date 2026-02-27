#!/usr/bin/env bash
# scripts/release-mac.sh — Build, properly sign, and package DMG for macOS
# Usage: ./scripts/release-mac.sh [version]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DESKTOP_DIR="$SCRIPT_DIR/../crates/desktop"
VERSION="${1:-$(node -p "require('$DESKTOP_DIR/package.json').version")}"

echo "🏗  Building Day 1 Doctor v$VERSION for macOS..."

# 1. Build with Tauri
cd "$DESKTOP_DIR"
export PATH="$HOME/.cargo/bin:$PATH"
npx tauri build

BUNDLE="$DESKTOP_DIR/src-tauri/target/release/bundle"
APP="$BUNDLE/macos/Day 1 Doctor.app"
DMG_DIR="$BUNDLE/dmg"
OUTPUT_DMG="$DMG_DIR/Day 1 Doctor_${VERSION}_aarch64.dmg"

# 2. Fix Info.plist: remove legacy LSRequiresCarbon key (added by Tauri template)
if /usr/libexec/PlistBuddy -c "Print :LSRequiresCarbon" "$APP/Contents/Info.plist" &>/dev/null; then
  /usr/libexec/PlistBuddy -c "Delete :LSRequiresCarbon" "$APP/Contents/Info.plist"
  echo "✅ Removed LSRequiresCarbon from Info.plist"
fi

# 3. Re-sign with --deep to properly seal Resources and bind Info.plist
codesign --force --deep --sign - "$APP"
echo "✅ Re-signed app bundle (ad-hoc, resources sealed)"

# 4. Verify signature
codesign -dv "$APP" 2>&1 | grep -E "Signature|Sealed Resources|Info.plist"

# 5. Build DMG with Applications shortcut
rm -f "$OUTPUT_DMG"
create-dmg \
  --volname "Day 1 Doctor $VERSION" \
  --volicon "$DMG_DIR/Day 1 Doctor.icns" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "Day 1 Doctor.app" 175 190 \
  --hide-extension "Day 1 Doctor.app" \
  --app-drop-link 425 190 \
  --skip-jenkins \
  --no-internet-enable \
  "$OUTPUT_DMG" \
  "$APP"

echo "✅ DMG created: $OUTPUT_DMG"
echo "📦 Size: $(du -sh "$OUTPUT_DMG" | cut -f1)"
