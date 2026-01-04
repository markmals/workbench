# Project Plan

> workbench (wb)

A personal CLI to bootstrap, evolve, and archive/restore projects using my own templates and standards.

## Goals

- **Fast project creation** that matches _my_ workflows (CLI-first + VS Code-first).
- **Deterministic, repeatable output**: every project has a machine-readable config that explains how it was generated.
- **Composable features**: `wb init` + `wb add/rm` can converge on the same end state.
- **Template-driven updates**: `wb update` can refresh boilerplate (AGENTS.md, mise.toml, .vscode/\*, etc.) based on project config.
- **Safe archiving/restoring** using my GitHub “Archive” org.
- **Go stdlib first**, except where explicitly chosen (Kong + Charm libs, etc.).
- **Fail safe** for destructive ops (`archive`): require clean git tree + explicit confirmation (or `--yes`).
- **Explicit config over inference**: write a project config file on `init` and keep it authoritative.
- **Idempotent-ish**: repeated `wb update` should be a no-op if nothing changed.## Proposed CLI shape

## Non-goals

- Supporting arbitrary third-party workflows.
- Being a general-purpose project generator for other people.
- Re-implementing git hosting features when `gh` can do it reliably.## Principles

Binary: `wb`

Primary commands:

- `wb init`
- `wb archive <dir?>`
- `wb restore <repo-name>`
- `wb add <feature>`
- `wb rm <feature>`
- `wb update`

Global flags (applies to most commands):

- `--cwd <dir>`: operate as if run from here.
- `--json`: output machine-readable status/errors (useful for agent runs).
- `--verbose`

### 1) Arg parsing

Use **Kong** for consistent flags/subcommands. Keep command handlers thin; move logic into `internal/*`.

### 2) GitHub integration

Use the **GitHub CLI (`gh`)** as the integration layer instead of a Go GitHub SDK:

- Authentication is already solved (`gh auth login`)
- Moving repos between orgs and creating repos is straightforward

### 3) Templates source

Support multiple template providers:

1. **Embedded bootstrap templates** (Go `embed`) so `wb init` works immediately.
2. **Templates repo cache** (cloned/pulled into `~/.cache/workbench/templates/<ref>`), used by:
    - `wb update` (default)
    - `wb init --templates=latest` (optional)

Templates are rendered with Go `text/template` (stdlib) plus a small helper func map.

### 4) Project configuration

Write a config file into the project on `init`. This is the “source of truth” for `update` and `add/rm`.

Recommended path:

- `.workbench/config.json(c)`

Config contains:

- project kind(s): website / tui / ios / monorepo
- website options: deployment, mode, data
- tui options: chosen Charm libs
- ios options: Tuist, data backend
- agent options: Codex / Claude Code / Gemini CLI
- enabled features list
- template version/ref used (for reproducibility)

## Repository layout

```
workbench/
    cmd/wb/
        main.go
    internal/
        app/ # wiring: kong -> handlers
        config/ # load/save/validate .workbench/config.json(c)
        prompt/ # interactive prompts (Huh) + non-interactive defaults
        templates/ # template providers + renderer
        features/ # feature modules, apply/remove
        gitx/ # git checks: is repo? clean tree? top-level?
        ghx/ # wrappers around gh CLI
        vscode/ # .vscode generation/merge
        mise/ # mise.toml generation/merge
        agents/ # AGENTS.md/CLAUDE.md/.gemini settings generation
        ops/ # filesystem operations with safety rails
        logx/ # structured logging + json mode
        templates/ # embedded bootstrap templates (minimal set)
    go.mod
    README.md
    AGENTS.md
    CLAUDE.md
    PLAN.md
```

## Command behavior specs

### `wb init`

**Creates a new project** (or monorepo) by:

- prompting for project type and options
- rendering templates accordingly
- creating `.workbench/config.json(c)`
- writing:
    - `mise.toml`
    - `.vscode/settings.json` and `.vscode/extensions.json`
    - agent files (AGENTS.md + related)
    - stack-specific starter files (website/tui/ios)
- ensures the result is usable from:
    - terminal-only
    - VS Code-only (as the single editor)

Flags:

