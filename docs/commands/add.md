# wb add

Add a feature to an existing project.

## Synopsis

```bash
wb add <feature> [flags]
```

## Description

The `add` command enables a feature in your Workbench project. This adds the necessary configuration files, dependencies, and code scaffolding.

## Arguments

| Argument | Description |
|----------|-------------|
| `feature` | Feature to add (e.g., `convex`, `claude`, `codex`) |

## Flags

| Flag | Description |
|------|-------------|
| `--dry-run` | Show what would be done without making changes |
| `-y, --yes` | Skip confirmation prompts |

### Global Flags

| Flag | Description |
|------|-------------|
| `--cwd` | Working directory (default: `.`) |
| `--json` | Output machine-readable JSON |
| `-v, --verbose` | Enable verbose logging |
| `-h, --help` | Show help |

## Examples

### Add Convex Backend

```bash
wb add convex
```

### Add Claude Support

```bash
wb add claude
```

### Preview Changes

```bash
wb add convex --dry-run
```

Output:

```
Would add feature: convex
  Create: convex/
  Create: convex.json
  Update: .workbench.toml
```

### Skip Confirmation

```bash
wb add convex -y
```

### From Different Directory

```bash
wb add convex --cwd ./my-project
```

## Available Features

| Feature | Description | Project Types |
|---------|-------------|---------------|
| `convex` | Real-time backend | website, ios |
| `claude` | Claude Code agent | all |
| `codex` | OpenAI Codex CLI | all |

## What Gets Added

### Convex

- `convex/` directory with schema and functions
- `convex.json` configuration
- Updated dependencies

### Claude

- `.claude/` directory
- `.claude/skills/` with built-in skills
- `CLAUDE.md` agent guidelines

### Codex

- `.codex/` directory
- `.codex/skills/` with built-in skills

## Configuration

After adding a feature, `.workbench.toml` is updated:

```toml
[project.features]
convex = true
```

## Errors

### Feature not available

```
Error: feature "unknown" is not available for project type "tui"
```

Check [Features](/guide/features) for available options per project type.

### Feature already enabled

```
Error: feature "convex" is already enabled
```

The feature is already part of your project.

## See Also

- [wb rm](/commands/rm) - Remove a feature
- [Features](/guide/features) - Available features documentation
