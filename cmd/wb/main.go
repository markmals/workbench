package main

import (
	"os"

	"github.com/alecthomas/kong"
	"github.com/markmals/workbench/internal/assets"
	"github.com/markmals/workbench/internal/cli"
	"github.com/markmals/workbench/internal/logx"
)

func main() {
	var c cli.CLI
	ctx := kong.Parse(&c,
		kong.Name("wb"),
		kong.Description("A personal CLI to bootstrap, evolve, and archive/restore projects."),
		kong.UsageOnError(),
		kong.Help(func(options kong.HelpOptions, ctx *kong.Context) error {
			assets.PrintLogo()
			return kong.DefaultHelpPrinter(options, ctx)
		}),
	)

	// Set up logger based on flags
	logger := logx.New(c.JSON, c.Verbose)

	// Bind dependencies for injection into Run() methods
	err := ctx.Run(&cli.Context{
		CLI:    &c,
		Logger: logger,
	})
	if err != nil {
		if c.JSON {
			logger.Error("command failed", "error", err)
		} else {
			logger.Error(err.Error())
		}
		os.Exit(1)
	}
}
