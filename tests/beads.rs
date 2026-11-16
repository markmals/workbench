//! `workbench beads sync|claim|close`: this machine's Beads replica and the project's remote.

mod support;

#[path = "beads/claim.rs"]
mod claim;
#[path = "beads/close.rs"]
mod close;
#[path = "beads/sync.rs"]
mod sync;

use support::Fixture;
use support::fake_bd::{FakeBd, available, bare_remote, remotes};

/// The claimant every Beads test acts as, unless it says otherwise.
const CLAIMANT: &str = "mark@alpha";

/// A scripted replica in an initialized workspace; tests add the replies they need.
fn replica() -> FakeBd {
    FakeBd::new().on("where", available())
}

/// A Git checkout without an `origin` whose `bd` is `beads`, acting as [`CLAIMANT`].
fn project(beads: &FakeBd) -> Fixture {
    let mut fixture = Fixture::git();
    beads.install(&fixture);
    fixture.set_env("BEADS_ACTOR", CLAIMANT);
    fixture
}

/// A project whose replica's Beads remote and Git `origin` are the same local bare repository.
fn project_with_remote(with_beads_history: bool, beads: FakeBd) -> Fixture {
    let fixture = project(&FakeBd::new());
    let url = bare_remote(&fixture, with_beads_history);
    set_origin(&fixture, url.trim_start_matches("git+"));
    beads
        .on("dolt remote list", remotes(&url))
        .install(&fixture);
    fixture
}

fn set_origin(fixture: &Fixture, url: &str) {
    let verb = if fixture.git_command(&["remote"]).trim().is_empty() {
        "add"
    } else {
        "set-url"
    };
    fixture.git_command(&["remote", verb, "origin", url]);
}

fn message(outcome: &support::Outcome) -> String {
    outcome.error()["message"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}
