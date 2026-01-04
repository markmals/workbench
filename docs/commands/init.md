# wb init

Create a new project with Workbench scaffolding.

## Synopsis

```bash
wb init [path] [flags]
```

## Description

The `init` command creates a new project with all necessary configuration files, dependencies, and boilerplate code. It can run interactively (prompting for options) or non-interactively with flags.

## Arguments

| Argument | Description |
|----------|-------------|
| `path` | Project directory (optional, defaults to current directory) |

## Flags

| Flag | Description |
|------|-------------|
| `--kind` | Project type: `website`, `tui`, `ios` |
| `--deployment` | Deployment target: `cloudflare`, `railway` (website only) |
| `--convex` / `--no-convex` | Include Convex backend |
| `--non-interactive` | Require all options via flags, no prompts |
| `--templates` | Custom template source (ref or path) |
| `-y, --yes` | Accept defaults and skip confirmations |

### Global Flags

| Flag | Description |
|------|-------------|
| `--cwd` | Working directory (default: `.`) |
| `--json` | Output machine-readable JSON |
| `-v, --verbose` | Enable verbose logging |
| `-h, --help` | Show help |

## Examples

### Interactive Mode

Start the project wizard:

```bash
wb init my-project
```

You'll be prompted for project type, features, and other options.

### Create in Current Directory

```bash
mkdir my-project && cd my-project
wb init
```

### Specify Project Type

```bash
wb init my-cli --kind tui
```

### Website with Deployment

```bash
wb init my-site --kind website --deployment cloudflare
```

### Include Convex Backend

```bash
wb init my-app --kind website --convex
```

### Non-Interactive

Skip all prompts by providing all required flags:

```bash
wb init my-project --kind website --deployment cloudflare --non-interactive
```

### Accept Defaults

```bash
wb init my-project --kind website -y
```

## Output

On success, `init` displays:

```
Created website project: my-project
Location: /Users/you/projects/my-project

Next steps:
  cd my-project
  mise install
  mise run dev
```

### JSON Output

With `--json`, returns structured output:

```json
{
  "path": "/Users/you/projects/my-project",
  "kind": "website",
  "name": "my-project",
  "features": ["convex"]
}
```

## Generated Files

Depending on project type, `init` creates:

| File | Purpose |
|------|---------|
| `.workbench.toml` | Workbench project configuration |
| `mise.toml` | Tool versions and tasks |
| `README.md` | Project documentation |
| `AGENTS.md` | AI agent guidelines |
| `.gitignore` | Git ignore patterns |

Plus project-specific files (see [Project Types](/guide/project-types)).

## See Also

- [Getting Started](/guide/getting-started) - Tutorial walkthrough
- [Project Types](/guide/project-types) - Available templates
- [Features](/guide/features) - Optional capabilities
