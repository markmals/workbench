//! `workbench context show`.

mod support;

use std::os::unix::fs::symlink;
use std::process::Command;

use serde_json::{Value, json};

use support::{Fixture, assert_matches_output_schema, declare_workbench, package_version};

const ORIGIN: &str = "https://github.com/example/project.git";

const PULL_REQUEST: &str = r#"[{"number":12,"url":"https://github.com/example/project/pull/12","title":"Add things","state":"OPEN","isDraft":true}]"#;

/// `gh` that answers `pr list` for `main` with `list`, and fails anything else.
fn fake_gh(fixture: &Fixture, list: &str) {
    fixture.fake_program(
        "gh",
        &format!(
            r#"if [ "$1 $2" = "pr list" ]; then
  case "$*" in *"--head main"*) ;; *) echo "unexpected: $*" >&2; exit 9 ;; esac
  echo '{list}'
  exit 0
fi
exit 1"#
        ),
    );
}

/// `bd` that reports `remotes` for `dolt remote list --json`, and only with the Dolt info
/// branch disabled.
fn fake_bd(fixture: &Fixture, remotes: &str) {
    fixture.fake_program(
        "bd",
        &format!(
            r#"[ "${{DOLT_REMOTE_INFO_BRANCH-unset}}" = "" ] || {{ echo "info branch enabled" >&2; exit 9; }}
if [ "$*" = "dolt remote list --json" ]; then echo '{remotes}'; exit 0; fi
exit 1"#
        ),
    );
}

fn show(fixture: &Fixture, args: &[&str]) -> Value {
    let mut command = vec!["context", "show"];
    command.extend_from_slice(args);
    let outcome = fixture.run(&command);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    let context = outcome.json();
    assert_matches_output_schema(&["context", "show"], &context);
    context
}

/// An adopted project with an `origin`, a Beads workspace, a site, and fake `gh` and `bd`.
fn adopted_project() -> Fixture {
    let mut fixture = Fixture::git();
    declare_workbench(&fixture).link_package();
    fixture.git_command(&["remote", "add", "origin", ORIGIN]);
    fixture
        .write(".beads/config.yaml", "issue-prefix: project\n")
        .write("docs/hugo.toml", "theme = \"site\"\n")
        .write(
            ".github/workflows/docs.yml",
            "jobs:\n  publish:\n    steps:\n      - run: mise run docs:build\n      - uses: JamesIves/github-pages-deploy-action@v4.9.0\n        with:\n          branch: gh-pages\n",
        )
        .write(".github/workflows/ci.yml", "jobs:\n  check:\n    steps:\n      - run: mise run check\n")
        .write(".agents/skills/our-skill/SKILL.md", "---\nname: our-skill\n---\n");
    symlink("../.agents/skills", fixture.path(".claude/skills")).unwrap();
    fake_gh(&fixture, PULL_REQUEST);
    fake_bd(
        &fixture,
        r#"[{"name":"origin","url":"git+https://github.com/example/project.git"}]"#,
    );
    fixture.set_env("BEADS_ACTOR", "agent@ci");
    fixture
}

#[test]
fn context_show_reports_the_project_setup() {
    let fixture = adopted_project();
    let context = show(&fixture, &[]);
    let root = fixture.root().canonicalize().unwrap();

    assert_eq!(context["project"], root.display().to_string());
    assert_eq!(context["branch"], "main");
    assert_eq!(context["origin"], ORIGIN);
    assert_eq!(
        context["pullRequest"],
        json!({
            "status": "found",
            "number": 12,
            "url": "https://github.com/example/project/pull/12",
            "title": "Add things",
            "state": "OPEN",
            "draft": true,
        })
    );
    assert_eq!(
        context["workbench"]["pin"],
        json!({ "status": "pinned", "version": package_version() })
    );
    assert_eq!(context["workbench"]["binary"], env!("CARGO_PKG_VERSION"));
    let link = &context["workbench"]["link"];
    assert_eq!(link["path"], ".workbench");
    assert_eq!(link["state"], "linked");
    assert_eq!(
        link["target"],
        support::plugin_dir()
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );
    assert_eq!(link["version"], package_version());
    assert_eq!(link["workingTree"], false);
    assert_eq!(
        context["harnesses"]["claudeCode"],
        json!({ "path": ".claude/settings.json", "state": "declared", "detail": null })
    );
    assert_eq!(
        context["harnesses"]["ohMyPi"],
        json!({ "path": ".omp/config.yml", "state": "declared", "detail": null })
    );
    assert_eq!(
        context["harnesses"]["projectLinks"],
        json!([
            { "path": ".claude/skills", "target": "../.agents/skills" },
            { "path": ".claude/rules", "target": null },
        ])
    );
    assert_eq!(
        context["beads"],
        json!({
            "workspace": ".beads",
            "remote": { "status": "configured", "url": "git+https://github.com/example/project.git" },
            "claimant": "agent@ci",
        })
    );
    assert_eq!(
        context["site"],
        json!({
            "config": "docs/hugo.toml",
            "workflows": [{ "path": ".github/workflows/docs.yml", "host": "github-pages" }],
        })
    );
}

