//! What `cli search` and `docs search` share: query words and bounded, resumable pages.

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

use crate::failure::Failure;

const STOP_WORDS: &[&str] = &[
    "a",
    "an",
    "and",
    "are",
    "as",
    "at",
    "be",
    "by",
    "can",
    "do",
    "does",
    "for",
    "from",
    "how",
    "i",
    "in",
    "into",
    "is",
    "it",
    "me",
    "my",
    "of",
    "on",
    "or",
    "the",
    "this",
    "that",
    "to",
    "what",
    "when",
    "where",
    "which",
    "with",
    "workbench",
];

/// The lowercase words of `text` that carry meaning for a search.
pub fn query_words(text: &str) -> Vec<String> {
    let mut words: Vec<String> = text
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty() && !STOP_WORDS.contains(word))
        .map(str::to_string)
        .collect();
    words.dedup();
    words
}

/// Every lowercase word of `text`, stop words included.
pub fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

/// How well `query` matches `word`: 1 for the same word, 0.6 for the same stem (`claim` and
/// `claims`, `validate` and `validation`), else 0. Two words share a stem when they share a
/// prefix of at least four letters that leaves at most two letters of the shorter word and at
/// most four of the longer one, so `work` does not match `workbench`.
pub fn word_match(query: &str, word: &str) -> f64 {
    if query == word {
        return 1.0;
    }
    let common = query
        .chars()
        .zip(word.chars())
        .take_while(|(left, right)| left == right)
        .count();
    let (shorter, longer) = if query.len() <= word.len() {
        (query, word)
    } else {
        (word, query)
    };
    let shorter = shorter.chars().count();
    let longer = longer.chars().count();
    if common >= 4 && shorter - common <= 2 && longer - common <= 4 {
        0.6
    } else {
        0.0
    }
}

/// The best match of `query` against any of `words`.
pub fn best_match(query: &str, words: &[String]) -> f64 {
    words
        .iter()
        .map(|word| word_match(query, word))
        .fold(0.0, f64::max)
}

/// One page of a ranked collection.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Whether more matches follow this page.
    pub has_more: bool,
    /// Pass as `--cursor` to get the next page; null when there is none.
    pub next_cursor: Option<String>,
}

/// Slices `ranked` from `cursor` (an offset from a previous `nextCursor`) to at most `limit` items.
pub fn page<T>(ranked: Vec<T>, cursor: Option<&str>, limit: u64) -> Result<Page<T>, Failure> {
    if limit == 0 {
        return Err(Failure::invalid_argument("--limit must be at least 1")
            .with_details(json!({ "argument": "limit", "value": limit })));
    }
    let offset = match cursor {
        None => 0,
        Some(cursor) => cursor.parse::<usize>().map_err(|_| {
            Failure::invalid_argument(format!(
                "--cursor `{cursor}` is not a cursor this command returned"
            ))
            .with_details(json!({ "argument": "cursor", "value": cursor }))
            .suggest("Pass the nextCursor value from the previous page, or omit --cursor")
        })?,
    };
    let total = ranked.len();
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let end = offset.saturating_add(limit).min(total);
    let items: Vec<T> = ranked.into_iter().skip(offset).take(limit).collect();
    let has_more = end < total;
    Ok(Page {
        items,
        has_more,
        next_cursor: has_more.then(|| end.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_stop_words_and_punctuation() {
        assert_eq!(
            query_words("Claim the next ready item!"),
            ["claim", "next", "ready", "item"]
        );
    }

    #[test]
    fn matches_words_that_share_a_stem() {
        assert_eq!(word_match("claim", "claims"), 0.6);
        assert_eq!(word_match("validate", "validation"), 0.6);
        assert_eq!(word_match("add", "address"), 0.0);
        assert_eq!(word_match("work", "workbench"), 0.0);
        assert_eq!(word_match("sync", "sync"), 1.0);
    }

    #[test]
    fn pages_resume_from_the_returned_cursor() {
        let first = page((0..7).collect(), None, 3).unwrap();
        assert_eq!(first.items, [0, 1, 2]);
        assert!(first.has_more);
        let second = page((0..7).collect(), first.next_cursor.as_deref(), 3).unwrap();
        assert_eq!(second.items, [3, 4, 5]);
        let last = page((0..7).collect(), second.next_cursor.as_deref(), 3).unwrap();
        assert_eq!(last.items, [6]);
        assert!(!last.has_more);
        assert_eq!(last.next_cursor, None);
    }

    #[test]
    fn rejects_a_cursor_it_did_not_issue() {
        let failure = page(vec![1], Some("abc"), 3).err().unwrap();
        assert_eq!(failure.code, crate::failure::ErrorCode::InvalidArgument);
    }
}
