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

const LOOM: &str = "https://github.com/beyond10x/loom";
const RUNTIME_MANIFEST: &str = "crates/control-plane-runtime/Cargo.toml";

/// A disposable workspace: a root manifest (`workspace`) and a runtime crate manifest whose
/// dependency table is `runtime`. The guard deletes it when the case ends.
fn workspace(root_dependencies: &str, runtime: &str) -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    for (path, contents) in [
        (
            "Cargo.toml".to_string(),
            format!(
                "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.dependencies]\nanyhow = \"1\"\n{root_dependencies}"
            ),
        ),
        (
            RUNTIME_MANIFEST.to_string(),
            format!(
                "[package]\nname = \"control-plane-runtime\"\nversion = \"0.1.0\"\n\n{runtime}"
            ),
        ),
    ] {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
    }
    Ok(dir)
}

fn pin_refusal(root_dependencies: &str, runtime: &str) -> String {
    let dir = workspace(root_dependencies, runtime).unwrap();
    match foundation::loom_pins(dir.path()) {
        Ok(()) => panic!("Loom pin check passed {root_dependencies:?} / {runtime:?}"),
        Err(error) => format!("{error:#}"),
    }
}

#[test]
fn loom_pin_must_be_a_release_tag() -> Result<()> {
    // A release tag passes, renamed or not, plain or `v`-prefixed, inline or as its own table.
    for runtime in [
        format!(
            "[dependencies]\nloom-sdk = {{ package = \"b10x-loom-sdk\", git = \"{LOOM}\", tag = \"0.20.0\" }}\n"
        ),
        format!("[dependencies]\nb10x-loom-sdk = {{ git = \"{LOOM}\", tag = \"v0.20.0\" }}\n"),
        format!(
            "[dependencies.loom-sdk]\npackage = \"b10x-loom-sdk\"\ngit = \"{LOOM}.git\"\ntag = \"0.20.0\"\n"
        ),
        "[dependencies]\nloom-sdk = { workspace = true }\n".to_string(),
    ] {
        let root = format!(
            "loom-sdk = {{ package = \"b10x-loom-sdk\", git = \"{LOOM}\", tag = \"0.20.0\" }}\n"
        );
        foundation::loom_pins(workspace(&root, &runtime)?.path())?;
    }
    // Every crate manifest is read: rev, branch and a missing tag are refused by manifest and crate.
    for (dependency, crate_name) in [
        (
            format!(
                "loom-sdk = {{ package = \"b10x-loom-sdk\", git = \"{LOOM}\", rev = \"1b25fe8939e1883f3e47c1292ac90cf75fc3f3a5\" }}"
            ),
            "b10x-loom-sdk",
        ),
        (
            format!("b10x-loom-governor = {{ git = \"{LOOM}\", branch = \"main\" }}"),
            "b10x-loom-governor",
        ),
        (
            format!("b10x-loom-executor = {{ git = \"{LOOM}\" }}"),
            "b10x-loom-executor",
        ),
        (
            "b10x-loom-commission = \"0.1\"".to_string(),
            "b10x-loom-commission",
        ),
        (
            "b10x-loom-intake-slice = { path = \"../../../loom/crates/intake-slice\" }".to_string(),
            "b10x-loom-intake-slice",
        ),
        // A key that hides the package name is still Loom by its repository.
        (
            format!("model = {{ git = \"{LOOM}.git\", rev = \"abc\" }}"),
            "model",
        ),
        // A tag that is not a release is no release.
        (
            format!(
                "loom-sdk = {{ package = \"b10x-loom-sdk\", git = \"{LOOM}\", tag = \"nightly\" }}"
            ),
            "b10x-loom-sdk",
        ),
    ] {
        for table in [
            "dependencies",
            "dev-dependencies",
            "target.'cfg(unix)'.dependencies",
        ] {
            let error = pin_refusal("", &format!("[{table}]\n{dependency}\n"));
            assert!(
                error.contains(RUNTIME_MANIFEST) && error.contains(crate_name),
                "{table} {dependency}: {error}"
            );
        }
    }
    // The workspace root's own table is read too.
    let error = pin_refusal(
        &format!("loom-sdk = {{ package = \"b10x-loom-sdk\", git = \"{LOOM}\", rev = \"abc\" }}\n"),
        "",
    );
    assert!(
        error.contains("Cargo.toml")
            && !error.contains(RUNTIME_MANIFEST)
            && error.contains("b10x-loom-sdk"),
        "{error}"
    );
    // Another repository whose name merely starts with loom is not Loom.
    foundation::loom_pins(
        workspace(
            "",
            "[dependencies]\nloomish = { git = \"https://github.com/beyond10x/loom-extras\", rev = \"abc\" }\n",
        )?
        .path(),
    )?;
    Ok(())
}

