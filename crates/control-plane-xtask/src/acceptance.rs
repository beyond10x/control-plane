//! Scenario names in the acceptance of active and implemented stories resolve to tests.
//!
//! A story's `## Acceptance` section names the scenarios that decide it. The rule for what counts
//! as a name is the one `story:acceptance-traceability` states: a token of three or more lower-case
//! words joined by `_` or `-`, backticked or not. A backticked span that contains whitespace is a
//! command and is skipped whole; an artifact id (`<kind>:<slug>`) and a path (a token containing
//! `/` or `.`) are single tokens that never match the name shape, so no part of them is a name
//! either. A word is lower-case letters and digits; the first word starts with a letter, so a date
//! such as `2026-10-06` is not a name.
//!
//! A name resolves when a Rust test function (`#[test]` or `#[tokio::test]`) anywhere under
//! `crates/` has that name, with `_` in place of each `-`, or when a `test(...)` or `it(...)` call
//! in a `frontend/src/**/*.test.js` file has that title.
use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// Where the planning store keeps stories, relative to the repository root.
pub const STORIES: &str = ".engineering/planning/story";
/// Story statuses whose scenario names must resolve.
pub const CHECKED: [&str; 2] = ["active", "implemented"];

pub fn run(root: &Path) -> Result<()> {
    println!("{}", check(root)?);
    Ok(())
}

/// A story whose status is checked, with the scenario names its acceptance lists.
#[derive(Debug, PartialEq, Eq)]
pub struct Story {
    pub id: String,
    pub names: Vec<String>,
}

/// Every name in every checked story under `root` resolves, or the error lists
/// `<story id>: <name>` for each one that does not.
pub fn check(root: &Path) -> Result<String> {
    let stories = stories(&root.join(STORIES))?;
    let tests = rust_tests(&root.join("crates"))?;
    let titles = frontend_titles(&root.join("frontend/src"))?;
    let names: usize = stories.iter().map(|story| story.names.len()).sum();
    let unresolved: Vec<String> = stories
        .iter()
        .flat_map(|story| {
            story
                .names
                .iter()
                .filter(|name| !resolves(name, &tests, &titles))
                .map(move |name| format!("{}: {name}", story.id))
        })
        .collect();
    if !unresolved.is_empty() {
        bail!(
            "{} of {names} scenario names in active and implemented stories name no test:\n{}",
            unresolved.len(),
            unresolved.join("\n")
        );
    }
    Ok(format!(
        "{names} scenario names in {} active and implemented stories resolve to tests",
        stories.len()
    ))
}

fn resolves(name: &str, tests: &BTreeSet<String>, titles: &BTreeSet<String>) -> bool {
    let underscored = name.replace('-', "_");
    tests.contains(&underscored) || titles.contains(name) || titles.contains(&underscored)
}

/// The checked stories in `dir`, in file-name order, each with the names its acceptance lists.
pub fn stories(dir: &Path) -> Result<Vec<Story>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("read stories in {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "md"));
    paths.sort();
    let mut stories = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let (id, status, body) = story_parts(&text)
            .with_context(|| format!("{} has no story header", path.display()))?;
        if CHECKED.contains(&status) {
            stories.push(Story {
                id: id.to_owned(),
                names: acceptance(body)
                    .map(|section| scenario_names(&section))
                    .unwrap_or_default(),
            });
        }
    }
    Ok(stories)
}

/// The `id`, `status` and body of a planning-store story file.
fn story_parts(text: &str) -> Option<(&str, &str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let (header, body) = (&rest[..end], &rest[end + "\n---\n".len()..]);
    let field = |key: &str| {
        header.lines().find_map(|line| {
            line.strip_prefix(key)
                .and_then(|value| value.strip_prefix(':'))
                .map(|value| value.trim().trim_matches('"'))
        })
    };
    Some((field("id")?, field("status")?, body))
}

/// The text of a body's `## Acceptance` section, up to the next heading of the same or higher
/// level.
pub fn acceptance(body: &str) -> Option<String> {
    let mut lines = body.lines();
    lines
        .by_ref()
        .find(|line| line.trim_end() == "## Acceptance")?;
    let section: Vec<&str> = lines
        .take_while(|line| !(line.starts_with("## ") || line.starts_with("# ")))
        .collect();
    Some(section.join("\n"))
}

/// The scenario names `text` lists, in order of first appearance, each once.
pub fn scenario_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut keep = |token: &str| {
        let token = token
            .trim_start_matches(['"', '\'', '*'])
            .trim_end_matches(['.', ',', ';', ':', '!', '?', '"', '\'', '*']);
        if is_name(token) && !names.iter().any(|name| name == token) {
            names.push(token.to_owned());
        }
    };
    let parts: Vec<&str> = text.split('`').collect();
    for (index, part) in parts.iter().enumerate() {
        // Odd parts lie between backticks; the text after an unclosed backtick stays plain.
        if index % 2 == 1 && index + 1 < parts.len() {
            if !part.chars().any(char::is_whitespace) {
                keep(part);
            }
            continue;
        }
        for token in part.split(|character: char| {
            character.is_whitespace() || matches!(character, ',' | ';' | '(' | ')' | '[' | ']')
        }) {
            keep(token);
        }
    }
    names
}

