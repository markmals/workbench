# What is Workbench?

Workbench (`wb`) is my CLI for managing the full lifecycle of projects—scaffolding, evolution, and archival.

## What It Does

- **`wb init`** - Scaffold a new project with my preferred stack
- **`wb add/rm`** - Toggle features as the project evolves
- **`wb archive`** - Ship a finished project to `markmals-archive` and clean up locally
- **`wb restore`** - Bring it back when I need it again

## Key Decisions

### mise Everywhere

Every project uses [mise](https://mise.jdx.dev/) for tool versions and task running. No more "works on my machine" issues between my devices.

### Declarative Templates

Project definitions live in TOML. Easy to tweak, easy to understand when I come back months later.

### Project Types

| Type | Stack |
|------|-------|
| `website` | React/Vite + Tailwind, optional Convex |
| `tui` | Go + Bubble Tea + Kong |
| `ios` | Swift + SwiftUI |

## Architecture Notes

- CLI parsing: Kong
- Templates: Go's `text/template` with embedded FS
- Config: `.workbench.toml` in project root
- Project definitions: `internal/projectdef/defs/*.toml`
