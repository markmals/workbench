# Hooks

Install the Git commit hook:

```sh
ln -s ../../.agents/hooks/commit-msg .git/hooks/commit-msg
```

You may set `core.hooksPath` instead. CI runs the same validation, so the hook is not the only line
of defence.

| Lifecycle event                           | Run                   |
| ----------------------------------------- | --------------------- |
| After a file edit                         | `mise run fmt <file>` |
| Before the agent declares a turn complete | `mise run check`      |
| Session start                             | Read `AGENTS.md`      |

Harness lifecycle wiring is harness-specific, lives outside `.agents/`, and is covered by
[`ADOPTING.md`](../../ADOPTING.md).
