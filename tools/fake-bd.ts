import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

/** What one call to the fake `bd` prints and how it exits. `json` is printed to stdout. */
export interface FakeReply {
    json?: unknown;
    stdout?: string;
    stderr?: string;
    exit?: number;
    sleepMs?: number;
}

/**
 * Replies keyed by the subcommand they answer, such as `sync`, `dolt push`, or `update`. The
 * key whose words begin the call's non-flag arguments answers it, the longest key winning. A
 * list answers successive calls in order and then repeats its last reply.
 */
export type FakeScript = Record<string, FakeReply | FakeReply[]>;

export interface FakeCall {
    args: string[];
    actor: string | undefined;
}

// Runs as a standalone executable, so it is plain JavaScript and reaches Node's modules
// through `process.getBuiltinModule`, which works whether Node loads it as CommonJS or ESM.
const FAKE_BD = `
let fs = process.getBuiltinModule("node:fs");
let path = process.getBuiltinModule("node:path");
let directory = path.dirname(process.argv[1]);
let file = name => path.join(directory, name);
let args = process.argv.slice(2);
fs.appendFileSync(file("bd.calls.jsonl"), JSON.stringify({ args, actor: process.env.BEADS_ACTOR }) + "\\n");
let script = JSON.parse(fs.readFileSync(file("bd.script.json"), "utf8"));
let words = args.filter(argument => !argument.startsWith("-"));
let key = Object.keys(script)
    .filter(candidate => candidate.split(" ").every((word, index) => words[index] === word))
    .sort((left, right) => right.length - left.length)[0];
if (key === undefined) {
    process.stderr.write("fake bd: no reply scripted for: " + args.join(" ") + "\\n");
    process.exit(1);
}
let state = fs.existsSync(file("bd.state.json")) ? JSON.parse(fs.readFileSync(file("bd.state.json"), "utf8")) : {};
let replies = [].concat(script[key]);
let reply = replies[Math.min(state[key] ?? 0, replies.length - 1)];
state[key] = (state[key] ?? 0) + 1;
fs.writeFileSync(file("bd.state.json"), JSON.stringify(state));
if (reply.sleepMs) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, reply.sleepMs);
if (reply.json !== undefined) process.stdout.write(JSON.stringify(reply.json) + "\\n");
if (reply.stdout) process.stdout.write(reply.stdout);
if (reply.stderr) process.stderr.write(reply.stderr);
process.exit(reply.exit ?? 0);
`;

/**
 * Writes a scripted `bd` into `bin`, so a test can put it first on `PATH` and drive Beads
 * through any outcome without a real database or remote. `calls` reads back every invocation.
 */
export function installFakeBd(bin: string, script: FakeScript): { calls: () => FakeCall[] } {
    mkdirSync(bin, { recursive: true });
    let executable = path.join(bin, "bd");
    writeFileSync(executable, `#!${process.execPath}\n${FAKE_BD}`);
    chmodSync(executable, 0o755);
    writeFileSync(path.join(bin, "bd.script.json"), JSON.stringify(script));
    let log = path.join(bin, "bd.calls.jsonl");
    return {
        calls: () =>
            existsSync(log)
                ? readFileSync(log, "utf8")
                      .trim()
                      .split("\n")
                      .map(line => JSON.parse(line) as FakeCall)
                : [],
    };
}
