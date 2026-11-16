//! `workbench beads sync`: one sync with the project's own Beads remote.

use std::time::{Duration, Instant};

use serde_json::json;

use super::{message, project, project_with_remote, replica, set_origin};
use crate::support::assert_matches_output_schema;
use crate::support::fake_bd::{FakeBd, Reply, bd_calls, no_workspace, remotes, synced};

fn sync_failure(reason: &str) -> Reply {
    Reply::json(json!({ "error": reason })).exit(1)
}

fn called(fixture: &crate::support::Fixture, prefix: &str) -> bool {
    bd_calls(fixture)
        .iter()
        .any(|call| call.line().starts_with(prefix))
}

#[test]
fn a_sync_that_pulls_and_pushes_reports_synced() {
    let fixture = project(&replica().on("sync", synced()));

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "synced");
    let root = fixture.root().canonicalize().unwrap();
    assert_eq!(receipt["project"], root.display().to_string());
    assert!(receipt.get("remote").is_some(), "{receipt}");
    assert_matches_output_schema(&["beads", "sync"], &receipt);
}

#[test]
fn a_sync_whose_credentials_are_rejected_is_unauthenticated_with_beads_reason() {
    let fixture = project_with_remote(
        true,
        replica().on(
            "sync",
            sync_failure("sync failed: push: authentication required"),
        ),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "UNAUTHENTICATED");
    assert!(message(&outcome).contains("authentication required"));
    assert!(
        !called(&fixture, "dolt push"),
        "a remote with Beads history is never pushed over"
    );
}

