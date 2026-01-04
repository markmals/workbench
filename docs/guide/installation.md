# Installation

## Homebrew (Recommended)

```bash
brew install markmals/tap/workbench
```

This installs pre-built bottles and automatically installs dependencies (`mise`, `gh`).

### Updating

```bash
brew upgrade workbench
```

## From Source

```bash
go install github.com/markmals/workbench/cmd/wb@latest
```

Or clone and build:

```bash
git clone https://github.com/markmals/workbench
cd workbench
go build -o wb ./cmd/wb
```

::: warning
Building from source doesn't install dependencies. You'll need to manually install [mise](https://mise.jdx.dev/) and [GitHub CLI](https://cli.github.com/).
:::

## Verify

```bash
wb version
```

```
wb 0.1.0
  commit:  abc1234
  built:   2024-01-01T00:00:00Z
  go:      go1.21.0
```

## Project-Specific Tools

These are installed automatically by mise when you run `mise install` in a project:

| Tool | Project Types |
|------|--------------|
| Node.js, pnpm | website |
| Go | tui |
| Xcode | ios (manual) |
