//! Each kind's required frontmatter and allowed statuses.

use workbench::commands::records::validate::validate;

use super::fixtures::{
    artifact, list, proposal_artifact, scalar, set, valid_artifacts, violation,
    violations_mentioning,
};

/// Also covers: every required key and allowed status is accepted, IDs matching unique filenames
/// are accepted, resolved references are accepted, and historical proposal supersession is valid.
#[test]
fn a_valid_record_set_has_no_violations() {
    assert_eq!(validate(&valid_artifacts()), vec![]);
}

#[test]
fn missing_required_keys_and_invalid_statuses_are_violations() {
    let mut records = valid_artifacts();
    records[0] = artifact(
        "proposals/0001-records.md",
        &[
            ("id", scalar("proposal.0001")),
            ("authors", list(&["Ada"])),
            ("status", scalar("retired")),
            ("pull-request", scalar("42")),
            ("supersedes", list(&[])),
        ],
        "",
    );
    set(&mut records[1], "supersedes", list(&[]));

    assert_eq!(
        validate(&records),
        vec![
            violation(
                "proposals/0001-records.md",
                r#"missing required key "title""#
            ),
            violation(
                "proposals/0001-records.md",
                r#"invalid status "retired" for proposal"#
            ),
        ]
    );
}

#[test]
fn a_proposal_requires_authors() {
    let mut missing = valid_artifacts();
    missing[0].frontmatter.remove("authors");
    assert_eq!(
        violations_mentioning(&missing, "authors"),
        vec![violation(
            "proposals/0001-records.md",
            r#"missing required key "authors""#
        )]
    );

    let mut present = valid_artifacts();
    set(&mut present[0], "authors", list(&["Ada"]));
    assert_eq!(violations_mentioning(&present, "authors"), vec![]);
}

#[test]
fn a_proposal_rejects_an_empty_authors_list() {
    let mut records = valid_artifacts();
    set(&mut records[0], "authors", list(&[]));
    assert_eq!(
        violations_mentioning(&records, "authors"),
        vec![violation(
            "proposals/0001-records.md",
            r#"key "authors" must be a non-empty inline list"#
        )]
    );
}

#[test]
fn each_kind_accepts_its_newer_statuses() {
    for (kind, index, status) in [
        ("proposal", 1, "rejected"),
        ("proposal", 1, "withdrawn"),
        ("policy", 2, "draft"),
        ("decision", 3, "proposed"),
    ] {
        let mut records = valid_artifacts();
        set(&mut records[index], "status", scalar(status));
        let message = format!(r#"invalid status "{status}" for {kind}"#);
        assert_eq!(
            violations_mentioning(&records, &message),
            vec![],
            "{kind} status {status}"
        );
    }
}

#[test]
fn a_proposal_accepts_every_lifecycle_status() {
    for status in [
        "draft",
        "awaiting-implementation",
        "active-review",
        "returned-for-revisions",
        "accepted",
        "implemented",
        "rejected",
        "withdrawn",
        "superseded",
    ] {
        assert_eq!(
            validate(&[proposal_artifact(status, "")]),
            vec![],
            "proposal status {status}"
        );
    }
}

#[test]
fn a_status_from_another_kind_is_invalid() {
    for (kind, index, status) in [
        ("proposal", 0, "active"),
        ("proposal", 0, "proposed"),
        ("policy", 2, "implemented"),
        ("policy", 2, "awaiting-implementation"),
        ("decision", 3, "awaiting-implementation"),
    ] {
        let mut records = valid_artifacts();
        set(&mut records[index], "status", scalar(status));
        let message = format!(r#"invalid status "{status}" for {kind}"#);
        assert_eq!(
            violations_mentioning(&records, &message),
            vec![violation(&records[index].path, &message)],
            "{kind} status {status}"
        );
    }
}

#[test]
fn a_proposal_accepts_an_absent_empty_or_populated_issues_list() {
    let populated = list(&["#12", "https://github.com/example/my-project/issues/34"]);
    for issues in [None, Some(list(&[])), Some(populated)] {
        let mut records = valid_artifacts();
        if let Some(issues) = issues.clone() {
            set(&mut records[0], "issues", issues);
        }
        assert_eq!(
            violations_mentioning(&records, "issues"),
            vec![],
            "issues {issues:?}"
        );
    }
}

#[test]
fn proposal_issues_must_be_an_inline_list() {
    let mut records = valid_artifacts();
    set(&mut records[0], "issues", scalar("#12"));
    assert_eq!(
        violations_mentioning(&records, "issues"),
        vec![violation(
            "proposals/0001-records.md",
            r#"key "issues" must be an inline list"#
        )]
    );
}
