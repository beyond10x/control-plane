//! Scenario names in the acceptance of active and implemented stories resolve to tests.
//!
//! A story's `## Acceptance` section names the scenarios that decide it. The rule for what counts
//! as a name is the one `story:acceptance-traceability` states: a token of three or more lower-case
//! words joined by `_` or `-`, backticked or not. A backticked span that contains whitespace is a
//! command and is skipped whole, and so is a fenced code block (opened by three or more backticks
//! or tildes), whose lines never end the section; a checked story whose body ends inside an open
//! fence is refused, since the fence can hide its Acceptance heading. Any other backticked span is
//! split like plain text, so `` `name()` `` gives `name`. An artifact id (`<kind>:<slug>`) and a
//! path (a token containing `/` or `.`) are single tokens that never match the name shape, so no
//! part of them is a name either. A word is lower-case letters and digits; the first word starts
//! with a letter, so a date such as `2026-10-06` is not a name.
//!
//! A name resolves when a Rust test function (`#[test]` or `#[tokio::test]`) anywhere under
//! `crates/` has that name, with `_` in place of each `-`, or when a `test(...)` or `it(...)` call
//! in a `frontend/src/**/*.test.js` file has that title. Resolution reads source text and never
//! builds or lists the tests: it skips comments, string and regular-expression literals, the body
//! of a `macro_rules!` macro its file never invokes and whatever `#[cfg(any())]` or `#[cfg(false)]`
//! switches off, and evaluates no other `cfg`, so a test switched off by a feature, a target or a
//! `mod` declaration in another file still resolves, and one in a macro invoked only from another
//! file does not.
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
            if ends_in_fence(body) {
                bail!(
                    "{id}: {} ends inside an open code fence, which can hide its Acceptance section",
                    path.display()
                );
            }
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
                .map(|value| value.trim().trim_matches(['"', '\'']))
        })
    };
    Some((field("id")?, field("status")?, body))
}

/// The text of a body's `## Acceptance` section, up to the next heading of the same or higher
/// level outside a fenced code block.
pub fn acceptance(body: &str) -> Option<String> {
    let mut fence = Fence::default();
    let mut lines = body.lines().map(|line| (fence.code(line), line));
    lines.find(|(code, line)| !code && line.trim_end() == "## Acceptance")?;
    let section: Vec<&str> = lines
        .take_while(|(code, line)| *code || !(line.starts_with("## ") || line.starts_with("# ")))
        .map(|(_, line)| line)
        .collect();
    Some(section.join("\n"))
}

/// Whether `body` ends inside a fenced code block, whose opening line can hide any heading after
/// it.
fn ends_in_fence(body: &str) -> bool {
    let mut fence = Fence::default();
    for line in body.lines() {
        fence.code(line);
    }
    fence.0.is_some()
}

/// Which lines of Markdown belong to a fenced code block: one opened by a line starting with
/// three or more backticks or tildes and closed by a line of at least as many of the same. A block
/// whose opening line is indented, as in a list item, also ends where the item does: at the first
/// line that is not blank and is indented less.
#[derive(Default)]
struct Fence(Option<Open>);

/// The opening line of a fenced block: its marker, how many of it, and its indentation.
#[derive(Clone, Copy)]
struct Open {
    marker: char,
    length: usize,
    indent: usize,
}

impl Fence {
    /// Whether `line`, the next line of the text, opens, lies in or closes a fenced block.
    fn code(&mut self, line: &str) -> bool {
        let text = line.trim_start();
        let indent = line.len() - text.len();
        let marker = text
            .chars()
            .next()
            .filter(|first| matches!(first, '`' | '~'));
        let length = marker.map_or(0, |marker| {
            text.chars().take_while(|c| *c == marker).count()
        });
        let rest = &text[length..];
        if let Some(open) = self.0 {
            if open.indent == 0 || text.is_empty() || indent >= open.indent {
                if marker == Some(open.marker) && length >= open.length && rest.trim().is_empty() {
                    self.0 = None;
                }
                return true;
            }
            // The list item has ended, and the block with it; the line is read afresh.
            self.0 = None;
        }
        match marker {
            // A backtick line with another backtick after the run is inline code, not a fence.
            Some(marker) if length >= 3 && !(marker == '`' && rest.contains('`')) => {
                self.0 = Some(Open {
                    marker,
                    length,
                    indent,
                });
                true
            }
            _ => false,
        }
    }
}