/// Three or more lower-case words joined by `_` or `-`; the first word starts with a letter.
fn is_name(token: &str) -> bool {
    let words: Vec<&str> = token.split(['_', '-']).collect();
    words.len() >= 3
        && token.starts_with(|character: char| character.is_ascii_lowercase())
        && words.iter().all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

/// Names of `#[test]` and `#[tokio::test]` functions in every `.rs` file under `dir`.
pub fn rust_tests(dir: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for path in files(dir, &|path| path.extension().is_some_and(|ext| ext == "rs"))? {
        let source =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        names.extend(rust_test_names(&source));
    }
    Ok(names)
}

/// Titles of `test(...)` and `it(...)` calls in every `*.test.js` file under `dir`.
pub fn frontend_titles(dir: &Path) -> Result<BTreeSet<String>> {
    let mut titles = BTreeSet::new();
    if !dir.is_dir() {
        return Ok(titles);
    }
    let test_file = |path: &Path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".test.js"))
    };
    for path in files(dir, &test_file)? {
        let source =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        titles.extend(frontend_test_titles(&source));
    }
    Ok(titles)
}

/// Regular files under `dir` that `wanted` selects, without following symbolic links and without
/// entering build output (`target`) or installed packages (`node_modules`).
fn files(dir: &Path, wanted: &dyn Fn(&Path) -> bool) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_dir() {
                if !matches!(entry.file_name().to_str(), Some("target" | "node_modules")) {
                    pending.push(path);
                }
            } else if kind.is_file() && wanted(&path) {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Ident(&'a str),
    Punct(char),
    Literal,
}

/// The names of the functions an attribute `#[test]` or `#[tokio::test]` (with or without
/// arguments) marks. Comments, string and character literals are skipped, so a test written in a
/// string or commented out does not count.
pub fn rust_test_names(source: &str) -> Vec<String> {
    let tokens = rust_tokens(source);
    let mut names = Vec::new();
    let mut marked = false;
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index] {
            Token::Punct('#') if tokens.get(index + 1) == Some(&Token::Punct('[')) => {
                let path: Vec<&Token> = tokens[index + 2..]
                    .iter()
                    .take_while(|token| matches!(token, Token::Ident(_) | Token::Punct(':')))
                    .collect();
                let path: Vec<&str> = path
                    .iter()
                    .filter_map(|token| match token {
                        Token::Ident(ident) => Some(*ident),
                        _ => None,
                    })
                    .collect();
                marked |= matches!(path.as_slice(), ["test"] | ["tokio", "test"]);
                let mut depth = 0usize;
                index += 1;
                while index < tokens.len() {
                    match tokens[index] {
                        Token::Punct('[') => depth += 1,
                        Token::Punct(']') => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    index += 1;
                }
            }
            Token::Ident("fn") if marked => {
                if let Some(Token::Ident(name)) = tokens.get(index + 1) {
                    names.push((*name).to_owned());
                }
                marked = false;
            }
            Token::Punct('{' | '}' | ';') => marked = false,
            _ => {}
        }
        index += 1;
    }
    names
}

fn rust_tokens(source: &str) -> Vec<Token<'_>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        let next = bytes.get(at + 1).copied();
        if byte.is_ascii_whitespace() {
            at += 1;
        } else if byte == b'/' && next == Some(b'/') {
            at = source[at..].find('\n').map_or(bytes.len(), |end| at + end);
        } else if byte == b'/' && next == Some(b'*') {
            let mut depth = 0usize;
            while at < bytes.len() {
                if bytes[at..].starts_with(b"/*") {
                    depth += 1;
                    at += 2;
                } else if bytes[at..].starts_with(b"*/") {
                    depth -= 1;
                    at += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    at += 1;
                }
            }
        } else if byte == b'"' {
            at = quoted(bytes, at + 1, b'"');
            tokens.push(Token::Literal);
        } else if byte == b'\'' {
            at = character_or_lifetime(source, at);
            tokens.push(Token::Literal);
        } else if byte == b'_' || byte.is_ascii_alphabetic() || !byte.is_ascii() {
            let start = at;
            while at < bytes.len()
                && (bytes[at] == b'_' || bytes[at].is_ascii_alphanumeric() || !bytes[at].is_ascii())
            {
                at += 1;
            }
            let ident = &source[start..at];
            let raw_prefix = matches!(ident, "r" | "br" | "cr");
            let quote_prefix = matches!(ident, "b" | "c");
            match bytes.get(at) {
                Some(b'"') if raw_prefix => {
                    at = raw(bytes, at, 0);
                    tokens.push(Token::Literal);
                }
                Some(b'"') if quote_prefix => {
                    at = quoted(bytes, at + 1, b'"');
                    tokens.push(Token::Literal);
                }
                Some(b'\'') if ident == "b" => {
                    at = quoted(bytes, at + 1, b'\'');
                    tokens.push(Token::Literal);
                }
                Some(b'#') if raw_prefix => {
                    let hashes = bytes[at..].iter().take_while(|byte| **byte == b'#').count();
                    if bytes.get(at + hashes) == Some(&b'"') {
                        at = raw(bytes, at + hashes, hashes);
                        tokens.push(Token::Literal);
                    } else if ident == "r" && hashes == 1 {
                        // A raw identifier: `r#name`.
                        let start = at + 1;
                        at = start;
                        while at < bytes.len()
                            && (bytes[at] == b'_' || bytes[at].is_ascii_alphanumeric())
                        {
                            at += 1;
                        }
                        tokens.push(Token::Ident(&source[start..at]));
                    } else {
                        tokens.push(Token::Ident(ident));
                    }
                }
                _ => tokens.push(Token::Ident(ident)),
            }
        } else if byte.is_ascii_digit() {
            while at < bytes.len() && (bytes[at] == b'_' || bytes[at].is_ascii_alphanumeric()) {
                at += 1;
            }
            tokens.push(Token::Literal);
        } else {
            tokens.push(Token::Punct(byte as char));
            at += 1;
        }
    }
    tokens
}

