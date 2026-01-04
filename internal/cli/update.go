package cli

import (
	"fmt"

	"github.com/markmals/workbench/internal/i18n"
)

// UpdateCmd updates managed files from templates.
type UpdateCmd struct {
	Templates string `help:"Template source (ref or path)" name:"templates"`
	Check     bool   `help:"Exit non-zero if updates would change files" name:"check"`
	Diff      bool   `help:"Print unified diffs for managed files" name:"diff"`
}

func (c *UpdateCmd) Run(ctx *Context) error {
	ctx.Logger.Info("updating managed files", "templates", c.Templates, "check", c.Check)

	// TODO: Implement update
	// 1. Load config
	// 2. Pull/update templates cache
	// 3. Re-render managed files
	// 4. Apply merge logic for sensitive files
	// 5. If --check, exit non-zero if changes
	// 6. If --diff, print diffs

	fmt.Println(i18n.T("UpdateNotImplemented"))
	return nil
}
