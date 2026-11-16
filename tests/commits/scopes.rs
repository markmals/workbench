//! `workbench commits scopes`: every scope a subject may use, and where each comes from.

use super::*;

fn scopes(fixture: &Fixture) -> Value {
    let outcome = fixture.run(&["commits", "scopes"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let result = outcome.json();
    assert_matches_output_schema(&["commits", "scopes"], &result);
    result
}

fn entries(result: &Value) -> Vec<(String, String)> {
    result["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| {
            (
                item["scope"].as_str().unwrap().to_string(),
                item["source"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn lists_the_built_in_scopes_then_the_scopes_file_then_one_per_proposal() {
    let fixture = Fixture::git();
    fixture
        .write(
            ".agents/commit-scopes",
            "# Product scopes\napi\n\nweb\ndocs\n",
        )
        .write("proposals/0002-export-queue.md", "# Export queue\n")
        .write("proposals/README.md", "# Proposals\n");

    let result = scopes(&fixture);
    let listed = entries(&result);

    let built_in: Vec<&str> = listed
        .iter()
        .filter(|(_, source)| source == "built-in")
        .map(|(scope, _)| scope.as_str())
        .collect();
    assert_eq!(built_in, BUILT_IN_SCOPES);
    let rest = &listed[BUILT_IN_SCOPES.len()..];
    assert_eq!(
        rest,
        [
            ("api".to_string(), "scopes-file".to_string()),
            ("web".to_string(), "scopes-file".to_string()),
            ("0002-export-queue".to_string(), "proposal".to_string()),
        ],
        "a scope the file repeats is listed once, under the built-in scopes"
    );
    let proposal = result["items"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(proposal["path"], "proposals/0002-export-queue.md");
}

#[test]
fn every_listed_scope_passes_the_commit_check() {
    let fixture = Fixture::git();
    fixture
        .write(".agents/commit-scopes", "api\n")
        .write("proposals/0002-export-queue.md", "# Export queue\n");
    for (scope, _) in entries(&scopes(&fixture)) {
        assert_passes(&fixture, &format!("{scope}: describe the change\n"));
    }
}

#[test]
fn text_output_groups_the_scopes_by_source() {
    let fixture = Fixture::git();
    fixture.write(".agents/commit-scopes", "api\n");
    let outcome = fixture.run(&["commits", "scopes", "--output", "text"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert!(outcome.stdout.contains("agents"), "{}", outcome.stdout);
    assert!(
        outcome.stdout.contains(".agents/commit-scopes"),
        "{}",
        outcome.stdout
    );
    assert!(outcome.stdout.contains("api"), "{}", outcome.stdout);
}

#[test]
fn outside_a_git_repository_is_a_precondition_failure() {
    let outcome = Fixture::empty().run(&["commits", "scopes"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
}
