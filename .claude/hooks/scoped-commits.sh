#!/usr/bin/env bash
# PreToolUse on Bash (gated by if: "Bash(git commit*)"):
# require a Scoped Commits subject — `<scope>: <description>` — anywhere in the
# command, so it handles both `git commit -m "web: ..."` and HEREDOC forms.
# See https://scopedcommits.com/ and .claude/rules/commit-discipline.md.
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

# Catch the Conventional Commits habit: a `type(subscope):` prefix — a word
# immediately followed by `(...)` with no space. Scoped Commits is scope-first;
# a ticket, if any, goes after a space: `web (PROJ-12): ...`.
if echo "$command" | grep -qE '[a-z]+\([a-z0-9_./-]+\):[[:space:]]'; then
    echo "Looks like a Conventional Commits 'type(scope):' subject. This repo uses Scoped Commits — lead with the scope: '<scope>: <description>' (ticket goes after a space, e.g. 'web (PROJ-12): ...'). See .claude/rules/commit-discipline.md." >&2
    exit 2
fi

# A Scoped Commits subject: one or more lowercase scope tokens (optionally with a
# ` (TICKET)` suffix), then `: ` and a description.
scope='[a-z0-9._/-]+([[:space:]]\([A-Za-z0-9-]+\))?'
if ! echo "$command" | grep -qE "${scope}(,[[:space:]]${scope})*:[[:space:]]"; then
    echo "Commit subject must be a Scoped Commit: '<scope>: <description>' (e.g. 'web:', 'vm.items.list:', 'specs:', 'hooks:', or 'treewide:'). See .claude/rules/commit-discipline.md." >&2
    exit 2
fi

exit 0
