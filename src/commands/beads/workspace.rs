//! Whether Beads is usable here, and who this machine claims as.

use std::path::Path;

use serde_json::json;

use super::bd::{bd, bd_error, interruption, parse_json};
use crate::failure::{ErrorCode, Failure};
use crate::process::{Program, RunError};

/// Why Beads cannot be used here.
pub struct Unavailable {
    pub reason: String,
    /// `bd` itself is absent, rather than a workspace or its database.
    pub missing_bd: bool,
}

/// Checks for `bd`, a Beads workspace, and its database.
pub fn check_workspace(root: &Path) -> Result<(), Unavailable> {
    let where_ = match bd(root, &["where", "--json"]).run() {
        Ok(run) => run,
        Err(error) => {
            return Err(Unavailable {
                reason: interruption(&error, "where"),
                missing_bd: matches!(error, RunError::Missing | RunError::Spawn(_)),
            });
        }
    };
    if !where_.success() {
        return Err(Unavailable {
            reason: bd_error(&where_, "where"),
            missing_bd: false,
        });
    }
    let workspace = parse_json(&where_.stdout).unwrap_or_default();
    if workspace["database_path"].as_str().is_some() {
        return Ok(());
    }
    let place = workspace["path"]
        .as_str()
        .map(|path| format!(" in {path}"))
        .unwrap_or_default();
    Err(Unavailable {
        reason: format!("the Beads database{place} is not initialized"),
        missing_bd: false,
    })
}

impl Unavailable {
    pub fn to_failure(&self) -> Failure {
        if self.missing_bd {
            return Failure::dependency_missing("bd").with_details(json!({
                "program": "bd",
                "reason": self.reason,
            }));
        }
        Failure::new(ErrorCode::PreconditionFailed, self.reason.clone())
            .with_details(json!({ "outcome": "unavailable" }))
            .suggest("Run `mise run setup`, which restores or creates this machine's Beads replica")
    }
}

/// The claimant this machine acts as: `BEADS_ACTOR`, or `<login user>@<short host name>` when
/// it is unset or empty.
pub fn claimant(root: &Path) -> Result<String, Failure> {
    if let Ok(actor) = std::env::var("BEADS_ACTOR")
        && !actor.is_empty()
    {
        return Ok(actor);
    }
    let user = first_line(root, "id", "-un")?;
    let host = first_line(root, "uname", "-n")?;
    let short_host = host.split('.').next().unwrap_or(&host);
    Ok(format!("{user}@{short_host}"))
}

fn first_line(root: &Path, program: &str, flag: &str) -> Result<String, Failure> {
    let output = Program::new(program, root)
        .arg(flag)
        .run()
        .ok()
        .filter(|run| run.success())
        .map(|run| run.stdout.trim().to_string())
        .filter(|line| !line.is_empty());
    output.ok_or_else(|| {
        Failure::precondition(format!(
            "cannot determine this machine's claimant: `{program} {flag}` failed"
        ))
        .suggest("Set BEADS_ACTOR to this machine's claimant, such as <user>@<short host name>")
    })
}
