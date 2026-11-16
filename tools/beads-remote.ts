import path from "node:path";

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
 * credentials, Dolt's `git+` prefix, and local paths, which resolve from `root` when relative.
 * A hosted repository is the same with or without `.git` and in any case, as GitHub's are; a
 * local path is compared exactly, because `repo` and `repo.git` can be two repositories.
 */
export function repositoryIdentity(url: string, root: string): string {
    let bare = url
        .trim()
        .replace(/^git\+/, "")
        .replace(/\/+$/, "");
    let hosted = (host: string, repositoryPath: string) =>
        `${host}/${repositoryPath.replace(/^\/+/, "").replace(/\.git$/, "")}`.toLowerCase();
    let scpLike = bare.match(/^[^/@:]+@([^:/]+):(.+)$/);
    if (scpLike) return hosted(scpLike[1]!, scpLike[2]!);
    if (!URL.canParse(bare)) return path.resolve(root, bare);
    let parsed = new URL(bare);
    if (parsed.protocol === "file:") return path.resolve(decodeURIComponent(parsed.pathname));
    return hosted(parsed.hostname, parsed.pathname);
}

/**
 * Why this replica must not sync, when its Beads remote is not the checkout's own `origin`,
 * as when a project created from the Workbench template still carries Workbench's `.beads/`,
 * or when there is no `origin` to compare it with. Null when the replica has no Beads remote
 * yet (`bd sync` then adopts `origin`) or when the two are the same repository.
 */
export function foreignRemote(root: string, deadline: number): string | null {
    let beadsRemote = doltRemoteUrl(root, deadline);
    if (beadsRemote === null) return null;
    let origin = runCommand("git", ["remote", "get-url", "origin"], root);
    if (origin.status !== 0)
        return `This replica's Beads remote is ${beadsRemote}, but this checkout has no origin to compare it with; nothing was pulled or pushed.`;
    let originUrl = origin.stdout.trim();
    if (repositoryIdentity(beadsRemote, root) === repositoryIdentity(originUrl, root)) return null;
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
