import type MarkdownIt from "markdown-it";

/**
 * Markdown-it plugin that styles `[NEEDS CLARIFICATION: <question>]` markers
 * inline so they stand out in rendered specs. The marker is replaced with a
 * `<span class="needs-clarification">` element preserving the original text.
 */
export function clarificationMarkerPlugin(md: MarkdownIt): void {
    const pattern = /\[NEEDS CLARIFICATION:\s*([^\]]+)\]/g;

    md.core.ruler.after("inline", "clarification-marker", (state) => {
        for (const token of state.tokens) {
            if (token.type !== "inline" || !token.children) continue;
            for (const child of token.children) {
                if (child.type !== "text") continue;
                if (!pattern.test(child.content)) continue;
                pattern.lastIndex = 0;
                child.type = "html_inline";
                child.content = child.content.replace(
                    pattern,
                    (_match, question: string) =>
                        `<span class="needs-clarification" title="Resolve with /sdd-clarify"><span class="needs-clarification__label">NEEDS CLARIFICATION</span><span class="needs-clarification__question">${escapeHtml(
                            question.trim(),
                        )}</span></span>`,
                );
            }
        }
    });
}

function escapeHtml(s: string): string {
    return s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}
