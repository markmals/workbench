package features

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/projectdef"
	"github.com/markmals/workbench/internal/shell"
	"github.com/markmals/workbench/internal/ui"
)

func init() {
	// Register all TOML-defined features from all project definitions
	for _, kind := range projectdef.List() {
		def, err := projectdef.Get(kind)
		if err != nil {
			continue
		}

		for name, feat := range def.Features {
			// Only register if not already registered (allows Go overrides)
			if Get(name) == nil {
				Register(&TOMLFeature{
					name:       name,
					kind:       kind,
					definition: feat,
				})
			}
		}
	}
}

// TOMLFeature is a feature defined entirely in TOML.
type TOMLFeature struct {
	name       string
	kind       string
	definition projectdef.Feature
}

func (f *TOMLFeature) Name() string {
	return f.name
}

func (f *TOMLFeature) Description() string {
	return f.definition.Description
}

func (f *TOMLFeature) Applies(cfg *config.Config) bool {
	return cfg.Kind == f.kind
}

func (f *TOMLFeature) Apply(ctx *Context) error {
	if ctx.DryRun {
		fmt.Printf("Would add %s to project:\n", f.name)
		if len(f.definition.Packages) > 0 {
			fmt.Printf("  - Install packages: %v\n", f.definition.Packages)
		}
		if len(f.definition.DevPackages) > 0 {
			fmt.Printf("  - Install dev packages: %v\n", f.definition.DevPackages)
		}
		return nil
	}

	runner := shell.New(ctx.Dir)
	bgCtx := context.Background()

	// Install runtime packages
	if len(f.definition.Packages) > 0 {
		args := append([]string{"add"}, f.definition.Packages...)
		err := ui.RunWithSpinner(bgCtx, fmt.Sprintf("Installing %s", f.name), func() error {
			return runner.Run(bgCtx, "pnpm", args...)
		})
		if err != nil {
			return err
		}
	}

	// Install dev packages
	if len(f.definition.DevPackages) > 0 {
		args := append([]string{"add", "-D"}, f.definition.DevPackages...)
		err := ui.RunWithSpinner(bgCtx, fmt.Sprintf("Installing %s dev deps", f.name), func() error {
			return runner.Run(bgCtx, "pnpm", args...)
		})
		if err != nil {
			return err
		}
	}

	// Add to features list
	ctx.Config.AddFeature(f.name)

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	// Show post-install message
	if f.definition.PostMessage != "" {
		fmt.Printf("\n%s\n", f.definition.PostMessage)
	}

	return nil
}

func (f *TOMLFeature) Remove(ctx *Context) error {
	remove := f.definition.Remove

	if ctx.DryRun {
		fmt.Printf("Would remove %s from project:\n", f.name)
		if len(remove.Directories) > 0 {
			fmt.Printf("  - Remove directories: %v\n", remove.Directories)
		}
		if len(remove.Packages) > 0 {
			fmt.Printf("  - Uninstall packages: %v\n", remove.Packages)
		}
		if len(remove.DevPackages) > 0 {
			fmt.Printf("  - Uninstall dev packages: %v\n", remove.DevPackages)
		}
		return nil
	}

	runner := shell.New(ctx.Dir)
	bgCtx := context.Background()

	// Remove directories
	for _, dir := range remove.Directories {
		dirPath := filepath.Join(ctx.Dir, dir)
		if err := os.RemoveAll(dirPath); err != nil {
			return fmt.Errorf("removing %s directory: %w", dir, err)
		}
	}

	// Remove packages (combined runtime and dev)
	allPackages := append(remove.Packages, remove.DevPackages...)
	if len(allPackages) > 0 {
		args := append([]string{"remove"}, allPackages...)
		_ = ui.RunWithSpinner(bgCtx, fmt.Sprintf("Removing %s packages", f.name), func() error {
			return runner.Run(bgCtx, "pnpm", args...)
		})
		// Don't fail if packages weren't installed
	}

	// Remove from features list
	ctx.Config.RemoveFeature(f.name)

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	return nil
}
