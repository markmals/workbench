package templates

import (
	"embed"
)

//go:embed bootstrap/*
var bootstrapFS embed.FS

// Bootstrap returns a Renderer using the embedded bootstrap templates.
func Bootstrap() *Renderer {
	return New(bootstrapFS)
}
