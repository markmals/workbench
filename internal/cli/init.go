package cli

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/assets"
	"github.com/markmals/workbench/internal/bootstrap"
	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/prompt"
	"github.com/markmals/workbench/internal/templates"
)

// InitCmd creates a new project.
type InitCmd struct {
	Path           string `arg:"" optional:"" help:"Project path (e.g. '.', './my-project')"`
	Kind           string `help:"Project type: website, tui, ios" name:"kind"`
	Deployment     string `help:"Deployment target: cloudflare, railway" name:"deployment"`
	Convex         bool   `help:"Include Convex backend" name:"convex" negatable:""`
	NonInteractive bool   `help:"Require flags/defaults; no prompts" name:"non-interactive"`
	Templates      string `help:"Template source (ref or path)" name:"templates"`
	Yes            bool   `help:"Accept defaults and skip confirmations" short:"y"`
}

func (c *InitCmd) Run(ctx *Context) error {
	// Show logo
	assets.PrintLogo()

	// Build defaults from flags
	var defaults *config.Config
	if c.Path != "" || c.Kind != "" {
		defaults = &config.Config{
			Path: c.Path,
			Kind: c.Kind,
		}
		if c.Convex {
			defaults.AddFeature("convex")
		}
		if c.Kind == "website" {
			defaults.Website = &config.WebsiteConfig{
				Deployment: c.Deployment,
			}
		}
	}

	// Run prompts or use defaults
	opts := prompt.Options{
		NonInteractive: c.NonInteractive,
		Defaults:       defaults,
	}

	result, err := prompt.Run(opts)
	if err != nil {
		// Silent exit on user abort (ctrl+c)
		if err.Error() == "user aborted" {
			return nil
		}
		return fmt.Errorf("gathering input: %w", err)
	}

	// Convert to config
	cfg := result.ToConfig()

	// Resolve path and infer name
	projectPath := cfg.Path
	if projectPath == "" {
		projectPath = "."
	}
	absDir, err := filepath.Abs(projectPath)
	if err != nil {
		return fmt.Errorf("resolving directory: %w", err)
	}

	// Infer name from directory
	cfg.Name = filepath.Base(absDir)

	ctx.Logger.Debug("initializing project", "dir", absDir, "name", cfg.Name)

	// Validate
	if err := config.Validate(cfg); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
	}

	// Create project directory if it doesn't exist
	if err := os.MkdirAll(absDir, 0755); err != nil {
		return fmt.Errorf("creating directory: %w", err)
	}

	// Save config
	if err := config.Save(absDir, cfg); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	ctx.Logger.Debug("saved config", "path", config.ConfigPath(absDir))

	// Render templates
	renderer := templates.Bootstrap()
	renderCtx := &templates.RenderContext{
		Name:     cfg.Name,
		Kind:     cfg.Kind,
		Features: cfg.Features,
	}

	if cfg.Website != nil {
		renderCtx.Website = &templates.WebsiteContext{
			Deployment: cfg.Website.Deployment,
		}
	}

	// Render common templates
	templateFiles := map[string]string{
		"bootstrap/gitignore.tmpl": ".gitignore",
		"bootstrap/readme.tmpl":    "README.md",
		"bootstrap/mise.tmpl":      "mise.toml",
		"bootstrap/agents.tmpl":    "AGENTS.md",
	}

	// Add vscode templates
	templateFiles["bootstrap/vscode-settings.tmpl"] = ".vscode/settings.json"
	templateFiles["bootstrap/vscode-extensions.tmpl"] = ".vscode/extensions.json"

	// Add website-specific templates
	if cfg.Kind == "website" {
		templateFiles["bootstrap/package.tmpl"] = "package.json"
		templateFiles["bootstrap/tsconfig.tmpl"] = "tsconfig.json"
		templateFiles["bootstrap/vite-config.tmpl"] = "vite.config.ts"
		templateFiles["bootstrap/react-router-config.tmpl"] = "react-router.config.ts"
		templateFiles["bootstrap/biome.tmpl"] = "biome.jsonc"
		templateFiles["bootstrap/prettierignore.tmpl"] = ".prettierignore"
		templateFiles["bootstrap/prettier-config.tmpl"] = "prettier.config.ts"
	}

	for tmpl, dest := range templateFiles {
		destPath := filepath.Join(absDir, dest)
		if err := renderer.RenderTo(tmpl, renderCtx, destPath); err != nil {
			ctx.Logger.Warn("failed to render template", "template", tmpl, "error", err)
			continue
		}
		ctx.Logger.Debug("rendered", "file", dest)
	}

	// Install dependencies for website projects
	if cfg.Kind == "website" {
		wb := &bootstrap.Website{
			Dir:    absDir,
			Config: cfg,
		}
		if err := wb.InstallDependencies(context.Background()); err != nil {
			return fmt.Errorf("installing dependencies: %w", err)
		}
	}

	// Print success message
	if ctx.CLI.JSON {
		// JSON output handled by caller
	} else {
		fmt.Printf("\n✓ Created %s project: %s\n", cfg.Kind, cfg.Name)
		fmt.Printf("  Location: %s\n", absDir)
		fmt.Println("\nNext steps:")
		if projectPath != "." {
			fmt.Println("  cd", projectPath)
		}
		fmt.Println("  mise install")
		fmt.Println("  mise run dev")
	}

	return nil
}
