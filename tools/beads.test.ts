import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { hostname, tmpdir, userInfo } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

import { claimBeadsItem, closeBeadsItem } from "./beads-claims.ts";
import { collectBeadsReport } from "./beads-report.ts";
import { syncBeads } from "./beads.ts";
import { installFakeBd, type FakeReply, type FakeScript } from "./fake-bd.ts";

let beadsCli = fileURLToPath(new URL("./beads-cli.ts", import.meta.url));

// Every test drives a scripted `bd`, never the real one, so no test can reach a real
// Beads database or remote: `BEADS_*` from the outer session is dropped along with git's
// hook variables, which would point a child `git` at the outer repository.
let cleanEnv = Object.fromEntries(
    Object.entries(process.env).filter(
        ([name]) => !name.startsWith("GIT_") && !name.startsWith("BEADS_"),
    ),
);

const AVAILABLE: FakeReply = {
    json: { database_path: "/project/.beads/embeddeddolt", path: "/project/.beads", prefix: "wb" },
};
const SYNCED: FakeReply = { json: { status: "ok", pulled: true, pushed: true, attempts: 1 } };

function useEnvironment(t: TestContext, overrides: Record<string, string | undefined>): void {
    let saved = process.env;
    process.env = { ...cleanEnv };
    for (let [name, value] of Object.entries(overrides)) {
        if (value === undefined) delete process.env[name];
        else process.env[name] = value;
    }
    t.after(() => {
        process.env = saved;
    });
}

/** A project directory whose `bd` is the fake, first on `PATH`. A null actor unsets `BEADS_ACTOR`. */
function project(t: TestContext, script: FakeScript, actor: string | null = "mark@alpha") {
    let root = mkdtempSync(path.join(tmpdir(), "workbench-beads-"));
    t.after(() => rmSync(root, { force: true, recursive: true }));
    let bin = path.join(root, "bin");
    let fake = installFakeBd(bin, script);
    useEnvironment(t, {
        PATH: `${bin}${path.delimiter}${cleanEnv.PATH ?? ""}`,
        BEADS_ACTOR: actor ?? undefined,
    });
    return { root, bin, calls: () => fake.calls().map(call => call.args.join(" ")), fake };
}

/** A bare Git repository standing in for the project's GitHub remote. */
function remote(t: TestContext, { withBeadsHistory }: { withBeadsHistory: boolean }): string {
    let directory = mkdtempSync(path.join(tmpdir(), "workbench-beads-remote-"));
    t.after(() => rmSync(directory, { force: true, recursive: true }));
    let git = (...arguments_: string[]) =>
        execFileSync("git", ["-C", directory, ...arguments_], {
            encoding: "utf8",
            env: cleanEnv,
        }).trim();
    git("init", "-q", "--bare");
    if (withBeadsHistory) {
        let tree = execFileSync("git", ["-C", directory, "mktree"], {
            encoding: "utf8",
            env: cleanEnv,
            input: "",
        }).trim();
        let commit = git(
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit-tree",
            tree,
            "-m",
            "dolt data",
        );
        git("update-ref", "refs/dolt/data", commit);
    }
    return `git+file://${directory}`;
}

function remotes(url: string): FakeReply {
    return { json: [{ name: "origin", url, sql_url: url, status: "ok" }] };
}

test("a sync that pulls and pushes reports synced", t => {
    let { root } = project(t, { sync: SYNCED });

    assert.equal(syncBeads(root).outcome, "synced");
});

test("a sync Beads cannot complete reports failed with Beads' reason", t => {
    let { root, calls } = projectWithRemote(t, true, {
        sync: { exit: 1, json: { error: "sync failed: push: authentication required" } },
    });

    let result = syncBeads(root);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /authentication required/);
    assert.ok(!calls().includes("dolt push"), "a remote with Beads history is never pushed over");
});

