import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import formatAfterEdit from "../.omp/extensions/format-after-edit.ts";

test("formatting skips Gherkin while still formatting supported files and reporting parse errors", async t => {
    let root = path.resolve(import.meta.dirname, "..");
    let fixture = mkdtempSync(path.join(root, ".workbench-format-"));
    t.after(() => rmSync(fixture, { recursive: true, force: true }));
    let feature = path.join(fixture, "workflow.feature");
    let source = "Feature: Workflow\n  Scenario: Ready\n    Given a ready workflow\n";
    writeFileSync(feature, source);
    let onResult: (
        event: { toolName: string; input: { path: string }; isError: boolean },
        context: { cwd: string },
    ) => Promise<{ additionalContext: string } | undefined> = async () => {
        throw new Error("Formatter handler was not registered");
    };
    formatAfterEdit({
        on(_name: string, handler: typeof onResult) {
            onResult = handler;
        },
        async exec(command: string, args: string[], options: { cwd: string }) {
            let run = spawnSync(command, args, { ...options, encoding: "utf8" });
            if (run.error) throw run.error;
            return { code: run.status, stdout: run.stdout, stderr: run.stderr };
        },
    } as unknown as Parameters<typeof formatAfterEdit>[0]);

    let context = await onResult(
        { toolName: "write", input: { path: feature }, isError: false },
        { cwd: root },
    );
    assert.equal(
        context,
        undefined,
        "An unsupported extension must not produce a failure diagnostic",
    );
    assert.equal(readFileSync(feature, "utf8"), source);

    let json = path.join(fixture, "config.json");
    let unformatted = '{"value":1}';
    writeFileSync(json, unformatted);
    let formatted = await onResult(
        { toolName: "write", input: { path: json }, isError: false },
        { cwd: root },
    );
    assert.ok(formatted);
    assert.match(formatted.additionalContext, /reformatted/);
    assert.notEqual(readFileSync(json, "utf8"), unformatted);
    assert.deepEqual(JSON.parse(readFileSync(json, "utf8")), { value: 1 });

    let invalid = path.join(fixture, "invalid.ts");
    writeFileSync(invalid, "let =");
    let failed = await onResult(
        { toolName: "write", input: { path: invalid }, isError: false },
        { cwd: root },
    );
    assert.ok(failed);
    assert.match(failed.additionalContext, /failed/);
    assert.equal(readFileSync(invalid, "utf8"), "let =");
});
