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
    def extension_manifest(self) -> dict[str, object]:
        journey = {"id": "sort", "title": "Sort table", "checkpoints": ["initial", "sorted"]}
        sizes = [{"cols": cols, "rows": rows, "allow_narrow_fallback": (cols, rows) in {(40, 12), (60, 18)}} for cols, rows in runner.EXTENSION_SIZES]
        cases = []
        for cols, rows in runner.EXTENSION_SIZES:
            modes = ("truecolor", "nocolor") if (cols, rows) in {(40, 12), (60, 18)} else runner.EXTENSION_COLORS
            checkpoints = ["initial", "narrow_revealed_80x24", "sorted"] if (cols, rows) in {(40, 12), (60, 18)} else ["initial", "sorted"]
            for color in modes:
                cases.append({"id": f"{runner.EXTENSION_CASE_PREFIX}sort::{cols}x{rows}::{color}", "journey_id": "sort", "cols": cols, "rows": rows, "color_mode": color, "checkpoints": checkpoints})
        return {"schema_version": 1, "extension_id_prefix": runner.EXTENSION_CASE_PREFIX, "journeys": [journey], "sizes": sizes, "color_modes": list(runner.EXTENSION_COLORS), "generated_cases": cases}

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
        with patch.dict("os.environ", {"VISUAL_TOOL": "capture", "VISUAL_PARTIAL_ORACLE_OK": "1", "TERMROCK_PAIR_PHASE": "candidate-check", "TERMROCK_PAIR_EXPECTED_ROOT": "x", "TERMROCK_PAIR_RECEIPT_ROOT": "y", "MIDTEST_BUILD": "1", "COMBO_FILTER": "x", "UPDATE_BASELINE": "1", "NEXTTEST_FILTER": "x", "INSTA_UPDATE": "always", "PATH": "/bin"}, clear=True):
            self.assertEqual(runner.clean_env(), {"PATH": "/bin"})

    def test_live_nextest_events_require_each_exact_selection_to_pass(self) -> None:
        selected = ["holla::files::120x40::truecolor", "tablepro::files::120x40::truecolor"]
        prefix = "junie-tui::visual_baseline$"
        output = "\n".join(
            [json.dumps({"type": "test", "event": "ignored", "name": prefix + "unselected::scenario"})]
            + [json.dumps({"type": "test", "event": "ok", "name": prefix + name}) for name in selected]
        )
        self.assertEqual(len(runner.nextest_events(output, selected)), 2)
        with self.assertRaisesRegex(runner.RunnerError, "coverage mismatch"):
            runner.nextest_events(output.splitlines()[0], selected)
        failed = json.dumps({"type": "test", "event": "failed", "name": prefix + selected[0]})
        with self.assertRaisesRegex(runner.RunnerError, "did not all pass"):
            runner.nextest_events(failed, [selected[0]])

    def test_live_receipts_bind_case_order_name_and_frame_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = "holla/files/120x40/truecolor"
            logical = "holla/files/120x40/truecolor"
            expected = root / "expected"
            receipt_root = root / "receipts"
            expected.mkdir()
            ansi = b"frame\x1b[?25h"
            identity = runner.checkpoint_identity(case, 0, logical)
            frame_name = f"checkpoint-00000000-{identity}.ansi"
            (expected / frame_name).write_bytes(ansi)
            receipt_dir = receipt_root / runner.hashlib.sha256(case.encode()).hexdigest()
            receipt_dir.mkdir(parents=True)
            receipt = {
                "case_id": case,
                "ordinal": 0,
                "logical_name": logical,
                "sha256": runner.hashlib.sha256(ansi).hexdigest(),
            }
            (receipt_dir / f"checkpoint-00000000-{identity}.json").write_text(json.dumps(receipt))
            found = runner.read_receipts(receipt_root, [case], expected)
            self.assertEqual(found, {(case, 0, logical): receipt["sha256"]})
            (expected / frame_name).write_bytes(b"changed")
            with self.assertRaisesRegex(runner.RunnerError, "hash mismatch"):
                runner.read_receipts(receipt_root, [case], expected)

    def test_checkpoint_paths_remain_below_portable_filename_limit(self) -> None:
        case = "holla/activities/activities-multi-with-history/120x40/truecolor"
        logical = "holla/activities/activities-multi-with-history/120x40/truecolor"
        identity = runner.checkpoint_identity(case, 0, logical)
        filename = f"checkpoint-00000000-{identity}.ansi"
        self.assertLessEqual(len(filename.encode()), 255)
        self.assertEqual(len(identity), 64)

    def test_sealed_runtime_store_rejects_symlinks_and_nested_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "store"
            root.mkdir()
            target = Path(temp) / "outside.ansi"
            target.write_bytes(b"outside")
            (root / "case.ansi").symlink_to(target)
            with self.assertRaisesRegex(runner.RunnerError, "symlink"):
                runner.sealed_tree(root)
            (root / "case.ansi").unlink()
            (root / "nested").mkdir()
            with self.assertRaisesRegex(runner.RunnerError, "unexpected directory"):
                runner.flat_sealed_store(root, "runtime expected")

    def test_live_chunks_are_deterministic_and_bounded(self) -> None:
        names = [f"case::{index}" for index in range(5)]
        self.assertEqual(runner.scenario_chunks(names, 2), [names[:2], names[2:4], names[4:]])
        with self.assertRaisesRegex(runner.RunnerError, "chunk size"):
            runner.scenario_chunks(names, 513)

    def test_chunk_coverage_rejects_missing_duplicate_skipped_and_repartitioned(self) -> None:
        selected = [f"case::{index}" for index in range(5)]
        good = [
            {"chunk": 0, "selected_cases": selected[:2], "status": "passed"},
            {"chunk": 1, "selected_cases": selected[2:4], "status": "passed"},
            {"chunk": 2, "selected_cases": selected[4:], "status": "passed"},
        ]
        runner.validate_chunk_coverage(selected, good, 2)
        for records in (
            good[:-1],
            [good[0], good[0], good[2]],
            [good[0], {**good[1], "status": "skipped"}, good[2]],
            [good[0], {**good[1], "status": "unfinished"}, good[2]],
            [good[0], {**good[1], "selected_cases": [selected[2]]}, good[2]],
        ):
            with self.subTest(records=records), self.assertRaises(runner.RunnerError):
                runner.validate_chunk_coverage(selected, records, 2)

    def test_empty_receipts_fail_closed_for_selected_case(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipts = root / "receipts"
            expected = root / "expected"
            receipts.mkdir()
            expected.mkdir()
            with self.assertRaisesRegex(runner.RunnerError, "missing checkpoint receipts"):
                runner.read_receipts(receipts, ["showcase/pages/overview"], expected)

    def test_registered_inventory_digest_must_match_prepared_source(self) -> None:
        inventory = {"scenario": True, "ordinary_check": False}
        runner.require_inventory_matches(inventory, runner.inventory_digest(inventory))
        with self.assertRaisesRegex(runner.RunnerError, "Nextest inventory hash mismatch"):
            runner.require_inventory_matches(inventory, "0" * 64)

    def test_extension_manifest_binds_exact_disjoint_cases_and_checkpoint_order(self) -> None:
        manifest = self.extension_manifest()
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "extensions.json"
            path.write_text(json.dumps(manifest))
            validated = runner.load_extension_manifest(path)
        base = {f"base::{index}": True for index in range(runner.EXPECTED_SCENARIOS)}
        base.update({f"ordinary::{index}": False for index in range(runner.EXPECTED_ORDINARY_TESTS)})
        extension_ids = [case["id"] for case in manifest["generated_cases"]]
        inventory = {**base, **{name: True for name in extension_ids}}
        original, extension_ids = runner.validate_extension_inventory(inventory, validated)
        self.assertEqual(original, base)
        self.assertEqual(extension_ids, sorted(extension_ids))
        for invalid in (
            {**base, **{name: True for name in extension_ids[:-1]}, extension_ids[-1]: False},
            {**base, "unexpected::extra": True, **{name: True for name in extension_ids}},
            {**{key: value for key, value in base.items() if key != "base::0"}, **{name: True for name in extension_ids}},
        ):
            with self.subTest(inventory_size=len(invalid)), self.assertRaises(runner.RunnerError):
                runner.validate_extension_inventory(invalid, validated)
        manifest["generated_cases"] = manifest["generated_cases"][:-1]
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "extensions.json"
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(runner.RunnerError, "matrix incomplete"):
                runner.load_extension_manifest(path)

    def test_extension_manifest_rejects_duplicate_ids_and_unordered_obligations(self) -> None:
        manifest = self.extension_manifest()
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "extensions.json"
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(runner.RunnerError, "duplicate extension case"):
                duplicate = dict(manifest)
                duplicate["generated_cases"] = [*manifest["generated_cases"], dict(manifest["generated_cases"][0])]
                path.write_text(json.dumps(duplicate))
                runner.load_extension_manifest(path)
            cases = list(manifest["generated_cases"])
            cases[0] = {**cases[0], "checkpoints": ["sorted", "initial"]}
            manifest["generated_cases"] = cases
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(runner.RunnerError, "checkpoint obligations"):
                runner.load_extension_manifest(path)

    def test_extension_receipts_must_match_ordered_manifest_checkpoints(self) -> None:
        name = runner.EXTENSION_CASE_PREFIX + "sort::80x24::truecolor"
        case = runner.scenario_case_id(name)
        manifest = {"generated_case_ids": [name], "checkpoint_obligations": {name: ["before", "sorted"]}}
        receipts = {(case, 0, "before"): "1" * 64, (case, 1, "sorted"): "2" * 64}
        self.assertEqual(runner.validate_extension_checkpoint_receipts([name], receipts, manifest), [])
        with self.assertRaisesRegex(runner.RunnerError, "checkpoint obligations differ"):
            runner.validate_extension_checkpoint_receipts([name], {(case, 0, "sorted"): "1" * 64}, manifest)

    def test_ordinary_inventory_separates_cli_cases_from_source_checks(self) -> None:
        cli = sorted(runner.EXPECTED_CLI_ORDINARY_TESTS)
        source_only_names = [f"source_check::{index}" for index in range(runner.EXPECTED_ORDINARY_TESTS - len(cli))]
        inventory = {name: False for name in [*cli, *source_only_names]}
        source_only, actual_cli = runner.ordinary_test_groups(inventory)
        self.assertEqual(source_only, sorted(source_only_names))
        self.assertEqual(actual_cli, cli)
        self.assertEqual((len(source_only), len(actual_cli)), (11, 9))
        self.assertIn("isolation::env_isolation", actual_cli)
        self.assertNotIn("isolation::env_isolation", source_only)
        inventory.pop(cli[0])
        inventory["unknown::cli_test"] = False
        with self.assertRaisesRegex(runner.RunnerError, "CLI ordinary-check inventory changed"):
            runner.ordinary_test_groups(inventory)

    def test_ordinary_cli_checks_run_with_both_explicit_binary_dirs(self) -> None:
        cli = sorted(runner.EXPECTED_CLI_ORDINARY_TESTS)
        source_only_names = [f"source_check::{index}" for index in range(runner.EXPECTED_ORDINARY_TESTS - len(cli))]
        inventory = {name: False for name in [*cli, *source_only_names]}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            live = root / "live"
            live.mkdir()
            run_root = root / "run"
            run_root.mkdir()
            baseline_bins = root / "baseline-bins"
            candidate_bins = root / "candidate-bins"
            baseline_bins.mkdir()
            candidate_bins.mkdir()
            calls: list[dict[str, object]] = []
            selected_groups = [sorted(source_only_names), cli, cli]

            def fake_run(command, *, cwd, timeout, env, quiet, stdout_path, stderr_path):
                group = len(calls)
                calls.append({"command": command, "env": env})
                if group == 0:
                    self.assertNotIn("isolation::env_isolation", " ".join(command))
                else:
                    expected_bins = baseline_bins if group == 1 else candidate_bins
                    self.assertIn("isolation::env_isolation", " ".join(command))
                    self.assertEqual(env.get("VISUAL_BIN_DIR"), str(expected_bins.resolve()))
                events = [json.dumps({"type": "test", "event": "ok", "name": "junie-tui::visual_baseline$" + name}) for name in selected_groups[group]]
                stdout_path.write_text("\n".join(events) + "\n")
                stderr_path.write_text("")
                return "\n".join(events)

            with patch.object(runner, "run", side_effect=fake_run):
                result = runner.run_ordinary_checks(live, root, run_root, inventory, baseline_bins, candidate_bins, 30)
            self.assertEqual(set(result), {"source-only", "baseline-cli", "candidate-cli"})
            self.assertNotIn("VISUAL_BIN_DIR", calls[0]["env"])
            self.assertEqual(calls[1]["env"]["VISUAL_BIN_DIR"], str(baseline_bins.resolve()))
            self.assertEqual(calls[2]["env"]["VISUAL_BIN_DIR"], str(candidate_bins.resolve()))
            for call in calls:
                self.assertFalse(any(key.startswith("TERMROCK_PAIR_") for key in call["env"]))

    def test_candidate_environment_isolation_failure_cannot_pass_green(self) -> None:
        cli = sorted(runner.EXPECTED_CLI_ORDINARY_TESTS)
        source_only_names = [f"source_check::{index}" for index in range(runner.EXPECTED_ORDINARY_TESTS - len(cli))]
        inventory = {name: False for name in [*cli, *source_only_names]}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            live = root / "live"
            live.mkdir()
            run_root = root / "run"
            run_root.mkdir()
            baseline_bins = root / "baseline-bins"
            candidate_bins = root / "candidate-bins"
            baseline_bins.mkdir()
            candidate_bins.mkdir()
            calls = 0

            def fake_run(command, *, cwd, timeout, env, quiet, stdout_path, stderr_path):
                nonlocal calls
                group = calls
                calls += 1
                selected = source_only_names if group == 0 else cli
                if group == 0:
                    self.assertNotIn("isolation::env_isolation", " ".join(command))
                    self.assertNotIn("VISUAL_BIN_DIR", env)
                else:
                    self.assertIn("isolation::env_isolation", " ".join(command))
                    expected_bins = baseline_bins if group == 1 else candidate_bins
                    self.assertEqual(env.get("VISUAL_BIN_DIR"), str(expected_bins.resolve()))
                events = [
                    json.dumps({
                        "type": "test",
                        "event": "failed" if group == 2 and name == "isolation::env_isolation" else "ok",
                        "name": "junie-tui::visual_baseline$" + name,
                    })
                    for name in selected
                ]
                stdout_path.write_text("\n".join(events) + "\n")
                stderr_path.write_text("")
                return "\n".join(events)

            with patch.object(runner, "run", side_effect=fake_run):
                with self.assertRaisesRegex(runner.RunnerError, "ordinary candidate-cli checks failed"):
                    runner.run_ordinary_checks(live, root, run_root, inventory, baseline_bins, candidate_bins, 30)
            candidate_report = json.loads((run_root / "ordinary" / "candidate-cli" / "report.json").read_text())
            self.assertEqual(candidate_report["status"], "failed")
            self.assertIn("isolation::env_isolation", candidate_report["selected_cases"])
            self.assertEqual(candidate_report["binary_dir"], str(candidate_bins.resolve()))

    def test_live_source_provenance_rejects_binary_harness_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            live = root / "live-source"
            module = live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"
            module.parent.mkdir(parents=True)
            module.write_text("module")
            (root / "live-source.json").write_text(json.dumps({
                "frozen_source_sha256": "a",
                "live_source_sha256": "b",
                "patch_sha256": "b",
                "helper_sha256": "different-helper",
                "module_sha256": "module-hash",
                "frozen_snapshots_sha256": "d",
                "live_snapshots_sha256": "d",
            }))
            repo = root / "repo"
            (repo / "tools" / "frozen-cli-frame-support" / "src").mkdir(parents=True)
            (repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs").write_text("different helper")
            patch_file = repo / "tools" / "frozen_cli_pair.patch"
            patch_file.write_text("patch")
            source = root / "frozen"
            (source / "snapshots").mkdir(parents=True)
            with patch.object(runner, "digest_source", side_effect=lambda path: "a" if path == source else "b" if path == live else "unexpected"), patch.object(runner, "digest_tree", return_value="d"), patch.object(runner, "sha256", side_effect=lambda path: "b" if path == patch_file else "c" if path.name == "lib.rs" else "module-hash"):
                with self.assertRaisesRegex(runner.RunnerError, "live source provenance changed: helper_sha256"):
                    runner.verify_live_source(repo, root, source, 10)

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
