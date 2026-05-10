# Workbench

![Workbench hero image](./docs/public/workbench-hero.png)

> A Claude-Native Spec-Driven Development Template

A GitHub template for building **spec-driven multiplatform apps with Claude Code** as your primary collaborator. Specs are the source of truth; every platform implements them natively. There is no shared code — reconciliation across platforms happens through agent-mediated regeneration, not through a shared library.

This template assumes you will work with [Claude Code](https://claude.com/claude-code) every day. Every convention, file, and workflow in here is shaped by that assumption.

## Quick start

1. Click **"Use this template"** on GitHub to create your repo.
2. Clone it locally and open it in your editor.
3. Customize the seed content:
    - [`specs/ARCHITECTURE.md`](specs/ARCHITECTURE.md) — fill in the `[NEEDS CLARIFICATION]` product overview and out-of-scope sections.
    - [`specs/DESIGN_SYSTEM.md`](specs/DESIGN_SYSTEM.md) — adjust tokens once branding is settled.
    - [`CLAUDE.md`](CLAUDE.md) — keep, but tweak the "Working with specs" note if you want a different reference platform than web.
    - `docs/index.md`, `docs/.vitepress/config.ts`, `docs/.vitepress/theme/components/Hero.vue` — set the project title.
    - `docs/public/` — replace `workbench-hero.png`, `workbench-icon.png`, `favicon.svg` with your own brand art.
4. Author your first feature: invoke the `brainstorming-feature` skill in Claude. It populates `features/0001-<your-slug>/`.
5. Scaffold your reference platform under `apps/<platform>/`. Add the per-platform `CLAUDE.md` and `mise.toml`. (See [`CLAUDE.md`](CLAUDE.md) for the recommended layout.)
6. Implement the feature on the reference platform with the `implementing-a-spec` skill, then mirror to other platforms via `/sdd-apply <spec-id> <platform>`.

## What "Claude-native" means

Lots of templates can be _used with_ an AI assistant. This one is _designed for_ one. Concretely:

- **The spec is the contract; the agent is the implementer.** Specs in [`specs/`](specs/) and `features/<NNNN>-<slug>/` describe behavior in a form Claude can read, reason about, and translate into native code on each platform. Implementations carry `// SPEC: <id>` reverse pointers so Claude can trace from a line of code back to the spec it came from — and detect drift in the other direction.
- **Workflows live as skills, not in your head.** Recurring procedures (writing a story, implementing a spec, debugging, verifying before claiming done) are encoded in [`.claude/skills/`](.claude/skills/) so that any session — yours, a teammate's, a fresh agent — picks up the same discipline.
- **Cross-cutting checks live as subagents.** Audits like drift detection, spec review, test-coverage gaps, and visual verification run as isolated subagents in [`.claude/agents/`](.claude/agents/) so they don't pollute the main conversation context.
- **Repetitive judgment is automated as hooks.** Formatting on edit, blocking edits to generated files, regenerating Convex / Tuist projects, linting on stop, surfacing reconciliation reminders when a spec changes — all in [`.claude/hooks/`](.claude/hooks/).
- **The orientation file is the orientation file.** [`CLAUDE.md`](CLAUDE.md) loads on every session and tells Claude how this repo works. There is no second README that drifts from the first.

You can absolutely work in this repo without Claude — the specs, tests, and code are all human-readable, the docs site renders the spec library, and `mise` runs everything from the terminal. But the workflows assume Claude is doing a lot of the typing.

## Why this template invents its own skills, agents, hooks, and conventions

There are several mature ecosystems for AI-assisted development — [**Superpowers**](https://github.com/obra/superpowers) (a curated skill library) and [**Beads**](https://github.com/steveyegge/beads) (an issue tracker designed for AI workflows) being two we drew inspiration from. We deliberately don't depend on either. Here's why.

### vs. Superpowers

Superpowers is a fantastic, broad skill library — debugging, brainstorming, code review, plan execution, worktree management, and more. We took the **patterns** but rewrote them lighter and tighter for this template's specific shape:

- **No plan documents, no branch ceremony.** Superpowers leans on plan files and worktree branches to coordinate multi-step work. This template uses TodoWrite + per-spec subagents (see the [`implementing-a-spec`](.claude/skills/implementing-a-spec/SKILL.md) skill) because the unit of work is "satisfy this spec on this platform" — finer-grained than a plan doc, larger than a single edit.
- **Reverse pointers replace task tracking.** Every line of code points back to its spec via `// SPEC: <id>`. Drift detection is `rg`-able, not ticket-shaped. You don't need a separate issue tracker to know what's done — `/sdd-drift` and `/sdd-cover` derive it from the code.
- **Smaller surface area = faster onboarding.** Superpowers ships ~50 skills. This template ships ~12, each tuned to the spec-driven workflow. A new contributor reads them all in a sitting.
- **`[NEEDS CLARIFICATION]` is the missing-info convention.** Borrowed in spirit from Superpowers' brainstorming flow but adapted: any unresolved question lives inline in the spec as `[NEEDS CLARIFICATION: ...]` and is resolved by the [`/sdd-clarify`](.claude/commands/sdd-clarify.md) command.

Several skills (notably [`brainstorming-feature`](.claude/skills/brainstorming-feature/SKILL.md), [`test-driven-development`](.claude/skills/test-driven-development/SKILL.md), [`systematic-debugging`](.claude/skills/systematic-debugging/SKILL.md), [`verification-before-completion`](.claude/skills/verification-before-completion/SKILL.md)) lift their core moves directly from Superpowers — see the attribution in each `SKILL.md` header. The originals are excellent; we just wanted them shaped to this repo's grain.

### vs. Beads

Beads is a thoughtful ticket tracker built around the way agents actually work — claim/release semantics, dependency graphs, ready-task surfacing. We chose to **derive the same information from the spec library itself** instead:

- **The spec ID _is_ the ticket ID.** When `vm.items.list` exists in `specs/`, that is the source of work. Tracking it separately in a ticket system means two systems to keep in sync — and the spec already has frontmatter (`id`, `kind`, `depends-on`, `[NEEDS CLARIFICATION]`) that subsumes most ticket fields.
- **"Ready work" is `/sdd-drift` + `/sdd-cover`.** Specs that exist but lack implementation, or whose implementation has drifted, are the ready queue. The [`drift-hunter`](.claude/agents/drift-hunter.md) subagent produces a prioritized punch list on demand.
- **Cross-platform coverage is structural, not a query.** Every spec maps to N platforms; `/sdd-cover <spec-id>` shows you which platforms implement it and which tests pass. No ticket joins required.
- **Fewer dependencies for a template.** Bringing in a tracker means everyone who clones the template installs and configures it before doing useful work. Specs and reverse pointers are just markdown and grep.

If your project _grows_ to need a ticket tracker (especially for cross-team coordination beyond the repo), Beads is a great choice — it composes cleanly alongside this template. We just didn't want it to be a precondition.

### vs. generic tooling

Most "AI-friendly" templates are AI-agnostic templates with a `CLAUDE.md` bolted on. This one inverts that: the spec format, the slash commands, the hook lifecycle, and the per-platform discipline were all designed assuming you'll be reading and writing markdown _alongside_ an agent that can navigate the repo. If we end up using a different agent later, much of the structure will still hold — but the optimization target is Claude Code today.

## How SDD works here

The flow is the same on every platform:

1. **Brainstorm a feature** — invoke the [`brainstorming-feature`](.claude/skills/brainstorming-feature/SKILL.md) skill. It walks narrative → stories → models → view-models → flows → errors and populates a `features/<NNNN>-<slug>/` folder using the templates in [`.claude/templates/feature/`](.claude/templates/feature/). Anything unresolved becomes `[NEEDS CLARIFICATION: ...]`.
2. **Clarify** — run [`/sdd-clarify <feature>`](.claude/commands/sdd-clarify.md) to resolve outstanding markers with you, the human.
3. **Review the spec** — dispatch the [`spec-reviewer`](.claude/agents/spec-reviewer.md) subagent for a P0/P1/P2 audit before implementation.
4. **Implement on the reference platform** — invoke the [`implementing-a-spec`](.claude/skills/implementing-a-spec/SKILL.md) skill, or run [`/sdd-apply <spec-id> web`](.claude/commands/sdd-apply.md). The skill writes failing tests first ([`test-driven-development`](.claude/skills/test-driven-development/SKILL.md)), then the minimum implementation to pass them, then runs spec-compliance and code-quality reviews.
5. **Mirror to other platforms** — `/sdd-apply <spec-id> ios`, `/sdd-apply <spec-id> android`. The web implementation becomes a worked example alongside the spec; the agent translates idiomatically.
6. **Verify** — `/sdd-verify <platform>` runs the platform's behavioral tests. The [`visual-verifier`](.claude/agents/visual-verifier.md) subagent walks the Gherkin scenarios through the actual UI (Chrome DevTools / iOS simulator / Android emulator).
7. **Audit drift over time** — `/sdd-drift <platform>` (or the [`drift-hunter`](.claude/agents/drift-hunter.md) subagent for a multi-platform sweep) lists spec IDs whose implementation has drifted.

The discipline is enforced by hooks: `block-generated.sh` refuses edits to generated artifacts, `format-on-edit.sh` formats every touched file, `spec-reconcile.sh` reminds you to `/sdd-apply` when a spec changes, and `stop-lint.sh` runs lint on dirty platforms before letting Claude declare done.

## Repo layout

```
.
├── CLAUDE.md                          ← orientation doc Claude loads every session
├── README.md                          ← this file (orientation for humans)
├── .mcp.json                          ← project-level MCP servers (Chromium DevTools)
├── .claude/                           ← everything Claude-shaped (see catalog below)
├── docs/                              ← VitePress site rendering specs/ and features/
├── specs/                             ← cross-cutting specs (CONVENTIONS, ARCHITECTURE, DESIGN_SYSTEM)
├── mise.toml                          ← root task runner (docs:* + per-platform orchestration)
│
├── features/                          ← (you create) feature-scoped specs as <NNNN>-<slug>/
├── apps/                              ← (you create) platform implementations
│   ├── web/                           ←   TanStack Start + Convex (recommended reference)
│   ├── ios/                           ←   Swift / SwiftUI / Swift Testing
│   └── android/                       ←   Kotlin / Jetpack Compose / kotlin.test
└── services/                          ← (you create) backend services
    └── convex/                        ←   Convex backend
```

The `apps/` and `services/` directories aren't committed in this template — you scaffold them per-platform as you start that platform's work. Each scaffolded directory will have its own `CLAUDE.md` (with stack idioms) and `mise.toml` (with build/test/lint tasks).

## Catalog of Claude-specific files

Everything below is what makes this template "Claude-native." If you copied just these files into another repo, you'd have most of the spec-driven workflow.

### Root-level

| Path                        | Purpose                                                                                                                                                                                                                                 |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`CLAUDE.md`](CLAUDE.md)    | Loaded on every Claude Code session. The top-level orientation: how the repo works, where to read first, slash command index, skill index. `@includes` the rule files below so they're part of every session.                           |
| [`.mcp.json`](.mcp.json)    | Project-level MCP server config. Currently registers the [Chrome DevTools MCP](https://github.com/ChromeDevTools/chrome-devtools-mcp) pointed at **Chromium** (not Chrome) in `--isolated` mode for visual verification of the web app. |
| `apps/<platform>/CLAUDE.md` | Per-platform orientation (created when you scaffold the platform). Stack idioms, test commands, where reverse pointers go in that language.                                                                                             |
| `services/convex/CLAUDE.md` | Backend orientation (created when you scaffold Convex). Schema-as-protocol conventions, mutation/query patterns.                                                                                                                        |

### `.claude/settings.json`

Project-level Claude Code settings. Wires up:

- **Permissions** — auto-allow safe read-only and build commands (`mise`, `pnpm`, `xcodebuild`, `adb`, `rg`, etc.); ask before `git push`, `convex deploy`, `rm -rf`.
- **Hooks** — registers every script in `.claude/hooks/` to its lifecycle event (`PreToolUse`, `PostToolUse`, `Stop`, `UserPromptSubmit`, `Notification`).
- **Default permission mode** — `auto` (continuous execution).

### `.claude/rules/` — `@included` into orientation docs

Loaded on every session via `@includes` from `CLAUDE.md`.

| Path                                                                       | Purpose                                                                                                        |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| [`.claude/rules/code-quality.md`](.claude/rules/code-quality.md)           | The "what good code looks like in this repo" rules — file size, naming, comments, abstraction, error handling. |
| [`.claude/rules/commit-discipline.md`](.claude/rules/commit-discipline.md) | When to commit, what one commit contains, message format, staging policy, push policy.                         |
| [`.claude/rules/spec-conventions.md`](.claude/rules/spec-conventions.md)   | The compact between specs, tests, and implementations. Loaded by every per-platform `CLAUDE.md`.               |

### `.claude/skills/` — procedural workflows

Skills are markdown files that encode "how we do X here." Claude invokes them via the `Skill` tool.

| Skill                                                                                      | When to use                                                                                                    |
| ------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| [`brainstorming-feature`](.claude/skills/brainstorming-feature/SKILL.md)                   | Before starting any new feature. Walks narrative → stories → models → view-models → flows → errors.            |
| [`writing-user-stories`](.claude/skills/writing-user-stories/SKILL.md)                     | When authoring or reviewing a story file. Enforces Gherkin discipline.                                         |
| [`implementing-a-spec`](.claude/skills/implementing-a-spec/SKILL.md)                       | The default "how to write code" workflow. Per-spec subagent dispatch + two-stage review. Used by `/sdd-apply`. |
| [`test-driven-development`](.claude/skills/test-driven-development/SKILL.md)               | When writing any production code. Iron Law: no production code without a failing test first.                   |
| [`verification-before-completion`](.claude/skills/verification-before-completion/SKILL.md) | Before claiming any work is complete. Run the verifying command in this turn; evidence before claims.          |
| [`systematic-debugging`](.claude/skills/systematic-debugging/SKILL.md)                     | When encountering any bug or unexpected behavior. Find the root cause before proposing a fix.                  |
| [`web-development`](.claude/skills/web-development/SKILL.md)                               | When writing web code. TanStack Start + Convex + Tailwind v4 + React Aria idioms.                              |
| [`web-verification`](.claude/skills/web-verification/SKILL.md)                             | When verifying web UI in a browser. Wraps the Chrome DevTools MCP.                                             |
| [`ios-development`](.claude/skills/ios-development/SKILL.md)                               | When writing iOS code. SwiftUI + `@Observable` + Swift Testing idioms, HIG link list.                          |
| [`ios-simulator-control`](.claude/skills/ios-simulator-control/SKILL.md)                   | When verifying iOS UI changes. Wraps `xcrun simctl` + `idb`.                                                   |
| [`android-development`](.claude/skills/android-development/SKILL.md)                       | When writing Android code. Compose + Material 3 + Kotlin coroutines/Flow idioms.                               |
| [`android-emulator-control`](.claude/skills/android-emulator-control/SKILL.md)             | When verifying Android UI changes. Wraps `adb` + `uiautomator`.                                                |

### `.claude/agents/` — cross-cutting subagents

Subagents run in their own context window and return a single message back to the main conversation. Use them for audits and isolated checks.

| Agent                                                  | Purpose                                                                                                                                                             |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`drift-hunter`](.claude/agents/drift-hunter.md)       | Audits cross-platform spec/impl drift. Runs `/sdd-drift` across platforms, cross-references with `/sdd-verify` output, returns a prioritized punch list. Read-only. |
| [`spec-reviewer`](.claude/agents/spec-reviewer.md)     | Reviews a spec file before it lands. Frontmatter, Gherkin discipline, `[NEEDS CLARIFICATION]` markers, reverse-pointer health. P0/P1/P2 issue list.                 |
| [`test-gap-finder`](.claude/agents/test-gap-finder.md) | Finds Gherkin scenarios that don't have a matching `[scenario.<id>]`-tagged test on a given platform. Test-coverage drift (vs. drift-hunter's code drift).          |
| [`visual-verifier`](.claude/agents/visual-verifier.md) | Drives Chrome DevTools / iOS simulator / Android emulator through each Gherkin scenario in a `story.*` spec, screenshots each state, reports rendering mismatches.  |
| [`handoff-builder`](.claude/agents/handoff-builder.md) | At the end of a development pass, generates or updates `HANDOFF.md` so a future session can pick up the branch with full context.                                   |

