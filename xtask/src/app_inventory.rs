//! Current application identity and finite capture obligations. Historical parity is separate.
use std::collections::BTreeSet;
use std::sync::LazyLock;

use cargo_metadata::{Metadata, TargetKind};
use serde::{Deserialize, Serialize};

const SOURCE: &str = include_str!("../../tools/app-inventory.json");
// Independent requirement pins: deleting an inventory entry cannot weaken acceptance.
const REQUIRED: [&str; 4] = ["showcase", "tablepro", "jackin-preview", "holla"];
const SCENARIOS: [&str; 11] = [
    "first-use",
    "rust-dirty",
    "monorepo-root",
    "monorepo-child",
    "docker-cleanup",
    "disk-cleanup",
    "upgrade-plan",
    "activities-multi",
    "remote-host",
    "launch-failure",
    "hard-cases",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Inventory<'a> {
    schema: u8,
    #[serde(borrow)]
    pub(crate) apps: Vec<AppPackage<'a>>,
    #[serde(borrow)]
    holla: Holla<'a>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppPackage<'a> {
    pub(crate) id: &'a str,
    pub(crate) package: &'a str,
    pub(crate) bin: &'a str,
    pub(crate) dir: &'a str,
    pub(crate) lib: &'a str,
    pub(crate) slice: &'a str,
    #[serde(borrow)]
    pub(crate) perf_targets: Vec<&'a str>,
    #[serde(borrow)]
    pub(crate) startup: Startup<'a>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Startup<'a> {
    #[serde(borrow)]
    pub(crate) themes: Vec<&'a str>,
    pub(crate) theme_argument: bool,
    #[serde(borrow)]
    pub(crate) args: Vec<&'a str>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Holla<'a> {
    reference_revision: &'a str,
    #[serde(borrow)]
    scenarios: Vec<&'a str>,
    sizes: Vec<(u16, u16)>,
    #[serde(borrow)]
    reference_colors: Vec<&'a str>,
    #[serde(borrow)]
    additional_colors: Vec<&'a str>,
    motion: &'a str,
    frame: u64,
    theme: &'a str,
    #[serde(borrow)]
    paper_view: ViewRequirement<'a>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ViewRequirement<'a> {
    suite: &'a str,
    driver: &'a str,
    theme: &'a str,
    required: bool,
}
#[derive(Debug, Serialize)]
pub(crate) struct HollaCase<'a> {
    pub(crate) namespace: &'static str,
    pub(crate) recipe_id: String,
    pub(crate) scenario: &'a str,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) color: &'a str,
    pub(crate) motion: &'a str,
    pub(crate) frame: u64,
    pub(crate) theme: &'a str,
    pub(crate) reference_revision: Option<&'a str>,
}

fn require(ok: bool, message: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("app inventory: {message}"))
    }
}

pub(crate) fn parse(source: &str) -> Result<Inventory<'_>, String> {
    let inventory: Inventory<'_> =
        serde_json::from_str(source).map_err(|error| format!("app inventory schema: {error}"))?;
    inventory.validate()?;
    Ok(inventory)
}
static INVENTORY: LazyLock<Result<Inventory<'static>, String>> = LazyLock::new(|| parse(SOURCE));
pub(crate) fn get() -> Result<&'static Inventory<'static>, String> {
    INVENTORY.as_ref().map_err(Clone::clone)
}

