use std::collections::HashSet;

struct Burndown {
    id: &'static str,
    cases: &'static [&'static str],
    test: &'static str,
    component: &'static str,
    owner: &'static str,
    reason: &'static str,
}

const BURNDOWN: &[Burndown] = &[Burndown {
    id: "BD-21",
    cases: &["W13-05"],
    test: "w13_filter_wide_trail_cells_clear",
    component: "W13",
    owner: "termrock-navigation",
    reason: "wide-char trail cells never cleared; stale glyphs leak",
}];

const OWNERSHIP_JSON: &str = include_str!("../../../component-ownership.json");
const REGISTRY_JSON: &str = include_str!("../../../tests/conformance/required_cases.json");
const MIRROR_SRC: &str = include_str!("control_states.rs");

#[test]
fn burndown_owners_match_ownership_map() {
    let doc: serde_json::Value =
        serde_json::from_str(OWNERSHIP_JSON).expect("ownership map must parse");
    for row in BURNDOWN {
        let found = doc["components"]
            .as_array()
            .expect("components array")
            .iter()
            .find(|c| c["id"].as_str() == Some(row.component));
        let entry = found
            .unwrap_or_else(|| panic!("{}: {} missing from ownership map", row.id, row.component));
        assert_eq!(
            entry["owner"].as_str(),
            Some(row.owner),
            "{}: owner drift for {}",
            row.id,
            row.component
        );
    }
}

#[test]
fn burndown_cases_exist_in_registry() {
    let doc: serde_json::Value = serde_json::from_str(REGISTRY_JSON).expect("registry must parse");
    let ids: HashSet<&str> = doc["cases"]
        .as_array()
        .expect("cases array")
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    assert_eq!(
        ids.iter().filter(|id| id.starts_with('W')).count(),
        222,
        "registry must carry the full 222-case W vocabulary"
    );
    for row in BURNDOWN {
        for case in row.cases {
            assert!(
                ids.contains(case),
                "{}: {case} is not a registry case id",
                row.id
            );
        }
    }
}

#[test]
fn burndown_matches_ignored_tests() {
    let ignore_lines: Vec<&str> = MIRROR_SRC
        .lines()
        .filter(|l| l.trim_start().starts_with("#[ignore = \"PARITY"))
        .collect();
    assert_eq!(
        ignore_lines.len(),
        BURNDOWN.len(),
        "ignore count drifted: {} PARITY ignores vs {} burndown rows",
        ignore_lines.len(),
        BURNDOWN.len()
    );
    let deferred_refs: usize = BURNDOWN.iter().map(|row| row.cases.len()).sum();
    assert_eq!(deferred_refs, 1, "deferred case references must total 1");
    for row in BURNDOWN {
        assert!(
            !row.reason.is_empty(),
            "{}: burndown row needs a reason",
            row.id
        );
        let head = format!("PARITY {}", row.cases.join("/"));
        assert!(
            ignore_lines.iter().any(|l| l.contains(&head)),
            "{} ({}): no PARITY ignore headed `{head}`",
            row.id,
            row.reason
        );
        assert!(
            MIRROR_SRC.contains(&format!("fn {}(", row.test)),
            "{}: ignored test fn {} missing",
            row.id,
            row.test
        );
    }
}