#[test]
fn context_show_never_prints_credentials_from_origin_or_the_beads_remote() {
    let fixture = adopted_project();
    fixture.git_command(&[
        "remote",
        "set-url",
        "origin",
        "https://agent:s3cret-token@github.com/example/project.git",
    ]);
    fake_bd(
        &fixture,
        r#"[{"name":"origin","url":"git+https://x-access-token:s3cret-token@github.com/example/project.git"}]"#,
    );

    for args in [&[][..], &["--output", "text"][..]] {
        let mut command = vec!["context", "show"];
        command.extend_from_slice(args);
        let outcome = fixture.run(&command);
        assert_eq!(outcome.status, 0, "{}", outcome.stderr);
        assert!(!outcome.stdout.contains("s3cret"), "{}", outcome.stdout);
        assert!(!outcome.stderr.contains("s3cret"), "{}", outcome.stderr);
    }
    let context = show(&fixture, &[]);
    assert_eq!(context["origin"], "https://github.com/example/project.git");
}

#[test]
fn context_show_reports_no_pull_request_when_github_has_none() {
    let fixture = adopted_project();
    fake_gh(&fixture, "[]");
    assert_eq!(
        show(&fixture, &[])["pullRequest"],
        json!({ "status": "none" })
    );
}

fn unknown_reason(fixture: &Fixture, args: &[&str]) -> String {
    let pull_request = show(fixture, args)["pullRequest"].clone();
    assert_eq!(pull_request["status"], "unknown", "{pull_request}");
    pull_request["reason"].as_str().unwrap().to_string()
}

#[test]
fn context_show_says_why_the_pull_request_is_unknown() {
    let fixture = adopted_project();

    fixture.fake_program(
        "gh",
        "echo 'To get started with GitHub CLI, please run:  gh auth login' >&2\nexit 4",
    );
    assert!(unknown_reason(&fixture, &[]).contains("not authenticated"));

    fixture.fake_program("gh", "sleep 5");
    let reason = unknown_reason(&fixture, &["--timeout", "1"]);
    assert!(reason.contains("1 second"), "{reason}");

    let two = PULL_REQUEST.replace(
        "}]",
        "},{\"number\":13,\"url\":\"u\",\"title\":\"t\",\"state\":\"OPEN\",\"isDraft\":false}]",
    );
    fake_gh(&fixture, &two);
    assert!(unknown_reason(&fixture, &[]).contains("2 pull requests"));

    fixture.git_command(&["remote", "remove", "origin"]);
    fixture.fake_program("gh", "touch gh-ran\nexit 1");
    assert!(unknown_reason(&fixture, &[]).contains("origin"));
    assert!(!fixture.path("gh-ran").exists(), "gh ran without an origin");
}

#[test]
fn context_show_reports_a_detached_head() {
    let fixture = adopted_project();
    fixture.git_command(&["checkout", "--quiet", "--detach"]);
    let context = show(&fixture, &[]);
    assert_eq!(context["branch"], Value::Null);
    assert!(
        context["pullRequest"]["reason"]
            .as_str()
            .unwrap()
            .contains("detached")
    );
}

#[test]
fn context_show_reports_unknown_when_gh_and_bd_are_missing() {
    let mut fixture = Fixture::git();
    fixture.git_command(&["remote", "add", "origin", ORIGIN]);
    fixture.write(".beads/config.yaml", "issue-prefix: project\n");
    let git = Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .unwrap();
    let git = String::from_utf8(git.stdout).unwrap();
    symlink(git.trim(), fixture.path(".fixture-bin/git")).unwrap();
    let path = format!("{}:/usr/bin:/bin", fixture.path(".fixture-bin").display());
    fixture.set_env("PATH", &path);

    let context = show(&fixture, &[]);
    assert!(
        context["pullRequest"]["reason"]
            .as_str()
            .unwrap()
            .contains("gh")
    );
    assert_eq!(context["beads"]["remote"]["status"], "unknown");
    assert!(
        context["beads"]["remote"]["reason"]
            .as_str()
            .unwrap()
            .contains("bd")
    );
}

