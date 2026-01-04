# Features

Features are optional capabilities that can be added to your Workbench projects. They can be enabled during project creation or added later.

## Available Features

### Convex

A real-time backend platform with database, functions, and file storage.

```bash
# Add during init
wb init my-project --kind website --convex

# Add to existing project
wb add convex
```

**What it adds:**

- Convex configuration files
- Database schema setup
- Function definitions
- TypeScript/Swift SDK integration

**Supported project types:** `website`, `ios`

### Claude

Claude Code agent support with pre-configured skills for AI-assisted development.

```bash
wb add claude
```

**What it adds:**

- `.claude/` directory with agent configuration
- Pre-built skills for common tasks
- CLAUDE.md guidelines for the AI agent

**Skills included:**

- `homebrew-release` - Automate Homebrew formula releases

**Supported project types:** `website`, `tui`, `ios`

### Codex

OpenAI Codex CLI agent support with skills for AI-assisted development.

```bash
wb add codex
```

**What it adds:**

- `.codex/` directory with agent configuration
- Pre-built skills for common tasks

**Supported project types:** `website`, `tui`, `ios`

## Adding Features

Use `wb add` to add a feature to an existing project:

```bash
wb add convex
```

### Dry Run

See what would be changed without making modifications:

```bash
wb add convex --dry-run
```

### Skip Confirmation

Accept changes without prompting:

```bash
wb add convex -y
```

## Removing Features

Use `wb rm` to remove a feature:

```bash
wb rm convex
```

This removes:

- Configuration files added by the feature
- Dependencies (where applicable)
- Updates to project configuration

### Dry Run

```bash
wb rm convex --dry-run
```

## Feature Configuration

Features are tracked in `.workbench.toml`:

```toml
[project]
kind = "website"
name = "my-project"

[project.features]
convex = true
claude = true
```

## Feature Dependencies

Some features have prerequisites:

| Feature | Requires |
|---------|----------|
| Convex | Node.js runtime |
| Claude | Claude Code CLI |
| Codex | OpenAI Codex CLI |

Workbench will warn you if prerequisites aren't met.

## Custom Features

Feature definitions live in TOML files under `internal/projectdef/defs/`. Each project type declares which features it supports:

```toml
[features.convex]
description = "Real-time backend with Convex"

[features.convex.remove]
directories = ["convex"]
files = ["convex.json"]
```

This makes it easy to extend Workbench with new features.
