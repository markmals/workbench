# Spec-Driven Development Template

A spec-driven multiplatform app harness. Specs are the source of truth; every platform implements them natively. Choose a primary "reference" platform — typically web (TanStack Start + Convex) — and mirror its behavior on additional platforms (iOS / Swift, Android / Kotlin) idiomatically.

This is the bare template. The `apps/` and `services/` directories are not committed — you scaffold them per-platform when you start that platform's implementation. The harness assumes (and the per-platform skills are written for) the layout below.

@.claude/rules/code-quality.md
@.claude/rules/commit-discipline.md

## How this repo works

**Specs are the source of truth.** Domain models, view models, user flows, errors, stories — all live as markdown in `specs/` (cross-cutting) and `features/<NNNN>-<slug>/` (feature-scoped). There is no shared code. Every platform implements the spec natively. Reconciliation between platforms happens through agent-mediated regeneration, not through a shared library.

If you are tempted to create a shared package, write a spec instead.

**Read these before doing anything substantial:**

1. `specs/CONVENTIONS.md` — spec format, ID taxonomy, frontmatter, reverse pointers, deviation marker, drift detection. **This is the contract.**
2. `specs/ARCHITECTURE.md` — top-level layering, data flow, deployment targets.
3. `specs/DESIGN_SYSTEM.md` — design tokens, component vocabulary, parity rules across platforms.
4. The platform `CLAUDE.md` for whichever app you're working on (`apps/web/CLAUDE.md`, `apps/ios/CLAUDE.md`, `apps/android/CLAUDE.md`, or `services/convex/CLAUDE.md`) once you've scaffolded it.

### Three places work comes from

The repo derives the work queue from three artifacts, in priority order:

1. **Specs and tests** — the work of building or evolving behavior. `/sdd-apply`, `/sdd-verify`, `/sdd-cover`. Source of truth for cross-platform behavior.
2. **Drift** — specs and implementations out of sync. `/sdd-drift`, `/sdd-reconcile`. Surfaced mechanically from the reverse pointers and mtimes.
3. **Sub-spec defects** — platform-local cosmetic / polish / quirk issues that the cross-platform spec deliberately doesn't cover. Tracked in `apps/<platform>/DEFECTS.md`. Filed via `/sdd-defect`, drained via the `triaging-defects` skill. This file should want to be empty.

If something doesn't fit any of those, it's either a future feature (write a spec) or out of scope.

## Layout

```
.
├── CLAUDE.md                          ← this file
├── .claude/
│   ├── agents/                        ← subagents for cross-cutting checks (drift, gaps, reviews)
│   ├── commands/                      ← slash commands (sdd-apply, sdd-verify, sdd-drift, ...)
│   ├── hooks/                         ← shell hooks (format-on-edit, codegen, lint-on-stop, ...)
│   ├── rules/                         ← shared content @included by platform CLAUDE.md files
│   ├── skills/                        ← procedural workflows (writing-user-stories, simulator control)
│   └── templates/                     ← canonical templates for new features and specs
├── .mcp.json                          ← project MCP servers (Chrome DevTools)
├── docs/                              ← VitePress site rendering specs/ and features/
│   └── index.md                       ← home page (URL-rewritten to /)
├── specs/                             ← cross-cutting specs (CONVENTIONS, ARCHITECTURE, DESIGN_SYSTEM)
├── features/                          ← (you create) feature-scoped specs as <NNNN>-<slug>/
├── apps/                              ← (you create) platform implementations
│   ├── web/                           ←   TanStack Start + Convex (recommended reference)
│   ├── ios/                           ←   Swift / SwiftUI / Swift Testing
│   └── android/                       ←   Kotlin / Jetpack Compose / kotlin.test
└── services/                          ← (you create) backend services
    └── convex/                        ←   Convex backend (schema is the data-layer protocol)
```

## Working with specs

- **Reverse pointers are mandatory.** Every implementation file/class/function that realizes a spec carries `// SPEC: <id>`. Tests are tagged with the spec IDs they verify. See `specs/CONVENTIONS.md` for the per-language form.
- **Reference platform first.** Build new features on your reference platform first (web, by default). Other platforms use that implementation as a worked example alongside the spec.
- **Platform divergence is allowed but explicit.** Use `// SPEC: <id> (deviates: <reason>)` when a platform must differ.
- **Stories use Gherkin acceptance criteria.** See the `writing-user-stories` skill. Scenarios have stable sub-IDs that tests trace back to.

## Slash commands

| Command                           | Purpose                                                                  |
| --------------------------------- | ------------------------------------------------------------------------ |
| `/sdd-apply <spec-id> <platform>` | Regenerate a spec's implementation + tests on a platform.                |
| `/sdd-verify <platform>`          | Run the platform's behavioral test suite.                                |
| `/sdd-drift <platform>`           | List spec IDs whose impl is stale, plus impl files with no spec pointer. |
| `/sdd-reconcile <platform>`       | Bring the spec + other platforms in line with this platform's impl.      |
| `/sdd-cover <spec-id>`            | Show which platforms implement a spec and which tests pass.              |
| `/sdd-defect <platform> <desc>`   | File a sub-spec defect into `apps/<platform>/DEFECTS.md` without breaking flow. |

