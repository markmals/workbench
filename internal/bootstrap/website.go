package bootstrap

import (
	"context"

	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/shell"
	"github.com/markmals/workbench/internal/ui"
)

// WebsiteDeps are the runtime dependencies for a website project.
var WebsiteDeps = []string{
	"@react-router/fs-routes",
	"@tailwindcss/vite",
	"clsx",
	"isbot",
	"react",
	"react-dom",
	"react-router",
	"zod",
}

// WebsiteDevDeps are the dev dependencies for a website project.
var WebsiteDevDeps = []string{
	"@biomejs/biome",
	"@prettier/plugin-oxc",
	"@react-router/dev",
	"@types/node",
	"@types/react",
	"@types/react-dom",
	"@typescript/native-preview",
	"babel-plugin-react-compiler",
	"prettier",
	"prettier-plugin-pkg",
	"prettier-plugin-sh",
	"prettier-plugin-tailwindcss",
	"prettier-plugin-toml",
	"tailwindcss",
	"vite",
	"vite-plugin-babel",
	"vitest",
}

// CloudflareDeps are additional deps for Cloudflare deployment.
var CloudflareDeps = []string{
	"@cloudflare/vite-plugin",
}

// CloudflareDevDeps are additional dev deps for Cloudflare deployment.
var CloudflareDevDeps = []string{
	"wrangler",
}

// Website bootstraps a website project.
type Website struct {
	Dir    string
	Config *config.Config
}

// InstallDependencies installs npm dependencies using pnpm.
func (w *Website) InstallDependencies(ctx context.Context) error {
	runner := shell.New(w.Dir)

	// Collect dependencies based on config
	deps := make([]string, 0, len(WebsiteDeps)+len(CloudflareDeps))
	deps = append(deps, WebsiteDeps...)

	devDeps := make([]string, 0, len(WebsiteDevDeps)+len(CloudflareDevDeps))
	devDeps = append(devDeps, WebsiteDevDeps...)

	// Add deployment-specific deps
	if w.Config.Website != nil {
		switch w.Config.Website.Deployment {
		case "cloudflare":
			deps = append(deps, CloudflareDeps...)
			devDeps = append(devDeps, CloudflareDevDeps...)
		}
	}

	// Install runtime dependencies with spinner
	args := append([]string{"add"}, deps...)
	err := ui.RunWithSpinner(ctx, "Installing dependencies", func() error {
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
