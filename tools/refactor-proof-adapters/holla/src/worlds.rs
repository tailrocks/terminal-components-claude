//! Oracle's 34-world registry.
//!
//! Names and order are the frozen oracle `Scenario::ALL`
//! (`src/bin/holla/scenario.rs` in the `visual-baseline` tag): 11 concept
//! worlds followed by the 23 HP01–HP23 parity worlds. Current production
//! exposes only the 11 concept worlds (`Scenario::ALL` in
//! `apps/holla/src/scenario.rs`). Missing parity worlds are observed as
//! unavailable; they are never substituted.

use holla_app::Scenario;

/// Oracle `Scenario::ALL` length.
pub const ORACLE_WORLD_COUNT: usize = 34;
/// Oracle `Scenario::CONCEPT` length, which matches current production `ALL`.
pub const CONCEPT_WORLD_COUNT: usize = 11;
/// Oracle `Scenario::PARITY` length; absent from current production.
pub const PARITY_WORLD_COUNT: usize = 23;

/// Oracle `Scenario::ALL` names in source order.
pub const ORACLE_WORLDS: [&str; ORACLE_WORLD_COUNT] = [
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
    "parity-discovery",
    "parity-history",
    "parity-files",
    "parity-browser",
    "parity-git-current",
    "parity-git-batch",
    "parity-task-sources",
    "parity-cargo",
    "parity-docker",
    "parity-brew-services",
    "parity-gradle",
    "parity-idea",
    "parity-upgrade-managers",
    "parity-executor",
    "parity-task-input",
    "parity-custom-actions",
    "parity-disk-scan",
    "parity-disk-navigation",
    "parity-insights",
    "parity-delete-safety",
    "parity-cleanup-results",
    "parity-platforms",
    "parity-platforms-linux",
];

/// Oracle `Scenario::CONCEPT` names.
pub const CONCEPT_WORLDS: [&str; CONCEPT_WORLD_COUNT] = [
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

/// Oracle `Scenario::PARITY` names.
pub const PARITY_WORLDS: [&str; PARITY_WORLD_COUNT] = [
    "parity-discovery",
    "parity-history",
    "parity-files",
    "parity-browser",
    "parity-git-current",
    "parity-git-batch",
    "parity-task-sources",
    "parity-cargo",
    "parity-docker",
    "parity-brew-services",
    "parity-gradle",
    "parity-idea",
    "parity-upgrade-managers",
    "parity-executor",
    "parity-task-input",
    "parity-custom-actions",
    "parity-disk-scan",
    "parity-disk-navigation",
    "parity-insights",
    "parity-delete-safety",
    "parity-cleanup-results",
    "parity-platforms",
    "parity-platforms-linux",
];

/// HO-BASE-01 … HO-BASE-34 in oracle world order.
pub const HO_BASE_IDS: [&str; ORACLE_WORLD_COUNT] = [
    "HO-BASE-01",
    "HO-BASE-02",
    "HO-BASE-03",
    "HO-BASE-04",
    "HO-BASE-05",
    "HO-BASE-06",
    "HO-BASE-07",
    "HO-BASE-08",
    "HO-BASE-09",
    "HO-BASE-10",
    "HO-BASE-11",
    "HO-BASE-12",
    "HO-BASE-13",
    "HO-BASE-14",
    "HO-BASE-15",
    "HO-BASE-16",
    "HO-BASE-17",
    "HO-BASE-18",
    "HO-BASE-19",
    "HO-BASE-20",
    "HO-BASE-21",
    "HO-BASE-22",
    "HO-BASE-23",
    "HO-BASE-24",
    "HO-BASE-25",
    "HO-BASE-26",
    "HO-BASE-27",
    "HO-BASE-28",
    "HO-BASE-29",
    "HO-BASE-30",
    "HO-BASE-31",
    "HO-BASE-32",
    "HO-BASE-33",
    "HO-BASE-34",
];

/// Intern an oracle world name.
#[must_use]
pub fn intern_world(name: &str) -> Option<&'static str> {
    ORACLE_WORLDS.iter().copied().find(|&world| world == name)
}

/// Production scenario for an oracle world name, if the current binary owns it.
#[must_use]
pub fn production_scenario(name: &str) -> Option<Scenario> {
    match name {
        "first-use" => Some(Scenario::FirstUse),
        "rust-dirty" => Some(Scenario::RustDirty),
        "monorepo-root" => Some(Scenario::MonorepoRoot),
        "monorepo-child" => Some(Scenario::MonorepoChild),
        "docker-cleanup" => Some(Scenario::DockerCleanup),
        "disk-cleanup" => Some(Scenario::DiskCleanup),
        "upgrade-plan" => Some(Scenario::UpgradePlan),
        "activities-multi" => Some(Scenario::ActivitiesMulti),
        "remote-host" => Some(Scenario::RemoteHost),
        "launch-failure" => Some(Scenario::LaunchFailure),
        "hard-cases" => Some(Scenario::HardCases),
        _ => None,
    }
}

/// Whether production currently constructs this oracle world.
#[must_use]
pub fn production_available(name: &str) -> bool {
    production_scenario(name).is_some()
}

/// Oracle world name for `HO-BASE-NN`.
#[must_use]
pub fn world_for_ho_base(id: &str) -> Option<&'static str> {
    HO_BASE_IDS
        .iter()
        .position(|&item| item == id)
        .and_then(|index| ORACLE_WORLDS.get(index).copied())
}

/// Count of oracle worlds production can construct without substitution.
#[must_use]
pub fn production_world_count() -> usize {
    Scenario::ALL.len()
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_34_with_11_production_and_23_parity() {
        assert_eq!(ORACLE_WORLDS.len(), 34);
        assert_eq!(CONCEPT_WORLDS.len(), 11);
        assert_eq!(PARITY_WORLDS.len(), 23);
        assert_eq!(production_world_count(), 11);
        assert_eq!(
            CONCEPT_WORLDS
                .iter()
                .filter(|world| production_available(world))
                .count(),
            11
        );
        assert_eq!(
            PARITY_WORLDS
                .iter()
                .filter(|world| production_available(world))
                .count(),
            0
        );
        assert_eq!(world_for_ho_base("HO-BASE-01"), Some("first-use"));
        assert_eq!(
            world_for_ho_base("HO-BASE-34"),
            Some("parity-platforms-linux")
        );
        assert!(production_scenario("parity-discovery").is_none());
    }
}
