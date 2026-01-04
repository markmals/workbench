package cli

import (
	"context"
	"fmt"
	"os/exec"
	"path/filepath"

	"github.com/charmbracelet/huh"
	"github.com/markmals/workbench/internal/ghx"
	"github.com/markmals/workbench/internal/gitx"
	"github.com/markmals/workbench/internal/ops"
	"github.com/markmals/workbench/internal/ui"
)

// ArchiveCmd archives a repo to the GitHub Archive org.
type ArchiveCmd struct {
	Dir        string `arg:"" optional:"" help:"Directory to archive" default:"." type:"path"`
	Org        string `help:"GitHub org to archive to" default:"markmals-archive" name:"org"`
	KeepLocal  bool   `help:"Don't delete local directory after archiving" name:"keep-local"`
	Yes        bool   `help:"Skip confirmation" short:"y"`
	DryRun     bool   `help:"Show what would happen without doing it" name:"dry-run"`
}

func (c *ArchiveCmd) Run(ctx *Context) error {
	bgCtx := context.Background()

	// Resolve directory
	dir, err := filepath.Abs(c.Dir)
	if err != nil {
		return fmt.Errorf("resolving directory: %w", err)
	}

	// 1. Verify target is a git repo
	if !gitx.IsRepo(dir) {
		return fmt.Errorf("not a git repository: %s", dir)
	}

	// 2. Verify git tree is clean
	if !gitx.IsClean(dir) {
		return fmt.Errorf("working tree has uncommitted changes")
	}

	// Get repo name from directory
	repoName := filepath.Base(dir)
	archiveRepo := fmt.Sprintf("%s/%s", c.Org, repoName)

	// 3. Confirm archive
	if !c.Yes && !c.DryRun {
		var confirm bool
		err := huh.NewConfirm().
			Title("Archive this project?").
			Description(fmt.Sprintf("Will push to %s and delete local copy", archiveRepo)).
			Affirmative("Yes, archive").
			Negative("Cancel").
			Value(&confirm).
			Run()
		if err != nil {
			return err
		}
		if !confirm {
			fmt.Println("Cancelled.")
			return nil
		}
	}

	// 4. Check if gh is available and authenticated
	if !ghx.IsInstalled() {
		return fmt.Errorf("GitHub CLI (gh) is not installed")
	}
	if !ghx.IsAuthenticated() {
		return fmt.Errorf("not logged in to GitHub (run: gh auth login)")
	}

	// 5. Create repo in archive org (or use existing)
	ghOpts := ghx.Options{DryRun: c.DryRun}

	if !ghx.RepoExists(bgCtx, archiveRepo) {
		err = ui.RunWithSpinner(bgCtx, fmt.Sprintf("Creating %s", archiveRepo), func() error {
			_, err := ghx.CreateRepo(bgCtx, archiveRepo, true, ghOpts)
			return err
		})
		if err != nil {
			return err
		}
	}

	// 6. Add remote and push
	if !c.DryRun {
		err = ui.RunWithSpinner(bgCtx, "Pushing to archive", func() error {
			// Add archive remote (use HTTPS with gh auth)
			remoteURL := fmt.Sprintf("https://github.com/%s.git", archiveRepo)
			_ = exec.Command("git", "-C", dir, "remote", "remove", "archive").Run() // Remove if exists
			if err := exec.Command("git", "-C", dir, "remote", "add", "archive", remoteURL).Run(); err != nil {
				return fmt.Errorf("adding remote: %w", err)
			}

			// Use gh to push (handles auth)
			pushCmd := exec.Command("gh", "repo", "sync", archiveRepo, "--source", dir, "--force")
			if out, err := pushCmd.CombinedOutput(); err != nil {
				// Fall back to git push
				gitPush := exec.Command("git", "-C", dir, "push", "archive", "--all")
				if out2, err2 := gitPush.CombinedOutput(); err2 != nil {
					return fmt.Errorf("%s: %w", string(out2), err2)
				}
			} else {
				_ = out // sync worked
			}

			// Push all tags
			tagsCmd := exec.Command("git", "-C", dir, "push", "archive", "--tags")
			if out, err := tagsCmd.CombinedOutput(); err != nil {
				return fmt.Errorf("pushing tags: %s: %w", string(out), err)
			}
			return nil
		})
		if err != nil {
			return fmt.Errorf("pushing to archive: %w", err)
		}
	} else {
		fmt.Printf("Would push all branches and tags to %s\n", archiveRepo)
	}

	// 7. Archive the repo on GitHub
	err = ui.RunWithSpinner(bgCtx, "Marking as archived on GitHub", func() error {
		return ghx.ArchiveRepo(bgCtx, archiveRepo, ghOpts)
	})
	if err != nil {
		return err
	}

	// 8. Delete local directory
	if !c.KeepLocal {
		err = ops.Remove(dir, ops.RemoveOptions{
			Force:  c.Yes,
			DryRun: c.DryRun,
		})
		if err != nil {
			return fmt.Errorf("removing local directory: %w", err)
		}
	}

	if !c.DryRun {
		fmt.Printf("\n✓ Archived to %s\n", archiveRepo)
	}

	return nil
}
