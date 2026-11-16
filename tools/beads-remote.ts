import { bd, parseJson, runCommand } from "./bd.ts";

// Which repository this replica syncs with, whether that is this project's own, and whether
// it holds Beads history yet.

/** The URL of the replica's `origin` Dolt remote, the one `bd sync` uses, or null. */
export function doltRemoteUrl(root: string, deadline: number): string | null {
    let remotes = parseJson(bd(["dolt", "remote", "list", "--json"], root, deadline).stdout);
    if (!Array.isArray(remotes)) return null;
    let remote = (remotes as { name?: string; url?: string }[]).find(
        ({ name }) => name === "origin",
    );
    return remote?.url ?? null;
}

/**
 * One spelling for every way of writing a repository's location: SSH, HTTPS with or without
 * credentials or `.git`, Dolt's `git+` prefix, and local paths. Hosted paths compare without
 * case, as GitHub's do.
 */
export function repositoryIdentity(url: string): string {
    let bare = url
        .trim()
        .replace(/^git\+/, "")
        .replace(/\/+$/, "")
        .replace(/\.git$/, "");
    let scpLike = bare.match(/^[^/@:]+@([^:/]+):(.+)$/);
    if (scpLike) return `${scpLike[1]}/${scpLike[2]}`.toLowerCase();
    if (!URL.canParse(bare)) return bare;
    let parsed = new URL(bare);
    if (parsed.protocol === "file:") return decodeURIComponent(parsed.pathname);
    return `${parsed.hostname}${parsed.pathname}`.toLowerCase();
}

/**
 * A description of the mismatch when this replica's Beads remote is not the checkout's own
 * `origin`, as when a project created from the Workbench template still carries Workbench's
 * `.beads/`. Null when they match or either one is unknown.
 */
export function foreignRemote(root: string, deadline: number): string | null {
    let origin = runCommand("git", ["remote", "get-url", "origin"], root);
    if (origin.status !== 0) return null;
    let beadsRemote = doltRemoteUrl(root, deadline);
    if (beadsRemote === null) return null;
    let originUrl = origin.stdout.trim();
    if (repositoryIdentity(beadsRemote) === repositoryIdentity(originUrl)) return null;
    return `This replica's Beads remote is ${beadsRemote}, not this project's origin (${originUrl}); nothing was pulled or pushed. The project's .beads/ belongs to another repository.`;
}

/** True only when the remote is reachable and positively has no `refs/dolt/data`. */
export function remoteLacksBeadsHistory(root: string, deadline: number): boolean {
    let url = doltRemoteUrl(root, deadline);
    if (url === null) return false;
    let listing = runCommand(
        "git",
        ["ls-remote", url.replace(/^git\+/, ""), "refs/dolt/data"],
        root,
        {
            timeoutMs: Math.max(1, deadline - Date.now()),
        },
    );
    return listing.status === 0 && listing.stdout.trim() === "";
}
