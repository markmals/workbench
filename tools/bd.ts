import { spawnSync } from "node:child_process";

// Running `bd` and `git` as child processes and reading what they report. Every Beads tool in
// this directory goes through these, so a missing binary or a timeout is detected one way.

export interface CommandRun {
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

/** Runs `bd`, given at most until `deadline` (epoch milliseconds) when one is set. */
export function bd(
    arguments_: string[],
    root: string,
    deadline?: number,
    env?: NodeJS.ProcessEnv,
): CommandRun {
    let timeoutMs = deadline === undefined ? undefined : Math.max(1, deadline - Date.now());
    return runCommand("bd", arguments_, root, { timeoutMs, env });
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