test("the first sync to a remote without Beads history publishes this replica", t => {
    let { root, calls } = projectWithRemote(t, false, {
        sync: { exit: 1, json: { error: "fetch failed: no branches found in remote 'origin'" } },
        "dolt push": { stdout: "Push complete.\n" },
    });

    assert.equal(syncBeads(root).outcome, "synced");
    assert.ok(calls().includes("dolt push"), `expected a first publish:\n${calls().join("\n")}`);
});

test("a first publish that fails reports failed with its reason", t => {
    let { root } = projectWithRemote(t, false, {
        sync: { exit: 1, json: { error: "fetch failed: no branches found in remote 'origin'" } },
        "dolt push": { exit: 1, stderr: "Error: push rejected by the remote\n" },
    });

    let result = syncBeads(root);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /push rejected by the remote/);
});

/** Makes the project directory a Git checkout whose `origin` is `url`, minus Dolt's `git+`. */
function setOrigin(root: string, url: string): void {
    execFileSync("git", ["init", "-q", root], { env: cleanEnv });
    execFileSync("git", ["-C", root, "remote", "add", "origin", url.replace(/^git\+/, "")], {
        env: cleanEnv,
    });
}

/** A project whose replica's Beads remote and Git `origin` are the same local bare repository. */
function projectWithRemote(t: TestContext, withBeadsHistory: boolean, script: FakeScript) {
    let url = remote(t, { withBeadsHistory });
    let created = project(t, { ...script, "dolt remote list": remotes(url) });
    setOrigin(created.root, url);
    return created;
}

test("a replica whose Beads remote is another repository fails without pulling or pushing", t => {
    let { root, calls } = project(t, {
        "dolt remote list": remotes("git+https://github.com/markmals/workbench.git"),
        sync: SYNCED,
        "dolt push": { stdout: "Push complete.\n" },
    });
    setOrigin(root, "https://github.com/someone/new-project.git");

    let result = syncBeads(root);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /markmals\/workbench/);
    assert.match(result.detail, /someone\/new-project/);
    assert.deepEqual(
        calls().filter(call => call.startsWith("sync") || call.startsWith("dolt push")),
        [],
    );
});

test("a checkout without a Git origin does not sync a replica that has a Beads remote", t => {
    let { root, calls } = project(t, {
        "dolt remote list": remotes("git+https://github.com/markmals/workbench.git"),
        sync: SYNCED,
    });
    execFileSync("git", ["init", "-q", root], { env: cleanEnv });

    let result = syncBeads(root);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /no origin/);
    assert.ok(!calls().some(call => call.startsWith("sync")), calls().join("\n"));
});

test("two local repositories that differ only by .git are different remotes", t => {
    let { root } = project(t, {
        "dolt remote list": remotes("git+file:///srv/git/project.git"),
        sync: SYNCED,
    });
    setOrigin(root, "/srv/git/project");

    assert.equal(syncBeads(root).outcome, "failed");
});

