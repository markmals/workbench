import { h } from "vue";
import type { Theme } from "vitepress";
import { themeContextKey, VoidZeroTheme } from "@voidzero-dev/vitepress-theme";
import footerBg from "@voidzero-dev/vitepress-theme/src/assets/vitest/footer-background.jpg";
import monoIcon from "@voidzero-dev/vitepress-theme/src/assets/icons/vitest-mono.svg";
import logo from "../../public/workbench-icon.png";
import "./custom.css";
import "virtual:group-icons.css";

// SpecHeader (frontmatter banner) deferred — VitePress 2 alpha doesn't expose
// frontmatter through useData() the way 1.x did when injecting via doc-before.
// Revisit when v2 stabilizes.

export default {
    extends: VoidZeroTheme,
    enhanceApp(ctx) {
        ctx.app.provide(themeContextKey, {
            logoDark: logo,
            logoLight: logo,
            logoAlt: "Spec-Driven Development",
            footerBg,
            monoIcon,
        });
    },
} satisfies Theme;
