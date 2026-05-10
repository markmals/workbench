// deno-lint-ignore-file no-explicit-any
import { defineConfig } from "vitepress";
import { groupIconMdPlugin, groupIconVitePlugin } from "vitepress-plugin-group-icons";
import { extendConfig } from "@voidzero-dev/vitepress-theme/config";
import { generateSidebar } from "./sidebar";
import { clarificationMarkerPlugin } from "./markdown/clarification";

const sidebar = generateSidebar();

const config = defineConfig({
    title: "Workbench",
    description: "A spec-driven multiplatform app harness — web, iOS, Android.",
    srcDir: "..",
    srcExclude: [
        "**/node_modules/**",
        ".claude/**",
        ".git/**",
        ".github/**",
        "docs/.vitepress/**",
        "docs/node_modules/**",
        "docs/public/**",
        "docs/package.json",
        "docs/mise.toml",
        "docs/pnpm-lock.yaml",
        "apps/**",
        "services/**",
    ],
    rewrites: {
        "docs/index.md": "index.md",
    },
    cleanUrls: true,
    markdown: {
        theme: { dark: "github-dark", light: "github-light" },
        config(md) {
            md.use(groupIconMdPlugin);
            md.use(clarificationMarkerPlugin);
        },
    },
    vite: {
        plugins: [groupIconVitePlugin() as any],
    },
    themeConfig: {
        logo: "/favicon.svg",
        outline: { level: "deep" },
        socialLinks: [{ icon: "github", link: "https://github.com/" }],
        nav: [
            { text: "Specs", link: "/specs/CONVENTIONS", activeMatch: "/specs/" },
            { text: "Features", link: sidebar.firstFeatureLink ?? "/", activeMatch: "/features/" },
        ],
        sidebar: {
            "/specs/": sidebar.specs,
            "/features/": sidebar.features,
        },
        search: { provider: "local" },
    },
    head: [
        ["link", { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" }],
        ["link", { rel: "preconnect", href: "https://fonts.googleapis.com" }],
        [
            "link",
            { rel: "preconnect", href: "https://fonts.gstatic.com", crossOrigin: "anonymous" },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Montserrat:ital,wght@0,600;0,700;0,800;0,900;1,600;1,700;1,800;1,900&display=swap",
            },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,400;0,500;0,600;0,700;1,400;1,500;1,600;1,700&display=swap",
            },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Inter:wght@100;200;300;400;500;600;700;800;900&display=swap",
            },
        ],
    ],
});

export default extendConfig(config);