test("claiming in a replica whose Beads remote is another repository changes nothing", t => {
    let { root, calls } = project(t, {
        "dolt remote list": remotes("git+https://github.com/markmals/workbench.git"),
        sync: SYNCED,
        update: { json: [{ id: "wb-1" }] },
        show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
    });
    setOrigin(root, "https://github.com/someone/new-project.git");

    assert.throws(() => claimBeadsItem(root, "wb-1"), /not this project's origin/);
    assert.ok(!calls().some(call => call.startsWith("update")), calls().join("\n"));
});

test("the same repository written another way is this project's own remote", t => {
    let pairs = [
        ["git@github.com:markmals/Workbench.git", "git+https://github.com/markmals/workbench.git"],
        [
            "ssh://git@github.com:22/markmals/workbench",
            "git+https://github.com/markmals/workbench.git/",
        ],
        [
            "https://token@github.com/markmals/workbench.git",
            "git+https://github.com/markmals/workbench",
        ],
        ["/srv/git/project.git", "git+file:///srv/git/project.git"],
    ];
    let { root, bin } = project(t, {});
    setOrigin(root, "unset");
    for (let [origin, beadsRemote] of pairs) {
        installFakeBd(bin, { "dolt remote list": remotes(beadsRemote!), sync: SYNCED });
        execFileSync("git", ["-C", root, "remote", "set-url", "origin", origin!], {
            env: cleanEnv,
        });

        assert.equal(syncBeads(root).outcome, "synced", `${origin} and ${beadsRemote}`);
    }

    let relativeOrigin = "../shared/repo.git";
    installFakeBd(bin, {
        "dolt remote list": remotes(`git+file://${path.resolve(root, relativeOrigin)}`),
        sync: SYNCED,
    });
    execFileSync("git", ["-C", root, "remote", "set-url", "origin", relativeOrigin], {
        env: cleanEnv,
    });
    assert.equal(syncBeads(root).outcome, "synced", "a relative origin resolves from the checkout");
});

test("a conflict Beads cannot settle names the tables and the items that differ from the remote", t => {
    let { root } = project(t, {
        sync: { exit: 2, json: { status: "conflict", conflicts: ["issues"], pushed: false } },
        diff: {
            json: [
                {
                    IssueID: "wb-1",
                    DiffType: "removed",
                    OldValue: { id: "wb-1", title: "Probe one" },
                    NewValue: null,
                },
                {
                    IssueID: "wb-2",
                    DiffType: "added",
                    OldValue: null,
                    NewValue: { id: "wb-2", title: "Probe two" },
                },
            ],
        },
    });

    let result = syncBeads(root);

    assert.equal(result.outcome, "conflict");
    assert.match(result.detail, /issues/);
    assert.match(result.detail, /wb-1 \(Probe one\)/);
    assert.match(result.detail, /wb-2 \(Probe two\)/);
});

test("a stuck dirty working set reports blocked with Beads' explanation", t => {
    let { root } = project(t, {
        sync: { exit: 4, json: { error: "pending graph edits blocked every attempt" } },
    });

    let result = syncBeads(root);

    assert.equal(result.outcome, "blocked");
    assert.match(result.detail, /pending graph edits blocked every attempt/);
});

test("push retries running out reports failed", t => {
    let { root } = projectWithRemote(t, true, {
        sync: { exit: 3, json: { error: "push race: retries exhausted" } },
    });

    let result = syncBeads(root);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /retries exhausted/);
});

test("a sync that outlasts its time limit reports failed", t => {
    let { root } = project(t, { sync: { ...SYNCED, sleepMs: 5_000 } });

    let result = syncBeads(root, 200);

    assert.equal(result.outcome, "failed");
    assert.match(result.detail, /did not finish within 0\.2 seconds/);
});

test("claiming an unheld item syncs, claims, syncs again, and confirms the claim", t => {
    let { root, fake } = project(t, {
        sync: SYNCED,
        update: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
        show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
    });

    let result = claimBeadsItem(root, "wb-1");

    assert.equal(result.outcome, "claimed");
    let steps = fake.calls().filter(({ args }) => args[0] !== "dolt");
    assert.deepEqual(
        steps.map(({ args }) => args[0]),
        ["sync", "update", "sync", "show"],
    );
    assert.equal(steps[1]?.actor, "mark@alpha");
    assert.ok(steps[1]?.args.includes("--claim"));
});

test("a claim whose second sync fails is claimed on this machine only", t => {
    let { root } = projectWithRemote(t, true, {
        sync: [SYNCED, { exit: 1, json: { error: "sync failed: remote unreachable" } }],
        update: { json: [{ id: "wb-1" }] },
        show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
    });

    let result = claimBeadsItem(root, "wb-1");

    assert.equal(result.outcome, "claimed-unpublished");
    assert.match(result.detail, /remote unreachable/);
});

