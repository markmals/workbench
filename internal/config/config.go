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
	Kind string `json:"kind"`

	// Name is the project name (inferred from path).
	Name string `json:"name"`

	// Path is the project directory path (used during init, not persisted).
	Path string `json:"-"`

	// Features is the list of enabled feature names.
	Features []string `json:"features"`

	// TemplateRef is the template version/ref used for reproducibility.
	TemplateRef string `json:"templateRef,omitempty"`

	// Website contains website-specific configuration.
	Website *WebsiteConfig `json:"website,omitempty"`

	// TUI contains TUI-specific configuration.
	TUI *TUIConfig `json:"tui,omitempty"`

	// IOS contains iOS-specific configuration.
	IOS *IOSConfig `json:"ios,omitempty"`

	// Agents contains agent-specific configuration.
	Agents *AgentsConfig `json:"agents,omitempty"`
}

// WebsiteConfig holds website-specific options.
type WebsiteConfig struct {
	// Deployment target: cloudflare, railway.
	Deployment string `json:"deployment"`

	// Convex indicates whether Convex is enabled.
	Convex bool `json:"convex"`

	// Mode is the rendering mode: spa, ssr, static.
	Mode string `json:"mode,omitempty"`
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

// AgentsConfig holds agent-specific options.
type AgentsConfig struct {
	// Codex indicates whether Codex agent support is enabled.
	Codex bool `json:"codex,omitempty"`

	// Claude indicates whether Claude Code agent support is enabled.
	Claude bool `json:"claude,omitempty"`

	// Gemini indicates whether Gemini CLI agent support is enabled.
	Gemini bool `json:"gemini,omitempty"`
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

	return &cfg, nil
}

// Save writes the config to the given directory.
func Save(dir string, cfg *Config) error {
	configDir := filepath.Join(dir, ConfigDir)

	// Ensure .workbench directory exists
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return fmt.Errorf("creating config directory: %w", err)
	}

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
		if cfg.Website.Deployment != "" && !validDeployments[cfg.Website.Deployment] {
			return fmt.Errorf("invalid deployment: %s (must be cloudflare or railway)", cfg.Website.Deployment)
		}
	}

	return nil
}

// New creates a new Config with default values.
func New(name, kind string) *Config {
	return &Config{
		Version:  "1",
		Kind:     kind,
		Name:     name,
		Features: []string{},
	}
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
