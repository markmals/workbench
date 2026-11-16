//! Workbench's entries in `mise.toml`, added with `toml_edit` so the rest of the file keeps its
//! formatting and comments.

use toml_edit::{Array, DocumentMut, Item, Table, TableLike, Value, value};

/// The hook that points `.workbench` at the installed package after every `mise install`.
pub const POSTINSTALL: &str =
    r#"ln -sfn "$(mise where github:markmals/workbench)" "{{config_root}}/.workbench""#;

/// The versions this release of Workbench was tested with.
const TOOLS: &[(&str, &str)] = &[
    ("hk", "2.8.1"),
    ("gh", "2.103.2"),
    ("github:gastownhall/beads", "1.3.2"),
];

const SITE_TOOLS: &[(&str, &str)] = &[
    ("hugo", "0.170.0"),
    ("github:CloudCannon/pagefind", "1.6.0"),
];

struct Task {
    name: &'static str,
    description: &'static str,
    run: &'static [&'static str],
}

const TASKS: &[Task] = &[
    Task {
        name: "status",
        description: "Where the work stands: current phase, next action, who acts, and Beads",
        run: &["workbench workflow status --output text"],
    },
    Task {
        name: "validate",
        description: "Check the project's records and its Workbench setup",
        run: &[
            "workbench records validate --output text",
            "workbench project validate --output text",
        ],
    },
    Task {
        name: "beads:sync",
        description: "Sync this machine's Beads replica with the project's remote",
        run: &["workbench beads sync --output text"],
    },
    Task {
        name: "beads:claim",
        description: "Claim a Beads item as this machine and confirm the claim survived a sync: `mise run beads:claim -- <id>`",
        run: &["workbench beads claim --output text"],
    },
    Task {
        name: "beads:close",
        description: "Close a Beads item as this machine's claimant, then sync: `mise run beads:close -- <id> --reason \"<reason>\"`",
        run: &["workbench beads close --output text"],
    },
];

const SITE_TASKS: &[Task] = &[
    Task {
        name: "docs:build",
        description: "Build the site into docs/public, then index it for search (HUGO_BASEURL sets a deployed base URL)",
        run: &[
            "hugo build --source docs --cleanDestinationDir",
            "pagefind --site docs/public",
        ],
    },
    Task {
        name: "docs:dev",
        description: "Serve the site with live reload; search needs `mise run docs:build`",
        run: &["hugo server --source docs"],
    },
];

/// The project-owned tasks Workbench relies on but does not write.
pub const PROJECT_TASKS: &[&str] = &["setup", "check", "fmt"];

/// What adding Workbench's entries to `mise.toml` did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Additions {
    /// The entries added, as dotted keys: `tools.hk`, `tasks."beads:sync"`.
    pub added: Vec<String>,
    /// Entries Workbench would add that exist with other content, each with what to do.
    pub conflicts: Vec<String>,
}

/// Adds the `postinstall` hook, the tool pins, and the tasks the document lacks. Existing entries
/// are never changed.
pub fn add_missing(document: &mut DocumentMut, site: bool) -> Additions {
    let mut additions = Additions::default();
    add_postinstall(document, &mut additions);
    let site_tools: &[(&str, &str)] = if site { SITE_TOOLS } else { &[] };
    add_tools(document, TOOLS.iter().chain(site_tools), &mut additions);
    let site_tasks: &[Task] = if site { SITE_TASKS } else { &[] };
    add_tasks(document, TASKS.iter().chain(site_tasks), &mut additions);
    additions
}

fn add_postinstall(document: &mut DocumentMut, additions: &mut Additions) {
    let Some(hooks) = table(document, "hooks", additions) else {
        return;
    };
    match hooks.get("postinstall") {
        None => {
            hooks.insert("postinstall", value(POSTINSTALL));
            additions.added.push("hooks.postinstall".into());
        }
        Some(existing) if !runs(existing).iter().any(|run| run.contains(".workbench")) => {
            additions.conflicts.push(format!(
                "`hooks.postinstall` does not link `.workbench`; add `{POSTINSTALL}` to it"
            ));
        }
        Some(_) => {}
    }
}

fn add_tools<'a>(
    document: &mut DocumentMut,
    tools: impl Iterator<Item = &'a (&'static str, &'static str)>,
    additions: &mut Additions,
) {
    let Some(table) = table(document, "tools", additions) else {
        return;
    };
    for (tool, version) in tools {
        if !table.contains_key(tool) {
            table.insert(tool, value(*version));
            additions.added.push(dotted("tools", tool));
        }
    }
}

fn add_tasks<'a>(
    document: &mut DocumentMut,
    tasks: impl Iterator<Item = &'a Task>,
    additions: &mut Additions,
) {
    let Some(table) = table(document, "tasks", additions) else {
        return;
    };
    for task in tasks {
        match table.get(task.name) {
            None => {
                table.insert(task.name, Item::Table(task_table(task)));
                additions.added.push(dotted("tasks", task.name));
            }
            Some(existing) if runs(existing) != task.run => {
                additions.conflicts.push(format!(
                    "`{}` runs `{}`; Workbench's runs `{}`",
                    dotted("tasks", task.name),
                    runs(existing).join("` then `"),
                    task.run.join("` then `"),
                ));
            }
            Some(_) => {}
        }
    }
}

