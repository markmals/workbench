// Package ghx provides GitHub CLI (gh) wrappers.
package ghx

import (
	"context"
	"encoding/json"
	"fmt"
	"os/exec"
	"strings"
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

// Repo represents a GitHub repository.
type Repo struct {
	Name        string `json:"name"`
	FullName    string `json:"nameWithOwner"`
	Description string `json:"description"`
	URL         string `json:"url"`
	SSHURL      string `json:"sshUrl"`
	CloneURL    string `json:"url"`
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
		fmt.Printf("Would create %s repository: %s\n", visibility, name)
		return &Repo{Name: name}, nil
	}

	args := []string{"repo", "create", name}
	if private {
		args = append(args, "--private")
	} else {
		args = append(args, "--public")
	}

	cmd := exec.CommandContext(ctx, "gh", args...)
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, fmt.Errorf("gh repo create: %s", string(exitErr.Stderr))
		}
		return nil, fmt.Errorf("gh repo create: %w", err)
	}

	return &Repo{Name: name, FullName: name}, nil
}

// DeleteRepo deletes a GitHub repository.
func DeleteRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Printf("Would delete repository: %s\n", nameWithOwner)
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "delete", nameWithOwner, "--yes")
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return fmt.Errorf("gh repo delete: %s", string(exitErr.Stderr))
		}
		return fmt.Errorf("gh repo delete: %w", err)
	}

	return nil
}

// CloneRepo clones a GitHub repository.
func CloneRepo(ctx context.Context, nameWithOwner, dest string, opts Options) error {
	if opts.DryRun {
		fmt.Printf("Would clone %s to %s\n", nameWithOwner, dest)
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "clone", nameWithOwner, dest)
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return fmt.Errorf("gh repo clone: %s", string(exitErr.Stderr))
		}
		return fmt.Errorf("gh repo clone: %w", err)
	}

	return nil
}

// ForkRepo forks a repository.
func ForkRepo(ctx context.Context, nameWithOwner string, opts Options) (*Repo, error) {
	if opts.DryRun {
		fmt.Printf("Would fork repository: %s\n", nameWithOwner)
		return &Repo{FullName: nameWithOwner}, nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "fork", nameWithOwner, "--clone=false")
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, fmt.Errorf("gh repo fork: %s", string(exitErr.Stderr))
		}
		return nil, fmt.Errorf("gh repo fork: %w", err)
	}

	return &Repo{FullName: nameWithOwner}, nil
}

// ArchiveRepo archives a repository.
func ArchiveRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Printf("Would archive repository: %s\n", nameWithOwner)
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "archive", nameWithOwner, "--yes")
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return fmt.Errorf("gh repo archive: %s", string(exitErr.Stderr))
		}
		return fmt.Errorf("gh repo archive: %w", err)
	}

	return nil
}

// UnarchiveRepo unarchives a repository.
func UnarchiveRepo(ctx context.Context, nameWithOwner string, opts Options) error {
	if opts.DryRun {
		fmt.Printf("Would unarchive repository: %s\n", nameWithOwner)
		return nil
	}

	cmd := exec.CommandContext(ctx, "gh", "repo", "unarchive", nameWithOwner, "--yes")
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return fmt.Errorf("gh repo unarchive: %s", string(exitErr.Stderr))
		}
		return fmt.Errorf("gh repo unarchive: %w", err)
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
		fmt.Printf("Would transfer %s to %s\n", nameWithOwner, newOwner)
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
