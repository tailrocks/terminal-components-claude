//! Versioned capture/build joins. Plans are obligations, never successful captures.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Evidence maps must not silently discard an earlier value for the same key.
pub(crate) fn unique_map<'de, D, V>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Visitor<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<V> {
        type Value = std::collections::BTreeMap<String, V>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object with unique keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut access: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = std::collections::BTreeMap::new();
            while let Some((key, value)) = access.next_entry::<String, V>()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate evidence map key"));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Visitor(std::marker::PhantomData))
}

fn unique_profile<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<serde_json::Value, D::Error> {
    let values = unique_map::<D, serde_json::Value>(deserializer)?;
    if values
        .values()
        .any(|value| value.is_object() || value.is_array())
    {
        return Err(serde::de::Error::custom(
            "compiler profile values must be scalar",
        ));
    }
    Ok(serde_json::Value::Object(values.into_iter().collect()))
}

/// Also check nested objects carried as untyped JSON (receipts/provenance).
pub(crate) fn strict_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    enum UniqueJson {
        Null(()),
        Bool(bool),
        I64(i64),
        U64(u64),
        F64(f64),
        String(String),
        Array(Vec<UniqueJson>),
        Object(
            #[serde(deserialize_with = "unique_map")]
            std::collections::BTreeMap<String, UniqueJson>,
        ),
    }
    // Typed parsing preserves its precise diagnostics. The second pass prevents
    // Value/Map fields from hiding duplicate keys at any depth.
    let result = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let _: UniqueJson = serde_json::from_slice(bytes)
        .map_err(|e| format!("non-unique or invalid JSON structure: {e}"))?;
    Ok(result)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) revision: String,
    pub(crate) fingerprint: String,
    pub(crate) lock_sha256: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Driver {
    Cli,
    ProductionView,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "mode", deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) enum Clock {
    NotApplicable,
    Paused { frame: u64 },
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    pub(crate) namespace: String,
    pub(crate) recipe_id: String,
    pub(crate) app: String,
    pub(crate) driver: Driver,
    pub(crate) scenario: Option<String>,
    pub(crate) state: String,
    pub(crate) viewport: (u16, u16),
    pub(crate) theme: String,
    pub(crate) color: String,
    pub(crate) clock: Clock,
    pub(crate) argv: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    pub(crate) schema: u8,
    pub(crate) inventory_sha256: String,
    pub(crate) cases: Vec<Case>,
    pub(crate) blockers: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Artifact {
    pub(crate) app: String,
    pub(crate) package_id: String,
    pub(crate) target: String,
    pub(crate) executable: String,
    pub(crate) sha256: String,
    pub(crate) features: Vec<String>,
    #[serde(deserialize_with = "unique_profile")]
    pub(crate) profile: serde_json::Value,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildManifest {
    pub(crate) schema: u8,
    pub(crate) source: Source,
    pub(crate) cargo: String,
    pub(crate) rustc: String,
    pub(crate) host_target: String,
    pub(crate) environment_sha256: String,
    #[serde(deserialize_with = "unique_map")]
    pub(crate) compiler_inputs: std::collections::BTreeMap<String, String>,
    pub(crate) confinement: crate::capture_confinement::Confinement,
    pub(crate) argv: Vec<String>,
    pub(crate) status: i32,
    pub(crate) messages_path: String,
    pub(crate) messages_sha256: String,
    pub(crate) stderr_path: String,
    pub(crate) stderr_sha256: String,
    pub(crate) artifacts: Vec<Artifact>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileHash {
    pub(crate) path: String,
    pub(crate) bytes: u64,
    pub(crate) sha256: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    pub(crate) name: String,
    pub(crate) status: i32,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureJoin {
    pub(crate) schema: u8,
    pub(crate) case: Case,
    pub(crate) case_sha256: String,
    pub(crate) plan_sha256: String,
    pub(crate) build_manifest_sha256: String,
    pub(crate) run_id: String,
    pub(crate) source_binary_sha256: String,
    pub(crate) staged_binary_sha256: String,
    pub(crate) stages: Vec<Stage>,
    #[serde(deserialize_with = "unique_map")]
    pub(crate) artifacts: std::collections::BTreeMap<String, FileHash>,
    pub(crate) provenance_sha256: String,
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn encoded_hash<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| hash(&bytes))
        .map_err(|e| e.to_string())
}
fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("capture contract: {message}"))
    }
}
fn sha(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}
impl Plan {
    pub(crate) fn validate(&self) -> Result<(), String> {
        require(
            self.schema == 1 && sha(&self.inventory_sha256),
            "invalid plan schema/inventory hash",
        )?;
        require(!self.cases.is_empty(), "empty capture plan")?;
        let mut identities = BTreeSet::new();
        for case in &self.cases {
            require(
                identities.insert((&case.namespace, &case.recipe_id)),
                "duplicate capture identity",
            )?;
            require(case.viewport.0 > 0 && case.viewport.1 > 0, "empty viewport")?;
            require(
                ["showcase", "tablepro", "jackin-preview", "holla"].contains(&case.app.as_str()),
                "unknown app",
            )?;
            require(
                ["truecolor", "256", "16", "mono"].contains(&case.color.as_str()),
                "unsupported color",
            )?;
            require(
                !case.namespace.is_empty() && !case.recipe_id.is_empty() && !case.state.is_empty(),
                "missing case identity",
            )?;
            if case.app == "holla" && case.driver == Driver::Cli {
                require(
                    case.theme == "reference-default"
                        && !case
                            .argv
                            .iter()
                            .any(|arg| arg == "--theme" || arg.starts_with("--theme=")),
                    "Holla CLI theme is fixed reference-default",
                )?;
                require(
                    matches!(case.clock, Clock::Paused { .. }),
                    "Holla CLI deterministic clock adapter required",
                )?;
            }
        }
        Ok(())
    }
}
impl BuildManifest {
    pub(crate) fn validate(&self, required: &[&str]) -> Result<(), String> {
        self.confinement.validate_shape()?;
        require(
            self.schema == 1 && self.status == 0,
            "build did not succeed",
        )?;
        require(
            self.source.revision.len() == 40
                && self.source.revision.bytes().all(|b| b.is_ascii_hexdigit())
                && sha(&self.source.fingerprint)
                && sha(&self.source.lock_sha256),
            "invalid source binding",
        )?;
        require(
            !self.cargo.is_empty()
                && !self.rustc.is_empty()
                && !self.host_target.is_empty()
                && sha(&self.environment_sha256),
            "missing toolchain/environment binding",
        )?;
        require(
            !self.compiler_inputs.is_empty()
                && self
                    .compiler_inputs
                    .iter()
                    .all(|(path, digest)| !path.is_empty() && sha(digest)),
            "missing compiler input binding",
        )?;
        require(
            sha(&self.messages_sha256)
                && sha(&self.stderr_sha256)
                && !self.messages_path.is_empty()
                && !self.stderr_path.is_empty(),
            "missing successful build transcript",
        )?;
        require(
            self.argv
                .starts_with(&["cargo".into(), "build".into(), "--locked".into()]),
            "build argv is not locked Cargo build",
        )?;
        let mut seen = BTreeSet::new();
        for artifact in &self.artifacts {
            require(seen.insert(artifact.app.as_str()), "duplicate build app")?;
            require(
                artifact.target == artifact.app
                    && !artifact.package_id.is_empty()
                    && !artifact.executable.is_empty()
                    && sha(&artifact.sha256),
                "invalid binary identity",
            )?;
            require(
                artifact
                    .profile
                    .get("test")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false),
                "test executable is not an application binary",
            )?;
            require(
                artifact.features.iter().collect::<BTreeSet<_>>().len() == artifact.features.len(),
                "duplicate build feature",
            )?;
        }
        require(
            seen == required.iter().copied().collect(),
            "missing or unexpected build app",
        )
    }
}
impl CaptureJoin {
    pub(crate) fn validate(
        &self,
        plan: &Plan,
        plan_hash: &str,
        build: &BuildManifest,
        build_hash: &str,
    ) -> Result<(), String> {
        plan.validate()?;
        let apps: Vec<_> = build
            .artifacts
            .iter()
            .map(|artifact| artifact.app.as_str())
            .collect();
        build.validate(&apps)?;
        require(
            self.schema == 1 && plan.blockers.is_empty(),
            "blocked or invalid capture",
        )?;
        require(
            self.plan_sha256 == plan_hash && self.build_manifest_sha256 == build_hash,
            "plan/build binding differs",
        )?;
        require(
            plan.cases.contains(&self.case) && self.case_sha256 == encoded_hash(&self.case)?,
            "case missing or changed",
        )?;
        let binary = build
            .artifacts
            .iter()
            .find(|artifact| artifact.app == self.case.app)
            .ok_or("capture app absent from build")?;
        require(
            !self.run_id.is_empty()
                && self.source_binary_sha256 == binary.sha256
                && self.staged_binary_sha256 == binary.sha256,
            "wrong source/staged executable",
        )?;
        require(
            self.stages
                .iter()
                .map(|stage| stage.name.as_str())
                .collect::<Vec<_>>()
                == ["start", "shot", "stop"]
                && self.stages.iter().all(|stage| stage.status == 0),
            "missing, duplicate or failed capture stage",
        )?;
        require(
            self.artifacts
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                == ["ansi", "cursor", "html", "png", "txt"],
            "missing or unexpected artifact",
        )?;
        require(
            self.artifacts
                .values()
                .all(|file| file.bytes > 0 && sha(&file.sha256) && !file.path.is_empty())
                && sha(&self.provenance_sha256),
            "empty or unbound artifact",
        )
    }
}
/// Deliberately checks schema/joins only, not execution or current-source provenance.
pub(crate) fn schema_command(paths: &[String]) -> Result<(), String> {
    if paths.len() != 3 {
        return Err("usage: capture-schema PLAN BUILD JOIN (schema only)".into());
    }
    let plan_bytes = std::fs::read(&paths[0]).map_err(|e| e.to_string())?;
    let build_bytes = std::fs::read(&paths[1]).map_err(|e| e.to_string())?;
    let join_bytes = std::fs::read(&paths[2]).map_err(|e| e.to_string())?;
    let plan: Plan = strict_json(&plan_bytes)?;
    let build: BuildManifest = strict_json(&build_bytes)?;
    let join: CaptureJoin = strict_json(&join_bytes)?;
    build.validate(&["showcase", "tablepro", "jackin-preview", "holla"])?;
    join.validate(&plan, &hash(&plan_bytes), &build, &hash(&build_bytes))?;
    println!(
        "schema and declared joins valid; execution, current source and artifact files NOT verified"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn untyped_receipts_and_nested_provenance_reject_duplicate_keys() {
        for raw in [
            r#"{"stage":{"name":"shot","status":1,"status":0},"error":null}"#,
            r#"{"run_id":"old","run\u005fid":"new"}"#,
            r#"[{"artifacts":{"png":{"sha256":"wrong","sha256":"right"}}}]"#,
        ] {
            // This is the old consumer's failure: valid JSON was accepted and
            // the earlier conflicting value was silently discarded.
            serde_json::from_str::<serde_json::Value>(raw).unwrap();
            assert!(strict_json::<serde_json::Value>(raw.as_bytes()).is_err());
        }
        let raw =
            br#"[{"stage":{"name":"shot","status":0},"error":null,"bytes":18446744073709551615}]"#;
        let actual: serde_json::Value = strict_json(raw).unwrap();
        assert_eq!(
            actual,
            serde_json::from_slice::<serde_json::Value>(raw).unwrap()
        );
    }
    fn case() -> Case {
        Case {
            namespace: "startup-v2".into(),
            recipe_id: "holla_reference-default_truecolor_80x24".into(),
            app: "holla".into(),
            driver: Driver::Cli,
            scenario: Some("first-use".into()),
            state: "startup".into(),
            viewport: (80, 24),
            theme: "reference-default".into(),
            color: "truecolor".into(),
            clock: Clock::Paused { frame: 4000 },
            argv: vec![],
        }
    }
    #[test]
    fn plan_rejects_duplicate_identity_and_invented_holla_cli_capabilities() {
        let mut plan = Plan {
            schema: 1,
            inventory_sha256: hash(b"inventory"),
            cases: vec![case()],
            blockers: vec![],
        };
        plan.validate().unwrap();
        plan.cases.push(case());
        assert!(plan.validate().is_err());
        plan.cases.pop();
        plan.cases[0].theme = "paper".into();
        assert!(plan.validate().is_err());
        plan.cases[0].driver = Driver::ProductionView;
        plan.validate().unwrap();
        let mut value = serde_json::to_value(&plan).unwrap();
        value["unexpected"] = true.into();
        assert!(serde_json::from_value::<Plan>(value).is_err());
    }
    fn fixture_build() -> BuildManifest {
        BuildManifest {
            schema: 1,
            source: Source {
                revision: "a".repeat(40),
                fingerprint: hash(b"source"),
                lock_sha256: hash(b"lock"),
            },
            cargo: "cargo fixture".into(),
            rustc: "rustc fixture".into(),
            host_target: "fixture".into(),
            environment_sha256: hash(b"env"),
            compiler_inputs: [("source.rs".into(), hash(b"source"))].into(),
            confinement: crate::capture_confinement::Confinement {
                backend: "macos-seatbelt-v1".into(),
                profile: "declared-only".into(),
                environment: ["PATH", "HOME", "CARGO_HOME", "RUSTC"]
                    .into_iter()
                    .map(|k| (k.into(), "/fixture".into()))
                    .collect(),
                input_roots: [("/fixture".into(), hash(b"inputs"))].into(),
                platform: "fixture".into(),
                cargo: "/fixture/cargo".into(),
            },
            argv: vec!["cargo".into(), "build".into(), "--locked".into()],
            status: 0,
            messages_path: "messages".into(),
            messages_sha256: hash(b"messages"),
            stderr_path: "stderr".into(),
            stderr_sha256: hash(b""),
            artifacts: vec![Artifact {
                app: "holla".into(),
                package_id: "holla fixture".into(),
                target: "holla".into(),
                executable: "/fixture/holla".into(),
                sha256: hash(b"binary"),
                features: vec![],
                profile: serde_json::json!({"test":false}),
            }],
        }
    }
    #[test]
    fn build_rejects_failed_duplicate_missing_and_test_executable() {
        let mut build = fixture_build();
        build.validate(&["holla"]).unwrap();
        build.status = 101;
        assert!(build.validate(&["holla"]).is_err());
        build.status = 0;
        build.artifacts.push(build.artifacts[0].clone());
        assert!(build.validate(&["holla"]).is_err());
        build.artifacts.pop();
        assert!(build.validate(&["holla", "showcase"]).is_err());
        build.artifacts[0].profile["test"] = true.into();
        assert!(build.validate(&["holla"]).is_err());
    }
    #[test]
    fn build_maps_reject_duplicate_keys_before_values_are_discarded() {
        let raw = serde_json::to_string(&fixture_build()).unwrap();
        serde_json::from_str::<BuildManifest>(&raw).unwrap();
        for (field, key, value) in [
            (
                "compiler_inputs",
                "source.rs",
                serde_json::json!("0".repeat(64)),
            ),
            ("environment", "PATH", serde_json::json!("/other")),
            ("input_roots", "/fixture", serde_json::json!("0".repeat(64))),
            ("profile", "test", serde_json::json!(true)),
        ] {
            let prefix = format!("\"{field}\":{{");
            let replacement = format!("{prefix}{}:{value},", serde_json::to_string(key).unwrap());
            let duplicate = raw.replacen(&prefix, &replacement, 1);
            assert_ne!(duplicate, raw);
            let error = serde_json::from_str::<BuildManifest>(&duplicate).unwrap_err();
            assert!(
                error.to_string().contains("duplicate evidence map key"),
                "{field}: {error}"
            );
        }
    }
    #[test]
    fn joins_reject_stale_case_wrong_binary_missing_artifact_and_failed_stage() {
        let build = fixture_build();
        let mut plan = Plan {
            schema: 1,
            inventory_sha256: hash(b"inventory"),
            cases: vec![case()],
            blockers: vec![],
        };
        let base = CaptureJoin {
            schema: 1,
            case: case(),
            case_sha256: encoded_hash(&case()).unwrap(),
            plan_sha256: hash(b"plan"),
            build_manifest_sha256: hash(b"build"),
            run_id: "run1".into(),
            source_binary_sha256: hash(b"binary"),
            staged_binary_sha256: hash(b"binary"),
            stages: ["start", "shot", "stop"]
                .into_iter()
                .map(|name| Stage {
                    name: name.into(),
                    status: 0,
                })
                .collect(),
            artifacts: ["ansi", "cursor", "html", "png", "txt"]
                .into_iter()
                .map(|name| {
                    (
                        name.into(),
                        FileHash {
                            path: name.into(),
                            bytes: 1,
                            sha256: hash(b"artifact"),
                        },
                    )
                })
                .collect(),
            provenance_sha256: hash(b"provenance"),
        };
        base.validate(&plan, &hash(b"plan"), &build, &hash(b"build"))
            .unwrap();
        let raw = serde_json::to_string(&base).unwrap();
        let duplicate = raw.replacen(
            "\"artifacts\":{",
            &format!(
                "\"artifacts\":{{\"ansi\":{},",
                serde_json::to_string(&base.artifacts["ansi"]).unwrap()
            ),
            1,
        );
        let error = serde_json::from_str::<CaptureJoin>(&duplicate).unwrap_err();
        assert!(error.to_string().contains("duplicate evidence map key"));
        for mutation in 0..5 {
            let mut join = base.clone();
            match mutation {
                0 => join.case.viewport = (1, 1),
                1 => join.staged_binary_sha256 = hash(b"other"),
                2 => {
                    join.artifacts.remove("png");
                }
                3 => join.stages[1].status = 1,
                _ => {
                    join.stages.pop();
                }
            };
            assert!(
                join.validate(&plan, &hash(b"plan"), &build, &hash(b"build"))
                    .is_err()
            );
        }
        plan.blockers.push("unsupported clock adapter".into());
        assert!(
            base.validate(&plan, &hash(b"plan"), &build, &hash(b"build"))
                .is_err()
        );
    }
}
