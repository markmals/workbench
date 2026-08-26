import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

import { parseFrontmatter, withoutCode } from "./validate.mjs";

const NON_TERMINAL_STATUSES = new Set([
    "draft",
    "awaiting-implementation",
    "active-review",
    "returned-for-revisions",
    "accepted",
]);
const SKILLS = {
    proposal: ".agents/skills/writing-a-proposal/",
    implementation: ".agents/skills/implementing-a-proposal/",
    completion: ".agents/skills/completing-a-feature/",
};

function step(phase, title, situation, next, actor, skill = null) {
    return { phase, title, situation, next, actor, skill };
}

const STEPS = {
    defaultBranch: step(
        "1",
        "Preparation",
        "You are on the default branch.",
        "Create a branch, push it, open a draft pull request.",
        "agent",
    ),
    noPullRequest: step(
        "1",
        "Preparation",
        "This branch has no pull request.",
        "Push the branch and open a draft pull request.",
        "agent",
    ),
    noProposal: step(
        "2",
        "Proposal",
        "The pull request has no proposal.",
        "Write the proposal and push it as soon as it is coherent enough to react to.",
        "agent",
        SKILLS.proposal,
    ),
    draft: step(
        "2",
        "Proposal",
        "The proposal is published and being iterated on.",
        "Edit the proposal directly, or tell the agent what to change. Move it to awaiting-implementation when you are satisfied.",
        "you",
    ),
    "awaiting-implementation": step(
        "3",
        "Implementation",
        "The proposal is approved for implementation.",
        "Failing tests, then guides from the proposal, then code, until mise run check passes.",
        "agent",
        SKILLS.implementation,
    ),
    "active-review": step(
        "5",
        "Human review",
        "A first full pass at tests, guides, and code is up for review.",
        "Review the diff and the preview. Comment on the pull request, or accept the work.",
        "you",
    ),
    "returned-for-revisions": step(
        "5 → 3",
        "Human review → Implementation",
        "The human returned the pull request for revisions.",
        "Process the feedback; update the proposal first if the design changed.",
        "agent",
        SKILLS.implementation,
    ),
    accepted: step(
        "5",
        "Completion",
        "The feature has been accepted and awaits completion.",
        "Update the vision, merge, release, clean up.",
        "agent",
        SKILLS.completion,
    ),
    done: step(
        "done",
        "Done",
        "The merged proposal is recorded as implemented.",
        "Nothing outstanding. Start the next feature when ready.",
        "you",
    ),
    unknown: step(
        "2",
        "Proposal",
        "The proposal status is not recognized.",
        "Update the proposal status in the pull request.",
        "you",
    ),
};

function commandOutput(command, arguments_, root) {
    try {
        return execFileSync(command, arguments_, {
            cwd: root,
            encoding: "utf8",
            stdio: ["ignore", "pipe", "ignore"],
        }).trim();
    } catch {
        return null;
    }
}

function commandSucceeds(command, arguments_, root) {
    try {
        execFileSync(command, arguments_, { cwd: root, stdio: "ignore" });
        return true;
    } catch {
        return false;
    }
}

function collectBranch(root) {
    const branch = commandOutput("git", ["rev-parse", "--abbrev-ref", "HEAD"], root);
    const defaultReference = commandOutput(
        "git",
        ["symbolic-ref", "refs/remotes/origin/HEAD"],
        root,
    );
    return {
        branch,
        defaultReference,
        isDefaultBranch: branch !== null && branch === defaultReference?.split("/").at(-1),
    };
}

function collectUpstream(root) {
    const upstream = commandOutput(
        "git",
        ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"],
        root,
    );
    const count =
        upstream && commandOutput("git", ["rev-list", "--count", "@{upstream}..HEAD"], root);
    return {
        hasUpstream: upstream !== null,
        unpushedCommits: Number.isSafeInteger(Number(count)) ? Number(count) : 0,
    };
}

function collectPullRequest(root) {
    try {
        const pullRequest = JSON.parse(
            commandOutput("gh", ["pr", "view", "--json", "number,isDraft,url,title"], root),
        );
        return Number.isInteger(pullRequest.number) ? pullRequest : null;
    } catch {
        return null;
    }
}

