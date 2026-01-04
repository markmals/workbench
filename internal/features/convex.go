package features

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
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
		fmt.Println("Would add Convex to project")
		return nil
	}

	// Create convex directory
	convexDir := filepath.Join(ctx.Dir, "convex")
	if err := os.MkdirAll(convexDir, 0755); err != nil {
		return fmt.Errorf("creating convex directory: %w", err)
	}

	// Create schema.ts
	schemaPath := filepath.Join(convexDir, "schema.ts")
	schemaContent := `import { defineSchema } from "convex/server";

export default defineSchema({
  // Define your tables here
});
`
	if err := os.WriteFile(schemaPath, []byte(schemaContent), 0644); err != nil {
		return fmt.Errorf("creating schema.ts: %w", err)
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
		fmt.Println("Would remove Convex from project")
		return nil
	}

	// Remove convex directory
	convexDir := filepath.Join(ctx.Dir, "convex")
	if err := os.RemoveAll(convexDir); err != nil {
		return fmt.Errorf("removing convex directory: %w", err)
	}

	// Remove from features list
	ctx.Config.RemoveFeature("convex")

	// Save config
	if err := config.Save(ctx.Dir, ctx.Config); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	return nil
}

