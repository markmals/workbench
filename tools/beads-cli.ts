import { claimBeadsItem, closeBeadsItem } from "./beads-claims.ts";
import { beadsUnavailable, formatOutcome, formatUnavailable, syncBeads } from "./beads.ts";

// The entry point for `mise run beads:sync`, `beads:claim -- <id>`, and
// `beads:close -- <id> "<reason>"`. Each prints `Beads: <outcome>. <detail>`; the exit status
// is zero for `synced`, `claimed`, `claimed-unpublished`, `closed`, and `closed-unpublished`.

const USAGE = [
    "Usage: node tools/beads-cli.ts sync",
    "       node tools/beads-cli.ts claim <id>",
    '       node tools/beads-cli.ts close <id> ["<reason>"]',
].join("\n");

function run(command: string, id: string | undefined, reason: string | undefined): number {
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
        let result =
            command === "claim" ? claimBeadsItem(root, id!) : closeBeadsItem(root, id!, reason);
        console.log(formatOutcome(result.outcome, result.detail));
        return result.outcome === "held" || result.outcome === "lost" ? 1 : 0;
    } catch (error) {
        console.log(formatOutcome(`${command} failed`, (error as Error).message));
        return 1;
    }
}

function main(): number {
    let [command, id, reason] = process.argv.slice(2);
    let valid = command === "sync" || ((command === "claim" || command === "close") && id);
    if (!valid) {
        console.error(USAGE);
        return 2;
    }
    return run(command!, id, reason);
}

process.exitCode = main();