function countClarifications(content) {
    return (withoutCode(content).match(/\[NEEDS CLARIFICATION:/g) ?? []).length;
}

function readProposal(root, name) {
    try {
        const source = readFileSync(path.join(root, "proposals", name), "utf8");
        const { frontmatter } = parseFrontmatter(source);
        if (!frontmatter) return null;
        return {
            id: frontmatter.id,
            title: frontmatter.title,
            status: frontmatter.status,
            pullRequest: frontmatter["pull-request"],
            clarificationCount: countClarifications(source),
        };
    } catch {
        return null;
    }
}

function collectProposals(root) {
    try {
        return readdirSync(path.join(root, "proposals"), { withFileTypes: true })
            .filter(
                (entry) =>
                    entry.isFile() && entry.name.endsWith(".md") && entry.name !== "README.md",
            )
            .map((entry) => readProposal(root, entry.name))
            .filter(Boolean);
    } catch {
        return [];
    }
}

function proposalNumber(proposal) {
    return Number(proposal.id?.match(/(\d+)$/)?.[1] ?? -1);
}

function selectProposal(proposals, pullRequest) {
    const ordered = [...proposals].sort(
        (left, right) => proposalNumber(right) - proposalNumber(left),
    );
    const number = String(pullRequest?.number);
    const matched =
        pullRequest &&
        ordered.find(
            ({ pullRequest: reference }) =>
                reference === number || reference?.endsWith(`/pull/${number}`),
        );
    return matched ?? ordered.find(({ status }) => NON_TERMINAL_STATUSES.has(status)) ?? null;
}

function isMergedIntoDefault(root, defaultReference) {
    return (
        defaultReference &&
        commandSucceeds(
            "git",
            [
                "merge-base",
                "--is-ancestor",
                "HEAD",
                defaultReference.replace(/^refs\/remotes\//, ""),
            ],
            root,
        )
    );
}

export function gatherFacts(root = process.cwd()) {
    const branch = collectBranch(root);
    const upstream = collectUpstream(root);
    const pullRequest = collectPullRequest(root);
    return {
        branch: branch.branch,
        isDefaultBranch: branch.isDefaultBranch,
        hasUpstream: upstream.hasUpstream,
        unpushedCommits: upstream.unpushedCommits,
        pullRequest: pullRequest && {
            ...pullRequest,
            isMerged: isMergedIntoDefault(root, branch.defaultReference),
        },
        proposal: selectProposal(collectProposals(root), pullRequest),
    };
}

function noticesFor(facts) {
    const notices = [];
    if (facts.proposal?.clarificationCount > 0)
        notices.push({ kind: "clarifications", count: facts.proposal.clarificationCount });
    if (facts.unpushedCommits > 0)
        notices.push({ kind: "unpushed-commits", count: facts.unpushedCommits });
    if (facts.proposal?.status === "active-review" && facts.pullRequest?.isDraft)
        notices.push({ kind: "draft-active-review" });
    return notices;
}

function result(facts, status) {
    return { ...status, notices: noticesFor(facts) };
}

export function deriveStatus(facts) {
    if (facts.isDefaultBranch) return result(facts, STEPS.defaultBranch);
    if (!facts.pullRequest) return result(facts, STEPS.noPullRequest);
    if (!facts.proposal) return result(facts, STEPS.noProposal);
    if (facts.proposal.status === "implemented")
        return result(facts, facts.pullRequest.isMerged ? STEPS.done : STEPS["active-review"]);
    return result(facts, STEPS[facts.proposal.status] ?? STEPS.unknown);
}

function noticeLine(notice) {
    if (notice.kind === "clarifications") return `Clarifications: ${notice.count} unresolved`;
    if (notice.kind === "unpushed-commits") return `Unpushed commits: ${notice.count}`;
    return "Warning: proposal is active-review, but the pull request remains draft.";
}

export function formatStatus(facts, derived = deriveStatus(facts)) {
    const proposal = facts.proposal
        ? `${facts.proposal.id}  ${facts.proposal.title}  ·  ${facts.proposal.status}`
        : "proposal — none";
    const pullRequest = facts.pullRequest
        ? `PR #${facts.pullRequest.number} (${facts.pullRequest.isDraft ? "draft" : "open"})  ${facts.pullRequest.url}`
        : "PR — none";
    return [
        proposal,
        pullRequest,
        `branch ${facts.branch ?? "unavailable"}`,
        "",
        `Phase ${derived.phase} — ${derived.title}`,
        `  ${derived.situation}`,
        "",
        `Next (${derived.actor}): ${derived.next}`,
        ...(derived.skill ? [`              Skill: ${derived.skill}`] : []),
        ...derived.notices.map(noticeLine),
    ].join("\n");
}

function main() {
    const facts = gatherFacts();
    const derived = deriveStatus(facts);
    console.log(
        process.argv.includes("--json") ? JSON.stringify(derived) : formatStatus(facts, derived),
    );
}

if (import.meta.main) main();
