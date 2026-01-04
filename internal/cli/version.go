package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime/debug"

	"github.com/markmals/workbench/internal/i18n"
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

	fmt.Println(i18n.T("VersionOutput", i18n.M{"Version": v.Version}))
	fmt.Println(i18n.T("VersionCommit", i18n.M{"Commit": v.Commit}))
	fmt.Println(i18n.T("VersionBuilt", i18n.M{"BuildDate": v.BuildDate}))
	fmt.Println(i18n.T("VersionGo", i18n.M{"GoVersion": v.GoVersion}))
	return nil
}
