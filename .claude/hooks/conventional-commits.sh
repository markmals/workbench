#!/usr/bin/env bash
# PreToolUse on Bash (gated by if: "Bash(git commit*)"):
# require a Conventional Commits prefix anywhere in the command — handles both
# `git commit -m "feat: ..."` and HEREDOC forms.
set -euo pipefail

input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command // empty')

# Skip if this isn't actually creating a commit (e.g. `git commit --help`).
if ! echo "$command" | grep -qE '(^|[[:space:]])git[[:space:]]+commit\b'; then
    exit 0
fi
if echo "$command" | grep -qE '(--help|--allow-empty-message|--amend)'; then
    exit 0
fi

if ! echo "$command" | grep -qE '(feat|fix|refactor|test|docs|chore|spec|perf|style|build|ci)(\([a-z0-9_./-]+\))?:[[:space:]]'; then
    echo "Commit message must start with a Conventional Commits prefix (feat:, fix:, refactor:, test:, docs:, chore:, spec:, perf:, style:, build:, ci:) — see .claude/rules/commit-discipline.md." >&2
    exit 2
fi

exit 0
