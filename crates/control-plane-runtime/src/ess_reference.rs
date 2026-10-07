//! Bounded transport of ESS-owned authoring syntax, never a second validator.
use crate::process::ProcessRunner;
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

const VERSION: &str = "0.53.0";
const SOURCE: &str = "https://github.com/beyond10x/ess/blob/a81a8729dc252830d4e0557176522b59be6ff253/schemas/generated/ess.schema.json";
const SCHEMA: &str = include_str!("../resources/ess-0.53.0/ess.schema.json");
const LIMIT: usize = 12 * 1024;

pub fn lookup(root: &Path, runner: &ProcessRunner, pointer: &str) -> Result<String> {
    let toolchain = runner.command(root, "ess", &["specify", "toolchain", "which"])?;
    ensure!(
        toolchain.lines().next() == Some(&format!("ess {VERSION}")),
        "ESS authoring reference is for {VERSION}; selected toolchain differs: {toolchain}"
    );
    project(pointer)
}

fn project(pointer: &str) -> Result<String> {
    ensure!(
        pointer.len() <= 1024 && (pointer.is_empty() || pointer.starts_with('/')),
        "ess_schema pointer must be an RFC6901 JSON pointer, at most 1024 bytes"
    );
    let schema: Value = serde_json::from_str(SCHEMA)?;
    let value = schema.pointer(pointer);
    let mut answer = json!({"ess_version":VERSION,"source":SOURCE,"pointer":pointer,
        "note":"Authoritative generated syntax only; ESS validation remains authoritative for semantics. Follow $ref through another ess_schema lookup."});
    if pointer.is_empty() {
        answer["properties"] = json!(
            schema["properties"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>()
        );
        answer["definitions"] = json!(
            schema["definitions"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>()
        );
    } else if let Some(value) = value {
        answer["schema"] = value.clone();
        if answer.to_string().len() > LIMIT {
            answer.as_object_mut().unwrap().remove("schema");
            answer["message"] = json!(
                "Selected node exceeds the response budget; request one of its child pointers."
            );
            answer["children"] = json!(
                value
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default()
            );
        }
    } else {
        answer["message"] = json!(
            "No such schema pointer. Use an empty pointer for the authoritative definition/property index."
        );
    }
    let output = answer.to_string();
    ensure!(
        output.len() <= LIMIT,
        "ESS schema index exceeds response budget"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    #[test]
    fn exact_foundation_schema_is_bounded_and_keeps_authoritative_references() {
        // Digest of the bytes at the source commit named above, not a local projection.
        assert_eq!(
            format!("{:x}", Sha256::digest(SCHEMA.as_bytes())),
            "06b5de6466154d4f891a32b6f4e4b981b07b84d961a68ab02f06554ef8a934e5"
        );
        let root: Value = serde_json::from_str(&project("").unwrap()).unwrap();
        assert!(
            root["definitions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d == "RawEntitySpec")
        );
        let entity: Value =
            serde_json::from_str(&project("/definitions/RawEntitySpec").unwrap()).unwrap();
        assert!(
            entity["schema"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "identity")
        );
        for pointer in [
            "",
            "/definitions",
            "/definitions/RawEntitySpec",
            "/not-found",
        ] {
            assert!(project(pointer).unwrap().len() <= LIMIT);
        }
        assert!(project("../private").is_err());
    }
}
