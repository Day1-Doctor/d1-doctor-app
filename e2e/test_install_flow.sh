#!/usr/bin/env bash
# e2e/test_install_flow.sh
# E2E smoke test: binary existence, help output, CLI structure validation
# Does NOT require a running daemon for the help/version tests.
# Requires built binaries for daemon lifecycle tests (Tests 5+).
# Usage: bash e2e/test_install_flow.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DAEMON_BIN="${DAEMON_BIN:-$ROOT_DIR/target/debug/d1-daemon}"
CLI_BIN="${CLI_BIN:-$ROOT_DIR/target/debug/d1-doctor}"
DAEMON_PORT="${DAEMON_PORT:-3030}"
DAEMON_PID=""
PASS_COUNT=0
FAIL_COUNT=0

log()  { printf '\033[1;34m[E2E]\033[0m %s\n' "$*"; }
pass() { printf '\033[1;32m[PASS]\033[0m %s\n' "$*"; ((PASS_COUNT++)); }
fail() { printf '\033[1;31m[FAIL]\033[0m %s\n' "$*"; ((FAIL_COUNT++)); }

assert_contains() {
  local label="$1" haystack="$2" needle="$3"
  if echo "$haystack" | grep -qF "$needle"; then
    pass "$label"
  else
    fail "$label — expected '$needle'"
    printf '  Got: %s\n' "$haystack"
  fi
}

wait_for_port() {
  local port="$1" timeout="${2:-10}" elapsed=0
  while ! nc -z 127.0.0.1 "$port" 2>/dev/null; do
    sleep 0.5
    ((elapsed++))
    if (( elapsed >= timeout * 2 )); then
      fail "daemon did not open port $port within ${timeout}s"
      return 1
    fi
  done
  return 0
}

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# ── Tests 1–4: CLI help (no daemon needed) ────────────────────────────────────

log "Test 1: CLI binary exists"
if [[ -x "$CLI_BIN" ]]; then
  pass "CLI binary found at $CLI_BIN"
else
  fail "CLI binary not found at $CLI_BIN (run: cargo build)"
  echo ""; echo "Results: $PASS_COUNT passed, $FAIL_COUNT failed"; exit 1
fi

log "Test 2: CLI --version"
version_out=$("$CLI_BIN" --version 2>&1 || true)
assert_contains "CLI --version has 'd1'" "$version_out" "d1"

log "Test 3: CLI --help mentions subcommands"
help_out=$("$CLI_BIN" --help 2>&1 || true)
assert_contains "help has 'install'" "$help_out" "install"
assert_contains "help has 'status'"  "$help_out" "status"
assert_contains "help has 'auth'"    "$help_out" "auth"

log "Test 4: install --help"
install_help=$("$CLI_BIN" install --help 2>&1 || true)
assert_contains "install --help works" "$install_help" "install"

# ── Tests 5–8: Daemon lifecycle (requires binary) ────────────────────────────

if [[ ! -x "$DAEMON_BIN" ]]; then
  log "Daemon binary not found — skipping daemon lifecycle tests"
  echo ""
  echo "────────────────────────────────────────"
  printf 'Results: \033[1;32m%d passed\033[0m, \033[1;31m%d failed\033[0m\n' "$PASS_COUNT" "$FAIL_COUNT"
  echo "────────────────────────────────────────"
  (( FAIL_COUNT > 0 )) && exit 1 || exit 0
fi

log "Test 5: Starting daemon…"
"$DAEMON_BIN" &
DAEMON_PID=$!

if wait_for_port "$DAEMON_PORT" 10; then
  pass "Daemon listening on port $DAEMON_PORT"
else
  fail "Daemon failed to start"
  exit 1
fi

log "Test 6: /health endpoint"
if command -v curl >/dev/null 2>&1; then
  health_out=$(curl -sf "http://127.0.0.1:$DAEMON_PORT/health" 2>&1 || true)
  assert_contains "/health returns JSON" "$health_out" "{"
fi

log "Test 7: CLI files --help"
files_help=$("$CLI_BIN" files --help 2>&1 || true)
assert_contains "files --help works" "$files_help" "files"

log "Test 8: CLI diagnose --help"
diag_help=$("$CLI_BIN" diagnose --help 2>&1 || true)
assert_contains "diagnose --help works" "$diag_help" "diagnose"

echo ""
echo "────────────────────────────────────────"
printf 'Results: \033[1;32m%d passed\033[0m, \033[1;31m%d failed\033[0m\n' "$PASS_COUNT" "$FAIL_COUNT"
echo "────────────────────────────────────────"
(( FAIL_COUNT > 0 )) && { echo "E2E FAILED"; exit 1; } || { echo "E2E PASSED"; exit 0; }
