// Package i18n provides centralized message strings for the CLI.
// All user-facing text is stored in messages.en.toml for easy editing.
package i18n

import (
	"embed"

	"github.com/BurntSushi/toml"
	"github.com/nicksnyder/go-i18n/v2/i18n"
	"golang.org/x/text/language"
)

//go:embed messages.en.toml
var messagesFS embed.FS

var (
	bundle    *i18n.Bundle
	localizer *i18n.Localizer
)

func init() {
	bundle = i18n.NewBundle(language.English)
	bundle.RegisterUnmarshalFunc("toml", toml.Unmarshal)

	// Load embedded messages
	bundle.LoadMessageFileFS(messagesFS, "messages.en.toml")

	localizer = i18n.NewLocalizer(bundle, "en")
}

// T returns the localized message for the given ID.
// If data is provided, it will be used for template substitution.
func T(id string, data ...map[string]any) string {
	cfg := &i18n.LocalizeConfig{
		MessageID: id,
	}
	if len(data) > 0 && data[0] != nil {
		cfg.TemplateData = data[0]
	}

	msg, err := localizer.Localize(cfg)
	if err != nil {
		// Return the ID if message not found (helpful for debugging)
		return "[" + id + "]"
	}
	return msg
}

// M is a convenience type for template data.
type M = map[string]any
