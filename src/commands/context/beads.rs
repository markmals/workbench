//! The project's Beads workspace, its Dolt remote, and who claims items from this machine.

use std::path::Path;
use std::time::Instant;

use serde::Deserialize;

use super::report::{Beads, BeadsRemote};
use crate::commands::beads::repository::without_credentials;
use crate::failure::Failure;
use crate::process::{Program, RunError};

pub fn beads(root: &Path, deadline: Instant, timeout: u64) -> Result<Beads, Failure> {
    let workspace = root.join(".beads").is_dir().then(|| ".beads".to_string());
    let remote = match workspace {
        None => BeadsRemote::Unknown {
            reason: "the project has no Beads workspace (.beads/)".into(),
        },
        Some(_) => dolt_remote(root, deadline, timeout),
    };
    Ok(Beads {
        workspace,
        remote,
        claimant: claimant(root)?,
    })
}

#[derive(Deserialize)]
struct Remote {
    name: String,
    url: String,
}

/// The workspace's `origin` Dolt remote, the one Beads syncs with.
fn dolt_remote(root: &Path, deadline: Instant, timeout: u64) -> BeadsRemote {
    let unknown = |reason: String| BeadsRemote::Unknown { reason };
    let listing = Program::new("bd", root)
        .args(["dolt", "remote", "list", "--json"])
        .env("DOLT_REMOTE_INFO_BRANCH", "")
        .deadline(deadline)
        .run();
    let run = match listing {
        Ok(run) if run.success() => run,
        Ok(run) => {
            let detail = run.stderr.lines().next().unwrap_or("no output").trim();
            return unknown(format!("`bd dolt remote list` failed: {detail}"));
        }
        Err(RunError::Missing) => return unknown("`bd` is not installed".into()),
        Err(RunError::TimedOut) => {
            return unknown(format!(
                "`bd` did not answer within {timeout} second{}",
                if timeout == 1 { "" } else { "s" }
            ));
        }
        Err(RunError::Spawn(reason)) => return unknown(format!("`bd` could not start: {reason}")),
    };
    match serde_json::from_str::<Vec<Remote>>(&run.stdout) {
        Ok(remotes) => remotes
            .into_iter()
            .find(|remote| remote.name == "origin")
            .map_or(BeadsRemote::None, |remote| BeadsRemote::Configured {
                url: without_credentials(&remote.url),
            }),
        Err(_) => unknown("`bd` listed Dolt remotes in a form workbench cannot read".into()),
    }
}

/// `BEADS_ACTOR`, or `<login user>@<short host name>` when it is unset or empty.
fn claimant(root: &Path) -> Result<String, Failure> {
    if let Ok(actor) = std::env::var("BEADS_ACTOR")
        && !actor.is_empty()
    {
        return Ok(actor);
    }
    let user = output(root, "id", &["-un"])?;
    let host = output(root, "uname", &["-n"])?;
    let short_host = host.split('.').next().unwrap_or(&host);
    Ok(format!("{user}@{short_host}"))
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, Failure> {
    match Program::new(program, root).args(args.iter().copied()).run() {
        Ok(run) if run.success() => Ok(run.stdout.trim().to_string()),
        _ => Err(Failure::internal(format!(
            "`{program} {}` did not report the claimant's identity",
            args.join(" ")
        ))),
    }
}