- `--dir <path>`: target directory (default: `.`).
- `--name <project-name>`: used for repo/package naming.
- `--non-interactive`: require flags/defaults; no prompts.
- `--templates <ref|path>`: choose template source.
- `--monorepo`: force monorepo path even with one package.
- `--yes`: accept defaults and skip confirmations.

Implementation outline:

- Gather inputs (prompt or flags)
- Validate option combinations (e.g., “Static files only allowed for SPA +/- prerender”)
- Build a “render context” object
- Apply features (each feature is a module)
- Write config and files
- Print a short summary (and in `--json`, emit config path + applied features)

### `wb archive <dir?>`

**Archives a repo to GitHub Archive org and deletes local copy**.

Rules:

- Target must be a git repo.
- Git tree must be clean.
- Default target dir is current directory.
- Must confirm destructive delete unless `--yes`.

Behavior:

- Determine repo name from folder (or `--repo-name`)
- Create repo in Archive org via `gh repo create <org>/<name> --private` (or keep visibility configurable)
- Add remote `archive` or set origin to archive
- Push `HEAD` + tags
- Optionally record a local index entry (`~/.local/state/workbench/archive.json`) for convenience
- Remove directory with `os.RemoveAll`

Flags:

- `--org <name>` (default: `Archive`)
- `--keep-remote`: don’t change existing origin; just add a remote
- `--yes`
- `--dry-run`

### `wb restore <repo-name>`

**Clones a repo from Archive org into cwd**.

Behavior:

- `gh repo clone <org>/<repo> <dest>`
- If `--rm`:
    - delete the repo from the Archive org after successful clone (`gh repo delete ... --yes`)
- Optionally run `wb update` after clone if config exists (behind `--update` flag)

Flags:

- `--org <name>` (default: `Archive`)
- `--dest <dir>` (default: `./<repo-name>`)
- `--rm`
- `--dry-run`

### `wb add <feature>` / `wb rm <feature>`

**Applies/removes a feature** in an existing project based on `.workbench/config.json(c)`.

Key points:

- Features are named (e.g., `agents.codex`, `vscode.base`, `website.cloudflare-workers`, `data.convex`)
- `add` updates config + applies templates/files
- `rm` updates config + removes or disables (prefer “disable” if removal is unsafe)

Flags:

- `--pkg <name>` for monorepo package targeting
- `--yes`

### `wb update`

**Pulls latest templates** and re-applies derived files:

- AGENTS.md (+ CLAUDE.md alias rules)
- `.gemini/settings.json` as needed
- `mise.toml`
- `.vscode/*`
- any “managed” boilerplate files tracked by the template system

Rules:

- Reads `.workbench/config.json(c)`
- Re-renders managed files
- For merge-sensitive files:
    - prefer safe merges (e.g., merge VS Code extension recommendations)
    - never overwrite user content outside managed sections unless explicitly opted in

Flags:

- `--templates <ref|path>`
- `--check`: exit non-zero if updates would change files (for CI-ish use)
- `--diff`: print unified diffs for managed files## Feature system

A **feature module** has:

- `Name() string`
- `Applies(cfg) bool` (or driven by config flags)
- `Apply(ctx) error`
- `Remove(ctx) error` (optional; sometimes “disable” only)

Features should be small and composable:

- `base.gitignore`
- `base.readme`
- `mise.base`
- `vscode.base`
- `agents.codex`, `agents.claude`, `agents.gemini`
- `website.<deployment>`, `website.<mode>`, `data.<provider>`
- `tui.charm.<lib>`
- `ios.tuist`, `ios.data.<provider>`## Safety rails & UX

- Every command returns structured errors with:
    - a human message
    - a stable error code (for `--json`)
- `archive` and `rm`-style operations:
    - show exactly what will happen
    - require confirmation unless `--yes`
- Detect missing dependencies early with clear messaging:
    - `git`
    - `gh`
    - `mise` (optional but recommended; warn if absent)

### Milestone 1 — Skeleton + plumbing

- Initialize Go module, Kong wiring, global flags
- Logging + JSON output mode
- `.workbench/config.json(c)` schema + validation
- Template renderer + embedded templates

