package assets

import (
	_ "embed"
	"fmt"
	"os"

	"golang.org/x/term"
)

//go:embed workbench-logo.ansi
var logo []byte

// PrintLogo prints the workbench logo to stdout.
// Only prints if stdout is a terminal.
func PrintLogo() {
	if !term.IsTerminal(int(os.Stdout.Fd())) {
		return
	}
	fmt.Print(string(logo))
}
