import {
    bdError,
    beadsUnavailable,
    formatOutcome,
    formatUnavailable,
    parseJson,
    runCommand,
    syncBeads,
    type SyncResult,
} from "./beads.ts";

// The Beads section of `mise run status`. Its contract is "Beads in the status report" in
// guides/record-validation.md.

export interface ReadyItem {
    id: string;
    title: string;
    priority: number;
}

export interface InProgressItem {
    id: string;
    title: string;
    assignee: string | null;
}

export type BeadsReport =
    | { sync: SyncResult; ready: ReadyItem[]; in_progress: InProgressItem[]; memories: string }
    | { unavailable: string };

// `bd prime` addresses session-start hook hosts in its first line; it is not a memory.
const PRIME_HOST_NOTICE = /^\[bd prime\][^\n]*\n+/;

/**
 * Syncs, then reads this machine's replica: ready work, items in progress, and memories. A
 * failed read makes the whole report unavailable, with bd's reason, rather than partial.
 */
export function collectBeadsReport(root: string): BeadsReport {
    let unavailable = beadsUnavailable(root);
    if (unavailable) return { unavailable };
    let sync = syncBeads(root);
    let ready = runCommand("bd", ["ready", "--json"], root);
    if (ready.status !== 0) return { unavailable: bdError(ready, "ready") };
    let inProgress = runCommand("bd", ["list", "--status", "in_progress", "--json"], root);
    if (inProgress.status !== 0) return { unavailable: bdError(inProgress, "list") };
    let prime = runCommand("bd", ["prime", "--memories-only"], root);
    if (prime.status !== 0) return { unavailable: bdError(prime, "prime") };
    let readyItems = (parseJson(ready.stdout) as ReadyItem[] | undefined) ?? [];
    let inProgressItems = (parseJson(inProgress.stdout) as InProgressItem[] | undefined) ?? [];
    return {
        sync,
        ready: readyItems.map(({ id, title, priority }) => ({ id, title, priority })),
        in_progress: inProgressItems.map(({ id, title, assignee }) => ({
            id,
            title,
            assignee: assignee ?? null,
        })),
        memories: prime.stdout.replace(PRIME_HOST_NOTICE, "").trim(),
    };
}

export function formatBeadsReport(report: BeadsReport): string {
    if ("unavailable" in report) return formatUnavailable(report.unavailable);
    let { sync } = report;
    let detail =
        sync.outcome === "synced"
            ? sync.detail
            : `${sync.detail}\nWhat follows is this machine's replica, which may be stale.`;
    return [
        formatOutcome(sync.outcome, detail),
        "",
        report.ready.length > 0 ? "Ready:" : "Ready: none",
        ...report.ready.map(({ id, title, priority }) => `  ${id}  P${priority}  ${title}`),
        report.in_progress.length > 0 ? "In progress:" : "In progress: none",
        ...report.in_progress.map(
            ({ id, title, assignee }) => `  ${id}  ${assignee ?? "unassigned"}  ${title}`,
        ),
        "",
        report.memories,
    ].join("\n");
}
