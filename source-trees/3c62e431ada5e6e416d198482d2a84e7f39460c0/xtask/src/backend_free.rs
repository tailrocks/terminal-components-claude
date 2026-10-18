//! Standalone consumers prove feature isolation and absence of backend dependencies.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use serde_json::Value;

pub(crate) fn check(root: &Path) -> Result<(), String> {
    for (name, required) in [
        ("core", vec!["junie-tui"]),
        ("testing", vec!["junie-tui", "junie-tui-testing"]),
    ] {
        let manifest = root.join(format!("xtask/fixtures/backend-free-{name}/Cargo.toml"));
        let output = Command::new("cargo")
            .args([
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--no-default-features",
                "--manifest-path",
            ])
            .arg(&manifest)
            .output()
            .map_err(|e| format!("{name} metadata: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "{name} metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        assert_backend_free(&metadata, &required)?;
        // Each fixture is its own workspace. Never compile inside the real workspace's
        // feature union; never share its active Cargo target lock either.
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("target"))
            .join(format!("backend-free-{name}"));
        let status = Command::new("cargo")
            .args([
                "run",
                "--locked",
                "--no-default-features",
                "--manifest-path",
            ])
            .arg(&manifest)
            .arg("--target-dir")
            .arg(target)
            .status()
            .map_err(|e| format!("{name} consumer: {e}"))?;
        if !status.success() {
            return Err(format!("backend-free {name} consumer failed: {status}"));
        }
    }
    Ok(())
}

fn assert_backend_free(metadata: &Value, required: &[&str]) -> Result<(), String> {
    let packages = metadata["packages"].as_array().ok_or("packages missing")?;
    let names: BTreeMap<&str, &str> = packages
        .iter()
        .map(|p| {
            Ok((
                p["id"].as_str().ok_or("package id missing")?,
                p["name"].as_str().ok_or("package name missing")?,
            ))
        })
        .collect::<Result<_, String>>()?;
    let resolve = &metadata["resolve"];
    let consumer = resolve["root"]
        .as_str()
        .ok_or("isolated consumer root missing")?;
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or("workspace members missing")?;
    if members.len() != 1 || members.first().and_then(Value::as_str) != Some(consumer) {
        return Err("consumer is not an isolated one-member workspace".to_owned());
    }
    let nodes: BTreeMap<&str, &Value> = resolve["nodes"]
        .as_array()
        .ok_or("resolved nodes missing")?
        .iter()
        .map(|n| Ok((n["id"].as_str().ok_or("node id missing")?, n)))
        .collect::<Result<_, String>>()?;
    let mut pending = vec![consumer];
    let mut visited = BTreeSet::new();
    let mut reached = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let name = *names.get(id).ok_or("resolved package absent")?;
        if matches!(name, "crossterm" | "termion" | "termwiz" | "ratatui")
            || (name.starts_with("ratatui-") && name != "ratatui-core")
        {
            return Err(format!(
                "backend dependency reachable from isolated consumer: {name}"
            ));
        }
        reached.insert(name);
        let node = nodes.get(id).ok_or("resolved node absent")?;
        for dep in node["deps"]
            .as_array()
            .ok_or("resolved dependencies missing")?
        {
            pending.push(
                dep["pkg"]
                    .as_str()
                    .ok_or("resolved dependency id missing")?,
            );
        }
    }
    for name in required {
        if !reached.contains(name) {
            return Err(format!("consumer does not reach {name}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn graph(backend: bool) -> Value {
        let tail = if backend { "crossterm" } else { "ratatui-core" };
        json!({
            "workspace_members": ["consumer"],
            "packages": [
                {"id":"consumer", "name":"consumer"},
                {"id":"core", "name":"junie-tui"},
                {"id":"testing", "name":"junie-tui-testing"},
                {"id":"tail", "name":tail}
            ],
            "resolve": {"root":"consumer", "nodes":[
                {"id":"consumer", "deps":[{"pkg":"testing"}]},
                {"id":"testing", "deps":[{"pkg":"core"}]},
                {"id":"core", "deps":[{"pkg":"tail"}]},
                {"id":"tail", "deps":[]}
            ]}
        })
    }

    #[test]
    fn transitive_backend_mutation_is_rejected() {
        let required = ["junie-tui", "junie-tui-testing"];
        assert!(assert_backend_free(&graph(false), &required).is_ok());
        assert!(
            assert_backend_free(&graph(true), &required).is_err_and(|e| e.contains("crossterm"))
        );
    }

    #[test]
    fn missing_subject_or_workspace_union_cannot_pass() {
        assert!(assert_backend_free(&graph(false), &["missing"]).is_err());
        let mut unified = graph(false);
        unified["workspace_members"] = json!(["consumer", "core"]);
        assert!(assert_backend_free(&unified, &["junie-tui"]).is_err());
        assert!(assert_backend_free(&json!({}), &["junie-tui"]).is_err());
    }
}
