# Project Types

Workbench supports multiple project types, each with its own template and feature set.

## Website

Create modern web applications with React and Vite.

```bash
wb init my-site --kind website
```

### Stack

- **Framework**: React 19 with TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS
- **Package Manager**: pnpm

### Deployment Targets

| Target | Description |
|--------|-------------|
| `cloudflare` | Cloudflare Pages with edge functions |
| `railway` | Railway with Docker deployment |

```bash
# Create with Cloudflare deployment
wb init my-site --kind website --deployment cloudflare
```

### Optional Features

- **Convex** - Real-time backend with database and functions
- **Claude** - AI agent support with Claude Code skills
- **Codex** - AI agent support with OpenAI Codex CLI

### Generated Structure

```
my-site/
├── src/
│   ├── app/
│   │   ├── routes/
│   │   │   └── index.tsx
│   │   └── root.tsx
│   └── main.tsx
├── public/
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── tsconfig.json
├── mise.toml
└── .workbench.toml
```

## TUI

Create terminal user interfaces with Go and Bubble Tea.

```bash
wb init my-cli --kind tui
```

### Stack

- **Language**: Go
- **TUI Framework**: [Bubble Tea](https://github.com/charmbracelet/bubbletea)
- **Components**: [Bubbles](https://github.com/charmbracelet/bubbles)
- **Styling**: [Lip Gloss](https://github.com/charmbracelet/lipgloss)
- **Prompts**: [Huh](https://github.com/charmbracelet/huh)
- **CLI Parsing**: [Kong](https://github.com/alecthomas/kong)

### Optional Features

- **Claude** - AI agent support with Claude Code skills
- **Codex** - AI agent support with OpenAI Codex CLI

### Generated Structure

```
my-cli/
├── cmd/
│   └── app/
│       └── main.go
├── internal/
│   ├── cli/
│   │   └── cli.go
│   ├── tui/
│   │   └── model.go
│   └── ui/
│       └── styles.go
├── go.mod
├── go.sum
├── mise.toml
└── .workbench.toml
```

## iOS

Create iOS applications with Swift and SwiftUI.

```bash
wb init my-app --kind ios
```

### Stack

- **Language**: Swift
- **UI Framework**: SwiftUI
- **Minimum iOS**: 17.0

### Optional Features

- **Convex** - Real-time backend with Swift SDK
- **Claude** - AI agent support with Claude Code skills
- **Codex** - AI agent support with OpenAI Codex CLI

### Generated Structure

```
my-app/
├── MyApp/
│   ├── App.swift
│   ├── ContentView.swift
│   ├── Assets.xcassets/
│   └── Info.plist
├── MyApp.xcodeproj/
├── mise.toml
└── .workbench.toml
```

## Choosing a Project Type

| If you want to build... | Use |
|------------------------|-----|
| Web application or SPA | `website` |
| CLI tool or terminal app | `tui` |
| iPhone or iPad app | `ios` |

All project types share common features:

- mise for tool management
- AGENTS.md for AI assistants
- .workbench.toml for project configuration
- Consistent project structure
