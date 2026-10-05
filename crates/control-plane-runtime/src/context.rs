//! Bounded, explicit observations for model input. Full files remain available through paging.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

const ENTRY_BYTES: usize = 16 * 1024;
const CONTEXT_BYTES: usize = 96 * 1024;

pub fn excerpt(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[Excerpt: {} of {} bytes shown. Request a relevant file page with read_range; omitted content is not observed here.]",
        &text[..end],
        end,
        text.len()
    )
}

pub fn index(label: &str, raw: &str) -> String {
    format!("{label}:\n{}", excerpt(raw, ENTRY_BYTES))
}

pub fn push(transcript: &mut Vec<String>, message: String) {
    let message = excerpt(&message, ENTRY_BYTES);
    // Repeated reads refresh the same observation instead of growing the prompt forever.
    if let Some(index) = transcript.iter().skip(3).position(|old| old == &message) {
        transcript.remove(index + 3);
    }
    transcript.push(message);
    while transcript.len() > 4 && transcript.iter().map(String::len).sum::<usize>() > CONTEXT_BYTES
    {
        transcript.remove(3);
    }
}

pub fn scan(raw: &str) -> Result<String> {
    let scan: Value = serde_json::from_str(raw)?;
    let mut summary = serde_json::Map::new();
    for (name, value) in scan.as_object().into_iter().flatten() {
        summary.insert(name.clone(), match value.as_array() {
            Some(items) => json!({"total":items.len(),"sample":items.iter().take(12).map(|item|{
                if let Some(fields)=item.as_object() {
                    Value::Object(fields.iter().filter(|(key,_)| !matches!(key.as_str(),"operations"|"body"|"contents")).map(|(k,v)|(k.clone(),v.clone())).collect())
                } else { item.clone() }
            }).collect::<Vec<_>>() }),
            None => value.clone(),
        });
    }
    Ok(index(
        "Repository scan index (counts and sampled locations; inspect relevant files)",
        &serde_json::to_string(&summary)?,
    ))
}

pub fn backlog(raw: &str) -> Result<String> {
    let items: Vec<Value> = serde_json::from_str(raw)?;
    let entries: Vec<_> = items.iter().map(|item|json!({"id":item["id"],"kind":item["kind"],"status":item["status"],"title":item["title"],"scope":item["scope"],"relations":item["relations"]})).collect();
    Ok(index(
        "Existing AEP index (use aep show to inspect relevant full artifacts)",
        &serde_json::to_string(&entries)?,
    ))
}

pub fn page(name: &str, content: &str, start: usize, count: usize) -> Result<String> {
    ensure!(
        start > 0 && (1..=200).contains(&count),
        "read_range uses a positive start_line and 1..=200 lines"
    );
    let total = content.lines().count();
    let mut body = String::new();
    let mut next = start;
    for (index, line) in content.lines().enumerate().skip(start - 1).take(count) {
        let rendered = format!("{}: {line}\n", index + 1);
        if body.len() + rendered.len() > 12 * 1024 && !body.is_empty() {
            break;
        }
        body.push_str(&excerpt(&rendered, 12 * 1024));
        next = index + 2;
    }
    Ok(format!(
        "File {name}, lines {start}..{} of {total}\n{body}\n{}",
        next.saturating_sub(1),
        if next <= total {
            format!(
                "More content available: read_range path={name:?} start_line={next} line_count={count}"
            )
        } else {
            "End of file page.".into()
        }
    ))
}
