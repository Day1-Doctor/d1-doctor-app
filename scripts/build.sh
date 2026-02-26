#!/bin/bash
# Day 1 Doctor Build Script
#
# Builds all crates for current platform
# Usage: ./scripts/build.sh [release]

set -e

PROFILE="${1:-debug}"
RELEASE_FLAG=""

if [ "$PROFILE" = "release" ]; then
    RELEASE_FLAG="--release"
fi

echo "Building Day 1 Doctor ($PROFILE mode)"

# Build all crates
cargo build --workspace $RELEASE_FLAG

# Run tests
echo "Running tests..."
cargo test --workspace

# Check formatting
echo "Checking formatting..."
cargo fmt --all -- --check

# Run clippy
echo "Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings

echo "Build complete!"

# Display build artifacts
BUILD_DIR="target/$PROFILE"
echo ""
echo "Build artifacts:"
echo "  Daemon:  $BUILD_DIR/d1-doctor-daemon"
echo "  CLI:     $BUILD_DIR/d1-doctor-cli"
echo "  SDK:     $BUILD_DIR/libd1_doctor_sdk.so (or .dylib)"

exit 0
