//! The files `project init` creates when they are absent, from the templates built into the
//! binary under `src/commands/project/templates/`.

use std::time::{SystemTime, UNIX_EPOCH};

/// A documentation-site host `--site` accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    GithubPages,
    Cloudflare,
}

impl Site {
    pub fn parse(name: &str) -> Option<Site> {
        match name {
            "github-pages" => Some(Site::GithubPages),
            "cloudflare" => Some(Site::Cloudflare),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Site::GithubPages => "github-pages",
            Site::Cloudflare => "cloudflare",
        }
    }
}

/// What the templates are filled in with.
pub struct Project {
    /// The name of the project's directory.
    pub name: String,
    /// The repository's web address, derived from `origin`.
    pub repository: Option<String>,
    /// The vision template: the project's override, or the package's.
    pub vision: Option<String>,
    /// Today, as `YYYY-MM-DD`.
    pub today: String,
}

/// Every file `project init` creates when absent, in the order the receipt lists them.
pub fn files(project: &Project, site: Option<Site>) -> Vec<(&'static str, Option<String>)> {
    let mut files = vec![
        (
            "hk.pkl",
            Some(include_str!("../templates/hk.pkl").to_string()),
        ),
        (
            ".github/workflows/workbench.yml",
            Some(include_str!("../templates/workbench.yml").to_string()),
        ),
        (
            "AGENTS.md",
            Some(include_str!("../templates/AGENTS.md").replace("{{project}}", &project.name)),
        ),
        (
            "VISION.md",
            project
                .vision
                .as_ref()
                .map(|vision| vision.replace("<YYYY-MM-DD>", &project.today)),
        ),
        (
            "proposals/README.md",
            Some(include_str!("../templates/proposals-README.md").to_string()),
        ),
        (
            "policies/README.md",
            Some(include_str!("../templates/policies-README.md").to_string()),
        ),
        (
            "decisions/README.md",
            Some(include_str!("../templates/decisions-README.md").to_string()),
        ),
        (
            "guides/README.md",
            Some(include_str!("../templates/guides-README.md").to_string()),
        ),
        (
            ".agents/commit-scopes",
            Some(include_str!("../templates/commit-scopes").to_string()),
        ),
        (
            "docs/hugo.toml",
            Some(
                include_str!("../templates/hugo.toml")
                    .replace("{{project}}", &project.name)
                    .replace(
                        "{{repository}}",
                        project.repository.as_deref().unwrap_or_default(),
                    ),
            ),
        ),
    ];
    if let Some(site) = site {
        files.extend(site_files(project, site));
    }
    files
}

/// The host's publishing and preview workflows and what the site build needs beside them.
fn site_files(project: &Project, site: Site) -> Vec<(&'static str, Option<String>)> {
    let (publish, preview) = match site {
        Site::GithubPages => (
            include_str!("../templates/github-pages-docs.yml"),
            include_str!("../templates/github-pages-preview.yml"),
        ),
        Site::Cloudflare => (
            include_str!("../templates/cloudflare-docs.yml"),
            include_str!("../templates/cloudflare-preview.yml"),
        ),
    };
    let mut files = vec![
        (".github/workflows/docs.yml", Some(publish.to_string())),
        (".github/workflows/preview.yml", Some(preview.to_string())),
        (
            "docs/.gitignore",
            Some(include_str!("../templates/docs-gitignore").to_string()),
        ),
    ];
    if site == Site::Cloudflare {
        let wrangler = include_str!("../templates/wrangler.jsonc")
            .replace("{{worker}}", &worker_name(&project.name))
            .replace("{{today}}", &project.today);
        files.push(("docs/wrangler.jsonc", Some(wrangler)));
    }
    files
}

/// The Cloudflare Worker that serves the site: the project's name, lowercase, with every run of
/// other characters as one hyphen, then `-docs`.
fn worker_name(project: &str) -> String {
    let mut name = String::new();
    for character in project.chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character.to_ascii_lowercase());
        } else if !name.is_empty() && !name.ends_with('-') {
            name.push('-');
        }
    }
    let name = name.trim_end_matches('-');
    if name.is_empty() {
        "docs".to_string()
    } else {
        format!("{name}-docs")
    }
}

/// The web address of a GitHub-style remote: `git@github.com:owner/repo.git` and
/// `https://user@github.com/owner/repo.git` both give `https://github.com/owner/repo`.
pub fn repository_url(remote: &str) -> Option<String> {
    let remote = remote.trim();
    let (host, path) = if let Some(rest) = remote.strip_prefix("https://") {
        let rest = rest.rsplit_once('@').map_or(rest, |(_, after)| after);
        rest.split_once('/')?
    } else if let Some(rest) = remote.strip_prefix("ssh://") {
        let rest = rest.split_once('@').map_or(rest, |(_, after)| after);
        rest.split_once('/')?
    } else {
        let (user_host, path) = remote.split_once(':')?;
        (user_host.split_once('@')?.1, path)
    };
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    (!host.is_empty() && path.contains('/')).then(|| format!("https://{host}/{path}"))
}

/// Today's date in UTC, as `YYYY-MM-DD`.
pub fn today() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    civil_date((seconds / 86_400) as i64)
}

/// The calendar date `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_date(days: i64) -> String {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_days_since_the_epoch_to_dates() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(11_017), "2000-03-01");
        assert_eq!(civil_date(19_782), "2024-02-29");
        assert_eq!(civil_date(-1), "1969-12-31");
    }

    #[test]
    fn names_the_worker_after_the_project() {
        assert_eq!(worker_name("Our_Project.v2"), "our-project-v2-docs");
        assert_eq!(worker_name("api"), "api-docs");
        assert_eq!(worker_name("--"), "docs");
    }

    #[test]
    fn derives_the_web_address_of_a_remote() {
        for remote in [
            "git@github.com:owner/repo.git",
            "https://github.com/owner/repo.git",
            "https://token@github.com/owner/repo",
            "ssh://git@github.com/owner/repo.git",
        ] {
            assert_eq!(
                repository_url(remote).as_deref(),
                Some("https://github.com/owner/repo"),
                "{remote}"
            );
        }
        assert_eq!(repository_url("/srv/git/repo.git"), None);
        assert_eq!(repository_url("../repo"), None);
    }
}