fn lock(packages: &[(&str, &str, Option<&str>)]) -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let mut contents = String::from("version = 4\n");
    for (name, version, source) in packages {
        contents.push_str(&format!(
            "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n"
        ));
        if let Some(source) = source {
            contents.push_str(&format!("source = \"{source}\"\n"));
        }
    }
    fs::write(dir.path().join("Cargo.lock"), contents)?;
    Ok(dir)
}

#[test]
fn loom_crates_resolve_once() -> Result<()> {
    let tagged = "git+https://github.com/beyond10x/loom?tag=0.20.0#aaaa";
    let pinned = "git+https://github.com/beyond10x/loom?rev=1b25fe8#1b25fe8";
    let crates_io = "registry+https://github.com/rust-lang/crates.io-index";
    // One copy of each Loom package passes; a duplicate non-Loom crate is not this check's.
    foundation::loom_resolution(
        lock(&[
            ("b10x-loom-sdk", "0.20.0", Some(tagged)),
            ("b10x-loom-governor", "0.20.0", Some(tagged)),
            ("windows-sys", "0.52.0", Some(crates_io)),
            ("windows-sys", "0.59.0", Some(crates_io)),
            ("control-plane-runtime", "0.1.0", None),
        ])?
        .path(),
    )?;
    // Two sources of one package are refused, naming the package and both sources.
    for packages in [
        [
            ("b10x-loom-sdk", "0.20.0", Some(tagged)),
            ("b10x-loom-sdk", "0.1.0", Some(pinned)),
        ],
        // Two versions from one source are two copies as well.
        [
            ("b10x-loom-governor", "0.20.0", Some(tagged)),
            ("b10x-loom-governor", "0.19.0", Some(tagged)),
        ],
    ] {
        let dir = lock(&packages)?;
        let error = match foundation::loom_resolution(dir.path()) {
            Ok(()) => panic!("Loom resolution check passed {packages:?}"),
            Err(error) => format!("{error:#}"),
        };
        let (name, first, _) = packages[0];
        let (_, second, _) = packages[1];
        assert!(
            error.contains(name)
                && packages
                    .iter()
                    .all(|(_, _, source)| error.contains(source.unwrap()))
                && error.contains(first)
                && error.contains(second),
            "{error}"
        );
    }
    // A local copy beside a git copy is two copies too.
    let dir = lock(&[
        ("b10x-loom-sdk", "0.20.0", Some(tagged)),
        ("b10x-loom-sdk", "0.20.0", None),
    ])?;
    let error = format!("{:#}", foundation::loom_resolution(dir.path()).unwrap_err());
    assert!(
        error.contains("b10x-loom-sdk") && error.contains(tagged),
        "{error}"
    );
    // A tree without a lock file is refused rather than passed.
    assert!(foundation::loom_resolution(tempfile::tempdir()?.path()).is_err());
    // The repository's own lock file resolves each Loom package once.
    foundation::loom_resolution(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    Ok(())
}
