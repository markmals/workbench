package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
)

const (
	// ConfigDir is the directory name for workbench config.
	ConfigDir = ".workbench"
	// ConfigFile is the config file name.
	ConfigFile = "config.jsonc"
)

// Config represents the project configuration.
type Config struct {
	// Version of the config schema.
	Version string `json:"version"`

	// Kind is the project type: website, tui, ios, monorepo.
	// Kept for compatibility; canonical value is stored under Project.Kind.
	Kind string `json:"kind,omitempty"`

	// Name is the project name (inferred from path).
	// Kept for compatibility; canonical value is stored under Project.Name.
	Name string `json:"name,omitempty"`

	// Project captures canonical project metadata.
	Project ProjectConfig `json:"project"`

	// Path is the project directory path (used during init, not persisted).
	Path string `json:"-"`

	// Features is the list of enabled feature names.
	Features []string `json:"features"`

	// TemplateRef is the template version/ref used for reproducibility.
	TemplateRef string `json:"templateRef,omitempty"`

	// Website contains website-specific configuration.
	Website *WebsiteConfig `json:"website,omitempty"`

	// Data contains backend/storage configuration.
	Data *DataConfig `json:"data,omitempty"`

	// UI contains styling/theming configuration.
	UI *UIConfig `json:"ui,omitempty"`

	// Tooling captures formatter, linter, and task runner preferences.
	Tooling *ToolingConfig `json:"tooling,omitempty"`

	// Agents controls synthesized agent documentation output.
	Agents *AgentsConfig `json:"agents,omitempty"`

	// TUI contains TUI-specific configuration.
	TUI *TUIConfig `json:"tui,omitempty"`

	// IOS contains iOS-specific configuration.
	IOS *IOSConfig `json:"ios,omitempty"`
}

// HasFeature returns true if the given feature is enabled.
func (c *Config) HasFeature(name string) bool {
	for _, f := range c.Features {
		if f == name {
			return true
		}
	}
	return false
}

// AddFeature adds a feature if not already present.
func (c *Config) AddFeature(name string) {
	if !c.HasFeature(name) {
		c.Features = append(c.Features, name)
	}
}

// RemoveFeature removes a feature if present.
func (c *Config) RemoveFeature(name string) {
	features := make([]string, 0, len(c.Features))
	for _, f := range c.Features {
		if f != name {
			features = append(features, f)
		}
	}
	c.Features = features
}

// ProjectConfig captures canonical project metadata.
type ProjectConfig struct {
	Kind        string `json:"kind"`
	Name        string `json:"name"`
	TemplateRef string `json:"templateRef,omitempty"`
}

// WebsiteConfig holds website-specific options.
type WebsiteConfig struct {
	// Framework: react-router (current), astro (planned).
	Framework string `json:"framework,omitempty"`

	// Rendering mode: ssr, rsc, spa, static.
	Rendering string `json:"rendering,omitempty"`

	// RouteMap enables @withsprinkles/react-router-route-map.
	RouteMap bool `json:"routeMap,omitempty"`

	// ReactRouter holds future flags and other tunables.
	ReactRouter ReactRouterConfig `json:"reactRouter,omitempty"`

	// Deployment targets and options.
	Deployment WebDeploymentConfig `json:"deployment,omitempty"`
}

// ReactRouterConfig captures React Router config toggles.
type ReactRouterConfig struct {
	FutureFlags map[string]bool `json:"futureFlags,omitempty"`
}

// WebDeploymentConfig configures deployment targets like Cloudflare or Railway.
type WebDeploymentConfig struct {
	Target     string            `json:"target,omitempty"`
	Cloudflare *CloudflareConfig `json:"cloudflare,omitempty"`
	// Placeholder for planned targets (railway, etc.).
	Railway map[string]any `json:"railway,omitempty"`
}

// CloudflareConfig holds Cloudflare-specific deployment options.
type CloudflareConfig struct {
	CompatibilityDate  string                `json:"compatibilityDate,omitempty"`
	CompatibilityFlags []string              `json:"compatibilityFlags,omitempty"`
	WorkersDev         *bool                 `json:"workersDev,omitempty"`
	AssetsDir          string                `json:"assetsDir,omitempty"`
	Logs               *CloudflareLogsConfig `json:"logs,omitempty"`
}

