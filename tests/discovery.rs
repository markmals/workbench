//! Discovery and the output contract every command shares.

mod support;

use support::{Fixture, assert_matches_output_schema};

#[test]
fn bare_workbench_and_help_orient_without_failing() {
    let fixture = Fixture::empty();
    for args in [&[][..], &["--help"][..]] {
        let outcome = fixture.run(args);
        assert_eq!(outcome.status, 0);
        assert!(
            outcome.stdout.contains("workbench cli search"),
            "{}",
            outcome.stdout
        );
        assert!(outcome.stdout.contains("workbench schema"));
        assert!(outcome.stdout.contains("docs"));
    }
}

#[test]
fn version_names_the_release() {
    let outcome = Fixture::empty().run(&["--version"]);
    assert_eq!(outcome.status, 0);
    assert_eq!(
        outcome.stdout.trim(),
        format!("workbench {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn cli_search_returns_at_most_five_ranked_matches_with_exact_commands() {
    let outcome = Fixture::empty().run(&[
        "cli",
        "search",
        "find",
        "a",
        "command",
        "or",
        "search",
        "documentation",
    ]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let result = outcome.json();
    let items = result["items"].as_array().unwrap();
    assert!(!items.is_empty() && items.len() <= 5);
    assert!(
        items[0]["command"]
            .as_str()
            .unwrap()
            .starts_with("workbench ")
    );
    assert!(items[0]["score"].as_f64().unwrap() > 0.0);
    assert!(result.get("hasMore").is_some() && result.get("nextCursor").is_some());
    assert_matches_output_schema(&["cli", "search"], &result);
}

#[test]
fn cli_search_pages_through_every_match_with_its_cursor() {
    let fixture = Fixture::empty();
    let first = fixture
        .run(&["cli", "search", "search", "schema", "shell", "--limit", "1"])
        .json();
    assert_eq!(first["items"].as_array().unwrap().len(), 1);
    assert_eq!(first["hasMore"], true);
    let cursor = first["nextCursor"].as_str().unwrap();
    let second = fixture
        .run(&[
            "cli", "search", "search", "schema", "shell", "--limit", "1", "--cursor", cursor,
        ])
        .json();
    assert_ne!(first["items"][0]["command"], second["items"][0]["command"]);
}

#[test]
fn schema_lists_every_command_and_describes_one() {
    let fixture = Fixture::empty();
    let list = fixture.run(&["schema"]).json();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["command"] == "workbench cli search")
    );

    let entry = fixture.run(&["schema", "cli", "search"]).json();
    assert_eq!(entry["id"], "cli.search");
    let limit = entry["arguments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|argument| argument["name"] == "limit")
        .unwrap();
    assert_eq!(limit["default"], "5");
    assert!(entry["output"].is_object());
    assert_eq!(
        fixture.run(&["schema", "cli.search"]).json()["id"],
        "cli.search"
    );
}

#[test]
fn an_unknown_command_in_schema_is_not_found() {
    let outcome = Fixture::empty().run(&["schema", "nothing", "here"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
    assert!(outcome.stdout.is_empty());
}

#[test]
fn missing_input_fails_at_once_naming_the_missing_argument() {
    let outcome = Fixture::empty().run(&["cli", "search"]);
    assert_eq!(outcome.status, 2);
    let error = outcome.error();
    assert_eq!(error["code"], "INVALID_ARGUMENT");
    assert_eq!(error["retryable"], false);
    assert!(
        error["details"]["missing"].to_string().contains("QUERY"),
        "{error}"
    );
    assert!(error["details"]["arguments"].is_array());
}

#[test]
fn an_unaccepted_value_lists_the_accepted_ones() {
    let outcome = Fixture::empty().run(&["cli", "completions", "tcsh"]);
    assert_eq!(outcome.status, 2);
    let error = outcome.error();
    assert_eq!(error["code"], "INVALID_ARGUMENT");
    assert!(
        error["details"]["acceptedValues"]
            .to_string()
            .contains("zsh"),
        "{error}"
    );
}

#[test]
fn a_resource_without_an_operation_lists_its_operations() {
    let outcome = Fixture::empty().run(&["cli"]);
    assert_eq!(outcome.status, 2);
    let error = outcome.error();
    assert!(
        error["details"]["commands"]
            .to_string()
            .contains("workbench cli search"),
        "{error}"
    );
}

#[test]
fn text_output_renders_for_humans() {
    let outcome = Fixture::empty().run(&["schema", "cli", "search", "--output", "text"]);
    assert_eq!(outcome.status, 0);
    assert!(outcome.stdout.starts_with("workbench cli search"));
    assert!(serde_json::from_str::<serde_json::Value>(&outcome.stdout).is_err());
}

#[test]
fn completions_print_a_script_for_the_shell() {
    let fixture = Fixture::empty();
    let result = fixture.run(&["cli", "completions", "bash"]).json();
    assert_eq!(result["shell"], "bash");
    assert!(result["script"].as_str().unwrap().contains("workbench"));
    let text = fixture.run(&["cli", "completions", "zsh", "--output", "text"]);
    assert!(text.stdout.starts_with("#compdef workbench"));
}

#[test]
fn docs_search_returns_package_paths_to_read() {
    let fixture = Fixture::git();
    fixture.link_package();
    let outcome = fixture.run(&["docs", "search", "write", "a", "proposal"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let result = outcome.json();
    let first = &result["items"][0];
    assert_eq!(
        first["path"],
        ".workbench/skills/writing-a-proposal/SKILL.md"
    );
    assert!(fixture.path(first["path"].as_str().unwrap()).is_file());
    assert!(!first["snippet"].as_str().unwrap().is_empty());
    assert_matches_output_schema(&["docs", "search"], &result);
}

#[test]
fn docs_search_without_the_package_link_names_the_fix() {
    let outcome = Fixture::git().run(&["docs", "search", "proposal"]);
    assert_eq!(outcome.status, 3);
    let error = outcome.error();
    assert_eq!(error["code"], "PRECONDITION_FAILED");
    assert!(error["suggestions"].to_string().contains("mise install"));
}

#[test]
fn docs_search_outside_a_git_repository_is_a_precondition_failure() {
    let fixture = Fixture::empty();
    let outcome = fixture.run(&["docs", "search", "proposal"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
}
