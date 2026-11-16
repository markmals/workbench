//! `workbench change-files check`: a range adds a valid change file or carries the trailer.

use super::*;

#[test]
fn a_range_that_adds_a_valid_change_file_passes() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(".changeset/add_the_export_queue.md", VALID_CHANGE);
    fixture.commit_all("docs: describe the export queue");
    let result = assert_check_passes(&fixture);
    assert_eq!(
        result["changeFiles"],
        json!([{ "path": ".changeset/add_the_export_queue.md", "valid": true, "problems": [] }])
    );
    assert_eq!(result["releaseNoteNone"], json!([]));
}

#[test]
fn only_markdown_files_directly_in_the_changeset_directory_are_change_files() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture
        .write(".changeset/brave-moons-shout.md", VALID_CHANGE)
        .write(".changeset/README.md", "# Changesets\n")
        .write(".changeset/config.json", "{}\n")
        .write(".changeset/nested/deeper.md", VALID_CHANGE)
        .write("docs/.changeset/notes.md", VALID_CHANGE)
        .write("src/main.ts", "export {};\n");
    fixture.commit_all("docs: describe a change");
    let outcome = check(&fixture);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let paths: Vec<String> = outcome.json()["changeFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        paths,
        [".changeset/README.md", ".changeset/brave-moons-shout.md"]
    );
    assert!(
        outcome.stderr.contains(".changeset/README.md"),
        "an invalid change file is warned about: {}",
        outcome.stderr
    );
}

#[test]
fn a_range_with_neither_a_change_file_nor_the_trailer_fails_naming_both_ways() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture
        .write("README.md", "# Readme\n")
        .write("mise.toml", "");
    fixture.commit_all("docs: edit the readme");
    let result = assert_check_fails(&fixture);
    assert_eq!(result["changeFiles"], json!([]));
    let outcome = check(&fixture);
    let error = outcome.error();
    let advice = format!("{} {}", error["message"], error["suggestions"]);
    assert!(advice.contains("workbench change-files create"), "{advice}");
    assert!(advice.contains("Release-Note: none"), "{advice}");
}

#[test]
fn a_release_note_none_trailer_satisfies_the_check() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.commit_all("docs: fix a typo\n\nRelease-Note: none");
    let head = fixture.git_command(&["rev-parse", "HEAD"]);
    let result = assert_check_passes(&fixture);
    assert_eq!(result["releaseNoteNone"], json!([head.trim()]));
}

#[test]
fn the_trailer_key_is_case_insensitive_and_its_value_exactly_none() {
    let fixture = project(None);
    fixture
        .commit_all("docs: fix a typo\n\nrelease-note:   none\nSigned-off-by: A <a@example.com>");
    assert_check_passes(&fixture);

    for message in [
        "docs: fix a typo\n\nRelease-Note: None",
        "docs: fix a typo\n\nRelease-Note: none (really)",
        "Release-Note: none",
        "docs: fix a typo\n\nSome prose that mentions it\nRelease-Note: none",
    ] {
        let fixture = project(None);
        fixture.commit_all(message);
        let result = assert_check_fails(&fixture);
        assert_eq!(result["releaseNoteNone"], json!([]), "{message:?}");
    }
}

#[test]
fn a_change_file_must_name_a_knope_package_and_a_change_type_it_accepts() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(".changeset/web.md", "---\nweb: minor\n---\n\n# Web\n");
    fixture.commit_all("docs: describe a change");
    let problems = problems_of(&assert_check_fails(&fixture));
    assert!(
        problems.contains("web") && problems.contains("default"),
        "{problems}"
    );

    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(".changeset/huge.md", "---\ndefault: huge\n---\n\n# Huge\n");
    fixture.commit_all("docs: describe a change");
    let problems = problems_of(&assert_check_fails(&fixture));
    for expected in ["huge", "major", "minor", "patch", "note"] {
        assert!(problems.contains(expected), "{expected}: {problems}");
    }

    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(
        ".changeset/note.md",
        "---\n'default': note\n---\n\n# A note\n",
    );
    fixture.commit_all("docs: describe a change");
    assert_check_passes(&fixture);
}

#[test]
fn a_multi_package_project_names_its_packages_and_their_own_types() {
    let fixture = project(Some(TWO_PACKAGES));
    fixture.write(
        ".changeset/both.md",
        "---\napi: minor\n\"web\": poem\n---\n\n# Both\n",
    );
    fixture.commit_all("docs: describe a change");
    assert_check_passes(&fixture);

    for frontmatter in ["api: poem", "default: patch"] {
        let fixture = project(Some(TWO_PACKAGES));
        fixture.write(
            ".changeset/one.md",
            &format!("---\n{frontmatter}\n---\n\n# One\n"),
        );
        fixture.commit_all("docs: describe a change");
        assert_check_fails(&fixture);
    }
}

