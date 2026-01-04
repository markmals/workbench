# Archiving Projects

Workbench makes it easy to archive completed projects to GitHub, keeping your workspace clean while preserving your work.

## What is Archiving?

Archiving moves a project to a dedicated GitHub organization and marks it as archived. This:

- Frees up local disk space
- Keeps your main GitHub organized
- Preserves the project for future reference
- Makes restoration easy when needed

## Archive a Project

From within a project directory:

```bash
wb archive
```

Or specify a directory:

```bash
wb archive ./old-project
```

### What Happens

1. Pushes all local changes to the remote
2. Transfers the repository to the archive organization
3. Marks the repository as archived on GitHub
4. Optionally deletes the local directory

### Archive Organization

By default, projects are archived to `markmals-archive`. You can specify a different organization:

```bash
wb archive --org my-archive-org
```

## Options

### Keep Local Copy

By default, the local directory is deleted after archiving. To keep it:

```bash
wb archive --keep-local
```

### Skip Confirmation

Archive without prompting for confirmation:

```bash
wb archive -y
```

### Dry Run

See what would happen without actually archiving:

```bash
wb archive --dry-run
```

Output:

```
Would archive ./my-project to markmals-archive/my-project
Would delete local directory ./my-project
```

## Prerequisites

Before archiving, ensure:

1. **GitHub CLI is authenticated** with permission to transfer repos
2. **Archive organization exists** and you have push access
3. **All changes are committed** (Workbench will warn about uncommitted changes)

## After Archiving

The archived repository will be:

- Transferred to the archive organization
- Marked as read-only (archived status)
- Visible on GitHub for reference

To bring it back, use [`wb restore`](/guide/restoring).

## Best Practices

### When to Archive

- Project is complete and stable
- You haven't touched it in months
- You need to free up disk space
- You want to declutter your GitHub

### When Not to Archive

- Active development is ongoing
- Others are contributing
- You need the project regularly

### Naming Conventions

Keep original repository names when archiving. This makes restoration intuitive:

```bash
# Archived as markmals-archive/my-website
wb archive ./my-website
```
