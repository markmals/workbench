//! `workbench change-files create`: writing a change file without Knope's prompts.

use super::*;

#[test]
fn create_writes_a_change_file_named_from_the_summary() {
    let fixture = project(Some(SINGLE_PACKAGE));
    let outcome = create(
        &fixture,
        &["--type", "minor", "--summary", "Add the export queue"],
    );
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_matches_output_schema(&["change-files", "create"], &receipt);
    assert_eq!(
        receipt,
        json!({
            "status": "created",
            "project": canonical_root(&fixture),
            "path": ".changeset/add_the_export_queue.md",
            "type": "minor",
            "package": "default",
        })
    );
    assert_eq!(
        fixture.read(".changeset/add_the_export_queue.md"),
        VALID_CHANGE
    );
}

#[test]
fn file_names_follow_knopes_naming() {
    let fixture = project(Some(SINGLE_PACKAGE));
    for (summary, name) in [
        (
            "`[i carry your_heart with-me(i carry it in]`",
            "i_carry_your_heart_with_mei_carry_it_in",
        ),
        ("This is a Title", "this_is_a_title"),
        ("Something ______ else", "something_else"),
    ] {
        let outcome = create(&fixture, &["--type", "patch", "--summary", summary]);
        assert_eq!(outcome.status, 0, "{}", outcome.stderr);
        let path = format!(".changeset/{name}.md");
        assert_eq!(outcome.json()["path"], path.as_str());
        assert!(fixture.read(&path).contains(&format!("\n# {summary}\n")));
    }
}

#[test]
fn the_body_follows_the_heading_as_text_or_from_a_file() {
    let fixture = project(Some(SINGLE_PACKAGE));
    create(
        &fixture,
        &[
            "--type",
            "patch",
            "--summary",
            "Fix export",
            "--body",
            "It no longer drops rows.",
        ],
    );
    assert_eq!(
        fixture.read(".changeset/fix_export.md"),
        "---\ndefault: patch\n---\n\n# Fix export\n\nIt no longer drops rows.\n"
    );

    fixture.write("notes.md", "Line one.\n\nLine two.\n\n");
    create(
        &fixture,
        &[
            "--type",
            "patch",
            "--summary",
            "Fix import",
            "--body",
            "@notes.md",
        ],
    );
    assert_eq!(
        fixture.read(".changeset/fix_import.md"),
        "---\ndefault: patch\n---\n\n# Fix import\n\nLine one.\n\nLine two.\n"
    );
}

#[test]
fn the_type_must_be_one_knope_accepts_for_the_package() {
    let fixture = project(Some(SINGLE_PACKAGE));
    let outcome = create(&fixture, &["--type", "huge", "--summary", "Huge"]);
    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(
        outcome.error()["details"]["acceptedValues"],
        json!(["major", "minor", "patch", "note"])
    );
    assert!(!fixture.path(".changeset/huge.md").exists());

    let outcome = create(&fixture, &["--type", "note", "--summary", "A note"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
}

#[test]
fn a_multi_package_project_requires_one_of_its_packages() {
    let fixture = project(Some(TWO_PACKAGES));
    let outcome = create(&fixture, &["--type", "minor", "--summary", "Add"]);
    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    let details = outcome.error()["details"].clone();
    assert_eq!(details["missing"], json!(["package"]));
    assert_eq!(details["acceptedValues"], json!(["api", "web"]));

    let outcome = create(
        &fixture,
        &["--type", "minor", "--summary", "Add", "--package", "nope"],
    );
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(
        outcome.error()["details"]["acceptedValues"],
        json!(["api", "web"])
    );

    let outcome = create(
        &fixture,
        &["--type", "poem", "--summary", "A poem", "--package", "api"],
    );
    assert_eq!(
        outcome.error_code(),
        "INVALID_ARGUMENT",
        "poem is web's type only"
    );

    let outcome = create(
        &fixture,
        &["--type", "poem", "--summary", "A poem", "--package", "web"],
    );
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert_eq!(outcome.json()["package"], "web");
    assert_eq!(
        fixture.read(".changeset/a_poem.md"),
        "---\nweb: poem\n---\n\n# A poem\n"
    );
}

#[test]
fn create_without_knope_toml_is_a_precondition_failure() {
    let fixture = project(None);
    let outcome = create(&fixture, &["--type", "minor", "--summary", "Add"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(
        !outcome.error()["suggestions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!fixture.path(".changeset").exists());
}

#[test]
fn create_never_overwrites_a_change_file() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(
        ".changeset/add_the_export_queue.md",
        "---\ndefault: major\n---\n\n# Mine\n",
    );
    let outcome = create(
        &fixture,
        &["--type", "minor", "--summary", "Add the export queue"],
    );
    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(
        outcome.error()["details"]["path"],
        ".changeset/add_the_export_queue.md"
    );
    assert_eq!(
        fixture.read(".changeset/add_the_export_queue.md"),
        "---\ndefault: major\n---\n\n# Mine\n"
    );
}

#[test]
fn a_dry_run_describes_the_file_without_writing_it() {
    let fixture = project(Some(SINGLE_PACKAGE));
    let outcome = create(
        &fixture,
        &[
            "--type",
            "minor",
            "--summary",
            "Add the export queue",
            "--dry-run",
        ],
    );
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_matches_output_schema(&["change-files", "create"], &receipt);
    assert_eq!(
        receipt,
        json!({
            "status": "dry-run",
            "project": canonical_root(&fixture),
            "path": ".changeset/add_the_export_queue.md",
            "type": "minor",
            "package": "default",
            "content": VALID_CHANGE,
            "cannotPredict": [],
        })
    );
    assert!(!fixture.path(".changeset").exists());
}

#[test]
fn a_dry_run_validates_like_a_real_run() {
    let fixture = project(Some(SINGLE_PACKAGE));
    let outcome = create(
        &fixture,
        &["--type", "huge", "--summary", "Huge", "--dry-run"],
    );
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(
        outcome.error()["details"]["acceptedValues"],
        json!(["major", "minor", "patch", "note"])
    );

    fixture.write(".changeset/taken.md", VALID_CHANGE);
    let outcome = create(
        &fixture,
        &["--type", "minor", "--summary", "Taken", "--dry-run"],
    );
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(outcome.error()["details"]["path"], ".changeset/taken.md");
}

#[test]
fn a_summary_is_one_line_with_a_letter_or_digit() {
    let fixture = project(Some(SINGLE_PACKAGE));
    for summary in ["!!!", "Two\nlines", "   "] {
        let outcome = create(&fixture, &["--type", "minor", "--summary", summary]);
        assert_eq!(outcome.status, 2, "{summary:?}");
        assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
        assert_eq!(outcome.error()["details"]["argument"], "summary");
    }
}

#[test]
fn missing_input_lists_the_missing_arguments() {
    let outcome = create(&project(Some(SINGLE_PACKAGE)), &["--summary", "Add"]);
    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert!(outcome.error()["details"].to_string().contains("type"));
}

#[test]
fn a_created_change_file_satisfies_the_check() {
    let fixture = project(Some(TWO_PACKAGES));
    let outcome = create(
        &fixture,
        &[
            "--type",
            "minor",
            "--summary",
            "Add the API",
            "--package",
            "api",
        ],
    );
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    fixture.commit_all("docs: describe the API");
    assert_check_passes(&fixture);
}
