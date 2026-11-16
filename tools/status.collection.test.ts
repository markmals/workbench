import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

import { installFakeBd, type FakeScript } from "./fake-bd.ts";

let statusScript = fileURLToPath(new URL("./status.ts", import.meta.url));

// Git exports GIT_DIR, GIT_INDEX_FILE and GIT_WORK_TREE to anything a hook runs, and a
// child `git` inheriting those acts on the outer repository rather than on the fixture.
// `BEADS_*` would point Beads at the outer session's database. Every fixture runs without
// either, and with a scripted `bd`, so no test can reach a real Beads replica.
let fixtureEnv = Object.fromEntries(
    Object.entries(process.env).filter(
        ([name]) => !name.startsWith("GIT_") && !name.startsWith("BEADS_"),
    ),
);

const NO_BEADS_WORKSPACE: FakeScript = {
    where: {
        exit: 1,
        json: { error: "no_beads_directory", message: "No active beads workspace found." },
    },
};

function statusWithPullRequests(t: TestContext, pullRequests: object[]): string {
    let root = mkdtempSync(path.join(tmpdir(), "workbench-status-"));
    t.after(() => rmSync(root, { force: true, recursive: true }));
    execFileSync("git", ["init", "-q", "-b", "feature", root], { env: fixtureEnv });
    execFileSync(
        "git",
        [
            "-C",
            root,
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "--allow-empty",
            "-qm",
            "fixture",
        ],
        { env: fixtureEnv },
    );
    let bin = path.join(root, "bin");
    mkdirSync(bin);
    let gh = path.join(bin, "gh");
    writeFileSync(
        gh,
        [
            `#!${process.execPath}`,
            "let command = process.argv[2];",
            'if (command === "--version" || command === "auth") process.exit(0);',
            'if (command === "repo") console.log("main");',
            'else if (command === "pr" && process.argv[3] === "list") {',
            '  let state = process.argv.indexOf("--state");',
            '  let head = process.argv.indexOf("--head");',
            '  console.log(process.argv[state + 1] === "all" && process.argv[head + 1] === "feature" ? process.env.TEST_PULL_REQUESTS : "[]");',
            "}",
            "else process.exit(1);",
        ].join("\n"),
    );
    chmodSync(gh, 0o755);
    installFakeBd(bin, NO_BEADS_WORKSPACE);
    mkdirSync(path.join(root, "proposals"));
    writeFileSync(
        path.join(root, "proposals", "0001-feature.md"),
        "---\nid: proposal.0001\ntitle: Feature\nstatus: implemented\npull-request: 1\n---\n",
    );
    return execFileSync(process.execPath, [statusScript], {
        cwd: root,
        encoding: "utf8",
        env: {
            ...fixtureEnv,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
            TEST_PULL_REQUESTS: JSON.stringify(pullRequests),
        },
    });
}

function assertContains(report: string, expected: string): void {
    assert.ok(report.includes(expected), `expected report to contain ${expected}:\n${report}`);
}

test("finds a merged feature PR after it leaves the open PR list", t => {
    let report = statusWithPullRequests(t, [
        {
            number: 1,
            state: "MERGED",
            baseRefName: "main",
            isDraft: false,
            title: "Feature",
            url: "https://github.com/owner/repo/pull/1",
        },
    ]);

    assertContains(report, "PR #1 (merged)");
    assertContains(report, "Phase 5 — Implementation merged");
});

test("does not select an arbitrary PR when a branch name has been reused", t => {
    let report = statusWithPullRequests(t, [
        { number: 1, state: "MERGED", baseRefName: "main", isDraft: false, title: "Earlier" },
        { number: 2, state: "CLOSED", baseRefName: "main", isDraft: false, title: "Later" },
    ]);

    assertContains(report, "PR — unknown (Multiple pull requests found for branch feature");
    assertContains(report, "inspect their proposal links");
    assertContains(report, "Check gh access and the branch's pull request association");
    assert.ok(
        !report.includes("Implementation merged"),
        `expected report to omit "Implementation merged":\n${report}`,
    );
});

