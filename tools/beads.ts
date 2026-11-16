import { spawnSync } from "node:child_process";
import { hostname, userInfo } from "node:os";

// Beads across machines: the one sync every sync point runs, and the claim that checks it
// survived. `mise run beads:sync`, `mise run beads:claim -- <id>`, and `mise run status`
// (through beads-report.ts) all run this module. The outcomes are a contract: the tracking-work
// skill tells agents what each one means, and guides/record-validation.md documents the ones
// `mise run status` reports.

export type SyncOutcome = "synced" | "failed" | "conflict" | "blocked";

export interface SyncResult {
    outcome: SyncOutcome;
    detail: string;
}

export type ClaimOutcome = "claimed" | "claimed-unpublished" | "held" | "lost";

export interface ClaimResult {
    outcome: ClaimOutcome;
    detail: string;
    /** Who holds the item after the claim: this machine's claimant unless another won. */
    holder: string | null;
}

/** How long one sync may take before its outcome is `failed`. */
export const SYNC_TIME_LIMIT_MS = 20_000;

interface CommandRun {
    status: number | null;
    stdout: string;
    stderr: string;
    missing: boolean;
    timedOut: boolean;
}

export function runCommand(
    command: string,
    arguments_: string[],
    root: string,
    { timeoutMs, env }: { timeoutMs?: number; env?: NodeJS.ProcessEnv } = {},
): CommandRun {
    let result = spawnSync(command, arguments_, {
        cwd: root,
        encoding: "utf8",
        env: env ?? process.env,
        stdio: ["ignore", "pipe", "pipe"],
        timeout: timeoutMs,
    });
    let code = (result.error as NodeJS.ErrnoException | undefined)?.code;
    return {
        status: result.status,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
        missing: code === "ENOENT",
        timedOut: code === "ETIMEDOUT",
    };
}

// bd prints machine-readable output with --json, but an older or failing bd may not.
export function parseJson(text: string): unknown {
    try {
        return JSON.parse(text);
    } catch {
        return undefined;
    }
}

/** The most specific reason bd gave for a failed command. */
export function bdError(run: CommandRun, command: string): string {
    let output = parseJson(run.stdout) as
        | { failed?: { error?: string }[]; message?: string; error?: string }
        | undefined;
    let reason = output?.failed?.[0]?.error ?? output?.message ?? output?.error;
    if (reason) return reason;
    let errorLines = run.stderr.split("\n").filter(line => line.startsWith("Error"));
    if (errorLines.length > 0) return errorLines.join(" ");
    return run.stderr.trim() || `bd ${command} exited with status ${run.status}`;
}

function bd(arguments_: string[], root: string, deadline?: number, env?: NodeJS.ProcessEnv) {
    let timeoutMs = deadline === undefined ? undefined : Math.max(1, deadline - Date.now());
    return runCommand("bd", arguments_, root, { timeoutMs, env });
}

function timedOut(timeoutMs: number): SyncResult {
    return {
        outcome: "failed",
        detail: `The sync did not finish within ${timeoutMs / 1000} seconds; nothing is known to have been pushed.`,
    };
}

/**
 * Runs one sync with the project's Beads remote: `bd sync`, or `bd dolt push` when the
 * remote has no Beads history yet, so the project's first sync publishes it. Never throws;
 * every way the sync can end is one of the outcomes.
 */
export function syncBeads(root: string, timeoutMs = SYNC_TIME_LIMIT_MS): SyncResult {
    let deadline = Date.now() + timeoutMs;
    let sync = bd(["sync", "--json"], root, deadline);
    if (sync.timedOut) return timedOut(timeoutMs);
    if (sync.status === 0)
        return { outcome: "synced", detail: "Pulled from and pushed to the remote." };
    if (sync.status === 2) return conflict(root, sync);
    if (sync.status === 4)
        return { outcome: "blocked", detail: `${bdError(sync, "sync")} Nothing was pushed.` };
    if (sync.status === 1 && remoteLacksBeadsHistory(root, deadline)) {
        let push = bd(["dolt", "push"], root, deadline);
        if (push.timedOut) return timedOut(timeoutMs);
        if (push.status === 0)
            return {
                outcome: "synced",
                detail: "Published this replica to a remote that had no Beads history.",
            };
        return { outcome: "failed", detail: bdError(push, "dolt push") };
    }
    return { outcome: "failed", detail: bdError(sync, "sync") };
}

function conflict(root: string, sync: CommandRun): SyncResult {
    let tables = (parseJson(sync.stdout) as { conflicts?: string[] } | undefined)?.conflicts;
    let lines = [
        `Beads halted on a conflict in ${tables?.length ? tables.join(", ") : "the database"}; nothing was pushed.`,
    ];
    // Embedded storage aborts the halted merge, so the conflicted rows are gone; what differs
    // from the remote's last fetched state is the closest Beads can show.
    let diff = bd(["diff", "main", "origin/main", "--json"], root);
    let items = parseJson(diff.stdout) as DiffEntry[] | undefined;
    if (diff.status === 0 && Array.isArray(items) && items.length > 0)
        lines.push(`Differs from the remote: ${items.map(describeDiffEntry).join(", ")}`);
    else if (diff.status !== 0)
        lines.push("Run `bd diff main origin/main` to see what differs from the remote.");
    return { outcome: "conflict", detail: lines.join("\n") };
}

interface DiffEntry {
    IssueID: string;
    OldValue?: { title?: string } | null;
    NewValue?: { title?: string } | null;
}

