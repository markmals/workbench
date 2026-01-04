package features

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/shell"
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
		fmt.Println("  - Run convex dev --once --configure=new")
		return nil
	}

	runner := shell.New(ctx.Dir)
	bgCtx := context.Background()

	// Install convex package
	fmt.Println("Installing convex...")
	if err := runner.Run(bgCtx, "pnpm", "add", "convex"); err != nil {
		return fmt.Errorf("installing convex: %w", err)
	}

	// Run convex dev to create convex/ directory with starter files
	fmt.Println("Initializing convex...")
	if err := runner.Run(bgCtx, "pnpm", "dlx", "convex", "dev", "--once", "--configure=new"); err != nil {
		return fmt.Errorf("running convex dev: %w", err)
	}

	// Add to features list
	ctx.Config.AddFeature("convex")

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

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

	// Uninstall convex package
	fmt.Println("Removing convex package...")
	if err := runner.Run(bgCtx, "pnpm", "remove", "convex"); err != nil {
		// Don't fail if package wasn't installed
		fmt.Printf("Note: %v\n", err)
	}

	// Remove from features list
	ctx.Config.RemoveFeature("convex")

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	return nil
}