// CloudflareLogsConfig configures observability/logging on Cloudflare.
type CloudflareLogsConfig struct {
	Enabled    *bool    `json:"enabled,omitempty"`
	Sampling   *float64 `json:"sampling,omitempty"`
	Invocation *bool    `json:"invocation,omitempty"`
	Persist    *bool    `json:"persist,omitempty"`
}

// DataConfig captures backend/storage choices.
type DataConfig struct {
	Backend    string         `json:"backend,omitempty"`
	Drizzle    *DrizzleConfig `json:"drizzle,omitempty"`
	Cloudflare map[string]any `json:"cloudflare,omitempty"`
}

// DrizzleConfig configures Drizzle ORM.
type DrizzleConfig struct {
	Driver        string `json:"driver,omitempty"`
	SchemaPath    string `json:"schemaPath,omitempty"`
	MigrationsDir string `json:"migrationsDir,omitempty"`
}

// UIConfig holds styling/theming options.
type UIConfig struct {
	Tailwind TailwindConfig `json:"tailwind,omitempty"`
	Shadcn   ShadcnConfig   `json:"shadcn,omitempty"`
}

// TailwindConfig holds Tailwind-specific preferences.
type TailwindConfig struct {
	CSSPath   string `json:"cssPath,omitempty"`
	BaseColor string `json:"baseColor,omitempty"`
}

// ShadcnConfig controls shadcn/ui defaults.
type ShadcnConfig struct {
	Style       string `json:"style,omitempty"`
	RSC         bool   `json:"rsc"`
	IconLibrary string `json:"iconLibrary,omitempty"`
}

// ToolingConfig records formatter/linter/toolchain choices.
type ToolingConfig struct {
	Node      string         `json:"node,omitempty"`
	Pnpm      string         `json:"pnpm,omitempty"`
	Tasks     map[string]any `json:"tasks,omitempty"`
	Formatter string         `json:"formatter,omitempty"`
	Linter    string         `json:"linter,omitempty"`
}

// AgentsConfig controls synthesized agent documentation.
type AgentsConfig struct {
	Bundle        bool                `json:"bundle"`
	Sources       []string            `json:"sources,omitempty"`
	Output        string              `json:"output,omitempty"`
	IncludeByMode map[string][]string `json:"includeByMode,omitempty"`
}

// TUIConfig holds TUI-specific options.
type TUIConfig struct {
	// Libs is the list of Charm libraries to include.
	Libs []string `json:"libs,omitempty"`
}

// IOSConfig holds iOS-specific options.
type IOSConfig struct {
	// Tuist indicates whether Tuist is enabled.
	Tuist bool `json:"tuist"`

	// DataBackend is the data backend: sqlite, convex.
	DataBackend string `json:"dataBackend,omitempty"`
}

// ConfigPath returns the path to the config file in the given directory.
func ConfigPath(dir string) string {
	return filepath.Join(dir, ConfigDir, ConfigFile)
}

// Exists checks if a config file exists in the given directory.
func Exists(dir string) bool {
	_, err := os.Stat(ConfigPath(dir))
	return err == nil
}

// Load reads and parses the config from the given directory.
func Load(dir string) (*Config, error) {
	path := ConfigPath(dir)

	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("config not found: %s", path)
		}
		return nil, fmt.Errorf("reading config: %w", err)
	}

	// Strip JSONC comments before parsing
	data = stripComments(data)

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("parsing config: %w", err)
	}

	cfg.ApplyDefaults()

	return &cfg, nil
}

// Save writes the config to the given directory.
func Save(dir string, cfg *Config) error {
	configDir := filepath.Join(dir, ConfigDir)

	// Ensure .workbench directory exists
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return fmt.Errorf("creating config directory: %w", err)
	}

	cfg.ApplyDefaults()

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("marshaling config: %w", err)
	}

	path := ConfigPath(dir)
	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("writing config: %w", err)
	}

	return nil
}

// Validate checks that the config has all required fields and valid values.
func Validate(cfg *Config) error {
	if cfg == nil {
		return errors.New("config is nil")
	}

	cfg.ApplyDefaults()

	if cfg.Version == "" {
		return errors.New("config version is required")
	}

	if cfg.Kind == "" {
		return errors.New("config kind is required")
	}

	validKinds := map[string]bool{
		"website":  true,
		"tui":      true,
		"ios":      true,
		"monorepo": true,
	}
	if !validKinds[cfg.Kind] {
		return fmt.Errorf("invalid kind: %s (must be website, tui, ios, or monorepo)", cfg.Kind)
	}

	if cfg.Name == "" {
		return errors.New("config name is required")
	}

	// Validate website-specific config
	if cfg.Kind == "website" && cfg.Website != nil {
		validDeployments := map[string]bool{
			"cloudflare": true,
			"railway":    true,
		}
		if cfg.Website.Deployment.Target != "" && !validDeployments[cfg.Website.Deployment.Target] {
			return fmt.Errorf("invalid deployment: %s (must be cloudflare or railway)", cfg.Website.Deployment.Target)
		}
	}

	return nil
}

