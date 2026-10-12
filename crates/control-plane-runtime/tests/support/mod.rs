//! What every runtime test binary checks of its fixtures: a fixture lives in one root under the
//! tree's `.scratch`, its managed worktree records with it, and the root goes when the test ends.

use std::path::{Path, PathBuf};

/// The fixture root: the common parent of the `XDG_STATE_HOME` and `XDG_CONFIG_HOME` the fixture
/// hands every `worktree` and `aep` command, so the worktree registry, its linked trees and their
/// records all lie inside it. Refuses a root outside `.scratch` or state kept elsewhere.
pub fn fixture_root(environment: &[(String, String)]) -> PathBuf {
    let directory = |name: &str| {
        PathBuf::from(
            &environment
                .iter()
                .find(|(key, _)| key == name)
                .unwrap_or_else(|| panic!("the fixture sets no {name}"))
                .1,
        )
    };
    let state = directory("XDG_STATE_HOME");
    let config = directory("XDG_CONFIG_HOME");
    let root = state.parent().expect("state has a parent").to_path_buf();
    assert_eq!(
        config.parent(),
        Some(root.as_path()),
        "worktree configuration is outside the fixture root"
    );
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch");
    assert!(root.starts_with(&scratch), "{}", root.display());
    // `worktree activate` wrote its profile here; the registry follows into `state` with the
    // first managed worktree.
    assert!(
        config.join("worktree").is_dir(),
        "the fixture's worktree configuration is not in {}",
        config.display()
    );
    root
}

/// Asserts the fixture root is gone once everything that held it has been dropped.
pub fn assert_removed(root: &Path) {
    assert!(!root.exists(), "the fixture left {} behind", root.display());
}
