// Package ops provides filesystem operations with safety rails.
package ops

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/charmbracelet/huh"
)

// RemoveOptions configures the Remove operation.
type RemoveOptions struct {
	// Force skips confirmation prompts.
	Force bool

	// DryRun shows what would be done without making changes.
	DryRun bool
}

// Remove deletes a file or directory with confirmation.
func Remove(path string, opts RemoveOptions) error {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return fmt.Errorf("resolving path: %w", err)
	}

	info, err := os.Stat(absPath)
	if err != nil {
		if os.IsNotExist(err) {
			return nil // Already gone
		}
		return fmt.Errorf("checking path: %w", err)
	}

	desc := "file"
	if info.IsDir() {
		desc = "directory"
	}

	if opts.DryRun {
		fmt.Printf("Would remove %s: %s\n", desc, absPath)
		return nil
	}

	if !opts.Force {
		var confirm bool
		err := huh.NewConfirm().
			Title(fmt.Sprintf("Remove %s?", desc)).
			Description(absPath).
			Affirmative("Yes, delete").
			Negative("No, keep").
			Value(&confirm).
			Run()
		if err != nil {
			return fmt.Errorf("confirmation: %w", err)
		}
		if !confirm {
			return nil
		}
	}

	if err := os.RemoveAll(absPath); err != nil {
		return fmt.Errorf("removing %s: %w", desc, err)
	}

	return nil
}

// MoveOptions configures the Move operation.
type MoveOptions struct {
	// Force overwrites destination without confirmation.
	Force bool

	// DryRun shows what would be done without making changes.
	DryRun bool
}

// Move renames/moves a file or directory with confirmation for overwrites.
func Move(src, dst string, opts MoveOptions) error {
	absSrc, err := filepath.Abs(src)
	if err != nil {
		return fmt.Errorf("resolving source: %w", err)
	}

	absDst, err := filepath.Abs(dst)
	if err != nil {
		return fmt.Errorf("resolving destination: %w", err)
	}

	// Check source exists
	if _, err := os.Stat(absSrc); err != nil {
		return fmt.Errorf("source not found: %w", err)
	}

	// Check if destination exists
	dstExists := false
	if _, err := os.Stat(absDst); err == nil {
		dstExists = true
	}

	if opts.DryRun {
		if dstExists {
			fmt.Printf("Would move %s -> %s (overwriting)\n", absSrc, absDst)
		} else {
			fmt.Printf("Would move %s -> %s\n", absSrc, absDst)
		}
		return nil
	}

	if dstExists && !opts.Force {
		var confirm bool
		err := huh.NewConfirm().
			Title("Overwrite existing destination?").
			Description(absDst).
			Affirmative("Yes, overwrite").
			Negative("No, cancel").
			Value(&confirm).
			Run()
		if err != nil {
			return fmt.Errorf("confirmation: %w", err)
		}
		if !confirm {
			return nil
		}
		// Remove existing destination
		if err := os.RemoveAll(absDst); err != nil {
			return fmt.Errorf("removing destination: %w", err)
		}
	}

	// Ensure parent directory exists
	if err := os.MkdirAll(filepath.Dir(absDst), 0755); err != nil {
		return fmt.Errorf("creating destination directory: %w", err)
	}

	if err := os.Rename(absSrc, absDst); err != nil {
		return fmt.Errorf("moving: %w", err)
	}

	return nil
}

// EnsureDir creates a directory if it doesn't exist.
func EnsureDir(path string) error {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return fmt.Errorf("resolving path: %w", err)
	}

	if err := os.MkdirAll(absPath, 0755); err != nil {
		return fmt.Errorf("creating directory: %w", err)
	}

	return nil
}

// Exists checks if a path exists.
func Exists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

// IsDir checks if a path is a directory.
func IsDir(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return info.IsDir()
}

// IsFile checks if a path is a regular file.
func IsFile(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	return info.Mode().IsRegular()
}
