package cli

import (
	"context"
	"fmt"
	"os/exec"
	"path/filepath"

	"github.com/charmbracelet/huh"
	"github.com/markmals/workbench/internal/ghx"
	"github.com/markmals/workbench/internal/gitx"
	"github.com/markmals/workbench/internal/i18n"
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
		return fmt.Errorf(i18n.T("ErrNotGitRepo", i18n.M{"Path": dir}))
	}

	// 2. Verify git tree is clean
	if !gitx.IsClean(dir) {
		return fmt.Errorf(i18n.T("ErrDirtyWorkingTree"))
	}

	// Get repo name from directory
	repoName := filepath.Base(dir)
	archiveRepo := fmt.Sprintf("%s/%s", c.Org, repoName)

	// 3. Confirm archive
	if !c.Yes && !c.DryRun {
		var confirm bool
		err := huh.NewConfirm().
			Title(i18n.T("ArchiveConfirmTitle")).
			Description(i18n.T("ArchiveConfirmDesc", i18n.M{"Repo": archiveRepo})).
			Affirmative(i18n.T("ArchiveConfirmYes")).
			Negative(i18n.T("ArchiveConfirmNo")).
			Value(&confirm).
			Run()
		if err != nil {
			return err
		}
		if !confirm {
			fmt.Println(i18n.T("Cancelled"))
			return nil
		}
	}

	// 4. Check if gh is available and authenticated
	if !ghx.IsInstalled() {
		return fmt.Errorf(i18n.T("ErrGhNotInstalled"))
	}
	if !ghx.IsAuthenticated() {
		return fmt.Errorf(i18n.T("ErrGhNotAuthenticated"))
	}

	// 5. Create repo in archive org (or use existing)
	ghOpts := ghx.Options{DryRun: c.DryRun}

	if !ghx.RepoExists(bgCtx, archiveRepo) {
		err = ui.RunWithSpinner(bgCtx, i18n.T("ArchiveCreatingRepo", i18n.M{"Repo": archiveRepo}), func() error {
			_, err := ghx.CreateRepo(bgCtx, archiveRepo, true, ghOpts)
			return err
		})
		if err != nil {
			return err
		}
	}

	// 6. Add remote and push
	if !c.DryRun {
		err = ui.RunWithSpinner(bgCtx, i18n.T("ArchivePushing"), func() error {
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
		fmt.Println(i18n.T("ArchiveWouldPush", i18n.M{"Repo": archiveRepo}))
	}

	// 7. Archive the repo on GitHub
	err = ui.RunWithSpinner(bgCtx, i18n.T("ArchiveMarking"), func() error {
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
		fmt.Println()
		fmt.Println(i18n.T("ArchiveSuccess", i18n.M{"Repo": archiveRepo}))
	}

	return nil
}
