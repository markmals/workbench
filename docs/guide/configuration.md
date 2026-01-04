# Configuration

::: warning WIP
Config format is changing from `.workbench/config.jsonc` to `.workbench.toml`. See [wb-dt6](https://github.com/markmals/workbench).
:::

## Configuration File

Every Workbench project has a `.workbench.toml` file:

```toml
[project]
kind = "website"
name = "my-project"

[project.features]
convex = true
claude = true

[project.website]
deployment = "cloudflare"
```

## Configuration Options

### Project Section

| Field | Type | Description |
|-------|------|-------------|
| `kind` | string | Project type (`website`, `tui`, `ios`) |
| `name` | string | Project name (inferred from directory) |

### Features Section

Boolean flags for enabled features:

```toml
[project.features]
convex = true
claude = false
codex = false
```

### Website Section

Website-specific configuration:

```toml
[project.website]
deployment = "cloudflare"  # or "railway"
```

## Environment Variables

Some Workbench behavior can be controlled via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `WB_ARCHIVE_ORG` | GitHub org for archives | `markmals-archive` |
| `WB_TEMPLATES` | Custom template source | (built-in) |

## mise Integration

Workbench projects use mise for tool management. The `mise.toml` file is generated based on project type:

```toml
[tools]
node = "22"
pnpm = "latest"

[tasks.dev]
run = "pnpm dev"

[tasks.build]
run = "pnpm build"
```

## Global Configuration

Workbench currently doesn't have a global configuration file. All configuration is per-project.

Future versions may add:

- `~/.config/workbench/config.toml` for defaults
- Custom template repositories
- Default feature selections

## Editing Configuration

You can edit `.workbench.toml` directly, but it's recommended to use the CLI commands:

```bash
# Add a feature
wb add convex

# Remove a feature
wb rm convex
```

These commands ensure all related files are updated correctly.

## Validation

Workbench validates configuration when running commands. Invalid configurations will produce helpful error messages:

```
Error: invalid configuration: unknown project kind "webapp"
```
