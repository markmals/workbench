//! Which phase the facts put the work in, and what happens next.

use regex::Regex;
use workbench::commands::workflow::{
    Actor, DerivedStatus, Facts, Notice, Proposal, ProposalSelection, ProposalSummary, PullRequest,
    PullRequestLookup, PullRequestState, default_branch_facts, derive_status, status_text,
};

fn open_pull_request() -> PullRequest {
    PullRequest {
        number: 1,
        is_draft: true,
        state: PullRequestState::Open,
        base_ref_name: "main".into(),
        url: "https://github.com/owner/repo/pull/1".into(),
        title: "Feature".into(),
    }
}

fn ready(state: PullRequestState) -> PullRequestLookup {
    PullRequestLookup::Found(PullRequest {
        is_draft: false,
        state,
        ..open_pull_request()
    })
}

fn facts() -> Facts {
    Facts {
        branch: Some("feature".into()),
        is_default_branch: Some(false),
        default_branch: Some("main".into()),
        has_upstream: true,
        unpushed_commits: 0,
        pull_request: PullRequestLookup::Found(open_pull_request()),
        proposal: None,
        proposal_successor: None,
        proposal_selection: ProposalSelection::default(),
    }
}

fn on_default_branch() -> Facts {
    Facts {
        branch: Some("main".into()),
        is_default_branch: Some(true),
        pull_request: PullRequestLookup::None,
        ..facts()
    }
}

fn with_proposal(status: &str) -> Facts {
    Facts {
        proposal: Some(proposal(status)),
        ..facts()
    }
}

fn proposal(status: &str) -> Proposal {
    Proposal {
        id: Some("proposal.0001".into()),
        title: Some("Feature".into()),
        status: Some(status.into()),
        pull_request: None,
        supersedes: None,
        clarification_count: 0,
    }
}

fn summary(id: &str, title: &str) -> ProposalSummary {
    ProposalSummary {
        id: Some(id.into()),
        title: Some(title.into()),
    }
}

fn assert_phase(
    input: &Facts,
    phase: &str,
    title: &str,
    actor: Actor,
    skill: Option<&str>,
) -> DerivedStatus {
    let status = derive_status(input);
    assert_eq!(
        (
            status.phase.as_str(),
            status.title.as_str(),
            status.actor,
            status.skill.as_deref()
        ),
        (phase, title, actor, skill),
        "{status:#?}"
    );
    status
}

fn assert_matches(text: &str, pattern: &str) {
    let regex = Regex::new(pattern).expect("a valid pattern");
    assert!(regex.is_match(text), "expected /{pattern}/ in:\n{text}");
}

fn assert_no_match(text: &str, pattern: &str) {
    let regex = Regex::new(pattern).expect("a valid pattern");
    assert!(!regex.is_match(text), "expected no /{pattern}/ in:\n{text}");
}

fn report(input: &Facts) -> String {
    status_text(input, &derive_status(input))
}

#[test]
fn derives_preparation_on_the_default_branch() {
    assert_phase(&on_default_branch(), "1", "Preparation", Actor::Agent, None);
}

#[test]
fn derives_preparation_for_a_branch_without_a_pull_request() {
    let input = Facts {
        pull_request: PullRequestLookup::None,
        ..facts()
    };
    assert_phase(&input, "1", "Preparation", Actor::Agent, None);
}

#[test]
fn derives_proposal_work_for_a_draft_pull_request_without_a_proposal() {
    assert_phase(
        &facts(),
        "2",
        "Proposal",
        Actor::Agent,
        Some("writing-a-proposal"),
    );
}

#[test]
fn asks_before_opening_a_proposal_workspace_or_drafting_a_proposal() {
    let without_pull_request = Facts {
        pull_request: PullRequestLookup::None,
        ..facts()
    };
    for (place, input) in [
        ("on the default branch", on_default_branch()),
        ("on a branch without a pull request", without_pull_request),
        ("on a pull request without a proposal", facts()),
    ] {
        let status = derive_status(&input);
        assert_eq!(status.actor, Actor::Agent, "{place}");
        assert_matches(
            &status.next,
            "(?i)ask the human whether they want a proposal",
        );
        assert_matches(&status.next, "(?i)before (creating|pushing|writing)");
    }
}

