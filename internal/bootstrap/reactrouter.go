package bootstrap

import (
	"archive/tar"
	"compress/gzip"
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/markmals/workbench/internal/config"
)

const reactRouterRepo = "remix-run/react-router-templates"

// ReactRouterTemplateName picks an upstream template based on deployment target.
func ReactRouterTemplateName(cfg *config.Config) string {
	if cfg.Website != nil && strings.EqualFold(cfg.Website.Deployment.Target, "cloudflare") {
		return "cloudflare"
	}
	return "default"
}

// FetchReactRouterTemplate downloads (or reuses cache) and returns the path to the upstream template directory.
func FetchReactRouterTemplate(ctx context.Context, ref, template string, refresh bool) (string, error) {
	if ref == "" || strings.EqualFold(ref, "latest") {
		ref = "main"
		refresh = true // refresh latest on every run to follow upstream
	}

	cacheDir, err := cacheDir(ref)
	if err != nil {
		return "", err
	}

	if !refresh {
		if cached, ok := findTemplateDir(cacheDir, template); ok {
			return cached, nil
		}
	}

	if refresh {
		_ = os.RemoveAll(cacheDir)
	}
	if err := os.MkdirAll(cacheDir, 0o755); err != nil {
		return "", fmt.Errorf("creating cache dir: %w", err)
	}

	url := fmt.Sprintf("https://codeload.github.com/%s/tar.gz/%s", reactRouterRepo, ref)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return "", fmt.Errorf("building request: %w", err)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("downloading templates: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("download failed: %s", resp.Status)
	}

	rootDir, err := extractTarGz(cacheDir, resp.Body)
	if err != nil {
		return "", fmt.Errorf("extracting templates: %w", err)
	}

	templatePath := filepath.Join(rootDir, template)
	if _, err := os.Stat(templatePath); err != nil {
		return "", fmt.Errorf("template %s not found in upstream archive (%s)", template, ref)
	}
	return templatePath, nil
}

// CopyTemplate copies the upstream template contents into the destination directory.
func CopyTemplate(src, dst string) error {
	return filepath.WalkDir(src, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		if rel == "." {
			return nil
		}
		// Skip git metadata
		if d.IsDir() && (d.Name() == ".git" || d.Name() == ".github") {
			return filepath.SkipDir
		}

		destPath := filepath.Join(dst, rel)
		if d.IsDir() {
			return os.MkdirAll(destPath, 0o755)
		}

		return copyFile(path, destPath)
	})
}

func cacheDir(ref string) (string, error) {
	userCache, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("locating cache dir: %w", err)
	}
	return filepath.Join(userCache, "workbench", "templates", "react-router", ref), nil
}

func findTemplateDir(base, template string) (string, bool) {
	entries, err := os.ReadDir(base)
	if err != nil {
		return "", false
	}

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		candidate := filepath.Join(base, entry.Name(), template)
		if info, err := os.Stat(candidate); err == nil && info.IsDir() {
			return candidate, true
		}
	}
	return "", false
}

func extractTarGz(dst string, r io.Reader) (string, error) {
	gzr, err := gzip.NewReader(r)
	if err != nil {
		return "", fmt.Errorf("opening gzip: %w", err)
	}
	defer gzr.Close()

	tr := tar.NewReader(gzr)
	var root string

	for {
		header, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return "", fmt.Errorf("reading tar: %w", err)
		}

		if root == "" {
			parts := strings.Split(header.Name, "/")
			if len(parts) > 0 {
				root = parts[0]
			}
		}

		targetPath := filepath.Join(dst, header.Name)

		switch header.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(targetPath, os.FileMode(header.Mode)); err != nil {
				return "", fmt.Errorf("creating dir %s: %w", targetPath, err)
			}
		case tar.TypeReg:
			if err := os.MkdirAll(filepath.Dir(targetPath), 0o755); err != nil {
				return "", fmt.Errorf("creating dir for %s: %w", targetPath, err)
			}
			f, err := os.OpenFile(targetPath, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, os.FileMode(header.Mode))
			if err != nil {
				return "", fmt.Errorf("creating file %s: %w", targetPath, err)
			}
			if _, err := io.Copy(f, tr); err != nil {
				_ = f.Close()
				return "", fmt.Errorf("writing file %s: %w", targetPath, err)
			}
			_ = f.Close()
		}
	}

	if root == "" {
		return "", fmt.Errorf("no root directory found in archive")
	}

	return filepath.Join(dst, root), nil
}

func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	stat, err := in.Stat()
	if err != nil {
		return err
	}

	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}

	out, err := os.OpenFile(dst, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, stat.Mode())
	if err != nil {
		return err
	}
	defer out.Close()

	if _, err := io.Copy(out, in); err != nil {
		return err
	}
	return nil
}