#[test]
fn context_show_reports_beads_without_a_workspace_or_a_remote() {
    let fixture = adopted_project();
    fake_bd(&fixture, "[]");
    assert_eq!(
        show(&fixture, &[])["beads"]["remote"],
        json!({ "status": "none" })
    );

    std::fs::remove_dir_all(fixture.path(".beads")).unwrap();
    let beads = show(&fixture, &[])["beads"].clone();
    assert_eq!(beads["workspace"], Value::Null);
    assert_eq!(beads["remote"]["status"], "unknown");
}

#[test]
fn context_show_names_the_login_claimant_without_beads_actor() {
    let mut fixture = adopted_project();
    fixture.set_env("BEADS_ACTOR", "");
    let output = |program: &str, arg: &str| {
        let output = Command::new(program).arg(arg).output().unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    let user = output("id", "-un");
    let host = output("uname", "-n");
    let short_host = host.split('.').next().unwrap();
    assert_eq!(
        show(&fixture, &[])["beads"]["claimant"],
        format!("{user}@{short_host}")
    );
}

#[test]
fn context_show_marks_workbench_s_own_repository() {
    let fixture = Fixture::git();
    fixture
        .write("Cargo.toml", "[package]\nname = \"workbench\"\n")
        .write("mise.toml", "[tools]\nrust = \"1.100.0\"\n")
        .write(
            "plugin/.claude-plugin/plugin.json",
            "{ \"version\": \"9.9.9\" }\n",
        );
    symlink("plugin", fixture.path(".workbench")).unwrap();
    let workbench = show(&fixture, &[])["workbench"].clone();
    assert_eq!(workbench["pin"], json!({ "status": "unpinned" }));
    assert_eq!(workbench["link"]["workingTree"], true);
    assert_eq!(workbench["link"]["version"], "9.9.9");
}

#[test]
fn context_show_reports_what_a_bare_repository_lacks() {
    let fixture = Fixture::git();
    symlink("/nonexistent/workbench", fixture.path(".workbench")).unwrap();
    let context = show(&fixture, &[]);
    assert_eq!(context["origin"], Value::Null);
    assert_eq!(context["workbench"]["pin"], json!({ "status": "unpinned" }));
    assert_eq!(
        context["workbench"]["link"],
        json!({
            "path": ".workbench",
            "state": "dangling",
            "target": "/nonexistent/workbench",
            "version": null,
            "workingTree": false,
        })
    );
    assert_eq!(context["harnesses"]["claudeCode"]["state"], "missing");
    assert_eq!(context["harnesses"]["ohMyPi"]["state"], "missing");
    assert_eq!(context["beads"]["workspace"], Value::Null);
    assert_eq!(context["site"], json!({ "config": null, "workflows": [] }));

    std::fs::remove_file(fixture.path(".workbench")).unwrap();
    fixture.write("mise.toml", "[tools\n");
    let workbench = show(&fixture, &[])["workbench"].clone();
    assert_eq!(workbench["link"]["state"], "missing");
    assert_eq!(workbench["pin"]["status"], "unreadable");
}

#[test]
fn context_show_names_the_host_of_each_site_workflow() {
    let fixture = adopted_project();
    fixture
        .write(
            ".github/workflows/docs-preview.yml",
            "steps:\n  - run: mise run docs:build\n  - uses: cloudflare/wrangler-action@v4\n",
        )
        .write(
            ".github/workflows/site.yaml",
            "# Could publish with cloudflare/wrangler-action to gh-pages instead.\nsteps:\n  - run: mise run docs:build\n",
        );
    assert_eq!(
        show(&fixture, &[])["site"]["workflows"],
        json!([
            { "path": ".github/workflows/docs-preview.yml", "host": "cloudflare" },
            { "path": ".github/workflows/docs.yml", "host": "github-pages" },
            { "path": ".github/workflows/site.yaml", "host": "unknown" },
        ])
    );
}

#[test]
fn context_show_renders_text_for_humans() {
    let fixture = adopted_project();
    let outcome = fixture.run(&["context", "show", "--output", "text"]);
    assert_eq!(outcome.status, 0, "{}", outcome.stderr);
    for expected in [
        "main",
        ORIGIN,
        "#12",
        "agent@ci",
        "docs/hugo.toml",
        ".workbench",
    ] {
        assert!(
            outcome.stdout.contains(expected),
            "lacks {expected}:\n{}",
            outcome.stdout
        );
    }
}

#[test]
fn context_show_outside_git_fails_with_a_precondition() {
    let outcome = Fixture::empty().run(&["context", "show"]);
    assert_eq!(outcome.status, 3);
    assert_eq!(outcome.error_code(), "PRECONDITION_FAILED");
}
