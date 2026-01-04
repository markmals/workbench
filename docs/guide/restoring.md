# Restoring Projects

Bring archived projects back to life with a single command.

## Restore a Project

Restore a project from your archive organization:

```bash
wb restore my-project
```

This will:

1. Clone the repository from the archive organization
2. Optionally unarchive it on GitHub (make it writable again)
3. Optionally remove it from the archive organization

## Options

### Custom Directory

Clone to a specific location:

```bash
wb restore my-project ./projects/my-project
```

### Unarchive on GitHub

Remove the archived status, making the repo writable:

```bash
wb restore my-project --unarchive
```

### Remove from Archive

Delete the repository from the archive org after restoring:

```bash
wb restore my-project --rm
```

### Full Restoration

Combine options for a complete restoration:

```bash
wb restore my-project --unarchive --rm
```

This restores the project, makes it writable, and cleans up the archive.

### Skip Confirmation

```bash
wb restore my-project -y
```

### Dry Run

```bash
wb restore my-project --dry-run
```

## Archive Organization

By default, projects are restored from `markmals-archive`. Specify a different organization:

```bash
wb restore my-project --org my-archive-org
```

## After Restoring

Once restored, the project is ready to use:

```bash
cd my-project
mise install
mise run dev
```

The `.workbench.toml` file preserves all project configuration, so Workbench commands work immediately.

## Workflow Example

### Complete Archive & Restore Cycle

```bash
# Archive a completed project
wb archive ./my-website -y

# ... months later ...

# Restore when needed again
wb restore my-website --unarchive --rm

# Back to development
cd my-website
mise run dev
```

## Troubleshooting

### Repository not found

Ensure the repository exists in the archive organization:

```bash
gh repo list markmals-archive
```

### Permission denied

Make sure GitHub CLI has permission to:

- Clone from the archive organization
- Delete repositories (if using `--rm`)
- Modify repository settings (if using `--unarchive`)

### Already exists locally

If the target directory already exists, Workbench will error:

```
Error: directory ./my-project already exists
```

Either remove it or specify a different path:

```bash
wb restore my-project ./my-project-restored
```
