// Package gitx provides git repository utilities.
package gitx

import (
	"context"
	"os/exec"
	"path/filepath"
	"strings"
)

// IsRepo checks if the given directory is inside a git repository.
func IsRepo(dir string) bool {
	cmd := exec.Command("git", "-C", dir, "rev-parse", "--git-dir")
	return cmd.Run() == nil
}

// TopLevel returns the root directory of the git repository.
func TopLevel(dir string) (string, error) {
	cmd := exec.Command("git", "-C", dir, "rev-parse", "--show-toplevel")
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

// IsClean returns true if the working tree has no uncommitted changes.
func IsClean(dir string) bool {
	cmd := exec.Command("git", "-C", dir, "status", "--porcelain")
	out, err := cmd.Output()
	if err != nil {
		return false
	}
	return len(strings.TrimSpace(string(out))) == 0
}

// CurrentBranch returns the name of the current branch.
func CurrentBranch(dir string) (string, error) {
	cmd := exec.Command("git", "-C", dir, "branch", "--show-current")
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

// DefaultBranch returns the default branch name (main or master).
func DefaultBranch(dir string) string {
	// Try to get from remote
	cmd := exec.Command("git", "-C", dir, "symbolic-ref", "refs/remotes/origin/HEAD")
	out, err := cmd.Output()
	if err == nil {
		ref := strings.TrimSpace(string(out))
		if strings.HasPrefix(ref, "refs/remotes/origin/") {
			return strings.TrimPrefix(ref, "refs/remotes/origin/")
		}
	}

	// Fallback: check if main or master exists
	if branchExists(dir, "main") {
		return "main"
	}
	if branchExists(dir, "master") {
		return "master"
	}

	return "main" // Default assumption
}

func branchExists(dir, branch string) bool {
	cmd := exec.Command("git", "-C", dir, "show-ref", "--verify", "--quiet", "refs/heads/"+branch)
	return cmd.Run() == nil
}

// Remote represents a git remote.
type Remote struct {
	Name     string
	FetchURL string
	PushURL  string
}

// Remotes returns all configured remotes.
func Remotes(dir string) ([]Remote, error) {
	cmd := exec.Command("git", "-C", dir, "remote", "-v")
	out, err := cmd.Output()
	if err != nil {
		return nil, err
	}

	remoteMap := make(map[string]*Remote)
	for _, line := range strings.Split(string(out), "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		parts := strings.Fields(line)
		if len(parts) < 3 {
			continue
		}

		name := parts[0]
		url := parts[1]
		typ := strings.Trim(parts[2], "()")

		r, ok := remoteMap[name]
		if !ok {
			r = &Remote{Name: name}
			remoteMap[name] = r
		}

		switch typ {
		case "fetch":
			r.FetchURL = url
		case "push":
			r.PushURL = url
		}
	}

	remotes := make([]Remote, 0, len(remoteMap))
	for _, r := range remoteMap {
		remotes = append(remotes, *r)
	}
	return remotes, nil
}

// HasRemote checks if a remote with the given name exists.
func HasRemote(dir, name string) bool {
	cmd := exec.Command("git", "-C", dir, "remote", "get-url", name)
	return cmd.Run() == nil
}

// RemoteURL returns the URL for the given remote.
func RemoteURL(dir, name string) (string, error) {
	cmd := exec.Command("git", "-C", dir, "remote", "get-url", name)
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

// Init initializes a new git repository.
func Init(ctx context.Context, dir string) error {
	cmd := exec.CommandContext(ctx, "git", "-C", dir, "init")
	return cmd.Run()
}

// Add stages files for commit.
func Add(ctx context.Context, dir string, paths ...string) error {
	args := append([]string{"-C", dir, "add"}, paths...)
	cmd := exec.CommandContext(ctx, "git", args...)
	return cmd.Run()
}

// Commit creates a commit with the given message.
func Commit(ctx context.Context, dir, message string) error {
	cmd := exec.CommandContext(ctx, "git", "-C", dir, "commit", "-m", message)
	return cmd.Run()
}

// RepoName extracts the repository name from a remote URL.
func RepoName(remoteURL string) string {
	// Handle both HTTPS and SSH URLs
	// https://github.com/user/repo.git -> repo
	// git@github.com:user/repo.git -> repo

	url := remoteURL
	url = strings.TrimSuffix(url, ".git")

	// Get the last path component
	if idx := strings.LastIndex(url, "/"); idx >= 0 {
		return url[idx+1:]
	}
	if idx := strings.LastIndex(url, ":"); idx >= 0 {
		return url[idx+1:]
	}

	return filepath.Base(url)
}
