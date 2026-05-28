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
    *.rs)
        # rustfmt is fast enough per-file. Covers apps/tui and apps/linux.
        (cd "$CLAUDE_PROJECT_DIR" && mise exec -- rustfmt "$file_path") 2>/dev/null || true
        ;;
    *.ts|*.tsx|*.js|*.jsx|*.mjs|*.cjs)
        # oxfmt reads the root .oxfmtrc.jsonc; covers web, website, cli, convex.
        if [[ "$file_path" == */apps/* || "$file_path" == */services/* ]]; then
            (cd "$CLAUDE_PROJECT_DIR" && mise exec -- oxfmt "$file_path") 2>/dev/null || true
        fi
        ;;
    # Kotlin (ktfmt), C# (dotnet format), and .astro formatting go through the
    # platform build (too slow per-file). Run `mise run -C apps/<platform> fmt`
    # manually; .astro is handled by the project's prettier config.
esac

exit 0
