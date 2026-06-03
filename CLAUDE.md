# Spec-Driven Development Template

A spec-driven multiplatform app harness. Specs are the source of truth; every platform implements them natively. The reference platform is **web** (React + TanStack Start + Convex); other clients — websites, Apple, Android, Windows, Linux, and a CLI (Node, Rust, or Go) — mirror its behavior idiomatically. The backend is **Convex** (database, file storage, cron, queues, realtime) with **Clerk** for identity.

This template ships as the **superset** of every platform the stack supports — see [`STACK.md`](STACK.md) for the full toolchain catalog. On a fresh copy, run **`/setup`** first: it asks which platforms your product actually uses and prunes the skills, hooks, permissions, and docs for the rest. Whatever you keep, the `apps/` and `services/` directories are not committed — you scaffold each one when you start that platform's implementation. The harness assumes (and the per-platform skills are written for) the layout below.

@.claude/rules/code-quality.md
@.claude/rules/commit-discipline.md

## How this repo works

**Specs are the source of truth.** Domain models, view models, user flows, errors, stories — all live as markdown in `specs/` (cross-cutting) and `features/<NNNN>-<slug>/` (feature-scoped). There is no shared code. Every platform implements the spec natively. Reconciliation between platforms happens through agent-mediated regeneration, not through a shared library.

If you are tempted to create a shared package, write a spec instead.

**One product, or several related ones.** The default — and simplest — shape is a single product projected across platforms: the `apps/<platform>/` layout below, every client a native realization of the same specs. A monorepo may also hold **several logically-related apps**, not just platform projections of one product — distinct apps that share specs (a common domain, cross-cutting conventions) and, when they run on the same stack, code. Disambiguate them by **name**, not platform alone: a second CLI is not a second per-language folder bolted onto `apps/cli`, it is a second named app with its own platform projections, and `web` stays each app's reference platform. The cross-platform rule is unchanged either way — **platform projections of the same app share no code**; they share specs and reconcile through regeneration. Ordinary library reuse _between_ same-stack related apps is fine; it is cross-_platform_ sharing of one app's behavior that this template refuses.

**Read these before doing anything substantial:**

1. `specs/CONVENTIONS.md` — spec format, ID taxonomy, frontmatter, reverse pointers, deviation marker, drift detection. **This is the contract.**
2. `specs/ARCHITECTURE.md` — top-level layering, data flow, deployment targets, the contract-first backend model.
3. `specs/DESIGN_SYSTEM.md` — design tokens, component vocabulary, parity rules across platforms.
4. `STACK.md` — the canonical toolchain catalog: every tool, framework, and service this template knows how to wire up, organized by layer.
5. The platform `CLAUDE.md` for whichever app you're working on (`apps/<platform>/CLAUDE.md` or `services/convex/CLAUDE.md`) once you've scaffolded it.

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
├── STACK.md                           ← canonical toolchain catalog (rendered in docs)
├── docs/                              ← VitePress site rendering specs/, features/, STACK
│   └── index.md                       ← home page (URL-rewritten to /)
├── specs/                             ← cross-cutting specs (CONVENTIONS, ARCHITECTURE, DESIGN_SYSTEM)
├── features/                          ← (you create) feature-scoped specs as <NNNN>-<slug>/
├── apps/                              ← (you create) platform implementations
│   ├── web/                           ←   React + TanStack Start + Convex (reference)
│   ├── website/                       ←   Astro + React islands (marketing / content)
│   ├── ios/                           ←   Swift / UIKit / SwiftData (Apple family: iOS · iPadOS · macOS · …)
│   ├── android/                       ←   Kotlin / Jetpack Compose / Room
│   ├── windows/                       ←   C# / WinUI 3 / EF Core
│   ├── linux/                         ←   Rust / GTK 4 + Adwaita / Relm4
│   └── cli/                           ←   the CLI — one stack: Node (TS-Rest) · Rust (charmed_rust) · Go (Charm)
└── services/                          ← (you create) backend services
    └── convex/                        ←   Convex backend (schema is the data-layer protocol) + Clerk auth
