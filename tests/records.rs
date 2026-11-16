//! `workbench records validate`: which files it reads, and how it reports what it finds.
//! The rules themselves are checked against in-memory records in `records/`.

mod support;

mod records {
    mod clarification;
    mod current;
    mod fixtures;
    mod frontmatter;
    mod references;
    mod schema;
    mod templates;
}

use std::fs;
use std::os::unix::fs::symlink;

use serde_json::json;
use support::{Fixture, assert_matches_output_schema};

const VALIDATE: &[&str] = &["records", "validate"];

fn proposal_source(id: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: Record Validation\nauthors: [Ada]\nstatus: draft\n\
         pull-request: 42\nsupersedes: []\n---\n"
    )
}

/// A project whose records are valid: READMEs to skip, an empty `decisions/`, no `policies/`.
fn valid_project() -> Fixture {
    let project = Fixture::git();
    project
        .write(
            "VISION.md",
            "---\ntitle: Example Vision\nupdated: 2026-11-15\n---\n",
        )
        .write("proposals/README.md", "# Proposals\n")
        .write(
            "proposals/0001-records.md",
            &proposal_source("proposal.0001"),
        )
        .write("guides/README.md", "# Guides\n")
        .write(
            "guides/getting-started.md",
            "---\nid: guide.getting-started\ntitle: Getting Started\n---\n",
        );
    fs::create_dir(project.path("decisions")).expect("an empty record directory");
    project
}

/// `valid_project` plus a policy missing its title with a status no policy has.
fn project_with_violations() -> Fixture {
    let project = valid_project();
    project.write(
        "policies/0001-rule.md",
        "---\nid: policy.0001\nstatus: retired\n---\n",
    );
    project
}

#[test]
fn valid_records_report_no_violations() {
    let outcome = valid_project().run(VALIDATE);

    assert_eq!(outcome.status, 0, "{outcome:?}");
    assert_eq!(outcome.json(), json!({ "violations": [], "count": 0 }));
    assert_eq!(outcome.stderr, "");
    assert_matches_output_schema(VALIDATE, &outcome.json());
}

#[test]
fn violations_go_to_stdout_and_the_check_fails() {
    let outcome = project_with_violations().run(VALIDATE);

    assert_eq!(outcome.status, 1, "{outcome:?}");
    assert_eq!(
        outcome.json(),
        json!({
            "violations": [
                { "path": "policies/0001-rule.md", "message": r#"missing required key "title""# },
                { "path": "policies/0001-rule.md", "message": r#"invalid status "retired" for policy"# },
            ],
            "count": 2,
        })
    );
    assert_eq!(outcome.error_code(), "CHECK_FAILED");
    assert_matches_output_schema(VALIDATE, &outcome.json());
}

#[test]
fn paths_are_relative_to_the_project_root_from_any_subdirectory() {
    let outcome = project_with_violations().run_in("guides", VALIDATE);

    assert_eq!(outcome.status, 1, "{outcome:?}");
    assert_eq!(outcome.json()["count"], 2);
    assert_eq!(
        outcome.json()["violations"][0]["path"],
        "policies/0001-rule.md"
    );
}

#[test]
fn text_output_lists_each_violation_then_the_count() {
    let outcome = project_with_violations().run(&["records", "validate", "--output", "text"]);

    assert_eq!(outcome.status, 1, "{outcome:?}");
    assert_eq!(
        outcome.stdout,
        "policies/0001-rule.md: missing required key \"title\"\n\
         policies/0001-rule.md: invalid status \"retired\" for policy\n\
         2 violations\n"
    );
    assert!(
        outcome.stderr.contains("CHECK_FAILED"),
        "{}",
        outcome.stderr
    );

    let valid = valid_project().run(&["records", "validate", "--output", "text"]);
    assert_eq!((valid.status, valid.stdout.as_str()), (0, "0 violations\n"));
}

#[test]
fn unparsable_frontmatter_is_reported_at_its_file() {
    let project = valid_project();
    project
        .write("proposals/0002-plain.md", "# No frontmatter\n")
        .write("proposals/0003-open.md", "---\ntitle: Open\n")
        .write("proposals/0004-twice.md", "---\ntitle: A\ntitle: B\n---\n")
        .write("guides/broken-list.md", "---\nauthors: [a, , b]\n---\n");

    let outcome = project.run(VALIDATE);

    assert_eq!(outcome.status, 1, "{outcome:?}");
    assert_eq!(
        outcome.json()["violations"],
        json!([
            { "path": "proposals/0002-plain.md", "message": "missing YAML frontmatter" },
            { "path": "proposals/0003-open.md", "message": "unterminated YAML frontmatter" },
            { "path": "proposals/0004-twice.md", "message": r#"duplicate frontmatter key "title""# },
            { "path": "guides/broken-list.md", "message": r#"cannot parse list value for "authors""# },
        ])
    );
}

#[test]
fn symlinked_files_are_checked_but_symlinked_directories_are_not_traversed() {
    let project = Fixture::git();
    project
        .write("record.md", &proposal_source("proposal.9999"))
        .write("nested/hidden.md", &proposal_source("proposal.0001"));
    fs::create_dir(project.path("proposals")).expect("a proposals directory");
    symlink(
        project.path("record.md"),
        project.path("proposals/0001-records.md"),
    )
    .expect("a symlinked record");
    symlink(project.path("nested"), project.path("proposals/nested"))
        .expect("a symlinked directory");

    let outcome = project.run(VALIDATE);

    assert_eq!(
        outcome.json(),
        json!({
            "violations": [{
                "path": "proposals/0001-records.md",
                "message": r#"ID "proposal.9999" must be "proposal.0001" for this filename"#,
            }],
            "count": 1,
        })
    );
}

#[test]
fn markdown_nested_in_a_record_directory_is_a_violation() {
    let project = Fixture::git();
    project.write(
        "proposals/drafts/0001-record.md",
        &proposal_source("proposal.0001"),
    );

    let outcome = project.run(VALIDATE);

    assert_eq!(
        outcome.json(),
        json!({
            "violations": [{
                "path": "proposals/drafts/0001-record.md",
                "message": "nested record file; move it to proposals/ or remove it",
            }],
            "count": 1,
        })
    );
}

#[test]
fn an_unreadable_record_is_a_precondition_failure() {
    let project = Fixture::git();
    fs::create_dir(project.path("VISION.md")).expect("a directory where VISION.md belongs");

    let outcome = project.run(VALIDATE);

    assert_eq!(outcome.status, 3, "{outcome:?}");
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert_eq!(outcome.error()["details"]["path"], "VISION.md");
}

#[test]
fn outside_a_git_repository_is_a_precondition_failure() {
    let outcome = Fixture::empty().run(VALIDATE);

    assert_eq!(outcome.status, 3, "{outcome:?}");
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
}

#[test]
fn without_git_the_dependency_is_missing() {
    let mut project = valid_project();
    let bin_only = project.path(".fixture-bin").display().to_string();
    project.set_env("PATH", &bin_only);

    let outcome = project.run(VALIDATE);

    assert_eq!(outcome.status, 3, "{outcome:?}");
    assert_eq!(outcome.error_code(), "DEPENDENCY_MISSING");
}
