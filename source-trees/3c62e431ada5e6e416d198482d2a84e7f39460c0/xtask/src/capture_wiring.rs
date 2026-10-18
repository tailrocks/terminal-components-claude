//! Execute finite case plans only with verified compiler build evidence.
use crate::CaptureCase;
use crate::capture_contract::{self as contract, CaptureJoin, Case, Clock, Driver, Plan};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn cases(suite: &str) -> Result<Vec<CaptureCase>, String> {
    if suite == "startup-v2" {
        return Ok(crate::capture_matrix_cases());
    }
    let additional = match suite {
        "holla-p6-794b095" => false,
        "holla-ansi16-v1" => true,
        "holla-paper-view-v1" => return Err(
            "Paper requires the Holla public production-view adapter; CLI capture is unsupported"
                .into(),
        ),
        _ => return Err("unsupported capture suite/clock adapter".into()),
    };
    Ok(crate::app_inventory::get()?
        .holla_cases(additional)
        .into_iter()
        .map(|c| CaptureCase {
            app: crate::CAPTURE_APPS[3],
            scenario: Some(c.scenario),
            width: c.width,
            height: c.height,
            color: c.color,
            theme: c.theme,
        })
        .collect())
}
fn suite(case: CaptureCase) -> &'static str {
    if case.scenario.is_none() {
        "startup-v2"
    } else if case.color == "16" {
        "holla-ansi16-v1"
    } else {
        "holla-p6-794b095"
    }
}
fn normalized(case: CaptureCase) -> Case {
    Case {
        namespace: suite(case).into(),
        recipe_id: case.shot_name(),
        app: case.app.name.into(),
        driver: Driver::Cli,
        scenario: (case.app.name == "holla").then(|| case.scenario.unwrap_or("first-use").into()),
        state: "startup".into(),
        viewport: (case.width, case.height),
        theme: case.theme.into(),
        color: case.color.into(),
        clock: match case.app.name {
            "holla" => Clock::Paused { frame: 4000 },
            "jackin-preview" => Clock::Paused { frame: 0 },
            _ => Clock::NotApplicable,
        },
        argv: crate::capture_arguments(case),
    }
}
pub(crate) fn provenance_path(shots: &Path, case: CaptureCase) -> PathBuf {
    if suite(case) == "startup-v2" {
        shots.join("capture-provenance.json")
    } else {
        shots.join(format!("capture-provenance-{}.json", suite(case)))
    }
}
pub(crate) struct Bundle {
    directory: PathBuf,
    plan: Plan,
    plan_hash: String,
    build_hash: String,
}
impl Bundle {
    fn shots(&self) -> PathBuf {
        self.directory.join("shots")
    }
    fn verify_files(&self, build_path: &Path) -> Result<(), String> {
        let build = crate::capture_build::verify(&crate::root(), build_path)?;
        if crate::capture_build::file_hash(build_path)? != self.build_hash {
            return Err("build manifest changed during capture".into());
        }
        if crate::capture_build::file_hash(&self.directory.join("plan.json"))? != self.plan_hash
            || self.plan.inventory_sha256
                != crate::capture_build::file_hash(&crate::root().join("tools/app-inventory.json"))?
        {
            return Err("capture plan or inventory changed".into());
        }
        let namespace = self
            .plan
            .cases
            .first()
            .ok_or("empty capture plan")?
            .namespace
            .as_str();
        let expected: Vec<_> = cases(namespace)?.into_iter().map(normalized).collect();
        if self.plan.cases != expected {
            return Err("capture plan differs from authoritative suite".into());
        }
        let joins = fs::read_dir(&self.directory)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let actual: std::collections::BTreeSet<_> = joins
            .iter()
            .filter_map(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .filter(|s| s.ends_with("-join.json"))
                    .map(str::to_owned)
            })
            .collect();
        let wanted: std::collections::BTreeSet<_> = self
            .plan
            .cases
            .iter()
            .map(|c| format!("{}-join.json", c.recipe_id))
            .collect();
        if actual != wanted {
            return Err("missing or unexpected capture join".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for case in &self.plan.cases {
            let path = self.directory.join(format!("{}-join.json", case.recipe_id));
            let join: CaptureJoin =
                contract::strict_json(&fs::read(path).map_err(|e| e.to_string())?)?;
            join.validate(&self.plan, &self.plan_hash, &build, &self.build_hash)?;
            if !seen.insert(join.run_id.clone()) {
                return Err("duplicate capture run".into());
            }
            for stage in &join.stages {
                let receipt: serde_json::Value = contract::strict_json(
                    &fs::read(
                        self.directory
                            .join(format!("{}-{}.json", case.recipe_id, stage.name)),
                    )
                    .map_err(|e| e.to_string())?,
                )?;
                if receipt["run_id"] != join.run_id
                    || receipt["stage"] != serde_json::to_value(stage).map_err(|e| e.to_string())?
                    || !receipt["error"].is_null()
                {
                    return Err("capture stage receipt changed".into());
                }
            }
            let generated = cases(namespace)?
                .into_iter()
                .find(|c| c.shot_name() == case.recipe_id)
                .ok_or("case disappeared")?;
            let records: Vec<serde_json::Value> = contract::strict_json(
                &fs::read(provenance_path(&self.shots(), generated)).map_err(|e| e.to_string())?,
            )?;
            let records: Vec<_> = records
                .iter()
                .filter(|r| r["name"] == case.recipe_id)
                .collect();
            let [record] = records.as_slice() else {
                return Err("provenance case missing or duplicated".into());
            };
            if record["run_id"] != join.run_id
                || contract::encoded_hash(record)? != join.provenance_sha256
            {
                return Err("capture provenance changed".into());
            }
            let staged = record["binary"]["resolved_path"]
                .as_str()
                .ok_or("staged binary path absent")?;
            if crate::capture_build::file_hash(Path::new(staged))? != join.staged_binary_sha256 {
                return Err("staged binary changed".into());
            }
            for (name, file) in &join.artifacts {
                if Path::new(&file.path) != self.shots().join(&case.recipe_id).join(name) {
                    return Err("capture artifact escaped authoritative bundle path".into());
                }
                if crate::capture_build::file_hash(Path::new(&file.path))? != file.sha256 {
                    return Err("artifact changed during capture".into());
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn verify_command(args: &[String]) -> Result<(), String> {
    let [build_flag, build_path, bundle_flag, directory] = args else {
        return Err("usage: capture-verify --build-manifest FILE --bundle DIRECTORY".into());
    };
    if build_flag != "--build-manifest" || bundle_flag != "--bundle" {
        return Err("invalid capture verification arguments".into());
    }
    let directory = PathBuf::from(directory);
    let path = directory.join("plan.json");
    let plan: Plan = contract::strict_json(&fs::read(&path).map_err(|e| e.to_string())?)?;
    let bundle = Bundle {
        directory,
        plan,
        plan_hash: crate::capture_build::file_hash(&path)?,
        build_hash: crate::capture_build::file_hash(Path::new(build_path))?,
    };
    bundle.verify_files(Path::new(build_path))?;
    let complete: serde_json::Value = contract::strict_json(
        &fs::read(bundle.directory.join("complete.json")).map_err(|e| e.to_string())?,
    )?;
    if complete
        != serde_json::json!({"classification":"captured-not-approved","plan_sha256":bundle.plan_hash,"build_manifest_sha256":bundle.build_hash,"cases":bundle.plan.cases.len()})
    {
        return Err("missing or stale capture completion receipt".into());
    }
    println!("capture source/build/run/artifact joins verified; visual approval separate");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_suite_plans_preserve_ids_and_refuse_unsupported_drivers() {
        assert_eq!(cases("startup-v2").unwrap().len(), 112);
        let pinned = cases("holla-p6-794b095").unwrap();
        assert_eq!(pinned.len(), 132);
        assert_eq!(cases("holla-ansi16-v1").unwrap().len(), 44);
        for case in pinned {
            let plan = normalized(case);
            assert!(plan.recipe_id.starts_with("h_p6_"));
            assert!(!plan.argv.iter().any(|a| a == "--theme"));
            assert_eq!(plan.argv[1], plan.scenario.unwrap());
        }
        assert!(cases("holla-paper-view-v1").is_err());
        assert!(cases("full-motion").is_err());
    }
    #[test]
    fn external_bundle_tsv_uses_external_artifacts_without_source_shots() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("capture-external-{}-{nonce}", std::process::id()));
        let shots = directory.join("shots");
        let case = cases("startup-v2").unwrap()[0];
        std::fs::create_dir_all(shots.join(case.shot_name())).unwrap();
        std::fs::write(
            shots.join(case.shot_name()).join("png"),
            b"path-and-hash fixture, not a production capture",
        )
        .unwrap();
        let record = crate::capture_record(&shots, case).unwrap();
        let manifest = shots.join("capture-matrix.tsv");
        crate::write_capture_manifest(&manifest, &[record]).unwrap();
        crate::validate_capture_tsv(&[case], &manifest).unwrap();
        let original = std::fs::read_to_string(&manifest).unwrap();
        let changed = original.replace(
            &shots.to_string_lossy().to_string(),
            &crate::root().join("shots").to_string_lossy(),
        );
        std::fs::write(&manifest, changed).unwrap();
        assert!(crate::validate_capture_tsv(&[case], &manifest).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