```

## Working with specs

- **Reverse pointers are mandatory.** Every implementation file/class/function that realizes a spec carries `// SPEC: <id>`. Tests are tagged with the spec IDs they verify. See `specs/CONVENTIONS.md` for the per-language form.
- **Reference platform first.** Build new features on your reference platform first (web, by default). Other platforms use that implementation as a worked example alongside the spec.
- **Platform divergence is allowed but explicit.** Use `// SPEC: <id> (deviates: <reason>)` when a platform must differ.
- **Stories use Gherkin acceptance criteria.** See the `writing-user-stories` skill. Scenarios have stable sub-IDs that tests trace back to.

## Slash commands

| Command                               | Purpose                                                                                                       |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `/setup`                              | **Run once on a fresh copy.** Asks which platforms + backend you're using and prunes everything for the rest. |
| `/sdd-apply <spec-id> <platform>`     | Regenerate a spec's implementation + tests on a platform.                                                     |
| `/sdd-verify <platform>`              | Run the platform's behavioral test suite.                                                                     |
| `/sdd-drift <platform>`               | List spec IDs whose impl is stale, plus impl files with no spec pointer.                                      |
| `/sdd-reconcile <platform>`           | Bring the spec + other platforms in line with this platform's impl.                                           |
| `/sdd-cover <spec-id>`                | Show which platforms implement a spec and which tests pass.                                                   |
| `/sdd-challenge <spec-id> <platform>` | Adversarially review a spec's implementation on a platform — try to break it. Read-only audit.                |
| `/sdd-defect <platform> <desc>`       | File a sub-spec defect into `apps/<platform>/DEFECTS.md` without breaking flow.                               |

These are scaffolded with intent docs; their internals are agent-driven (no automation yet — the agent uses `rg`, `Edit`, `AskUserQuestion`, etc.).

## Workflow skills

Procedural skills live under `.claude/skills/`. Use them rather than reaching for ad-hoc patterns — they encode the discipline this template expects.

| Skill                            | When to invoke                                                                                                                                                                                        |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `brainstorming-feature`          | Before starting any new feature or substantial change. Walks narrative → stories → models → view-models → flows → errors.                                                                             |
| `writing-user-stories`           | When authoring or reviewing a story file. Enforces Gherkin discipline.                                                                                                                                |
| `implementing-a-spec`            | The default "how to write code" workflow. Per-spec subagent dispatch + three-stage review (spec-compliance, code-quality, adversarial). Used by `/sdd-apply`.                                         |
| `test-driven-development`        | When writing any production code. Iron Law: no production code without a failing test first. Invariants ("for all") get a property-based test, not just examples.                                     |
| `adversarial-review`             | The refutational third review stage, after spec-compliance and code-quality pass. Fresh context, different model; assumes the code is broken and tries to break it. Invoked by `implementing-a-spec`. |
| `verification-before-completion` | Before claiming any work is complete. Run the verifying command in this turn; evidence before claims.                                                                                                 |
| `systematic-debugging`           | When encountering any bug or unexpected behavior. Find the root cause before proposing a fix.                                                                                                         |
| `triaging-defects`               | When `apps/<platform>/DEFECTS.md` is non-empty and you're in a polish pass. Classify each entry as fix-in-place, promote-to-spec, or won't-fix; resolve; delete.                                      |
| `web-development`                | When writing web-app code. React + TanStack suite + Convex + Tailwind + React Aria idioms, `/llms.txt` doc links.                                                                                     |
| `web-verification`               | When verifying web UI in a browser. Wraps the Chrome DevTools MCP.                                                                                                                                    |
| `website-development`            | When writing marketing/content site code. Astro + React islands + content collections idioms.                                                                                                         |
| `ios-development`                | When writing Apple-family code. UIKit (AppKit on macOS, SwiftUI on watchOS) + Observation + SwiftData + Swift Testing idioms, HIG link list.                                                          |
| `ios-simulator-control`          | When verifying Apple UI changes. Wraps `xcrun simctl` + `idb`.                                                                                                                                        |
| `android-development`            | When writing Android code. Compose + Material 3 + Kotlin flow + Room + Ktor idioms.                                                                                                                   |
| `android-emulator-control`       | When verifying Android UI changes. Wraps `adb` + `uiautomator`.                                                                                                                                       |
| `windows-development`            | When writing Windows code. C# + WinUI 3 + XAML + MVVM Toolkit + EF Core idioms.                                                                                                                       |
| `linux-development`              | When writing Linux desktop code. Rust + GTK 4 + Adwaita + Relm4 + Diesel idioms.                                                                                                                      |
| `server-cli-development`         | When writing the **Node** CLI stack (`apps/cli`). TS-Rest + Bombshell (args/clack/tab) + Drizzle + plainjob idioms; single-file exe packaging. Hosts the API in OpenAPI mode.                  |
| `rust-cli-development`           | When writing the **Rust** CLI stack (`apps/cli`). Clap + charmed_rust (bubbletea/bubbles/lipgloss/huh/glamour/harmonica/wish) + Diesel + reqwest + Progenitor idioms.                          |
| `go-cli-development`             | When writing the **Go** CLI stack (`apps/cli`). Cobra/Fang + Bubble Tea + Bubbles + Lip Gloss + Huh + Glamour + database/sql (go-sqlite) + oapi-codegen idioms.                                |

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

