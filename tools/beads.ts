import { bd, bdError, parseJson, type CommandRun } from "./bd.ts";
import { foreignRemote, remoteLacksBeadsHistory } from "./beads-remote.ts";

// Beads across machines: the one sync every sync point runs, and whether Beads is usable here.
// `mise run beads:sync` (through beads-cli.ts), the claim and close tasks (beads-claims.ts), and
// `mise run status` (beads-report.ts) all sync through this module. The outcomes are a contract:
// the tracking-work skill tells agents what each one means, and guides/record-validation.md
// documents the ones `mise run status` reports.

export type SyncOutcome = "synced" | "failed" | "conflict" | "blocked";

export interface SyncResult {
    outcome: SyncOutcome;
    detail: string;
}

/** How long one sync may take before its outcome is `failed`. */
export const SYNC_TIME_LIMIT_MS = 20_000;

function timedOut(timeoutMs: number): SyncResult {
    return {
        outcome: "failed",
        detail: `The sync did not finish within ${timeoutMs / 1000} seconds; nothing is known to have been pushed.`,
    };
}

/**
 * Runs one sync with the project's Beads remote: `bd sync`, or `bd dolt push` when the
 * remote has no Beads history yet, so the project's first sync publishes it. A replica whose
 * remote is not the checkout's own `origin` is never synced. Never throws; every way the
 * sync can end is one of the outcomes.
 */
export function syncBeads(root: string, timeoutMs = SYNC_TIME_LIMIT_MS): SyncResult {
    let deadline = Date.now() + timeoutMs;
    let foreign = foreignRemote(root, deadline);
    if (foreign) return { outcome: "failed", detail: foreign };
    let sync = bd(["sync", "--json"], root, deadline);
    if (sync.timedOut) return timedOut(timeoutMs);
    if (sync.status === 0)
        return { outcome: "synced", detail: "Pulled from and pushed to the remote." };
    if (sync.status === 2) return conflict(root, sync, deadline);
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

function conflict(root: string, sync: CommandRun, deadline: number): SyncResult {
    let tables = (parseJson(sync.stdout) as { conflicts?: string[] } | undefined)?.conflicts;
    let lines = [
        `Beads halted on a conflict in ${tables?.length ? tables.join(", ") : "the database"}; nothing was pushed.`,
    ];
    // Embedded storage aborts the halted merge, so the conflicted rows are gone; what differs
    // from the remote's last fetched state is the closest Beads can show.
    let diff = bd(["diff", "main", "origin/main", "--json"], root, deadline);
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
