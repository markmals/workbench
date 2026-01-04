package cli

import (
	"fmt"

	"github.com/markmals/workbench/internal/assets"
)

// HomeCmd is the default command shown when wb is run without arguments.
type HomeCmd struct{}

func (c *HomeCmd) Run(ctx *Context) error {
	assets.PrintLogo()
	fmt.Println("  A personal CLI to bootstrap, evolve, and archive/restore projects.")
	fmt.Println()
	fmt.Println("  Run 'wb --help' for usage information.")
	fmt.Println()
	return nil
}
