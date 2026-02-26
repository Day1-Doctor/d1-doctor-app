#!/usr/bin/env bash
set -euo pipefail
APP_REPO="$(cd "$(dirname "$0")/.." && pwd)"
PROTO_REPO="$(cd "$APP_REPO/../../../d1-doctor-proto/.worktrees/sprint1" && pwd)"
echo "Syncing proto files from $PROTO_REPO → $APP_REPO/proto/"
rsync -av --delete "$PROTO_REPO/proto/" "$APP_REPO/proto/"
echo "Done."