### `.claude/commands/` — slash commands

User-typed commands. Each is intent-only at the moment — the agent uses `rg`, `Edit`, `AskUserQuestion`, etc. to fulfill them; no automation script behind them yet.

| Command                                                                 | Purpose                                                                                           |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| [`/sdd-apply <spec-id> <platform>`](.claude/commands/sdd-apply.md)      | Regenerate a spec's implementation and tests on a target platform.                                |
| [`/sdd-verify <platform>`](.claude/commands/sdd-verify.md)              | Run the platform's behavioral test suite and report which spec IDs pass.                          |
| [`/sdd-drift <platform>`](.claude/commands/sdd-drift.md)                | List spec IDs whose implementation has drifted from the spec on a platform.                       |
| [`/sdd-cover <spec-id>`](.claude/commands/sdd-cover.md)                 | Show which platforms implement a spec and which of their tests pass.                              |
| [`/sdd-reconcile <source-platform>`](.claude/commands/sdd-reconcile.md) | Bring the spec + other platforms in line with this platform's impl (when a platform raced ahead). |
| [`/sdd-clarify <feature-or-spec>`](.claude/commands/sdd-clarify.md)     | Scan a feature or spec for `[NEEDS CLARIFICATION]` markers and resolve them with the user.        |
| [`/sdd-analyze <feature>`](.claude/commands/sdd-analyze.md)             | Read-only cross-artifact consistency check for a feature folder.                                  |