test("an item another claimant holds is refused as held, naming the claimant", t => {
    let { root, calls } = project(t, {
        sync: SYNCED,
        update: {
            exit: 1,
            json: {
                error: "1 of 1 issues failed to update",
                failed: [{ id: "wb-1", error: "issue already claimed by mark@beta" }],
            },
        },
        show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@beta" }] },
    });

    let result = claimBeadsItem(root, "wb-1");

    assert.equal(result.outcome, "held");
    assert.equal(result.holder, "mark@beta");
    assert.match(result.detail, /mark@beta/);
    assert.equal(calls().filter(call => call.startsWith("sync")).length, 1);
});

test("a claim a concurrent claim replaced during the sync is lost, naming the winner", t => {
    let { root } = project(t, {
        sync: SYNCED,
        update: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
        show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@beta" }] },
    });

    let result = claimBeadsItem(root, "wb-1");

    assert.equal(result.outcome, "lost");
    assert.equal(result.holder, "mark@beta");
    assert.match(result.detail, /mark@beta/);
});

test("without BEADS_ACTOR a claim is made as the login user at the short host name", t => {
    let claimant = `${userInfo().username}@${hostname().split(".")[0]}`;
    let { root, fake } = project(
        t,
        {
            sync: SYNCED,
            update: { json: [{ id: "wb-1" }] },
            show: { json: [{ id: "wb-1", status: "in_progress", assignee: claimant }] },
        },
        null,
    );

    let result = claimBeadsItem(root, "wb-1");

    assert.equal(result.outcome, "claimed");
    assert.equal(fake.calls().find(({ args }) => args[0] === "update")?.actor, claimant);
});

test("a claim Beads rejects for another reason fails with Beads' error", t => {
    let { root } = project(t, {
        sync: SYNCED,
        update: {
            exit: 1,
            json: {
                error: "1 of 1 issues failed",
                failed: [{ id: "wb-9", error: "issue not found" }],
            },
        },
        show: { exit: 1, json: { error: "no issues found matching the provided IDs" } },
    });

    assert.throws(() => claimBeadsItem(root, "wb-9"), /issue not found/);
});

test("the report syncs first, then lists ready work, items in progress, and memories", t => {
    let { root, calls } = project(t, {
        where: AVAILABLE,
        sync: SYNCED,
        ready: {
            json: [
                {
                    id: "wb-1",
                    title: "Fix the parser",
                    priority: 1,
                    status: "open",
                    description: "x",
                },
            ],
        },
        list: {
            json: [
                { id: "wb-2", title: "Port status", status: "in_progress", assignee: "mark@beta" },
            ],
        },
        prime: { stdout: "## Persistent Memories (1)\n\n### fact\nA durable fact.\n" },
    });

    let report = collectBeadsReport(root);

    assert.ok("sync" in report, JSON.stringify(report));
    assert.equal(report.sync.outcome, "synced");
    assert.deepEqual(
        { ready: report.ready, in_progress: report.in_progress, memories: report.memories },
        {
            ready: [{ id: "wb-1", title: "Fix the parser", priority: 1 }],
            in_progress: [{ id: "wb-2", title: "Port status", assignee: "mark@beta" }],
            memories: "## Persistent Memories (1)\n\n### fact\nA durable fact.",
        },
    );
    assert.deepEqual(
        calls()
            .filter(call => !call.startsWith("dolt"))
            .map(call => call.split(" ")[0]),
        ["where", "sync", "ready", "list", "prime"],
    );
    assert.ok(calls().includes("list --status in_progress --json"), calls().join("\n"));
    assert.ok(calls().includes("prime --memories-only"), calls().join("\n"));
});

test("a failed sync still reports this machine's replica", t => {
    let { root } = projectWithRemote(t, true, {
        where: AVAILABLE,
        sync: { exit: 1, json: { error: "sync failed: remote unreachable" } },
        ready: { json: [{ id: "wb-1", title: "Fix the parser", priority: 2 }] },
        list: { json: [] },
        prime: { stdout: "No memories stored.\n" },
    });

    let report = collectBeadsReport(root);

    assert.ok("sync" in report, JSON.stringify(report));
    assert.equal(report.sync.outcome, "failed");
    assert.deepEqual(report.ready, [{ id: "wb-1", title: "Fix the parser", priority: 2 }]);
});

