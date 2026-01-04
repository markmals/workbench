package cli

import (
	"fmt"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/features"
	// Import features to register them
	_ "github.com/markmals/workbench/internal/features"
)

// AddCmd adds a feature to the project.
type AddCmd struct {
	Feature string `arg:"" help:"Feature to add (e.g., convex)"`
	DryRun  bool   `help:"Show what would be done without making changes" name:"dry-run"`
	Yes     bool   `help:"Skip confirmation" short:"y"`
}

func (c *AddCmd) Run(ctx *Context) error {
	// Resolve project directory
	dir, err := filepath.Abs(ctx.CLI.CWD)
	if err != nil {
		return fmt.Errorf("resolving directory: %w", err)
	}

	// Load config
	cfg, err := config.Load(dir)
	if err != nil {
		return fmt.Errorf("loading config: %w", err)
	}

	// Look up feature
	feature := features.Get(c.Feature)
	if feature == nil {
		// List available features
		available := features.ListApplicable(cfg)
		if len(available) == 0 {
			return fmt.Errorf("unknown feature: %s (no features available for %s projects)", c.Feature, cfg.Kind)
		}
		fmt.Printf("Unknown feature: %s\n\nAvailable features for %s projects:\n", c.Feature, cfg.Kind)
		for _, f := range available {
			status := ""
			if cfg.HasFeature(f.Name()) {
				status = " (installed)"
			}
			fmt.Printf("  %-12s  %s%s\n", f.Name(), f.Description(), status)
		}
		return nil
	}

	// Check if applicable
	if !feature.Applies(cfg) {
		return fmt.Errorf("feature %s does not apply to %s projects", c.Feature, cfg.Kind)
	}

	// Apply feature
	fctx := &features.Context{
		Dir:    dir,
		Config: cfg,
		DryRun: c.DryRun,
	}
	if err := feature.Apply(fctx); err != nil {
		return fmt.Errorf("applying feature: %w", err)
	}

	if c.DryRun {
		fmt.Println("Dry run complete.")
	} else {
		fmt.Printf("✓ Added %s\n", c.Feature)
	}
	return nil
}
