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
    for name in ["engine.rs", "fleet.rs"] {
        let source =
            std::fs::read_to_string(root.join("crates/control-plane-runtime/src").join(name))?;
        for duplicated in [
            "impl Governor for",
            ".evaluate_effect(",
            "call_tool(",
            "for _ in 0..host.config.max_steps",
        ] {
            ensure!(
                !source.contains(duplicated),
                "{name} duplicates foundation execution: {duplicated}"
            );
        }
    }
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
