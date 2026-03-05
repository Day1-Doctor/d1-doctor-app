#!/bin/bash
# Quality gate before PR. Copy to .claude/hooks/pre-pr-gate.sh in product repos.
INPUT=$(cat)
CWD=$(echo "$INPUT" | jq -r '.cwd // "."')
cd "$CWD"

BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
ISSUE_ID=$(echo "$BRANCH" | grep -oE '[A-Z]+-[0-9]+' | head -1 || echo "")
[[ -z "$ISSUE_ID" ]] && exit 0

echo "[mf-gate] Running quality gate for $ISSUE_ID..." >&2

PASS=true
if [[ -f "package.json" ]]; then
  npm test --if-present --silent 2>/dev/null || PASS=false
elif [[ -f "pyproject.toml" ]] || [[ -f "pytest.ini" ]]; then
  python -m pytest -q 2>/dev/null || PASS=false
elif [[ -f "Cargo.toml" ]]; then
  cargo test --quiet 2>/dev/null || PASS=false
fi

if [[ "$PASS" == "false" ]]; then
  echo "⚠️  QUALITY GATE FAILED: Tests did not pass."
  echo "Fix failing tests before creating a PR."
fi