#[test]
fn keeps_a_draft_proposal_in_proposal_work() {
    assert_phase(&with_proposal("draft"), "2", "Proposal", Actor::You, None);
}

#[test]
fn derives_implementation_for_an_awaiting_proposal() {
    assert_phase(
        &with_proposal("awaiting-implementation"),
        "3",
        "Implementation",
        Actor::Agent,
        Some("implementing-a-proposal"),
    );
}

#[test]
fn guides_the_agent_to_ready_a_draft_pr_only_after_complete_implementation() {
    let status = derive_status(&with_proposal("awaiting-implementation"));

    assert_matches(&status.next, "(?i)complete implementation");
    assert_matches(&status.next, "(?i)implementation.*push.*mark.*ready");
    assert_matches(&status.next, "(?i)readiness report");
}

#[test]
fn a_ready_feature_pr_stays_in_implementation_until_final_review_evidence_is_ready() {
    let input = Facts {
        pull_request: ready(PullRequestState::Open),
        ..with_proposal("awaiting-implementation")
    };
    let status = assert_phase(
        &input,
        "3",
        "Implementation",
        Actor::Agent,
        Some("implementing-a-proposal"),
    );

    assert_matches(&status.next, "(?i)readiness report");
    assert_no_match(&status.next, "(?i)mark.*pull request ready");
}

#[test]
fn derives_human_review_for_an_active_review_proposal() {
    assert_phase(
        &with_proposal("active-review"),
        "4",
        "Human review",
        Actor::You,
        None,
    );
}

#[test]
fn derives_revisions_for_a_returned_proposal() {
    assert_phase(
        &with_proposal("returned-for-revisions"),
        "4 → 3",
        "Human review → Implementation",
        Actor::Agent,
        Some("implementing-a-proposal"),
    );
}

#[test]
fn derives_completion_for_an_accepted_proposal() {
    assert_phase(
        &with_proposal("accepted"),
        "5",
        "Completion",
        Actor::Agent,
        Some("completing-a-feature"),
    );
}

#[test]
fn confirms_the_status_record_before_readying_an_accepted_draft_pr() {
    let status = derive_status(&with_proposal("accepted"));

    assert_eq!(status.phase, "5");
    assert_matches(
        &status.next,
        "(?i)record.*confirm.*pull request.*mark.*ready",
    );
    assert_no_match(&status.next, "(?i)perform.*merge");
}

#[test]
fn checks_a_merged_pr_before_reconciling_a_locally_accepted_proposal() {
    let input = Facts {
        pull_request: ready(PullRequestState::Merged),
        ..with_proposal("accepted")
    };
    let status = assert_phase(
        &input,
        "5",
        "Completion review",
        Actor::Agent,
        Some("completing-a-feature"),
    );

    assert_matches(&status.next, "(?i)confirm.*merged.*if absent.*reconcil");
    assert_no_match(
        &status.next,
        "(?i)perform.*merge|record.*in this feature pull request",
    );
    assert!(report(&input).contains("PR #1 (merged)"));
}

#[test]
fn keeps_merged_implementation_in_completion_while_release_remains_separate() {
    let input = Facts {
        pull_request: ready(PullRequestState::Merged),
        ..with_proposal("implemented")
    };
    let status = assert_phase(
        &input,
        "5",
        "Implementation merged",
        Actor::Agent,
        Some("completing-a-feature"),
    );

    assert!(report(&input).contains("PR #1 (merged)"));
    assert_matches(&status.next, "required release");
    assert_no_match(&status.next, "(?i)perform.*merge");
}

#[test]
fn does_not_treat_an_unpushed_implementation_record_as_merged() {
    let input = Facts {
        unpushed_commits: 1,
        pull_request: ready(PullRequestState::Merged),
        ..with_proposal("implemented")
    };
    let status = derive_status(&input);
    let text = report(&input);

    assert_eq!(status.phase, "5");
    assert!(text.contains("Unpushed commits: 1"), "{text}");
    assert_matches(&text, "(?i)confirm.*implemented record.*merged");
}