- **Chrome DevTools** (`.mcp.json`) — DOM, console, screenshots, network, lighthouse. Configured for **Chromium**, not Chrome. Use it aggressively when debugging or verifying web visuals; running it in a tight verify-iterate loop is the intended workflow. This one is committed because it's `npx`-launched and self-contained.

- **Per-platform IDE bridges (user/local config, not committed).** Several toolchains expose the IDE to the agent over MCP — building, testing, and reading the code model with structured results. These are **per-machine** (they need the IDE running with the feature enabled), so configure them in your user or `.mcp.local.json`, never the shared `.mcp.json`:
    - **Apple** — Xcode's [external agent access](https://developer.apple.com/documentation/xcode/giving-external-agents-access-to-xcode). See `ios-development`.
    - **Android** — Android Studio / IntelliJ [MCP Server](https://www.jetbrains.com/help/idea/mcp-server.html#external-client-setup). See `android-development`.
    - **Windows** — [RoslynMcpExtension](https://github.com/sailro/RoslynMcpExtension) for the C# code model. See `windows-development`.

For Apple and Android simulator control, see the `ios-simulator-control` and `android-emulator-control` skills (zsh-based recipes around `xcrun simctl`/`idb` and `adb`/`uiautomator`). Windows, Linux, and CLI verification is documented in-skill (no GUI-automation MCP) — each `*-development` skill carries a "Verifying" section.

## Local tooling

`mise` manages tool versions and tasks. The root `mise.toml` covers docs tasks; per-platform tools and tasks live in `apps/*/mise.toml` and `services/*/mise.toml` once you scaffold them.

```sh
mise run docs:dev          # docs site
mise tasks                 # list everything available
```

When you add a platform, define `fmt` and `lint` tasks in its `mise.toml` (its formatter / linter; `fmt` accepts optional file paths) — the `format-on-edit` and `stop-lint` hooks dispatch to them, which is what keeps those hooks platform-agnostic. Also add the orchestration task at the root (`web:dev`, `ios:test`, etc.) so cross-platform commands work.

## Editing

All editing is done via your editor of choice. Builds, tests, simulators, and emulators are driven through `mise` tasks — do not assume an IDE will launch them.

## What lives where

| Question                                           | Where to look                                                      |
| -------------------------------------------------- | ------------------------------------------------------------------ |
| "What's a spec ID look like?"                      | `specs/CONVENTIONS.md`                                             |
| "How do I add a new feature?"                      | `specs/CONVENTIONS.md` → "Adding a new feature"                    |
| "What tool/framework does the template use for X?" | `STACK.md`                                                         |
| "Which platforms is this copy set up for?"         | Run `/setup`, or read the platform rows in `specs/ARCHITECTURE.md` |
| "What's the web stack?"                            | `STACK.md` → Web Apps; `apps/web/CLAUDE.md` (after scaffolding)    |
| "How do I run iOS tests?"                          | `apps/ios/mise.toml` + `apps/ios/CLAUDE.md` (after scaffolding)    |
| "How do I write a user story?"                     | `.claude/skills/writing-user-stories/SKILL.md`                     |
| "How do I take a screenshot of the iOS simulator?" | `.claude/skills/ios-simulator-control/SKILL.md`                    |
