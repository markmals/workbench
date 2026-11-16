//! Which repository this replica syncs with, whether that is the project's own, and whether
//! it holds Beads history yet.

use std::path::Path;
use std::time::Instant;

use serde_json::json;

use super::bd::{bd, parse_json};
use super::repository::{repository_identity, without_credentials};
use crate::failure::Failure;
use crate::process::Program;

/// The replica's Beads remote and the checkout's Git `origin`, as written.
pub struct Remotes {
    pub beads: Option<String>,
    pub origin: Option<String>,
}

/// Why this replica must not sync with its Beads remote.
#[derive(Debug)]
pub enum Refusal {
    /// The checkout has no `origin` to compare the Beads remote with.
    NoOrigin { beads: String },
    /// The Beads remote is another repository than `origin`.
    Foreign { beads: String, origin: String },
}

pub fn read_remotes(root: &Path, deadline: Instant) -> Remotes {
    Remotes {
        beads: dolt_remote_url(root, deadline),
        origin: git_origin(root),
    }
}

/// The URL of the replica's `origin` Dolt remote, the one `bd sync` uses, if it has one.
pub fn dolt_remote_url(root: &Path, deadline: Instant) -> Option<String> {
    let run = bd(root, &["dolt", "remote", "list", "--json"])
        .deadline(deadline)
        .run()
        .ok()?;
    let remotes = parse_json(&run.stdout)?;
    let origin = remotes
        .as_array()?
        .iter()
        .find(|remote| remote["name"] == "origin")?;
    origin["url"].as_str().map(str::to_string)
}

fn git_origin(root: &Path) -> Option<String> {
    let run = Program::new("git", root)
        .args(["remote", "get-url", "origin"])
        .run()
        .ok()?;
    run.success().then(|| run.stdout.trim().to_string())
}

impl Remotes {
    /// Why this replica must not sync: its Beads remote is not the checkout's own `origin`, as
    /// when a project created from the Workbench template still carries Workbench's `.beads/`,
    /// or there is no `origin` to compare it with. `None` when the replica has no Beads remote
    /// yet (`bd sync` then adopts `origin`) or when the two are the same repository.
    pub fn refusal(&self, root: &Path) -> Option<Refusal> {
        let beads = self.beads.clone()?;
        let Some(origin) = self.origin.clone() else {
            return Some(Refusal::NoOrigin { beads });
        };
        (repository_identity(&beads, root) != repository_identity(&origin, root))
            .then_some(Refusal::Foreign { beads, origin })
    }

    /// The remote a sync uses, without credentials: the replica's Beads remote, or the
    /// checkout's `origin`, which `bd sync` adopts when the replica has none.
    pub fn target(&self) -> Option<String> {
        self.beads
            .as_deref()
            .or(self.origin.as_deref())
            .map(without_credentials)
    }
}

impl Refusal {
    pub fn message(&self) -> String {
        match self {
            Refusal::NoOrigin { beads } => format!(
                "This replica's Beads remote is {}, but this checkout has no origin to compare it \
                 with; nothing was pulled or pushed.",
                without_credentials(beads)
            ),
            Refusal::Foreign { beads, origin } => format!(
                "This replica's Beads remote is {}, not this project's origin ({}); nothing was \
                 pulled or pushed. The project's .beads/ belongs to another repository.",
                without_credentials(beads),
                without_credentials(origin)
            ),
        }
    }

    pub fn to_failure(&self) -> Failure {
        let (details, suggestion) = match self {
            Refusal::NoOrigin { beads } => (
                json!({ "beadsRemote": without_credentials(beads), "origin": null }),
                "Ask the human to add the project's origin; until then no sync runs",
            ),
            Refusal::Foreign { beads, origin } => (
                json!({
                    "beadsRemote": without_credentials(beads),
                    "origin": without_credentials(origin),
                }),
                "Tell the human: replacing the project's .beads/ with its own tracker is their decision",
            ),
        };
        Failure::precondition(self.message())
            .with_details(details)
            .suggest(suggestion)
    }
}

/// True only when the remote is reachable and positively has no `refs/dolt/data`.
pub fn remote_lacks_beads_history(root: &Path, deadline: Instant) -> bool {
    let Some(url) = dolt_remote_url(root, deadline) else {
        return false;
    };
    let repository = url.strip_prefix("git+").unwrap_or(&url);
    Program::new("git", root)
        .args(["ls-remote", repository, "refs/dolt/data"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .deadline(deadline)
        .run()
        .is_ok_and(|listing| listing.success() && listing.stdout.trim().is_empty())
}
