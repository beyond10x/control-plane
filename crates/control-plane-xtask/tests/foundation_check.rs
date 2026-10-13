//! `foundation-check`'s source rules: Loom owns the model loop. Outside the one host binding
//! (`crates/control-plane-runtime/src/loom_model.rs`) no production source calls a provider or
//! builds an `AgentLoop`, and `AgentModel::respond_in` has no default body that could answer a role
//! without a Loom session. The gate module is compiled in by path so each case drives it over a
//! disposable tree.
#[allow(dead_code)]
#[path = "../src/foundation.rs"]
mod foundation;

use anyhow::Result;
use std::{fs, path::Path};

const TRAIT: &str = "pub trait AgentModel: Send + Sync {\n    fn respond_in(&self, request: &ModelRequest, environment: &ModelEnvironment) -> Result<Value>;\n}\n";
const DEFAULT_BODY: &str = "pub trait AgentModel: Send + Sync {\n    fn respond_in(&self, request: &ModelRequest, environment: &ModelEnvironment) -> Result<Value> {\n        let _ = environment;\n        self.respond(request)\n    }\n}\n";
const BINDING: &str = "use loom_sdk::loom::harness::turn_loop::AgentLoop;\nfn run(provider: &dyn llm_core::Model) { let _ = provider.turn(request, sink, cancel); AgentLoop::new(port, tools, approvals, config); }\n";

/// A disposable tree holding a clean runtime crate plus `files` (path, contents), which replace
/// a clean file of the same path. The guard deletes it when the case ends.
fn tree(files: &[(&str, &str)]) -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let clean = [
        ("crates/control-plane-runtime/src/lib.rs", TRAIT),
        ("crates/control-plane-runtime/src/loom_model.rs", BINDING),
        (
            "crates/control-plane-runtime/src/engine.rs",
            "fn plan() { let _ = model.respond_in(&request, &environment); }\n",
        ),
        (
            "crates/control-plane-app/src/cli.rs",
            "fn bind() { CodexAgentModel::new(sessions); }\n",
        ),
        (
            "crates/control-plane-xtask/src/foundation.rs",
            "const FORBIDDEN: &[&str] = &[\"llm_core\", \".turn(\", \"AgentLoop\"];\n",
        ),
    ];
    for (path, contents) in clean.iter().chain(files) {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
    }
    Ok(dir)
}

fn refusal(files: &[(&str, &str)]) -> String {
    let dir = tree(files).unwrap();
    match foundation::sources(dir.path()) {
        Ok(()) => panic!("foundation check passed a tree with {files:?}"),
        Err(error) => format!("{error:#}"),
    }
}

#[test]
fn foundation_check_rejects_direct_model_loop() -> Result<()> {
    foundation::sources(tree(&[])?.path())?;
    let engine = "crates/control-plane-runtime/src/engine.rs";
    for (contents, named) in [
        (
            "fn plan() { let model = b10x_llm_tool_call::codex_model(name); }\nfn ask(m: &dyn llm_core::Model) {}\n",
            "llm_core",
        ),
        (
            "fn plan() { let outcome = provider.turn(&request, &mut sink, &cancel); }\n",
            ".turn(",
        ),
        (
            "fn plan() { AgentLoop::new(&mut port, &mut tools, &mut approvals, config).run(); }\n",
            "AgentLoop",
        ),
    ] {
        let error = refusal(&[(engine, contents)]);
        assert!(
            error.contains("engine.rs") && error.contains(named),
            "{named}: {error}"
        );
    }
    let error = refusal(&[("crates/control-plane-runtime/src/lib.rs", DEFAULT_BODY)]);
    assert!(error.contains("respond_in"), "{error}");
    // A comment naming the provider is not a call.
    foundation::sources(
        tree(&[(
            engine,
            "// Loom calls llm_core through loom_model.rs, never here.\n",
        )])?
        .path(),
    )?;
    Ok(())
}

#[test]
fn foundation_check_scans_every_runtime_source_file() -> Result<()> {
    for path in [
        "crates/control-plane-runtime/src/review_loop.rs",
        "crates/control-plane-runtime/src/roles/reviewer.rs",
        "crates/control-plane-app/src/live_model.rs",
        "crates/control-plane-core/src/memory.rs",
    ] {
        let error = refusal(&[(
            path,
            "fn review() { let outcome = provider.turn(&request, &mut sink, &cancel); }\n",
        )]);
        assert!(error.contains(path), "{path}: {error}");
    }
    // The repository's own tree passes.
    foundation::sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    Ok(())
}
