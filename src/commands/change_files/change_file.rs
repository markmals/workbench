//! Knope's change-file format: naming, writing, and checking it.
//!
//! The frontmatter rules follow the `changesets` crate Knope reads change files with: a `---`
//! line, one `<package>: <type>` line per package (split at the first `:`, the package
//! optionally quoted), and a closing `---` line. After it comes the summary heading, which Knope
//! turns into the changelog entry, then any detail.

use super::knope::KnopeConfig;

/// Knope's file name for a change with `summary`: ASCII letters and digits lowercased, each run
/// of spaces, `-`, and `_` one `_`, and everything else dropped.
pub fn file_stem(summary: &str) -> String {
    let mut stem = String::new();
    let mut previous_was_underscore = false;
    for character in summary.chars() {
        if character.is_ascii_alphanumeric() {
            previous_was_underscore = false;
            stem.push(character.to_ascii_lowercase());
        } else if matches!(character, ' ' | '_' | '-') && !previous_was_underscore {
            previous_was_underscore = true;
            stem.push('_');
        }
    }
    stem
}

/// A change file for one package: the frontmatter, the summary as its heading, then the body.
pub fn render(package: &str, change_type: &str, summary: &str, body: Option<&str>) -> String {
    let mut content = format!("---\n{package}: {change_type}\n---\n\n# {summary}\n");
    let body = body
        .map(|body| body.trim_start_matches(['\n', '\r']).trim_end())
        .filter(|body| !body.is_empty());
    if let Some(body) = body {
        content.push('\n');
        content.push_str(body);
        content.push('\n');
    }
    content
}

/// What keeps `content` from being a change file Knope accepts and the release notes can use;
/// empty when it is one. With a `config`, every package must be one it declares and every type
/// one that package accepts.
pub fn problems(content: &str, config: Option<&KnopeConfig>) -> Vec<String> {
    let Parsed { entries, body } = match frontmatter(content) {
        Ok(parsed) => parsed,
        Err(problem) => return vec![problem],
    };
    let mut problems: Vec<String> = entries
        .into_iter()
        .filter_map(|(package, change_type)| entry_problem(package, change_type, config))
        .collect();
    if !opens_with_summary(body) {
        problems.push(
            "the first line after the frontmatter must be the summary, a heading such as \
             `# Add the export queue`; Knope writes the summary into the changelog"
                .into(),
        );
    }
    problems
}

/// Whether the first non-blank line of `body` is an ATX heading with text: one to six `#`, then
/// a space or tab, then something other than whitespace.
fn opens_with_summary(body: &str) -> bool {
    let Some(line) = body.lines().map(str::trim).find(|line| !line.is_empty()) else {
        return false;
    };
    let text = line.trim_start_matches('#');
    let level = line.len() - text.len();
    (1..=6).contains(&level) && text.starts_with([' ', '\t']) && !text.trim().is_empty()
}

/// A change file split at its frontmatter.
struct Parsed<'a> {
    /// The `(package, type)` pairs of the frontmatter.
    entries: Vec<(&'a str, &'a str)>,
    /// Everything after the closing `---` line.
    body: &'a str,
}

fn frontmatter(content: &str) -> Result<Parsed<'_>, String> {
    let mut lines = content.split_inclusive('\n');
    let opening = lines.next().unwrap_or_default();
    if opening.trim() != "---" {
        return Err("the file must start with a `---` line opening its frontmatter".into());
    }
    let mut offset = opening.len();
    let mut entries = Vec::new();
    for raw in lines {
        offset += raw.len();
        let line = raw.trim_end_matches(['\n', '\r']);
        if line.trim() == "---" {
            if entries.is_empty() {
                return Err("the frontmatter names no package".into());
            }
            return Ok(Parsed {
                entries,
                body: &content[offset..],
            });
        }
        let (package, change_type) = line.split_once(':').ok_or_else(|| {
            format!("frontmatter line `{line}` is not `<package>: <change type>`")
        })?;
        entries.push((unquote(package.trim()), change_type.trim()));
    }
    Err("the frontmatter has no closing `---` line".into())
}

/// A package name may be wrapped in a matching pair of single or double quotes.
fn unquote(name: &str) -> &str {
    ['\'', '"']
        .into_iter()
        .find_map(|quote| name.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(name)
}

fn entry_problem(package: &str, change_type: &str, config: Option<&KnopeConfig>) -> Option<String> {
    if package.is_empty() || change_type.is_empty() {
        return Some(format!(
            "frontmatter line `{package}: {change_type}` needs both a package and a change type"
        ));
    }
    let config = config?;
    let Some(declared) = config.package(package) else {
        return Some(format!(
            "`{package}` is not a package in knope.toml; its packages are: {}",
            config.package_names().join(", ")
        ));
    };
    if declared.accepts(change_type) {
        return None;
    }
    Some(format!(
        "`{change_type}` is not a change type of `{package}`; its types are: {}",
        declared.change_types.join(", ")
    ))
}