/// The project-owned tasks the document does not define.
pub fn missing_project_tasks(document: &DocumentMut) -> Vec<&'static str> {
    let tasks = document.get("tasks").and_then(Item::as_table_like);
    PROJECT_TASKS
        .iter()
        .copied()
        .filter(|task| !tasks.is_some_and(|tasks| tasks.contains_key(task)))
        .collect()
}

/// The top-level table `name`, created when absent. A `tasks` table is implicit, so only its
/// `[tasks.<name>]` headers appear. A key that holds something other than a table is left as
/// it is and reported.
fn table<'a>(
    document: &'a mut DocumentMut,
    name: &str,
    additions: &mut Additions,
) -> Option<&'a mut dyn TableLike> {
    let item = document.entry(name).or_insert_with(|| {
        let mut table = Table::new();
        table.set_implicit(name == "tasks");
        Item::Table(table)
    });
    let table = item.as_table_like_mut();
    if table.is_none() {
        additions.conflicts.push(format!(
            "`{name}` is not a table, so nothing was added to it"
        ));
    }
    table
}

fn task_table(task: &Task) -> Table {
    let mut table = Table::new();
    table.insert("description", value(task.description));
    let run = match task.run {
        [single] => value(*single),
        many => Item::Value(Value::Array(many.iter().copied().collect::<Array>())),
    };
    table.insert("run", run);
    table
}

/// The commands an existing task or hook runs: its string, its list of strings, or a task
/// table's `run`.
fn runs(item: &Item) -> Vec<String> {
    let run = item
        .as_table_like()
        .and_then(|table| table.get("run"))
        .unwrap_or(item);
    match run {
        Item::Value(Value::String(command)) => vec![command.value().clone()],
        Item::Value(Value::Array(commands)) => commands
            .iter()
            .filter_map(|command| command.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

/// `table.key`, quoting the key as TOML requires: `tasks."beads:sync"`.
pub fn dotted(table: &str, key: &str) -> String {
    let bare = key
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character));
    if bare {
        format!("{table}.{key}")
    } else {
        format!("{table}.\"{key}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn added(source: &str, site: bool) -> (String, Additions) {
        let mut document: DocumentMut = source.parse().unwrap();
        let additions = add_missing(&mut document, site);
        (document.to_string(), additions)
    }

    #[test]
    fn adds_every_entry_to_a_pin_only_file_and_keeps_its_text() {
        let source = "# Tools.\n[tools]\n\"github:markmals/workbench\" = \"3.0.0\" # pinned\n";
        let (text, additions) = added(source, false);
        assert!(text.starts_with(source.trim_end()), "{text}");
        let parsed: toml::Table = text.parse().unwrap();
        assert_eq!(parsed["hooks"]["postinstall"].as_str(), Some(POSTINSTALL));
        assert_eq!(
            parsed["tools"]["github:gastownhall/beads"].as_str(),
            Some("1.3.2")
        );
        assert_eq!(
            parsed["tasks"]["validate"]["run"].as_array().unwrap().len(),
            2
        );
        assert!(!text.contains("[tasks]\n"), "{text}");
        assert!(text.contains("[tasks.\"beads:sync\"]"), "{text}");
        assert!(additions.conflicts.is_empty());
        assert_eq!(additions.added.len(), 9);
    }

    #[test]
    fn leaves_existing_entries_and_reports_the_ones_that_differ() {
        let source = "[tools]\nhk = \"2.0.0\"\n\n[hooks]\npostinstall = \"echo hi\"\n\n[tasks.status]\nrun = \"node tools/status.ts\"\n\n[tasks.\"beads:sync\"]\nrun = [\"workbench beads sync --output text\"]\n";
        let (text, additions) = added(source, false);
        let mut remaining = text.lines();
        let kept = source
            .lines()
            .all(|line| remaining.by_ref().any(|candidate| candidate == line));
        assert!(kept, "{text}");
        assert!(!additions.added.contains(&"tools.hk".to_string()));
        assert!(!additions.added.contains(&"hooks.postinstall".to_string()));
        assert_eq!(additions.conflicts.len(), 2, "{:?}", additions.conflicts);
        assert!(additions.conflicts[0].contains("hooks.postinstall"));
        assert!(additions.conflicts[1].contains("node tools/status.ts"));
    }

    #[test]
    fn a_site_adds_hugo_pagefind_and_the_docs_tasks() {
        let (text, additions) = added("[tools]\n", true);
        let parsed: toml::Table = text.parse().unwrap();
        assert_eq!(parsed["tools"]["hugo"].as_str(), Some("0.170.0"));
        assert!(
            additions
                .added
                .contains(&"tasks.\"docs:build\"".to_string())
        );
    }

    #[test]
    fn names_the_project_tasks_it_does_not_write() {
        let document: DocumentMut = "[tasks.setup]\nrun = \"x\"\n".parse().unwrap();
        assert_eq!(missing_project_tasks(&document), ["check", "fmt"]);
        let document: DocumentMut = "".parse().unwrap();
        assert_eq!(missing_project_tasks(&document), PROJECT_TASKS);
    }
}
