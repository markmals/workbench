package features

import (
	"fmt"
	"sort"

	"github.com/markmals/workbench/internal/config"
)

// Registry holds all registered features.
type Registry struct {
	features map[string]Feature
}

// NewRegistry creates a new empty feature registry.
func NewRegistry() *Registry {
	return &Registry{
		features: make(map[string]Feature),
	}
}

// Register adds a feature to the registry.
func (r *Registry) Register(f Feature) {
	r.features[f.Name()] = f
}

// Get returns a feature by name, or nil if not found.
func (r *Registry) Get(name string) Feature {
	return r.features[name]
}

// List returns all registered features, sorted by name.
func (r *Registry) List() []Feature {
	features := make([]Feature, 0, len(r.features))
	for _, f := range r.features {
		features = append(features, f)
	}
	sort.Slice(features, func(i, j int) bool {
		return features[i].Name() < features[j].Name()
	})
	return features
}

// ListApplicable returns features that apply to the given config.
func (r *Registry) ListApplicable(cfg *config.Config) []Feature {
	var applicable []Feature
	for _, f := range r.List() {
		if f.Applies(cfg) {
			applicable = append(applicable, f)
		}
	}
	return applicable
}

// Apply applies a feature by name.
func (r *Registry) Apply(name string, ctx *Context) error {
	f := r.Get(name)
	if f == nil {
		return fmt.Errorf("unknown feature: %s", name)
	}
	if !f.Applies(ctx.Config) {
		return fmt.Errorf("feature %s does not apply to %s projects", name, ctx.Config.Kind)
	}
	return f.Apply(ctx)
}

// Remove removes a feature by name.
func (r *Registry) Remove(name string, ctx *Context) error {
	f := r.Get(name)
	if f == nil {
		return fmt.Errorf("unknown feature: %s", name)
	}
	return f.Remove(ctx)
}

// Default is the global feature registry.
var Default = NewRegistry()

// Register adds a feature to the default registry.
func Register(f Feature) {
	Default.Register(f)
}

// Get returns a feature from the default registry.
func Get(name string) Feature {
	return Default.Get(name)
}

// List returns all features from the default registry.
func List() []Feature {
	return Default.List()
}

// ListApplicable returns applicable features from the default registry.
func ListApplicable(cfg *config.Config) []Feature {
	return Default.ListApplicable(cfg)
}
