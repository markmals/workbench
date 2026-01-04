# Getting Started

This guide will walk you through creating your first project with Workbench.

## Prerequisites

Before you begin, make sure you have:

- [mise](https://mise.jdx.dev/) installed
- [GitHub CLI](https://cli.github.com/) (`gh`) installed and authenticated
- macOS or Linux (Windows is not currently supported)

## Installation

The quickest way to install Workbench is via Homebrew:

```bash
brew install markmals/tap/workbench
```

See the [Installation guide](/guide/installation) for other installation methods.

## Create Your First Project

### Interactive Mode

Run `wb init` to start the interactive project wizard:

```bash
wb init my-project
```

You'll be prompted to choose:

1. **Project type** - website, tui, or ios
2. **Features** - optional additions like Convex backend
3. **Deployment target** - for website projects

The wizard guides you through each step with sensible defaults.

### Non-Interactive Mode

For scripting or when you know what you want, use flags:

```bash
wb init my-website --kind website --deployment cloudflare
```

## Project Structure

After initialization, your project will have:

```
my-project/
├── .workbench.toml    # Project configuration
├── mise.toml          # Tool versions and tasks
├── README.md          # Project documentation
├── AGENTS.md          # AI agent guidelines
└── ...                # Project-specific files
```

## Running Your Project

Workbench projects use mise for task management:

```bash
# Install dependencies
mise install

# Start development server (for websites)
mise run dev

# Build for production
mise run build
```

## Adding Features

As your project evolves, you can add features:

```bash
# Add Convex backend
wb add convex
```

Or remove them:

```bash
# Remove Convex
wb rm convex
```

## Next Steps

- [Project Types](/guide/project-types) - Learn about each project template
- [Features](/guide/features) - Explore available features
- [Commands Reference](/commands/init) - Full command documentation
