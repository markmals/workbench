//! `workbench beads claim`: claiming an item as this machine's claimant, visibly to the others.

use std::process::Command;

use serde_json::json;

use super::{CLAIMANT, message, project, project_with_remote, replica, set_origin};
use crate::support::assert_matches_output_schema;
use crate::support::fake_bd::{
    FakeBd, Reply, bd_calls, bd_steps, item, open_item, remotes, synced,
};

fn held_by_another() -> FakeBd {
    replica()
        .on("sync", synced())
        .on(
            "update",
            Reply::json(json!({
                "error": "1 of 1 issues failed to update",
                "failed": [{ "id": "wb-1", "error": "issue already claimed by mark@beta" }],
            }))
            .exit(1),
        )
        .on("show", item("wb-1", "in_progress", "mark@beta"))
}

fn first_line(program: &str, args: &[&str]) -> String {
    let output = Command::new(program).args(args).output().expect("runs");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn claiming_an_unheld_item_syncs_claims_syncs_again_and_confirms_the_claim() {
    let fixture = project(
        &replica()
            .on("sync", synced())
            .on("update", item("wb-1", "in_progress", CLAIMANT))
            .on("show", item("wb-1", "in_progress", CLAIMANT)),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "claimed");
    assert_eq!(receipt["id"], "wb-1");
    assert_eq!(receipt["claimant"], CLAIMANT);
    assert_eq!(receipt["published"], true);
    assert!(receipt["project"].is_string() && receipt.get("remote").is_some());
    assert_matches_output_schema(&["beads", "claim"], &receipt);
    assert_eq!(
        bd_steps(&fixture),
        [
            "sync --json",
            "update wb-1 --claim --json",
            "sync --json",
            "show wb-1 --json"
        ]
    );
    let update = bd_calls(&fixture)
        .into_iter()
        .find(|call| call.args[0] == "update")
        .unwrap();
    assert_eq!(update.actor.as_deref(), Some(CLAIMANT));
}

#[test]
fn a_claim_whose_second_sync_fails_is_claimed_on_this_machine_only() {
    let fixture = project_with_remote(
        true,
        replica()
            .on_each(
                "sync",
                vec![
                    synced(),
                    Reply::json(json!({ "error": "sync failed: remote unreachable" })).exit(1),
                ],
            )
            .on("update", Reply::json(json!([{ "id": "wb-1" }])))
            .on("show", item("wb-1", "in_progress", CLAIMANT)),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "claimed-unpublished");
    assert_eq!(receipt["published"], false);
    assert_eq!(receipt["sync"]["code"], "UNAVAILABLE", "{receipt}");
    assert_eq!(receipt["sync"]["retryable"], true, "{receipt}");
    assert!(outcome.stderr.contains("warning:"), "{}", outcome.stderr);
    assert!(
        outcome.stderr.contains("remote unreachable"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn a_claim_whose_sync_is_refused_credentials_names_that_cause() {
    let fixture = project_with_remote(
        true,
        replica()
            .on_each(
                "sync",
                vec![
                    synced(),
                    Reply::json(json!({ "error": "sync failed: push: authentication required" }))
                        .exit(1),
                ],
            )
            .on("update", Reply::json(json!([{ "id": "wb-1" }])))
            .on("show", item("wb-1", "in_progress", CLAIMANT)),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["sync"]["code"], "UNAUTHENTICATED", "{receipt}");
    assert_eq!(receipt["sync"]["retryable"], false, "{receipt}");
    assert!(
        outcome.stderr.contains("UNAUTHENTICATED"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn an_item_another_claimant_holds_is_refused_as_held_naming_the_claimant() {
    let fixture = project(&held_by_another());

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 4);
    let error = outcome.error();
    assert_eq!(error["code"], "CONFLICT");
    assert_eq!(error["details"]["outcome"], "held");
    assert_eq!(error["details"]["holder"], "mark@beta");
    assert!(message(&outcome).contains("mark@beta"));
    let syncs = bd_steps(&fixture)
        .iter()
        .filter(|step| step.starts_with("sync"))
        .count();
    assert_eq!(syncs, 1);
}

#[test]
fn a_claim_a_concurrent_claim_replaced_during_the_sync_is_lost_naming_the_winner() {
    let fixture = project(
        &replica()
            .on("sync", synced())
            .on("update", item("wb-1", "in_progress", CLAIMANT))
            .on("show", item("wb-1", "in_progress", "mark@beta")),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 4);
    let error = outcome.error();
    assert_eq!(error["code"], "CONFLICT");
    assert_eq!(error["details"]["outcome"], "lost");
    assert_eq!(error["details"]["holder"], "mark@beta");
    assert!(message(&outcome).contains("mark@beta"));
}

#[test]
fn without_beads_actor_a_claim_is_made_as_the_login_user_at_the_short_host_name() {
    let host = first_line("uname", &["-n"]);
    let short_host = host.split('.').next().unwrap();
    let claimant = format!("{}@{short_host}", first_line("id", &["-un"]));
    let mut fixture = project(
        &replica()
            .on("sync", synced())
            .on("update", Reply::json(json!([{ "id": "wb-1" }])))
            .on("show", item("wb-1", "in_progress", &claimant)),
    );
    fixture.set_env("BEADS_ACTOR", "");

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    assert_eq!(outcome.json()["claimant"], claimant);
    let update = bd_calls(&fixture)
        .into_iter()
        .find(|call| call.args[0] == "update")
        .unwrap();
    assert_eq!(update.actor, Some(claimant));
}

#[test]
fn a_claim_of_an_unknown_item_is_not_found_with_beads_error() {
    let fixture = project(
        &replica()
            .on("sync", synced())
            .on(
                "update",
                Reply::json(json!({
                    "error": "1 of 1 issues failed",
                    "failed": [{ "id": "wb-9", "error": "issue not found" }],
                }))
                .exit(1),
            )
            .on(
                "show",
                Reply::json(json!({ "error": "no issues found matching the provided IDs" }))
                    .exit(1),
            ),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-9"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
    assert!(message(&outcome).contains("issue not found"));
}

#[test]
fn a_claim_beads_refuses_for_another_reason_is_a_precondition_failure_with_beads_reason() {
    let fixture = project(
        &replica()
            .on("sync", synced())
            .on(
                "update",
                Reply::json(json!({
                    "error": "1 of 1 issues failed to update",
                    "failed": [{ "id": "wb-1", "error": "updating issue: issue not claimable: status closed" }],
                }))
                .exit(1),
            )
            .on("show", item("wb-1", "closed", CLAIMANT)),
    );

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(message(&outcome).contains("not claimable"));
}

#[test]
fn claiming_in_a_replica_whose_beads_remote_is_another_repository_changes_nothing() {
    let fixture = project(
        &replica()
            .on(
                "dolt remote list",
                remotes("git+https://github.com/markmals/workbench.git"),
            )
            .on("sync", synced())
            .on("update", Reply::json(json!([{ "id": "wb-1" }])))
            .on("show", item("wb-1", "in_progress", CLAIMANT)),
    );
    set_origin(&fixture, "https://github.com/someone/new-project.git");

    let outcome = fixture.run(&["beads", "claim", "wb-1"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(message(&outcome).contains("not this project's origin"));
    assert!(
        !bd_steps(&fixture)
            .iter()
            .any(|step| step.starts_with("update") || step.starts_with("sync")),
        "{:?}",
        bd_steps(&fixture)
    );
}

#[test]
fn a_dry_run_reports_the_item_its_current_claimant_and_the_claimant_it_would_claim_as() {
    let fixture = project(&held_by_another());

    let outcome = fixture.run(&["beads", "claim", "wb-1", "--dry-run"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "dry-run");
    assert_eq!(receipt["id"], "wb-1");
    assert_eq!(receipt["title"], "Fix the parser");
    assert_eq!(receipt["currentClaimant"], "mark@beta");
    assert_eq!(receipt["claimant"], CLAIMANT);
    assert!(
        receipt["cannotPredict"]
            .as_array()
            .is_some_and(|list| !list.is_empty())
    );
    assert_eq!(bd_steps(&fixture), ["show wb-1 --json"]);
    assert_matches_output_schema(&["beads", "claim"], &receipt);
}

#[test]
fn a_dry_run_of_an_unheld_item_reports_no_current_claimant() {
    let fixture = project(&replica().on("show", open_item("wb-1")));

    let receipt = fixture.run(&["beads", "claim", "wb-1", "--dry-run"]).json();

    assert_eq!(receipt["currentClaimant"], json!(null));
    assert_eq!(receipt["itemStatus"], "open");
}

#[test]
fn a_dry_run_for_an_unknown_item_is_not_found() {
    let fixture = project(&replica().on(
        "show",
        Reply::json(json!({ "error": "no issues found matching the provided IDs" })).exit(1),
    ));

    let outcome = fixture.run(&["beads", "claim", "wb-9", "--dry-run"]);

    assert_eq!(outcome.error_code(), "NOT_FOUND");
}

#[test]
fn the_claim_exits_zero_when_claimed_and_nonzero_when_held_saying_the_outcome() {
    let claimed = project(
        &replica()
            .on("sync", synced())
            .on("update", Reply::json(json!([{ "id": "wb-1" }])))
            .on("show", item("wb-1", "in_progress", CLAIMANT)),
    );
    let held = project(&held_by_another());

    let claimed_outcome = claimed.run(&["beads", "claim", "wb-1", "--output", "text"]);
    let held_outcome = held.run(&["beads", "claim", "wb-1", "--output", "text"]);

    assert_eq!(claimed_outcome.status, 0, "{}", claimed_outcome.stderr);
    assert!(
        claimed_outcome.stdout.contains("claimed"),
        "{}",
        claimed_outcome.stdout
    );
    assert_ne!(held_outcome.status, 0);
    assert!(
        held_outcome.stderr.contains("held"),
        "{}",
        held_outcome.stderr
    );
    assert!(
        held_outcome.stderr.contains("mark@beta"),
        "{}",
        held_outcome.stderr
    );
}
