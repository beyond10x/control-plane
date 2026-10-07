//! Syntax refusal belongs to the host adapter. Loom owns subsequent turns;
//! confinement and operational failures remain fatal and are never reclassified.
use std::{
    fmt,
    path::{Component, Path},
};

pub const PATH_HELP: &str = "Read paths are worktree-relative, for example AGENTS.md, TASK.md or go.mod. Registered read-only context uses workspace:<directory_id>/relative-file. Never use an absolute primary-repository path, parent traversal or a filesystem path copied from directory metadata. Writes remain worktree-relative and within accepted scope.";

#[derive(Debug)]
pub struct ReadPathSyntax(&'static str);
impl fmt::Display for ReadPathSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ReadPathSyntax {}

pub fn parse(name: &str) -> Result<(Option<&str>, &str), ReadPathSyntax> {
    if name.len() > 4096 || name.contains('\0') {
        return Err(ReadPathSyntax("read path exceeds syntax limits"));
    }
    let (directory, relative) = if let Some(reference) = name.strip_prefix("workspace:") {
        let (id, path) = reference.split_once('/').ok_or(ReadPathSyntax(
            "context syntax requires workspace:<directory_id>/relative-file",
        ))?;
        if id.is_empty() {
            return Err(ReadPathSyntax("workspace directory id must not be empty"));
        }
        (Some(id), path)
    } else {
        (None, name)
    };
    if relative.is_empty()
        || !Path::new(relative)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(ReadPathSyntax(
            "read path must be normalized and worktree-relative; absolute and parent paths are refused",
        ));
    }
    Ok((directory, relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_syntax_is_typed_and_feedback_is_bounded() {
        for path in [
            "",
            "/tmp/private",
            "../private",
            "workspace:dir/../private",
            "workspace:",
            "workspace:/private",
            "workspace:dir//private",
            "bad\0path",
        ] {
            let refusal =
                crate::refusal::refusal_of(&anyhow::Error::new(parse(path).unwrap_err())).unwrap();
            assert_eq!(refusal.code, "read_path_syntax");
            assert!(refusal.observation().len() < 2048);
        }
        assert_eq!(parse("Cargo.toml").unwrap(), (None, "Cargo.toml"));
        assert_eq!(
            parse("workspace:directory/README.md").unwrap(),
            (Some("directory"), "README.md")
        );
    }

    #[test]
    fn confinement_authority_and_root_failures_are_not_syntax_feedback() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret"), "external secret").unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret"), root.path().join("link"))
            .unwrap();
        let goal = serde_json::json!({"directories":[]});
        for path in ["link", ".git/config"] {
            let error = crate::engine::context_path(root.path(), path, &goal, true).unwrap_err();
            assert!(
                crate::refusal::refusal_of(&error).is_none(),
                "security failure became recoverable: {path}"
            );
        }
        // An unregistered directory id reads nothing; the model may correct the id.
        let unregistered =
            crate::engine::context_path(root.path(), "workspace:unregistered/secret", &goal, true)
                .unwrap_err();
        assert_eq!(
            crate::refusal::refusal_of(&unregistered).map(|r| r.code),
            Some("context_directory_unknown")
        );
        let error =
            crate::engine::context_path(&root.path().join("missing-root"), "secret", &goal, true)
                .unwrap_err();
        assert!(
            crate::refusal::refusal_of(&error).is_none(),
            "unavailable root became recoverable"
        );
    }
}
