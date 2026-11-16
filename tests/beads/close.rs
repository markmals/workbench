//! `workbench beads close`: closing an item as this machine's claimant, visibly to the others.

use serde_json::json;

use super::{CLAIMANT, message, project, project_with_remote, replica, set_origin};
use crate::support::assert_matches_output_schema;
use crate::support::fake_bd::{FakeBd, Reply, bd_calls, bd_steps, item, remotes, synced};

fn closes() -> FakeBd {
    replica()
        .on(
            "close",
            Reply::json(json!([{ "id": "wb-1", "status": "closed" }])),
        )
        .on("sync", synced())
        .on("show", item("wb-1", "in_progress", CLAIMANT))
}

fn refused_as_held() -> FakeBd {
    replica()
        .on(
            "close",
            Reply::json(json!({
                "error": "cannot close wb-1: assignee is \"mark@beta\", actor is \"mark@alpha\"",
            }))
            .exit(1),
        )
        .on("sync", synced())
        .on("show", item("wb-1", "in_progress", "mark@beta"))
}

#[test]
fn closing_an_item_closes_it_as_this_machines_claimant_then_publishes_it() {
    let fixture = project(&closes());

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "Fixed in PR #42"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "closed");
    assert_eq!(receipt["id"], "wb-1");
    assert_eq!(receipt["claimant"], CLAIMANT);
    assert_eq!(receipt["published"], true);
    assert_matches_output_schema(&["beads", "close"], &receipt);
    assert_eq!(
        bd_steps(&fixture),
        ["close wb-1 --reason=Fixed in PR #42 --json", "sync --json"]
    );
    let close = bd_calls(&fixture)
        .into_iter()
        .find(|call| call.args[0] == "close")
        .unwrap();
    assert_eq!(close.actor.as_deref(), Some(CLAIMANT));
    assert!(close.args.contains(&"--reason=Fixed in PR #42".to_string()));
}

#[test]
fn a_close_whose_sync_fails_is_closed_on_this_machine_only() {
    let fixture = project_with_remote(
        true,
        closes().on(
            "sync",
            Reply::json(json!({ "error": "sync failed: remote unreachable" })).exit(1),
        ),
    );

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "Done"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "closed-unpublished");
    assert_eq!(receipt["published"], false);
    assert!(
        outcome.stderr.contains("remote unreachable"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn a_close_refused_because_another_claimant_holds_the_item_is_a_conflict_and_does_not_sync() {
    let fixture = project(&refused_as_held());

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "Done"]);

    assert_eq!(outcome.status, 4);
    let error = outcome.error();
    assert_eq!(error["code"], "CONFLICT");
    assert_eq!(error["details"]["holder"], "mark@beta");
    assert!(message(&outcome).contains("assignee is \"mark@beta\""));
    assert!(
        !bd_steps(&fixture)
            .iter()
            .any(|step| step.starts_with("sync")),
        "{:?}",
        bd_steps(&fixture)
    );
}

#[test]
fn closing_an_unknown_item_is_not_found() {
    let fixture = project(
        &replica()
            .on(
                "close",
                Reply::json(
                    json!({ "error": "resolving ID wb-9: no issue found matching \"wb-9\"" }),
                )
                .exit(1),
            )
            .on(
                "show",
                Reply::json(json!({ "error": "no issues found matching the provided IDs" }))
                    .exit(1),
            ),
    );

    let outcome = fixture.run(&["beads", "close", "wb-9", "--reason", "Done"]);

    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "NOT_FOUND");
}

#[test]
fn closing_in_a_replica_whose_beads_remote_is_another_repository_changes_nothing() {
    let fixture = project(&closes().on(
        "dolt remote list",
        remotes("git+https://github.com/markmals/workbench.git"),
    ));
    set_origin(&fixture, "https://github.com/someone/new-project.git");

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "Done"]);

    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
    assert!(message(&outcome).contains("not this project's origin"));
    assert!(
        !bd_steps(&fixture)
            .iter()
            .any(|step| step.starts_with("close")),
        "{:?}",
        bd_steps(&fixture)
    );
}

#[test]
fn the_close_exits_zero_when_closed_and_nonzero_when_beads_refuses_saying_the_outcome() {
    let closed = project(&closes());
    let refused = project(&refused_as_held());

    let closed_outcome = closed.run(&[
        "beads",
        "close",
        "wb-1",
        "--reason",
        "Fixed in PR #42",
        "--output",
        "text",
    ]);
    let refused_outcome = refused.run(&[
        "beads",
        "close",
        "wb-1",
        "--reason",
        "Fixed in PR #42",
        "--output",
        "text",
    ]);

    assert_eq!(closed_outcome.status, 0, "{}", closed_outcome.stderr);
    assert!(
        closed_outcome.stdout.contains("closed"),
        "{}",
        closed_outcome.stdout
    );
    assert_ne!(refused_outcome.status, 0);
    assert!(
        refused_outcome.stderr.contains("cannot close wb-1"),
        "{}",
        refused_outcome.stderr
    );
}

#[test]
fn closing_without_a_reason_is_an_invalid_argument_that_closes_nothing() {
    let fixture = project(&closes());

    let outcome = fixture.run(&["beads", "close", "wb-1"]);

    assert_eq!(outcome.status, 2);
    let error = outcome.error();
    assert_eq!(error["code"], "INVALID_ARGUMENT");
    assert!(
        error["details"]["missing"].to_string().contains("reason"),
        "{error}"
    );
    assert!(bd_calls(&fixture).is_empty(), "{:?}", bd_calls(&fixture));
}

#[test]
fn closing_with_an_empty_reason_is_an_invalid_argument() {
    let fixture = project(&closes());

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "  "]);

    assert_eq!(outcome.status, 2);
    assert_eq!(outcome.error_code(), "INVALID_ARGUMENT");
    assert_eq!(outcome.error()["details"]["argument"], "reason");
    assert!(bd_calls(&fixture).is_empty(), "{:?}", bd_calls(&fixture));
}

#[test]
fn a_dry_run_reports_the_item_its_current_claimant_and_the_claimant_it_would_close_as() {
    let fixture = project(&refused_as_held());

    let outcome = fixture.run(&["beads", "close", "wb-1", "--reason", "Done", "--dry-run"]);

    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let receipt = outcome.json();
    assert_eq!(receipt["status"], "dry-run");
    assert_eq!(receipt["id"], "wb-1");
    assert_eq!(receipt["currentClaimant"], "mark@beta");
    assert_eq!(receipt["claimant"], CLAIMANT);
    assert!(receipt["cannotPredict"].is_array());
    assert_eq!(bd_steps(&fixture), ["show wb-1 --json"]);
    assert_matches_output_schema(&["beads", "close"], &receipt);
}
