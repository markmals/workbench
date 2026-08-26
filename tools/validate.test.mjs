import assert from "node:assert/strict";
import test from "node:test";

import { validateArtifacts } from "./validate.mjs";

function artifact(path, frontmatter, content = "") {
    return { path, frontmatter, content };
}

function validArtifacts() {
    return [
        artifact("proposals/0001-records.md", {
            id: "proposal.0001",
            title: "Record Validation",
            status: "superseded",
            "pull-request": "42",
            supersedes: [],
        }),
        artifact("proposals/0002-replaces.md", {
            id: "proposal.0002",
            title: "Replacement Record",
            status: "accepted",
            "pull-request": "43",
            supersedes: ["proposal.0001"],
        }),
        artifact("policies/0001-records.md", {
            id: "policy.0001",
            title: "Record Validation Policy",
            status: "active",
            "established-by": "proposal.0002",
            supersedes: [],
        }),
        artifact("decisions/0001-records.md", {
            id: "decision.0001",
            title: "Record Validation Decision",
            status: "accepted",
            "established-by": "proposal.0002",
            supersedes: [],
        }),
        artifact("guides/records.md", {
            id: "guide.records",
            title: "Record Validation Guide",
            describes: ["proposal.0002"],
        }),
        artifact("VISION.md", {
            title: "Workbench Vision",
            updated: "2026-08-25",
        }),
    ];
}

test("rule 1 rejects missing required keys and invalid statuses", () => {
    const records = validArtifacts();
    records[0] = artifact("proposals/0001-records.md", {
        id: "proposal.0001",
        status: "retired",
        "pull-request": "42",
        supersedes: [],
    });
    records[1].frontmatter.supersedes = [];

    assert.deepEqual(validateArtifacts(records), [
        {
            path: "proposals/0001-records.md",
            message: 'missing required key "title"',
        },
        {
            path: "proposals/0001-records.md",
            message: 'invalid status "retired" for proposal',
        },
    ]);
});

test("rule 1 accepts every required key and allowed status", () => {
    assert.equal(
        validateArtifacts(validArtifacts()).some(
            ({ message }) => message.includes("required key") || message.includes("invalid status"),
        ),
        false,
    );
});

test("rule 2 rejects IDs that do not match their filenames and duplicates", () => {
    const records = validArtifacts();
    records[0].frontmatter.id = "proposal.9999";
    records[1].frontmatter.id = "policy.0001";

    assert.deepEqual(
        validateArtifacts(records).filter(({ message }) => message.includes("ID")),
        [
            {
                path: "proposals/0001-records.md",
                message: 'ID "proposal.9999" must be "proposal.0001" for this filename',
            },
            {
                path: "proposals/0002-replaces.md",
                message: 'ID "policy.0001" must be "proposal.0002" for this filename',
            },
            {
                path: "proposals/0002-replaces.md",
                message: 'duplicate ID "policy.0001"; first declared in policies/0001-records.md',
            },
        ],
    );
});

test("rule 2 accepts IDs matching unique filenames", () => {
    assert.equal(
        validateArtifacts(validArtifacts()).some(({ message }) => message.includes("ID")),
        false,
    );
});

test("rule 3 rejects unresolved cross-references", () => {
    const records = validArtifacts();
    records[4].frontmatter.describes = ["proposal.9999"];

    assert.deepEqual(
        validateArtifacts(records).filter(({ message }) => message.includes("does not resolve")),
        [
            {
                path: "guides/records.md",
                message: 'reference "proposal.9999" in describes does not resolve',
            },
        ],
    );
});

test("rule 3 accepts resolved cross-references", () => {
    assert.equal(
        validateArtifacts(validArtifacts()).some(({ message }) =>
            message.includes("does not resolve"),
        ),
        false,
    );
});

function policySupersession(status) {
    return [
        artifact("proposals/0001-records.md", {
            id: "proposal.0001",
            title: "Record Validation",
            status: "accepted",
            "pull-request": "42",
            supersedes: [],
        }),
        artifact("policies/0001-old.md", {
            id: "policy.0001",
            title: "Old Policy",
            status,
            "established-by": "proposal.0001",
            supersedes: [],
        }),
        artifact("policies/0002-new.md", {
            id: "policy.0002",
            title: "New Policy",
            status: "active",
            "established-by": "proposal.0001",
            supersedes: ["policy.0001"],
        }),
    ];
}

test("rule 4 rejects a superseded record that remains active", () => {
    assert.deepEqual(
        validateArtifacts(policySupersession("active")).filter(({ message }) =>
            message.includes("must have status"),
        ),
        [
            {
                path: "policies/0001-old.md",
                message: 'record superseded by policy.0002 must have status "superseded"',
            },
        ],
    );
});

test("rule 4 accepts a record marked superseded", () => {
    assert.equal(
        validateArtifacts(policySupersession("superseded")).some(({ message }) =>
            message.includes("must have status"),
        ),
        false,
    );
});

test("rule 5 rejects clarification markers in a finalized proposal", () => {
    const records = validArtifacts();
    records[1].content = "[NEEDS CLARIFICATION: What is the release boundary?]";

    assert.deepEqual(
        validateArtifacts(records).filter(({ message }) => message.includes("NEEDS CLARIFICATION")),
        [
            {
                path: "proposals/0002-replaces.md",
                message: "finalized proposal contains a [NEEDS CLARIFICATION:] marker",
            },
        ],
    );
});

test("rule 5 permits clarification markers in draft proposals", () => {
    const records = validArtifacts();
    records[1].frontmatter.status = "draft";
    records[1].content = "[NEEDS CLARIFICATION: What is the release boundary?]";

    assert.equal(
        validateArtifacts(records).some(({ message }) => message.includes("NEEDS CLARIFICATION")),
        false,
    );
});

test("a valid full record set has no violations", () => {
    assert.deepEqual(validateArtifacts(validArtifacts()), []);
});
