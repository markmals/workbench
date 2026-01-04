package cli

import (
	"fmt"
)

// InitCmd creates a new project.
type InitCmd struct {
	Dir            string `help:"Target directory" default:"." type:"path" name:"dir"`
	Name           string `help:"Project name (used for repo/package naming)" name:"name"`
	NonInteractive bool   `help:"Require flags/defaults; no prompts" name:"non-interactive"`
	Templates      string `help:"Template source (ref or path)" name:"templates"`
	Monorepo       bool   `help:"Force monorepo layout" name:"monorepo"`
	Yes            bool   `help:"Accept defaults and skip confirmations" short:"y"`
}

func (c *InitCmd) Run(ctx *Context) error {
	ctx.Logger.Info("initializing project", "dir", c.Dir, "name", c.Name)

	// TODO: Implement project initialization
	// 1. Gather inputs (prompt or flags)
	// 2. Validate option combinations
	// 3. Build render context
	// 4. Apply features
	// 5. Write config and files

	fmt.Println("wb init is not yet implemented")
	return nil
}
