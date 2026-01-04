# Contributor Guidelines

This repo builds a Go CLI/TUI for macOS + Linux.

## Agent workflow: issue tracking is Beads-first

We use **Beads** (`bd`) for task tracking in this repo.

**Rules**

- **All user stories go in Beads.**
- **All bugs go in Beads** as soon as they’re discovered.
- **All reasonable subtasks go in Beads** as we break work down.
- Track **as much as is reasonable** in Beads so long-horizon work stays coherent.

**Minimum expectations for agents**

- Start work by checking what’s unblocked: `bd ready`
- Create a Beads task for each meaningful unit of work before coding:
    - `bd create "Title" -p 1`
- When you discover follow-up work, create child tasks and link dependencies:
    - `bd dep add <child> <parent>`
- When a task is completed, mark it closed and leave a brief outcome note so future work has context.

**Quick start**

**Hierarchy**
Use epics/tasks/subtasks as needed:

- `bd-a3f8` (Epic)
- `bd-a3f8.1` (Task)
- `bd-a3f8.1.1` (Sub-task)

**Stealth mode (optional)**
If you want Beads locally without committing `.beads/` to the repo:

```bash
bd init --stealth
```

## Core principles

- Prefer the Go standard library unless a dependency is explicitly listed below.
- Keep the UX consistent across:
    - non-interactive CLI usage (flags, stdin/stdout)
    - interactive prompts (forms)
    - full-screen TUIs
- Keep binaries small and distributable (single executable).

## Allowed dependencies (explicit)

- CLI parsing: **Kong** (`github.com/alecthomas/kong`)
- TUI runtime: **Bubble Tea** (`github.com/charmbracelet/bubbletea`)
- TUI components: **Bubbles** (`github.com/charmbracelet/bubbles`)
- Styling/layout: **Lip Gloss** (`github.com/charmbracelet/lipgloss`)
- Prompts/forms: **Huh** (`github.com/charmbracelet/huh`)
- Logging: **charmbracelet/log** (`github.com/charmbracelet/log`)

If you think we need something else, stop and discuss with me why you think so before adding it.

## Recommended repo layout

- `cmd/<app>/main.go` — entrypoint (wire CLI, logging, config)
- `internal/cli/` — Kong command structs + Run() methods
- `internal/tui/` — Bubble Tea programs, models, components, styles
- `internal/workbench/` — core domain logic (file ops, templating, git, github, etc.)
- `internal/config/` — config loading, defaults, env vars
- `internal/ui/` — shared UI helpers (Lip Gloss styles, tables, rendering utils)

Keep domain logic out of `cmd/` and out of the Bubble Tea model where possible.

## Tooling expectations

### Go toolchain via `mise`

- Project should declare a Go version in `mise.toml`.
- Use `mise` exclusively for installing/switching Go versions.

### All Tasks go through `mise`

- Mise is used exclusively for running tasks.
- If you use a CLI command often, consider adding it to a task.
- If you use a CLI command with complex dependencies, consider adding it to a task.

## CLI architecture (Kong)

### Pattern

- Define a root CLI struct with nested command structs.
- **Every leaf command must implement `Run(...) error`.**
- Parse once, then call `ctx.Run(...)` to execute the selected command.
- Use Kong bindings to pass shared dependencies (logger, config, filesystem, git client, etc.) into `Run(...)`.

### Guidelines

- Command structs: only flags/args + minimal wiring.
- `Run(...)` should delegate to `internal/workbench` services.
- Avoid command-string switching; prefer `Run(...)` methods for stability.

### Hooks

If you need lifecycle behavior (e.g. enable debug logging early), prefer Kong hooks (`BeforeApply`, etc.) rather than manual parsing logic.

## Logging (charmbracelet/log)

- Use `github.com/charmbracelet/log` for all logging (no `fmt.Printf` for diagnostics).
- Default level should be `info`; allow elevation via a flag/env.
- Prefer structured key/value logging for operational output.
- Pick formatter by environment:
    - default `TextFormatter` for human TTY use
    - `JSONFormatter` for machine-readable mode
    - `LogfmtFormatter` for simple structured logs
- For TUIs: do not log to stdout while the TUI is running. Log to a file or stderr only when safe.

## TUI architecture (Bubble Tea)

### Model shape

- Follow the Elm-style model with `Init`, `Update`, `View`.
- Keep `Update` pure: update state, return commands; don’t do blocking I/O inside `Update`.
- Use Bubble Tea commands for I/O and long-running work.

### Composition

- Treat Bubbles components as sub-models (e.g. list, viewport, spinner).
- Prefer a single top-level “app model” that delegates to sub-models.

### Debugging

- If interactive debugging is needed, use Delve headless mode.
- For runtime diagnostics inside a TUI, log to a file.

## Prompts and forms (Huh)

Use Huh for interactive setup flows (project init, feature add/rm, etc.) when a full-screen TUI isn’t necessary.

### Accessibility

- Support screen-reader friendly mode via an env var or config.
- When accessible mode is enabled, prefer standard prompts over full TUIs.

### Dynamic forms

- Use title/options functions when later questions depend on earlier answers.
- Avoid re-running expensive dynamic option loaders too frequently.

## Styling & layout (Lip Gloss)

- Centralize styles in `internal/ui/styles.go` (or similar).
- Use Lip Gloss layout utilities for composing views:
    - joining blocks horizontally/vertically
    - measuring rendered width/height
    - placing blocks in whitespace (centering, corners)

- Avoid hardcoding assumptions about terminal color support—Lip Gloss adapts to terminal profiles.
- If rendering to non-stdio outputs (SSH, pipes, etc.), use a custom renderer.

## Bubbles components usage

Prefer Bubbles for common interactions rather than inventing new widgets:

- list browsing + filtering + help
- viewport scrolling
- spinner/progress
- table rendering for structured data

Wrap them in small adapters so domain logic never depends on UI types.

## Output rules (important)

- CLI output intended for piping must go to stdout and be stable.
- Logs, progress, spinners, debug output should go to stderr (or a log file for TUIs).
- Provide `--json` (or similar) output modes where it makes sense for scripting.

## Concurrency rules (Go)

- Use `context.Context` for cancellation across all I/O-heavy work.
- Prefer simple goroutines + channels; keep ownership clear.
- Ensure every goroutine has a clear shutdown path.
