package cli

import (
	"fmt"
)

// AddCmd adds a feature to the project.
type AddCmd struct {
	Feature string `arg:"" help:"Feature to add (e.g., agents.codex, vscode.base)"`
	Pkg     string `help:"Target package in monorepo" name:"pkg"`
	Yes     bool   `help:"Skip confirmation" short:"y"`
}

func (c *AddCmd) Run(ctx *Context) error {
	ctx.Logger.Info("adding feature", "feature", c.Feature, "pkg", c.Pkg)

	// TODO: Implement add
	// 1. Load config
	// 2. Look up feature in registry
	// 3. Apply feature
	// 4. Update config

	fmt.Println("wb add is not yet implemented")
	return nil
}
