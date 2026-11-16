//! Repository locations: one spelling for every way of writing one, and a form safe to show.

use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use regex::{Captures, Regex};

static SCP_LIKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^/@:]+@([^:/]+):(.+)$").expect("valid regex"));
static SCHEME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z][A-Za-z0-9+.-]*):(.*)$").expect("valid regex"));
static USERINFO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^((?:git\+)?([A-Za-z][A-Za-z0-9+.-]*)://)([^/@:]*)(?::[^/@]*)?@")
        .expect("valid regex")
});

/// One spelling for every way of writing a repository's location: SSH, HTTPS with or without
/// credentials, Dolt's `git+` prefix, and local paths, which resolve from `root` when relative.
/// A hosted repository is the same with or without `.git` and in any case, as GitHub's are; a
/// local path is compared exactly, because `repo` and `repo.git` can be two repositories.
pub fn repository_identity(url: &str, root: &Path) -> String {
    let trimmed = url.trim();
    let bare = trimmed
        .strip_prefix("git+")
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    if let Some(captures) = SCP_LIKE.captures(bare) {
        return hosted(&captures[1], &captures[2]);
    }
    let Some(captures) = SCHEME.captures(bare) else {
        return resolve(root, bare);
    };
    let (authority, path) = split_authority(&captures[2]);
    if captures[1].eq_ignore_ascii_case("file") {
        return resolve(root, &percent_decoded(path));
    }
    hosted(host(authority), path)
}

/// `url` without the credentials it may carry, for results and errors. HTTPS user names are
/// often tokens, so they go too; an SSH user name such as `git` stays.
pub fn without_credentials(url: &str) -> String {
    USERINFO
        .replace(url, |captures: &Captures| {
            let scheme = captures[2].to_ascii_lowercase();
            if scheme == "http" || scheme == "https" || captures[3].is_empty() {
                captures[1].to_string()
            } else {
                format!("{}{}@", &captures[1], &captures[3])
            }
        })
        .into_owned()
}

fn hosted(host: &str, path: &str) -> String {
    let path = path.trim_start_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    format!("{host}/{path}").to_lowercase()
}

/// The authority and path of a URL's scheme-specific part: `//host:22/path` or `/path`.
fn split_authority(rest: &str) -> (&str, &str) {
    let Some(after) = rest.strip_prefix("//") else {
        return ("", rest);
    };
    match after.find('/') {
        Some(index) => (&after[..index], &after[index..]),
        None => (after, ""),
    }
}

fn host(authority: &str) -> &str {
    let host = authority.rsplit('@').next().unwrap_or(authority);
    match host.strip_prefix('[') {
        Some(_) => host.split_inclusive(']').next().unwrap_or(host),
        None => host.split(':').next().unwrap_or(host),
    }
}

fn percent_decoded(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let escape = (bytes[index] == b'%')
            .then(|| path.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escape {
            Some(byte) => {
                decoded.push(byte);
                index += 3;
            }
            None => {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// `path` resolved against `root` and normalized lexically, as Node's `path.resolve` does.
fn resolve(root: &Path, path: &str) -> String {
    let mut resolved = PathBuf::new();
    for component in root.join(path).components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => resolved.push(other),
        }
    }
    resolved.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosted_spellings_of_one_repository_agree() {
        let root = Path::new("/work/project");
        let canonical = repository_identity("https://github.com/markmals/workbench", root);
        for spelling in [
            "git@github.com:markmals/Workbench.git",
            "ssh://git@github.com:22/markmals/workbench",
            "https://token@github.com/markmals/workbench.git",
            "git+https://github.com/markmals/workbench.git/",
        ] {
            assert_eq!(repository_identity(spelling, root), canonical, "{spelling}");
        }
    }

    #[test]
    fn local_paths_resolve_from_the_checkout_and_keep_their_suffix() {
        let root = Path::new("/work/project");
        assert_eq!(
            repository_identity("../shared/repo.git", root),
            "/work/shared/repo.git"
        );
        assert_eq!(
            repository_identity("git+file:///srv/git/project.git", root),
            repository_identity("/srv/git/project.git", root)
        );
        assert_ne!(
            repository_identity("/srv/git/project", root),
            repository_identity("/srv/git/project.git", root)
        );
    }

    #[test]
    fn credentials_never_show() {
        assert_eq!(
            without_credentials("git+https://user:secret@github.com/owner/repo.git"),
            "git+https://github.com/owner/repo.git"
        );
        assert_eq!(
            without_credentials("https://token@github.com/owner/repo"),
            "https://github.com/owner/repo"
        );
        assert_eq!(
            without_credentials("ssh://git:secret@host/repo"),
            "ssh://git@host/repo"
        );
        assert_eq!(
            without_credentials("git@github.com:owner/repo.git"),
            "git@github.com:owner/repo.git"
        );
    }
}