#[test]
fn the_first_sync_to_a_remote_without_beads_history_publishes_this_replica() {
    let fixture = project_with_remote(
        false,
        replica()
            .on(
                "sync",
                sync_failure("fetch failed: no branches found in remote 'origin'"),
            )
            .on("dolt push", Reply::stdout("Push complete.\n")),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert_eq!(outcome.json()["status"], "synced");
    assert!(called(&fixture, "dolt push"), "{:?}", bd_calls(&fixture));
}

#[test]
fn a_first_publish_that_fails_reports_its_reason() {
    let fixture = project_with_remote(
        false,
        replica()
            .on(
                "sync",
                sync_failure("fetch failed: no branches found in remote 'origin'"),
            )
            .on(
                "dolt push",
                Reply::stdout("")
                    .stderr("Error: push rejected by the remote\n")
                    .exit(1),
            ),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 5);
    assert_eq!(outcome.error_code(), "UNAVAILABLE");
    assert!(message(&outcome).contains("push rejected by the remote"));
}

#[test]
fn a_replica_whose_beads_remote_is_another_repository_fails_without_pulling_or_pushing() {
    let fixture = project(
        &replica()
            .on(
                "dolt remote list",
                remotes("git+https://github.com/markmals/workbench.git"),
            )
            .on("sync", synced())
            .on("dolt push", Reply::stdout("Push complete.\n")),
    );
    set_origin(&fixture, "https://github.com/someone/new-project.git");

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    let reason = message(&outcome);
    assert!(reason.contains("markmals/workbench"), "{reason}");
    assert!(reason.contains("someone/new-project"), "{reason}");
    assert!(!called(&fixture, "sync") && !called(&fixture, "dolt push"));
}

#[test]
fn a_checkout_without_a_git_origin_does_not_sync_a_replica_that_has_a_beads_remote() {
    let fixture = project(
        &replica()
            .on(
                "dolt remote list",
                remotes("git+https://github.com/markmals/workbench.git"),
            )
            .on("sync", synced()),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(message(&outcome).contains("no origin"));
    assert!(!called(&fixture, "sync"), "{:?}", bd_calls(&fixture));
}

#[test]
fn two_local_repositories_that_differ_only_by_dot_git_are_different_remotes() {
    let fixture = project(
        &replica()
            .on(
                "dolt remote list",
                remotes("git+file:///srv/git/project.git"),
            )
            .on("sync", synced()),
    );
    set_origin(&fixture, "/srv/git/project");

    assert_eq!(
        fixture.run(&["beads", "sync"]).error_code(),
        "PRECONDITION_FAILED"
    );
}

#[test]
fn the_same_repository_written_another_way_is_this_projects_own_remote() {
    let fixture = project(&replica());
    set_origin(&fixture, "unset");
    let pairs = [
        (
            "git@github.com:markmals/Workbench.git",
            "git+https://github.com/markmals/workbench.git",
        ),
        (
            "ssh://git@github.com:22/markmals/workbench",
            "git+https://github.com/markmals/workbench.git/",
        ),
        (
            "https://token@github.com/markmals/workbench.git",
            "git+https://github.com/markmals/workbench",
        ),
        ("/srv/git/project.git", "git+file:///srv/git/project.git"),
    ];
    for (origin, beads_remote) in pairs {
        replica()
            .on("dolt remote list", remotes(beads_remote))
            .on("sync", synced())
            .install(&fixture);
        set_origin(&fixture, origin);

        let outcome = fixture.run(&["beads", "sync"]);
        assert_eq!(
            outcome.status, 0,
            "{origin} and {beads_remote}: {}",
            outcome.stderr
        );
    }

    let root = fixture.root().canonicalize().unwrap();
    let shared = root.parent().unwrap().join("shared/repo.git");
    replica()
        .on(
            "dolt remote list",
            remotes(&format!("git+file://{}", shared.display())),
        )
        .on("sync", synced())
        .install(&fixture);
    set_origin(&fixture, "../shared/repo.git");
    let outcome = fixture.run(&["beads", "sync"]);
    assert_eq!(
        outcome.status, 0,
        "a relative origin resolves from the checkout: {}",
        outcome.stderr
    );
}

#[test]
fn a_receipt_never_shows_the_credentials_in_a_remote_url() {
    let fixture = project(
        &replica()
            .on(
                "dolt remote list",
                remotes("git+https://user:secret-token@github.com/markmals/workbench.git"),
            )
            .on("sync", synced()),
    );
    set_origin(&fixture, "https://github.com/markmals/workbench.git");

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert!(
        !outcome.stdout.contains("secret-token"),
        "{}",
        outcome.stdout
    );
    assert!(
        outcome.json()["remote"]
            .as_str()
            .unwrap()
            .contains("github.com/markmals/workbench")
    );
}

#[test]
fn a_conflict_beads_cannot_settle_names_the_tables_and_the_items_that_differ_from_the_remote() {
    let fixture = project(
        &replica()
            .on(
                "sync",
                Reply::json(
                    json!({ "status": "conflict", "conflicts": ["issues"], "pushed": false }),
                )
                .exit(2),
            )
            .on(
                "diff",
                Reply::json(json!([
                    {
                        "IssueID": "wb-1",
                        "DiffType": "removed",
                        "OldValue": { "id": "wb-1", "title": "Probe one" },
                        "NewValue": null,
                    },
                    {
                        "IssueID": "wb-2",
                        "DiffType": "added",
                        "OldValue": null,
                        "NewValue": { "id": "wb-2", "title": "Probe two" },
                    },
                ])),
            ),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 4);
    let error = outcome.error();
    assert_eq!(error["code"], "CONFLICT");
    assert_eq!(error["details"]["tables"], json!(["issues"]));
    assert_eq!(
        error["details"]["differing"],
        json!([
            { "id": "wb-1", "title": "Probe one", "change": "removed" },
            { "id": "wb-2", "title": "Probe two", "change": "added" },
        ])
    );
    let reason = message(&outcome);
    assert!(reason.contains("issues"), "{reason}");
    assert!(reason.contains("wb-1 (Probe one)"), "{reason}");
    assert!(reason.contains("wb-2 (Probe two)"), "{reason}");
}

#[test]
fn a_stuck_dirty_working_set_is_blocked_with_beads_explanation() {
    let fixture = project(&replica().on(
        "sync",
        Reply::json(json!({ "error": "pending graph edits blocked every attempt" })).exit(4),
    ));

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 3);
    let error = outcome.error();
    assert_eq!(error["code"], "PRECONDITION_FAILED");
    assert_eq!(error["details"]["outcome"], "blocked");
    assert!(message(&outcome).contains("pending graph edits blocked every attempt"));
}

#[test]
fn push_retries_running_out_is_unavailable_and_retryable() {
    let fixture = project_with_remote(
        true,
        replica().on(
            "sync",
            Reply::json(json!({ "error": "push race: retries exhausted" })).exit(3),
        ),
    );

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 5);
    let error = outcome.error();
    assert_eq!(error["code"], "UNAVAILABLE");
    assert_eq!(error["retryable"], true);
    assert!(message(&outcome).contains("retries exhausted"));
}

