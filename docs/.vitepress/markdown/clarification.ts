import type MarkdownIt from "markdown-it";

/**
 * Styles `[NEEDS CLARIFICATION: <question>]` markers inline in draft proposals.
 */
export function clarificationMarkerPlugin(md: MarkdownIt): void {
    const pattern = /\[NEEDS CLARIFICATION:\s*([^\]]+)\]/g;

    md.core.ruler.after("inline", "clarification-marker", (state) => {
        for (const token of state.tokens) {
            if (token.type !== "inline" || !token.children) continue;

            for (const child of token.children) {
                if (child.type !== "text" || !pattern.test(child.content)) continue;

                const text = child.content;
                let cursor = 0;
                let html = "";
                let match: RegExpExecArray | null;

                pattern.lastIndex = 0;
                while ((match = pattern.exec(text))) {
                    html += escapeHtml(text.slice(cursor, match.index));
                    html += clarificationHtml(match[1].trim());
                    cursor = match.index + match[0].length;
                }

                child.type = "html_inline";
                child.content = html + escapeHtml(text.slice(cursor));
            }
        }
    });
}

function escapeHtml(value: string): string {
    return value
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

function clarificationHtml(question: string): string {
    return `<span class="needs-clarification" title="Resolve with the human before acceptance"><span class="needs-clarification__label">NEEDS CLARIFICATION</span><span class="needs-clarification__question">${escapeHtml(question)}</span></span>`;
}