/// The offset just past a literal whose body starts at `at` and ends at an unescaped `close`.
fn quoted(bytes: &[u8], mut at: usize, close: u8) -> usize {
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            byte if byte == close => return at + 1,
            _ => at += 1,
        }
    }
    bytes.len()
}

/// The offset just past a raw string whose opening `"` is at `at`, closed by `"` and `hashes` `#`.
fn raw(bytes: &[u8], at: usize, hashes: usize) -> usize {
    let mut close = vec![b'"'];
    close.extend(std::iter::repeat_n(b'#', hashes));
    bytes[at + 1..]
        .windows(close.len())
        .position(|window| window == close.as_slice())
        .map_or(bytes.len(), |end| at + 1 + end + close.len())
}

/// The offset just past a character literal or a lifetime (or label) starting with `'` at `at`.
fn character_or_lifetime(source: &str, at: usize) -> usize {
    let bytes = source.as_bytes();
    if bytes.get(at + 1) == Some(&b'\\') {
        return quoted(bytes, at + 1, b'\'');
    }
    let mut characters = source[at + 1..].char_indices();
    match (characters.next(), characters.next()) {
        (Some(_), Some((offset, '\''))) => at + 1 + offset + 1,
        _ => {
            let mut end = at + 1;
            while end < bytes.len() && (bytes[end] == b'_' || bytes[end].is_ascii_alphanumeric()) {
                end += 1;
            }
            end
        }
    }
}

/// Titles of `test('…', …)` and `it('…', …)` calls in a JavaScript test file. A call reached
/// through a member (`foo.test(…)`) or with a computed title (a template literal holding `${`) is
/// not counted; comments and other string literals are skipped.
pub fn frontend_test_titles(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut titles = Vec::new();
    let mut at = 0;
    let mut previous = b' ';
    while at < bytes.len() {
        let byte = bytes[at];
        let next = bytes.get(at + 1).copied();
        if byte == b'/' && next == Some(b'/') {
            at = source[at..].find('\n').map_or(bytes.len(), |end| at + end);
        } else if byte == b'/' && next == Some(b'*') {
            at = source[at + 2..]
                .find("*/")
                .map_or(bytes.len(), |end| at + 2 + end + 2);
        } else if matches!(byte, b'"' | b'\'' | b'`') {
            at = quoted(bytes, at + 1, byte);
            previous = byte;
        } else if byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic() {
            let start = at;
            while at < bytes.len()
                && (bytes[at] == b'_' || bytes[at] == b'$' || bytes[at].is_ascii_alphanumeric())
            {
                at += 1;
            }
            let ident = &source[start..at];
            if matches!(ident, "test" | "it")
                && previous != b'.'
                && let Some((title, end)) = call_title(source, at)
            {
                titles.push(title);
                at = end;
            }
            previous = b'a';
        } else {
            if !byte.is_ascii_whitespace() {
                previous = byte;
            }
            at += 1;
        }
    }
    titles
}

/// The static title of a call whose `(` follows `at`, and the offset just past the title.
fn call_title(source: &str, at: usize) -> Option<(String, usize)> {
    let rest = source[at..].trim_start();
    let rest = rest.strip_prefix('(')?.trim_start();
    let quote = rest
        .chars()
        .next()
        .filter(|c| matches!(c, '"' | '\'' | '`'))?;
    let start = source.len() - rest.len() + 1;
    let end = quoted(source.as_bytes(), start, quote as u8);
    let title = source.get(start..end.checked_sub(1)?)?;
    if quote == '`' && title.contains("${") {
        return None;
    }
    Some((title.to_owned(), end))
}
