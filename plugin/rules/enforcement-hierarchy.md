---
description: Where a new rule belongs in a Workbench project — a check, a task, a template, a skill, or prose, strongest first.
---

# Enforcement Hierarchy

A rule an agent must remember is the weakest kind of rule. Under load, prose gets missed. Durable conventions are machine-enforced, not merely recalled. When you add a rule, reach down this hierarchy first.

## The tiers — strongest to weakest

| Tier | Mechanism | Role |
| --- | --- | --- |
| **Tier 0 — Checks** | hk's git hooks (`commit-msg` runs `workbench commits check`, `pre-commit` runs the project's formatters and linters on staged files, and the Beads steps: `pre-push`, `post-merge`, and `post-rewrite` after a rebase sync Beads with `workbench beads sync`, and every Beads-owned hook runs `bd hooks run <hook>`), GitHub Actions running `mise run check` on every push and `workbench commits check` and `workbench change-files check` on every pull request, and the record and project checks `workbench records validate` and `workbench project validate` | Deterministic; runs whether or not the agent remembers. A rule that can live here must. |
| **Tier 1 — Tasks and commands** | What the process invokes by name: the mise tasks `mise run check` and `mise run fmt`; the `workbench` commands `workflow status`, `beads claim`, `beads close`, `beads sync`, and `change-files create`; and the `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` tasks that wrap them for humans | Deterministic once invoked, but something must invoke them. |
| **Tier 2 — Templates** | `.workbench/templates/`, and a project's overrides in `.agents/templates/` | Shape the work so the correct thing is the path of least resistance. |
| **Tier 3 — Prose** | The process contract, the project's `AGENTS.md`, `SKILL.md` files, and these rules | Necessary for judgment that cannot be mechanized; the tier most likely to be missed. |

## The rule

- Before writing a prose rule, ask whether a check or template could enforce it deterministically. If it can, build that instead — usually a new hk step or a new check inside `mise run check`, so humans and CI run the same thing.
- Where a check already enforces a rule, do not also state it in prose — delete the prose.
- Keep judgment in prose. Good naming, the right abstraction, and a justified deviation are not cheap deterministic checks.

This hierarchy is why `completing-a-feature` requires attempting a check before writing a prose policy.

## Worked example — policies and decisions

`PROPOSAL.md` carries a “Policies and decisions checked” section. It makes checking the existing record part of the path of least resistance, instead of a policy saying “remember to check policies.” That is a Tier 2 template solution, not a Tier 3 reminder.