impl Inventory<'_> {
    fn validate(&self) -> Result<(), String> {
        require(self.schema == 1, "unsupported schema")?;
        require(
            self.apps.iter().map(|app| app.id).collect::<Vec<_>>() == REQUIRED,
            "exact four required app IDs/order must be preserved",
        )?;
        let mut bins = BTreeSet::new();
        for app in &self.apps {
            require(
                app.package == app.id && app.bin == app.id && bins.insert(app.bin),
                "package/binary identity differs or duplicates",
            )?;
            require(
                app.dir == format!("apps/{}", app.id),
                "application source directory differs",
            )?;
            let expected_lib = match app.id {
                "showcase" => "showcase_app",
                "tablepro" => "tablepro_app",
                "jackin-preview" => "jackin_app",
                "holla" => "holla_app",
                _ => unreachable!(),
            };
            require(
                app.lib == expected_lib && !app.slice.is_empty(),
                "library/slice identity differs",
            )?;
            require(
                app.perf_targets == ["perf"],
                "required perf target omitted or changed",
            )?;
            let (themes, args): (&[&str], &[&str]) = match app.id {
                "holla" => (
                    &["reference-default"],
                    &[
                        "--scenario",
                        "first-use",
                        "--motion",
                        "paused",
                        "--frame",
                        "4000",
                    ],
                ),
                "jackin-preview" => (&["junie", "paper"], &["--motion", "paused", "--frame", "0"]),
                _ => (&["junie", "paper"], &[]),
            };
            require(
                app.startup.themes == themes
                    && app.startup.args == args
                    && app.startup.theme_argument == (app.id != "holla"),
                "startup capability contract differs",
            )?;
        }
        let h = &self.holla;
        require(
            h.reference_revision == "794b095c196562d38f1b6f7ce379c128af2a023d"
                && h.scenarios == SCENARIOS
                && h.sizes == [(80, 24), (100, 30), (120, 40), (160, 50)]
                && h.reference_colors == ["truecolor", "256", "mono"]
                && h.additional_colors == ["16"]
                && h.motion == "paused"
                && h.frame == 4000
                && h.theme == "reference-default",
            "pinned Holla132 or additional ANSI16 obligations differ",
        )?;
        require(
            h.paper_view.suite == "holla-paper-view-v1"
                && h.paper_view.driver == "production-view"
                && h.paper_view.theme == "paper"
                && h.paper_view.required,
            "required Paper production-view coverage omitted or changed",
        )
    }

    /// Real metadata evidence, never an inferred executable or test success.
    pub(crate) fn validate_targets(&self, metadata: &Metadata) -> Result<(), String> {
        let mut errors = Vec::new();
        for app in &self.apps {
            let packages: Vec<_> = metadata
                .workspace_packages()
                .into_iter()
                .filter(|package| package.name.as_str() == app.package)
                .collect();
            let [package] = packages.as_slice() else {
                errors.push(format!("{}: required package absent or duplicated", app.id));
                continue;
            };
            if package.manifest_path.parent()
                != Some(metadata.workspace_root.join(app.dir).as_path())
            {
                errors.push(format!("{}: package source directory differs", app.id));
            }
            let owners = metadata
                .workspace_packages()
                .into_iter()
                .flat_map(|p| &p.targets)
                .filter(|target| target.kind.contains(&TargetKind::Bin) && target.name == app.bin)
                .count();
            if owners != 1
                || !package
                    .targets
                    .iter()
                    .any(|t| t.kind.contains(&TargetKind::Bin) && t.name == app.bin)
            {
                errors.push(format!(
                    "{}: required binary absent, wrong owner or duplicated",
                    app.id
                ));
            }
            if !package
                .targets
                .iter()
                .any(|t| t.kind.contains(&TargetKind::Lib) && t.name == app.lib)
            {
                errors.push(format!("{}: required library absent", app.id));
            }
            for name in &app.perf_targets {
                if !package.targets.iter().any(|t| {
                    t.kind.contains(&TargetKind::Test)
                        && t.name == *name
                        && t.test
                        && t.required_features.is_empty()
                }) {
                    errors.push(format!(
                        "{}: required ungated perf target {name} absent",
                        app.id
                    ));
                }
            }
        }
        require(errors.is_empty(), &errors.join("; "))
    }

    pub(crate) fn holla_cases(&self, additional: bool) -> Vec<HollaCase<'_>> {
        let h = &self.holla;
        let colors = if additional {
            &h.additional_colors
        } else {
            &h.reference_colors
        };
        let mut cases = Vec::new();
        for scenario in &h.scenarios {
            for &(width, height) in &h.sizes {
                for color in colors {
                    cases.push(HollaCase {
                        namespace: if additional {
                            "holla-ansi16-v1"
                        } else {
                            "holla-p6-794b095"
                        },
                        recipe_id: format!("h_p6_{scenario}_{width}x{height}_{color}"),
                        scenario,
                        width,
                        height,
                        color,
                        motion: h.motion,
                        frame: h.frame,
                        theme: h.theme,
                        reference_revision: (!additional).then_some(h.reference_revision),
                    });
                }
            }
        }
        cases
    }

    pub(crate) fn perf_commands(&self) -> Vec<Vec<String>> {
        self.apps
            .iter()
            .map(|app| {
                let mut args = vec![
                    "test".into(),
                    "--locked".into(),
                    "-p".into(),
                    app.package.into(),
                ];
                for target in &app.perf_targets {
                    args.extend(["--test".into(), (*target).into()]);
                }
                args.extend([
                    "--release".into(),
                    "--".into(),
                    "--test-threads=1".into(),
                    "--nocapture".into(),
                ]);
                args
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_covers_four_apps_and_separate_pinned_holla_suites() {
        let inventory = parse(SOURCE).unwrap();
        let original = inventory.holla_cases(false);
        let additional = inventory.holla_cases(true);
        assert_eq!(original.len(), 132);
        assert_eq!(additional.len(), 44);
        let ids: BTreeSet<_> = original
            .iter()
            .chain(&additional)
            .map(|c| &c.recipe_id)
            .collect();
        assert_eq!(ids.len(), 176);
        assert!(original.iter().all(|c| c.reference_revision.is_some()));
        assert!(additional.iter().all(|c| c.reference_revision.is_none()));
        assert_eq!(inventory.perf_commands().len(), 4);
        assert!(
            inventory
                .perf_commands()
                .iter()
                .all(|args| args.contains(&"--locked".into()))
        );
    }
    #[test]
    fn schema_and_obligation_removal_fail_closed() {
        let source: serde_json::Value = serde_json::from_str(SOURCE).unwrap();
        for mutation in 0..6 {
            let mut value = source.clone();
            match mutation {
                0 => {
                    value["apps"].as_array_mut().unwrap().pop();
                }
                1 => value["apps"][3]["startup"]["theme_argument"] = true.into(),
                2 => value["apps"][3]["perf_targets"] = serde_json::json!([]),
                3 => value["holla"]["paper_view"]["required"] = false.into(),
                4 => {
                    value["holla"]["scenarios"].as_array_mut().unwrap().pop();
                }
                _ => value["unknown"] = true.into(),
            }
            assert!(parse(&value.to_string()).is_err(), "mutation {mutation}");
        }
        assert!(
            parse(&SOURCE.replacen("\"schema\": 1", "\"schema\": 1, \"schema\": 1", 1)).is_err()
        );
    }
    #[test]
    fn real_cargo_metadata_rejects_missing_wrong_owner_and_gated_targets() {
        let directory =
            std::env::temp_dir().join(format!("app-inventory-metadata-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let inventory = parse(SOURCE).unwrap();
        let members: Vec<_> = inventory.apps.iter().map(|app| app.dir).collect();
        std::fs::write(
            directory.join("Cargo.toml"),
            format!("[workspace]\nresolver=\"2\"\nmembers={members:?}\n"),
        )
        .unwrap();
        for app in &inventory.apps {
            let dir = directory.join(app.dir);
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::create_dir(dir.join("tests")).unwrap();
            std::fs::write(dir.join("Cargo.toml"),format!("[package]\nname=\"{}\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[lib]\nname=\"{}\"\n",app.package,app.lib)).unwrap();
            std::fs::write(dir.join("src/lib.rs"), "").unwrap();
            std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
            std::fs::write(dir.join("tests/perf.rs"), "#[test] fn fixture() {}\n").unwrap();
        }
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(directory.join("Cargo.toml"))
            .no_deps()
            .other_options(vec!["--offline".into()])
            .exec()
            .unwrap();
        inventory.validate_targets(&metadata).unwrap();
        for case in 0..4 {
            let mut changed = metadata.clone();
            let index = changed
                .packages
                .iter()
                .position(|p| p.name.as_str() == "holla")
                .unwrap();
            match case {
                0 => changed.packages[index]
                    .targets
                    .retain(|t| !t.kind.contains(&TargetKind::Bin)),
                1 => changed.packages[index]
                    .targets
                    .retain(|t| !t.kind.contains(&TargetKind::Test)),
                2 => changed.packages[index]
                    .targets
                    .iter_mut()
                    .find(|t| t.kind.contains(&TargetKind::Test))
                    .unwrap()
                    .required_features
                    .push("disabled".into()),
                _ => {
                    let bin = changed.packages[index]
                        .targets
                        .iter()
                        .find(|t| t.kind.contains(&TargetKind::Bin))
                        .unwrap()
                        .clone();
                    let other = changed
                        .packages
                        .iter_mut()
                        .find(|p| p.name.as_str() == "showcase")
                        .unwrap();
                    other.targets.push(bin);
                }
            }
            assert!(inventory.validate_targets(&changed).is_err(), "case {case}");
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
