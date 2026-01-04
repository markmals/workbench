package logx

import (
	"os"

	"github.com/charmbracelet/log"
)

// New creates a new logger configured based on the provided options.
// If jsonMode is true, output is JSON formatted for machine consumption.
// If verbose is true, debug-level logging is enabled.
func New(jsonMode, verbose bool) *log.Logger {
	logger := log.NewWithOptions(os.Stderr, log.Options{
		ReportTimestamp: true,
	})

	// Set formatter based on mode
	if jsonMode {
		logger.SetFormatter(log.JSONFormatter)
	} else {
		logger.SetFormatter(log.TextFormatter)
	}

	// Set level based on verbosity
	if verbose {
		logger.SetLevel(log.DebugLevel)
	} else {
		logger.SetLevel(log.InfoLevel)
	}

	return logger
}

// WithPrefix returns a new logger with the given prefix.
func WithPrefix(logger *log.Logger, prefix string) *log.Logger {
	return logger.WithPrefix(prefix)
}