These are scaffolded with intent docs; their internals are agent-driven (no automation yet — the agent uses `rg`, `Edit`, `AskUserQuestion`, etc.).

## Workflow skills

Procedural skills live under `.claude/skills/`. Use them rather than reaching for ad-hoc patterns — they encode the discipline this template expects.

| Skill                            | When to invoke                                                                                                            |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `brainstorming-feature`          | Before starting any new feature or substantial change. Walks narrative → stories → models → view-models → flows → errors. |
| `writing-user-stories`           | When authoring or reviewing a story file. Enforces Gherkin discipline.                                                    |
| `implementing-a-spec`            | The default "how to write code" workflow. Per-spec subagent dispatch + two-stage review. Used by `/sdd-apply`.            |
| `test-driven-development`        | When writing any production code. Iron Law: no production code without a failing test first.                              |
| `verification-before-completion` | Before claiming any work is complete. Run the verifying command in this turn; evidence before claims.                     |
| `systematic-debugging`           | When encountering any bug or unexpected behavior. Find the root cause before proposing a fix.                             |
| `triaging-defects`               | When `apps/<platform>/DEFECTS.md` is non-empty and you're in a polish pass. Classify each entry as fix-in-place, promote-to-spec, or won't-fix; resolve; delete. |
| `web-development`                | When writing web code. Stack idioms, `/llms.txt` doc links.                                                               |
| `web-verification`               | When verifying web UI in a browser. Wraps the Chrome DevTools MCP.                                                        |
| `ios-development`                | When writing iOS code. SwiftUI + `@Observable` + Swift Testing idioms, HIG link list.                                     |
| `ios-simulator-control`          | When verifying iOS UI changes. Wraps `xcrun simctl` + `idb`.                                                              |
| `android-development`            | When writing Android code. Compose + Material 3 + Kotlin flow idioms.                                                     |
| `android-emulator-control`       | When verifying Android UI changes. Wraps `adb` + `uiautomator`.                                                           |

These skills are deliberately lighter than the official `superpowers` suite and adapted to this template's spec-driven shape (no plan documents, no branch ceremony, reverse pointers everywhere). Several lift patterns from superpowers; see each `SKILL.md` header for attribution.

## Docs site

The `docs/` directory is a VitePress site that renders `specs/` and `features/` (plus the root `index.md`) into a navigable, searchable, hyperlinked reference. Sidebar is auto-generated from the filesystem; spec frontmatter renders as a metadata banner; `[NEEDS CLARIFICATION]` markers render as styled inline tags.

```sh
mise run docs:dev          # local dev server
mise run docs:build        # static build to docs/.vitepress/dist
mise run docs:preview      # preview the built site
```

The site reads markdown directly from `specs/` and `features/` (no copying, no symlinks) via VitePress's `srcDir`. Authoring stays in the canonical locations.

## MCP servers

- **Chrome DevTools** (`.mcp.json`) — DOM, console, screenshots, network, lighthouse. Configured for **Chromium**, not Chrome. Use it aggressively when debugging or verifying web visuals; running it in a tight verify-iterate loop is the intended workflow.

For iOS and Android simulator control, see the `ios-simulator-control` and `android-emulator-control` skills (zsh-based recipes around `xcrun simctl`/`idb` and `adb`/`uiautomator`).

## Local tooling

`mise` manages tool versions and tasks. The root `mise.toml` covers docs tasks; per-platform tools and tasks live in `apps/*/mise.toml` and `services/*/mise.toml` once you scaffold them.

```sh
mise run docs:dev          # docs site
mise tasks                 # list everything available
```

When you add a platform, also add the orchestration task at the root (`web:dev`, `ios:test`, etc.) so cross-platform commands work.

## Editing

All editing is done via your editor of choice. Builds, tests, simulators, and emulators are driven through `mise` tasks — do not assume an IDE will launch them.

## What lives where

| Question                                           | Where to look                                                   |
| -------------------------------------------------- | --------------------------------------------------------------- |
| "What's a spec ID look like?"                      | `specs/CONVENTIONS.md`                                          |
| "How do I add a new feature?"                      | `specs/CONVENTIONS.md` → "Adding a new feature"                 |
| "What's the web stack?"                            | `apps/web/CLAUDE.md` (after scaffolding)                        |
| "How do I run iOS tests?"                          | `apps/ios/mise.toml` + `apps/ios/CLAUDE.md` (after scaffolding) |
| "How do I write a user story?"                     | `.claude/skills/writing-user-stories/SKILL.md`                  |
| "How do I take a screenshot of the iOS simulator?" | `.claude/skills/ios-simulator-control/SKILL.md`                 |
