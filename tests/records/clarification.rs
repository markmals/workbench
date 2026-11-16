//! Unresolved clarification markers in proposal prose.

use workbench::commands::records::validate::Violation;
use workbench::frontmatter::without_code;

use super::fixtures::{
    proposal_artifact, scalar, set, valid_artifacts, violation, violations_mentioning,
};

const MARKER: &str = "NEEDS CLARIFICATION";
const QUESTION: &str = "[NEEDS CLARIFICATION: What is the release boundary?]";

/// Whether an accepted proposal with `content` is reported for a clarification marker.
fn reports_marker(content: &str) -> bool {
    !violations_mentioning(&[proposal_artifact("accepted", content)], MARKER).is_empty()
}

/// Marker-shaped code in `content` is ignored, but a prose question after it is reported.
fn assert_code_ignored_but_prose_reported(content: &str) {
    assert!(!reports_marker(content), "code reported:\n{content}");
    let with_prose = format!("{content}\n\n[NEEDS CLARIFICATION: What remains unresolved?]");
    assert!(reports_marker(&with_prose), "prose missed:\n{with_prose}");
}

/// The replacement proposal with `status` and `content`, among a valid record set.
fn second_proposal_reports(status: &str, content: &str) -> Vec<Violation> {
    let mut records = valid_artifacts();
    set(&mut records[1], "status", scalar(status));
    records[1].content = content.to_string();
    violations_mentioning(&records, MARKER)
}

#[test]
fn a_final_proposal_rejects_a_clarification_marker() {
    for status in [
        "awaiting-implementation",
        "active-review",
        "accepted",
        "implemented",
        "superseded",
    ] {
        let message =
            format!(r#"proposal status "{status}" does not allow a [NEEDS CLARIFICATION:] marker"#);
        assert_eq!(
            second_proposal_reports(status, QUESTION),
            vec![violation("proposals/0002-replaces.md", &message)],
            "status {status}"
        );
    }
}

#[test]
fn an_unfinished_proposal_permits_a_clarification_marker() {
    for status in ["draft", "returned-for-revisions", "rejected", "withdrawn"] {
        assert_eq!(
            second_proposal_reports(status, QUESTION),
            vec![],
            "status {status}"
        );
    }
}

#[test]
fn a_marker_in_inline_code_is_ignored() {
    let content = "Rule 5 rejects a `[NEEDS CLARIFICATION:` marker in a final proposal.";
    assert_eq!(second_proposal_reports("accepted", content), vec![]);
}

#[test]
fn a_marker_in_a_fenced_block_is_ignored() {
    let content = "Example:\n\n```md\n- [NEEDS CLARIFICATION: how?]\n```\n";
    assert_eq!(second_proposal_reports("accepted", content), vec![]);
}

#[test]
fn a_prose_marker_beside_a_quoted_one_is_reported() {
    let content = "The `[NEEDS CLARIFICATION:` form is the convention.\n\n\
        - [NEEDS CLARIFICATION: is this genuinely unresolved?]\n";
    assert_eq!(second_proposal_reports("accepted", content).len(), 1);
}

#[test]
fn a_marker_in_a_tilde_fence_is_ignored() {
    assert_code_ignored_but_prose_reported("~~~md\n[NEEDS CLARIFICATION: This is code.]\n~~~");
}

#[test]
fn a_fence_closes_only_with_a_fence_as_long() {
    assert_code_ignored_but_prose_reported(
        "````md\n```\n[NEEDS CLARIFICATION: This is code.]\n```\n````",
    );
}

#[test]
fn an_info_string_with_a_backtick_opens_no_fence() {
    assert!(reports_marker(
        "``` `note`\n[NEEDS CLARIFICATION: This remains prose.]\n```\n"
    ));
}

#[test]
fn inline_code_on_its_own_line_opens_no_fence() {
    assert!(reports_marker(
        "`note`\n[NEEDS CLARIFICATION: This remains prose.]"
    ));
}

#[test]
fn without_code_strips_code_and_keeps_surrounding_prose() {
    assert_eq!(without_code("before `x` after"), "before  after");

    let stripped = without_code("before\n\n```\nexample\n```\n\nafter");
    assert!(!stripped.contains("example"), "code kept: {stripped}");
    assert!(stripped.starts_with("before"), "prose lost: {stripped}");
    assert!(stripped.ends_with("after"), "prose lost: {stripped}");
}
