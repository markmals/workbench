#!/usr/bin/env bash
# PostToolUse: regenerate the Xcode project when Project.swift changes.
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')

[[ "$file_path" == */apps/ios/Project.swift ]] || exit 0

cd "$CLAUDE_PROJECT_DIR/apps/ios"
if output=$(mise run g 2>&1); then
    echo "tuist generate: $(echo "$output" | tail -1)"
else
    echo "tuist generate failed:" >&2
    echo "$output" >&2
    exit 1
fi
