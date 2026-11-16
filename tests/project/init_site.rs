//! `workbench project init --site`: the documentation site on GitHub Pages or Cloudflare.

use crate::fixtures::{init, paths, pinned_repository, toml};
use crate::init_files::task_run;

#[test]
fn init_with_a_github_pages_site_adds_pins_tasks_and_workflows() {
    let fixture = pinned_repository();
    let receipt = init(&fixture, &["--site", "github-pages"]);
    assert_eq!(receipt["site"], "github-pages");
    let mise = toml(&fixture, "mise.toml");
    assert_eq!(mise["tools"]["hugo"].as_str(), Some("0.170.0"));
    assert_eq!(
        mise["tools"]["github:CloudCannon/pagefind"].as_str(),
        Some("1.6.0")
    );
    assert_eq!(
        task_run(&mise, "docs:build"),
        toml::Value::Array(vec![
            "hugo build --source docs --cleanDestinationDir".into(),
            "pagefind --site docs/public".into(),
        ])
    );
    assert_eq!(
        task_run(&mise, "docs:dev").as_str(),
        Some("hugo server --source docs")
    );
    let publish = fixture.read(".github/workflows/docs.yml");
    for expected in [
        "mise run docs:build",
        "HUGO_BASEURL",
        "gh-pages",
        "docs/public",
    ] {
        assert!(publish.contains(expected), "docs.yml lacks {expected}");
    }
    let preview = fixture.read(".github/workflows/preview.yml");
    for expected in ["pr-preview", "closed", "HUGO_BASEURL"] {
        assert!(preview.contains(expected), "preview.yml lacks {expected}");
    }
    let created = paths(&receipt["created"]);
    for path in [
        ".github/workflows/docs.yml",
        ".github/workflows/preview.yml",
        "docs/.gitignore",
    ] {
        assert!(created.contains(&path.to_string()), "{path} not created");
    }
    assert!(!created.contains(&"docs/wrangler.jsonc".to_string()));
    assert_eq!(
        fixture.read("docs/.gitignore"),
        "/public/\n/resources/\n/.hugo_build.lock\n"
    );
    assert_eq!(fixture.read(".gitignore"), ".workbench\n");
}

#[test]
fn init_with_a_cloudflare_site_writes_cloudflare_workflows_and_the_worker() {
    let fixture = pinned_repository();
    let receipt = init(&fixture, &["--site", "cloudflare"]);
    assert_eq!(receipt["site"], "cloudflare");
    for workflow in [
        ".github/workflows/docs.yml",
        ".github/workflows/preview.yml",
    ] {
        let text = fixture.read(workflow);
        for expected in [
            "cloudflare/wrangler-action",
            "CLOUDFLARE_API_TOKEN",
            "CLOUDFLARE_ACCOUNT_ID",
            "mise run docs:build",
            "workingDirectory: docs",
        ] {
            assert!(text.contains(expected), "{workflow} lacks {expected}");
        }
    }
    assert!(
        fixture
            .read(".github/workflows/preview.yml")
            .contains("preview delete")
    );
    let wrangler = fixture.read("docs/wrangler.jsonc");
    let directory = fixture
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_lowercase();
    assert!(
        wrangler.contains(&format!("\"name\": \"{directory}-docs\"")),
        "{wrangler}"
    );
    assert!(wrangler.contains("\"directory\": \"./public\""));
    assert!(!wrangler.contains("{{"), "{wrangler}");
}
