//! `workbench commits check --message-file`: the subject a commit-msg hook is about to commit.

use super::*;

#[test]
fn the_subject_is_the_first_line_that_is_neither_blank_nor_a_comment() {
    let fixture = Fixture::git();
    for message in [
        "docs: add a guide\n",
        "\n\n#  a comment\n  docs: add a guide  \n\nbody\n",
        "docs: add a guide\r\nbody\r\n",
    ] {
        let subject = assert_passes(&fixture, message);
        assert_eq!(subject["subject"], "docs: add a guide", "{message:?}");
        assert_eq!(subject["verdict"], "accepted");
    }
}

#[test]
fn a_message_with_only_comments_or_nothing_has_no_subject_and_passes() {
    let fixture = Fixture::git();
    for message in ["# only comments\n\n", ""] {
        let subject = assert_passes(&fixture, message);
        assert_eq!(subject["verdict"], "noSubject", "{message:?}");
        assert!(subject.get("subject").is_none(), "{subject}");
    }
}

#[test]
fn the_scope_is_read_only_from_a_well_formed_subject() {
    let fixture = Fixture::git();
    fixture.write("proposals/0007-export-queue.md", "");
    assert_eq!(
        assert_passes(&fixture, "docs: add a guide\n")["scope"],
        "docs"
    );
    assert_eq!(
        assert_passes(&fixture, "0007-export-queue: reject duplicates\n")["scope"],
        "0007-export-queue"
    );
    for malformed in ["docs:add a guide\n", "docs: \n", "no colon here\n"] {
        assert_rejected(&fixture, malformed, "unscoped");
        assert!(
            checked(&check_message(&fixture, malformed))
                .get("scope")
                .is_none()
        );
    }
}

#[test]
fn the_scopes_file_ignores_blank_lines_and_comments_and_trims_the_rest() {
    let fixture = Fixture::git();
    fixture.write(
        ".agents/commit-scopes",
        "# header\n\n  widgets  \ngadgets\n\n#tail\n",
    );
    assert_passes(&fixture, "widgets: add a widget\n");
    assert_passes(&fixture, "gadgets: add a gadget\n");
    let scopes = accepted_scopes(&check_message(&fixture, "widgets: x\n"));
    assert!(scopes.contains(&"widgets".to_string()), "{scopes:?}");
    assert!(
        !scopes
            .iter()
            .any(|scope| scope.is_empty() || scope.starts_with('#')),
        "{scopes:?}"
    );

    fixture.write(".agents/commit-scopes", "");
    assert_eq!(
        accepted_scopes(&check_message(&fixture, "docs: x\n")),
        BUILT_IN_SCOPES
    );
}

#[test]
fn a_scope_the_scopes_file_lists_passes() {
    let fixture = Fixture::git();
    fixture.write(".agents/commit-scopes", "cli\n");
    assert_passes(&fixture, "cli: add getting started guide\n");
}

#[test]
fn proposal_is_a_scope_only_where_the_scopes_file_lists_it() {
    let fixture = Fixture::git();
    fixture.write("proposals/0007-export-queue.md", "");
    assert_rejected(
        &fixture,
        "proposal: draft the export queue\n",
        "unknownScope",
    );
    assert_passes(&fixture, "0007-export-queue: draft the proposal\n");

    fixture.write(".agents/commit-scopes", "proposal\n");
    assert_passes(&fixture, "proposal: number proposals from one\n");
}

#[test]
fn the_built_in_process_scopes_need_no_scopes_file() {
    let fixture = Fixture::git();
    for scope in BUILT_IN_SCOPES {
        assert_passes(&fixture, &format!("{scope}: change something\n"));
    }
    assert_eq!(
        accepted_scopes(&check_message(&fixture, "docs: x\n")),
        BUILT_IN_SCOPES
    );
}

#[test]
fn a_subject_without_a_scope_fails() {
    let violation = assert_rejected(&Fixture::git(), "Added stuff\n", "unscoped");
    assert_eq!(violation["subject"], "Added stuff");
    assert!(
        violation["message"]
            .as_str()
            .unwrap()
            .contains("must be a Scoped Commit"),
        "{violation}"
    );
}

#[test]
fn conventional_commits_subjects_are_rejected_by_name() {
    let fixture = Fixture::git();
    for subject in ["feat(api): add thing\n", "feat(api)!: add thing\n"] {
        let violation = assert_rejected(&fixture, subject, "conventionalCommit");
        assert!(
            violation["message"]
                .as_str()
                .unwrap()
                .contains("Conventional Commits 'type(scope):' subjects are not allowed"),
            "{violation}"
        );
    }
}

