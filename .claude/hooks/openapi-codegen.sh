#!/usr/bin/env bash
# PostToolUse: when the OpenAPI contract changes, remind the agent to regenerate
# the per-platform typed clients. It does NOT run codegen — the contract producer
# and the per-platform generators are project choices (see specs/ARCHITECTURE.md),
# and the apps may not be scaffolded yet. Reminder only, like spec-reconcile.sh.
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')
[[ -z "$file_path" ]] && exit 0

# Fire on edits to the contract / OpenAPI document. Adjust these globs to match
# where your project emits its OpenAPI document (Convex HTTP actions or the
# TS-Rest gateway — see the open question in specs/ARCHITECTURE.md).
case "$file_path" in
    */contract/*|*openapi*.json|*openapi*.yaml|*openapi*.yml) ;;
    *) exit 0 ;;
esac

echo "OpenAPI contract changed ($file_path). Regenerate the typed client on every native/CLI platform you ship:
  - Apple    → Swift OpenAPI Generator    (mise run -C apps/ios codegen)
  - Android  → OpenAPI Generator (Kotlin)  (mise run -C apps/android codegen)
  - Windows  → Kiota                       (mise run -C apps/windows codegen)
  - Linux    → Progenitor                  (mise run -C apps/linux codegen)
  - Rust CLI → Progenitor                  (mise run -C apps/tui codegen)
Web/website consume Convex directly and need no OpenAPI client."

exit 0
