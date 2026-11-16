//! `workbench commits check <base> <head>`: every commit a range adds.

use super::*;

/// A repository whose `feature` branch adds one commit per subject on top of `main`.
fn branch_with(subjects: &[&str]) -> Fixture {
    let fixture = Fixture::git();
    fixture.git_command(&["checkout", "--quiet", "-b", "feature"]);
    for subject in subjects {
        fixture.commit_all(subject);
    }
    fixture
}

#[test]
fn a_range_of_scoped_commits_passes_and_lists_each_commit() {
    let fixture = branch_with(&["docs: add a guide", "rules: tighten a rule"]);
    let outcome = fixture.run(&["commits", "check", "main", "feature"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let result = outcome.json();
    assert_matches_output_schema(&["commits", "check"], &result);
    let commits = result["commits"].as_array().unwrap();
    let subjects: Vec<&str> = commits
        .iter()
        .map(|commit| commit["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["docs: add a guide", "rules: tighten a rule"]);
    let head = fixture.git_command(&["rev-parse", "feature"]);
    assert_eq!(commits[1]["commit"], head.trim());
    assert_eq!(result["violations"], serde_json::json!([]));
}

#[test]
fn a_range_reports_every_rejected_subject_not_only_the_first() {
    let fixture = branch_with(&[
        "docs: add a guide",
        "Added stuff",
        "nope: do a thing",
        "fixup! docs: add a guide",
    ]);
    let outcome = fixture.run(&["commits", "check", "main", "feature"]);
    assert_eq!(outcome.status, 1);
    assert_eq!(outcome.error_code(), "CHECK_FAILED");
    let result = outcome.json();
    assert_matches_output_schema(&["commits", "check"], &result);
    assert_eq!(result["commits"].as_array().unwrap().len(), 4);
    let rules: Vec<&str> = result["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect();
    assert_eq!(rules, ["unscoped", "unknownScope"]);
    assert!(
        result["violations"][0]["commit"]
            .as_str()
            .is_some_and(|commit| commit.len() == 40)
    );
}

#[test]
fn an_unknown_revision_is_not_found() {
    let outcome = Fixture::git().run(&["commits", "check", "main", "no-such-branch"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
    assert_eq!(outcome.error()["details"]["revision"], "no-such-branch");
}

#[test]
fn a_commit_with_an_empty_message_has_no_subject_to_check() {
    let fixture = branch_with(&[]);
    fixture.git_command(&[
        "commit",
        "--quiet",
        "--allow-empty",
        "--allow-empty-message",
        "--message",
        "",
    ]);
    let outcome = fixture.run(&["commits", "check", "main", "feature"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert_eq!(outcome.json()["commits"][0]["verdict"], "noSubject");
}
