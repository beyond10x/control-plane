//! Audit Cargo's resolved production dependency graph, including renamed/transitive edges.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

fn repository(package: &Value) -> Option<&str> {
    [package["source"].as_str(), package["repository"].as_str()]
        .into_iter()
        .flatten()
        .find_map(|url| {
            url.split_once("github.com/beyond10x/").map(|(_, tail)| {
                tail.split(['?', '#', '/'])
                    .next()
                    .unwrap_or(tail)
                    .trim_end_matches(".git")
            })
        })
}

fn admitted(package: &Value, inside_sdk: bool) -> bool {
    let name = package["name"].as_str().unwrap_or("");
    if name == "control-plane-xtask" {
        return false;
    }
    let Some(repo) = repository(package) else {
        // A local checkout of an ecosystem crate must supply provenance too.
        if package["source"].is_null() {
            return matches!(
                name,
                "controlplane"
                    | "control-plane-core"
                    | "control-plane-protocol"
                    | "control-plane-runtime"
                    | "control-plane-app"
            );
        }
        return !name.starts_with("b10x-") && !name.starts_with("ess-");
    };
    match repo {
        "control-plane" => name != "control-plane-xtask",
        "loom" => {
            if !inside_sdk {
                name == "b10x-loom-sdk"
            } else {
                matches!(
                    name,
                    "b10x-loom-sdk"
                        | "b10x-loom-commission"
                        | "b10x-loom-governor"
                        | "b10x-loom-executor"
                        | "b10x-loom-intake-router"
                        | "b10x-loom-intake-slice"
                        | "b10x-loom-intake-references"
                        | "loom"
                        | "commission"
                )
            }
        }
        "canon" => matches!(name, "b10x-canon" | "b10x-canon-core"),
        "engineering-protocols" => inside_sdk && name == "b10x-canon-engineering",
        "eventlog" => matches!(name, "eventlog-core" | "eventlog-sqlite"),
        "llm" => {
            name.starts_with("b10x-llm-") && !matches!(name, "b10x-llm-cli" | "b10x-llm-gateway")
        }
        "ess" => matches!(name, "ess-primitives" | "ess-runtime"),
        _ => false,
    }
}

