package features

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/shell"
	"github.com/markmals/workbench/internal/ui"
)

func init() {
	Register(&ConvexFeature{})
}

// ConvexFeature adds Convex backend support to website projects.
type ConvexFeature struct{}

func (f *ConvexFeature) Name() string {
	return "convex"
}

func (f *ConvexFeature) Description() string {
	return "Convex real-time backend with functions and auth"
}

func (f *ConvexFeature) Applies(cfg *config.Config) bool {
	return cfg.Kind == "website"
}

func (f *ConvexFeature) Apply(ctx *Context) error {
	if ctx.DryRun {
		fmt.Println("Would add Convex to project:")
		fmt.Println("  - Install convex package")
		return nil
	}

	runner := shell.New(ctx.Dir)
	bgCtx := context.Background()

	// Install convex package with spinner
	err := ui.RunWithSpinner(bgCtx, "Installing convex", func() error {
		return runner.Run(bgCtx, "pnpm", "add", "convex")
	})
	if err != nil {
		return err
	}

	// Add to features list
	ctx.Config.AddFeature("convex")

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	fmt.Println("\nTo complete Convex setup, run:")
	fmt.Println("  convex dev")

	return nil
}

func (f *ConvexFeature) Remove(ctx *Context) error {
	if ctx.DryRun {
		fmt.Println("Would remove Convex from project:")
		fmt.Println("  - Remove convex directory")
		fmt.Println("  - Uninstall convex package")
		return nil
	}

	runner := shell.New(ctx.Dir)
	bgCtx := context.Background()

	// Remove convex directory
	convexDir := filepath.Join(ctx.Dir, "convex")
	if err := os.RemoveAll(convexDir); err != nil {
		return fmt.Errorf("removing convex directory: %w", err)
	}

	// Uninstall convex package with spinner
	_ = ui.RunWithSpinner(bgCtx, "Removing convex package", func() error {
		return runner.Run(bgCtx, "pnpm", "remove", "convex")
	})
	// Don't fail if package wasn't installed

	// Remove from features list
	ctx.Config.RemoveFeature("convex")

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	return nil
}

