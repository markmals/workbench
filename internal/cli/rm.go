package cli

import (
	"fmt"
)

// RmCmd removes a feature from the project.
type RmCmd struct {
	Feature string `arg:"" help:"Feature to remove (e.g., agents.codex, vscode.base)"`
	Pkg     string `help:"Target package in monorepo" name:"pkg"`
	Yes     bool   `help:"Skip confirmation" short:"y"`
}

func (c *RmCmd) Run(ctx *Context) error {
	ctx.Logger.Info("removing feature", "feature", c.Feature, "pkg", c.Pkg)

	// TODO: Implement rm
	// 1. Load config
	// 2. Look up feature in registry
	// 3. Remove/disable feature
	// 4. Update config

	fmt.Println("wb rm is not yet implemented")
	return nil
}
