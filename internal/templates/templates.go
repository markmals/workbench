package templates

import (
	"bytes"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"text/template"
)

// Renderer handles template rendering with a func map and template source.
type Renderer struct {
	fs      fs.FS
	funcMap template.FuncMap
}

// New creates a new Renderer using the provided filesystem.
func New(fsys fs.FS) *Renderer {
	return &Renderer{
		fs:      fsys,
		funcMap: defaultFuncMap(),
	}
}

// NewWithFuncs creates a new Renderer with additional template functions.
func NewWithFuncs(fsys fs.FS, funcs template.FuncMap) *Renderer {
	r := New(fsys)
	for k, v := range funcs {
		r.funcMap[k] = v
	}
	return r
}

// defaultFuncMap returns the default template helper functions.
func defaultFuncMap() template.FuncMap {
	return template.FuncMap{
		// String manipulation
		"lower": strings.ToLower,
		"upper": strings.ToUpper,
		"title": strings.Title,
		"trim":  strings.TrimSpace,

		// String joining/splitting
		"join":  strings.Join,
		"split": strings.Split,

		// Conditionals
		"contains": strings.Contains,
		"hasPrefix": strings.HasPrefix,
		"hasSuffix": strings.HasSuffix,

		// Lists
		"list": func(args ...any) []any {
			return args
		},
		"first": func(list []any) any {
			if len(list) > 0 {
				return list[0]
			}
			return nil
		},
		"last": func(list []any) any {
			if len(list) > 0 {
				return list[len(list)-1]
			}
			return nil
		},

		// Default values
		"default": func(def, val any) any {
			if val == nil || val == "" {
				return def
			}
			return val
		},

		// Boolean helpers
		"not": func(b bool) bool {
			return !b
		},
		"and": func(a, b bool) bool {
			return a && b
		},
		"or": func(a, b bool) bool {
			return a || b
		},
	}
}

// Render renders the named template with the given data.
func (r *Renderer) Render(name string, data any) (string, error) {
	content, err := fs.ReadFile(r.fs, name)
	if err != nil {
		return "", fmt.Errorf("reading template %s: %w", name, err)
	}

	tmpl, err := template.New(name).Funcs(r.funcMap).Parse(string(content))
	if err != nil {
		return "", fmt.Errorf("parsing template %s: %w", name, err)
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("executing template %s: %w", name, err)
	}

	return buf.String(), nil
}

// RenderTo renders the named template and writes it to the destination path.
func (r *Renderer) RenderTo(name string, data any, dest string) error {
	content, err := r.Render(name, data)
	if err != nil {
		return err
	}

	// Ensure parent directory exists
	dir := filepath.Dir(dest)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("creating directory %s: %w", dir, err)
	}

	if err := os.WriteFile(dest, []byte(content), 0644); err != nil {
		return fmt.Errorf("writing file %s: %w", dest, err)
	}

	return nil
}

// List returns all template files in the filesystem.
func (r *Renderer) List() ([]string, error) {
	var templates []string
	err := fs.WalkDir(r.fs, ".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() && strings.HasSuffix(path, ".tmpl") {
			templates = append(templates, path)
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("walking templates: %w", err)
	}
	return templates, nil
}

// RenderContext holds data passed to templates during rendering.
type RenderContext struct {
	// Project info
	Name string
	Kind string

	// Features
	Features []string

	// Website config
	Website *WebsiteContext

	// TUI config
	TUI *TUIContext

	// iOS config
	IOS *IOSContext

	// Agent config
	Agents *AgentsContext
}

// WebsiteContext holds website-specific template data.
type WebsiteContext struct {
	Deployment string
	Convex     bool
	Mode       string
}

// TUIContext holds TUI-specific template data.
type TUIContext struct {
	Libs []string
}

// IOSContext holds iOS-specific template data.
type IOSContext struct {
	Tuist       bool
	DataBackend string
}

// AgentsContext holds agent-specific template data.
type AgentsContext struct {
	Codex  bool
	Claude bool
	Gemini bool
}

// HasFeature returns true if the given feature is enabled.
func (c *RenderContext) HasFeature(name string) bool {
	for _, f := range c.Features {
		if f == name {
			return true
		}
	}
	return false
}
