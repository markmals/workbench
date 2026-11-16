//! The steps of the process `workflow status` can report: each one's phase, situation, next
//! action, who takes it, and the skill that covers it.

use schemars::JsonSchema;
use serde::Serialize;

/// Who acts next: the agent, or the human reading the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    Agent,
    You,
}

impl Actor {
    pub fn as_str(self) -> &'static str {
        match self {
            Actor::Agent => "agent",
            Actor::You => "you",
        }
    }
}

/// One step, as fixed text.
pub struct Step {
    pub phase: &'static str,
    pub title: &'static str,
    pub situation: &'static str,
    pub next: &'static str,
    pub actor: Actor,
    pub skill: Option<&'static str>,
}

const PROPOSAL_SKILL: Option<&str> = Some("writing-a-proposal");
const IMPLEMENTATION_SKILL: Option<&str> = Some("implementing-a-proposal");
const COMPLETION_SKILL: Option<&str> = Some("completing-a-feature");

pub const DEFAULT_BRANCH: Step = Step {
    phase: "1",
    title: "Preparation",
    situation: "You are on the default branch.",
    next: "If considering a proposal, ask the human whether they want a proposal for this change before creating a branch or draft pull request; otherwise handle the correction or polish directly.",
    actor: Actor::Agent,
    skill: None,
};

pub const NO_PULL_REQUEST: Step = Step {
    phase: "1",
    title: "Preparation",
    situation: "This branch has no pull request.",
    next: "If considering a proposal, ask the human whether they want a proposal for this change before pushing the branch or opening a draft pull request. Corrections and look-and-feel polish need no proposal.",
    actor: Actor::Agent,
    skill: None,
};

pub const NO_PROPOSAL: Step = Step {
    phase: "2",
    title: "Proposal",
    situation: "The pull request has no proposal.",
    next: "Corrections and look-and-feel polish continue without a proposal. Otherwise ask the human whether they want a proposal for this change before writing one, unless they already answered for this change. If yes, draft and publish it.",
    actor: Actor::Agent,
    skill: PROPOSAL_SKILL,
};

pub const DRAFT: Step = Step {
    phase: "2",
    title: "Proposal",
    situation: "The proposal is published and being iterated on.",
    next: "Edit the proposal directly, or tell the agent what to change. Move it to awaiting-implementation when you are satisfied.",
    actor: Actor::You,
    skill: None,
};

pub const AWAITING_IMPLEMENTATION: Step = Step {
    phase: "3",
    title: "Implementation",
    situation: "The proposal is approved for implementation.",
    next: "Update the current specifications, then write failing tests, guides, and code using the project's native tools, and pass the gates. When the complete implementation is pushed, mark the pull request ready for maintainer review. Then self-check the current requirements and approved change, get one independent review, and post the readiness report.",
    actor: Actor::Agent,
    skill: IMPLEMENTATION_SKILL,
};

pub const AWAITING_IMPLEMENTATION_READY: Step = Step {
    phase: "3",
    title: "Implementation",
    situation: "The proposal is approved for implementation; its pull request is already ready for human inspection.",
    next: "Self-check the current requirements and approved change, get one independent review, then post the readiness report and move the proposal to active-review. Address any review revisions with fresh checks.",
    actor: Actor::Agent,
    skill: IMPLEMENTATION_SKILL,
};

pub const ACTIVE_REVIEW: Step = Step {
    phase: "4",
    title: "Human review",
    situation: "The current specifications, tests, guides, and implementation are up for review.",
    next: "Review the current requirements, diff, and preview against the approved change. Ask for look-and-feel changes as you go, comment on the pull request, or accept the work.",
    actor: Actor::You,
    skill: None,
};

pub const RETURNED_FOR_REVISIONS: Step = Step {
    phase: "4 → 3",
    title: "Human review → Implementation",
    situation: "The human returned the pull request for revisions.",
    next: "Process the feedback; discuss changes to intent or scope with the human, revise the proposal when needed, and update affected current specifications before tests, guides, and implementation.",
    actor: Actor::Agent,
    skill: IMPLEMENTATION_SKILL,
};

