package templates

import (
	"embed"
	"io/fs"
)

//go:embed bootstrap/* skills/*
var templatesFS embed.FS

// Bootstrap returns a Renderer using the embedded templates.
func Bootstrap() *Renderer {
	return New(templatesFS)
}

// BootstrapFS returns the raw embedded filesystem for direct access.
func BootstrapFS() fs.FS {
	return templatesFS
}
