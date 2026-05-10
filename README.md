# Spec-Driven Development Template

A GitHub template for building spec-driven multiplatform apps with Claude Code. Specs are the source of truth; every platform implements them natively. There is no shared code — reconciliation across platforms happens through agent-mediated regeneration, not through a shared library.

## What you get

- **Conventions** — `specs/CONVENTIONS.md` defines IDs, frontmatter, reverse pointers, deviation markers, and `[NEEDS CLARIFICATION]` discipline. **This is the contract.**
- **Templates** — `.claude/templates/feature/` for new feature folders; `.claude/templates/spec/` for cross-cutting specs.
- **Slash commands** — `/sdd-apply`, `/sdd-verify`, `/sdd-drift`, `/sdd-reconcile`, `/sdd-cover`, `/sdd-clarify`, `/sdd-analyze`. (Intent-only at the moment — agent-driven, no automation yet.)
- **Procedural skills** — `brainstorming-feature`, `writing-user-stories`, `implementing-a-spec`, `test-driven-development`, `verification-before-completion`, `systematic-debugging`, plus per-platform development + verification skills (web, iOS, Android).
- **Subagents** — `drift-hunter`, `spec-reviewer`, `test-gap-finder`, `visual-verifier`, `handoff-builder`.
- **Hooks** — format-on-edit, codegen, lint-on-stop, conventional-commits, etc. (All under `.claude/hooks/`.)
- **Docs site** — VitePress at `docs/`, renders `specs/` and `features/` directly with no copying.
- **Per-platform CLAUDE.md scaffolds** are documented but not committed; you scaffold `apps/web/`, `apps/ios/`, `apps/android/`, and `services/convex/` when you start that platform.

## Quick start

1. Click **"Use this template"** on GitHub to create your repo.
2. Clone it locally.
3. Customize:
    - `specs/ARCHITECTURE.md` — fill in the `[NEEDS CLARIFICATION]` product overview and out-of-scope sections.
    - `specs/DESIGN_SYSTEM.md` — adjust tokens once branding is settled.
    - `CLAUDE.md` — keep, but tweak the "Working with specs" note if you want a different reference platform than web.
    - `docs/index.md`, `docs/.vitepress/config.ts`, `docs/.vitepress/theme/components/Hero.vue` — set the project title.
    - `docs/public/` — replace `workbench-hero.png`, `workbench-icon.png`, `favicon.svg` with your own brand art.
4. Author your first feature with the `brainstorming-feature` skill, populating `features/0001-<your-slug>/`.
5. Scaffold your reference platform under `apps/<platform>/` and add the per-platform `CLAUDE.md` and `mise.toml`.
6. Implement the feature on the reference platform with the `implementing-a-spec` skill, then mirror to other platforms via `/sdd-apply <spec-id> <platform>`.

## Local tooling

`mise` is the task runner. The root `mise.toml` only ships `docs:*` tasks; per-platform tasks live in `apps/*/mise.toml` and `services/*/mise.toml` once you scaffold them. See `mise.toml` for the documented orchestration patterns.

```sh
mise run docs:dev          # docs site
mise tasks                 # see what's available
```

## Read next

- [`CLAUDE.md`](CLAUDE.md) — the orientation doc Claude Code loads on every session.
- [`specs/CONVENTIONS.md`](specs/CONVENTIONS.md) — the spec contract.
- [`specs/ARCHITECTURE.md`](specs/ARCHITECTURE.md) — layering, data flow, deployment.
- [`specs/DESIGN_SYSTEM.md`](specs/DESIGN_SYSTEM.md) — tokens, components, parity rules.
- [`.claude/skills/brainstorming-feature/SKILL.md`](.claude/skills/brainstorming-feature/SKILL.md) — how to author a feature folder.
- [`.claude/skills/implementing-a-spec/SKILL.md`](.claude/skills/implementing-a-spec/SKILL.md) — the default "how to write code" workflow.

## What's deliberately not included

- `apps/` and `services/` directories — you scaffold these when you choose your stack.
- The `/sdd-*` command automation — those are agent-driven via `rg`, `Edit`, `AskUserQuestion`, etc. for now.
- A worked example feature — the original repo this was extracted from has a contacts-app pass; this template ships clean.