pub fn check(metadata: &Value) -> Result<()> {
    let packages = metadata["packages"]
        .as_array()
        .context("Cargo packages")?
        .iter()
        .map(|p| Ok((p["id"].as_str().context("package ID")?, p)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .context("Cargo resolved graph")?
        .iter()
        .map(|n| Ok((n["id"].as_str().context("node ID")?, n)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let roots = metadata["workspace_members"]
        .as_array()
        .context("workspace members")?
        .iter()
        .map(|id| id.as_str().context("member ID"))
        .collect::<Result<Vec<_>>>()?;
    for root in roots {
        let root_package = packages.get(root).context("root package")?;
        if root_package["name"] == "control-plane-xtask" {
            continue;
        }
        // Embedded implementation crates are admitted only through the public SDK boundary,
        // not merely because a dependency happens to be more than one edge from the app.
        let mut pending = vec![(root, false)];
        let mut visited = BTreeSet::new();
        while let Some((id, inside_sdk)) = pending.pop() {
            if !visited.insert((id, inside_sdk)) {
                continue;
            }
            let node = nodes.get(id).context("resolved package node")?;
            for dep in node["deps"].as_array().context("resolved dependencies")? {
                let kinds = dep["dep_kinds"].as_array().context("dependency kinds")?;
                if kinds.iter().all(|k| k["kind"] == "dev") {
                    continue;
                }
                let next = dep["pkg"].as_str().context("dependency ID")?;
                let package = packages.get(next).context("dependency package")?;
                ensure!(
                    admitted(package, inside_sdk),
                    "non-foundation dependency {} reachable from {} via {}",
                    package["name"],
                    root_package["name"],
                    packages[id]["name"]
                );
                let enters_sdk =
                    repository(package) == Some("loom") && package["name"] == "b10x-loom-sdk";
                pending.push((next, inside_sdk || enters_sdk));
            }
        }
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<()> {
    sources(root)?;
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--locked"])
        .output()?;
    ensure!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    check(&serde_json::from_slice(&output.stdout)?)?;
    println!("production dependency graph uses admitted foundation libraries");
    loom_resolution(root)?;
    println!("Cargo.lock resolves each Loom crate once");
    loom_pins(root)?;
    println!("every Loom dependency is pinned to a release tag");
    Ok(())
}

/// The one host binding to Loom's loop and the provider port, relative to the repository root.
const LOOM_BINDING: &str = "crates/control-plane-runtime/src/loom_model.rs";
/// Foundation execution a production source never duplicates.
const DUPLICATED: [&str; 4] = [
    "impl Governor for",
    ".evaluate_effect(",
    "call_tool(",
    "for _ in 0..host.config.max_steps",
];
/// A provider call or a model loop of its own; only [`LOOM_BINDING`] may hold them.
const MODEL_LOOP: [&str; 4] = ["llm_core", "b10x_llm", ".turn(", "AgentLoop"];

/// Refuse foundation execution in production sources: every `.rs` file under `src/` of every
/// crate except this tooling crate. Outside [`LOOM_BINDING`] no source names the provider or
/// builds a loop ([`MODEL_LOOP`]); no source duplicates [`DUPLICATED`]; and the declaration of
/// `AgentModel::respond_in` has no default body, so no model binding answers a role outside its
/// Loom session. Line comments are not code and are not searched.
pub fn sources(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    for krate in std::fs::read_dir(root.join("crates")).context("crates directory")? {
        let krate = krate?.path();
        if krate
            .file_name()
            .is_some_and(|name| name == "control-plane-xtask")
        {
            continue;
        }
        let src = krate.join("src");
        if src.is_dir() {
            rust_files(&src, &mut files)?;
        }
    }
    files.sort();
    ensure!(
        !files.is_empty(),
        "no production sources under {}",
        root.display()
    );
    let mut declares_respond_in = false;
    for file in files {
        let shown = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let code = std::fs::read_to_string(&file)
            .with_context(|| format!("read {shown}"))?
            .lines()
            .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
            .collect::<Vec<_>>()
            .join("\n");
        for duplicated in DUPLICATED {
            ensure!(
                !code.contains(duplicated),
                "{shown} duplicates foundation execution: {duplicated}"
            );
        }
        if shown != LOOM_BINDING {
            for call in MODEL_LOOP {
                ensure!(
                    !code.contains(call),
                    "{shown} drives the model outside {LOOM_BINDING}: {call}"
                );
            }
        }
        if let Some((_, declaration)) = code.split_once("trait AgentModel") {
            let (_, signature) = declaration
                .split_once("fn respond_in")
                .with_context(|| format!("{shown}: AgentModel declares no respond_in"))?;
            let end = signature
                .find([';', '{'])
                .with_context(|| format!("{shown}: unterminated respond_in"))?;
            ensure!(
                signature[end..].starts_with(';'),
                "{shown}: AgentModel::respond_in has a default body, which answers without a Loom session"
            );
            declares_respond_in = true;
        }
    }
    ensure!(
        declares_respond_in,
        "no production source declares AgentModel::respond_in"
    );
    Ok(())
}

fn rust_files(dir: &Path, found: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    Ok(())
}

/// The repository every Loom crate comes from; a URL names it when its path ends here.
const LOOM_REPOSITORY: &str = "github.com/beyond10x/loom";
/// Cargo's dependency tables, each also read under every `target.<cfg>`.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Whether `url` (a manifest `git` or a lock `source`) names the Loom repository itself, not a
/// repository whose name merely starts with `loom`.
fn names_loom(url: &str) -> bool {
    url.split_once(LOOM_REPOSITORY).is_some_and(|(_, tail)| {
        let tail = tail.strip_prefix(".git").unwrap_or(tail);
        tail.is_empty() || tail.starts_with(['?', '#', '/'])
    })
}

/// A release tag: `<major>.<minor>.<patch>`, optionally `v`-prefixed.
fn release_tag(tag: &str) -> bool {
    let parts = tag
        .strip_prefix('v')
        .unwrap_or(tag)
        .split('.')
        .collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Every `Cargo.toml` under `dir`, skipping build output, dependencies and hidden directories.
fn manifests(dir: &Path, found: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if !(name.starts_with('.') || name == "target" || name == "node_modules") {
                manifests(&path, found)?;
            }
        } else if name == "Cargo.toml" {
            found.push(path);
        }
    }
    Ok(())
}

/// Why a Loom dependency declaration is not a release pin, or `None` when it is one (or when it
/// inherits the workspace's declaration, which is read where it is made).
fn unreleased(declaration: &toml::Value) -> Option<String> {
    let Some(table) = declaration.as_table() else {
        return Some("a registry version, not a release tag of the Loom repository".into());
    };
    if table.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
        return None;
    }
    if let Some(rev) = table.get("rev") {
        return Some(format!("rev {rev}"));
    }
    if let Some(branch) = table.get("branch") {
        return Some(format!("branch {branch}"));
    }
    match table.get("tag").and_then(toml::Value::as_str) {
        None => Some("no tag".into()),
        Some(tag) if !release_tag(tag) => Some(format!("tag {tag:?}, not <major>.<minor>.<patch>")),
        Some(_) => None,
    }
}

/// Refuse a Loom dependency that is not pinned to a release tag, in the workspace root and every
/// crate manifest under `root`. A dependency is Loom when its package (or, unrenamed, its key) is
/// `b10x-loom-*`, or when its `git` names the Loom repository. Every refusal names the manifest,
/// relative to `root`, and the crate.
pub fn loom_pins(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    manifests(root, &mut files)?;
    files.sort();
    ensure!(!files.is_empty(), "no Cargo.toml under {}", root.display());
    let mut refusals = Vec::new();
    for file in files {
        let shown = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let manifest: toml::Table = std::fs::read_to_string(&file)
            .with_context(|| format!("read {shown}"))?
            .parse()
            .with_context(|| format!("parse {shown}"))?;
        let mut tables = Vec::new();
        for name in DEPENDENCY_TABLES {
            tables.extend(manifest.get(name));
        }
        tables.extend(
            manifest
                .get("workspace")
                .and_then(|workspace| workspace.get("dependencies")),
        );
        for platform in manifest
            .get("target")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|targets| targets.values())
        {
            for name in DEPENDENCY_TABLES {
                tables.extend(platform.get(name));
            }
        }
        for (key, declaration) in tables
            .into_iter()
            .filter_map(toml::Value::as_table)
            .flat_map(|table| table.iter())
        {
            let package = declaration
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(key);
            let from_loom = declaration
                .get("git")
                .and_then(toml::Value::as_str)
                .is_some_and(names_loom);
            if !(package.starts_with("b10x-loom-") || from_loom) {
                continue;
            }
            if let Some(reason) = unreleased(declaration) {
                refusals.push(format!(
                    "{shown}: Loom crate {package} is not pinned to a release tag ({reason})"
                ));
            }
        }
    }
    ensure!(refusals.is_empty(), "{}", refusals.join("\n"));
    Ok(())
}

/// Refuse a `Cargo.lock` under `root` that resolves one Loom package (`b10x-loom-*`, or any
/// package sourced from the Loom repository) more than once, from two sources or at two versions.
/// Every refusal names the package and each copy's version and source.
pub fn loom_resolution(root: &Path) -> Result<()> {
    let lock: toml::Table = std::fs::read_to_string(root.join("Cargo.lock"))
        .context("read Cargo.lock")?
        .parse()
        .context("parse Cargo.lock")?;
    let mut copies = BTreeMap::<&str, Vec<String>>::new();
    for package in lock
        .get("package")
        .and_then(toml::Value::as_array)
        .context("Cargo.lock packages")?
    {
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .context("Cargo.lock package name")?;
        let source = package.get("source").and_then(toml::Value::as_str);
        if !(name.starts_with("b10x-loom-") || source.is_some_and(names_loom)) {
            continue;
        }
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .unwrap_or("?");
        copies.entry(name).or_default().push(format!(
            "{version} from {}",
            source.unwrap_or("a local path")
        ));
    }
    let refusals = copies
        .into_iter()
        .filter(|(_, found)| found.len() > 1)
        .map(|(name, found)| {
            format!(
                "Loom crate {name} resolves {} times: {}",
                found.len(),
                found.join("; ")
            )
        })
        .collect::<Vec<_>>();
    ensure!(refusals.is_empty(), "{}", refusals.join("\n"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn embedded_graph(through_sdk: bool) -> Value {
        json!({"workspace_members":["app"], "packages":[
            {"id":"app","name":"control-plane-app"},
            {"id":"runtime","name":"control-plane-runtime"},
            {"id":"entry","name":if through_sdk {"b10x-loom-sdk"} else {"unrelated-wrapper"},"source":if through_sdk {"git+https://github.com/beyond10x/loom?rev=pin"} else {"registry+https://github.com/rust-lang/crates.io-index"}},
            {"id":"internal","name":"b10x-loom-commission","source":"git+https://github.com/beyond10x/loom?rev=pin"},
            {"id":"protocol","name":"b10x-canon-engineering","source":"git+https://github.com/beyond10x/engineering-protocols?tag=0.1.0"},
            {"id":"model","name":"commission","source":"git+https://github.com/beyond10x/loom?rev=pin"}
        ],"resolve":{"nodes":[
            {"id":"app","deps":[{"pkg":"runtime","dep_kinds":[{"kind":null}]}]},
            {"id":"runtime","deps":[{"pkg":"entry","dep_kinds":[{"kind":null}]}]},
            {"id":"entry","deps":[{"pkg":"internal","dep_kinds":[{"kind":null}]}]},
            {"id":"internal","deps":[{"pkg":"protocol","dep_kinds":[{"kind":null}]},{"pkg":"model","dep_kinds":[{"kind":null}]}]},
            {"id":"protocol","deps":[]}, {"id":"model","deps":[]}
        ]}})
    }

    #[test]
    fn app_runtime_sdk_retains_embedded_protocol_and_generated_models() {
        check(&embedded_graph(true)).unwrap();
    }

    #[test]
    fn unrelated_transitive_wrapper_cannot_admit_sdk_private_crates() {
        let mut graph = embedded_graph(false);
        graph["resolve"]["nodes"][3]["deps"] = json!([]);
        assert!(check(&graph).is_err());
    }

    #[test]
    fn renamed_transitive_higher_layer_is_rejected_but_tooling_and_sdk_internals_are_allowed()
    -> Result<()> {
        let mut graph = json!({"workspace_members":["app","checks"], "packages":[
            {"id":"app","name":"control-plane-app","repository":"https://github.com/beyond10x/control-plane"},
            {"id":"checks","name":"control-plane-xtask","repository":"https://github.com/beyond10x/control-plane"},
            {"id":"sdk","name":"b10x-loom-sdk","source":"git+https://github.com/beyond10x/loom?rev=pin"},
            {"id":"nested","name":"b10x-loom-commission","source":"git+https://github.com/beyond10x/loom?rev=pin"},
            {"id":"compiler","name":"ess-conformance","source":"git+https://github.com/beyond10x/ess?rev=pin"}
        ], "resolve":{"nodes":[
            {"id":"app","deps":[{"name":"renamed","pkg":"sdk","dep_kinds":[{"kind":null}]}]},
            {"id":"checks","deps":[{"pkg":"compiler","dep_kinds":[{"kind":null}]}]},
            {"id":"sdk","deps":[{"pkg":"nested","dep_kinds":[{"kind":null}]}]},
            {"id":"nested","deps":[]}, {"id":"compiler","deps":[]}
        ]}});
        check(&graph)?;
        graph["packages"][3]["name"] = json!("metaharness-core");
        graph["packages"][3]["source"] =
            json!("git+https://github.com/beyond10x/metaharness?rev=pin");
        assert!(check(&graph).is_err());
        graph["resolve"]["nodes"][2]["deps"][0]["dep_kinds"][0]["kind"] = json!("dev");
        check(&graph)?;
        graph["resolve"]["nodes"][0]["deps"][0]["pkg"] = json!("compiler");
        assert!(check(&graph).is_err());
        Ok(())
    }
}
