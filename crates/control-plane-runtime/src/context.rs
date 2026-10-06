//! Bounded, explicit observations for model input. Full files remain available through paging.
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::collections::VecDeque;

const ENTRY_BYTES: usize = 16 * 1024;
const CONTEXT_BYTES: usize = 96 * 1024;
const PAGE_BYTES: usize = 12 * 1024;

/// Model-facing task context. The durable goal remains the host's authority;
/// receipts and activity are observations, never part of a repeated task brief.
pub fn goal_brief(goal: &Value) -> Value {
    let mut brief = serde_json::Map::new();
    for key in [
        "goal_id",
        "workspace_id",
        "revision",
        "objective",
        "acceptance",
        "directories",
        "max_minutes",
        "max_attempts",
    ] {
        if let Some(value) = goal.get(key) {
            brief.insert(key.into(), value.clone());
        }
    }
    Value::Object(brief)
}

#[derive(Default)]
pub struct ActionMemory {
    steps: usize,
    journal: VecDeque<String>,
    observations: VecDeque<(String, String, usize)>,
}

impl ActionMemory {
    pub fn attempted(&mut self, label: &str) {
        self.steps += 1;
        self.note(&format!("attempted {label}"));
    }

    pub fn note(&mut self, message: &str) {
        self.journal
            .push_back(format!("Step {}: {}", self.steps, excerpt(message, 2048)));
        while self.journal.len() > 12 {
            self.journal.pop_front();
        }
    }

    pub fn observed(&mut self, key: String, digest: String) -> usize {
        let previous = self
            .observations
            .iter()
            .position(|(prior, _, _)| prior == &key)
            .and_then(|index| self.observations.remove(index));
        let count = match previous {
            Some((_, previous_digest, count)) if previous_digest == digest => count + 1,
            _ => 1,
        };
        self.observations.push_back((key, digest, count));
        while self.observations.len() > 64 {
            self.observations.pop_front();
        }
        count
    }

    pub fn changed(&mut self) {
        self.observations.clear();
        self.note("repository mutation completed; prior unchanged-read counts reset");
    }

    pub fn prompt(&self) -> String {
        format!(
            "Recent attempted actions ({} total steps; bounded history):\n{}",
            self.steps,
            self.journal.iter().cloned().collect::<Vec<_>>().join("\n")
        )
    }
}

pub fn action_label(action: &Value) -> String {
    let mut label = action.clone();
    if let Some(fields) = label.as_object_mut() {
        for key in ["body", "contents", "summary"] {
            fields.remove(key);
        }
    }
    excerpt(&label.to_string(), 2048)
}

pub fn bytes(name: &str, content: &str, start: usize, count: usize) -> Result<String> {
    ensure!(
        (1..=PAGE_BYTES).contains(&count),
        "read_bytes byte_count must be 1..=12288"
    );
    ensure!(
        start <= content.len() && content.is_char_boundary(start),
        "read_bytes start_byte must be an existing UTF-8 boundary (or end of file)"
    );
    let reserve=format!("File {name}, UTF-8 bytes {start}..{} of {} (end exclusive)\n\nMore content available: read_bytes path={name:?} start_byte={} byte_count={count}",content.len(),content.len(),content.len()).len();
    ensure!(
        reserve < PAGE_BYTES,
        "file name exceeds page metadata budget"
    );
    let mut end = start + count.min(PAGE_BYTES - reserve).min(content.len() - start);
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    ensure!(
        end > start || start == content.len(),
        "byte_count is too small for the next UTF-8 character"
    );
    let next = if end < content.len() {
        format!(
            "More content available: read_bytes path={name:?} start_byte={end} byte_count={count}"
        )
    } else {
        "End of file page.".into()
    };
    Ok(format!(
        "File {name}, UTF-8 bytes {start}..{end} of {} (end exclusive)\n{}\n{next}",
        content.len(),
        &content[start..end]
    ))
}

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
    let mut offset = 0;
    for (index, raw) in content.split_inclusive('\n').enumerate() {
        let line_offset = offset;
        offset += raw.len();
        if index < start - 1 {
            continue;
        }
        if index - (start - 1) >= count {
            break;
        }
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let rendered = format!("{}: {line}\n", index + 1);
        if rendered.len() > PAGE_BYTES && body.is_empty() {
            return bytes(name, content, line_offset, PAGE_BYTES);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_memory_survives_observation_eviction_and_resets_after_mutation() {
        let mut memory = ActionMemory::default();
        let mut transcript = vec!["seed1".into(), "seed2".into(), "seed3".into()];
        memory.attempted("read first");
        assert_eq!(memory.observed("first".into(), "unchanged".into()), 1);
        push(&mut transcript, "first observed body".into());
        for index in 0..20 {
            memory.attempted(&format!("read page {index}"));
            memory.observed(format!("page{index}"), format!("result{index}"));
            push(
                &mut transcript,
                format!("Page {index}: {}", "x".repeat(16000)),
            );
        }
        assert!(
            !transcript
                .iter()
                .any(|entry| entry == "first observed body")
        );
        assert_eq!(memory.observed("first".into(), "unchanged".into()), 2);
        assert!(memory.prompt().contains("21 total steps"));
        assert!(memory.prompt().len() < 26000);
        assert!(transcript.iter().map(String::len).sum::<usize>() <= CONTEXT_BYTES);
        assert_eq!(memory.observed("first".into(), "changed".into()), 1);
        memory.changed();
        assert_eq!(memory.observed("first".into(), "changed".into()), 1);
        for index in 0..100 {
            memory.observed(format!("key{index}"), "result".into());
        }
        assert_eq!(memory.observations.len(), 64);
        assert!(memory.journal.len() <= 12);
    }

    #[test]
    fn long_line_continuation_preserves_every_utf8_byte_with_bounded_pages() {
        let content = format!("{}tail", "界🦀".repeat(7000));
        let mut page = page("context.json", &content, 1, 1).unwrap();
        let mut observed = String::new();
        loop {
            assert!(page.len() <= PAGE_BYTES);
            let body = page.split_once('\n').unwrap().1;
            let (body, footer) = body.rsplit_once('\n').unwrap();
            observed.push_str(body);
            if footer == "End of file page." {
                break;
            }
            let next = footer
                .split("start_byte=")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            page = bytes("context.json", &content, next, PAGE_BYTES).unwrap();
        }
        assert_eq!(observed, content);
        assert!(bytes("x", "界", 1, 12).is_err());
        assert!(bytes("x", "界", 0, 1).is_err());
        assert!(bytes("x", "界", 4, 12).is_err());
        assert!(
            bytes("x", "界", 3, 12)
                .unwrap()
                .contains("End of file page.")
        );
    }

    #[test]
    fn long_line_after_crlf_reports_original_file_byte_offset() {
        let content = format!("first\r\n{}", "界".repeat(5000));
        assert!(
            page("x", &content, 2, 1)
                .unwrap()
                .contains("UTF-8 bytes 7..")
        );
    }
}