#[test]
fn an_unknown_scope_is_rejected_with_every_accepted_scope() {
    let fixture = Fixture::git();
    fixture.write(".agents/commit-scopes", "cli\n");
    fixture.write("proposals/0007-export-queue.md", "");
    let violation = assert_rejected(&fixture, "nope: do a thing\n", "unknownScope");
    let message = violation["message"].as_str().unwrap();
    assert!(message.contains("scope 'nope' is not allowed"), "{message}");

    let outcome = check_message(&fixture, "nope: do a thing\n");
    let error = outcome.error()["message"].as_str().unwrap().to_string();
    for scope in BUILT_IN_SCOPES
        .into_iter()
        .chain(["cli", "0007-export-queue"])
    {
        assert!(message.contains(scope), "{scope} missing from {message}");
        assert!(error.contains(scope), "{scope} missing from {error}");
    }
}

#[test]
fn mechanical_subjects_are_exempt() {
    let fixture = Fixture::git();
    for subject in [
        "Merge branch main",
        "merge: branch main",
        "Revert \"docs: add a guide\"",
        "revert: docs: add a guide",
        "fixup! docs: add a guide",
        "squash! docs: add a guide",
    ] {
        let checked = assert_passes(&fixture, &format!("{subject}\n"));
        assert_eq!(checked["verdict"], "exempt", "{subject}");
    }
}

#[test]
fn a_scope_that_merely_starts_with_a_mechanical_word_is_not_exempt() {
    let violation = assert_rejected(&Fixture::git(), "merged: do a thing\n", "unknownScope");
    assert!(
        violation["message"]
            .as_str()
            .unwrap()
            .contains("scope 'merged' is not allowed")
    );
}

#[test]
fn a_proposal_scope_is_accepted_only_while_its_proposal_exists() {
    let fixture = Fixture::git();
    let subject = "0007-export-queue: reject duplicates\n";
    assert_rejected(&fixture, subject, "unknownScope");

    fixture.write("proposals/0007-export-queue.md", "");
    fixture.write("proposals/README.md", "# Proposals\n");
    assert_passes(&fixture, subject);
    let scopes = accepted_scopes(&check_message(&fixture, subject));
    assert!(scopes.contains(&"0007-export-queue".to_string()));
    assert!(!scopes.iter().any(|scope| scope.contains("README")));
}

#[test]
fn the_project_root_supplies_the_scopes_from_a_subdirectory() {
    let fixture = Fixture::git();
    fixture.write(".agents/commit-scopes", "widgets\n");
    fixture.write("src/.keep", "");
    fixture.write("MESSAGE", "widgets: add a widget\n");
    let outcome = fixture.run_in("src", &["commits", "check", "--message-file", "../MESSAGE"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
}

#[test]
fn a_missing_message_file_is_not_found() {
    let outcome = Fixture::git().run(&["commits", "check", "--message-file", "absent"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
    assert!(outcome.stdout.is_empty());
}

#[test]
fn exactly_one_form_must_be_given() {
    let fixture = Fixture::git();
    fixture.write("MESSAGE", "docs: x\n");
    for args in [
        &["commits", "check"][..],
        &[
            "commits",
            "check",
            "--message-file",
            "MESSAGE",
            "main",
            "main",
        ][..],
        &["commits", "check", "--message-file", "MESSAGE", "main"][..],
    ] {
        let outcome = fixture.run(args);
        assert_eq!(outcome.status, 2, "{args:?}: {}", outcome.stderr);
        assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    }

    let outcome = fixture.run(&["commits", "check", "main"]);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(outcome.error()["details"]["missing"][0], "head");
}

#[test]
fn text_output_names_each_rejected_subject() {
    let fixture = Fixture::git();
    fixture.write("MESSAGE", "nope: do a thing\n");
    let outcome = fixture.run(&[
        "commits",
        "check",
        "--message-file",
        "MESSAGE",
        "--output",
        "text",
    ]);
    assert_eq!(outcome.status, 1);
    assert!(
        outcome.stdout.contains("nope: do a thing"),
        "{}",
        outcome.stdout
    );
    assert!(outcome.stdout.contains("scope 'nope' is not allowed"));
}
