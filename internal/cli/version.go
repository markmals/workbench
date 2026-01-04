package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime/debug"
)

// Version information set at build time.
var (
	Version   = "dev"
	Commit    = "unknown"
	BuildDate = "unknown"
)

// VersionCmd shows version information.
type VersionCmd struct{}

type versionInfo struct {
	Version   string `json:"version"`
	Commit    string `json:"commit"`
	BuildDate string `json:"buildDate"`
	GoVersion string `json:"goVersion"`
}

func (c *VersionCmd) Run(ctx *Context) error {
	goVersion := "unknown"
	if info, ok := debug.ReadBuildInfo(); ok {
		goVersion = info.GoVersion
	}

	v := versionInfo{
		Version:   Version,
		Commit:    Commit,
		BuildDate: BuildDate,
		GoVersion: goVersion,
	}

	if ctx.CLI.JSON {
		enc := json.NewEncoder(os.Stdout)
		enc.SetIndent("", "  ")
		return enc.Encode(v)
	}

	fmt.Printf("wb %s\n", v.Version)
	fmt.Printf("  commit:  %s\n", v.Commit)
	fmt.Printf("  built:   %s\n", v.BuildDate)
	fmt.Printf("  go:      %s\n", v.GoVersion)
	return nil
}
