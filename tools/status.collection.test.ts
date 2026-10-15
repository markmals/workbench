import { execFileSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { expect, onTestFinished, test } from "vite-plus/test";

let statusScript = new URL("./status.ts", import.meta.url).pathname;

function statusWithPullRequests(pullRequests: object[]): string {
    let root = mkdtempSync(path.join(tmpdir(), "workbench-status-"));
    onTestFinished(() => rmSync(root, { force: true, recursive: true }));
    execFileSync("git", ["init", "-q", "-b", "feature", root]);
    execFileSync("git", [
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
    ]);
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
    mkdirSync(path.join(root, "proposals"));
    writeFileSync(
        path.join(root, "proposals", "0001-feature.md"),
        "---\nid: proposal.0001\ntitle: Feature\nstatus: implemented\npull-request: 1\n---\n",
    );
    return execFileSync(process.execPath, [statusScript], {
        cwd: root,
        encoding: "utf8",
        env: {
            ...process.env,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
            TEST_PULL_REQUESTS: JSON.stringify(pullRequests),
        },
    });
}

test("finds a merged feature PR after it leaves the open PR list", () => {
    let report = statusWithPullRequests([
        {
            number: 1,
            state: "MERGED",
            baseRefName: "main",
            isDraft: false,
            title: "Feature",
            url: "https://github.com/owner/repo/pull/1",
        },
    ]);

    expect(report).toContain("PR #1 (merged)");
    expect(report).toContain("Phase 5 — Implementation merged");
});

test("does not select an arbitrary PR when a branch name has been reused", () => {
    let report = statusWithPullRequests([
        { number: 1, state: "MERGED", baseRefName: "main", isDraft: false, title: "Earlier" },
        { number: 2, state: "CLOSED", baseRefName: "main", isDraft: false, title: "Later" },
    ]);

    expect(report).toContain("PR — unknown (Multiple pull requests found for branch feature");
    expect(report).toContain("inspect their proposal links");
    expect(report).toContain("Check gh access and the branch's pull request association");
    expect(report).not.toContain("Implementation merged");
});