// ApplyDefaults populates missing fields with sensible defaults.
func (c *Config) ApplyDefaults() {
	if c.Version == "" {
		c.Version = "2"
	}

	// Sync legacy fields with canonical project fields.
	if c.Project.Kind == "" {
		c.Project.Kind = c.Kind
	}
	if c.Project.Name == "" {
		c.Project.Name = c.Name
	}
	if c.Kind == "" {
		c.Kind = c.Project.Kind
	}
	if c.Name == "" {
		c.Name = c.Project.Name
	}
	if c.TemplateRef == "" {
		c.TemplateRef = c.Project.TemplateRef
	} else if c.Project.TemplateRef == "" {
		c.Project.TemplateRef = c.TemplateRef
	}

	switch c.Kind {
	case "website":
		if c.Website == nil {
			c.Website = DefaultWebsiteConfig()
		} else {
			if !c.Website.RouteMap && c.Version == "1" {
				c.Website.RouteMap = true
			}
			applyWebsiteDefaults(c.Website)
		}
		if c.Data == nil {
			c.Data = DefaultDataConfig()
		} else {
			applyDataDefaults(c.Data)
		}
		if c.UI == nil {
			c.UI = DefaultUIConfig()
		} else {
			applyUIDefaults(c.UI)
		}
		if c.Tooling == nil {
			c.Tooling = DefaultToolingConfig()
		} else {
			applyToolingDefaults(c.Tooling)
		}
		if c.Agents == nil {
			c.Agents = DefaultAgentsConfig()
		}
	default:
		// Non-website projects still get agent defaults if missing.
		if c.Agents == nil {
			c.Agents = DefaultAgentsConfig()
		}
	}
}

// DefaultWebsiteConfig returns defaults aligned with CONFIG.md.
func DefaultWebsiteConfig() *WebsiteConfig {
	return &WebsiteConfig{
		Framework: "react-router",
		Rendering: "ssr",
		RouteMap:  true,
		ReactRouter: ReactRouterConfig{
			FutureFlags: map[string]bool{
				"v8_viteEnvironmentApi": true,
				"v8_middleware":         true,
				"v8_splitRouteModules":  true,
			},
		},
		Deployment: WebDeploymentConfig{
			Target: "cloudflare",
			Cloudflare: &CloudflareConfig{
				CompatibilityDate:  "2026-01-20",
				CompatibilityFlags: []string{"nodejs_als"},
				WorkersDev:         boolPtr(true),
				AssetsDir:          "./build/client",
				Logs: &CloudflareLogsConfig{
					Enabled:    boolPtr(true),
					Sampling:   floatPtr(1.0),
					Invocation: boolPtr(true),
					Persist:    boolPtr(true),
				},
			},
		},
	}
}

// DefaultDataConfig returns default data/storage selections.
func DefaultDataConfig() *DataConfig {
	return &DataConfig{
		Backend: "drizzle",
		Drizzle: &DrizzleConfig{
			Driver:        "sqlite",
			SchemaPath:    "./app/lib/db/schema.ts",
			MigrationsDir: "./drizzle/migrations",
		},
	}
}

// DefaultUIConfig returns default UI/styling options.
func DefaultUIConfig() *UIConfig {
	return &UIConfig{
		Tailwind: TailwindConfig{
			CSSPath:   "app/styles/app.css",
			BaseColor: "neutral",
		},
		Shadcn: ShadcnConfig{
			Style:       "new-york",
			RSC:         true,
			IconLibrary: "lucide",
		},
	}
}

// DefaultToolingConfig returns default tooling/linter selections.
func DefaultToolingConfig() *ToolingConfig {
	return &ToolingConfig{
		Node:      "24",
		Pnpm:      "latest",
		Formatter: "prettier",
		Linter:    "oxlint",
	}
}

