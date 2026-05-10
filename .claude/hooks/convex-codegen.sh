#!/usr/bin/env bash
# PostToolUse: regenerate Convex types when schema.ts changes.
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')

[[ "$file_path" == */services/convex/*schema.ts ]] || exit 0

cd "$CLAUDE_PROJECT_DIR/services/convex"
if output=$(mise run codegen 2>&1); then
    echo "convex codegen: $(echo "$output" | tail -1)"
else
    echo "convex codegen failed:" >&2
    echo "$output" >&2
    exit 1
fi