/// The scenario names `text` lists, in order of first appearance, each once. Fenced code blocks
/// are skipped whole.
pub fn scenario_names(text: &str) -> Vec<String> {
    let mut fence = Fence::default();
    let lines: Vec<(bool, &str)> = text.lines().map(|line| (fence.code(line), line)).collect();
    let mut names = Vec::new();
    for run in lines.chunk_by(|line, next| line.0 == next.0) {
        if !run[0].0 {
            let prose: Vec<&str> = run.iter().map(|(_, line)| *line).collect();
            prose_names(&prose.join("\n"), &mut names);
        }
    }
    names
}

/// Adds the names in the Markdown prose `text` that `names` does not hold yet.
fn prose_names(text: &str, names: &mut Vec<String>) {
    let parts: Vec<&str> = text.split('`').collect();
    for (index, part) in parts.iter().enumerate() {
        // Odd parts lie between backticks; the text after an unclosed backtick stays plain. A
        // backticked span holding whitespace is a command; any other is split like plain text.
        if index % 2 == 1 && index + 1 < parts.len() && part.chars().any(char::is_whitespace) {
            continue;
        }
        for token in part.split(|character: char| {
            character.is_whitespace() || matches!(character, ',' | ';' | '(' | ')' | '[' | ']')
        }) {
            let token = token
                .trim_start_matches(['"', '\'', '*'])
                .trim_end_matches(['.', ',', ';', ':', '!', '?', '"', '\'', '*']);
            if is_name(token) && !names.iter().any(|name| name == token) {
                names.push(token.to_owned());
            }
        }
    }
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
/// arguments) marks. Comments, string and character literals and the body of a `macro_rules!`
/// macro that `source` invokes nowhere outside it are skipped, and so is an item under
/// `#[cfg(any())]` or `#[cfg(false)]` and the rest of a module or file that such an inner
/// attribute (`#![cfg(any())]`) opens; no other `cfg` is evaluated.
pub fn rust_test_names(source: &str) -> Vec<String> {
    let tokens = rust_tokens(source);
    let mut names = Vec::new();
    let mut marked = false;
    let mut index = 0;
    while index < tokens.len() {
        index = match tokens[index] {
            Token::Punct('#') => match attribute(&tokens, index) {
                Some((inner, content, end)) if switched_off(content) => {
                    marked = false;
                    if inner {
                        block_end(&tokens, end)
                    } else {
                        item_end(&tokens, end)
                    }
                }
                Some((inner, content, end)) => {
                    marked |= !inner && is_test(content);
                    end
                }
                None => index + 1,
            },
            Token::Ident("macro_rules") if tokens.get(index + 1) == Some(&Token::Punct('!')) => {
                let end = item_end(&tokens, index + 2);
                match tokens.get(index + 2) {
                    // An invoked macro expands to its body, so the body is read like other code.
                    Some(Token::Ident(name))
                        if invoked(&tokens[..index], name) || invoked(&tokens[end..], name) =>
                    {
                        index + 1
                    }
                    _ => {
                        marked = false;
                        end
                    }
                }
            }
            Token::Ident("fn") if marked => {
                if let Some(Token::Ident(name)) = tokens.get(index + 1) {
                    names.push((*name).to_owned());
                }
                marked = false;
                index + 1
            }
            Token::Punct('{' | '}' | ';') => {
                marked = false;
                index + 1
            }
            _ => index + 1,
        };
    }
    names
}

/// Whether `tokens` invoke the macro `name`: `name!` and the delimiter that opens its input.
fn invoked(tokens: &[Token], name: &str) -> bool {
    tokens.windows(3).any(|window| {
        matches!(
            window,
            [Token::Ident(ident), Token::Punct('!'), Token::Punct('(' | '[' | '{')] if *ident == name
        )
    })
}

/// The attribute whose `#` is at `at`: whether it is an inner one (`#![…]`), the tokens between
/// its brackets and the offset just past them.
fn attribute<'t, 'a>(tokens: &'t [Token<'a>], at: usize) -> Option<(bool, &'t [Token<'a>], usize)> {
    let inner = tokens.get(at + 1) == Some(&Token::Punct('!'));
    let open = at + 1 + usize::from(inner);
    if tokens.get(open) != Some(&Token::Punct('[')) {
        return None;
    }
    let close = group_close(tokens, open).unwrap_or(tokens.len());
    Some((inner, &tokens[open + 1..close], close + 1))
}

