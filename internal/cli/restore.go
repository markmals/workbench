package cli

import (
	"fmt"
)

// RestoreCmd restores a repo from the GitHub Archive org.
type RestoreCmd struct {
	Repo   string `arg:"" help:"Repository name to restore"`
	Org    string `help:"GitHub org to restore from" default:"markmals-archive" name:"org"`
	Dest   string `help:"Destination directory" name:"dest"`
	Rm     bool   `help:"Delete repo from archive after restore" name:"rm"`
	Update bool   `help:"Run wb update after restore" name:"update"`
	DryRun bool   `help:"Show what would happen without doing it" name:"dry-run"`
}

func (c *RestoreCmd) Run(ctx *Context) error {
	dest := c.Dest
	if dest == "" {
		dest = "./" + c.Repo
	}

	ctx.Logger.Info("restoring project", "repo", c.Repo, "org", c.Org, "dest", dest)

	// TODO: Implement restore
	// 1. Clone repo from archive org via gh
	// 2. If --rm, delete from archive after clone
	// 3. If --update and config exists, run wb update

	fmt.Println("wb restore is not yet implemented")
	return nil
}