test("without bd installed the report is unavailable", t => {
    let empty = mkdtempSync(path.join(tmpdir(), "workbench-beads-empty-"));
    t.after(() => rmSync(empty, { force: true, recursive: true }));
    useEnvironment(t, { PATH: empty });

    assert.deepEqual(collectBeadsReport(empty), { unavailable: "bd is not installed" });
});

test("a directory without a Beads workspace is unavailable, with Beads' reason", t => {
    let { root, calls } = project(t, {
        where: {
            exit: 1,
            json: { error: "no_beads_directory", message: "No active beads workspace found." },
        },
    });

    let report = collectBeadsReport(root);

    assert.ok("unavailable" in report, JSON.stringify(report));
    assert.match(report.unavailable, /No active beads workspace found/);
    assert.deepEqual(calls(), ["where --json"]);
});

test("a workspace whose database is not initialized is unavailable", t => {
    let { root } = project(t, { where: { json: { path: "/project/.beads" } } });

    let report = collectBeadsReport(root);

    assert.ok("unavailable" in report, JSON.stringify(report));
    assert.match(report.unavailable, /not initialized/);
});

test("a replica that cannot be read is unavailable, and the report keeps the sync's outcome", t => {
    let { root } = projectWithRemote(t, true, {
        where: AVAILABLE,
        sync: { exit: 1, json: { error: "sync failed: remote unreachable" } },
        ready: { exit: 1, json: { error: "database is locked" } },
    });

    let report = collectBeadsReport(root);

    assert.ok("unavailable" in report, JSON.stringify(report));
    assert.match(report.unavailable, /database is locked/);
    assert.match(report.unavailable, /failed/);
    assert.match(report.unavailable, /remote unreachable/);
});

test("describing a conflict stays within the sync's time limit", t => {
    let { root } = project(t, {
        sync: { exit: 2, json: { status: "conflict", conflicts: ["issues"], pushed: false } },
        diff: { json: [], sleepMs: 5_000 },
    });

    let started = Date.now();
    let result = syncBeads(root, 500);

    assert.ok(Date.now() - started < 2_000, `took ${Date.now() - started} ms`);
    assert.equal(result.outcome, "conflict");
    assert.match(result.detail, /bd diff main origin\/main/);
});

test("closing an item closes it as this machine's claimant, then publishes it", t => {
    let { root, fake } = project(t, {
        close: { json: [{ id: "wb-1", status: "closed" }] },
        sync: SYNCED,
    });

    let result = closeBeadsItem(root, "wb-1", "Fixed in PR #42");

    assert.equal(result.outcome, "closed");
    let steps = fake.calls().filter(({ args }) => args[0] !== "dolt");
    assert.deepEqual(
        steps.map(({ args }) => args[0]),
        ["close", "sync"],
    );
    assert.equal(steps[0]?.actor, "mark@alpha");
    assert.ok(steps[0]?.args.includes("--reason=Fixed in PR #42"), steps[0]?.args.join(" "));
});

test("a close whose sync fails is closed on this machine only", t => {
    let { root } = projectWithRemote(t, true, {
        close: { json: [{ id: "wb-1", status: "closed" }] },
        sync: { exit: 1, json: { error: "sync failed: remote unreachable" } },
    });

    let result = closeBeadsItem(root, "wb-1", "Done");

    assert.equal(result.outcome, "closed-unpublished");
    assert.match(result.detail, /remote unreachable/);
});

test("a close Beads refuses fails with Beads' reason and does not sync", t => {
    let { root, calls } = project(t, {
        close: {
            exit: 1,
            json: { error: 'cannot close wb-1: assignee is "mark@beta", actor is "mark@alpha"' },
        },
        sync: SYNCED,
    });

    assert.throws(() => closeBeadsItem(root, "wb-1", "Done"), /assignee is "mark@beta"/);
    assert.ok(!calls().some(call => call.startsWith("sync")), calls().join("\n"));
});

