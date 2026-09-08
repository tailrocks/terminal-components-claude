#!/usr/bin/env python3
"""Mutation proofs plus a real Cargo/libtest/rustdoc fixture, no dependencies."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import inventory as gate


PROFILE = {"id": "fixture-all", "package": "inventory-fixture", "default_features": True,
           "features": [], "all_features": True}
TOOLCHAIN = os.environ.get("INVENTORY_TEST_TOOLCHAIN", "1.88.0")


class Parsers(unittest.TestCase):
    def test_missing_and_duplicate_list_names_fail(self):
        for text in ("a: test\n2 tests, 0 benchmarks\n", "a: test\na: test\n2 tests, 0 benchmarks\n"):
            with self.assertRaises(gate.Invalid):
                gate.listed(text)

    def test_zero_filter_and_duplicate_execution_fail(self):
        for text in (
            "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0s",
            "test a ... ok\ntest a ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s",
        ):
            with self.assertRaises(gate.Invalid):
                gate.executed(text, ["a"])

    def test_duplicate_profile_command_fails(self):
        duplicate = dict(PROFILE, id="renamed")
        with self.assertRaises(gate.Invalid):
            gate.validate_profiles([PROFILE, duplicate])


class CargoContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="inventory-proof-")
        cls.base = Path(cls.temp.name)
        cls.root = cls.base / "repo"
        cls.root.mkdir()
        files = {
            "Cargo.toml": '[package]\nname="inventory-fixture"\nversion="0.0.0"\nedition="2024"\n[features]\nextra=[]\n[workspace]\n',
            "src/lib.rs": '''/// Example.
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
/// ```compile_fail
/// let x: bool = 42;
/// ```
pub fn example() {}
#[test] fn library() {}
#[test] #[cfg(feature="extra")] fn gated() {}
''',
            "src/main.rs": 'fn main() {}\n#[test] fn binary_local() {}\n',
            "tests/contract.rs": '#[test] fn integration() {}\n#[test] #[ignore="subprocess helper"] fn helper() {}\n',
            ".gitignore": "target/\n",
        }
        for name, text in files.items():
            p = cls.root / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text)
        subprocess.run(["git", "init", "-q"], cwd=cls.root, check=True, capture_output=True)
        subprocess.run(["git", "add", "."], cwd=cls.root, check=True, capture_output=True)
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=cls.root, check=True, capture_output=True)
        cls.capture = gate.capture(cls.root, [PROFILE], cls.base / "capture.json", True, TOOLCHAIN)
        cls.required = {"schema": 1, "approval": "reviewed", "profiles": [PROFILE], "targets": [],
                        "obligations": []}
        for row in cls.capture["targets"]:
            target = {k: row[k] for k in ("profile", "package", "kind", "target")}
            target.update({"identities": row["listed"], "ignored": {"helper": "fixture subprocess helper"}
                           if "helper" in row["listed"] else {}})
            cls.required["targets"].append(target)
            for name in row["listed"]:
                cls.required["obligations"].append({"source_id": str((row["kind"], name)),
                    "review": "synthetic fixture only", "destinations": [[row[k] for k in
                    ("profile", "package", "kind", "target")] + [name]]})
        cls.catalog = {"obligations": [{"source_id": r["source_id"]} for r in cls.required["obligations"]]}
        cls.required["catalog_sha256"] = gate.digest(gate.encoded(cls.catalog))

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_real_binary_integration_feature_and_docs_are_executed(self):
        gate.verify(self.capture, self.required, self.catalog)
        self.assertEqual({r["kind"] for r in self.capture["targets"]}, {"lib", "bin", "test", "doc"})
        names = [n for r in self.capture["targets"] for n in r["listed"]]
        self.assertIn("gated", names)
        self.assertEqual(len(names), 7)

    def test_required_missing_feature_doc_or_target_fails(self):
        for kind in ("lib", "bin", "test", "doc"):
            cap = copy.deepcopy(self.capture)
            cap["targets"] = [r for r in cap["targets"] if r["kind"] != kind]
            with self.assertRaises(gate.Invalid):
                gate.verify(cap, self.required, self.catalog)
        cap = copy.deepcopy(self.capture)
        lib = next(r for r in cap["targets"] if r["kind"] == "lib")
        lib["listed"].remove("gated")
        with self.assertRaises(gate.Invalid):
            gate.verify(cap, self.required, self.catalog)

    def test_duplicate_target_and_unexecuted_capture_fail(self):
        cap = copy.deepcopy(self.capture)
        cap["targets"].append(copy.deepcopy(cap["targets"][0]))
        with self.assertRaises(gate.Invalid):
            gate.verify(cap, self.required, self.catalog)
        cap = copy.deepcopy(self.capture)
        cap["targets"][0]["executed"] = None
        with self.assertRaises(gate.Invalid):
            gate.verify(cap, self.required, self.catalog)

    def test_ignored_test_needs_review_and_mapping_cannot_disappear(self):
        required = copy.deepcopy(self.required)
        for row in required["targets"]:
            row["ignored"] = {}
        with self.assertRaises(gate.Invalid):
            gate.verify(self.capture, required, self.catalog)
        required = copy.deepcopy(self.required)
        required["obligations"][0]["destinations"] = []
        with self.assertRaises(gate.Invalid):
            gate.verify(self.capture, required, self.catalog)
        required["approval"] = "unreviewed"
        with self.assertRaises(gate.Invalid):
            gate.verify(self.capture, required, self.catalog)
        required = copy.deepcopy(self.required)
        required["obligations"].pop()
        with self.assertRaises(gate.Invalid):
            gate.verify(self.capture, required, self.catalog)

    def test_custom_harness_and_stale_output_fail_closed(self):
        with self.assertRaises(gate.Invalid):
            gate.capture(self.root, [PROFILE], self.base / "capture.json", True, TOOLCHAIN)
        manifest = self.root / "Cargo.toml"
        original = manifest.read_text()
        try:
            manifest.write_text(original + '\n[[test]]\nname="custom"\nharness=false\n')
            (self.root / "tests/custom.rs").write_text('fn main() {}\n')
            cap = gate.capture(self.root, [PROFILE], self.base / "custom.json", False, TOOLCHAIN)
            self.assertTrue(any("custom harness" in b["reason"] for b in cap["blocked"]))
            with self.assertRaises(gate.Invalid):
                gate.verify(cap, self.required, self.catalog)
        finally:
            manifest.write_text(original)
            (self.root / "tests/custom.rs").unlink()


if __name__ == "__main__":
    unittest.main()
