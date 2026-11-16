//! `workbench context`: the project's Workbench context.

mod beads;
mod github;
mod report;
mod site;

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use schemars::schema_for;

use crate::commands::project::setup;
use crate::failure::{ErrorCode, Failure};
use crate::invocation::Invocation;
use crate::registry::{
    ArgumentKind, ArgumentSpec, CommandSpec, Effect, Example, Requirement, ValueKind,
};
use crate::response::Response;
use report::{Context, Harnesses, ProjectLink, Workbench};

pub static COMMANDS: &[CommandSpec] = &[CommandSpec {
    id: "context.show",
    path: &["context", "show"],
    summary: "Show the project, branch, pull request, Workbench pin and link, harnesses, Beads, and site",
    description: "Reports the project's Workbench context without judging or changing it: the \
        project root (the Git top level of the current directory), the branch (null when HEAD is \
        detached), and the `origin` URL; the branch's pull request from `gh`, or `unknown` with the \
        reason (no branch, no `origin`, `gh` missing or unauthenticated, no answer within the time \
        limit, or several pull requests for the branch); the `github:markmals/workbench` version \
        `mise.toml` pins, this binary's version, and where `.workbench` points with the package \
        version there (in Workbench's own repository it links to the working tree's `plugin/` and \
        nothing is pinned); whether `.claude/settings.json` and `.omp/config.yml` declare the \
        package, and the `.claude/skills` and `.claude/rules` links to the project's own skills \
        and rules; the Beads workspace, its Dolt remote as `bd` reports it, and the claimant \
        claims and closes act as (`BEADS_ACTOR`, or `<login user>@<short host name>`); and the \
        documentation site: `docs/hugo.toml` and the workflows that build it, with their host. A \
        missing `gh` or `bd` makes the affected field `unknown` rather than failing. `gh` and `bd` \
        together get `--timeout` seconds. Use `workbench project validate` to check the setup.",
    arguments: &[TIMEOUT],
    effect: Effect::Reads,
    requires: &[Requirement::Git],
    dry_run: false,
    idempotent: true,
    errors: &[
        ErrorCode::InvalidArgument,
        ErrorCode::PreconditionFailed,
        ErrorCode::DependencyMissing,
    ],
    examples: &[Example {
        summary: "Show the project's Workbench context",
        args: &["context", "show"],
    }],
    synonyms: &[
        "info",
        "environment",
        "where",
        "version",
        "pin",
        "link",
        "branch",
        "pull",
        "request",
        "claimant",
        "remote",
        "harness",
        "orient",
    ],
    output_schema: || schema_for!(report::Context),
    run: show,
}];

const TIMEOUT: ArgumentSpec = ArgumentSpec {
    name: "timeout",
    kind: ArgumentKind::Option,
    value: ValueKind::Integer,
    required: false,
    default: Some("10"),
    multiple: false,
    summary: "Seconds to wait for `gh` and `bd`; past it, what they would report is unknown",
};

fn show(invocation: &Invocation) -> Result<Response, Failure> {
    let project = invocation.project()?;
    let root = project.root.as_path();
    let timeout = invocation
        .integer("timeout")
        .expect("timeout has a default");
    let deadline = Instant::now() + Duration::from_secs(timeout);

    let branch = github::branch(root);
    let origin = setup::origin(root);
    let pull_request =
        github::pull_request(root, branch.as_deref(), origin.is_some(), deadline, timeout);
    let context = Context {
        project: root.display().to_string(),
        branch,
        origin,
        pull_request,
        workbench: Workbench {
            pin: setup::read_pin(root),
            binary: env!("CARGO_PKG_VERSION").to_string(),
            link: setup::read_link(root),
        },
        harnesses: Harnesses {
            claude_code: setup::claude_declaration(root),
            oh_my_pi: setup::omp_declaration(root),
            project_links: project_links(root),
        },
        beads: beads::beads(root, deadline, timeout)?,
        site: site::site(root),
    };
    Ok(Response::new(&context))
}

fn project_links(root: &Path) -> Vec<ProjectLink> {
    ["skills", "rules"]
        .into_iter()
        .map(|kind| {
            let path = format!(".claude/{kind}");
            let target = fs::read_link(root.join(&path))
                .ok()
                .map(|target| target.display().to_string());
            ProjectLink { path, target }
        })
        .collect()
}