### `.claude/hooks/` — lifecycle scripts

Bash scripts wired up in `settings.json`. Each runs at a specific Claude Code lifecycle event. Failures are logged but don't crash the agent (except where blocking is intentional, e.g. `stop-lint.sh`).

| Hook                                                               | Event                                       | Purpose                                                                                                                                                        |
| ------------------------------------------------------------------ | ------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`block-generated.sh`](.claude/hooks/block-generated.sh)           | `PreToolUse` (Edit/Write/MultiEdit)         | Refuses edits to files that are tool-generated (Convex `_generated/`, Xcode-derived data, etc.).                                                               |
| [`conventional-commits.sh`](.claude/hooks/conventional-commits.sh) | `PreToolUse` (Bash, gated to `git commit*`) | Requires a Conventional Commits prefix (`feat:`, `fix:`, etc.) in the commit message.                                                                          |
| [`format-on-edit.sh`](.claude/hooks/format-on-edit.sh)             | `PostToolUse` (Edit/Write/MultiEdit)        | Formats the touched file in place using the right formatter for the extension (`oxfmt`, `swiftformat`, `ktlint`).                                              |
| [`convex-codegen.sh`](.claude/hooks/convex-codegen.sh)             | `PostToolUse` (Edit/Write/MultiEdit)        | Regenerates Convex types when `schema.ts` changes.                                                                                                             |
| [`tuist-regen.sh`](.claude/hooks/tuist-regen.sh)                   | `PostToolUse` (Edit/Write/MultiEdit)        | Regenerates the Xcode project when `Project.swift` changes.                                                                                                    |
| [`spec-reconcile.sh`](.claude/hooks/spec-reconcile.sh)             | `PostToolUse` (Edit/Write/MultiEdit)        | When a spec is edited, lists implementations that reference its ID and suggests `/sdd-apply` per platform. When code is edited, surfaces drift hints.          |
| [`stop-lint.sh`](.claude/hooks/stop-lint.sh)                       | `Stop`                                      | Runs lint on whichever platforms have uncommitted changes since `HEAD`. **Blocks the stop** if any lint fails — Claude can't declare "done" with a dirty lint. |
| [`user-prompt-context.sh`](.claude/hooks/user-prompt-context.sh)   | `UserPromptSubmit`                          | Injects current branch + uncommitted changes into the agent's context, so natural commit points are obvious.                                                   |
| [`notify-long-task.sh`](.claude/hooks/notify-long-task.sh)         | `Notification`                              | Surfaces a macOS notification when Claude Code needs attention (long-running task, permission prompt).                                                         |

