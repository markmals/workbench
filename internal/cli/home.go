package cli

import (
	"fmt"

	"github.com/markmals/workbench/internal/assets"
	"github.com/markmals/workbench/internal/i18n"
)

// HomeCmd is the default command shown when wb is run without arguments.
type HomeCmd struct{}

func (c *HomeCmd) Run(ctx *Context) error {
	assets.PrintLogo()
	fmt.Println("  " + i18n.T("AppTagline"))
	fmt.Println()
	fmt.Println("  " + i18n.T("RunHelpHint"))
	fmt.Println()
	return nil
}
