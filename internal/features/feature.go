package features

import (
	"github.com/markmals/workbench/internal/config"
)

// Feature represents a composable project feature that can be added or removed.
type Feature interface {
	// Name returns the unique identifier for this feature.
	Name() string

	// Description returns a human-readable description.
	Description() string

	// Applies returns true if this feature is applicable to the given config.
	// For example, a "convex" feature only applies to website projects.
	Applies(cfg *config.Config) bool

	// Apply adds this feature to the project at the given directory.
	Apply(ctx *Context) error

	// Remove removes this feature from the project at the given directory.
	Remove(ctx *Context) error
}

// Context provides dependencies for feature operations.
type Context struct {
	// Dir is the project root directory.
	Dir string

	// Config is the project configuration.
	Config *config.Config

	// DryRun indicates whether to simulate changes without writing.
	DryRun bool
}
