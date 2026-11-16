//! The packages and change types a project's `knope.toml` declares.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;
use serde_json::json;

use crate::failure::Failure;

pub const KNOPE_TOML: &str = "knope.toml";

/// Every package accepts these; `extra_changelog_sections` adds more.
const BUILT_IN_TYPES: [&str; 3] = ["major", "minor", "patch"];

/// Knope's name for the package of a single-package project, declared as `[package]`.
const SINGLE_PACKAGE: &str = "default";

const PACKAGES_REFERENCE: &str = "https://knope.tech/reference/config-file/packages/";

#[derive(Debug, PartialEq, Eq)]
pub struct KnopeConfig {
    pub packages: Vec<KnopePackage>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct KnopePackage {
    /// The key a change file's frontmatter uses for the package.
    pub name: String,
    pub change_types: Vec<String>,
}

#[derive(Deserialize)]
struct Document {
    package: Option<PackageTable>,
    #[serde(default)]
    packages: BTreeMap<String, PackageTable>,
}

#[derive(Deserialize)]
struct PackageTable {
    #[serde(default)]
    extra_changelog_sections: Vec<Section>,
}

#[derive(Deserialize)]
struct Section {
    #[serde(default)]
    types: Vec<String>,
}

impl KnopeConfig {
    /// The project's Knope configuration, or `None` when it has no `knope.toml`.
    pub fn load(root: &Path) -> Result<Option<KnopeConfig>, Failure> {
        let path = root.join(KNOPE_TOML);
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(unusable(format!("cannot read knope.toml: {error}"))),
        };
        KnopeConfig::parse(&source).map(Some).map_err(unusable)
    }

    pub fn parse(source: &str) -> Result<KnopeConfig, String> {
        let document: Document =
            toml::from_str(source).map_err(|error| format!("knope.toml is not valid: {error}"))?;
        let packages = match (document.package, document.packages) {
            (Some(_), packages) if !packages.is_empty() => {
                return Err(
                    "knope.toml declares both [package] and [packages.<name>]; Knope accepts one"
                        .into(),
                );
            }
            (Some(single), _) => vec![KnopePackage::new(SINGLE_PACKAGE, single)],
            (None, packages) if !packages.is_empty() => packages
                .into_iter()
                .map(|(name, table)| KnopePackage::new(&name, table))
                .collect(),
            (None, _) => {
                return Err("knope.toml declares no [package] or [packages.<name>]".into());
            }
        };
        Ok(KnopeConfig { packages })
    }

    pub fn package(&self, name: &str) -> Option<&KnopePackage> {
        self.packages.iter().find(|package| package.name == name)
    }

    pub fn package_names(&self) -> Vec<&str> {
        self.packages
            .iter()
            .map(|package| package.name.as_str())
            .collect()
    }
}

impl KnopePackage {
    fn new(name: &str, table: PackageTable) -> KnopePackage {
        let mut change_types: Vec<String> =
            BUILT_IN_TYPES.iter().map(ToString::to_string).collect();
        for change_type in table
            .extra_changelog_sections
            .into_iter()
            .flat_map(|section| section.types)
        {
            if !change_types.contains(&change_type) {
                change_types.push(change_type);
            }
        }
        KnopePackage {
            name: name.to_string(),
            change_types,
        }
    }

    pub fn accepts(&self, change_type: &str) -> bool {
        self.change_types
            .iter()
            .any(|accepted| accepted == change_type)
    }
}

fn unusable(problem: String) -> Failure {
    Failure::precondition(problem)
        .with_details(json!({ "path": KNOPE_TOML }))
        .suggest(format!(
            "Fix knope.toml so it declares [package] or [packages.<name>]; see {PACKAGES_REFERENCE}"
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_package_is_named_default_and_gains_its_extra_types() {
        let config = KnopeConfig::parse(
            "[package]\nextra_changelog_sections = [{ name = \"Notes\", footers = [\"Changelog-Note\"] }, { name = \"Security\", types = [\"security\", \"patch\"] }]\n",
        )
        .unwrap();
        assert_eq!(
            config.packages,
            [KnopePackage {
                name: "default".into(),
                change_types: vec![
                    "major".into(),
                    "minor".into(),
                    "patch".into(),
                    "security".into()
                ],
            }]
        );
    }

    #[test]
    fn named_packages_keep_their_own_types() {
        let config = KnopeConfig::parse(
            "[packages.web]\nextra_changelog_sections = [{ name = \"Poems\", types = [\"poem\"] }]\n[packages.api]\n",
        )
        .unwrap();
        assert_eq!(config.package_names(), ["api", "web"]);
        assert!(!config.package("api").unwrap().accepts("poem"));
        assert!(config.package("web").unwrap().accepts("poem"));
    }

    #[test]
    fn a_config_needs_exactly_one_way_of_declaring_packages() {
        assert!(KnopeConfig::parse("[changes]\nignore_conventional_commits = true\n").is_err());
        assert!(KnopeConfig::parse("[package]\n[packages.api]\n").is_err());
        assert!(KnopeConfig::parse("[package\n").is_err());
    }
}