/// `test` or `tokio::test`, with or without arguments.
fn is_test(content: &[Token]) -> bool {
    let path: Vec<&str> = content
        .iter()
        .take_while(|token| matches!(token, Token::Ident(_) | Token::Punct(':')))
        .filter_map(|token| match token {
            Token::Ident(ident) => Some(*ident),
            _ => None,
        })
        .collect();
    matches!(path.as_slice(), ["test"] | ["tokio", "test"])
}

/// `cfg(any())` and `cfg(false)`, the two conditions no build satisfies.
fn switched_off(content: &[Token]) -> bool {
    use Token::{Ident, Punct};
    matches!(
        content,
        [
            Ident("cfg"),
            Punct('('),
            Ident("any"),
            Punct('('),
            Punct(')'),
            Punct(')')
        ] | [Ident("cfg"), Punct('('), Ident("false"), Punct(')')]
    )
}

/// The offset of the delimiter that closes the group whose opening delimiter is at `open`.
fn group_close(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::Punct('(' | '[' | '{') => depth += 1,
            Token::Punct(')' | ']' | '}') => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
    }
    None
}

/// The offset just past the item that starts at `start`: past its `{…}` body, or past the `;` of
/// an item without one.
fn item_end(tokens: &[Token], start: usize) -> usize {
    let mut at = start;
    while at < tokens.len() {
        match tokens[at] {
            Token::Punct('{') => {
                return group_close(tokens, at).map_or(tokens.len(), |close| close + 1);
            }
            Token::Punct('(' | '[') => {
                at = group_close(tokens, at).map_or(tokens.len(), |close| close + 1)
            }
            Token::Punct(';') => return at + 1,
            Token::Punct(')' | ']' | '}') => return at,
            _ => at += 1,
        }
    }
    tokens.len()
}

/// The offset of the delimiter that closes the block `start` lies in, or the end of the file.
fn block_end(tokens: &[Token], start: usize) -> usize {
    let mut at = start;
    while at < tokens.len() {
        match tokens[at] {
            Token::Punct('(' | '[' | '{') => {
                at = group_close(tokens, at).map_or(tokens.len(), |close| close + 1)
            }
            Token::Punct(')' | ']' | '}') => return at,
            _ => at += 1,
        }
    }
    tokens.len()
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
/// not counted; comments, regular-expression literals and other string literals are skipped.
pub fn frontend_test_titles(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut titles = Vec::new();
    let mut at = 0;
    // The last byte that is not whitespace: `a` after a name, `0` after a literal.
    let mut previous = b' ';
    // Whether that name is a keyword after which an expression, so a regular expression, starts.
    let mut keyword = false;
    while at < bytes.len() {
        let byte = bytes[at];
        let next = bytes.get(at + 1).copied();
        if byte == b'/' && next == Some(b'/') {
            at = source[at..].find('\n').map_or(bytes.len(), |end| at + end);
        } else if byte == b'/' && next == Some(b'*') {
            at = source[at + 2..]
                .find("*/")
                .map_or(bytes.len(), |end| at + 2 + end + 2);
        } else if byte == b'/'
            && regex_may_start(previous, keyword)
            && let Some(end) = regex_end(bytes, at)
        {
            at = end;
            previous = b'0';
        } else if matches!(byte, b'"' | b'\'' | b'`') {
            at = quoted(bytes, at + 1, byte);
            previous = b'0';
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
            keyword = matches!(
                ident,
                "return"
                    | "typeof"
                    | "instanceof"
                    | "in"
                    | "of"
                    | "new"
                    | "delete"
                    | "void"
                    | "throw"
                    | "case"
                    | "do"
                    | "else"
                    | "yield"
                    | "await"
            );
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

/// Whether a `/` after `previous` starts a regular expression rather than dividing: not after a
/// literal, a number, a closing `)` or `]`, or a name other than a keyword.
fn regex_may_start(previous: u8, keyword: bool) -> bool {
    match previous {
        b'a' => keyword,
        b'0' | b')' | b']' => false,
        byte => !byte.is_ascii_digit(),
    }
}

/// The offset just past the regular-expression literal whose opening `/` is at `at`, flags
/// included, or `None` when the line ends first.
fn regex_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut class = false;
    let mut end = at + 1;
    loop {
        match *bytes.get(end)? {
            b'\n' => return None,
            b'\\' => end += 1,
            b'[' => class = true,
            b']' => class = false,
            b'/' if !class => break,
            _ => {}
        }
        end += 1;
    }
    end += 1;
    while bytes.get(end).is_some_and(u8::is_ascii_alphabetic) {
        end += 1;
    }
    Some(end)
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