// DefaultAgentsConfig returns default agent bundling options.
func DefaultAgentsConfig() *AgentsConfig {
	return &AgentsConfig{
		Bundle: true,
		Output: "AGENTS.md",
	}
}

func applyWebsiteDefaults(cfg *WebsiteConfig) {
	if cfg.Framework == "" {
		cfg.Framework = "react-router"
	}
	if cfg.Rendering == "" {
		cfg.Rendering = "ssr"
	}
	if cfg.ReactRouter.FutureFlags == nil {
		cfg.ReactRouter.FutureFlags = DefaultWebsiteConfig().ReactRouter.FutureFlags
	}
	if cfg.Deployment.Target == "" {
		cfg.Deployment.Target = "cloudflare"
	}
	if cfg.Deployment.Target == "cloudflare" {
		if cfg.Deployment.Cloudflare == nil {
			cfg.Deployment.Cloudflare = DefaultWebsiteConfig().Deployment.Cloudflare
		}
		if cfg.Deployment.Cloudflare.CompatibilityDate == "" {
			cfg.Deployment.Cloudflare.CompatibilityDate = "2026-01-20"
		}
		if cfg.Deployment.Cloudflare.CompatibilityFlags == nil {
			cfg.Deployment.Cloudflare.CompatibilityFlags = []string{"nodejs_als"}
		}
		if cfg.Deployment.Cloudflare.WorkersDev == nil {
			cfg.Deployment.Cloudflare.WorkersDev = boolPtr(true)
		}
		if cfg.Deployment.Cloudflare.AssetsDir == "" {
			cfg.Deployment.Cloudflare.AssetsDir = "./build/client"
		}
		if cfg.Deployment.Cloudflare.Logs == nil {
			cfg.Deployment.Cloudflare.Logs = DefaultWebsiteConfig().Deployment.Cloudflare.Logs
		}
	}
}

func applyDataDefaults(cfg *DataConfig) {
	if cfg.Backend == "" {
		cfg.Backend = "drizzle"
	}
	if cfg.Drizzle == nil {
		cfg.Drizzle = DefaultDataConfig().Drizzle
	} else {
		if cfg.Drizzle.Driver == "" {
			cfg.Drizzle.Driver = "sqlite"
		}
		if cfg.Drizzle.SchemaPath == "" {
			cfg.Drizzle.SchemaPath = "./app/lib/db/schema.ts"
		}
		if cfg.Drizzle.MigrationsDir == "" {
			cfg.Drizzle.MigrationsDir = "./drizzle/migrations"
		}
	}
}

func applyUIDefaults(cfg *UIConfig) {
	if cfg.Tailwind.CSSPath == "" {
		cfg.Tailwind.CSSPath = "app/styles/app.css"
	}
	if cfg.Tailwind.BaseColor == "" {
		cfg.Tailwind.BaseColor = "neutral"
	}
	if cfg.Shadcn.Style == "" {
		cfg.Shadcn.Style = "new-york"
	}
	if cfg.Shadcn.IconLibrary == "" {
		cfg.Shadcn.IconLibrary = "lucide"
	}
}

func applyToolingDefaults(cfg *ToolingConfig) {
	if cfg.Node == "" {
		cfg.Node = "24"
	}
	if cfg.Pnpm == "" {
		cfg.Pnpm = "latest"
	}
	if cfg.Formatter == "" {
		cfg.Formatter = "prettier"
	}
	if cfg.Linter == "" {
		cfg.Linter = "oxlint"
	}
}

func boolPtr(v bool) *bool { return &v }

func floatPtr(v float64) *float64 { return &v }

// New creates a new Config with default values.
func New(name, kind string) *Config {
	cfg := &Config{
		Version: "2",
		Kind:    kind,
		Name:    name,
		Project: ProjectConfig{
			Kind: kind,
			Name: name,
		},
		Features: []string{},
	}
	cfg.ApplyDefaults()
	return cfg
}

// stripComments removes // and /* */ style comments from JSONC.
var (
	lineCommentRe  = regexp.MustCompile(`(?m)^\s*//.*$|//[^"]*$`)
	blockCommentRe = regexp.MustCompile(`/\*[\s\S]*?\*/`)
)

func stripComments(data []byte) []byte {
	// Remove block comments first
	data = blockCommentRe.ReplaceAll(data, nil)
	// Then line comments
	data = lineCommentRe.ReplaceAll(data, nil)
	return data
}
