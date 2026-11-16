//! Adding a member to an object in a JSON file as text, so the rest of the file keeps its key
//! order, indentation, and spacing.

use serde::Serialize;
use serde_json::Value;
use serde_json::ser::PrettyFormatter;

const DEFAULT_UNIT: &str = "    ";

/// `source` with `"key": value` added as the last member of the object at `path` (a list of
/// member names from the root object). The source must be valid JSON in which that object
/// exists.
pub fn add_member(source: &str, path: &[&str], key: &str, value: &Value) -> Result<String, String> {
    let bytes = source.as_bytes();
    let mut object = parse_object(bytes, skip_whitespace(bytes, 0))?;
    for name in path {
        let member = object
            .members
            .iter()
            .find(|member| member.key == *name)
            .ok_or_else(|| format!("no `{name}` member"))?;
        object = parse_object(bytes, member.value_start)?;
    }

    let base = line_indent(source, object.open);
    let member_text = |indent: &str, unit: &str| {
        let key = serde_json::to_string(key).expect("strings serialize");
        let value = pretty(value, unit).replace('\n', &format!("\n{indent}"));
        format!("{key}: {value}")
    };
    let edited = match object.members.last() {
        Some(last) if source[object.open..last.key_start].contains('\n') => {
            let indent = line_indent(source, last.key_start);
            let unit = indent.strip_prefix(base).filter(|unit| !unit.is_empty());
            let addition = format!(
                ",\n{indent}{}",
                member_text(indent, unit.unwrap_or(DEFAULT_UNIT))
            );
            splice(source, last.value_end, last.value_end, &addition)
        }
        Some(last) => {
            let compact = format!(
                ", {}: {}",
                serde_json::to_string(key).expect("strings serialize"),
                serde_json::to_string(value).expect("values serialize")
            );
            splice(source, last.value_end, last.value_end, &compact)
        }
        None => {
            let indent = format!("{base}{DEFAULT_UNIT}");
            let inner = format!("\n{indent}{}\n{base}", member_text(&indent, DEFAULT_UNIT));
            splice(source, object.open + 1, object.close, &inner)
        }
    };
    Ok(edited)
}

/// A new JSON file holding `members` in order, indented four spaces.
pub fn new_document(members: &[(&str, Value)]) -> String {
    let mut document = "{}\n".to_string();
    for (key, value) in members {
        document = add_member(&document, &[], key, value).expect("the new document is an object");
    }
    document
}

struct Object {
    open: usize,
    close: usize,
    members: Vec<Member>,
}

struct Member {
    key: String,
    key_start: usize,
    value_start: usize,
    value_end: usize,
}

fn parse_object(bytes: &[u8], open: usize) -> Result<Object, String> {
    if bytes.get(open) != Some(&b'{') {
        return Err("not an object".into());
    }
    let mut members = Vec::new();
    let mut index = skip_whitespace(bytes, open + 1);
    while bytes.get(index) == Some(&b'"') {
        let key_end = string_end(bytes, index)?;
        let key: String = serde_json::from_slice(&bytes[index..key_end])
            .map_err(|error| format!("bad key: {error}"))?;
        let colon = skip_whitespace(bytes, key_end);
        if bytes.get(colon) != Some(&b':') {
            return Err("expected `:`".into());
        }
        let value_start = skip_whitespace(bytes, colon + 1);
        let value_end = value_end(bytes, value_start)?;
        members.push(Member {
            key,
            key_start: index,
            value_start,
            value_end,
        });
        index = skip_whitespace(bytes, value_end);
        if bytes.get(index) == Some(&b',') {
            index = skip_whitespace(bytes, index + 1);
        }
    }
    if bytes.get(index) != Some(&b'}') {
        return Err("expected `}`".into());
    }
    Ok(Object {
        open,
        close: index,
        members,
    })
}

fn value_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    match bytes.get(start) {
        Some(b'"') => string_end(bytes, start),
        Some(b'{' | b'[') => {
            let mut depth = 0usize;
            let mut index = start;
            while let Some(&byte) = bytes.get(index) {
                match byte {
                    b'"' => {
                        index = string_end(bytes, index)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(index + 1);
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            Err("unterminated value".into())
        }
        Some(_) => Ok(bytes[start..]
            .iter()
            .position(|byte| b",}] \t\r\n".contains(byte))
            .map_or(bytes.len(), |offset| start + offset)),
        None => Err("missing value".into()),
    }
}

/// The index just past the string starting at `start`.
fn string_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    let mut index = start + 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index += 2,
            b'"' => return Ok(index + 1),
            _ => index += 1,
        }
    }
    Err("unterminated string".into())
}

fn skip_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    index
}

/// The whitespace that begins the line containing `index`.
fn line_indent(source: &str, index: usize) -> &str {
    let line_start = source[..index].rfind('\n').map_or(0, |newline| newline + 1);
    let line = &source[line_start..];
    &line[..line.len() - line.trim_start().len()]
}

fn pretty(value: &Value, unit: &str) -> String {
    let mut text = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(
        &mut text,
        PrettyFormatter::with_indent(unit.as_bytes()),
    );
    value.serialize(&mut serializer).expect("values serialize");
    String::from_utf8(text).expect("serde_json writes UTF-8")
}

fn splice(source: &str, start: usize, end: usize, insertion: &str) -> String {
    format!("{}{insertion}{}", &source[..start], &source[end..])
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn appends_with_the_file_s_indentation() {
        let source = "{\n  \"permissions\": { \"allow\": [] },\n  \"enabledPlugins\": {\n    \"other@x\": true\n  }\n}\n";
        let edited = add_member(
            source,
            &["enabledPlugins"],
            "workbench@workbench",
            &json!(true),
        )
        .unwrap();
        assert_eq!(
            edited,
            "{\n  \"permissions\": { \"allow\": [] },\n  \"enabledPlugins\": {\n    \"other@x\": true,\n    \"workbench@workbench\": true\n  }\n}\n"
        );
        let edited = add_member(source, &[], "extra", &json!({ "a": 1 })).unwrap();
        assert!(
            edited.ends_with("  },\n  \"extra\": {\n    \"a\": 1\n  }\n}\n"),
            "{edited}"
        );
    }

    #[test]
    fn fills_an_empty_object_and_extends_a_single_line_one() {
        assert_eq!(
            add_member("{}", &[], "a", &json!(true)).unwrap(),
            "{\n    \"a\": true\n}"
        );
        assert_eq!(
            add_member("{ \"x\": { \"b\": \"}\" } }", &["x"], "c", &json!(1)).unwrap(),
            "{ \"x\": { \"b\": \"}\", \"c\": 1 } }"
        );
    }

    #[test]
    fn a_new_document_keeps_its_members_in_order() {
        let document = new_document(&[("z", json!(1)), ("a", json!({ "b": true }))]);
        assert_eq!(
            document,
            "{\n    \"z\": 1,\n    \"a\": {\n        \"b\": true\n    }\n}\n"
        );
        let parsed: Value = serde_json::from_str(&document).unwrap();
        assert_eq!(parsed, json!({ "z": 1, "a": { "b": true } }));
    }

    #[test]
    fn rejects_a_missing_object() {
        assert!(add_member("{ \"x\": 1 }", &["x"], "a", &json!(1)).is_err());
        assert!(add_member("{ \"x\": 1 }", &["y"], "a", &json!(1)).is_err());
    }
}
