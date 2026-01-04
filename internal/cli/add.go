package cli

import (
	"fmt"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/features"
	"github.com/markmals/workbench/internal/i18n"
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
			return fmt.Errorf(i18n.T("ErrNoFeaturesAvailable", i18n.M{"Feature": c.Feature, "Kind": cfg.Kind}))
		}
		fmt.Println(i18n.T("UnknownFeature", i18n.M{"Feature": c.Feature}))
		fmt.Println()
		fmt.Println(i18n.T("AvailableFeaturesHeader", i18n.M{"Kind": cfg.Kind}))
		for _, f := range available {
			status := ""
			if cfg.HasFeature(f.Name()) {
				status = i18n.T("FeatureInstalledSuffix")
			}
			fmt.Printf("  %-12s  %s%s\n", f.Name(), f.Description(), status)
		}
		return nil
	}

	// Check if applicable
	if !feature.Applies(cfg) {
		return fmt.Errorf(i18n.T("ErrFeatureNotApplicable", i18n.M{"Feature": c.Feature, "Kind": cfg.Kind}))
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
		fmt.Println(i18n.T("DryRunComplete"))
	} else {
		fmt.Println(i18n.T("FeatureAdded", i18n.M{"Feature": c.Feature}))
	}
	return nil
}
