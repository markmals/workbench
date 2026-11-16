//! The report `workbench workflow status` gathers from Git, `gh`, the proposals, and Beads.

use serde_json::{Value, json};

use crate::support::fake_bd::{
    FakeBd, Reply, available, bare_remote, bd_calls, no_workspace, remotes, synced,
};
use crate::support::fake_gh::{install_broken_gh, install_fake_gh, install_unauthenticated_gh};
use crate::support::{Fixture, Outcome, assert_matches_output_schema};

fn merged_pull_request() -> Value {
    json!({
        "number": 1,
        "state": "MERGED",
        "baseRefName": "main",
        "isDraft": false,
        "title": "Feature",
        "url": "https://github.com/owner/repo/pull/1",
    })
}

/// A checkout on `feature` whose `gh` lists `pull_requests` for it, with an implemented
/// proposal that names pull request 1.
fn status_with_pull_requests(pull_requests: Value) -> Outcome {
    let fixture = Fixture::git();
    fixture.git_command(&["switch", "--quiet", "--create", "feature"]);
    install_fake_gh(&fixture, "feature", "main", &pull_requests);
    FakeBd::new().on("where", no_workspace()).install(&fixture);
    fixture.write(
        "proposals/0001-feature.md",
        "---\nid: proposal.0001\ntitle: Feature\nstatus: implemented\npull-request: 1\n---\n",
    );
    fixture.run(&["workflow", "status", "--output", "text"])
}

/// A checkout with no `origin` and no usable `gh`, so its local `main` is the only evidence
/// of the default branch, whose Beads replica is `beads`.
fn without_remote(beads: &FakeBd, stay_on_main: bool) -> Fixture {
    let fixture = Fixture::git();
    if !stay_on_main {
        fixture.git_command(&["switch", "--quiet", "--create", "feature/x"]);
    }
    install_broken_gh(&fixture);
    beads.install(&fixture);
    fixture
}

fn replica() -> FakeBd {
    FakeBd::new()
        .on("where", available())
        .on("sync", synced())
        .on(
            "ready",
            Reply::json(json!([
                { "id": "wb-1", "title": "Fix the parser", "priority": 1, "status": "open", "description": "x" }
            ])),
        )
        .on(
            "list",
            Reply::json(json!([
                { "id": "wb-2", "title": "Port status", "status": "in_progress", "assignee": "mark@beta" }
            ])),
        )
        .on(
            "prime",
            Reply::stdout(
                "[bd prime] If this output is truncated by your host, read the full output.\n\n## Persistent Memories (1)\n\n### fact\nA durable fact.\n",
            ),
        )
}

/// A replica whose Beads remote and Git `origin` are the same local bare repository.
fn with_own_remote(fixture: &Fixture, beads: FakeBd) {
    let url = bare_remote(fixture, true);
    fixture.git_command(&["remote", "add", "origin", url.trim_start_matches("git+")]);
    beads.on("dolt remote list", remotes(&url)).install(fixture);
}

fn status_json(fixture: &Fixture, args: &[&str]) -> Value {
    let mut command = vec!["workflow", "status"];
    command.extend_from_slice(args);
    let outcome = fixture.run(&command);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let result = outcome.json();
    assert_matches_output_schema(&["workflow", "status"], &result);
    result
}

