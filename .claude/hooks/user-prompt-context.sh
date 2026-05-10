#!/usr/bin/env bash
# UserPromptSubmit: inject current branch + uncommitted changes so commits at natural points are obvious.
set -euo pipefail

cd "$CLAUDE_PROJECT_DIR"

branch=$(git branch --show-current 2>/dev/null || echo "(detached)")
status=$(git status --short 2>/dev/null || true)

if [[ -n "$status" ]]; then
    context="Current branch: $branch
Uncommitted changes:
$status"
else
    context="Current branch: $branch (clean working tree)"
fi

jq -n --arg ctx "$context" '{hookSpecificOutput: {hookEventName: "UserPromptSubmit", additionalContext: $ctx}}'
