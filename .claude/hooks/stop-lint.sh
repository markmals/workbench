#!/usr/bin/env bash
# Stop: run lint on whichever platforms have uncommitted changes since HEAD.
# Blocks the stop (decision: block) if any lint fails, so the agent fixes before declaring done.
set -euo pipefail

input=$(cat)
stop_hook_active=$(echo "$input" | jq -r '.stop_hook_active // false')
[[ "$stop_hook_active" == "true" ]] && exit 0

cd "$CLAUDE_PROJECT_DIR"

failures=""

run_lint() {
    local label="$1" dir="$2" task="$3"
    if ! git diff --quiet HEAD -- "$dir" 2>/dev/null; then
        if ! output=$(mise run -C "$dir" "$task" 2>&1); then
            failures="${failures}${failures:+

}[$label]
$output"
        fi
    fi
}

run_lint "web"     "apps/web"      "lint"
run_lint "website" "apps/website"  "lint"
run_lint "convex"  "services/convex" "lint"
run_lint "cli"     "apps/cli"      "lint"   # stack-dependent: oxlint | clippy | go vet
run_lint "ios"     "apps/ios"      "l"
run_lint "linux"   "apps/linux"    "lint"   # cargo clippy
# Android (Gradle) and Windows (dotnet) lint are slow. Run
# `mise run -C apps/android lint` / `mise run -C apps/windows lint` manually before merge.

if [[ -n "$failures" ]]; then
    jq -n --arg reason "Lint failures before stop:

$failures" '{decision: "block", reason: $reason}'
fi

exit 0
