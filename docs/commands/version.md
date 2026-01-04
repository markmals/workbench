# wb version

Display version information.

## Synopsis

```bash
wb version [flags]
```

## Description

The `version` command displays the Workbench version, build information, and Go runtime version.

## Flags

### Global Flags

| Flag | Description |
|------|-------------|
| `--cwd` | Working directory (default: `.`) |
| `--json` | Output machine-readable JSON |
| `-v, --verbose` | Enable verbose logging |
| `-h, --help` | Show help |

## Examples

### Display Version

```bash
wb version
```

Output:

```
wb 0.1.0
  commit:  abc1234
  built:   2024-01-15T10:30:00Z
  go:      go1.21.5
```

### JSON Output

```bash
wb version --json
```

```json
{
  "version": "0.1.0",
  "commit": "abc1234",
  "buildDate": "2024-01-15T10:30:00Z",
  "goVersion": "go1.21.5"
}
```

## Version Information

| Field | Description |
|-------|-------------|
| `version` | Semantic version number |
| `commit` | Git commit hash or tap user |
| `built` | Build timestamp (ISO 8601) |
| `go` | Go runtime version |

## Checking for Updates

To check if a newer version is available:

```bash
# If installed via Homebrew
brew outdated workbench

# Update if available
brew upgrade workbench
```

## See Also

- [Installation](/guide/installation) - How to install and update
