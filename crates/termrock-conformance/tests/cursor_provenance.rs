//! Q07 cursor-provenance record gate (FIX-011 / R1).
//!
//! `tests/conformance/cursor_provenance.json` holds the per-root cursor state
//! reacquired from real `visual-baseline` tag execution (live PTY, store gate
//! bypassed). These tests bind that record to the vendored registry and the
//! snapshot migration map so silent edits, dropped roots, or vocabulary drift
//! fail loudly. They assert record integrity only; they never execute the tag
//! and never write approvals.

use std::collections::{BTreeMap, BTreeSet};

const PROVENANCE_JSON: &str = include_str!("../../../tests/conformance/cursor_provenance.json");
const REGISTRY_JSON: &str = include_str!("../../../tests/conformance/required_cases.json");
const MIGRATION_JSON: &str = include_str!("../../../docs/verification/snapshot-migration-map.json");

const EXPECTED_COMBOS: [&str; 2] = ["100x30/truecolor", "80x24/truecolor"];

fn provenance() -> serde_json::Value {
    serde_json::from_str(PROVENANCE_JSON).expect("parse cursor_provenance.json")
}

fn registry_legacy_ids() -> BTreeSet<String> {
    let registry: serde_json::Value =
        serde_json::from_str(REGISTRY_JSON).expect("parse required_cases.json");
    registry["cases"]
        .as_array()
        .expect("registry cases array")
        .iter()
        .filter_map(|case| case["id"].as_str())
        .filter(|id| id.starts_with("LEGACY:"))
        .map(str::to_string)
        .collect()
}

fn migration_targets() -> BTreeMap<String, String> {
    let migration: serde_json::Value =
        serde_json::from_str(MIGRATION_JSON).expect("parse snapshot-migration-map.json");
    migration["root_mappings"]
        .as_array()
        .expect("migration root_mappings array")
        .iter()
        .map(|entry| {
            (
                entry["legacy_root"]
                    .as_str()
                    .expect("legacy_root str")
                    .to_string(),
                entry["target_root"]
                    .as_str()
                    .expect("target_root str")
                    .to_string(),
            )
        })
        .collect()
}

fn records(doc: &serde_json::Value) -> &Vec<serde_json::Value> {
    doc["records"].as_array().expect("records array")
}

#[test]
fn record_header_pins_schema_and_reference() {
    let doc = provenance();
    assert_eq!(
        doc["schema"].as_str(),
        Some("termrock-spec/q07-cursor-provenance-v1")
    );
    assert_eq!(doc["slice"].as_str(), Some("Q07"));
    assert_eq!(
        doc["reference"]["commit"].as_str(),
        Some("4a79c0a2d40fca46fc406b77157ce3b3f12ec16b")
    );
    assert_eq!(
        doc["reference"]["capture_tool_rev"].as_str(),
        Some("2d43458ad2bc37d76653c22d56e61ee74512d893")
    );
    assert_eq!(
        doc["acquisition"]["method"].as_str(),
        Some("live-pty-tag-execution")
    );
}

#[test]
fn every_legacy_registry_root_has_exactly_one_record() {
    let doc = provenance();
    let legacy_ids = registry_legacy_ids();
    assert_eq!(legacy_ids.len(), 302, "vendored LEGACY case count");
    let mut seen = BTreeSet::new();
    for record in records(&doc) {
        let id = record["registry_id"].as_str().expect("registry_id str");
        assert!(
            legacy_ids.contains(id),
            "record {id} is not a vendored LEGACY registry id"
        );
        assert!(seen.insert(id.to_string()), "duplicate record {id}");
        assert_eq!(
            record["legacy_root"].as_str(),
            Some(id.strip_prefix("LEGACY:").expect("LEGACY prefix")),
            "record {id} legacy_root must match its registry id"
        );
    }
    assert_eq!(seen.len(), 302, "one record per LEGACY root");
    let missing: Vec<_> = legacy_ids.difference(&seen).collect();
    assert!(missing.is_empty(), "roots without records: {missing:?}");
}

#[test]
fn record_targets_match_migration_map() {
    let doc = provenance();
    let targets = migration_targets();
    assert_eq!(targets.len(), 302, "migration map root count");
    for record in records(&doc) {
        let legacy = record["legacy_root"].as_str().expect("legacy_root str");
        let expected = targets
            .get(legacy)
            .unwrap_or_else(|| panic!("{legacy} missing from migration map"));
        assert_eq!(
            record["target_root"].as_str(),
            Some(expected.as_str()),
            "{legacy} target drifted from migration map"
        );
    }
}

