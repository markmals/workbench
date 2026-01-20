package bootstrap

import (
	"context"
	"os"
	"path/filepath"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/projectdef"
	"github.com/markmals/workbench/internal/shell"
	"github.com/markmals/workbench/internal/ui"
)

// Website bootstraps a website project.
type Website struct {
	Dir    string
	Config *config.Config
}

// InstallDependencies installs npm dependencies using pnpm.
func (w *Website) InstallDependencies(ctx context.Context) error {
	def, err := projectdef.Get("website")
	if err != nil {
		return err
	}

	runner := shell.New(w.Dir)

	// If a package.json exists, prefer honoring it directly to preserve upstream scripts.
	if _, err := os.Stat(filepath.Join(w.Dir, "package.json")); err == nil {
		return ui.RunWithSpinner(ctx, "Installing dependencies", func() error {
			return runner.Run(ctx, "pnpm", "install")
		})
	}

	// Determine conditions based on config (fallback path)
	var conditions []string
	if w.Config.Website != nil && w.Config.Website.Deployment.Target != "" {
		conditions = append(conditions, w.Config.Website.Deployment.Target)
	}

	// Get all dependencies
	deps := def.Dependencies.AllDeps(conditions...)
	devDeps := def.Dependencies.AllDevDeps(conditions...)

	// Install runtime dependencies with spinner
	args := append([]string{"add"}, deps...)
	err = ui.RunWithSpinner(ctx, "Installing dependencies", func() error {
		return runner.Run(ctx, "pnpm", args...)
	})
	if err != nil {
		return err
	}

	// Install dev dependencies with spinner
	devArgs := append([]string{"add", "-D"}, devDeps...)
	err = ui.RunWithSpinner(ctx, "Installing dev dependencies", func() error {
		return runner.Run(ctx, "pnpm", devArgs...)
	})
	if err != nil {
		return err
	}

	return nil
}
