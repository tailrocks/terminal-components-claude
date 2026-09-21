#!/usr/bin/env python3
"""Unit tests for the test-disposition engine. Fixture-only; never approval."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import disposition
from disposition import Invalid


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


BASE = "d" * 40


def verdict(ident="t::x", verdict="preserve", **extra):
    package, kind, target, name = "p", "lib", "t", ident
    decision = {
        "id": extra.pop("id", "V-1"),
        "identity": {"package": package, "kind": kind, "target": target, "name": name},
        "path": "src/x.rs",
        "verdict": verdict,
        "spans": extra.pop("spans", []),
        "evidence": ["fixture"],
    }
    decision.update(extra)
    return decision


def listing(names=("t::x",)):
    return {
        "targets": [
            {
                "profile": "prof",
                "package": "p",
                "kind": "lib",
                "target": "t",
                "listed": list(names),
            }
        ]
    }


def conflicts(names=("t::x",)):
    return {
        "conflicts": [
            {
                "package": "p",
                "kind": "lib",
                "target": "t",
                "identity": name,
                "path": "src/x.rs",
            }
            for name in names
        ]
    }


def stage():
    return {
        "schema": 1,
        "base_commit": BASE,
        "entries": [{"owner": "TASK-021", "evidence": ["fixture"]}],
    }


def manifest(patches):
    return {"schema": 1, "base_commit": BASE, "patches": patches}


def patch(pid="P-1", old="old bytes\n", new="new bytes\n", **extra):
    entry = {
        "id": pid,
        "path": "src/x.rs",
        "verdict_ref": "V-1",
        "span_keys": ["A1"],
        "old": old,
        "old_sha256": sha(old.encode()),
        "new": new,
        "new_sha256": sha(new.encode()),
    }
    entry.update(extra)
    return entry


class DecideTests(unittest.TestCase):
    def run_decide(self, directory, decisions, patches, names=("t::x",), **kw):
        root = Path(directory)
        (root / "verdicts.json").write_text(
            json.dumps(
                {
                    "schema": 1,
                    "base_commit": BASE,
                    "rules": [],
                    "decisions": decisions,
                    "reviews": kw.get("reviews", []),
                }
            )
        )
        (root / "stage.json").write_text(json.dumps(stage()))
        (root / "manifest.json").write_text(json.dumps(manifest(patches)))
        (root / "conflicts.json").write_text(json.dumps(conflicts(kw.get("conflict_names", names))))
        (root / "listing.json").write_text(json.dumps(listing(names)))
        import argparse

        ns = argparse.Namespace(
            root=str(root),
            verdicts=str(root / "verdicts.json"),
            stage=str(root / "stage.json"),
            manifest=str(root / "manifest.json"),
            conflicts=str(root / "conflicts.json"),
            listing=str(root / "listing.json"),
            output=str(root / "out.json"),
            expand_tsv=str(root / "out.tsv") if kw.get("expand") else None,
        )
        code = disposition.cmd_decide(ns)
        self.assertEqual(code, 0)
        return json.loads((root / "out.json").read_text())

    def test_preserve_all(self):
        with tempfile.TemporaryDirectory() as directory:
            out = self.run_decide(
                directory,
                [verdict(spans=[{"key": "A1", "disposition": "preserve", "evidence": ["e"]}])],
                [patch()],
            )
            self.assertEqual(out["counts"]["listing_identities"], 1)
            self.assertEqual(out["counts"]["bulk_preserved"], 0)
            self.assertEqual(out["counts"]["relocations"], 0)

    def test_missing_conflict_decision_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(Invalid):
                self.run_decide(directory, [], [patch()], conflict_names=("t::y",))

    def test_non_preserve_span_without_patch_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            decisions = [
                verdict(
                    verdict="replace",
                    spans=[{"key": "A1", "disposition": "replace", "evidence": ["e"]}],
                )
            ]
            with self.assertRaises(Invalid):
                self.run_decide(directory, decisions, [])

    def test_relocate_requires_recorded_replacement(self):
        with tempfile.TemporaryDirectory() as directory:
            decisions = [verdict(verdict="relocate", spans=[])]
            with self.assertRaises(Invalid):
                self.run_decide(directory, decisions, [patch()])

    def test_relocate_to_listed_name_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            decisions = [
                verdict(
                    id="V-1",
                    ident="t::old",
                    verdict="relocate",
                    replacement_identity={
                        "package": "p",
                        "kind": "lib",
                        "target": "t",
                        "name": "t::old",
                    },
                    spans=[{"key": "N", "disposition": "relocate", "evidence": ["e"]}],
                )
            ]
            with self.assertRaises(Invalid):
                self.run_decide(
                    directory, decisions, [patch()], names=("t::old",), conflict_names=()
                )

    def test_unknown_manifest_verdict_ref_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(Invalid):
                self.run_decide(
                    directory,
                    [verdict()],
                    [patch(verdict_ref="V-9")],
                )

    def test_review_identity_must_exist_in_listing(self):
        with tempfile.TemporaryDirectory() as directory:
            reviews = [
                {
                    "id": "R-1",
                    "evidence": ["e"],
                    "target": {
                        "package": "p",
                        "kind": "lib",
                        "target": "t",
                        "prefix": "t::",
                    },
                    "tests": [{"name": "ghost"}],
                }
            ]
            with self.assertRaises(Invalid):
                self.run_decide(directory, [verdict()], [patch()], reviews=reviews)

    def test_expand_tsv_covers_every_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.run_decide(
                directory,
                [verdict(spans=[{"key": "A1", "disposition": "preserve", "evidence": ["e"]}])],
                [patch()],
                names=("t::x", "t::y"),
                conflict_names=("t::x",),
                expand=True,
            )
            rows = (root / "out.tsv").read_text().splitlines()
            self.assertEqual(len(rows), 3)
            self.assertTrue(rows[1].endswith("preserve\tV-1"))
            self.assertTrue(rows[2].endswith("preserve\t"))


class ApplyTests(unittest.TestCase):
    def test_apply_replaces_exact_span(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("head\nold bytes\ntail\n")
            (root / "manifest.json").write_text(json.dumps(manifest([patch()])))
            import argparse

            ns = argparse.Namespace(root=str(root), manifest=str(root / "manifest.json"))
            self.assertEqual(disposition.cmd_apply(ns), 0)
            self.assertEqual(
                (root / "src" / "x.rs").read_text(), "head\nnew bytes\ntail\n"
            )

    def test_apply_rejects_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("head\nold bytes!\ntail\n")
            (root / "manifest.json").write_text(json.dumps(manifest([patch()])))
            import argparse

            ns = argparse.Namespace(root=str(root), manifest=str(root / "manifest.json"))
            with self.assertRaises(Invalid):
                disposition.cmd_apply(ns)

    def test_apply_rejects_ambiguity_and_overlap(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("old bytes\nold bytes\n")
            (root / "manifest.json").write_text(json.dumps(manifest([patch()])))
            import argparse

            ns = argparse.Namespace(root=str(root), manifest=str(root / "manifest.json"))
            with self.assertRaises(Invalid):
                disposition.cmd_apply(ns)

    def test_apply_checks_hashes_and_offsets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("old bytes\n")
            bad = patch(old_sha256="0" * 64)
            (root / "manifest.json").write_text(json.dumps(manifest([bad])))
            import argparse

            ns = argparse.Namespace(root=str(root), manifest=str(root / "manifest.json"))
            with self.assertRaises(Invalid):
                disposition.cmd_apply(ns)
            bad = patch(old_start=99)
            (root / "manifest.json").write_text(json.dumps(manifest([bad])))
            with self.assertRaises(Invalid):
                disposition.cmd_apply(ns)


class VerifyTests(unittest.TestCase):
    def init_repo(self, root: Path):
        env = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
        import os

        full = dict(os.environ)
        full.update(env)
        subprocess.run(["git", "init", "-q"], cwd=root, check=True, env=full)
        subprocess.run(["git", "add", "-A"], cwd=root, check=True, env=full)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True, env=full)
        return subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root, capture_output=True, text=True, check=True
        ).stdout.strip()

    def test_verify_accepts_manifest_exact_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("old bytes\n")
            base = self.init_repo(root)
            archive = root / "archives" / "test-authority"
            (archive / "src").mkdir(parents=True)
            (archive / "src" / "x.rs").write_text("old bytes\n")
            (archive / "manifest.json").write_text(
                json.dumps(
                    {
                        "base_commit": base,
                        "files": [
                            {
                                "path": "src/x.rs",
                                "sha256": sha(b"old bytes\n"),
                            }
                        ],
                    }
                )
            )
            tool = root / "tools" / "test-disposition"
            tool.mkdir(parents=True)
            (tool / "mf.json").write_text(
                json.dumps(manifest([patch()]) | {"base_commit": base})
            )
            (root / "src" / "x.rs").write_text("new bytes\n")
            import argparse

            ns = argparse.Namespace(
                root=str(root),
                manifest=str(tool / "mf.json"),
                archive=str(archive),
            )
            self.assertEqual(disposition.cmd_verify(ns), 0)

    def test_verify_rejects_unexpected_changed_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("old bytes\n")
            (root / "src" / "other.rs").write_text("untouched\n")
            base = self.init_repo(root)
            archive = root / "archives" / "test-authority"
            (archive / "src").mkdir(parents=True)
            (archive / "src" / "x.rs").write_text("old bytes\n")
            (archive / "manifest.json").write_text(
                json.dumps(
                    {
                        "base_commit": base,
                        "files": [{"path": "src/x.rs", "sha256": sha(b"old bytes\n")}],
                    }
                )
            )
            tool = root / "tools" / "test-disposition"
            tool.mkdir(parents=True)
            (tool / "mf.json").write_text(
                json.dumps(manifest([patch()]) | {"base_commit": base})
            )
            (root / "src" / "x.rs").write_text("new bytes\n")
            (root / "src" / "other.rs").write_text("touched\n")
            import argparse

            ns = argparse.Namespace(
                root=str(root),
                manifest=str(tool / "mf.json"),
                archive=str(archive),
            )
            with self.assertRaises(Invalid):
                disposition.cmd_verify(ns)

    def test_verify_rejects_outside_span_edit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src" / "x.rs").write_text("old bytes\n")
            base = self.init_repo(root)
            archive = root / "arch"
            (archive / "src").mkdir(parents=True)
            (archive / "src" / "x.rs").write_text("old bytes\n")
            (archive / "manifest.json").write_text(
                json.dumps(
                    {
                        "base_commit": base,
                        "files": [{"path": "src/x.rs", "sha256": sha(b"old bytes\n")}],
                    }
                )
            )
            (root / "mf.json").write_text(
                json.dumps(manifest([patch()]) | {"base_commit": base})
            )
            (root / "src" / "x.rs").write_text("new bytes\nEXTRA\n")
            import argparse

            ns = argparse.Namespace(
                root=str(root), manifest=str(root / "mf.json"), archive=str(archive)
            )
            with self.assertRaises(Invalid):
                disposition.cmd_verify(ns)


if __name__ == "__main__":
    unittest.main()
