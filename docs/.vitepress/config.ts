import { defineConfig } from "vitepress";

export default defineConfig({
    title: "Workbench",
    description:
        "A personal CLI to bootstrap, evolve, and archive/restore projects",
    base: "/workbench/",

    lastUpdated: true,
    cleanUrls: true,

    markdown: {
        theme: {
            dark: "github-dark",
            light: "github-light",
        },
    },

    head: [
        ["link", { rel: "icon", href: "/workbench/favicon.ico" }],
        [
            "link",
            {
                rel: "icon",
                type: "image/png",
                sizes: "32x32",
                href: "/workbench/workbench-icon.png",
            },
        ],
        ["link", { rel: "preconnect", href: "https://fonts.googleapis.com" }],
        [
            "link",
            {
                rel: "preconnect",
                href: "https://fonts.gstatic.com",
                crossorigin: "",
            },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Montserrat:wght@600;700;800;900&display=swap",
            },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap",
            },
        ],
        [
            "link",
            {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap",
            },
        ],
        // Open Graph
        ["meta", { property: "og:type", content: "website" }],
        ["meta", { property: "og:title", content: "Workbench" }],
        [
            "meta",
            {
                property: "og:description",
                content: "A personal CLI to bootstrap, evolve, and archive/restore projects",
            },
        ],
        [
            "meta",
            {
                property: "og:image",
                content: "https://markmals.github.io/workbench/workbench-hero.png",
            },
        ],
        ["meta", { property: "og:url", content: "https://markmals.github.io/workbench/" }],
        // Twitter Card
        ["meta", { name: "twitter:card", content: "summary_large_image" }],
        ["meta", { name: "twitter:title", content: "Workbench" }],
        [
            "meta",
            {
                name: "twitter:description",
                content: "A personal CLI to bootstrap, evolve, and archive/restore projects",
            },
        ],
        [
            "meta",
            {
                name: "twitter:image",
                content: "https://markmals.github.io/workbench/workbench-hero.png",
            },
        ],
    ],

    themeConfig: {
        logo: "/workbench-icon.png",
        siteTitle: "Workbench",

        nav: [
            { text: "Guide", link: "/guide/getting-started" },
            { text: "Commands", link: "/commands/init" },
            {
                text: "v0.1.0",
                items: [
                    {
                        text: "Changelog",
                        link: "https://github.com/markmals/workbench/releases",
                    },
                ],
            },
        ],

        sidebar: {
            "/guide/": [
                {
                    text: "Introduction",
                    items: [
                        { text: "What is Workbench?", link: "/guide/what-is-workbench" },
                        { text: "Getting Started", link: "/guide/getting-started" },
                        { text: "Installation", link: "/guide/installation" },
                    ],
                },
                {
                    text: "Core Concepts",
                    items: [
                        { text: "Project Types", link: "/guide/project-types" },
                        { text: "Features", link: "/guide/features" },
                        { text: "Configuration", link: "/guide/configuration" },
                    ],
                },
                {
                    text: "Workflows",
                    items: [
                        { text: "Archiving Projects", link: "/guide/archiving" },
                        { text: "Restoring Projects", link: "/guide/restoring" },
                    ],
                },
            ],
            "/commands/": [
                {
                    text: "Commands",
                    items: [
                        { text: "wb init", link: "/commands/init" },
                        { text: "wb add", link: "/commands/add" },
                        { text: "wb rm", link: "/commands/rm" },
                        { text: "wb archive", link: "/commands/archive" },
                        { text: "wb restore", link: "/commands/restore" },
                        { text: "wb version", link: "/commands/version" },
                    ],
                },
            ],
        },

        socialLinks: [
            { icon: "github", link: "https://github.com/markmals/workbench" },
        ],

        search: {
            provider: "local",
        },
    },
});
