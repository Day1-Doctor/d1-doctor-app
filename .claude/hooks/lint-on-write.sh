#!/bin/bash
# Auto-lint after file writes. Copy to .claude/hooks/lint-on-write.sh in product repos.
INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.path // .tool_input.file_path // ""')
[[ -z "$FILE" || ! -f "$FILE" ]] && exit 0

if [[ -f "package.json" ]]; then
  npx eslint "$FILE" --fix --quiet 2>/dev/null || true
  npx prettier --write "$FILE" 2>/dev/null || true
elif [[ -f "pyproject.toml" ]] || [[ -f "setup.py" ]]; then
  command -v ruff &>/dev/null && ruff check "$FILE" --fix --quiet 2>/dev/null || true
elif [[ -f "Cargo.toml" ]]; then
  command -v rustfmt &>/dev/null && rustfmt "$FILE" 2>/dev/null || true
fi