pub const ACCEPTED: Step = Step {
    phase: "5",
    title: "Completion",
    situation: "The feature has been accepted and awaits completion.",
    next: "Update the vision, record the accepted implementation in this feature pull request, verify that revision, then merge and complete any separately authorized required delivery; clean up afterward.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const ACCEPTED_DRAFT: Step = Step {
    phase: "5",
    title: "Completion",
    situation: "The feature has been accepted, but its pull request remains draft.",
    next: "Update the vision, record the accepted implementation and confirm its status in the feature pull request, then mark the PR ready. Verify the latest integration candidate before requesting a merge.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const ACCEPTED_AFTER_MERGE: Step = Step {
    phase: "5",
    title: "Completion review",
    situation: "The feature pull request merged, but the local proposal still says accepted; the merged record has not been checked.",
    next: "Confirm whether the implemented record is in the merged pull request; if absent, ask the human how to reconcile without a status-only PR. Track any required delivery separately; do not try to edit the merged PR.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const REJECTED: Step = closed("The work is closed: the proposal was rejected.");
pub const WITHDRAWN: Step = closed("The work is closed: the proposal was withdrawn.");
pub const SUPERSEDED: Step = closed("The work is closed: the proposal was superseded.");

const fn closed(situation: &'static str) -> Step {
    Step {
        phase: "done",
        title: "Closed",
        situation,
        next: "Nothing outstanding.",
        actor: Actor::You,
        skill: None,
    }
}

pub const DRAFT_IMPLEMENTATION: Step = Step {
    phase: "5",
    title: "Completion",
    situation: "The local proposal is implemented, but the feature pull request remains draft.",
    next: "Confirm the implemented record is in the feature pull request, then mark it ready for review and verify the latest integration candidate before seeking a merge.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const READY_TO_MERGE: Step = Step {
    phase: "5",
    title: "Completion",
    situation: "The local proposal says implemented; its status in the feature pull request has not been verified.",
    next: "Confirm the implemented record is in this feature pull request before the merge. Verify the latest integration candidate, then merge and complete any separately authorized delivery; clean up afterward.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const MERGED: Step = Step {
    phase: "5",
    title: "Implementation merged",
    situation: "The feature pull request is merged; the local implemented status does not prove its record reached the target branch.",
    next: "Confirm the implemented record is in the merged pull request; if missing, report the inconsistency rather than opening a status-only PR. Track any required release until authorized and delivered, then perform authorized cleanup.",
    actor: Actor::Agent,
    skill: COMPLETION_SKILL,
};

pub const CLOSED_WITHOUT_MERGE: Step = Step {
    phase: "5",
    title: "Completion blocked",
    situation: "The feature pull request closed without merging.",
    next: "Ask the human whether to reopen the pull request or withdraw the proposal; do not attempt to merge a closed pull request.",
    actor: Actor::You,
    skill: None,
};

pub const UNKNOWN_STATUS: Step = Step {
    phase: "2",
    title: "Proposal",
    situation: "The proposal status is not recognized.",
    next: "Update the proposal status in the pull request.",
    actor: Actor::You,
    skill: None,
};

pub const UNKNOWN_BRANCH_ROLE: Step = Step {
    phase: "unknown",
    title: "Branch role unknown",
    situation: "The default branch cannot be determined, so this branch's role is unknown.",
    next: "Determine the default branch before taking pull request actions.",
    actor: Actor::Agent,
    skill: None,
};

/// The step a proposal status leads to when nothing about the pull request overrides it.
pub fn for_status(status: &str) -> Option<&'static Step> {
    match status {
        "draft" => Some(&DRAFT),
        "awaiting-implementation" => Some(&AWAITING_IMPLEMENTATION),
        "active-review" => Some(&ACTIVE_REVIEW),
        "returned-for-revisions" => Some(&RETURNED_FOR_REVISIONS),
        "accepted" => Some(&ACCEPTED),
        "rejected" => Some(&REJECTED),
        "withdrawn" => Some(&WITHDRAWN),
        "superseded" => Some(&SUPERSEDED),
        _ => None,
    }
}