### Milestone 2 — `wb init` (minimum viable)

- Interactive prompts (Huh) + `--non-interactive`
- Generate:
    - config
    - mise.toml (base)
    - .vscode settings/extensions (base)
    - AGENTS.md (based on agent selection)
- One project type end-to-end (recommend starting with **TUI** since it exercises Kong + Charm libs cleanly)

### Milestone 3 — Feature system + `add/rm`

- Implement feature registry
- Ensure `init` uses the same feature modules as `add`

### Milestone 4 — `archive` / `restore`

- `git` checks (repo exists, clean tree)
- `gh` wrapper + dry-run mode
- Safe delete + confirmations

### Milestone 5 — `update`

- Templates repo cache
- Managed-file re-rendering
- Merge logic for `.vscode/extensions.json` and similar## Bootstrap template set (embedded)

Keep the embedded set intentionally small:

- `.gitignore`
- `README.md` (minimal)
- `.vscode/settings.json`
- `.vscode/extensions.json`
- `mise.toml`
- `AGENTS.md` (with sections toggled by agent selection)
- `.workbench/config.json(c)` (written by code, not templated)

Everything beyond that comes from the templates repo cache once available.

## Definition of “done” for first usable version

- `wb init` produces a working project directory with:
    - config present
    - mise + vscode + agents files generated
    - at least one stack scaffold (tui or website) that builds
- `wb add` can add at least one additional feature and update files accordingly
- `wb archive` refuses unsafe states and succeeds on a clean repo

## Stacks

- TypeScript
    - React for interactive UI and application logic
    - React Router for routing and server-side rendering React applications
    - TanStack Query for asynchronous state, caching, and synchronization when necessary
    - Tailwind CSS for utility-first styling
    - Node for development tooling
    - Convex for real-time backend data, functions, and auth-adjacent state
    - Cloudflare Workers for deployment
        - Cloudflare Workers KV optional
        - Cloudflare D1 optional
        - Cloudflare R2 optional
    - OR Railway Node for deployment
        - Redis on Railway optional
        - Postgres on Railway optional
        - Railway Buckets optional
    - Use Vitest for all testing
- Go
    - Use the standard library whenever possible (except for the libraries I say to use)
    - CSP concurrency for all I/O-heavy workloads
    - Bubble Tea, Bubbles, Lip Gloss, and more for terminal UIs and CLIs
    - Small, statically linked, single-file binaries for CLI distribution
    - Use Go’s “testing” library for all testing
- Swift
    - UIKit for user interfaces on iOS
    - Reactive state management using [`@Observable`](https://developer.apple.com/documentation/Observation/Observable) via the [Swift.Observation](https://developer.apple.com/documentation/Observation) module
    - [Utilize the new iOS 26 UIKit `updateProperties()` method for reactive state changes](https://developer.apple.com/documentation/uikit/updating-views-automatically-with-observation-tracking)
    - Simple on-device data management with [pointfreeco/sqlite-data](https://github.com/pointfreeco/sqlite-data)
    - Use declarative extensions from my [“Cider” Swift library](https://github.com/markmals/Cider/blob/main/Sources/Cider/UIKit/NSLayoutConstraint%2BBuilder.swift)
    - Use Convex when the application is centered around it and convert it into an async stream or write to an observable property
    - Diffable Collection View for all lists
    - iOS (iPhone) as primary target
    - Native system integration via Swift Concurrency and system frameworks
    - Use the new modern [Swift Testing framework](https://developer.apple.com/xcode/swift-testing/) for all testing
    - Build apps, run apps in the simulator, test apps, sign apps, and deploy apps to TestFlight and the App Store via CLI commands (Tuist, xcodebuild, fastlane, xcrun, swift, whatever we need to use) and alias the most popular/common ones as Mise tasks - Never make the user open Xcode
- Rust
    - Shared business logic can be written in Rust and shared with Swift (UniFFI) and TypeScript (wasm-bindgen)
    - If a desktop app is necessary, we can also provide a Tauri wrapper for cross-platform desktop apps on macOS, Windows, and Linux
    - Core systems logic with strong correctness and safety guarantees
    - Performance-critical or long-lived background processes
