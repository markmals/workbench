//! The `extensions:` list in Oh My Pi's `.omp/config.yml`, read and extended as text so the rest
//! of the file keeps its formatting. Handles the forms a project writes for that key: a block
//! list (`- ./.workbench` items) or a single-line flow list (`[./.workbench]`).

const KEY: &str = "extensions:";
const INDENT: &str = "    ";

/// The entries of the top-level `extensions:` list, or `None` when the file has no such key.
pub fn extensions(source: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = source.lines().collect();
    let key_line = key_line(&lines)?;
    let value = without_comment(&lines[key_line][KEY.len()..]);
    if let Some(flow) = flow_list(value) {
        return Some(
            flow.split(',')
                .map(unquote)
                .filter(|entry| !entry.is_empty())
                .collect(),
        );
    }
    Some(
        block_items(&lines, key_line)
            .into_iter()
            .map(|index| unquote(item_value(lines[index])))
            .collect(),
    )
}

/// `source` with `entry` appended to its top-level `extensions:` list, adding the key when the
/// file has none. Every other line is kept as it was.
pub fn with_extension(source: &str, entry: &str) -> String {
    let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
    let borrowed: Vec<&str> = source.lines().collect();
    match key_line(&borrowed) {
        None => {
            if lines.last().is_some_and(|line| !line.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(KEY.to_string());
            lines.push(format!("{INDENT}- {entry}"));
        }
        Some(index) => {
            let value = without_comment(&borrowed[index][KEY.len()..]);
            if let Some(flow) = flow_list(value) {
                let entries = if flow.trim().is_empty() {
                    entry.to_string()
                } else {
                    format!("{}, {entry}", flow.trim())
                };
                lines[index] = format!("{KEY} [{entries}]");
            } else {
                let items = block_items(&borrowed, index);
                let (after, indent) = match items.last() {
                    Some(&last) => (last, leading_whitespace(borrowed[last]).to_string()),
                    None => (index, INDENT.to_string()),
                };
                lines.insert(after + 1, format!("{indent}- {entry}"));
            }
        }
    }
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn key_line(lines: &[&str]) -> Option<usize> {
    lines.iter().position(|line| line.starts_with(KEY))
}

/// The inside of a flow list written on the key's line: `[a, b]` gives `a, b`.
fn flow_list(value: &str) -> Option<&str> {
    value.trim().strip_prefix('[')?.strip_suffix(']')
}

/// The indices of the block list's item lines after the key, up to the next top-level key.
fn block_items(lines: &[&str], key_line: usize) -> Vec<usize> {
    let mut items = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(key_line + 1) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "-" || trimmed.starts_with("- ") {
            items.push(index);
        } else if !line.starts_with([' ', '\t']) {
            break;
        }
    }
    items
}

fn item_value(line: &str) -> &str {
    let item = line.trim_start().trim_start_matches('-');
    without_comment(item)
}

fn without_comment(value: &str) -> &str {
    let value = value.trim();
    if value.starts_with(['"', '\'']) {
        return value;
    }
    match value.find(" #") {
        Some(index) => value[..index].trim_end(),
        None if value.starts_with('#') => "",
        None => value,
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner.to_string();
        }
    }
    value.to_string()
}

fn leading_whitespace(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_block_and_flow_lists() {
        let block = "theme: dark\nextensions:\n    - ./other # a comment\n    - \"./.workbench\"\nmodel: x\n";
        assert_eq!(
            extensions(block),
            Some(vec!["./other".to_string(), "./.workbench".to_string()])
        );
        assert_eq!(
            extensions("extensions: [./a, './.workbench']\n"),
            Some(vec!["./a".to_string(), "./.workbench".to_string()])
        );
        assert_eq!(extensions("extensions: []\n"), Some(vec![]));
        assert_eq!(extensions("theme: dark\n"), None);
    }

    #[test]
    fn a_list_ends_at_the_next_top_level_key() {
        let source = "extensions:\n  - ./a\nother:\n  - ./b\n";
        assert_eq!(extensions(source), Some(vec!["./a".to_string()]));
    }

    #[test]
    fn appends_after_the_last_item_with_its_indentation() {
        let source = "extensions:\n  - ./other\n\nmodel: x\n";
        assert_eq!(
            with_extension(source, "./.workbench"),
            "extensions:\n  - ./other\n  - ./.workbench\n\nmodel: x\n"
        );
    }

    #[test]
    fn appends_to_a_flow_list_and_fills_an_empty_one() {
        assert_eq!(
            with_extension("extensions: [./a]\n", "./.workbench"),
            "extensions: [./a, ./.workbench]\n"
        );
        assert_eq!(
            with_extension("extensions: []\n", "./.workbench"),
            "extensions: [./.workbench]\n"
        );
        assert_eq!(
            with_extension("extensions:\nmodel: x\n", "./.workbench"),
            "extensions:\n    - ./.workbench\nmodel: x\n"
        );
    }

    #[test]
    fn adds_the_key_when_the_file_has_none() {
        assert_eq!(
            with_extension("model: x\n", "./.workbench"),
            "model: x\n\nextensions:\n    - ./.workbench\n"
        );
        assert_eq!(
            with_extension("", "./.workbench"),
            "extensions:\n    - ./.workbench\n"
        );
    }
}