function describeDiffEntry({ IssueID, OldValue, NewValue }: DiffEntry): string {
    let title = NewValue?.title ?? OldValue?.title;
    return title ? `${IssueID} (${title})` : IssueID;
}

/** True only when the remote is reachable and positively has no `refs/dolt/data`. */
function remoteLacksBeadsHistory(root: string, deadline: number): boolean {
    let remotes = parseJson(bd(["dolt", "remote", "list", "--json"], root, deadline).stdout);
    if (!Array.isArray(remotes)) return false;
    let remote = (remotes as { name?: string; url?: string }[]).find(
        ({ name }) => name === "origin",
    );
    if (!remote?.url) return false;
    let timeoutMs = Math.max(1, deadline - Date.now());
    let listing = runCommand(
        "git",
        ["ls-remote", remote.url.replace(/^git\+/, ""), "refs/dolt/data"],
        root,
        {
            timeoutMs,
        },
    );
    return listing.status === 0 && listing.stdout.trim() === "";
}

/** The claimant this machine claims as: `BEADS_ACTOR`, or `<login user>@<short host name>`. */
export function claimant(): string {
    return process.env.BEADS_ACTOR || `${userInfo().username}@${hostname().split(".")[0]}`;
}

interface Item {
    status?: string;
    assignee?: string;
}

function readItem(root: string, id: string): Item | null {
    let show = bd(["show", id, "--json"], root);
    let items = parseJson(show.stdout);
    if (show.status !== 0 || !Array.isArray(items)) return null;
    return (items[0] as Item | undefined) ?? null;
}

/**
 * Claims an item so other machines see it: sync, claim as this machine's claimant, sync
 * again, then re-read who holds it. Throws when Beads rejects the claim for any reason other
 * than another claimant holding the item, such as an unknown id.
 */
export function claimBeadsItem(
    root: string,
    id: string,
    timeoutMs = SYNC_TIME_LIMIT_MS,
): ClaimResult {
    let actor = claimant();
    syncBeads(root, timeoutMs);
    let claim = bd(["update", id, "--claim", "--json"], root, undefined, {
        ...process.env,
        BEADS_ACTOR: actor,
    });
    if (claim.status !== 0) {
        let item = readItem(root, id);
        if (item?.status === "in_progress" && item.assignee && item.assignee !== actor)
            return {
                outcome: "held",
                holder: item.assignee,
                detail: `${id} is held by ${item.assignee}. It is not available.`,
            };
        throw new Error(bdError(claim, "update --claim"));
    }
    let second = syncBeads(root, timeoutMs);
    let item = readItem(root, id);
    if (item === null) throw new Error(`Could not read ${id} back after claiming it.`);
    if (item.status !== "in_progress" || item.assignee !== actor) {
        let holder = item.status === "in_progress" ? (item.assignee ?? null) : null;
        let now = holder
            ? `is now held by ${holder}`
            : `is no longer claimed (status ${item.status})`;
        return {
            outcome: "lost",
            holder,
            detail: `${id} ${now}: a change from another machine won. Do not start the work.`,
        };
    }
    if (second.outcome !== "synced")
        return {
            outcome: "claimed-unpublished",
            holder: actor,
            detail: `${id} is claimed as ${actor} on this machine only; the sync ended ${second.outcome}: ${second.detail} Run \`mise run beads:sync\` until it succeeds.`,
        };
    return {
        outcome: "claimed",
        holder: actor,
        detail: `${id} is claimed as ${actor}, and other machines can see the claim.`,
    };
}

/** Why Beads cannot be used here, or null when it can: bd, a workspace, and its database. */
export function beadsUnavailable(root: string): string | null {
    let where = bd(["where", "--json"], root);
    if (where.missing) return "bd is not installed";
    if (where.status !== 0) return bdError(where, "where");
    let workspace = parseJson(where.stdout) as
        | { database_path?: string; path?: string }
        | undefined;
    if (!workspace?.database_path)
        return `the Beads database${workspace?.path ? ` in ${workspace.path}` : ""} is not initialized`;
    return null;
}

/** One outcome line, then any further detail lines indented beneath it. */
export function formatOutcome(outcome: string, detail: string): string {
    let [first, ...rest] = detail.split("\n");
    return [`Beads: ${outcome}. ${first}`, ...rest.map(line => `  ${line}`)].join("\n");
}

export function formatUnavailable(reason: string): string {
    return formatOutcome("unavailable", `${reason}. Run \`mise run setup\`.`);
}

function main(): number {
    let [command, id] = process.argv.slice(2);
    if (command !== "sync" && !(command === "claim" && id)) {
        console.error("Usage: node tools/beads.ts sync | node tools/beads.ts claim <id>");
        return 2;
    }
    let root = process.cwd();
    let unavailable = beadsUnavailable(root);
    if (unavailable) {
        console.log(formatUnavailable(unavailable));
        return 1;
    }
    if (command === "sync") {
        let result = syncBeads(root);
        console.log(formatOutcome(result.outcome, result.detail));
        return result.outcome === "synced" ? 0 : 1;
    }
    try {
        let result = claimBeadsItem(root, id!);
        console.log(formatOutcome(result.outcome, result.detail));
        return result.outcome === "claimed" || result.outcome === "claimed-unpublished" ? 0 : 1;
    } catch (error) {
        console.log(formatOutcome("claim failed", (error as Error).message));
        return 1;
    }
}

if (import.meta.main) process.exitCode = main();
