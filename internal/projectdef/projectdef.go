// Package projectdef provides declarative project type definitions loaded from TOML.
package projectdef

import (
	"embed"
	"fmt"
	"io/fs"

	"github.com/BurntSushi/toml"
)

//go:embed defs/*.toml
var defsFS embed.FS

// Definition represents a complete project type definition.
type Definition struct {
	Project      Project            `toml:"project"`
	Dependencies Dependencies       `toml:"dependencies"`
	Templates    TemplateConfig     `toml:"templates"`
	Features     map[string]Feature `toml:"features"`
}

// TemplateConfig holds both static and conditional templates.
type TemplateConfig struct {
	// Static templates (parsed from inline key-value pairs)
	Static map[string]string
	// Conditional templates keyed by feature name
	When map[string]map[string]string
}

// UnmarshalTOML implements custom TOML unmarshaling for TemplateConfig.
func (t *TemplateConfig) UnmarshalTOML(data any) error {
	t.Static = make(map[string]string)
	t.When = make(map[string]map[string]string)

	m, ok := data.(map[string]any)
	if !ok {
		return nil
	}

	for k, v := range m {
		if k == "when" {
			// Handle conditional templates
			whenMap, ok := v.(map[string]any)
			if !ok {
				continue
			}
			for feature, templates := range whenMap {
				templatesMap, ok := templates.(map[string]any)
				if !ok {
					continue
				}
				t.When[feature] = make(map[string]string)
				for dest, tmpl := range templatesMap {
					if tmplStr, ok := tmpl.(string); ok {
						t.When[feature][dest] = tmplStr
					}
				}
			}
		} else {
			// Static template
			if tmplStr, ok := v.(string); ok {
				t.Static[k] = tmplStr
			}
		}
	}
	return nil
}

// Project holds basic project metadata.
type Project struct {
	Kind        string `toml:"kind"`
	Description string `toml:"description"`
}

// Dependencies defines packages to install.
type Dependencies struct {
	Runtime []string                   `toml:"runtime"`
	Dev     []string                   `toml:"dev"`
	When    map[string]ConditionalDeps `toml:"when"`
}

// ConditionalDeps are dependencies added when a condition is met.
type ConditionalDeps struct {
	Runtime []string `toml:"runtime"`
	Dev     []string `toml:"dev"`
}

// Feature defines an optional feature that can be added/removed.
type Feature struct {
	Description string        `toml:"description"`
	Packages    []string      `toml:"packages"`
	DevPackages []string      `toml:"dev_packages"`
	PostMessage string        `toml:"post_message"`
	Remove      FeatureRemove `toml:"remove"`
}

// FeatureRemove defines what to remove when a feature is removed.
type FeatureRemove struct {
	Packages    []string `toml:"packages"`
	DevPackages []string `toml:"dev_packages"`
	Directories []string `toml:"directories"`
}

// registry holds loaded definitions keyed by kind.
var registry = make(map[string]*Definition)

func init() {
	// Load all embedded definitions at startup
	entries, err := fs.ReadDir(defsFS, "defs")
	if err != nil {
		return
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		data, err := fs.ReadFile(defsFS, "defs/"+entry.Name())
		if err != nil {
			continue
		}

		var def Definition
		if _, err := toml.Decode(string(data), &def); err != nil {
			continue
		}

		registry[def.Project.Kind] = &def
	}
}

// Get returns the definition for the given project kind.
func Get(kind string) (*Definition, error) {
	def, ok := registry[kind]
	if !ok {
		return nil, fmt.Errorf("unknown project kind: %s", kind)
	}
	return def, nil
}

// List returns all available project kinds.
func List() []string {
	kinds := make([]string, 0, len(registry))
	for k := range registry {
		kinds = append(kinds, k)
	}
	return kinds
}

// AllDeps returns all runtime dependencies including conditional ones.
func (d *Dependencies) AllDeps(conditions ...string) []string {
	deps := make([]string, len(d.Runtime))
	copy(deps, d.Runtime)

	for _, cond := range conditions {
		if cdeps, ok := d.When[cond]; ok {
			deps = append(deps, cdeps.Runtime...)
		}
	}
	return deps
}

// AllDevDeps returns all dev dependencies including conditional ones.
func (d *Dependencies) AllDevDeps(conditions ...string) []string {
	deps := make([]string, len(d.Dev))
	copy(deps, d.Dev)

	for _, cond := range conditions {
		if cdeps, ok := d.When[cond]; ok {
			deps = append(deps, cdeps.Dev...)
		}
	}
	return deps
}
