import assert from "node:assert/strict";
import { test } from "node:test";

import { validateArtifacts, type Artifact } from "./validate.ts";

function record(kind: "policy" | "decision", status: string): Artifact {
    return {
        path: `${kind === "policy" ? "policies" : "decisions"}/0001-current.md`,
        frontmatter: { id: `${kind}.0001`, title: "Current contract", status },
        content: "The current rule or choice.",
    };
}

for (let [kind, statuses] of [
    ["policy", ["draft", "active"]],
    ["decision", ["proposed", "accepted"]],
] as const) {
    for (let status of statuses) {
        test(`${kind} ${status} can be revised in place without provenance or a replacement`, () => {
            let current = record(kind, status);
            assert.deepEqual(validateArtifacts([current]), []);
            current.content = "The revised rule or choice, under the same identity.";
            assert.deepEqual(validateArtifacts([current]), []);
        });
    }

    test(`${kind} supersession metadata gives migration guidance even when empty`, () => {
        let current = record(kind, statuses[1]);
        current.frontmatter.supersedes = [];
        let violations = validateArtifacts([current]);
        assert.equal(violations.length, 1);
        assert.equal(violations[0].path, current.path);
        assert.match(violations[0].message, /supersedes/);
        assert.match(violations[0].message, /revise|retire|remove/i);
    });

    test(`${kind} superseded status explains retirement`, () => {
        let current = record(kind, "superseded");
        let violations = validateArtifacts([current]);
        assert.equal(violations.length, 1);
        assert.match(violations[0].message, /superseded/);
        assert.match(violations[0].message, /retire|delete|remove/i);
    });

    test(`${kind} optional provenance must be a scalar reference`, () => {
        let current = record(kind, statuses[1]);
        current.frontmatter["established-by"] = ["proposal.0001"];
        assert.deepEqual(validateArtifacts([current]), [
            { path: current.path, message: 'key "established-by" must be a scalar' },
        ]);
        current.frontmatter["established-by"] = "proposal.9999";
        assert.deepEqual(validateArtifacts([current]), [
            {
                path: current.path,
                message: 'reference "proposal.9999" in established-by does not resolve',
            },
        ]);
    });

    test(`${kind} provenance cannot name a current record instead of a proposal`, () => {
        let current = record(kind, statuses[1]);
        current.frontmatter["established-by"] = `${kind}.0001`;
        assert.deepEqual(validateArtifacts([current]), [
            {
                path: current.path,
                message: `reference "${kind}.0001" in established-by must name a proposal`,
            },
        ]);
    });
}

test("guides use ordinary links without a describes registry", () => {
    let guide: Artifact = {
        path: "guides/current-contract.md",
        frontmatter: { id: "guide.current-contract", title: "Current contract" },
        content: "See [the current rule](../policies/0001-current.md).",
    };
    assert.deepEqual(validateArtifacts([guide]), []);
    guide.frontmatter.describes = ["proposal.0001"];
    let violations = validateArtifacts([guide]);
    assert.equal(violations.length, 1);
    assert.equal(violations[0].path, guide.path);
    assert.match(violations[0].message, /describes/);
    assert.match(violations[0].message, /links/i);
});
