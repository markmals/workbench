package cli

import (
	"fmt"
)

// ArchiveCmd archives a repo to the GitHub Archive org.
type ArchiveCmd struct {
	Dir        string `arg:"" optional:"" help:"Directory to archive" default:"." type:"path"`
	Org        string `help:"GitHub org to archive to" default:"markmals-archive" name:"org"`
	KeepRemote bool   `help:"Don't change existing origin; just add a remote" name:"keep-remote"`
	Yes        bool   `help:"Skip confirmation" short:"y"`
	DryRun     bool   `help:"Show what would happen without doing it" name:"dry-run"`
}

func (c *ArchiveCmd) Run(ctx *Context) error {
	ctx.Logger.Info("archiving project", "dir", c.Dir, "org", c.Org, "dry-run", c.DryRun)

	// TODO: Implement archive
	// 1. Verify target is a git repo
	// 2. Verify git tree is clean
	// 3. Create repo in Archive org via gh
	// 4. Push HEAD + tags
	// 5. Remove local directory (with confirmation)

	fmt.Println("wb archive is not yet implemented")
	return nil
}
