package cli

import (
	"errors"
	"fmt"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/features"
	"github.com/markmals/workbench/internal/i18n"
)

// RmCmd removes a feature from the project.
type RmCmd struct {
	Feature string `arg:"" help:"Feature to remove (e.g., convex)"`
	DryRun  bool   `help:"Show what would be done without making changes" name:"dry-run"`
	Yes     bool   `help:"Skip confirmation" short:"y"`
}

func (c *RmCmd) Run(ctx *Context) error {
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
		return errors.New(i18n.T("ErrUnknownFeature", i18n.M{"Feature": c.Feature}))
	}

	// Remove feature
	fctx := &features.Context{
		Dir:    dir,
		Config: cfg,
		DryRun: c.DryRun,
	}
	if err := feature.Remove(fctx); err != nil {
		return fmt.Errorf("removing feature: %w", err)
	}

	if c.DryRun {
		fmt.Println(i18n.T("DryRunComplete"))
	} else {
		fmt.Println(i18n.T("FeatureRemoved", i18n.M{"Feature": c.Feature}))
	}
	return nil
}
