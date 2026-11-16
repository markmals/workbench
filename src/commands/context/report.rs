//! The result of `workbench context show` and its text rendering.

use schemars::JsonSchema;
use serde::Serialize;

use crate::commands::project::setup::{Declaration, DeclarationState, Link, LinkState, Pin};
use crate::response::RenderText;

/// The project's Workbench context.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// The project root: the Git top level of the current directory.
    pub project: String,
    /// The checked-out branch, or null when HEAD is detached.
    pub branch: Option<String>,
    /// The URL of the `origin` remote, or null when there is none.
    pub origin: Option<String>,
    pub pull_request: PullRequest,
    pub workbench: Workbench,
    pub harnesses: Harnesses,
    pub beads: Beads,
    pub site: Site,
}

/// The branch's pull request.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum PullRequest {
    Found {
        number: u64,
        url: String,
        title: String,
        /// `OPEN`, `CLOSED`, or `MERGED`.
        state: String,
        draft: bool,
    },
    /// GitHub has no pull request for the branch.
    None,
    /// The pull request could not be determined.
    Unknown { reason: String },
}

#[derive(Serialize, JsonSchema)]
pub struct Workbench {
    /// The `github:markmals/workbench` version `mise.toml` pins.
    pub pin: Pin,
    /// The version of the `workbench` binary that ran this command.
    pub binary: String,
    pub link: Link,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Harnesses {
    /// Claude Code's declaration of the package in `.claude/settings.json`.
    pub claude_code: Declaration,
    /// Oh My Pi's declaration of the package in `.omp/config.yml`.
    pub oh_my_pi: Declaration,
    /// `.claude/skills` and `.claude/rules`, through which Claude Code loads the project's own
    /// skills and rules from `.agents/`.
    pub project_links: Vec<ProjectLink>,
}

#[derive(Serialize, JsonSchema)]
pub struct ProjectLink {
    pub path: String,
    /// Where the link points, as written; null when there is no link at the path.
    pub target: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct Beads {
    /// `.beads` when the project has a Beads workspace, otherwise null.
    pub workspace: Option<String>,
    /// The workspace's Dolt remote, which Beads syncs with.
    pub remote: BeadsRemote,
    /// Who `workbench beads claim` and `close` act as on this machine.
    pub claimant: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum BeadsRemote {
    /// The URL of the workspace's `origin` Dolt remote.
    Configured {
        url: String,
    },
    /// The workspace has no `origin` Dolt remote yet.
    None,
    Unknown {
        reason: String,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct Site {
    /// `docs/hugo.toml` when present, otherwise null.
    pub config: Option<String>,
    /// The GitHub workflows that build the site (they run `docs:build`).
    pub workflows: Vec<SiteWorkflow>,
}

#[derive(Serialize, JsonSchema)]
pub struct SiteWorkflow {
    pub path: String,
    pub host: SiteHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SiteHost {
    GithubPages,
    Cloudflare,
    /// The workflow builds the site but publishes it nowhere Workbench recognizes.
    Unknown,
}

impl RenderText for Context {
    fn render_text(&self) -> String {
        let rows = [
            ("Project", self.project.clone()),
            (
                "Branch",
                self.branch
                    .clone()
                    .unwrap_or_else(|| "(detached HEAD)".into()),
            ),
            (
                "Origin",
                self.origin.clone().unwrap_or_else(|| "none".into()),
            ),
            ("Pull request", pull_request_text(&self.pull_request)),
            ("Workbench", workbench_text(&self.workbench)),
            (".workbench", link_text(&self.workbench.link)),
            ("Claude Code", declaration_text(&self.harnesses.claude_code)),
            ("Oh My Pi", declaration_text(&self.harnesses.oh_my_pi)),
            (
                "Project links",
                project_links_text(&self.harnesses.project_links),
            ),
            ("Beads", beads_text(&self.beads)),
            ("Site", site_text(&self.site)),
        ];
        let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0) + 1;
        rows.iter()
            .map(|(label, value)| format!("{:width$} {value}\n", format!("{label}:")))
            .collect()
    }
}

fn pull_request_text(pull_request: &PullRequest) -> String {
    match pull_request {
        PullRequest::Found {
            number,
            url,
            title,
            state,
            draft,
        } => {
            let draft = if *draft { ", draft" } else { "" };
            format!("#{number} {title} ({state}{draft}) {url}")
        }
        PullRequest::None => "none for this branch".into(),
        PullRequest::Unknown { reason } => format!("unknown: {reason}"),
    }
}

fn workbench_text(workbench: &Workbench) -> String {
    let pin = match &workbench.pin {
        Pin::Pinned { version } => format!("pinned {version}"),
        Pin::Unpinned => "not pinned".into(),
        Pin::Unreadable { reason } => reason.clone(),
    };
    format!("{pin}; this binary {}", workbench.binary)
}

fn link_text(link: &Link) -> String {
    let target = link.target.as_deref().unwrap_or("nothing");
    match link.state {
        LinkState::Missing => "missing".into(),
        LinkState::Dangling => format!("dangling: points at {target}"),
        LinkState::Linked => {
            let version = link.version.as_deref().unwrap_or("unknown version");
            let working_tree = if link.working_tree {
                ", this repository's working tree"
            } else {
                ""
            };
            format!("{target} (Workbench {version}{working_tree})")
        }
    }
}

fn declaration_text(declaration: &Declaration) -> String {
    let state = match declaration.state {
        DeclarationState::Declared => "declares Workbench",
        DeclarationState::Missing => "missing",
        DeclarationState::Incomplete => "incomplete",
        DeclarationState::Unreadable => "unreadable",
    };
    match &declaration.detail {
        Some(detail) => format!("{} {state}: {detail}", declaration.path),
        None => format!("{} {state}", declaration.path),
    }
}

fn project_links_text(links: &[ProjectLink]) -> String {
    links
        .iter()
        .map(|link| match &link.target {
            Some(target) => format!("{} -> {target}", link.path),
            None => format!("{} absent", link.path),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn beads_text(beads: &Beads) -> String {
    let workspace = beads.workspace.as_deref().unwrap_or("no workspace");
    let remote = match &beads.remote {
        BeadsRemote::Configured { url } => format!("remote {url}"),
        BeadsRemote::None => "no remote".into(),
        BeadsRemote::Unknown { reason } => format!("remote unknown: {reason}"),
    };
    format!("{workspace}; {remote}; claimant {}", beads.claimant)
}

fn site_text(site: &Site) -> String {
    let mut parts = vec![
        site.config
            .clone()
            .unwrap_or_else(|| "no docs/hugo.toml".into()),
    ];
    for workflow in &site.workflows {
        let host = match workflow.host {
            SiteHost::GithubPages => "GitHub Pages",
            SiteHost::Cloudflare => "Cloudflare",
            SiteHost::Unknown => "unknown host",
        };
        parts.push(format!("{} ({host})", workflow.path));
    }
    parts.join("; ")
}