#[test]
fn without_knope_toml_any_well_formed_frontmatter_counts() {
    let fixture = project(None);
    fixture.write(
        ".changeset/custom.md",
        "---\nanything: custom type\n---\n# Custom\n",
    );
    fixture.commit_all("docs: describe a change");
    assert_check_passes(&fixture);

    for content in [
        "# No frontmatter\n",
        "---\ndefault: minor\n# never closed\n",
        "---\nno separator here\n---\n",
        "---\n---\n\n# Names no package\n",
        "---\ndefault:\n---\n",
        "",
    ] {
        let fixture = project(None);
        fixture.write(".changeset/broken.md", content);
        fixture.commit_all("docs: describe a change");
        let result = assert_check_fails(&fixture);
        assert!(!problems_of(&result).is_empty(), "{content:?}");
    }
}

/// Knope turns the summary into the changelog entry, so a file without one writes an empty
/// bullet into the release notes.
#[test]
fn a_change_file_counts_only_when_a_summary_heading_comes_first() {
    for content in [
        "---\ndefault: minor\n---\n",
        "---\ndefault: minor\n---\n\n",
        "---\ndefault: minor\n---\n\nJust a sentence, no heading.\n",
        "---\ndefault: minor\n---\n\nA sentence first.\n\n# Then a heading\n",
        "---\ndefault: minor\n---\n\n#\n",
        "---\ndefault: minor\n---\n\n#   \n",
        "---\ndefault: minor\n---\n\n#No space\n",
    ] {
        let fixture = project(Some(SINGLE_PACKAGE));
        fixture.write(".changeset/unsummarized.md", content);
        fixture.commit_all("docs: describe a change");
        let result = assert_check_fails(&fixture);
        assert!(
            problems_of(&result).contains("summary"),
            "{content:?}: {result}"
        );
    }

    for content in [
        "---\ndefault: minor\n---\n# Add the export queue\n",
        "---\ndefault: minor\n---\n\n\n## Add the export queue\n\nDetail.\n",
    ] {
        let fixture = project(Some(SINGLE_PACKAGE));
        fixture.write(".changeset/summarized.md", content);
        fixture.commit_all("docs: describe a change");
        assert_check_passes(&fixture);
    }
}

#[test]
fn a_change_file_that_existed_before_the_range_does_not_count() {
    let fixture = Fixture::git();
    fixture.write(".changeset/old.md", VALID_CHANGE);
    fixture.commit_all("docs: describe an old change");
    fixture.git_command(&["checkout", "--quiet", "-b", "feature"]);
    fixture.write(
        ".changeset/old.md",
        "---\ndefault: patch\n---\n\n# Edited\n",
    );
    fixture.commit_all("docs: edit the old change");
    assert_check_fails(&fixture);
}

#[test]
fn change_files_the_base_released_after_the_branch_point_do_not_count() {
    let fixture = Fixture::git();
    fixture.write(".changeset/old.md", VALID_CHANGE);
    fixture.commit_all("docs: describe an old change");
    fixture.git_command(&["branch", "feature"]);
    fixture.git_command(&["rm", "--quiet", ".changeset/old.md"]);
    fixture.commit_all("release: 1.1.0");
    fixture.git_command(&["checkout", "--quiet", "feature"]);
    fixture.commit_all("docs: edit nothing user-visible");
    assert_check_fails(&fixture);
}

#[test]
fn change_files_are_read_as_committed_at_head() {
    let fixture = project(Some(SINGLE_PACKAGE));
    fixture.write(".changeset/add.md", VALID_CHANGE);
    fixture.commit_all("docs: describe a change");
    fixture.write(".changeset/add.md", "not a change file\n");
    assert_check_passes(&fixture);
}

#[test]
fn an_unknown_revision_is_not_found() {
    let outcome = project(None).run(&["change-files", "check", "main", "no-such-branch"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
    assert_eq!(outcome.error()["details"]["revision"], "no-such-branch");
}

#[test]
fn an_unreadable_knope_toml_is_a_precondition_failure() {
    let fixture = project(Some("[package\n"));
    fixture.write(".changeset/add.md", VALID_CHANGE);
    fixture.commit_all("docs: describe a change");
    let outcome = check(&fixture);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(
        !outcome.error()["suggestions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
