# wb archive

Archive a repository to a GitHub organization.

## Synopsis

```bash
wb archive [dir] [flags]
```

## Description

The `archive` command transfers a repository to your archive organization on GitHub, marks it as archived (read-only), and optionally removes the local copy. This helps keep your workspace clean while preserving projects for future reference.

## Arguments

| Argument | Description |
|----------|-------------|
| `dir` | Directory to archive (default: current directory) |

## Flags

| Flag | Description |
|------|-------------|
| `--org` | GitHub organization for archives (default: `markmals-archive`) |
| `--keep-local` | Don't delete local directory after archiving |
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

### Archive Current Project

```bash
wb archive
```

### Archive Specific Directory

```bash
wb archive ./old-project
```

### Keep Local Copy

```bash
wb archive --keep-local
```

### Custom Archive Organization

```bash
wb archive --org my-company-archive
```

### Preview Without Archiving

```bash
wb archive --dry-run
```

Output:

```
Would push all local changes
Would transfer repo to markmals-archive/my-project
Would mark repository as archived
Would delete local directory ./my-project
```

### Skip Confirmation

```bash
wb archive -y
```

## Process

The archive command performs these steps:

1. **Check prerequisites** - Verify GitHub CLI auth and org access
2. **Push changes** - Ensure all local changes are on the remote
3. **Transfer repository** - Move to archive organization
4. **Mark as archived** - Set read-only status on GitHub
5. **Clean up** - Remove local directory (unless `--keep-local`)

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated
- Access to both the source repo and archive organization
- Push access to transfer repositories

## Errors

### Uncommitted changes

```
Error: uncommitted changes in ./my-project
Please commit or stash your changes first
```

### Organization not accessible

```
Error: cannot access organization "my-archive-org"
Ensure you have the necessary permissions
```

### Repository already in archive

```
Error: repository already exists in markmals-archive
```

## See Also

- [wb restore](/commands/restore) - Restore an archived project
- [Archiving Projects](/guide/archiving) - Detailed guide
