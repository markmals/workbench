package prompt

import (
	"errors"
	"os"

	"github.com/charmbracelet/huh"
	"github.com/markmals/workbench/internal/config"
)

// Options configures the prompt behavior.
type Options struct {
	// NonInteractive disables interactive prompts and requires defaults/flags.
	NonInteractive bool

	// Accessible enables screen reader friendly mode.
	Accessible bool

	// Defaults provides default values for non-interactive mode.
	Defaults *config.Config
}

// Result holds the collected prompt responses.
type Result struct {
	Path string
	Kind string

	// Website-specific
	Deployment string
	Convex     bool

	// Agent selection (populated from Agents slice)
	Claude bool
	Codex  bool
	Gemini bool

	// Raw agent selection from multi-select
	Agents []string
}

// ToConfig converts prompt results to a Config.
func (r *Result) ToConfig() *config.Config {
	cfg := config.New("", r.Kind) // Name will be set by init command from path
	cfg.Path = r.Path

	if r.Kind == "website" {
		cfg.Website = &config.WebsiteConfig{
			Deployment: r.Deployment,
			Convex:     r.Convex,
		}
	}

	if r.Claude || r.Codex || r.Gemini {
		cfg.Agents = &config.AgentsConfig{
			Claude: r.Claude,
			Codex:  r.Codex,
			Gemini: r.Gemini,
		}
	}

	return cfg
}

// Run executes the interactive prompts and returns the result.
func Run(opts Options) (*Result, error) {
	if opts.NonInteractive {
		return runNonInteractive(opts)
	}
	return runInteractive(opts)
}

func runNonInteractive(opts Options) (*Result, error) {
	if opts.Defaults == nil {
		return nil, errors.New("defaults required for non-interactive mode")
	}

	d := opts.Defaults
	result := &Result{
		Path: d.Path,
		Kind: d.Kind,
	}

	if d.Website != nil {
		result.Deployment = d.Website.Deployment
		result.Convex = d.Website.Convex
	}

	if d.Agents != nil {
		result.Claude = d.Agents.Claude
		result.Codex = d.Agents.Codex
		result.Gemini = d.Agents.Gemini
	}

	return result, nil
}

func runInteractive(opts Options) (*Result, error) {
	var result Result

	// Check for accessible mode
	accessible := opts.Accessible || os.Getenv("ACCESSIBLE") != ""

	// Group 1: Basic project info
	basicGroup := huh.NewGroup(
		huh.NewInput().
			Title("Project path").
			Placeholder(".").
			Description("Use '.' for current directory, or a path like './my-project'").
			Validate(func(s string) error {
				if s == "" {
					return errors.New("project path is required")
				}
				return nil
			}).
			Value(&result.Path),

		huh.NewSelect[string]().
			Title("Project type").
			Options(
				huh.NewOption("Website (React Router + TypeScript)", "website"),
				huh.NewOption("CLI/TUI (Go + Bubble Tea)", "tui"),
				huh.NewOption("iOS App (Swift + UIKit)", "ios"),
			).
			Value(&result.Kind),
	)

	// Group 2: Website-specific options (shown conditionally)
	websiteGroup := huh.NewGroup(
		huh.NewSelect[string]().
			Title("Deployment target").
			Options(
				huh.NewOption("Cloudflare Workers", "cloudflare"),
				huh.NewOption("Railway (Node)", "railway"),
			).
			Value(&result.Deployment),

		huh.NewConfirm().
			Title("Include Convex?").
			Description("Real-time backend, functions, and auth").
			Affirmative("Yes").
			Negative("No").
			Value(&result.Convex),
	).WithHideFunc(func() bool {
		return result.Kind != "website"
	})

	// Group 3: Agent selection
	agentGroup := huh.NewGroup(
		huh.NewMultiSelect[string]().
			Title("Enable coding agent support").
			Options(
				huh.NewOption("Claude Code", "claude"),
				huh.NewOption("Codex", "codex"),
				huh.NewOption("Gemini CLI", "gemini"),
			).
			Value(&result.Agents),
	)

	form := huh.NewForm(basicGroup, websiteGroup, agentGroup).
		WithAccessible(accessible)

	err := form.Run()
	if err != nil {
		return nil, err
	}

	// Apply defaults for website
	if result.Kind == "website" {
		if result.Deployment == "" {
			result.Deployment = "cloudflare"
		}
	}

	// Convert agent selection to booleans
	for _, agent := range result.Agents {
		switch agent {
		case "claude":
			result.Claude = true
		case "codex":
			result.Codex = true
		case "gemini":
			result.Gemini = true
		}
	}

	return &result, nil
}
