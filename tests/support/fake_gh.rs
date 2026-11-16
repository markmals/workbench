//! A scripted `gh` for tests: an authenticated GitHub CLI that knows one repository.

use serde_json::Value;

use super::Fixture;

/// Installs a `gh` that is authenticated, reports `default_branch` as the repository's default
/// branch, and lists `pull_requests` for `pr list --state all --head <head>` (none otherwise).
pub fn install_fake_gh(fixture: &Fixture, head: &str, default_branch: &str, pull_requests: &Value) {
    fixture.write(
        ".fixture-bin/gh.pull-requests.json",
        &pull_requests.to_string(),
    );
    fixture.fake_program(
        "gh",
        &format!(
            r#"case "$1" in
  --version) echo "gh version 2.103.2"; exit 0 ;;
  auth) exit 0 ;;
  repo) echo "{default_branch}"; exit 0 ;;
  pr)
    [ "$2" = list ] || exit 1
    all=no; head=
    while [ $# -gt 0 ]; do
      case "$1" in
        --state) [ "$2" = all ] && all=yes ;;
        --head) head="$2" ;;
      esac
      shift
    done
    if [ "$all" = yes ] && [ "$head" = "{head}" ]; then
      cat "$(dirname "$0")/gh.pull-requests.json"
    else
      echo "[]"
    fi
    exit 0 ;;
  *) exit 1 ;;
esac"#
        ),
    );
}

/// Installs a `gh` that is installed but not logged in.
pub fn install_unauthenticated_gh(fixture: &Fixture) {
    fixture.fake_program(
        "gh",
        r#"case "$1" in
  --version) echo "gh version 2.103.2"; exit 0 ;;
  *) echo "You are not logged into any GitHub hosts. To log in, run: gh auth login" >&2; exit 1 ;;
esac"#,
    );
}

/// Installs a `gh` that fails every command, as an unusable installation does.
pub fn install_broken_gh(fixture: &Fixture) {
    fixture.fake_program("gh", "exit 1");
}
