# wb restore

Restore a repository from an archive organization.

## Synopsis

```bash
wb restore <repo> [dir] [flags]
```

## Description

The `restore` command clones a repository from your archive organization, optionally unarchiving it on GitHub and removing the archived copy. This brings archived projects back to active development.

## Arguments

| Argument | Description |
|----------|-------------|
| `repo` | Repository name to restore (without org prefix) |
| `dir` | Directory to clone into (default: repo name) |

## Flags

| Flag | Description |
|------|-------------|
| `--org` | GitHub organization to restore from (default: `markmals-archive`) |
| `--rm` | Delete repository from archive after restoring |
| `--unarchive` | Unarchive the repo on GitHub (make it writable) |
| `-y, --yes` | Skip confirmation prompts |
| `--dry-run` | Show what would happen without doing it |

### Global Flags

| Flag | Description |
|------|-------------|
| `--cwd` | Working directory (default: `.`) |
| `--json` | Output machine-readable JSON |
| `-v, --verbose` | Enable verbose logging |
| `-h, --help` | Show help |

## Examples

### Basic Restore

```bash
wb restore my-project
```

Clones to `./my-project`.

### Restore to Custom Directory

```bash
wb restore my-project ./projects/restored-project
```

### Full Restoration

Restore, unarchive, and clean up the archive:

```bash
wb restore my-project --unarchive --rm
```

### From Custom Organization

```bash
wb restore my-project --org company-archive
```

### Preview Restoration

```bash
wb restore my-project --dry-run
```

Output:

```
Would clone markmals-archive/my-project to ./my-project
```

With `--unarchive --rm`:

```
Would clone markmals-archive/my-project to ./my-project
Would unarchive repository on GitHub
Would delete markmals-archive/my-project
```

## Process

The restore command performs these steps:

1. **Clone repository** - From the archive organization
2. **Unarchive** (if `--unarchive`) - Remove read-only status
3. **Delete from archive** (if `--rm`) - Clean up archived copy

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- Access to the archive organization
- Delete permissions (if using `--rm`)

## Errors

### Repository not found

```
Error: repository "my-project" not found in markmals-archive
```

Check the repository name and organization.

### Directory exists

```
Error: directory ./my-project already exists
```

Either remove the existing directory or specify a different path.

### Permission denied

```
Error: permission denied to delete repository
```

You need admin access to use `--rm`.

## See Also

- [wb archive](/commands/archive) - Archive a project
- [Restoring Projects](/guide/restoring) - Detailed guide
