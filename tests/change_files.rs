//! `workbench change-files`: creating Knope change files and requiring one in a commit range.

mod support;

use serde_json::{Value, json};
use support::{Fixture, Outcome, assert_matches_output_schema};

const SINGLE_PACKAGE: &str = "[package]\nversioned_files = []\nextra_changelog_sections = [\n    { name = \"Notes\", types = [\"note\"] },\n]\n\n[changes]\nignore_conventional_commits = true\n";

const TWO_PACKAGES: &str = "[packages.api]\nversioned_files = []\n\n[packages.web]\nversioned_files = []\nextra_changelog_sections = [{ name = \"Poems\", types = [\"poem\"] }]\n";

const VALID_CHANGE: &str = "---\ndefault: minor\n---\n\n# Add the export queue\n";

/// A repository with `knope_toml` (when given) committed on `main`, checked out on a new
/// `feature` branch.
fn project(knope_toml: Option<&str>) -> Fixture {
    let fixture = Fixture::git();
    if let Some(config) = knope_toml {
        fixture.write("knope.toml", config);
        fixture.commit_all("release: configure knope");
    }
    fixture.git_command(&["checkout", "--quiet", "-b", "feature"]);
    fixture
}

fn check(fixture: &Fixture) -> Outcome {
    let outcome = fixture.run(&["change-files", "check", "main", "feature"]);
    if outcome.status <= 1 {
        assert_matches_output_schema(&["change-files", "check"], &outcome.json());
    }
    outcome
}

fn assert_check_passes(fixture: &Fixture) -> Value {
    let outcome = check(fixture);
    assert_eq!(
        outcome.status, 0,
        "the check failed:\n{}\n{}",
        outcome.stdout, outcome.stderr
    );
    outcome.json()
}

fn assert_check_fails(fixture: &Fixture) -> Value {
    let outcome = check(fixture);
    assert_eq!(outcome.status, 1, "the check passed:\n{}", outcome.stdout);
    assert_eq!(outcome.error_code(), "CHECK_FAILED");
    outcome.json()
}

/// The problems the check reports for the one change file the range adds.
fn problems_of(result: &Value) -> String {
    let files = result["changeFiles"].as_array().unwrap();
    assert_eq!(files.len(), 1, "{result}");
    assert_eq!(files[0]["valid"], false, "{result}");
    files[0]["problems"].to_string()
}

fn create(fixture: &Fixture, args: &[&str]) -> Outcome {
    let mut all = vec!["change-files", "create"];
    all.extend_from_slice(args);
    fixture.run(&all)
}

fn canonical_root(fixture: &Fixture) -> String {
    fixture.root().canonicalize().unwrap().display().to_string()
}

#[path = "change_files/check.rs"]
mod check;
#[path = "change_files/create.rs"]
mod create;
