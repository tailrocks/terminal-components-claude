#!/usr/bin/env python3
"""Historical obligation mapping proofs. Mapping is not execution or approval."""
import json
import tempfile
import unittest
from pathlib import Path

from inventory import Invalid, digest, encoded
import reconcile as rec


def identity(package, name, kind="lib", target=None, classification="preserve"):
    target = target or package.replace("-", "_")
    return {
        "package": package,
        "kind": kind,
        "target": target,
        "identity": name,
        "path": f"{package}/src/lib.rs",
        "line": 1,
        "origin": "fn-test",
        "classification": classification,
        "ignored": None,
        "blob_sha256": "0" * 64,
        "macro": None,
        "assertions": [{"kind": "assert", "line": 2, "byte_start": 0, "byte_end": 1,
                        "classification": classification}],
        "byte_start": 0,
        "byte_end": 10,
        "end_line": 3,
    }


def catalog(n=3211):
    obligations = []
    for i in range(n):
        obligations.append({
            "source_id": f"origin:{i:04d}",
            "origin": "initial-repaired-main",
            "target": "Running unittests src/lib.rs [junie_tui-deadbeef]",
            "identity": f"theme::tests::case_{i}",
        })
    return {"schema": 1, "classification": "historical-obligations-not-current-acceptance",
            "obligations": obligations}


def required_from(historical):
    return {
        "schema": 1,
        "approval": "pending",
        "catalog_sha256": digest(encoded(historical)),
        "profiles": [],
        "targets": [],
        "obligations": [{"source_id": row["source_id"], "review": None, "destinations": []}
                        for row in historical["obligations"]],
    }


def canonical(n=620):
    rows = [{"id": f"A{i:03d}", "tests_gates": "unit"} for i in range(n)]
    return rows


class Mapping(unittest.TestCase):
    def test_exact_identity_maps_and_duplicate_origins_stay_independent(self):
        historical = catalog(2)
        historical["obligations"][1]["origin"] = "publication-parent"
        historical["obligations"][1]["identity"] = historical["obligations"][0]["identity"]
        discovery = {"identities": [identity("junie-tui", "theme::tests::case_0")], "blockers": []}
        required = required_from(historical)
        result = rec.reconcile(historical, discovery, canonical(), required)
        rec.refuse_approval(result)
        self.assertEqual(result["approval"], "pending")
        self.assertFalse(result["complete_matrix"])
        self.assertEqual(result["matched"], 2)
        self.assertEqual(result["unresolved"], 0)
        self.assertEqual(result["obligations"][0]["source_id"], "origin:0000")
        self.assertEqual(result["obligations"][1]["source_id"], "origin:0001")
        self.assertEqual(result["obligations"][0]["destinations"][0][4], "theme::tests::case_0")
        self.assertIsNone(result["obligations"][0]["review"])

    def test_unapproved_relocation_stays_unresolved(self):
        historical = catalog(1)
        historical["obligations"][0]["identity"] = "widgets::brand::tests::clickable"
        discovery = {"identities": [identity("junie-tui", "components::brand::tests::clickable")],
                     "blockers": []}
        result = rec.reconcile(historical, discovery, canonical(), required_from(historical))
        self.assertEqual(result["obligations"][0]["status"], "unapproved-relocation")
        self.assertEqual(result["obligations"][0]["destinations"], [])
        self.assertEqual(result["unresolved"], 1)

    def test_missing_name_is_unresolved_not_dropped(self):
        historical = catalog(1)
        result = rec.reconcile(historical, {"identities": [], "blockers": []},
                               canonical(), required_from(historical))
        self.assertEqual(len(result["obligations"]), 1)
        self.assertEqual(result["obligations"][0]["status"], "unresolved")
        self.assertEqual(result["matched"], 0)

    def test_canonical_union_must_be_620(self):
        historical = catalog(1)
        with self.assertRaises(Invalid):
            rec.reconcile(historical, {"identities": [], "blockers": []},
                          canonical(619), required_from(historical))

    def test_cannot_approve_or_fill_targets(self):
        historical = catalog(1)
        required = required_from(historical)
        required["approval"] = "reviewed"
        with self.assertRaises(Invalid):
            rec.reconcile(historical, {"identities": [], "blockers": []}, canonical(), required)
        result = rec.reconcile(historical, {"identities": [], "blockers": []},
                               canonical(), required_from(historical))
        result["approval"] = "reviewed"
        with self.assertRaises(Invalid):
            rec.refuse_approval(result)

    def test_conflicts_are_emitted_for_task_008(self):
        historical = catalog(1)
        row = identity("holla", "app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts",
                       classification="oracle-conflict")
        result = rec.reconcile(historical, {"identities": [row], "blockers": []},
                               canonical(), required_from(historical))
        self.assertEqual(len(result["conflicts"]), 1)
        self.assertEqual(result["conflicts"][0]["classification"], "oracle-conflict")
        self.assertEqual(result["conflicts"][0]["identity"], row["identity"])

    def test_historical_count_is_exact(self):
        historical = catalog(2)
        required = required_from(historical)
        required["obligations"].pop()
        with self.assertRaises(Invalid):
            rec.reconcile(historical, {"identities": [], "blockers": []},
                          canonical(), required)

    def test_write_refuses_existing_and_keeps_pending(self):
        historical = catalog(1)
        result = rec.reconcile(historical, {"identities": [], "blockers": []},
                               canonical(), required_from(historical))
        with tempfile.TemporaryDirectory() as scratch:
            output = Path(scratch) / "out.json"
            rec.write_reconcile(result, output)
            self.assertEqual(json.loads(output.read_text())["approval"], "pending")
            with self.assertRaises(Invalid):
                rec.write_reconcile(result, output)


class RepoUnion(unittest.TestCase):
    def test_current_required_stays_pending_and_canonical_is_620(self):
        root = Path(__file__).resolve().parents[2]
        required = json.loads((root / "tools/test-inventory/required.json").read_text())
        self.assertEqual(required["approval"], "pending")
        canonical_path = root / "docs/refactoring-plan/historical-obligations-canonical.tsv"
        if not canonical_path.is_file():
            self.skipTest("canonical union not present")
        rows = rec.load_canonical(canonical_path)
        self.assertEqual(len(rows), 620)
        historical = json.loads((root / "tools/test-inventory/historical.json").read_text())
        self.assertEqual(len(historical["obligations"]), 3211)
        self.assertEqual(required["catalog_sha256"], digest(encoded(historical)))


if __name__ == "__main__":
    unittest.main()
