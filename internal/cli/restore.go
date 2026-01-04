package cli

import (
	"context"
	"fmt"
	"path/filepath"

	"github.com/charmbracelet/huh"
	"github.com/markmals/workbench/internal/ghx"
	"github.com/markmals/workbench/internal/ui"
)

// RestoreCmd restores a repo from the GitHub Archive org.
type RestoreCmd struct {
	Repo      string `arg:"" help:"Repository name to restore (without org prefix)"`
	Dir       string `arg:"" optional:"" help:"Directory to clone into" type:"path"`
	Org       string `help:"GitHub org to restore from" default:"markmals-archive" name:"org"`
	Rm        bool   `help:"Delete repo from archive after restoring" name:"rm"`
	Unarchive bool   `help:"Unarchive the repo on GitHub (make it active again)" name:"unarchive"`
	Yes       bool   `help:"Skip confirmation" short:"y"`
	DryRun    bool   `help:"Show what would happen without doing it" name:"dry-run"`
}

func (c *RestoreCmd) Run(ctx *Context) error {
	bgCtx := context.Background()

	// Resolve destination directory
	destDir := c.Dir
	if destDir == "" {
		destDir = c.Repo
	}
	destDir, err := filepath.Abs(destDir)
	if err != nil {
		return fmt.Errorf("resolving directory: %w", err)
	}

	archiveRepo := fmt.Sprintf("%s/%s", c.Org, c.Repo)

	// 1. Check if gh is available and authenticated
	if !ghx.IsInstalled() {
		return fmt.Errorf("GitHub CLI (gh) is not installed")
	}
	if !ghx.IsAuthenticated() {
		return fmt.Errorf("not logged in to GitHub (run: gh auth login)")
	}

	// 2. Check if repo exists in archive
	repo, err := ghx.GetRepo(bgCtx, archiveRepo)
	if err != nil {
		return fmt.Errorf("repository not found: %s", archiveRepo)
	}

	// 3. Confirm restore
	if !c.Yes && !c.DryRun {
		desc := fmt.Sprintf("Will clone %s to %s", archiveRepo, destDir)
		if c.Rm {
			desc += " and delete from archive"
		}
		if c.Unarchive {
			desc += " and unarchive on GitHub"
		}

		var confirm bool
		err := huh.NewConfirm().
			Title("Restore this project?").
			Description(desc).
			Affirmative("Yes, restore").
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

	ghOpts := ghx.Options{DryRun: c.DryRun}

	// 4. Unarchive on GitHub if requested (before clone so we can push changes later)
	if c.Unarchive && repo.IsArchived {
		err = ui.RunWithSpinner(bgCtx, "Unarchiving on GitHub", func() error {
			return ghx.UnarchiveRepo(bgCtx, archiveRepo, ghOpts)
		})
		if err != nil {
			return err
		}
	}

	// 5. Clone the repo
	err = ui.RunWithSpinner(bgCtx, fmt.Sprintf("Cloning %s", archiveRepo), func() error {
		return ghx.CloneRepo(bgCtx, archiveRepo, destDir, ghOpts)
	})
	if err != nil {
		return err
	}

	// 6. Delete from archive if requested
	if c.Rm {
		err = ui.RunWithSpinner(bgCtx, "Deleting from archive", func() error {
			return ghx.DeleteRepo(bgCtx, archiveRepo, ghOpts)
		})
		if err != nil {
			return err
		}
	}

	if !c.DryRun {
		fmt.Printf("\n✓ Restored %s to %s\n", c.Repo, destDir)
		if c.Rm {
			fmt.Printf("✓ Deleted %s from archive\n", archiveRepo)
		}
	}

	return nil
}
