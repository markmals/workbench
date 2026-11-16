//! The documentation site's configuration: `docs/hugo.toml` and the workflows that build it.

use std::fs;
use std::path::Path;

use super::report::{Site, SiteHost, SiteWorkflow};

pub fn site(root: &Path) -> Site {
    let config = root
        .join("docs/hugo.toml")
        .is_file()
        .then(|| "docs/hugo.toml".to_string());
    Site {
        config,
        workflows: workflows(root),
    }
}

/// The GitHub workflows that run `docs:build`, by file name.
fn workflows(root: &Path) -> Vec<SiteWorkflow> {
    let Ok(listing) = fs::read_dir(root.join(".github/workflows")) else {
        return Vec::new();
    };
    let mut names: Vec<String> = listing
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".yml") || name.ends_with(".yaml"))
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let path = format!(".github/workflows/{name}");
            let text = fs::read_to_string(root.join(&path)).ok()?;
            text.contains("docs:build").then(|| SiteWorkflow {
                host: host(&text),
                path,
            })
        })
        .collect()
}

/// The host a workflow publishes to, judged by the actions its steps use, not its comments.
fn host(workflow: &str) -> SiteHost {
    let steps: String = workflow
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    if steps.contains("cloudflare/wrangler-action") {
        SiteHost::Cloudflare
    } else if ["gh-pages", "actions/deploy-pages", "pr-preview-action"]
        .iter()
        .any(|marker| steps.contains(marker))
    {
        SiteHost::GithubPages
    } else {
        SiteHost::Unknown
    }
}
