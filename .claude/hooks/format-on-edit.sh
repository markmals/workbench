#!/usr/bin/env bash
# PostToolUse: format the touched file via the governing platform's `fmt` task.
#
# The formatter choice lives in each platform's mise.toml `fmt` task (the root
# `fmt` handles repo-level files) — NOT here. This hook only dispatches by
# directory, so adding a platform never means editing this file. Each platform's
# `fmt` task should accept optional file arguments (format just those; format the
# whole platform when given none).
#
# Best-effort: any failure — including a not-yet-scaffolded platform with no
# `fmt` task — is silenced so the agent's edit isn't disrupted.
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')
[[ -z "$file_path" || ! -f "$file_path" ]] && exit 0

rel="${file_path#"$CLAUDE_PROJECT_DIR"/}"
case "$rel" in
    apps/*/* | services/*/*) dir=$(printf '%s' "$rel" | cut -d/ -f1-2) ;; # apps/<platform> | services/<svc>
    *) dir="." ;;                                                          # repo-root files
esac

(cd "$CLAUDE_PROJECT_DIR" && mise run -C "$dir" fmt -- "$file_path") 2>/dev/null || true
exit 0
