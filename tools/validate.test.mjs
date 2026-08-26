import assert from "node:assert/strict";
import test from "node:test";

import { validateArtifacts, withoutCode } from "./validate.mjs";

function artifact(path, frontmatter, content = "") {
    return { path, frontmatter, content };
}

function validArtifacts() {
    return [
        artifact("proposals/0001-records.md", {
            id: "proposal.0001",
            title: "Record Validation",
            authors: ["Ada"],
            status: "superseded",
            "pull-request": "42",
            supersedes: [],
        }),
        artifact("proposals/0002-replaces.md", {
            id: "proposal.0002",
            title: "Replacement Record",
            authors: ["Ada"],
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
function proposalArtifact(status, content = "") {
    return artifact(
        "proposals/0001-records.md",
        {
            id: "proposal.0001",
            title: "Record Validation",
            authors: ["Ada"],
            status,
            "pull-request": "42",
            supersedes: [],
        },
        content,
    );
}

test("rule 1 rejects missing required keys and invalid statuses", () => {
    const records = validArtifacts();
    records[0] = artifact("proposals/0001-records.md", {
        id: "proposal.0001",
        authors: ["Ada"],
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

test("rule 1 requires a non-empty authors list on proposals", () => {
    const missingAuthors = validArtifacts();
    delete missingAuthors[0].frontmatter.authors;

    assert.deepEqual(
        validateArtifacts(missingAuthors).filter(({ message }) => message.includes("authors")),
        [
            {
                path: "proposals/0001-records.md",
                message: 'missing required key "authors"',
            },
        ],
    );

    const presentAuthors = validArtifacts();
    presentAuthors[0].frontmatter.authors = ["Ada"];

    assert.equal(
        validateArtifacts(presentAuthors).some(({ message }) => message.includes("authors")),
        false,
    );
});

test("rule 1 rejects an empty authors list on proposals", () => {
    const records = validArtifacts();
    records[0].frontmatter.authors = [];

    assert.deepEqual(
        validateArtifacts(records).filter(({ message }) => message.includes("authors")),
        [
            {
                path: "proposals/0001-records.md",
                message: 'key "authors" must be a non-empty inline list',
            },
        ],
    );
});

test("rule 1 accepts newly allowed statuses for their record kinds", () => {
    const cases = [
        ["proposal", 1, "rejected"],
        ["proposal", 1, "withdrawn"],
        ["policy", 2, "draft"],
        ["decision", 3, "proposed"],
    ];

    for (const [type, index, status] of cases) {
        const records = validArtifacts();
        records[index].frontmatter.status = status;

        assert.equal(
            validateArtifacts(records).some(
                ({ message }) => message === `invalid status "${status}" for ${type}`,
            ),
            false,
        );
    }
});

test("rule 1 accepts every proposal status", () => {
    const statuses = [
        "draft",
        "awaiting-implementation",
        "active-review",
        "returned-for-revisions",
        "accepted",
        "implemented",
        "rejected",
        "withdrawn",
        "superseded",
    ];

    for (const status of statuses) {
        assert.deepEqual(validateArtifacts([proposalArtifact(status)]), []);
    }
});

test("rule 1 keeps proposal, policy, and decision statuses distinct", () => {
    const cases = [
        ["proposal", 0, "active"],
        ["proposal", 0, "proposed"],
        ["policy", 2, "implemented"],
        ["policy", 2, "awaiting-implementation"],
        ["decision", 3, "awaiting-implementation"],
    ];

    for (const [type, index, status] of cases) {
        const records = validArtifacts();
        records[index].frontmatter.status = status;

        assert.deepEqual(
            validateArtifacts(records).filter(
                ({ message }) => message === `invalid status "${status}" for ${type}`,
            ),
            [
                {
                    path: records[index].path,
                    message: `invalid status "${status}" for ${type}`,
                },
            ],
        );
    }
});

test("rule 1 accepts absent, empty, and populated proposal issues lists", () => {
    const absentIssues = validArtifacts();
    const emptyIssues = validArtifacts();
    const populatedIssues = validArtifacts();
    emptyIssues[0].frontmatter.issues = [];
    populatedIssues[0].frontmatter.issues = [
        "#12",
        "https://github.com/example/workbench/issues/34",
    ];

    for (const records of [absentIssues, emptyIssues, populatedIssues]) {
        assert.equal(
            validateArtifacts(records).some(({ message }) => message.includes("issues")),
            false,
        );
    }
});

test("rule 1 rejects a proposal issues value that is not a list", () => {
    const records = validArtifacts();
    records[0].frontmatter.issues = "#12";

    assert.deepEqual(
        validateArtifacts(records).filter(({ message }) => message.includes("issues")),
        [
            {
                path: "proposals/0001-records.md",
                message: 'key "issues" must be an inline list',
            },
        ],
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
            authors: ["Ada"],
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

test("rule 5 rejects clarification markers in statuses that require a finished design", () => {
    const statuses = [
        "awaiting-implementation",
        "active-review",
        "accepted",
        "implemented",
        "superseded",
    ];

    for (const status of statuses) {
        const records = validArtifacts();
        records[1].frontmatter.status = status;
        records[1].content = "[NEEDS CLARIFICATION: What is the release boundary?]";

        assert.deepEqual(
            validateArtifacts(records).filter(({ message }) =>
                message.includes("NEEDS CLARIFICATION"),
            ),
            [
                {
                    path: "proposals/0002-replaces.md",
                    message: `proposal status "${status}" does not allow a [NEEDS CLARIFICATION:] marker`,
                },
            ],
        );
    }
});

test("rule 5 permits clarification markers in statuses that allow design changes", () => {
    const statuses = ["draft", "returned-for-revisions", "rejected", "withdrawn"];

    for (const status of statuses) {
        const records = validArtifacts();
        records[1].frontmatter.status = status;
        records[1].content = "[NEEDS CLARIFICATION: What is the release boundary?]";

        assert.equal(
            validateArtifacts(records).some(({ message }) =>
                message.includes("NEEDS CLARIFICATION"),
            ),
            false,
        );
    }
});

test("rule 5 ignores a marker inside an inline code span", () => {
    const records = validArtifacts();
    records[1].frontmatter.status = "accepted";
    records[1].content = "Rule 5 rejects a `[NEEDS CLARIFICATION:` marker in a final proposal.";

    assert.equal(
        validateArtifacts(records).some(({ message }) => message.includes("NEEDS CLARIFICATION")),
        false,
    );
});

test("rule 5 ignores a marker inside a fenced code block", () => {
    const records = validArtifacts();
    records[1].frontmatter.status = "accepted";
    records[1].content = "Example:\n\n```md\n- [NEEDS CLARIFICATION: how?]\n```\n";

    assert.equal(
        validateArtifacts(records).some(({ message }) => message.includes("NEEDS CLARIFICATION")),
        false,
    );
});

test("rule 5 still fires on a prose marker beside a quoted one", () => {
    const records = validArtifacts();
    records[1].frontmatter.status = "accepted";
    records[1].content =
        "The `[NEEDS CLARIFICATION:` form is the convention.\n\n" +
        "- [NEEDS CLARIFICATION: is this genuinely unresolved?]\n";

    assert.equal(
        validateArtifacts(records).some(({ message }) => message.includes("NEEDS CLARIFICATION")),
        true,
    );
});

test("withoutCode strips fences and spans but keeps surrounding prose", () => {
    assert.equal(withoutCode("before `x` after"), "before  after");
    assert.equal(withoutCode("a\n\n```\nb\n```\n\nc"), "a\n\n\n\nc");
});

test("a valid full record set has no violations", () => {
    assert.deepEqual(validateArtifacts(validArtifacts()), []);
});
