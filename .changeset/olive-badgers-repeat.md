---
"workbench": minor
---

Replace the Vite+ toolchain with the platform-neutral tooling layer the project templates share: mise owns every pinned tool and every task, hk owns every git hook, Fnox owns secrets, Beads owns the work graph, Changesets owns the changelog, and Worktrunk owns worktrees.

What adopters have to do: install mise, then run `mise install` and `mise run setup` instead of `vp install`. In an existing checkout, first run `git config --unset core.hooksPath`: hk registers its hooks through git's `hook.<name>.command` config, and a leftover `core.hooksPath` from `vp config` keeps the old Vite+ hooks running beside them, so commits fail. Every command changes name — `mise run check` is the gate, `mise run status` reports the phase, `mise run fmt` formats. Formatting, lint and type checking now come from `.oxfmtrc.jsonc`, `.oxlintrc.jsonc` and `tsrs` rather than `vite.config.ts`, which is gone along with `.agents/hooks/`; the `commit-msg` hook reads its scopes from the new `.agents/commit-scopes`. Deferred defects go to Beads instead of GitHub issues, and the `tracking-defects` skill is now `tracking-work`. Every pull request carries a changeset.