#[test]
fn does_not_claim_integration_when_a_pr_merged_into_a_non_default_branch() {
    let mut pull_request = open_pull_request();
    pull_request.is_draft = false;
    pull_request.state = PullRequestState::Merged;
    pull_request.base_ref_name = "staging".into();
    let input = Facts {
        pull_request: PullRequestLookup::Found(pull_request),
        ..with_proposal("implemented")
    };
    let status = derive_status(&input);

    assert_eq!(status.phase, "5");
    assert_eq!(status.actor, Actor::You);
    assert_matches(&status.next, "(?i)staging.*main");
    assert_ne!(status.title, "Implementation merged");
}

#[test]
fn does_not_claim_integration_when_the_target_branch_cannot_be_established() {
    let input = Facts {
        default_branch: None,
        pull_request: ready(PullRequestState::Merged),
        ..with_proposal("implemented")
    };
    let status = derive_status(&input);

    assert_eq!(status.phase, "5");
    assert_ne!(status.title, "Implementation merged");
    assert_matches(&status.next, "(?i)default branch");
}

#[test]
fn derives_a_closed_state_for_a_rejected_proposal() {
    assert_phase(
        &with_proposal("rejected"),
        "done",
        "Closed",
        Actor::You,
        None,
    );
}

#[test]
fn derives_a_closed_state_for_a_withdrawn_proposal() {
    assert_phase(
        &with_proposal("withdrawn"),
        "done",
        "Closed",
        Actor::You,
        None,
    );
}

#[test]
fn derives_a_closed_state_with_its_successor_for_a_superseded_proposal() {
    let input = Facts {
        proposal_successor: Some(summary("proposal.0002", "Replacement")),
        ..with_proposal("superseded")
    };
    assert_phase(&input, "done", "Closed", Actor::You, None);

    assert!(report(&input).contains("proposal.0002"));
}

#[test]
fn routes_an_implemented_open_pr_to_completion_instead_of_declaring_inconsistency() {
    let input = Facts {
        pull_request: ready(PullRequestState::Open),
        ..with_proposal("implemented")
    };
    let status = assert_phase(
        &input,
        "5",
        "Completion",
        Actor::Agent,
        Some("completing-a-feature"),
    );

    assert_eq!(status.notices, vec![]);
    assert_matches(
        &status.next,
        "(?i)confirm.*implemented record.*pull request.*before.*merge",
    );
}

#[test]
fn confirms_the_status_record_before_readying_an_implemented_draft_pr() {
    let status = derive_status(&with_proposal("implemented"));

    assert_eq!(status.phase, "5");
    assert_matches(
        &status.next,
        "(?i)confirm.*implemented record.*pull request.*mark.*ready",
    );
    assert_no_match(&status.next, "(?i)perform.*merge");
}

#[test]
fn stops_for_a_human_decision_when_the_proposals_pr_closes_without_merging() {
    for proposal_status in ["accepted", "implemented"] {
        let input = Facts {
            pull_request: ready(PullRequestState::Closed),
            ..with_proposal(proposal_status)
        };
        let status = derive_status(&input);

        assert_eq!(status.phase, "5", "{proposal_status}");
        assert_eq!(status.actor, Actor::You, "{proposal_status}");
        assert_matches(&status.next, "reopen|withdraw");
        assert!(report(&input).contains("PR #1 (closed)"));
    }
}

#[test]
fn flags_a_draft_pull_request_when_the_proposal_is_in_active_review() {
    let status = derive_status(&with_proposal("active-review"));

    assert_eq!(status.notices, vec![Notice::DraftActiveReview]);
}

