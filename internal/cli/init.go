package cli

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/markmals/workbench/internal/assets"
	"github.com/markmals/workbench/internal/bootstrap"
	"github.com/markmals/workbench/internal/config"
	"github.com/markmals/workbench/internal/i18n"
	"github.com/markmals/workbench/internal/projectdef"
	"github.com/markmals/workbench/internal/prompt"
	"github.com/markmals/workbench/internal/templates"
)

// InitCmd creates a new project.
type InitCmd struct {
	Path           string `arg:"" optional:"" help:"Project path (e.g. '.', './my-project')"`
	Kind           string `help:"Project type: website, tui, ios" name:"kind"`
	Deployment     string `help:"Deployment target: cloudflare, railway" name:"deployment"`
	Convex         bool   `help:"Include Convex backend" name:"convex" negatable:""`
	NonInteractive bool   `help:"Require flags/defaults; no prompts" name:"non-interactive"`
	Templates      string `help:"Template source (ref or path)" name:"templates"`
	Yes            bool   `help:"Accept defaults and skip confirmations" short:"y"`
}

func (c *InitCmd) Run(ctx *Context) error {
	// Show logo
	assets.PrintLogo()

	// Build defaults from flags
	var defaults *config.Config
	if c.Path != "" || c.Kind != "" || c.Deployment != "" || c.Convex {
		defaults = config.New("", c.Kind)
		defaults.Path = c.Path
		defaults.Project.Kind = c.Kind
		if c.Convex {
			defaults.AddFeature("convex")
		}
		if c.Kind == "website" {
			defaults.Website.Deployment.Target = c.Deployment
		}
	}

	// Run prompts or use defaults
	opts := prompt.Options{
		NonInteractive: c.NonInteractive,
		Defaults:       defaults,
	}

	result, err := prompt.Run(opts)
	if err != nil {
		// Silent exit on user abort (ctrl+c)
		if err.Error() == "user aborted" {
			return nil
		}
		return fmt.Errorf("gathering input: %w", err)
	}

	// Convert to config
	cfg := result.ToConfig()

	// Template ref (ref or path) recorded for reproducibility.
	ref := c.Templates
	if ref == "" {
		ref = "main"
	}
	cfg.TemplateRef = ref
	cfg.Project.TemplateRef = ref

	// Resolve path and infer name
	projectPath := cfg.Path
	if projectPath == "" {
		projectPath = "."
	}
	absDir, err := filepath.Abs(projectPath)
	if err != nil {
		return fmt.Errorf("resolving directory: %w", err)
	}

	// Infer name from directory
	cfg.Name = filepath.Base(absDir)
	cfg.Project.Name = cfg.Name

	ctx.Logger.Debug("initializing project", "dir", absDir, "name", cfg.Name)

	// Validate
	if err := config.Validate(cfg); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
	}

	// Create project directory if it doesn't exist
	if err := os.MkdirAll(absDir, 0755); err != nil {
		return fmt.Errorf("creating directory: %w", err)
	}

	// Save config
	if err := config.Save(absDir, cfg); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	ctx.Logger.Debug("saved config", "path", config.ConfigPath(absDir))

	// For website projects, hydrate upstream React Router template first.
	if cfg.Kind == "website" {
		templateName := bootstrap.ReactRouterTemplateName(cfg)
		refresh := ref == "" || strings.EqualFold(ref, "main") || strings.EqualFold(ref, "latest")

		upstreamPath, err := bootstrap.FetchReactRouterTemplate(context.Background(), ref, templateName, refresh)
		if err != nil {
			return fmt.Errorf("fetching React Router template: %w", err)
		}

		if err := bootstrap.CopyTemplate(upstreamPath, absDir); err != nil {
			return fmt.Errorf("copying upstream template: %w", err)
		}

		ctx.Logger.Info("fetched upstream template", "template", templateName, "ref", ref)
	}

	// Render templates from project definition
	renderer := templates.Bootstrap()
	renderCtx := &templates.RenderContext{
		Name:     cfg.Name,
		Kind:     cfg.Kind,
		Features: cfg.Features,
		Config:   cfg,
	}

	if cfg.Website != nil {
		renderCtx.Website = &templates.WebsiteContext{
			Deployment: cfg.Website.Deployment.Target,
			Framework:  cfg.Website.Framework,
			Rendering:  cfg.Website.Rendering,
			RouteMap:   cfg.Website.RouteMap,
			Future:     cfg.Website.ReactRouter.FutureFlags,
		}
	}

	// Load template mappings from project definition
	def, err := projectdef.Get(cfg.Kind)
	if err != nil {
		return fmt.Errorf("loading project definition: %w", err)
	}

	// Render static templates
	for dest, tmpl := range def.Templates.Static {
		if cfg.Kind == "website" && dest == "wrangler.jsonc" && !strings.EqualFold(cfg.Website.Deployment.Target, "cloudflare") {
			continue
		}
		destPath := filepath.Join(absDir, dest)
		if err := renderer.RenderTo(tmpl, renderCtx, destPath); err != nil {
			ctx.Logger.Warn("failed to render template", "template", tmpl, "error", err)
			continue
		}
		ctx.Logger.Debug("rendered", "file", dest)
	}

	// Render conditional templates based on enabled features
	for feature, templates := range def.Templates.When {
		if !cfg.HasFeature(feature) {
			continue
		}
		for dest, tmpl := range templates {
			if cfg.Kind == "website" && dest == "wrangler.jsonc" && !strings.EqualFold(cfg.Website.Deployment.Target, "cloudflare") {
				continue
			}
			destPath := filepath.Join(absDir, dest)
			if err := renderer.RenderTo(tmpl, renderCtx, destPath); err != nil {
				ctx.Logger.Warn("failed to render conditional template", "feature", feature, "template", tmpl, "error", err)
				continue
			}
			ctx.Logger.Debug("rendered conditional", "feature", feature, "file", dest)
		}
	}

	// Install dependencies for website projects
	if cfg.Kind == "website" {
		if err := bootstrap.ApplyPackagePreferences(absDir, cfg); err != nil {
			return fmt.Errorf("applying package defaults: %w", err)
		}

		wb := &bootstrap.Website{
			Dir:    absDir,
			Config: cfg,
		}
		if err := wb.InstallDependencies(context.Background()); err != nil {
			return fmt.Errorf("installing dependencies: %w", err)
		}
	}

	// Print success message
	if ctx.CLI.JSON {
		// JSON output handled by caller
	} else {
		fmt.Println()
		fmt.Println(i18n.T("InitCreatedProject", i18n.M{"Kind": cfg.Kind, "Name": cfg.Name}))
		fmt.Println(i18n.T("InitLocation", i18n.M{"Dir": absDir}))
		fmt.Println()
		fmt.Println(i18n.T("InitNextSteps"))
		if projectPath != "." {
			fmt.Println(i18n.T("InitCdHint", i18n.M{"Path": projectPath}))
		}
		fmt.Println(i18n.T("InitMiseInstall"))
		fmt.Println(i18n.T("InitMiseRunDev"))
	}

	return nil
}
