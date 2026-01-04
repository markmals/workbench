package shell

import (
	"bytes"
	"context"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// Runner executes shell commands.
type Runner struct {
	// Dir is the working directory for commands.
	Dir string

	// Env is additional environment variables.
	Env []string

	// Stdout captures standard output (if nil, discarded).
	Stdout *bytes.Buffer

	// Stderr captures standard error (if nil, discarded).
	Stderr *bytes.Buffer
}

// New creates a Runner for the given directory.
func New(dir string) *Runner {
	return &Runner{
		Dir:    dir,
		Stdout: &bytes.Buffer{},
		Stderr: &bytes.Buffer{},
	}
}

// Run executes a command and returns any error.
func (r *Runner) Run(ctx context.Context, name string, args ...string) error {
	cmd := exec.CommandContext(ctx, name, args...)
	cmd.Dir = r.Dir
	cmd.Env = append(os.Environ(), r.Env...)

	if r.Stdout != nil {
		cmd.Stdout = r.Stdout
	}
	if r.Stderr != nil {
		cmd.Stderr = r.Stderr
	}

	if err := cmd.Run(); err != nil {
		stderr := ""
		if r.Stderr != nil {
			stderr = strings.TrimSpace(r.Stderr.String())
		}
		if stderr != "" {
			return fmt.Errorf("%s %s: %w\n%s", name, strings.Join(args, " "), err, stderr)
		}
		return fmt.Errorf("%s %s: %w", name, strings.Join(args, " "), err)
	}

	return nil
}

// Output runs a command and returns its stdout.
func (r *Runner) Output(ctx context.Context, name string, args ...string) (string, error) {
	r.Stdout = &bytes.Buffer{}
	r.Stderr = &bytes.Buffer{}

	if err := r.Run(ctx, name, args...); err != nil {
		return "", err
	}

	return strings.TrimSpace(r.Stdout.String()), nil
}
