package ui

import (
	"github.com/charmbracelet/lipgloss"
)

// Colors used throughout the application.
var (
	Primary   = lipgloss.AdaptiveColor{Light: "#7C3AED", Dark: "#A78BFA"}
	Secondary = lipgloss.AdaptiveColor{Light: "#6B7280", Dark: "#9CA3AF"}
	Success   = lipgloss.AdaptiveColor{Light: "#059669", Dark: "#34D399"}
	Warning   = lipgloss.AdaptiveColor{Light: "#D97706", Dark: "#FBBF24"}
	Error     = lipgloss.AdaptiveColor{Light: "#DC2626", Dark: "#F87171"}
	Muted     = lipgloss.AdaptiveColor{Light: "#9CA3AF", Dark: "#6B7280"}
)

// Text styles.
var (
	Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(Primary)

	Subtitle = lipgloss.NewStyle().
		Foreground(Secondary)

	Bold = lipgloss.NewStyle().
		Bold(true)

	Mute = lipgloss.NewStyle().
		Foreground(Muted)

	SuccessText = lipgloss.NewStyle().
		Foreground(Success)

	WarningText = lipgloss.NewStyle().
		Foreground(Warning)

	ErrorText = lipgloss.NewStyle().
		Foreground(Error)
)

// Container styles.
var (
	Box = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(Secondary).
		Padding(1, 2)

	Card = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(Primary).
		Padding(1, 2).
		MarginBottom(1)
)

// List styles.
var (
	ListItem = lipgloss.NewStyle().
		PaddingLeft(2)

	SelectedItem = lipgloss.NewStyle().
		Foreground(Primary).
		Bold(true).
		PaddingLeft(2)
)

// Help text style.
var (
	HelpKey = lipgloss.NewStyle().
		Foreground(Primary).
		Bold(true)

	HelpText = lipgloss.NewStyle().
		Foreground(Muted)

	HelpSeparator = lipgloss.NewStyle().
		Foreground(Muted).
		SetString(" • ")
)

// Spinner style.
var SpinnerStyle = lipgloss.NewStyle().
	Foreground(Primary)

// Render helpers.

// RenderSuccess returns a styled success message.
func RenderSuccess(msg string) string {
	return SuccessText.Render("✓ ") + msg
}

// RenderWarning returns a styled warning message.
func RenderWarning(msg string) string {
	return WarningText.Render("⚠ ") + msg
}

// RenderError returns a styled error message.
func RenderError(msg string) string {
	return ErrorText.Render("✗ ") + msg
}

// RenderTitle returns a styled title.
func RenderTitle(title string) string {
	return Title.Render(title)
}

// RenderSubtitle returns a styled subtitle.
func RenderSubtitle(subtitle string) string {
	return Subtitle.Render(subtitle)
}
