#!/usr/bin/env bash
# PostToolUse: format the touched file in place, dispatched by extension.
# Best-effort: any failure is silenced so the agent's edit isn't disrupted.
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')
[[ -z "$file_path" || ! -f "$file_path" ]] && exit 0

case "$file_path" in
    *.swift)
        (cd "$CLAUDE_PROJECT_DIR/apps/ios" && mise exec -- swift-format format -i "$file_path") 2>/dev/null || true
        ;;
    *.ts|*.tsx|*.js|*.jsx|*.mjs|*.cjs)
        if [[ "$file_path" == */apps/web/* ]]; then
            (cd "$CLAUDE_PROJECT_DIR/apps/web" && mise exec -- oxfmt "$file_path") 2>/dev/null || true
        elif [[ "$file_path" == */services/convex/* ]]; then
            (cd "$CLAUDE_PROJECT_DIR/services/convex" && mise exec -- oxfmt "$file_path") 2>/dev/null || true
        fi
        ;;
    # Kotlin format goes through Gradle, too slow for per-file. Run `mise run -C apps/android fmt` manually.
esac

exit 0