### `.claude/templates/` — canonical scaffolds

Markdown templates for new specs and features. Used by the `brainstorming-feature` skill and by anyone authoring a spec by hand.

| Path                                                                 | Purpose                                                                                                                                                                                                              |
| -------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`.claude/templates/feature/`](.claude/templates/feature/)           | Full structure for a new `features/<NNNN>-<slug>/` folder: `NARRATIVE.md`, `stories/STORY.md`, `models/MODEL.md`, `view-models/VIEW_MODEL.md`, `use-cases/USE_CASE.md`, `user-flow/USER_FLOW.md`, `errors/ERROR.md`. |
| [`.claude/templates/spec/MODEL.md`](.claude/templates/spec/MODEL.md) | Canonical template for a cross-cutting spec under `specs/`.                                                                                                                                                          |

## Local tooling

[`mise`](https://mise.jdx.dev) is the task runner. The root [`mise.toml`](mise.toml) only ships `docs:*` tasks and `fmt`; per-platform tasks live in `apps/*/mise.toml` and `services/*/mise.toml` once you scaffold them.

```sh
mise run docs:dev          # docs site (VitePress) at http://localhost:5173
mise run docs:build        # static build to docs/.vitepress/dist
mise run docs:preview      # preview the built site
mise run fmt               # format the entire project (oxfmt)
mise tasks                 # list everything available
```

When you scaffold a platform, also add its orchestration task at the root (e.g. `web:dev`, `ios:test`) so cross-platform commands work from the repo root.

---

## What's deliberately not included

- **`apps/` and `services/` directories.** Scaffold these when you choose your stack — different teams will want different platforms in different orders.
- **Automation behind the `/sdd-*` commands.** They are intent-only at the moment; the agent uses `rg`, `Edit`, `AskUserQuestion`, etc. to fulfill them. As patterns stabilize, some of this will move into shell scripts or a small CLI.
- **A worked example feature.** The original repo this was extracted from has a contacts-app pass; this template ships clean so you can put your own thing in `features/0001-*/`.
- **A ticket tracker.** As discussed above — the spec library is the source of work. Bring in Beads or your tracker of choice if you outgrow that.

## Read next

- [`CLAUDE.md`](CLAUDE.md) — the orientation doc Claude Code loads on every session. Read this even if you're working without an agent; it's the canonical "how this repo works."
- [`specs/CONVENTIONS.md`](specs/CONVENTIONS.md) — the spec contract: IDs, kinds, frontmatter, reverse pointers, deviation markers, drift detection.
- [`specs/ARCHITECTURE.md`](specs/ARCHITECTURE.md) — top-level layering, data flow, deployment.
- [`specs/DESIGN_SYSTEM.md`](specs/DESIGN_SYSTEM.md) — design tokens, component vocabulary, parity rules across platforms.
- [`.claude/skills/brainstorming-feature/SKILL.md`](.claude/skills/brainstorming-feature/SKILL.md) — how to author a feature folder.
- [`.claude/skills/implementing-a-spec/SKILL.md`](.claude/skills/implementing-a-spec/SKILL.md) — the default "how to write code" workflow.
