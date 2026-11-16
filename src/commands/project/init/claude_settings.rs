//! Workbench's declaration in Claude Code's `.claude/settings.json`: the `workbench` marketplace
//! at `./.workbench` and the enabled `workbench@workbench` plugin.

use serde_json::{Map, Value};

use super::json_members::{add_member, new_document};
use crate::commands::project::setup::{self, MARKETPLACE, PLUGIN};

/// One entry Workbench needs: `section.key`, its value, and whether an existing value serves.
struct Wanted {
    section: &'static str,
    key: &'static str,
    value: Value,
    serves: fn(&Value) -> bool,
}

fn wanted() -> [Wanted; 2] {
    [
        Wanted {
            section: "extraKnownMarketplaces",
            key: MARKETPLACE,
            value: setup::package_marketplace(),
            serves: setup::is_package_marketplace,
        },
        Wanted {
            section: "enabledPlugins",
            key: PLUGIN,
            value: Value::Bool(true),
            serves: |value| *value == Value::Bool(true),
        },
    ]
}

/// A settings file holding only Workbench's declaration.
pub fn new_settings() -> String {
    let members: Vec<(&str, Value)> = wanted()
        .into_iter()
        .map(|entry| (entry.section, object(entry.key, entry.value)))
        .collect();
    new_document(&members)
}

/// An existing settings file with Workbench's entries added where absent.
pub struct Edit {
    pub contents: String,
    /// The entries added, as `section.key`.
    pub entries: Vec<String>,
    /// Entries that exist with values that do not load the package, left as they were.
    pub conflicts: Vec<String>,
}

/// Adds Workbench's entries to `source`, or says why it cannot: the file is not a JSON object.
pub fn add_declaration(source: &str) -> Result<Edit, String> {
    let settings: Value =
        serde_json::from_str(source).map_err(|error| format!("it is not valid JSON ({error})"))?;
    if !settings.is_object() {
        return Err("it is not a JSON object".into());
    }
    let mut edit = Edit {
        contents: source.to_string(),
        entries: Vec::new(),
        conflicts: Vec::new(),
    };
    for Wanted {
        section,
        key,
        value,
        serves,
    } in wanted()
    {
        let entry = format!("{section}.{key}");
        let added = match settings.get(section) {
            None => add_member(&edit.contents, &[], section, &object(key, value))?,
            Some(Value::Object(members)) => match members.get(key) {
                None => add_member(&edit.contents, &[section], key, &value)?,
                Some(existing) if serves(existing) => continue,
                Some(existing) => {
                    edit.conflicts.push(format!(
                        "`{entry}` is {existing}, which does not load Workbench; Workbench needs {value}"
                    ));
                    continue;
                }
            },
            Some(_) => {
                edit.conflicts.push(format!(
                    "`{section}` is not an object, so `{key}` was not added"
                ));
                continue;
            }
        };
        edit.contents = added;
        edit.entries.push(entry);
    }
    Ok(edit)
}

fn object(key: &str, value: Value) -> Value {
    Value::Object(Map::from_iter([(key.to_string(), value)]))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_new_file_declares_the_marketplace_and_the_plugin() {
        let settings: Value = serde_json::from_str(&new_settings()).unwrap();
        assert_eq!(
            settings,
            json!({
                "extraKnownMarketplaces": {
                    "workbench": { "source": { "source": "directory", "path": "./.workbench" } }
                },
                "enabledPlugins": { "workbench@workbench": true }
            })
        );
    }

    #[test]
    fn keeps_a_declaration_that_already_serves() {
        let source = new_settings();
        let edit = add_declaration(&source).unwrap();
        assert_eq!(edit.contents, source);
        assert!(edit.entries.is_empty() && edit.conflicts.is_empty());
    }

    #[test]
    fn reports_values_that_do_not_load_workbench() {
        let edit =
            add_declaration("{ \"enabledPlugins\": { \"workbench@workbench\": false } }").unwrap();
        assert_eq!(edit.entries, ["extraKnownMarketplaces.workbench"]);
        assert_eq!(edit.conflicts.len(), 1);
        assert!(add_declaration("[]").is_err());
        assert!(add_declaration("{").is_err());
    }
}