#[test]
fn record_shape_and_verdicts_are_consistent() {
    let doc = provenance();
    for record in records(&doc) {
        let id = record["registry_id"].as_str().expect("registry_id str");
        assert_eq!(
            record["method"].as_str(),
            Some("live-pty-tag-execution"),
            "{id} method"
        );
        let argv_class = record["argv_class"].as_str().expect("argv_class str");
        let argv = record["argv"].as_array().expect("argv array");
        match argv_class {
            "run-once" => {
                let head = argv.first().and_then(|v| v.as_str()).unwrap_or_default();
                assert!(
                    head.starts_with("<tag-bin:") && head.ends_with('>'),
                    "{id} run-once argv must start with a normalized tag binary: {head:?}"
                );
            }
            "live-session" => assert!(
                argv.is_empty(),
                "{id} live-session argv must be empty, got {argv:?}"
            ),
            other => panic!("{id} unknown argv_class {other:?}"),
        }
        let combos = record["combos"].as_object().expect("combos object");
        assert_eq!(
            combos.len(),
            EXPECTED_COMBOS.len(),
            "{id} must sample exactly the documented combos"
        );
        for combo in EXPECTED_COMBOS {
            let entry = combos
                .get(combo)
                .unwrap_or_else(|| panic!("{id} missing combo {combo}"));
            let cursor = &entry["cursor"];
            assert!(cursor["x"].is_u64(), "{id} {combo} cursor.x");
            assert!(cursor["y"].is_u64(), "{id} {combo} cursor.y");
            assert!(cursor["visible"].is_boolean(), "{id} {combo} visible");
            assert_eq!(
                cursor["style"].as_str(),
                Some("Block"),
                "{id} {combo} style"
            );
            assert_eq!(
                cursor["blinking"].as_bool(),
                Some(false),
                "{id} {combo} blinking"
            );
            assert!(entry["frame_digest"].is_u64(), "{id} {combo} digest");
            let (dims, _) = combo.split_once('/').expect("combo dims");
            let (cols, rows) = dims.split_once('x').expect("combo WxH");
            assert_eq!(
                entry["frame_cols"].as_u64(),
                Some(cols.parse().expect("cols")),
                "{id} {combo} frame_cols"
            );
            assert_eq!(
                entry["frame_rows"].as_u64(),
                Some(rows.parse().expect("rows")),
                "{id} {combo} frame_rows"
            );
            let expected_verdict = if cursor["visible"].as_bool() == Some(true) {
                "live-visible-vs-replay-hidden"
            } else {
                assert!(
                    cursor["x"] != serde_json::json!(0) || cursor["y"] != serde_json::json!(0),
                    "{id} {combo} hidden live cursor must be displaced from (0,0)"
                );
                "live-hidden-displaced"
            };
            assert_eq!(
                entry["verdict"].as_str(),
                Some(expected_verdict),
                "{id} {combo} verdict"
            );
        }
        let replay = &record["replay_cursor"];
        assert_eq!(replay["x"].as_u64(), Some(0), "{id} replay x");
        assert_eq!(replay["y"].as_u64(), Some(0), "{id} replay y");
        assert_eq!(
            replay["visible"].as_bool(),
            Some(false),
            "{id} replay visible"
        );
        assert!(
            record["visibility_size_stable"].is_boolean(),
            "{id} visibility_size_stable"
        );
        assert!(
            record["position_size_stable"].is_boolean(),
            "{id} position_size_stable"
        );
    }
}

#[test]
fn summary_totals_match_records() {
    let doc = provenance();
    let list = records(&doc);
    let summary = &doc["summary"];
    assert_eq!(summary["roots"].as_u64(), Some(list.len() as u64));
    assert_eq!(summary["combos_per_root"].as_u64(), Some(2));
    assert_eq!(summary["replay_default_matches"].as_u64(), Some(0));
    for combo in EXPECTED_COMBOS {
        let visible = list
            .iter()
            .filter(|r| r["combos"][combo]["cursor"]["visible"] == serde_json::json!(true))
            .count() as u64;
        let dims = combo.split('/').next().expect("combo dims");
        let key = format!("visible_live_{dims}");
        assert_eq!(
            summary[key.as_str()].as_u64(),
            Some(visible),
            "summary {key}"
        );
    }
    let live_session = list
        .iter()
        .filter(|r| r["argv_class"] == serde_json::json!("live-session"))
        .count() as u64;
    assert_eq!(summary["live_session_records"].as_u64(), Some(live_session));
    let position_stable = list
        .iter()
        .filter(|r| r["position_size_stable"] == serde_json::json!(true))
        .count() as u64;
    assert_eq!(
        summary["position_size_stable_roots"].as_u64(),
        Some(position_stable)
    );
}

#[test]
fn acquisition_headline_findings_anchored() {
    // Guards the record's headline claims against silent edits: replay
    // defaults match nothing, and the visibility/size-stability split below
    // is exactly what the two live sweeps observed.
    let doc = provenance();
    assert_eq!(doc["summary"]["roots"].as_u64(), Some(302));
    assert_eq!(doc["summary"]["replay_default_matches"].as_u64(), Some(0));
    assert_eq!(doc["summary"]["visible_live_100x30"].as_u64(), Some(78));
    assert_eq!(doc["summary"]["visible_live_80x24"].as_u64(), Some(78));
    assert_eq!(doc["summary"]["live_session_records"].as_u64(), Some(28));
    assert_eq!(
        doc["summary"]["position_size_stable_roots"].as_u64(),
        Some(68)
    );
    assert_eq!(
        records(&doc)
            .iter()
            .filter(|r| r["visibility_size_stable"] == serde_json::json!(true))
            .count(),
        302
    );
}
