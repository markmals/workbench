import { hostname, userInfo } from "node:os";

import { bd, bdError, parseJson } from "./bd.ts";
import { foreignRemote } from "./beads-remote.ts";
import { SYNC_TIME_LIMIT_MS, syncBeads } from "./beads.ts";

// Claiming and closing items as this machine's claimant. `bd update --claim` and `bd close`
// both act as `BEADS_ACTOR`, and `bd close` refuses an item claimed by another actor, so both
// run here with the claimant set rather than as the Git user name a plain `bd` uses. The
// tracking-work skill documents the outcomes.

export type ClaimOutcome = "claimed" | "claimed-unpublished" | "held" | "lost";

export interface ClaimResult {
    outcome: ClaimOutcome;
    detail: string;
    /** Who holds the item after the claim: this machine's claimant unless another won. */
    holder: string | null;
}

export type CloseOutcome = "closed" | "closed-unpublished";

export interface CloseResult {
    outcome: CloseOutcome;
    detail: string;
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

function refuseForeignRemote(root: string, timeoutMs: number): void {
    let foreign = foreignRemote(root, Date.now() + timeoutMs);
    if (foreign) throw new Error(foreign);
}

/**
 * Claims an item so other machines see it: sync, claim as this machine's claimant, sync
 * again, then re-read who holds it. Throws, changing nothing, when the replica's Beads remote
 * is not this project's own, and throws when Beads rejects the claim for any reason other
 * than another claimant holding the item, such as an unknown id.
 */
export function claimBeadsItem(
    root: string,
    id: string,
    timeoutMs = SYNC_TIME_LIMIT_MS,
): ClaimResult {
    refuseForeignRemote(root, timeoutMs);
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

/**
 * Closes an item as this machine's claimant, then syncs so other machines see it. Throws,
 * changing nothing, when the replica's Beads remote is not this project's own, and throws with
 * Beads' reason when it refuses the close, such as for an item another claimant holds.
 */
export function closeBeadsItem(
    root: string,
    id: string,
    reason: string,
    timeoutMs = SYNC_TIME_LIMIT_MS,
): CloseResult {
    refuseForeignRemote(root, timeoutMs);
    let actor = claimant();
    let close = bd(["close", id, `--reason=${reason}`, "--json"], root, undefined, {
        ...process.env,
        BEADS_ACTOR: actor,
    });
    if (close.status !== 0) throw new Error(bdError(close, "close"));
    let sync = syncBeads(root, timeoutMs);
    if (sync.outcome !== "synced")
        return {
            outcome: "closed-unpublished",
            detail: `${id} is closed on this machine only; the sync ended ${sync.outcome}: ${sync.detail} Run \`mise run beads:sync\` until it succeeds.`,
        };
    return {
        outcome: "closed",
        detail: `${id} is closed as ${actor}, and other machines can see it.`,
    };
}