fn status_text(fixture: &Fixture) -> String {
    let outcome = fixture.run(&["workflow", "status", "--output", "text"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    outcome.stdout
}

fn assert_contains(report: &str, expected: &str) {
    assert!(
        report.contains(expected),
        "expected the report to contain {expected}:\n{report}"
    );
}

#[test]
fn finds_a_merged_feature_pr_after_it_leaves_the_open_pr_list() {
    let outcome = status_with_pull_requests(json!([merged_pull_request()]));

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert_contains(&outcome.stdout, "PR #1 (merged)");
    assert_contains(&outcome.stdout, "Phase 5 — Implementation merged");
}

#[test]
fn does_not_select_an_arbitrary_pr_when_a_branch_name_has_been_reused() {
    let outcome = status_with_pull_requests(json!([
        { "number": 1, "state": "MERGED", "baseRefName": "main", "isDraft": false, "title": "Earlier" },
        { "number": 2, "state": "CLOSED", "baseRefName": "main", "isDraft": false, "title": "Later" },
    ]));
    let report = outcome.stdout;

    assert_contains(
        &report,
        "PR — unknown (Multiple pull requests found for branch feature",
    );
    assert_contains(&report, "inspect their proposal links");
    assert_contains(
        &report,
        "Check gh access and the branch's pull request association",
    );
    assert!(!report.contains("Implementation merged"), "{report}");
}

#[test]
fn recognizes_a_local_default_branch_when_there_is_no_remote_and_no_gh() {
    let fixture = without_remote(&FakeBd::new().on("where", no_workspace()), false);
    let report = status_text(&fixture);

    assert_contains(&report, "branch feature/x");
    assert_contains(&report, "PR — unknown (gh is unavailable)");
    assert_contains(&report, "Phase 1 — Preparation");
    assert!(!report.contains("Branch role unknown"), "{report}");
}

#[test]
fn an_unauthenticated_gh_is_an_unknown_pull_request_not_a_missing_one() {
    let fixture = Fixture::git();
    fixture.git_command(&["switch", "--quiet", "--create", "feature"]);
    install_unauthenticated_gh(&fixture);
    FakeBd::new().on("where", no_workspace()).install(&fixture);

    let result = status_json(&fixture, &[]);

    assert_eq!(
        result["facts"]["pullRequest"],
        json!({ "status": "unknown", "reason": "gh is not authenticated" })
    );
    assert_eq!(result["reason"], "gh is not authenticated");
    assert_eq!(result["phase"], "1");
}

#[test]
fn reports_beads_after_the_phase_sync_outcome_ready_work_items_in_progress_memories() {
    let fixture = without_remote(&replica(), true);
    let report = status_text(&fixture);

    let phase = report.find("Phase 1 — Preparation").expect("a phase");
    let beads = report.find("Beads").expect("a Beads section");
    assert!(
        phase < beads,
        "the Beads section follows the phase:\n{report}"
    );
    for expected in [
        "synced",
        "wb-1",
        "Fix the parser",
        "wb-2",
        "mark@beta",
        "A durable fact.",
    ] {
        assert_contains(&report, expected);
    }
}

#[test]
fn adds_a_beads_object_to_the_status_result() {
    let fixture = without_remote(&replica(), true);
    let status = status_json(&fixture, &[]);

    assert_eq!(status["phase"], "1");
    assert_eq!(status["beads"]["sync"]["outcome"], "synced");
    assert!(status["beads"]["sync"]["detail"].is_string());
    assert_eq!(
        status["beads"]["ready"],
        json!([{ "id": "wb-1", "title": "Fix the parser", "priority": 1 }])
    );
    assert_eq!(
        status["beads"]["inProgress"],
        json!([{ "id": "wb-2", "title": "Port status", "assignee": "mark@beta" }])
    );
    assert_eq!(
        status["beads"]["memories"],
        "## Persistent Memories (1)\n\n### fact\nA durable fact."
    );
}

#[test]
fn the_report_syncs_first_then_reads_ready_work_items_in_progress_and_memories() {
    let fixture = without_remote(&replica(), true);
    status_json(&fixture, &[]);

    let calls: Vec<String> = bd_calls(&fixture)
        .iter()
        .filter(|call| call.args[0] != "dolt")
        .map(|call| call.line())
        .collect();
    let commands: Vec<&str> = calls
        .iter()
        .map(|call| call.split(' ').next().unwrap_or_default())
        .collect();
    assert_eq!(commands, ["where", "sync", "ready", "list", "prime"]);
    assert!(
        calls.contains(&"list --status in_progress --limit 0 --json".to_string()),
        "{calls:?}"
    );
    assert!(
        calls.contains(&"ready --limit 0 --json".to_string()),
        "{calls:?}"
    );
    assert!(
        calls.contains(&"prime --memories-only".to_string()),
        "{calls:?}"
    );
}

#[test]
fn no_sync_reports_the_replica_without_pulling_or_pushing() {
    let fixture = without_remote(&replica(), true);
    let status = status_json(&fixture, &["--no-sync"]);

    assert_eq!(status["beads"]["sync"]["outcome"], "skipped");
    assert_eq!(status["beads"]["ready"][0]["id"], "wb-1");
    assert!(
        !bd_calls(&fixture)
            .iter()
            .any(|call| call.args[0] == "sync" || call.args[0] == "dolt"),
        "{:?}",
        bd_calls(&fixture)
    );
}

#[test]
fn a_failed_beads_sync_is_reported_with_its_reason_and_does_not_fail_status() {
    let beads = replica()
        .on(
            "sync",
            Reply::json(json!({ "error": "sync failed: could not resolve host github.com" }))
                .exit(1),
        )
        .on(
            "dolt remote list",
            Reply::stdout("").stderr("Error: no remotes\n").exit(1),
        );
    let fixture = without_remote(&beads, true);
    let report = status_text(&fixture);

    assert_contains(&report, "failed");
    assert_contains(&report, "could not resolve host github.com");
    assert_contains(&report, "this machine");
    assert_contains(&report, "Fix the parser");
}

#[test]
fn a_failed_sync_still_reports_this_machines_replica() {
    let fixture = without_remote(&FakeBd::new(), true);
    with_own_remote(
        &fixture,
        replica()
            .on(
                "sync",
                Reply::json(json!({ "error": "sync failed: remote unreachable" })).exit(1),
            )
            .on(
                "ready",
                Reply::json(json!([{ "id": "wb-1", "title": "Fix the parser", "priority": 2 }])),
            ),
    );
    let status = status_json(&fixture, &[]);

    assert_eq!(status["beads"]["sync"]["outcome"], "failed");
    assert_eq!(
        status["beads"]["ready"],
        json!([{ "id": "wb-1", "title": "Fix the parser", "priority": 2 }])
    );
}

#[test]
fn an_unavailable_beads_workspace_replaces_the_section_with_a_pointer_to_setup() {
    let fixture = without_remote(&FakeBd::new().on("where", no_workspace()), true);
    let report = status_text(&fixture);
    let status = status_json(&fixture, &[]);

    assert_contains(&report, "unavailable");
    assert_contains(&report, "No active beads workspace found.");
    assert_contains(&report, "mise run setup");
    let keys: Vec<&String> = status["beads"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["unavailable"]);
    assert!(
        status["beads"]["unavailable"]
            .as_str()
            .unwrap()
            .contains("No active beads workspace found")
    );
    let calls: Vec<String> = bd_calls(&fixture).iter().map(|call| call.line()).collect();
    assert_eq!(calls, ["where --json", "where --json"]);
}

#[test]
fn without_bd_installed_the_report_is_unavailable() {
    let mut fixture = without_remote(&FakeBd::new(), true);
    std::fs::remove_file(fixture.path(".fixture-bin/bd")).unwrap();
    let path = format!("{}:/usr/bin:/bin", fixture.path(".fixture-bin").display());
    fixture.set_env("PATH", &path);

    let status = status_json(&fixture, &[]);

    assert_eq!(
        status["beads"],
        json!({ "unavailable": "bd is not installed" })
    );
}

#[test]
fn a_workspace_whose_database_is_not_initialized_is_unavailable() {
    let beads = FakeBd::new().on("where", Reply::json(json!({ "path": "/project/.beads" })));
    let fixture = without_remote(&beads, true);
    let status = status_json(&fixture, &[]);

    let reason = status["beads"]["unavailable"].as_str().unwrap();
    assert!(reason.contains("not initialized"), "{reason}");
}

#[test]
fn a_replica_that_cannot_be_read_is_unavailable_and_the_report_keeps_the_syncs_outcome() {
    let fixture = without_remote(&FakeBd::new(), true);
    with_own_remote(
        &fixture,
        replica()
            .on(
                "sync",
                Reply::json(json!({ "error": "sync failed: remote unreachable" })).exit(1),
            )
            .on(
                "ready",
                Reply::json(json!({ "error": "database is locked" })).exit(1),
            ),
    );
    let status = status_json(&fixture, &[]);

    let reason = status["beads"]["unavailable"].as_str().unwrap();
    assert!(reason.contains("database is locked"), "{reason}");
    assert!(reason.contains("failed"), "{reason}");
    assert!(reason.contains("remote unreachable"), "{reason}");
}
