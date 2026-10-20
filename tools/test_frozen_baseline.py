#!/usr/bin/env python3
"""Focused guards for the frozen binary runner."""

from __future__ import annotations

import io
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import frozen_baseline as runner


class RunnerGuards(unittest.TestCase):
    def test_archive_rejects_path_escape(self) -> None:
        payload = io.BytesIO()
        with tarfile.open(fileobj=payload, mode="w") as archive:
            member = tarfile.TarInfo("../escape")
            member.size = 1
            archive.addfile(member, io.BytesIO(b"x"))
        payload.seek(0)
        with tempfile.TemporaryDirectory() as temp:
            with tarfile.open(fileobj=payload, mode="r:") as archive:
                with self.assertRaisesRegex(runner.RunnerError, "unsafe archive path"):
                    runner.safe_extract(archive, Path(temp))
            self.assertFalse((Path(temp).parent / "escape").exists())

    def test_source_mutation_is_rejected_and_runtime_paths_are_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "root"
            source = root / "source"
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("original")
            (source / "snapshots").mkdir()
            (source / "snapshots" / "base.ansi").write_text("frozen")
            (source / "target").mkdir()
            (source / "target" / "runtime.txt").write_text("generated")
            manifest = {"source_commit": runner.BASELINE_COMMIT, "tag_object": runner.BASELINE_TAG_OBJECT, "source_tree": runner.BASELINE_TREE, "source_sha256": runner.digest_source(source), "snapshots_sha256": runner.digest_tree(source / "snapshots")}
            (root / "provenance.json").write_text(json.dumps(manifest))
            runner.verify_source(root, source)
            (source / "target" / "runtime.txt").write_text("new runtime artifact")
            runner.verify_source(root, source)
            (source / "Cargo.toml").write_text("mutated")
            with self.assertRaisesRegex(runner.RunnerError, "source was modified"):
                runner.verify_source(root, source)
            (source / "Cargo.toml").write_text("original")
            for mutation in ("changed", "deleted", "added"):
                snapshot = source / "snapshots" / "base.ansi"
                if mutation == "changed":
                    snapshot.write_text("mutated")
                elif mutation == "deleted":
                    snapshot.unlink()
                else:
                    (source / "snapshots" / "unexpected.ansi").write_text("mutated")
                with self.assertRaisesRegex(runner.RunnerError, "snapshots were modified"):
                    runner.verify_source(root, source)
                if mutation == "changed":
                    snapshot.write_text("frozen")
                elif mutation == "deleted":
                    snapshot.write_text("frozen")
                else:
                    (source / "snapshots" / "unexpected.ansi").unlink()

    def test_nextest_selection_requires_one_exact_ignored_case(self) -> None:
        listing = {
            "rust-suites": {
                "visual_baseline": {
                    "testcases": {
                        "holla::files::120x40::truecolor": {"ignored": True},
                        "tablepro::files::120x40::truecolor": {"ignored": True},
                        "store_integrity": {"ignored": False},
                    }
                }
            }
        }
        with patch.object(runner, "run", return_value=json.dumps(listing)):
            expression, count = runner.nextest_selection(Path("."), Path("."), "holla::files::120x40::truecolor", False, 30, {})
            self.assertEqual(expression, "test(=holla::files::120x40::truecolor)")
            self.assertEqual(count, 1)
            with self.assertRaisesRegex(runner.RunnerError, "selected zero"):
                runner.nextest_selection(Path("."), Path("."), "absent", False, 30, {})
            with self.assertRaisesRegex(runner.RunnerError, "not an ignored visual scenario"):
                runner.nextest_selection(Path("."), Path("."), "store_integrity", False, 30, {})

    def test_host_tool_mode_is_removed(self) -> None:
        with patch.dict("os.environ", {"VISUAL_TOOL": "capture", "VISUAL_PARTIAL_ORACLE_OK": "1", "MIDTEST_BUILD": "1", "COMBO_FILTER": "x", "UPDATE_BASELINE": "1", "NEXTTEST_FILTER": "x", "INSTA_UPDATE": "always", "PATH": "/bin"}, clear=True):
            self.assertEqual(runner.clean_env(), {"PATH": "/bin"})

    def test_phase_artifacts_are_separate_and_never_overwritten(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "root"
            source = root / "source"
            actual = source / "target" / "tuiscotti" / "actual"
            diff = source / "target" / "tuiscotti" / "diff"
            actual.mkdir(parents=True)
            diff.mkdir(parents=True)
            (actual / "frame.ansi").write_text("baseline")
            (diff / "delta.png").write_bytes(b"baseline-diff")
            baseline = runner.archive_artifacts(source, root, "baseline")
            (actual / "frame.ansi").write_text("candidate")
            candidate = runner.archive_artifacts(source, root, "candidate")
            self.assertEqual((baseline / "actual" / "frame.ansi").read_text(), "baseline")
            self.assertEqual((candidate / "actual" / "frame.ansi").read_text(), "candidate")
            with self.assertRaisesRegex(runner.RunnerError, "refusing to overwrite"):
                runner.archive_artifacts(source, root, "baseline")

    def test_exact_frame_comparator_rejects_cell_and_cursor_byte_change(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            baseline = root / "baseline"
            candidate = root / "candidate"
            for directory in (baseline, candidate):
                actual = directory / "actual" / "holla" / "scenario"
                actual.mkdir(parents=True)
                (actual / "frame.ansi").write_bytes(b"cells")
                (actual / "frame.frame.json").write_bytes(b"cells-and-cursor")
            self.assertEqual(runner.exact_frame_differences(baseline, candidate), [])
            (candidate / "actual" / "holla" / "scenario" / "frame.frame.json").write_bytes(b"cells-and-cursor-different")
            self.assertEqual(runner.exact_frame_differences(baseline, candidate), ["frame-bytes-differ:holla/scenario/frame.frame.json"])


if __name__ == "__main__":
    unittest.main()
