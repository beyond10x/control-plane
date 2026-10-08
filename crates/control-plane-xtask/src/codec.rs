//! Lossless Integer transport; no binary64 conversion occurs at this boundary.
use anyhow::{Context, Result, bail};
use ess_primitives::{facts::Number, node::Node};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn to_json(node: &Node) -> Result<Value> {
    Ok(match node {
        Node::Null => Value::Null,
        Node::Bool(v) => Value::Bool(*v),
        Node::Text(v) => Value::String(v.clone()),
        Node::Number(v) => Value::from(v.as_i64().context("contract requires exact i64")?),
        Node::Seq(v) => Value::Array(v.iter().map(to_json).collect::<Result<_>>()?),
        Node::Map(v) => Value::Object(
            v.iter()
                .map(|(k, v)| Ok((k.clone(), to_json(v)?)))
                .collect::<Result<_>>()?,
        ),
    })
}

pub fn from_json(value: &Value) -> Result<Node> {
    Ok(match value {
        Value::Null => Node::Null,
        Value::Bool(v) => Node::Bool(*v),
        Value::String(v) => Node::Text(v.clone()),
        Value::Number(v) => Node::Number(Number::from(
            v.as_i64().context("contract returned a non-i64 number")?,
        )),
        Value::Array(v) => Node::Seq(v.iter().map(from_json).collect::<Result<_>>()?),
        Value::Object(v) => Node::Map(
            v.iter()
                .map(|(k, v)| Ok((k.clone(), from_json(v)?)))
                .collect::<Result<_>>()?,
        ),
    })
}

pub fn fields(value: &Value) -> Result<BTreeMap<String, Node>> {
    match from_json(value)? {
        Node::Map(v) => Ok(v),
        _ => bail!("expected object"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_codec_preserves_extrema_and_values_beyond_binary64() -> Result<()> {
        for n in [
            i64::MIN,
            -9_007_199_254_740_993,
            0,
            9_007_199_254_740_993,
            i64::MAX,
        ] {
            let node = Node::Number(Number::from(n));
            let bytes = serde_json::to_vec(&to_json(&node)?)?;
            assert_eq!(from_json(&serde_json::from_slice(&bytes)?)?, node);
        }
        assert!(from_json(&serde_json::json!(u64::MAX)).is_err());
        assert!(from_json(&serde_json::json!(1.5)).is_err());
        Ok(())
    }
    #[test]
    fn actual_dispatch_event_and_replayed_view_preserve_i64() -> Result<()> {
        use control_plane_core::{Actor, contract::ContractStore};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let scratch = root.join(".scratch/integer-codec");
        std::fs::create_dir_all(&scratch)?;
        let temp = tempfile::tempdir_in(scratch)?;
        let path = temp.path().join("host.sqlite");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(async {
            let mut store = ContractStore::open(&path).await?;
            let workspace = store.execute("RegisterWorkspace", serde_json::json!({"name":"integer test","path":"fixture"}), Actor::Operator).await?;
            let input = serde_json::json!({
                "workspace_id": workspace["published"][0]["payload"]["workspace_id"],
                "objective":"objective", "acceptance":"acceptance", "planner_model":"p",
                "implementor_model":"i", "reviewer_model":"r", "merge_authority":false,
                "max_workers":i64::MAX, "max_attempts":i64::MAX - 1, "max_minutes":9_007_199_254_740_993_i64
            });
            let answer = store.execute("CreateGoal", to_json(&from_json(&input)?)?, Actor::Operator).await?;
            let event = fields(&answer["published"][0]["payload"])?;
            drop(store);
            let rows = ContractStore::open(path).await?.query("GoalList")?;
            let view = fields(&rows[0])?;
            for key in ["max_workers", "max_attempts", "max_minutes"] {
                let expected = from_json(&input[key])?;
                assert_eq!(event[key], expected);
                assert_eq!(view[key], expected);
            }
            Ok(())
        })
    }
}