#[test]
fn a_sync_that_outlasts_its_time_limit_times_out() {
    let fixture = project(&replica().on("sync", synced().after_seconds(5)));

    let started = Instant::now();
    let outcome = fixture.run(&["beads", "sync", "--timeout", "1"]);

    assert!(
        started.elapsed() < Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(outcome.status, 5);
    assert_eq!(outcome.error_code(), "TIMEOUT");
    assert!(message(&outcome).contains("did not finish within 1 second"));
}

#[test]
fn describing_a_conflict_stays_within_the_syncs_time_limit() {
    let fixture = project(
        &replica()
            .on(
                "sync",
                Reply::json(
                    json!({ "status": "conflict", "conflicts": ["issues"], "pushed": false }),
                )
                .exit(2),
            )
            .on("diff", Reply::json(json!([])).after_seconds(5)),
    );

    let started = Instant::now();
    let outcome = fixture.run(&["beads", "sync", "--timeout", "1"]);

    assert!(
        started.elapsed() < Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(outcome.error_code(), "CONFLICT");
    assert!(message(&outcome).contains("bd diff main origin/main"));
}

#[test]
fn the_sync_exits_zero_when_synced_and_nonzero_otherwise_saying_the_outcome() {
    let synced_fixture = project(&replica().on("sync", synced()));
    let blocked_fixture =
        project(&replica().on("sync", Reply::json(json!({ "error": "stuck" })).exit(4)));

    let synced_outcome = synced_fixture.run(&["beads", "sync", "--output", "text"]);
    let blocked_outcome = blocked_fixture.run(&["beads", "sync", "--output", "text"]);

    assert_eq!(synced_outcome.status, 0, "{}", synced_outcome.stderr);
    assert!(
        synced_outcome.stdout.contains("synced"),
        "{}",
        synced_outcome.stdout
    );
    assert_ne!(blocked_outcome.status, 0);
    assert!(
        blocked_outcome.stderr.contains("blocked"),
        "{}",
        blocked_outcome.stderr
    );
}

#[test]
fn without_bd_the_sync_reports_the_missing_dependency() {
    let mut fixture = project(&replica());
    std::fs::remove_file(fixture.path(".fixture-bin/bd")).unwrap();
    let path = format!("{}:/usr/bin:/bin", fixture.path(".fixture-bin").display());
    fixture.set_env("PATH", &path);

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "DEPENDENCY_MISSING");
}

#[test]
fn outside_a_beads_workspace_the_sync_is_a_precondition_failure_naming_setup() {
    let fixture = project(&FakeBd::new().on("where", no_workspace()));

    let outcome = fixture.run(&["beads", "sync"]);

    assert_eq!(outcome.status, 3);
    let error = outcome.error();
    assert_eq!(error["code"], "PRECONDITION_FAILED");
    assert!(message(&outcome).contains("No active beads workspace found"));
    assert!(error["suggestions"].to_string().contains("mise run setup"));
}

#[test]
fn every_bd_process_runs_with_an_empty_dolt_info_branch() {
    let mut fixture = project(&replica().on("sync", synced()));
    fixture.set_env("DOLT_REMOTE_INFO_BRANCH", "main");

    assert_eq!(fixture.run(&["beads", "sync"]).status, 0);

    let calls = bd_calls(&fixture);
    assert!(!calls.is_empty());
    for call in calls {
        assert_eq!(call.info_branch.as_deref(), Some(""), "{}", call.line());
    }
}

#[test]
fn no_bd_process_can_prompt_for_git_credentials() {
    let mut fixture = project(&replica().on("sync", synced()));
    fixture.set_env("GIT_TERMINAL_PROMPT", "1");

    assert_eq!(fixture.run(&["beads", "sync"]).status, 0);

    for call in bd_calls(&fixture) {
        assert_eq!(
            call.terminal_prompt.as_deref(),
            Some("0"),
            "{}",
            call.line()
        );
    }
}

#[test]
fn a_dry_run_reports_the_remote_and_unpublished_changes_without_syncing() {
    let fixture = project_with_remote(
        true,
        replica().on("sync", synced()).on(
            "diff",
            Reply::json(json!([{
                "IssueID": "wb-3",
                "DiffType": "added",
                "OldValue": null,
                "NewValue": { "id": "wb-3", "title": "Record the fix" },
            }])),
        ),
    );

    let outcome = fixture.run(&["beads", "sync", "--dry-run"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "dry-run");
    assert!(
        receipt["remote"].as_str().unwrap().ends_with("remote.git"),
        "{receipt}"
    );
    assert_eq!(receipt["unpublished"]["determined"], true);
    assert_eq!(
        receipt["unpublished"]["items"],
        json!([{ "id": "wb-3", "title": "Record the fix", "change": "added" }])
    );
    assert!(
        receipt["cannotPredict"].to_string().contains("conflict"),
        "{receipt}"
    );
    assert!(!called(&fixture, "sync") && !called(&fixture, "dolt push"));
    assert!(
        called(&fixture, "diff origin/main main"),
        "{:?}",
        bd_calls(&fixture)
    );
    assert_matches_output_schema(&["beads", "sync"], &receipt);
}

#[test]
fn a_dry_run_says_when_unpublished_changes_cannot_be_determined() {
    let fixture = project(
        &replica().on(
            "diff",
            Reply::json(json!({ "error": "failed to get diff: branch not found: origin/main" }))
                .exit(1),
        ),
    );

    let receipt = fixture.run(&["beads", "sync", "--dry-run"]).json();

    assert_eq!(receipt["unpublished"]["determined"], false);
    assert!(
        receipt["unpublished"]["reason"]
            .as_str()
            .unwrap()
            .contains("branch not found: origin/main"),
        "{receipt}"
    );
}

#[test]
fn a_dry_run_refuses_a_replica_whose_beads_remote_is_another_repository() {
    let fixture = project(&replica().on(
        "dolt remote list",
        remotes("git+https://github.com/markmals/workbench.git"),
    ));
    set_origin(&fixture, "https://github.com/someone/new-project.git");

    let outcome = fixture.run(&["beads", "sync", "--dry-run"]);

    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
}

#[test]
fn a_time_limit_of_zero_seconds_is_invalid() {
    let fixture = project(&replica().on("sync", synced()));

    let outcome = fixture.run(&["beads", "sync", "--timeout", "0"]);

    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(outcome.error()["details"]["argument"], "timeout");
    assert!(!called(&fixture, "sync"));
}
