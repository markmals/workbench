import assert from "node:assert/strict";
import test from "node:test";

import { deriveStatus } from "./status.mjs";

function facts(overrides = {}) {
    return {
        branch: "feature",
        isDefaultBranch: false,
        hasUpstream: true,
        unpushedCommits: 0,
        pullRequest: {
            number: 1,
            isDraft: true,
            url: "https://github.com/owner/repo/pull/1",
            title: "Feature",
            isMerged: false,
        },
        proposal: null,
        ...overrides,
    };
}

function proposal(status, clarificationCount = 0) {
    return {
        id: "proposal.0001",
        title: "Feature",
        status,
        clarificationCount,
    };
}

// Asserts the derived routing — phase, who acts, which skill. The `next` sentence is human-facing
// copy that will be reworded; pinning it here would make every wording change a test failure.
function assertPhase(input, expected) {
    const status = deriveStatus(input);

    assert.equal(status.phase, expected.phase);
    assert.equal(status.title, expected.title);
    assert.equal(status.actor, expected.actor);
    assert.equal(status.skill, expected.skill);
    assert.ok(status.next.length > 0, "every phase must say what happens next");
    assert.ok(status.situation.length > 0, "every phase must describe the situation");
    return status;
}

test("derives preparation on the default branch", () => {
    assertPhase(facts({ branch: "main", isDefaultBranch: true, pullRequest: null }), {
        phase: "1",
        title: "Preparation",
        actor: "agent",
        skill: null,
    });
});

test("derives preparation for a branch without a pull request", () => {
    assertPhase(facts({ pullRequest: null }), {
        phase: "1",
        title: "Preparation",
        actor: "agent",
        skill: null,
    });
});

test("derives proposal work for a draft pull request without a proposal", () => {
    assertPhase(facts(), {
        phase: "2",
        title: "Proposal",
        actor: "agent",
        skill: ".agents/skills/writing-a-proposal/",
    });
});

test("keeps a draft proposal in proposal work", () => {
    assertPhase(facts({ proposal: proposal("draft") }), {
        phase: "2",
        title: "Proposal",
        actor: "you",
        skill: null,
    });
});

test("derives implementation for an awaiting proposal", () => {
    assertPhase(facts({ proposal: proposal("awaiting-implementation") }), {
        phase: "3",
        title: "Implementation",
        actor: "agent",
        skill: ".agents/skills/implementing-a-proposal/",
    });
});

test("derives human review for an active-review proposal", () => {
    assertPhase(facts({ proposal: proposal("active-review") }), {
        phase: "5",
        title: "Human review",
        actor: "you",
        skill: null,
    });
});

test("derives revisions for a returned proposal", () => {
    assertPhase(facts({ proposal: proposal("returned-for-revisions") }), {
        phase: "5 → 3",
        title: "Human review → Implementation",
        actor: "agent",
        skill: ".agents/skills/implementing-a-proposal/",
    });
});

test("derives completion for an accepted proposal", () => {
    assertPhase(facts({ proposal: proposal("accepted") }), {
        phase: "5",
        title: "Completion",
        actor: "agent",
        skill: ".agents/skills/completing-a-feature/",
    });
});

test("derives done for an implemented merged pull request", () => {
    assertPhase(facts({ proposal: proposal("implemented"), pullRequest: { isMerged: true } }), {
        phase: "done",
        title: "Done",
        actor: "you",
        skill: null,
    });
});

test("flags a draft pull request when the proposal is in active review", () => {
    const status = deriveStatus(facts({ proposal: proposal("active-review") }));

    assert.deepEqual(status.notices, [{ kind: "draft-active-review" }]);
});

test("surfaces unresolved clarification markers and unpushed commits", () => {
    const status = deriveStatus(facts({ proposal: proposal("draft", 2), unpushedCommits: 3 }));

    assert.deepEqual(status.notices, [
        { kind: "clarifications", count: 2 },
        { kind: "unpushed-commits", count: 3 },
    ]);
});

test("treats unavailable gh data as a branch without a pull request", () => {
    assertPhase(facts({ pullRequest: null, ghAvailable: false }), {
        phase: "1",
        title: "Preparation",
        actor: "agent",
        skill: null,
    });
});