test("closing in a replica whose Beads remote is another repository changes nothing", t => {
    let { root, calls } = project(t, {
        "dolt remote list": remotes("git+https://github.com/markmals/workbench.git"),
        close: { json: [{ id: "wb-1", status: "closed" }] },
        sync: SYNCED,
    });
    setOrigin(root, "https://github.com/someone/new-project.git");

    assert.throws(() => closeBeadsItem(root, "wb-1", "Done"), /not this project's origin/);
    assert.ok(!calls().some(call => call.startsWith("close")), calls().join("\n"));
});

function runTask(t: TestContext, script: FakeScript, ...arguments_: string[]) {
    let root = mkdtempSync(path.join(tmpdir(), "workbench-beads-task-"));
    t.after(() => rmSync(root, { force: true, recursive: true }));
    let bin = path.join(root, "bin");
    mkdirSync(bin);
    installFakeBd(bin, script);
    return spawnSync(process.execPath, [beadsCli, ...arguments_], {
        cwd: root,
        encoding: "utf8",
        env: {
            ...cleanEnv,
            PATH: `${bin}${path.delimiter}${cleanEnv.PATH ?? ""}`,
            BEADS_ACTOR: "mark@alpha",
        },
    });
}

test("beads:sync exits zero when synced and nonzero otherwise, printing the outcome", t => {
    let synced = runTask(t, { where: AVAILABLE, sync: SYNCED }, "sync");
    let blocked = runTask(
        t,
        { where: AVAILABLE, sync: { exit: 4, json: { error: "stuck" } } },
        "sync",
    );

    assert.equal(synced.status, 0, synced.stderr);
    assert.match(synced.stdout, /synced/);
    assert.notEqual(blocked.status, 0);
    assert.match(blocked.stdout, /blocked/);
});

test("beads:claim exits zero when claimed and nonzero when held, printing the outcome", t => {
    let claimed = runTask(
        t,
        {
            where: AVAILABLE,
            sync: SYNCED,
            update: { json: [{ id: "wb-1" }] },
            show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@alpha" }] },
        },
        "claim",
        "wb-1",
    );
    let held = runTask(
        t,
        {
            where: AVAILABLE,
            sync: SYNCED,
            update: { exit: 1, json: { failed: [{ id: "wb-1", error: "already claimed" }] } },
            show: { json: [{ id: "wb-1", status: "in_progress", assignee: "mark@beta" }] },
        },
        "claim",
        "wb-1",
    );

    assert.equal(claimed.status, 0, claimed.stderr);
    assert.match(claimed.stdout, /claimed/);
    assert.notEqual(held.status, 0);
    assert.match(held.stdout, /held/);
    assert.match(held.stdout, /mark@beta/);
});

test("beads:close exits zero when closed and nonzero when Beads refuses, printing the outcome", t => {
    let closed = runTask(
        t,
        { where: AVAILABLE, close: { json: [{ id: "wb-1" }] }, sync: SYNCED },
        "close",
        "wb-1",
        "Fixed in PR #42",
    );
    let refused = runTask(
        t,
        { where: AVAILABLE, close: { exit: 1, json: { error: "cannot close wb-1: held" } } },
        "close",
        "wb-1",
        "Fixed in PR #42",
    );

    assert.equal(closed.status, 0, closed.stderr);
    assert.match(closed.stdout, /Beads: closed\./);
    assert.notEqual(refused.status, 0);
    assert.match(refused.stdout, /close failed/);
    assert.match(refused.stdout, /cannot close wb-1: held/);
});

test("beads:close without a reason is a usage error that closes nothing", t => {
    let result = runTask(
        t,
        { where: AVAILABLE, close: { json: [{ id: "wb-1" }] }, sync: SYNCED },
        "close",
        "wb-1",
    );

    assert.equal(result.status, 2);
    assert.match(result.stderr, /close <id> "<reason>"/);
    assert.doesNotMatch(result.stdout, /closed/);
});
