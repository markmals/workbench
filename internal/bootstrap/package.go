package bootstrap

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/markmals/workbench/internal/config"
)

var (
	baseRuntimeDeps = map[string]string{
		"isbot":                                 "^5.1.31",
		"react":                                 "^19.2.3",
		"react-dom":                             "^19.2.3",
		"react-router":                          "7.12.0",
		"cva":                                   "1.0.0-beta.4",
		"react-concurrent-store":                "^0.0.1",
		"drizzle-orm":                           "^0.44.6",
		"drizzle-zod":                           "^0.8.3",
		"zod":                                   "^4.1.13",
		"@withsprinkles/react-router-route-map": "^0.1.0",
	}

	baseDevDeps = map[string]string{
		"@babel/core":                  "^7.26.7",
		"@babel/preset-typescript":     "^7.26.0",
		"@prettier/plugin-oxc":         "^0.1.3",
		"@react-router/dev":            "7.12.0",
		"@tailwindcss/vite":            "^4.1.13",
		"@types/node":                  "^24.10.1",
		"@types/react":                 "^19.2.7",
		"@types/react-dom":             "^19.2.3",
		"drizzle-kit":                  "^0.31.5",
		"oxlint":                       "^1.41.0",
		"prettier":                     "^3.8.0",
		"prettier-plugin-pkg":          "^0.21.2",
		"prettier-plugin-sh":           "^0.18.0",
		"prettier-plugin-sort-imports": "^1.8.9",
		"prettier-plugin-tailwindcss":  "^0.7.2",
		"prettier-plugin-toml":         "^2.0.6",
		"tailwindcss":                  "^4.1.13",
		"typescript":                   "^5.9.2",
		"babel-plugin-react-compiler":  "^1.0.0",
		"vite":                         "8.0.0-beta.8",
		"vite-plugin-babel":            "^1.4.1",
		"vite-plugin-devtools-json":    "1.0.0",
	}
)

// ApplyPackagePreferences merges upstream package.json with Workbench defaults.
func ApplyPackagePreferences(dir string, cfg *config.Config) error {
	if cfg == nil || strings.ToLower(cfg.Kind) != "website" {
		return nil
	}

	path := filepath.Join(dir, "package.json")

	pkg := map[string]any{}
	if data, err := os.ReadFile(path); err == nil {
		_ = json.Unmarshal(data, &pkg)
	}

	name := sanitizeName(cfg.Project.Name)
	pkg["name"] = name
	pkg["private"] = true
	pkg["type"] = "module"
	pkg["sideEffects"] = false

	scripts := ensureStringMap(pkg, "scripts")
	mergeScripts(scripts, cfg)
	pkg["scripts"] = scripts

	deps := ensureStringMap(pkg, "dependencies")
	devDeps := ensureStringMap(pkg, "devDependencies")

	mergeDependencies(deps, devDeps, cfg)
	pkg["dependencies"] = deps
	pkg["devDependencies"] = devDeps

	pnpm := ensureMap(pkg, "pnpm")
	overrides := ensureStringMap(pnpm, "overrides")
	overrides["vite"] = "8.0.0-beta.8"
	pnpm["overrides"] = overrides
	pkg["pnpm"] = pnpm

	data, err := json.MarshalIndent(pkg, "", "  ")
	if err != nil {
		return fmt.Errorf("marshaling package.json: %w", err)
	}

	if err := os.WriteFile(path, data, 0o644); err != nil {
		return fmt.Errorf("writing package.json: %w", err)
	}

	return nil
}

func mergeScripts(scripts map[string]string, cfg *config.Config) {
	target := ""
	if cfg.Website != nil {
		target = strings.ToLower(cfg.Website.Deployment.Target)
	}

	common := map[string]string{
		"typecheck": "react-router typegen && tsc",
		"lint":      "oxlint",
		"fmt":       "prettier --write .",
		"fix":       "pnpm fmt && pnpm lint",
	}

	cloudflare := map[string]string{
		"dev":         "react-router dev",
		"build":       "react-router build",
		"preview":     "npm run build && vite preview",
		"deploy":      "npm run build && wrangler deploy",
		"cf-typegen":  "wrangler types",
		"typecheck":   "npm run cf-typegen && react-router typegen && tsc -b",
		"postinstall": "npm run cf-typegen",
	}

	defaultScripts := map[string]string{
		"build":   "react-router build",
		"dev":     "react-router dev",
		"start":   "react-router-serve ./build/server/index.js",
		"preview": "npm run build && vite preview",
	}

	for k, v := range common {
		scripts[k] = v
	}

	if target == "cloudflare" {
		for k, v := range cloudflare {
			scripts[k] = v
		}
		return
	}

	for k, v := range defaultScripts {
		scripts[k] = v
	}
}

func mergeDependencies(deps, devDeps map[string]string, cfg *config.Config) {
	for k, v := range baseRuntimeDeps {
		deps[k] = v
	}
	if cfg.Website != nil && !cfg.Website.RouteMap {
		delete(deps, "@withsprinkles/react-router-route-map")
	}

	for k, v := range baseDevDeps {
		devDeps[k] = v
	}

	target := ""
	if cfg.Website != nil {
		target = strings.ToLower(cfg.Website.Deployment.Target)
	}

	if target == "cloudflare" {
		devDeps["@cloudflare/vite-plugin"] = "^1.13.11"
		devDeps["wrangler"] = "^4.42.1"
		// Cloudflare template doesn't need react-router node/serve at runtime.
		delete(deps, "@react-router/node")
		delete(deps, "@react-router/serve")
	} else {
		deps["@react-router/node"] = "7.12.0"
		deps["@react-router/serve"] = "7.12.0"
	}
}

func ensureStringMap(m map[string]any, key string) map[string]string {
	raw, ok := m[key]
	if !ok {
		res := map[string]string{}
		m[key] = res
		return res
	}

	if strMap, ok := raw.(map[string]string); ok {
		return strMap
	}

	res := map[string]string{}
	if ifaceMap, ok := raw.(map[string]any); ok {
		for k, v := range ifaceMap {
			if s, ok := v.(string); ok {
				res[k] = s
			}
		}
	}
	m[key] = res
	return res
}

func ensureMap(m map[string]any, key string) map[string]any {
	raw, ok := m[key]
	if !ok {
		res := map[string]any{}
		m[key] = res
		return res
	}
	if inner, ok := raw.(map[string]any); ok {
		return inner
	}
	res := map[string]any{}
	m[key] = res
	return res
}

func sanitizeName(name string) string {
	slug := strings.ToLower(strings.TrimSpace(name))
	slug = regexp.MustCompile(`[^a-z0-9-_]+`).ReplaceAllString(slug, "-")
	slug = strings.Trim(slug, "-")
	if slug == "" {
		return "app"
	}
	return slug
}