function statusInRepositoryWithoutRemote(
    t: TestContext,
    { beads = NO_BEADS_WORKSPACE, stayOnMain = false, json = false } = {},
): string {
    let root = mkdtempSync(path.join(tmpdir(), "workbench-status-local-"));
    t.after(() => rmSync(root, { force: true, recursive: true }));
    execFileSync("git", ["init", "-q", "-b", "main", root], { env: fixtureEnv });
    execFileSync(
        "git",
        [
            "-C",
            root,
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "--allow-empty",
            "-qm",
            "fixture",
        ],
        { env: fixtureEnv },
    );
    if (!stayOnMain)
        execFileSync("git", ["-C", root, "switch", "-q", "-c", "feature/x"], { env: fixtureEnv });

    // No origin and no usable gh, so refs/heads/main is the only evidence of which
    // branch is the default one.
    let bin = path.join(root, "bin");
    mkdirSync(bin);
    let gh = path.join(bin, "gh");
    writeFileSync(gh, `#!${process.execPath}\nprocess.exit(1);\n`);
    chmodSync(gh, 0o755);
    installFakeBd(bin, beads);

    return execFileSync(process.execPath, [statusScript, ...(json ? ["--json"] : [])], {
        cwd: root,
        encoding: "utf8",
        env: {
            ...fixtureEnv,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        },
    });
}

test("recognizes a local default branch when there is no remote and no gh", t => {
    let report = statusInRepositoryWithoutRemote(t);

    assertContains(report, "branch feature/x");
    assertContains(report, "PR — unknown (gh is unavailable)");
    assertContains(report, "Phase 1 — Preparation");
    assert.ok(
        !report.includes("Branch role unknown"),
        `expected report to omit "Branch role unknown":\n${report}`,
    );
});

const AVAILABLE_BEADS: FakeScript = {
    where: { json: { database_path: "/project/.beads/embeddeddolt", path: "/project/.beads" } },
    ready: { json: [{ id: "wb-1", title: "Fix the parser", priority: 1, status: "open" }] },
    list: {
        json: [{ id: "wb-2", title: "Port status", status: "in_progress", assignee: "mark@beta" }],
    },
    prime: { stdout: "## Persistent Memories (1)\n\n### fact\nA durable fact.\n" },
};

test("reports Beads after the phase: sync outcome, ready work, items in progress, memories", t => {
    let report = statusInRepositoryWithoutRemote(t, {
        stayOnMain: true,
        beads: { ...AVAILABLE_BEADS, sync: { json: { status: "ok", pulled: true, pushed: true } } },
    });

    assert.ok(
        report.indexOf("Phase 1 — Preparation") < report.indexOf("Beads"),
        `expected the Beads section after the phase report:\n${report}`,
    );
    for (let expected of [
        "synced",
        "wb-1",
        "Fix the parser",
        "wb-2",
        "mark@beta",
        "A durable fact.",
    ])
        assertContains(report, expected);
});

test("adds a beads object to the JSON status", t => {
    let status = JSON.parse(
        statusInRepositoryWithoutRemote(t, {
            stayOnMain: true,
            json: true,
            beads: { ...AVAILABLE_BEADS, sync: { json: { status: "ok" } } },
        }),
    );

    assert.equal(status.phase, "1");
    assert.equal(status.beads.sync.outcome, "synced");
    assert.equal(typeof status.beads.sync.detail, "string");
    assert.deepEqual(status.beads.ready, [{ id: "wb-1", title: "Fix the parser", priority: 1 }]);
    assert.deepEqual(status.beads.in_progress, [
        { id: "wb-2", title: "Port status", assignee: "mark@beta" },
    ]);
    assert.equal(status.beads.memories, "## Persistent Memories (1)\n\n### fact\nA durable fact.");
});

test("a failed Beads sync is reported with its reason and does not fail status", t => {
    let report = statusInRepositoryWithoutRemote(t, {
        stayOnMain: true,
        beads: {
            ...AVAILABLE_BEADS,
            sync: { exit: 1, json: { error: "sync failed: could not resolve host github.com" } },
            "dolt remote list": { exit: 1, stderr: "Error: no remotes\n" },
        },
    });

    assertContains(report, "failed");
    assertContains(report, "could not resolve host github.com");
    assertContains(report, "this machine");
    assertContains(report, "Fix the parser");
});

test("an unavailable Beads workspace replaces the section with a pointer to setup", t => {
    let report = statusInRepositoryWithoutRemote(t, { stayOnMain: true });
    let status = JSON.parse(statusInRepositoryWithoutRemote(t, { stayOnMain: true, json: true }));

    assertContains(report, "unavailable");
    assertContains(report, "No active beads workspace found.");
    assertContains(report, "mise run setup");
    assert.deepEqual(Object.keys(status.beads), ["unavailable"]);
    assert.match(status.beads.unavailable, /No active beads workspace found/);
});
