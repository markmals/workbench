# wb rm

Remove a feature from a project.

## Synopsis

```bash
wb rm <feature> [flags]
```

## Description

The `rm` command removes a feature from your Workbench project. This deletes the associated configuration files and updates the project configuration.

## Arguments

| Argument | Description |
|----------|-------------|
| `feature` | Feature to remove (e.g., `convex`, `claude`, `codex`) |

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

### Remove Convex

```bash
wb rm convex
```

### Preview Changes

```bash
wb rm convex --dry-run
```

Output:

```
Would remove feature: convex
  Delete: convex/
  Delete: convex.json
  Update: .workbench.toml
```

### Skip Confirmation

```bash
wb rm convex -y
```

## What Gets Removed

Each feature defines what to remove in its project definition:

### Convex

- `convex/` directory
- `convex.json` configuration

### Claude

- `.claude/` directory

### Codex

- `.codex/` directory

## Configuration Update

After removing a feature, `.workbench.toml` is updated:

```toml
[project.features]
# convex = true  <- removed
```

## Warnings

### Uncommitted Changes

If the files to be removed have uncommitted changes:

```
Warning: convex/ has uncommitted changes
Continue? [y/N]
```

### Data Loss

Removing features like Convex may delete data:

```
Warning: This will delete your Convex schema and functions.
This action cannot be undone. Continue? [y/N]
```

## Errors

### Feature not enabled

```
Error: feature "convex" is not enabled
```

The feature isn't part of your project.

### Files not found

```
Warning: Expected file convex.json not found, skipping
```

Some expected files may already be missing.

## See Also

- [wb add](/commands/add) - Add a feature
- [Features](/guide/features) - Available features documentation
