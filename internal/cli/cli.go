package cli

import (
	"github.com/charmbracelet/log"
)

// CLI defines the root command structure with global flags.
type CLI struct {
	// Global flags
	CWD     string `help:"Working directory" default:"." type:"path" name:"cwd"`
	JSON    bool   `help:"Output machine-readable JSON" name:"json"`
	Verbose bool   `help:"Enable verbose logging" short:"v"`

	// Commands
	Home    HomeCmd    `cmd:"" default:"withargs" hidden:""`
	Init    InitCmd    `cmd:"" help:"Create a new project"`
	Archive ArchiveCmd `cmd:"" help:"Archive repo to GitHub org"`
	Restore RestoreCmd `cmd:"" help:"Restore repo from archive org"`
	Add     AddCmd     `cmd:"" help:"Add a feature to the project"`
	Rm      RmCmd      `cmd:"" help:"Remove a feature from the project"`
	// Update  UpdateCmd  `cmd:"" help:"Update managed files from templates"` // TODO: Implement
	Version VersionCmd `cmd:"" help:"Show version information"`
}

// Context holds shared dependencies injected into command Run() methods.
type Context struct {
	CLI    *CLI
	Logger *log.Logger
}
