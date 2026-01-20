// Package ghx provides GitHub CLI (gh) wrappers.
package ghx

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os/exec"
	"strings"

	"github.com/markmals/workbench/internal/i18n"
)

// Options configures gh operations.
type Options struct {
	// DryRun shows what would be done without making changes.
	DryRun bool
}

// IsInstalled checks if the gh CLI is available.
func IsInstalled() bool {
	cmd := exec.Command("gh", "--version")
	return cmd.Run() == nil
}

// IsAuthenticated checks if the user is logged in to GitHub.
func IsAuthenticated() bool {
	cmd := exec.Command("gh", "auth", "status")
	return cmd.Run() == nil
}

// EnsureAuth checks that gh is installed and authenticated.
// Returns a user-friendly error if not.
func EnsureAuth() error {
	if !IsInstalled() {
		return errors.New(i18n.T("ErrGhNotInstalled"))
	}
	if !IsAuthenticated() {
		return errors.New(i18n.T("ErrGhNotAuthenticated"))
	}
	return nil
}

// Repo represents a GitHub repository.
type Repo struct {
	Name        string `json:"name"`
	FullName    string `json:"nameWithOwner"`
	Description string `json:"description"`
	URL         string `json:"url"`
	SSHURL      string `json:"sshUrl"`
	IsPrivate   bool   `json:"isPrivate"`
	IsArchived  bool   `json:"isArchived"`
}

// CreateRepo creates a new GitHub repository.
func CreateRepo(ctx context.Context, name string, private bool, opts Options) (*Repo, error) {
	if opts.DryRun {
		visibility := "public"
		if private {
			visibility = "private"
		}
		fmt.Println(i18n.T("WouldCreateRepo", i18n.M{"Visibility": visibility, "Repo": name}))
		return &Repo{Name: name}, nil
	}

	args := []string{"repo", "create", name}
	if private {
		args = append(args, "--private")
	} else {
		args = append(args, "--public")
	}

	cmd := exec.CommandContext(ctx, "gh", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("gh repo create: %s", strings.TrimSpace(string(out)))
	}

	return &Repo{Name: name, FullName: name}, nil
}

// DeleteRepo deletes a GitHub repository.
func DeleteRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldDeleteRepo", i18n.M{"Repo": nameWithOwner}))
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "delete", nameWithOwner, "--yes")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("gh repo delete: %s", strings.TrimSpace(string(out)))
	}

	return nil
}

// CloneRepo clones a GitHub repository.
func CloneRepo(ctx context.Context, nameWithOwner, dest string, opts Options) error {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldCloneRepo", i18n.M{"Repo": nameWithOwner, "Dest": dest}))
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "clone", nameWithOwner, dest)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("gh repo clone: %s", strings.TrimSpace(string(out)))
	}

	return nil
}

// ForkRepo forks a repository.
func ForkRepo(ctx context.Context, nameWithOwner string, opts Options) (*Repo, error) {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldForkRepo", i18n.M{"Repo": nameWithOwner}))
		return &Repo{FullName: nameWithOwner}, nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "fork", nameWithOwner, "--clone=false")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("gh repo fork: %s", strings.TrimSpace(string(out)))
	}

	return &Repo{FullName: nameWithOwner}, nil
}

// ArchiveRepo archives a repository.
func ArchiveRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldArchiveRepo", i18n.M{"Repo": nameWithOwner}))
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "archive", nameWithOwner, "--yes")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("gh repo archive: %s", strings.TrimSpace(string(out)))
	}

	return nil
}

// UnarchiveRepo unarchives a repository.
func UnarchiveRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldUnarchiveRepo", i18n.M{"Repo": nameWithOwner}))
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "unarchive", nameWithOwner, "--yes")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("gh repo unarchive: %s", strings.TrimSpace(string(out)))
	}

	return nil
}

// GetRepo gets information about a repository.
func GetRepo(ctx context.Context, nameWithOwner string) (*Repo, error) {
	cmd := exec.CommandContext(ctx, "gh", "repo", "view", nameWithOwner, "--json", "name,nameWithOwner,description,url,sshUrl,isPrivate,isArchived")
	out, err := cmd.Output()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, fmt.Errorf("gh repo view: %s", string(exitErr.Stderr))
		}
		return nil, fmt.Errorf("gh repo view: %w", err)
	}

	var repo Repo
	if err := json.Unmarshal(out, &repo); err != nil {
		return nil, fmt.Errorf("parsing response: %w", err)
	}

	return &repo, nil
}

// RepoExists checks if a repository exists.
func RepoExists(ctx context.Context, nameWithOwner string) bool {
	_, err := GetRepo(ctx, nameWithOwner)
	return err == nil
}

// ListRepos lists repositories for an owner (user or org).
func ListRepos(ctx context.Context, owner string, limit int) ([]Repo, error) {
	args := []string{"repo", "list", owner, "--json", "name,nameWithOwner,description,url,sshUrl,isPrivate,isArchived"}
	if limit > 0 {
		args = append(args, "--limit", fmt.Sprintf("%d", limit))
	}

	cmd := exec.CommandContext(ctx, "gh", args...)
	out, err := cmd.Output()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, fmt.Errorf("gh repo list: %s", string(exitErr.Stderr))
		}
		return nil, fmt.Errorf("gh repo list: %w", err)
	}

	var repos []Repo
	if err := json.Unmarshal(out, &repos); err != nil {
		return nil, fmt.Errorf("parsing response: %w", err)
	}

	return repos, nil
}

// TransferRepo transfers a repository to a new owner.
func TransferRepo(ctx context.Context, nameWithOwner, newOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Println(i18n.T("WouldTransferRepo", i18n.M{"Repo": nameWithOwner, "NewOwner": newOwner}))
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "rename", nameWithOwner, "--repo", newOwner+"/"+repoNameFromFull(nameWithOwner))
	if err := cmd.Run(); err != nil {
		// Fall back to API for transfer
		cmd = exec.CommandContext(ctx, "gh", "api", "-X", "POST",
			fmt.Sprintf("/repos/%s/transfer", nameWithOwner),
			"-f", fmt.Sprintf("new_owner=%s", newOwner))
		if err := cmd.Run(); err != nil {
			if exitErr, ok := err.(*exec.ExitError); ok {
				return fmt.Errorf("gh api transfer: %s", string(exitErr.Stderr))
			}
			return fmt.Errorf("gh api transfer: %w", err)
		}
	}

	return nil
}

func repoNameFromFull(nameWithOwner string) string {
	parts := strings.Split(nameWithOwner, "/")
	if len(parts) == 2 {
		return parts[1]
	}
	return nameWithOwner
}
