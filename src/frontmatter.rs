//! Record frontmatter: the restricted YAML subset records use, and code-free prose.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontmatterValue {
    Scalar(String),
    List(Vec<String>),
}

impl FrontmatterValue {
    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            FrontmatterValue::Scalar(value) => Some(value),
            FrontmatterValue::List(_) => None,
        }
    }

    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            FrontmatterValue::List(values) => Some(values),
            FrontmatterValue::Scalar(_) => None,
        }
    }
}

pub type Frontmatter = BTreeMap<String, FrontmatterValue>;

static KEY_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z][a-z-]*):[ \t]*(.*)$").expect("valid regex"));

/// Parses the frontmatter at the top of `source`: scalar values and inline lists only.
/// The error says what is wrong in words fit for a violation message.
pub fn parse(source: &str) -> Result<Frontmatter, String> {
    let lines: Vec<&str> = source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    if lines.first() != Some(&"---") {
        return Err("missing YAML frontmatter".to_string());
    }
    let Some(end) = lines
        .iter()
        .skip(1)
        .position(|line| *line == "---")
        .map(|index| index + 1)
    else {
        return Err("unterminated YAML frontmatter".to_string());
    };

    let mut frontmatter = Frontmatter::new();
    for line in &lines[1..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(captures) = KEY_LINE.captures(line) else {
            return Err(format!("cannot parse frontmatter line \"{line}\""));
        };
        let key = captures[1].to_string();
        let raw = &captures[2];
        if frontmatter.contains_key(&key) {
            return Err(format!("duplicate frontmatter key \"{key}\""));
        }
        let value = if raw.starts_with('[') {
            FrontmatterValue::List(parse_list(&key, raw)?)
        } else {
            FrontmatterValue::Scalar(raw.to_string())
        };
        frontmatter.insert(key, value);
    }
    Ok(frontmatter)
}

fn parse_list(key: &str, raw: &str) -> Result<Vec<String>, String> {
    let malformed = || format!("cannot parse list value for \"{key}\"");
    let inner = raw
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(malformed)?;
    let contents = inner.trim();
    if contents.contains('[') || contents.contains(']') {
        return Err(malformed());
    }
    if contents.is_empty() {
        return Ok(Vec::new());
    }
    let values: Vec<String> = contents
        .split(',')
        .map(|value| value.trim().to_string())
        .collect();
    if values.iter().any(String::is_empty) {
        return Err(malformed());
    }
    Ok(values)
}

static CLOSING_FENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(`+|~+)[ \t]*$").expect("valid regex"));
static OPENING_FENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(`{3,}|~{3,})(.*)$").expect("valid regex"));
static INLINE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`[^`\n]*`").expect("valid regex"));

/// `markdown` with fenced blocks and inline code spans removed, so only prose remains.
///
/// A proposal that documents the clarification convention necessarily writes the marker down;
/// only prose counts as an unresolved question.
pub fn without_code(markdown: &str) -> String {
    let mut prose = String::with_capacity(markdown.len());
    let mut fence: Option<(char, usize)> = None;

    for segment in markdown.split_inclusive('\n') {
        let (line, ending) = split_ending(segment);
        if let Some((character, length)) = fence {
            let closes = CLOSING_FENCE.captures(line).is_some_and(|captures| {
                let marker = &captures[1];
                marker.starts_with(character) && marker.len() >= length
            });
            if closes {
                fence = None;
                prose.push_str(ending);
            }
            continue;
        }
        if let Some(captures) = OPENING_FENCE.captures(line)
            && !captures[2].contains('`')
        {
            let marker = &captures[1];
            fence = Some((
                marker.chars().next().expect("a fence has a character"),
                marker.len(),
            ));
            continue;
        }
        prose.push_str(&INLINE_CODE.replace_all(line, ""));
        prose.push_str(ending);
    }
    prose
}

fn split_ending(segment: &str) -> (&str, &str) {
    if let Some(line) = segment.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = segment.strip_suffix('\n') {
        (line, "\n")
    } else {
        (segment, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar(value: &str) -> FrontmatterValue {
        FrontmatterValue::Scalar(value.to_string())
    }

    #[test]
    fn parses_scalars_and_inline_lists() {
        let parsed =
            parse("---\nid: proposal.0001\nauthors: [a, b]\nsupersedes: []\n---\n# Body\n")
                .unwrap();
        assert_eq!(parsed["id"], scalar("proposal.0001"));
        assert_eq!(
            parsed["authors"],
            FrontmatterValue::List(vec!["a".into(), "b".into()])
        );
        assert_eq!(parsed["supersedes"], FrontmatterValue::List(vec![]));
    }

    #[test]
    fn skips_blank_and_comment_lines() {
        let parsed = parse("---\n\n# note\ntitle: T\n---\n").unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn accepts_crlf_line_endings() {
        let parsed = parse("---\r\ntitle: T\r\n---\r\n").unwrap();
        assert_eq!(parsed["title"], scalar("T"));
    }

    #[test]
    fn rejects_missing_unterminated_and_malformed_frontmatter() {
        assert_eq!(
            parse("# No frontmatter").unwrap_err(),
            "missing YAML frontmatter"
        );
        assert_eq!(
            parse("---\ntitle: T\n").unwrap_err(),
            "unterminated YAML frontmatter"
        );
        assert_eq!(
            parse("---\nNot A Key\n---\n").unwrap_err(),
            "cannot parse frontmatter line \"Not A Key\""
        );
        assert_eq!(
            parse("---\na: 1\na: 2\n---\n").unwrap_err(),
            "duplicate frontmatter key \"a\""
        );
        for list in ["[a, b", "[a, [b]]", "[a, , b]"] {
            assert_eq!(
                parse(&format!("---\nlist: {list}\n---\n")).unwrap_err(),
                "cannot parse list value for \"list\""
            );
        }
    }

    #[test]
    fn removes_fenced_blocks_and_inline_code_but_keeps_prose() {
        let markdown = "keep `drop` this\n```md\n[NEEDS CLARIFICATION: x]\n```\nafter\n~~~~\ninside\n~~~\nstill inside\n~~~~\nend\n";
        let prose = without_code(markdown);
        assert_eq!(prose, "keep  this\n\nafter\n\nend\n");
    }

    #[test]
    fn treats_an_info_string_with_a_backtick_as_inline_code() {
        assert_eq!(without_code("```not a fence` text\n"), " text\n");
    }

    #[test]
    fn an_unclosed_fence_hides_the_rest() {
        assert_eq!(without_code("a\n```\nb\nc"), "a\n");
    }
}