#[test]
fn surfaces_a_heuristic_proposal_selection_and_its_other_candidates() {
    let mut selected = proposal("draft");
    selected.pull_request = Some("2".into());
    let input = Facts {
        proposal: Some(selected),
        proposal_selection: ProposalSelection {
            other_candidates: vec![summary("proposal.0002", "Other feature")],
            pull_request_mismatch: true,
        },
        ..facts()
    };
    let status = derive_status(&input);

    assert_eq!(
        status.notices,
        vec![Notice::AmbiguousProposalSelection {
            selected: Some("proposal.0001".into()),
            other_candidates: vec![summary("proposal.0002", "Other feature")],
            pull_request_mismatch: true,
            proposal_pull_request: Some("2".into()),
            pull_request_number: Some(1),
        }]
    );
    assert!(report(&input).contains("proposal.0002"));
}

#[test]
fn surfaces_unresolved_clarification_markers_and_unpushed_commits() {
    let mut draft = proposal("draft");
    draft.clarification_count = 2;
    let input = Facts {
        proposal: Some(draft),
        unpushed_commits: 3,
        ..facts()
    };

    assert_eq!(
        derive_status(&input).notices,
        vec![
            Notice::Clarifications { count: 2 },
            Notice::UnpushedCommits { count: 3 }
        ]
    );
}

#[test]
fn reports_an_unavailable_pull_request_state_instead_of_assuming_no_pull_request() {
    let input = Facts {
        pull_request: PullRequestLookup::Unknown {
            reason: "gh is unavailable".into(),
        },
        ..facts()
    };
    let status = assert_phase(&input, "1", "Preparation", Actor::Agent, None);

    assert_eq!(status.reason.as_deref(), Some("gh is unavailable"));
}

#[test]
fn reports_an_unauthenticated_pull_request_state() {
    let input = Facts {
        pull_request: PullRequestLookup::Unknown {
            reason: "gh is not authenticated".into(),
        },
        ..facts()
    };
    let status = assert_phase(&input, "1", "Preparation", Actor::Agent, None);

    assert_eq!(status.reason.as_deref(), Some("gh is not authenticated"));
}

#[test]
fn reports_an_unknown_branch_role_instead_of_recommending_a_pull_request() {
    let input = Facts {
        pull_request: PullRequestLookup::None,
        is_default_branch: None,
        ..facts()
    };
    assert_phase(&input, "unknown", "Branch role unknown", Actor::Agent, None);
}

#[test]
fn takes_the_default_branch_from_origin_head_before_anything_else() {
    let found = default_branch_facts(
        Some("feature"),
        Some("refs/remotes/origin/trunk"),
        Some("main"),
        Some("main"),
    );
    assert_eq!(found.default_branch.as_deref(), Some("trunk"));
    assert_eq!(found.is_default_branch, Some(false));
}

#[test]
fn recognizes_a_local_default_branch_when_origin_head_and_gh_are_unavailable() {
    let feature = default_branch_facts(Some("feature/x"), None, None, Some("main"));
    assert_eq!(feature.default_branch.as_deref(), Some("main"));
    assert_eq!(feature.is_default_branch, Some(false));

    let main = default_branch_facts(Some("main"), None, None, Some("main"));
    assert_eq!(main.default_branch.as_deref(), Some("main"));
    assert_eq!(main.is_default_branch, Some(true));
}

#[test]
fn gives_a_branch_cut_from_a_local_main_the_phase_1_advice_not_an_unknown_role() {
    let found = default_branch_facts(Some("feature/x"), None, None, Some("main"));
    let input = Facts {
        branch: Some("feature/x".into()),
        is_default_branch: found.is_default_branch,
        pull_request: PullRequestLookup::None,
        ..facts()
    };
    assert_phase(&input, "1", "Preparation", Actor::Agent, None);
}

#[test]
fn leaves_the_branch_role_unknown_when_no_default_branch_can_be_found() {
    let found = default_branch_facts(Some("feature/x"), None, None, None);
    assert_eq!(found.default_branch, None);
    assert_eq!(found.is_default_branch, None);
}

#[test]
fn reserves_unknown_routing_for_an_invalid_proposal_status() {
    assert_phase(&with_proposal("invalid"), "2", "Proposal", Actor::You, None);
}
