import { readdirSync, readFileSync, statSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { DefaultTheme } from "vitepress";

const repoRoot = fileURLToPath(new URL("../..", import.meta.url));

interface SidebarBuild {
    specs: DefaultTheme.SidebarItem[];
    features: DefaultTheme.SidebarItem[];
    firstFeatureLink: string | null;
}

export function generateSidebar(): SidebarBuild {
    return {
        specs: buildSpecsSidebar(),
        features: buildFeaturesSidebar(),
        firstFeatureLink: firstFeatureLink(),
    };
}

// ---- specs/ ----

function buildSpecsSidebar(): DefaultTheme.SidebarItem[] {
    const specsDir = join(repoRoot, "specs");
    if (!exists(specsDir)) return [];

    const cross: DefaultTheme.SidebarItem[] = [];
    const subdirs: DefaultTheme.SidebarItem[] = [];

    for (const entry of readdirSyncSafe(specsDir)) {
        const fullPath = join(specsDir, entry);
        const isDir = statSync(fullPath).isDirectory();

        if (isDir) {
            const items = listMarkdown(fullPath, `/specs/${entry}/`);
            if (items.length > 0) {
                subdirs.push({
                    text: titleCase(entry),
                    collapsed: true,
                    items,
                });
            }
        } else if (extname(entry) === ".md") {
            const stem = entry.slice(0, -3);
            cross.push({
                text: deriveTitle(fullPath, stem),
                link: `/specs/${stem}`,
            });
        }
    }

    return [...(cross.length > 0 ? [{ text: "Cross-cutting", items: cross }] : []), ...subdirs];
}

// ---- features/ ----

function buildFeaturesSidebar(): DefaultTheme.SidebarItem[] {
    const featuresDir = join(repoRoot, "features");
    if (!exists(featuresDir)) return [];

    const items: DefaultTheme.SidebarItem[] = [];
    for (const entry of readdirSyncSafe(featuresDir).sort()) {
        const fullPath = join(featuresDir, entry);
        if (!statSync(fullPath).isDirectory()) continue;

        const featureItems = walkFeatureFolder(fullPath, `/features/${entry}/`);
        if (featureItems.length === 0) continue;

        items.push({
            text: featureFolderTitle(entry),
            collapsed: false,
            items: featureItems,
        });
    }
    return items;
}

// Top-level files in a feature folder map to fixed sidebar labels in a fixed
// order. Anything not in this list falls back to deriving from `# heading`.
const KNOWN_TOP_FILES: Array<{ stem: string; title: string }> = [
    { stem: "README", title: "Overview" },
    { stem: "NARRATIVE", title: "Narrative" },
];

function walkFeatureFolder(absDir: string, urlPrefix: string): DefaultTheme.SidebarItem[] {
    const items: DefaultTheme.SidebarItem[] = [];

    const entries = readdirSyncSafe(absDir);
    const topFiles: DefaultTheme.SidebarItem[] = [];
    const subdirs: DefaultTheme.SidebarItem[] = [];

    const knownStems = new Set(KNOWN_TOP_FILES.map((f) => f.stem));
    const presentMd = new Set(
        entries.filter((e) => extname(e) === ".md").map((e) => e.slice(0, -3)),
    );

    // Known top files first, in declared order
    for (const { stem, title } of KNOWN_TOP_FILES) {
        if (!presentMd.has(stem)) continue;
        topFiles.push({
            text: title,
            link: `${urlPrefix}${stem}`,
        });
    }

    // Other top-level files alphabetically; then subdirs
    for (const entry of entries) {
        const fullPath = join(absDir, entry);
        const isDir = statSync(fullPath).isDirectory();

        if (isDir) {
            const dirItems = listMarkdown(fullPath, `${urlPrefix}${entry}/`);
            if (dirItems.length > 0) {
                subdirs.push({
                    text: titleCase(entry),
                    collapsed: true,
                    items: dirItems,
                });
            }
        } else if (extname(entry) === ".md") {
            const stem = entry.slice(0, -3);
            if (knownStems.has(stem)) continue;
            topFiles.push({
                text: deriveTitle(fullPath, stem),
                link: `${urlPrefix}${stem}`,
            });
        }
    }

    items.push(...topFiles);
    items.push(...subdirs);
    return items;
}

// "0001-managing-contacts" → "0001: Managing Contacts"
function featureFolderTitle(slug: string): string {
    const match = slug.match(/^(\d+)-(.+)$/);
    if (!match) return titleCase(slug);
    const [, num, rest] = match;
    return `${num}: ${titleCase(rest)}`;
}

// ---- helpers ----

function listMarkdown(absDir: string, urlPrefix: string): DefaultTheme.SidebarItem[] {
    return readdirSyncSafe(absDir)
        .filter((name) => extname(name) === ".md")
        .sort()
        .map((name) => {
            const stem = name.slice(0, -3);
            return {
                text: deriveTitle(join(absDir, name), stem),
                link: `${urlPrefix}${stem}`,
            };
        });
}

function deriveTitle(filePath: string, fallback: string): string {
    try {
        const content = readFileSync(filePath, "utf-8");
        // Skip frontmatter
        const body = content.replace(/^---[\s\S]*?---\n?/, "");
        const match = body.match(/^#\s+(.+)$/m);
        if (match) return match[1].trim().replace(/\s*<!--.*?-->\s*/g, "");
    } catch {
        /* fall through */
    }
    return titleCase(fallback);
}

function titleCase(slug: string): string {
    return slug
        .replace(/^\d+-/, "") // strip "0001-" prefix from feature folders
        .replace(/[-_.]/g, " ")
        .replace(/\b\w/g, (c) => c.toUpperCase());
}

function readdirSyncSafe(dir: string): string[] {
    try {
        return readdirSync(dir).filter((n) => !n.startsWith(".") && n !== "node_modules");
    } catch {
        return [];
    }
}

function exists(p: string): boolean {
    try {
        statSync(p);
        return true;
    } catch {
        return false;
    }
}

function firstFeatureLink(): string | null {
    const featuresDir = join(repoRoot, "features");
    if (!exists(featuresDir)) return null;
    const entries = readdirSyncSafe(featuresDir)
        .filter((n) => statSync(join(featuresDir, n)).isDirectory())
        .sort();
    if (entries.length === 0) return null;
    const first = entries[0];
    // Prefer README (Overview); fall back to NARRATIVE; fall back to the folder index
    if (exists(join(featuresDir, first, "README.md"))) {
        return `/features/${first}/README`;
    }
    if (exists(join(featuresDir, first, "NARRATIVE.md"))) {
        return `/features/${first}/NARRATIVE`;
    }
    return `/features/${first}/`;
}
