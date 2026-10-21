#!/usr/bin/env python3
"""Focused guards for the frozen binary runner."""

from __future__ import annotations

import io
import json
import os
import selectors
import signal
import stat
import sys
import subprocess
import tarfile
import tempfile
import threading
import time
import unittest
from contextlib import ExitStack, redirect_stderr, redirect_stdout
from pathlib import Path
from typing import Callable
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import frozen_baseline as runner


class RunnerGuards(unittest.TestCase):
    def make_process_bundle(self, root: Path, *, phase: str = "candidate-check", app: str = "showcase") -> tuple[Path, Path, dict[str, str], dict[str, object], bytes, str, str]:
        bins = root / "bins"
        bins.mkdir(parents=True, exist_ok=True)
        for name in runner.APPS:
            executable = bins / name
            if not executable.exists():
                executable.write_bytes(f"binary-{name}".encode())
        case = "showcase::fixture::live"
        case_id = runner.scenario_receipt_id(case)
        expected_hashes = {name: runner.sha256(bins / name) for name in runner.APPS}
        receipt_root = root / "receipts"
        process_root = receipt_root / "process" / runner.hashlib.sha256(case_id.encode()).hexdigest()
        process_root.mkdir(parents=True)
        key = bytes(range(32))
        run_id = "smoke-run:chunk-00000"
        path = str((bins / app).resolve())
        payload: dict[str, object] = {
            "app": app,
            "argv_provenance": "tuiscotti-builder-input",
            "case_id": case_id,
            "constructed_argv_hmac_sha256": "a" * 64,
            "constructed_env_hmac_sha256": "b" * 64,
            "constructed_env_keys": ["HOME", "TERM"],
            "expected_file_identity": "1:2:3:4:5",
            "expected_path": path,
            "expected_sha256": expected_hashes[app],
            "image_hash_source": "linux-proc-exe-open-fd" if sys.platform.startswith("linux") else "macos-proc-pidpath-file",
            "observed_file_identity": "1:2:3:4:5",
            "observed_path": path,
            "observed_sha256": expected_hashes[app],
            "process_start_identity": "1234" if sys.platform.startswith("linux") else "1234:56",
            "previous_receipt_sha256": None,
            "run_id": run_id,
            "phase": phase,
            "pid": 12345,
            "ordinal": 0,
            "schema": 1,
        }

        def sign(record: dict[str, object]) -> dict[str, object]:
            signed = dict(record)
            signed["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(record), runner.hashlib.sha256).hexdigest()
            return signed

        launch_path = process_root / "launch-00000000.json"
        launch_path.write_bytes(runner.canonical_receipt_json(sign(payload)))
        launch_digest = runner.sha256(launch_path)
        close_payload: dict[str, object] = {
            "case_id": case_id,
            "launch_receipt_sha256": launch_digest,
            "ordinal": 0,
            "phase": phase,
            "run_id": run_id,
            "schema": 1,
        }
        (process_root / "closed-00000000.json").write_bytes(runner.canonical_receipt_json(sign(close_payload)))
        return receipt_root, bins, expected_hashes, payload, key, run_id, launch_digest

    def test_candidate_build_refuses_populated_cache_without_removing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "candidate-source"
            repo.mkdir()
            (repo / "Cargo.toml").write_text("[workspace]\n")
            for variant in ("top-level-only", "cached-dependency-and-top-level"):
                target = root / variant
                debug = target / "debug"
                debug.mkdir(parents=True)
                (debug / "showcase").write_bytes(b"preexisting-baseline-copy")
                if variant == "cached-dependency-and-top-level":
                    deps = debug / "deps"
                    deps.mkdir()
                    (deps / "libjunie_tui-cached.rlib").write_bytes(b"preexisting-cached-reference-artifact")
                before = {path.relative_to(target).as_posix(): path.read_bytes() for path in target.rglob("*") if path.is_file()}
                with patch.object(runner, "run", side_effect=AssertionError("Cargo must not run against an existing target")) as cargo:
                    with self.assertRaisesRegex(runner.RunnerError, "must be absent"):
                        runner.build_candidate(repo, root / f"record-{variant}", target, 30)
                cargo.assert_not_called()
                after = {path.relative_to(target).as_posix(): path.read_bytes() for path in target.rglob("*") if path.is_file()}
                self.assertEqual(after, before)

    def test_candidate_build_claims_fresh_target_and_records_binary_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "candidate-source"
            repo.mkdir()
            (repo / "Cargo.toml").write_text("[workspace]\n")
            target = root / "fresh-target"

            def fake_cargo(_command, *, env, **_kwargs):
                bin_dir = Path(env["CARGO_TARGET_DIR"]) / "debug"
                bin_dir.mkdir(parents=True)
                for app in runner.APPS:
                    binary = bin_dir / app
                    binary.write_bytes(f"newly-built-{app}".encode())
                    binary.chmod(0o755)

            with patch.object(runner, "git_state", return_value=("candidate-revision", "c" * 64, [])), patch.object(runner, "run", side_effect=fake_cargo):
                with patch.object(sys, "stdout", io.StringIO()):
                    runner.build_candidate(repo, root / "evidence", target, 30)
            record = json.loads((root / "evidence" / "candidate-build.json").read_text())
            self.assertEqual(set(record["binary_files"]), set(runner.APPS))
            for app in runner.APPS:
                self.assertEqual(record["binary_files"][app], runner.stable_private_binary(target / "debug" / app, f"candidate binary {app}")[1])

    def provenance_payload(self) -> dict[str, object]:
        env_values = {key: {"present": False, "value_sha256": None} for key in runner.PROVENANCE_ENV_KEYS}
        env_record = {
            "policy": "hash-only-explicit-build-environment-allowlist",
            "allowlisted_values": env_values,
            "fingerprint_sha256": runner.canonical_json_digest(env_values),
            "unknown_environment_values_recorded": False,
            "effective_env_was_runner_cleaned": True,
            "hermetic": False,
            "hermeticity_note": "test fixture",
        }
        stages = [
            {
                "name": name,
                "argv": ["cargo", name],
                "effective_environment": env_record,
                "output_application_binary_sha256": {app: "0" * 64 for app in runner.APPS},
            }
            for name in ("build-frozen-binaries", "build-frozen-nextest-harness")
        ]
        return {
            "schema": runner.BUILD_PROVENANCE_SCHEMA,
            "schema_version": runner.BUILD_PROVENANCE_VERSION,
            "status": "observed-not-hermetic",
            "observed_at_utc": "2026-10-20T00:00:00+00:00",
            "reference": {"tag": runner.BASELINE_TAG, "tag_object": runner.BASELINE_TAG_OBJECT, "commit": runner.BASELINE_COMMIT, "tree": runner.BASELINE_TREE, "source_digest": "1" * 64, "archive_sha256": "2" * 64, "archive_argv": ["git", "-C", "/repo", "archive", "--format=tar", runner.BASELINE_COMMIT], "cargo_lock_sha256": "3" * 64, "rust_toolchain_toml_sha256": None, "tuiscotti_rev": runner.TUISCOTTI_REV},
            "runner_inputs": {name: None for name in ("runner", "harness_patch", "frame_helper_manifest", "frame_helper_lock", "frame_helper", "extension_manifest", "extension_source", "reference_decisions")},
            "host": {"system": "test", "release": "test", "machine": "test", "python": "test"},
            "target_dir": "/tmp/isolated-target",
            "config_inputs": [],
            "toolchain": {name: None for name in ("cargo_path_entry", "cargo_selected_toolchain", "cargo_selection_method", "rustup", "active_toolchain", "rustc_selected_toolchain", "rustc_selection_method", "rustc_host_triple", "llvm_version", "rustc_sysroot", "rustc_sysroot_identity_files", "cc", "clang", "linker", "xcrun", "xcode_select", "sdk_path", "sdk_version", "xcode_developer_directory", "link_invocation")},
            "build": {"stage_order": [stage["name"] for stage in stages], "stages": stages, "environment_policy": "test", "profile": "Cargo default dev profile", "features": "default features", "target_triple": None, "target_selection": "test", "final_application_binary_sha256": {app: "0" * 64 for app in runner.APPS}, "final_hash_point": "after build"},
            "admission": {"eligible": False, "reason": "provenance observation only; no reference decision or binary admission is performed"},
        }

    def test_build_provenance_seals_and_checks_integrity(self) -> None:
        payload = self.provenance_payload()
        sealed = runner.seal_build_provenance(payload)
        runner.validate_build_provenance(sealed)
        changed = dict(sealed)
        changed["host"] = {"system": "tampered"}
        with self.assertRaisesRegex(runner.RunnerError, "content digest mismatch"):
            runner.validate_build_provenance(changed)
        missing = dict(sealed)
        missing.pop("toolchain")
        with self.assertRaisesRegex(runner.RunnerError, "required fields"):
            runner.validate_build_provenance(missing)
        missing_runner_input = self.provenance_payload()
        missing_runner_input["runner_inputs"].pop("runner")
        with self.assertRaisesRegex(runner.RunnerError, "runner input hashes"):
            runner.validate_build_provenance(runner.seal_build_provenance(missing_runner_input))

    def test_build_provenance_final_hashes_match_last_stage(self) -> None:
        payload = self.provenance_payload()
        payload["build"]["final_application_binary_sha256"][runner.APPS[0]] = "1" * 64
        with self.assertRaisesRegex(runner.RunnerError, "final binaries differ"):
            runner.validate_build_provenance(runner.seal_build_provenance(payload))

    def test_build_provenance_payload_collects_tool_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            source = repo / "frozen-source"
            source.mkdir()
            (source / "Cargo.lock").write_text("lock")
            (source / "rust-toolchain.toml").write_text("toolchain")
            inputs = {
                "tools/frozen_cli_pair.patch": "patch",
                "tools/frozen-cli-frame-support/Cargo.toml": "manifest",
                "tools/frozen-cli-frame-support/Cargo.lock": "helper-lock",
                "tools/frozen-cli-frame-support/src/lib.rs": "helper",
                str(runner.EXTENSION_MANIFEST): "extensions",
                str(runner.EXTENSION_SOURCE): "extension-source",
                str(runner.REFERENCE_DECISIONS): "decisions",
            }
            for relative, contents in inputs.items():
                path = repo / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents)
            toolchain = {name: None for name in (
                "cargo_path_entry", "cargo_selected_toolchain", "cargo_selection_method", "rustup", "active_toolchain",
                "rustc_selected_toolchain", "rustc_selection_method", "rustc_host_triple", "llvm_version", "rustc_sysroot",
                "rustc_sysroot_identity_files", "cc", "clang", "linker", "xcrun", "xcode_select", "sdk_path", "sdk_version",
                "xcode_developer_directory", "link_invocation",
            )}
            stage_names = ("build-frozen-binaries", "build-frozen-nextest-harness")
            stage_hashes = {app: "0" * 64 for app in runner.APPS}
            stages = [
                {"name": name, "argv": ["cargo", name], "env": {"PATH": "/bin"}, "output_application_binary_sha256": stage_hashes}
                for name in stage_names
            ]
            with patch.object(runner.subprocess, "run", return_value=runner.subprocess.CompletedProcess([], 0, b"archive", b"")), patch.object(runner, "toolchain_provenance", return_value=toolchain):
                payload = runner.build_provenance_payload(repo, source, "1" * 64, repo / "target", stage_hashes, stages, {"PATH": "/bin"}, 20)
            sealed = runner.seal_build_provenance(payload)
            runner.validate_build_provenance(sealed)
            self.assertEqual(sealed["runner_inputs"]["runner"], runner.sha256(Path(runner.__file__).resolve()))
            self.assertEqual(sealed["runner_inputs"]["frame_helper_lock"], runner.sha256(repo / "tools/frozen-cli-frame-support/Cargo.lock"))

    def test_build_provenance_cohort_fingerprint_excludes_observation_time(self) -> None:
        first = runner.seal_build_provenance(self.provenance_payload())
        second_payload = self.provenance_payload()
        second_payload["observed_at_utc"] = "2026-10-21T00:00:00+00:00"
        second = runner.seal_build_provenance(second_payload)
        self.assertEqual(first["cohort_fingerprint_sha256"], second["cohort_fingerprint_sha256"])
        self.assertNotEqual(first["provenance_sha256"], second["provenance_sha256"])

    def test_build_environment_records_only_hashes_for_safe_allowlist(self) -> None:
        secret = "never-serialize-this-token"
        record = runner.build_environment_fingerprint({"PATH": "/safe/bin", "CFLAGS": "-O2", "AWS_SECRET_ACCESS_KEY": secret})
        encoded = json.dumps(record)
        self.assertNotIn(secret, encoded)
        self.assertNotIn("AWS_SECRET_ACCESS_KEY", encoded)
        self.assertEqual(record["allowlisted_values"]["PATH"]["value_sha256"], runner.hashlib.sha256(b"/safe/bin").hexdigest())
        self.assertFalse(record["hermetic"])

    def test_term_profiles_are_explicit_and_reported(self) -> None:
        for profile in ("xterm-256color", "linux", "dumb"):
            env = {"TERM": "inherited"}
            record = runner.apply_term_profile(env, profile)
            self.assertEqual(env["TERM"], profile)
            self.assertEqual(record["profile"], profile)
            self.assertEqual(record["runner_term"], profile)
            self.assertEqual(record["child_term"], profile)
            self.assertEqual(len(record["sha256"]), 64)
        env = {"TERM": "inherited"}
        record = runner.apply_term_profile(env, "unset")
        self.assertNotIn("TERM", env)
        self.assertFalse(record["runner_term_present"])
        self.assertFalse(record["child_term_present"])
        self.assertIsNone(record["child_term"])
        inherited = {"TERM": "host-term"}
        inherited_record = runner.apply_term_profile(inherited, None)
        self.assertEqual(inherited_record["runner_term"], "host-term")
        self.assertEqual(inherited_record["child_term"], "xterm-256color")
        with self.assertRaisesRegex(runner.RunnerError, "unsupported TERM profile"):
            runner.apply_term_profile({}, "vt100")

    def test_grown_reference_exception_is_fixed_to_its_term(self) -> None:
        decision = runner.validate_reference_decision_document(json.loads((Path(runner.__file__).resolve().parent / "frozen_cli_references.json").read_text()))
        decisions = [decision]
        runner.validate_reference_term_profile(decisions, [runner.REFERENCE_CASE], None)
        for profile in ("xterm-256color", "linux", "dumb", "unset"):
            with self.assertRaisesRegex(runner.RunnerError, "pinned grown-reference exception"):
                runner.validate_reference_term_profile(decisions, [runner.REFERENCE_CASE], profile)
        runner.validate_reference_term_profile(decisions, ["showcase::pages::overview::120x40::truecolor"], "linux")

    def test_versioned_harness_applies_term_profile_to_tuiscotti_child(self) -> None:
        patch = Path(runner.__file__).resolve().parent / "frozen_cli_pair.patch"
        self.assertEqual(runner.sha256(patch), runner.EXTENSION_PATCH_SHA256)
        source = patch.read_text()
        self.assertIn('std::env::var("TERMROCK_PAIR_TERM_PROFILE")', source)
        self.assertIn('Some("unset") => { child_env.remove("TERM"); }', source)
        self.assertIn('tui = tui.env("TERM", profile)', source)
        self.assertIn('env_remove("TERMROCK_PAIR_PROCESS_RECEIPT_KEY")', source)
        self.assertIn('prepare_process_launch(&case.name, &argv, &child_env)', source)

    def semantic_frame(self, *, cursor_style: str = "Bar") -> bytes:
        cell = {"x": 0, "y": 0, "symbol": "x", "width": 1, "continuation": False, "fg": {"Rgb": {"r": 255, "g": 255, "b": 255}}, "bg": {"Rgb": {"r": 0, "g": 0, "b": 0}}, "mods": {"hidden": False, "blink": False, "bold": False, "dim": False, "italic": False, "underline": False, "strikethrough": False, "reverse": False}}
        frame = {"version": 3, "cols": 1, "rows": 1, "cells": [cell], "cursor": {"x": 0, "y": 0, "visible": False, "style": cursor_style, "blinking": False}, "provenance": {"tool": "tuiscotti", "tool_version": "0.2.0", "profile": "tuiscotti-default", "source": "screen", "argv": [], "created_unix": 0}}
        return json.dumps(frame, separators=(",", ":")).encode()

    def extension_manifest(self) -> dict[str, object]:
        journey = {"id": "sort", "title": "Sort table", "checkpoints": ["initial", "sorted", "cleared"]}
        sizes = [{"cols": cols, "rows": rows, "allow_narrow_fallback": (cols, rows) in {(40, 12), (60, 18)}} for cols, rows in runner.EXTENSION_SIZES]
        cases = []
        for cols, rows in runner.EXTENSION_SIZES:
            modes = ("truecolor", "nocolor") if (cols, rows) in {(40, 12), (60, 18)} else runner.EXTENSION_COLORS
            if (cols, rows) in {(40, 12), (60, 18)}:
                checkpoints = ["initial", "narrow_revealed_120x40", "sorted", "cleared"]
            elif cols < 120:
                checkpoints = ["initial", "table_revealed_120x40", "sorted", "cleared"]
            else:
                checkpoints = ["initial", "sorted", "cleared"]
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

    def test_live_receipts_bind_case_order_name_and_both_frame_digests(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            case = "holla/files/120x40/truecolor"
            logical = "holla/files/120x40/truecolor"
            expected = root / "expected"
            receipt_root = root / "receipts"
            expected.mkdir()
            actual_root = root / "receipts.actual"
            actual_root.mkdir()
            semantic_actual_root = root / "receipts.semantic-actual"
            semantic_actual_root.mkdir()
            ansi = b"frame\x1b[?25h"
            semantic = self.semantic_frame()
            identity = runner.checkpoint_identity(case, 0, logical)
            stem = f"checkpoint-00000000-{identity}"
            (expected / f"{stem}.ansi").write_bytes(ansi)
            (expected / f"{stem}.frame.json").write_bytes(semantic)
            (actual_root / f"{stem}.ansi").write_bytes(ansi)
            (actual_root / f"{stem}.frame.json").write_bytes(semantic)
            (semantic_actual_root / f"{stem}.frame.json").write_bytes(semantic)
            receipt_dir = receipt_root / runner.hashlib.sha256(case.encode()).hexdigest()
            receipt_dir.mkdir(parents=True)
            receipt = {
                "case_id": case,
                "ordinal": 0,
                "logical_name": logical,
                "sha256": runner.hashlib.sha256(ansi).hexdigest(),
                "semantic_frame_sha256": runner.hashlib.sha256(semantic).hexdigest(),
                "process_receipt_sha256": "f" * 64,
            }
            (receipt_dir / f"{stem}.json").write_text(json.dumps(receipt))
            actual_with_host_provenance = json.loads(semantic)
            actual_with_host_provenance["provenance"]["created_unix"] = 123
            actual_with_host_provenance["provenance"]["source"] = "ansi"
            (actual_root / f"{stem}.frame.json").write_text(json.dumps(actual_with_host_provenance, separators=(",", ":")))
            (semantic_actual_root / f"{stem}.frame.json").write_bytes(semantic)
            found = runner.read_receipts(receipt_root, [case], expected)
            self.assertEqual(found, {(case, 0, logical): (receipt["sha256"], receipt["semantic_frame_sha256"], None)})
            (expected / f"{stem}.ansi").write_bytes(b"changed")
            with self.assertRaisesRegex(runner.RunnerError, "ANSI frame missing or hash mismatch"):
                runner.read_receipts(receipt_root, [case], expected)
            (expected / f"{stem}.ansi").write_bytes(ansi)
            (expected / f"{stem}.frame.json").write_bytes(self.semantic_frame(cursor_style="Block"))
            with self.assertRaisesRegex(runner.RunnerError, "semantic frame hash mismatch"):
                runner.read_receipts(receipt_root, [case], expected)
            (expected / f"{stem}.frame.json").write_bytes(semantic)
            (actual_root / f"{stem}.frame.json").write_bytes(self.semantic_frame(cursor_style="Block"))
            with self.assertRaisesRegex(runner.RunnerError, "actual semantic frame hash mismatch"):
                runner.read_receipts(receipt_root, [case], expected)
            (actual_root / f"{stem}.frame.json").write_bytes(semantic)
            (actual_root / f"{stem}.ansi").write_bytes(b"bad actual bytes")
            with self.assertRaisesRegex(runner.RunnerError, "actual ANSI frame missing or hash mismatch"):
                runner.read_receipts(receipt_root, [case], expected)
            (actual_root / f"{stem}.ansi").write_bytes(ansi)
            (actual_root / f"{stem}.extra.frame.json").write_bytes(semantic)
            with self.assertRaisesRegex(runner.RunnerError, "runtime actual Frame inventory mismatch"):
                runner.read_receipts(receipt_root, [case], expected)
            (actual_root / f"{stem}.extra.frame.json").unlink()
            (semantic_actual_root / f"{stem}.extra.frame.json").write_bytes(semantic)
            with self.assertRaisesRegex(runner.RunnerError, "runtime semantic actual inventory mismatch"):
                runner.read_receipts(receipt_root, [case], expected)

    def test_process_receipts_bind_real_phase_path_hash_and_complete_spawn_inventory(self) -> None:
        if not (sys.platform.startswith("linux") or sys.platform == "darwin"):
            self.skipTest("paired process image probing is unsupported on this platform")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root)
            case_id = "showcase/fixture/live"
            checkpoint = {"case_id": case_id, "ordinal": 0, "logical_name": "fixture/screen", "process_receipt_sha256": runner.sha256(next((receipt_root / "process").rglob("launch-*.json")))}
            checkpoint_dir = receipt_root / runner.hashlib.sha256(case_id.encode()).hexdigest()
            checkpoint_dir.mkdir()
            (checkpoint_dir / "checkpoint.json").write_text(json.dumps(checkpoint))
            links, report = runner.verify_process_receipts(
                receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes,
                None, run_id, key.hex(), {(case_id, 0, "fixture/screen"): ("1" * 64, "2" * 64, None)},
            )
            self.assertEqual(len(links[case_id]), 1)
            self.assertEqual(report["receipt_count"], 1)
            self.assertEqual(report["pid_count"], 1)

    def test_process_receipts_reject_baseline_substitution_wrong_app_and_wrong_phase(self) -> None:
        if not (sys.platform.startswith("linux") or sys.platform == "darwin"):
            self.skipTest("paired process image probing is unsupported on this platform")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root, phase="candidate-check")
            process = next((receipt_root / "process").rglob("launch-*.json"))
            baseline_bins = root / "baseline-bins"
            baseline_bins.mkdir()
            (baseline_bins / "showcase").write_bytes(b"baseline-showcase")
            record = json.loads(process.read_text())
            record["expected_path"] = str((baseline_bins / "showcase").resolve())
            record["observed_path"] = record["expected_path"]
            record["expected_sha256"] = runner.sha256(baseline_bins / "showcase")
            record["observed_sha256"] = record["expected_sha256"]
            record.pop("hmac_sha256")
            record["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(record), runner.hashlib.sha256).hexdigest()
            process.write_bytes(runner.canonical_receipt_json(record))
            with self.assertRaisesRegex(runner.RunnerError, "path does not match"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            record = json.loads(process.read_text())
            record["app"] = "tablepro"
            record.pop("hmac_sha256")
            record["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(record), runner.hashlib.sha256).hexdigest()
            process.write_bytes(runner.canonical_receipt_json(record))
            with self.assertRaisesRegex(runner.RunnerError, "path does not match"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            record = json.loads(process.read_text())
            record["phase"] = "baseline-check"
            record.pop("hmac_sha256")
            record["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(record), runner.hashlib.sha256).hexdigest()
            process.write_bytes(runner.canonical_receipt_json(record))
            with self.assertRaisesRegex(runner.RunnerError, "identity or ordering"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root / "wrong-hash")
            hashes["showcase"] = "0" * 64
            with self.assertRaisesRegex(runner.RunnerError, "hash does not match"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

    def test_process_receipts_reject_missing_duplicate_tampered_and_reordered_launches(self) -> None:
        if not (sys.platform.startswith("linux") or sys.platform == "darwin"):
            self.skipTest("paired process image probing is unsupported on this platform")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root)
            process_dir = next((receipt_root / "process").iterdir())
            (process_dir / "closed-00000000.json").unlink()
            with self.assertRaisesRegex(runner.RunnerError, "missing closed record"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root / "duplicate")
            process_dir = next((receipt_root / "process").iterdir())
            duplicate = json.loads((process_dir / "launch-00000000.json").read_text())
            duplicate["ordinal"] = 1
            duplicate["previous_receipt_sha256"] = runner.sha256(process_dir / "launch-00000000.json")
            duplicate.pop("hmac_sha256")
            duplicate["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(duplicate), runner.hashlib.sha256).hexdigest()
            (process_dir / "launch-00000001.json").write_bytes(runner.canonical_receipt_json(duplicate))
            with self.assertRaisesRegex(runner.RunnerError, "completion sequence"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root / "tampered")
            process = next((receipt_root / "process").rglob("launch-*.json"))
            changed = json.loads(process.read_text())
            changed["constructed_env_hmac_sha256"] = "c" * 64
            process.write_bytes(runner.canonical_receipt_json(changed))
            with self.assertRaisesRegex(runner.RunnerError, "authentication failed"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            receipt_root, bins, hashes, _, key, run_id, _ = self.make_process_bundle(root / "missing")
            next((receipt_root / "process").rglob("launch-*.json")).unlink()
            with self.assertRaisesRegex(runner.RunnerError, "missing launch record"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

    def test_process_receipts_require_start_identity_and_authenticated_launch_chain(self) -> None:
        if not (sys.platform.startswith("linux") or sys.platform == "darwin"):
            self.skipTest("paired process image probing is unsupported on this platform")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipt_root, bins, hashes, _, key, run_id, first_digest = self.make_process_bundle(root)
            process_dir = next((receipt_root / "process").iterdir())
            launch_path = process_dir / "launch-00000000.json"
            record = json.loads(launch_path.read_text())
            record.pop("process_start_identity")
            record.pop("hmac_sha256")
            record["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(record), runner.hashlib.sha256).hexdigest()
            launch_path.write_bytes(runner.canonical_receipt_json(record))
            with self.assertRaisesRegex(runner.RunnerError, "fields differ from schema"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

            receipt_root, bins, hashes, payload, key, run_id, first_digest = self.make_process_bundle(root / "chain")
            process_dir = next((receipt_root / "process").iterdir())
            second = dict(payload)
            second.update({"ordinal": 1, "pid": 12346, "previous_receipt_sha256": first_digest})
            second.pop("hmac_sha256", None)
            second["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(second), runner.hashlib.sha256).hexdigest()
            second_path = process_dir / "launch-00000001.json"
            second_path.write_bytes(runner.canonical_receipt_json(second))
            second_digest = runner.sha256(second_path)
            close = {
                "case_id": payload["case_id"], "launch_receipt_sha256": second_digest,
                "ordinal": 1, "phase": payload["phase"], "run_id": run_id, "schema": 1,
            }
            close["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(close), runner.hashlib.sha256).hexdigest()
            (process_dir / "closed-00000001.json").write_bytes(runner.canonical_receipt_json(close))
            second["previous_receipt_sha256"] = "f" * 64
            second.pop("hmac_sha256")
            second["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(second), runner.hashlib.sha256).hexdigest()
            second_path.write_bytes(runner.canonical_receipt_json(second))
            second_digest = runner.sha256(second_path)
            close["launch_receipt_sha256"] = second_digest
            close.pop("hmac_sha256")
            close["hmac_sha256"] = runner.hmac.new(key, runner.canonical_receipt_json(close), runner.hashlib.sha256).hexdigest()
            (process_dir / "closed-00000001.json").write_bytes(runner.canonical_receipt_json(close))
            with self.assertRaisesRegex(runner.RunnerError, "chain is broken"):
                runner.verify_process_receipts(receipt_root, ["showcase::fixture::live"], "candidate-check", bins, hashes, None, run_id, key.hex(), {})

    def test_phase_binary_inventory_rejects_symlinks_and_postbuild_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bins = Path(temp) / "bins"
            bins.mkdir()
            for app in runner.APPS:
                (bins / app).write_text(app)
            original = runner.phase_binary_hashes(bins)
            (bins / "showcase").unlink()
            (bins / "showcase").symlink_to(bins / "tablepro")
            with self.assertRaisesRegex(runner.RunnerError, "unsafe"):
                runner.phase_binary_hashes(bins)
            (bins / "showcase").unlink()
            (bins / "showcase").write_text("replacement")
            self.assertNotEqual(original, runner.phase_binary_hashes(bins))

    def test_semantic_frame_accepts_optional_underline_color(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "frame.json"
            frame = json.loads(self.semantic_frame())
            frame["cells"][0]["underline_color"] = {"Rgb": {"r": 1, "g": 2, "b": 3}}
            payload = json.dumps(frame, separators=(",", ":")).encode()
            path.write_bytes(payload)
            runner.validate_semantic_frame_file(path, runner.hashlib.sha256(payload).hexdigest())
            frame["cells"][0]["underline_color"] = "invalid"
            path.write_text(json.dumps(frame, separators=(",", ":")))
            with self.assertRaisesRegex(runner.RunnerError, "invalid semantic frame cell"):
                runner.validate_semantic_frame_file(path, "")

    def test_semantic_frame_accepts_default_and_indexed_colors(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "frame.json"
            frame = json.loads(self.semantic_frame())
            frame["cells"][0]["fg"] = "Default"
            frame["cells"][0]["bg"] = {"Indexed": 0}
            frame["cells"][0]["underline_color"] = "Default"
            payload = json.dumps(frame, separators=(",", ":")).encode()
            path.write_bytes(payload)
            runner.validate_semantic_frame_file(path, runner.hashlib.sha256(payload).hexdigest())
            for invalid in ({"Indexed": 256}, {"Rgb": {"r": 0, "g": 0, "b": 999}}, {"Other": 1}, {"Rgb": {"r": True, "g": 0, "b": 0}}):
                frame["cells"][0]["fg"] = invalid
                path.write_text(json.dumps(frame, separators=(",", ":")))
                with self.assertRaisesRegex(runner.RunnerError, "invalid semantic frame cell"):
                    runner.validate_semantic_frame_file(path, "")

    def test_reference_decision_is_pinned_to_exact_tuple_and_two_frames(self) -> None:
        document = json.loads((Path(runner.__file__).resolve().parent / "frozen_cli_references.json").read_text())
        decision = runner.validate_reference_decision_document(document)
        self.assertEqual(decision["tuple"]["test_id"], runner.REFERENCE_CASE)
        self.assertEqual(len(decision["independent_replays"]), 2)
        for mutate in (
            lambda data: data["decisions"][0]["tuple"].update({"test_id": "showcase::overview::120x40::truecolor"}),
            lambda data: data["decisions"][0]["tuple"].update({"ordinal": 1}),
            lambda data: data["decisions"][0]["live_frame"].update({"semantic_frame_sha256": "0" * 64}),
            lambda data: data["decisions"][0]["independent_replays"].pop(),
            lambda data: data["decisions"].append(dict(data["decisions"][0])),
        ):
            altered = json.loads(json.dumps(document))
            mutate(altered)
            with self.assertRaises(runner.RunnerError):
                runner.validate_reference_decision_document(altered)

    def test_reference_receipt_is_exact_and_rejects_ascii_or_cursor_diff(self) -> None:
        decision = runner.validate_reference_decision_document(json.loads((Path(runner.__file__).resolve().parent / "frozen_cli_references.json").read_text()))
        key = (decision["tuple"]["case_id"], decision["tuple"]["ordinal"], decision["tuple"]["logical_name"])
        reference = decision["live_frame"]
        correct = {key: (reference["ansi_sha256"], reference["semantic_frame_sha256"], runner.REFERENCE_DECISIONS_SHA256)}
        runner.validate_reference_receipts(correct, [runner.REFERENCE_CASE], "baseline-capture", decision, runner.REFERENCE_DECISIONS_SHA256)
        wrong_ascii = {key: ("0" * 64, reference["semantic_frame_sha256"], runner.REFERENCE_DECISIONS_SHA256)}
        wrong_cursor = {key: (reference["ansi_sha256"], "0" * 64, runner.REFERENCE_DECISIONS_SHA256)}
        for altered in (wrong_ascii, wrong_cursor):
            with self.assertRaisesRegex(runner.RunnerError, "reviewed live-frame reference mismatch"):
                runner.validate_reference_receipts(altered, [runner.REFERENCE_CASE], "baseline-capture", decision, runner.REFERENCE_DECISIONS_SHA256)
        with self.assertRaisesRegex(runner.RunnerError, "reference decision receipt binding"):
            runner.validate_reference_receipts({key: (reference["ansi_sha256"], reference["semantic_frame_sha256"], None)}, [runner.REFERENCE_CASE], "baseline-capture", decision, runner.REFERENCE_DECISIONS_SHA256)

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
        self.assertEqual(runner.scenario_chunks(["before", runner.REFERENCE_CASE, "after"], 2), [["before", "after"], [runner.REFERENCE_CASE]])
        with self.assertRaisesRegex(runner.RunnerError, "duplicated"):
            runner.scenario_chunks([runner.REFERENCE_CASE, runner.REFERENCE_CASE], 2)
        with self.assertRaisesRegex(runner.RunnerError, "chunk size"):
            runner.scenario_chunks(names, 513)

    def test_fresh_expected_chunk_digest_binds_modes_and_rejects_aliases(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "expected"
            root.mkdir()
            (root / "checkpoint.ansi").write_bytes(b"frame")
            first = runner._fresh_expected_tree_sha256(root)
            (root / "checkpoint.ansi").chmod(0o444)
            second = runner._fresh_expected_tree_sha256(root)
            self.assertNotEqual(first, second)
            link = root / "linked.ansi"
            link.symlink_to(root / "checkpoint.ansi")
            with self.assertRaisesRegex(runner.RunnerError, "aliased or non-regular"):
                runner._fresh_expected_tree_sha256(root)

    def test_fresh_journal_publish_is_immutable_and_rejects_symlink_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            run_root = Path(temp) / "run"
            run_root.mkdir(mode=0o700)
            envelope = {"receipt": {"ordinal": 0}, "hmac_sha256": "a" * 64}
            published = runner._fresh_journal_append(run_root, envelope, 0)
            self.assertEqual(published.stat().st_mode & 0o777, 0o400)
            self.assertEqual(runner._fresh_journal_envelopes(run_root), [envelope])
            with self.assertRaisesRegex(runner.RunnerError, "refusing to replace"):
                runner._fresh_journal_append(run_root, envelope, 0)
            outside = Path(temp) / "outside"
            outside.mkdir(mode=0o700)
            runner.shutil.rmtree(run_root / "journal")
            (run_root / "journal").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(runner.RunnerError, "journal directory is unsafe"):
                runner._fresh_journal_append(run_root, envelope, 1)

    def test_fresh_advance_cli_is_capture_replay_only_and_bounded(self) -> None:
        parsed = runner.parser().parse_args([
            "--timeout", "225", "fresh-run-advance", "--cohort", "frozen-cohort",
            "--baseline-bin-dir", "/sealed", "--expected-plan-sha256", "a" * 64,
            "--key-path", "/private/key", "--expected-ordinal", "0", "--expected-phase", "baseline-capture",
        ])
        self.assertEqual(parsed.command, "fresh-run-advance")
        self.assertEqual(parsed.timeout, 225)
        self.assertFalse(hasattr(parsed, "phase"))
        self.assertEqual(parsed.expected_ordinal, 0)
        self.assertEqual(parsed.expected_phase, "baseline-capture")
        with self.assertRaises(SystemExit):
            runner.parser().parse_args([
                "fresh-run-advance", "--cohort", "frozen-cohort", "--baseline-bin-dir", "/sealed",
                "--expected-plan-sha256", "a" * 64, "--key-path", "/private/key", "--phase", "candidate",
            ])
        with patch.object(runner.sys, "argv", ["frozen_baseline.py",
                "fresh-run-advance", "--cohort", "frozen-cohort", "--baseline-bin-dir", "/sealed",
                "--expected-plan-sha256", "a" * 64, "--key-path", "/private/key", "--expected-ordinal", "0",
            ]), patch.object(runner.sys, "stderr", io.StringIO()), patch.object(runner, "advance_fresh_run") as advance:
            status = runner.main()
        self.assertEqual(status, 1)
        advance.assert_not_called()

    def test_fresh_advance_expected_step_rejects_stale_future_and_wrong_phase_before_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp))
            live = unittest.mock.Mock(side_effect=self._passing_live_chunk)
            phase_root = Path(fixture["plan"]["execution"]["artifact_roots"]["baseline_capture"])
            cases = ((1, "baseline-capture"), (9, "baseline-capture"), (0, "baseline-replay"))
            for expected_ordinal, expected_phase in cases:
                with self.subTest(expected_ordinal=expected_ordinal, expected_phase=expected_phase):
                    with self._advance_fresh_fixture(fixture, live):
                        with self.assertRaisesRegex(runner.RunnerError, "differs from the expected ordinal and phase"):
                            runner.advance_fresh_run(
                                fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30,
                                fixture["creation_sha256"], fixture["key_path"],
                                expected_ordinal=expected_ordinal, expected_phase=expected_phase,
                            )
                    self.assertFalse(phase_root.exists())
            live.assert_not_called()

    def test_fresh_advance_repeated_last_capture_expectation_cannot_start_replay(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp), chunk_size=2)
            with self._advance_fresh_fixture(fixture, self._passing_live_chunk):
                capture = runner.advance_fresh_run(
                    fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30,
                    fixture["creation_sha256"], fixture["key_path"],
                    expected_ordinal=0, expected_phase="baseline-capture",
                )
            self.assertEqual((capture["journal_ordinal"], capture["phase"]), (0, "baseline-capture"))
            replay_root = Path(fixture["plan"]["execution"]["artifact_roots"]["baseline_replay"])
            retry = unittest.mock.Mock(side_effect=self._passing_live_chunk)
            with self._advance_fresh_fixture(fixture, retry):
                with self.assertRaisesRegex(runner.RunnerError, r"actual \(1, baseline-replay\)"):
                    runner.advance_fresh_run(
                        fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30,
                        fixture["creation_sha256"], fixture["key_path"],
                        expected_ordinal=0, expected_phase="baseline-capture",
                    )
            retry.assert_not_called()
            self.assertFalse(replay_root.exists())

    def test_fresh_advance_expected_step_arguments_are_exact_and_paired(self) -> None:
        for ordinal in (True, -1, 0.0):
            with self.subTest(ordinal=ordinal), self.assertRaisesRegex(runner.RunnerError, "nonnegative integer"):
                runner.advance_fresh_run(Path("/unused"), Path("/unused"), "cohort", Path("/unused"), 30, "a" * 64, Path("/unused"), expected_ordinal=ordinal, expected_phase="baseline-capture")
        for phase in ("candidate-check", [], None):
            if phase is None:
                with self.assertRaisesRegex(runner.RunnerError, "supplied together"):
                    runner.advance_fresh_run(Path("/unused"), Path("/unused"), "cohort", Path("/unused"), 30, "a" * 64, Path("/unused"), expected_ordinal=0, expected_phase=phase)
            else:
                with self.subTest(phase=phase), self.assertRaisesRegex(runner.RunnerError, "baseline-capture or baseline-replay"):
                    runner.advance_fresh_run(Path("/unused"), Path("/unused"), "cohort", Path("/unused"), 30, "a" * 64, Path("/unused"), expected_ordinal=0, expected_phase=phase)
        with self.assertRaisesRegex(runner.RunnerError, "supplied together"):
            runner.advance_fresh_run(Path("/unused"), Path("/unused"), "cohort", Path("/unused"), 30, "a" * 64, Path("/unused"), expected_ordinal=0)

    def test_fresh_run_lock_serializes_two_processes_without_timing_sleeps(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            run_root = Path(temp) / "run"
            run_root.mkdir(mode=0o700)
            owner_fd = runner._fresh_run_lock(run_root)
            source_dir = Path(__file__).resolve().parent
            child_code = (
                "import sys; from pathlib import Path; "
                f"sys.path.insert(0, {str(source_dir)!r}); import frozen_baseline as r; "
                "print('launch-marker', flush=True); sys.stdin.readline(); "
                "\ntry:\n fd = r._fresh_run_lock(Path(sys.argv[1]))\n"
                "except r.RunnerError as error:\n print('blocked:' + str(error), flush=True)\n"
                "else:\n r.fcntl.flock(fd, r.fcntl.LOCK_UN); r.os.close(fd); print('acquired', flush=True)"
            )
            child = subprocess.Popen(
                [sys.executable, "-c", child_code, str(run_root)],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, start_new_session=True,
            )

            def read_line() -> str:
                assert child.stdout is not None
                selector = selectors.DefaultSelector()
                selector.register(child.stdout, selectors.EVENT_READ)
                try:
                    ready = selector.select(5)
                    self.assertTrue(ready, "lock contender did not reach its synchronized launch marker")
                    return child.stdout.readline().rstrip("\n")
                finally:
                    selector.close()

            try:
                self.assertEqual(read_line(), "launch-marker")
                assert child.stdin is not None
                child.stdin.write("attempt\n")
                child.stdin.flush()
                self.assertIn("blocked:another fresh run chunk advance is active", read_line())
                self.assertEqual(child.wait(timeout=5), 0)
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait(timeout=5)
                if child.stdout is not None:
                    child.stdout.close()
                if child.stderr is not None:
                    child.stderr.close()
                if child.stdin is not None:
                    child.stdin.close()
                runner.fcntl.flock(owner_fd, runner.fcntl.LOCK_UN)
                runner.os.close(owner_fd)

    def _fresh_plan_fixture(self, root: Path, *, chunk_size: int = 1, execution: runner.PrepareExecution | None = None, advance_budget_seconds: int = runner.FRESH_PLAN_MAX_TIMEOUT) -> dict[str, object]:
        repo = root / "repo"
        source = root / "source"
        live = root / "live-source"
        for directory in (repo / "tools" / "frozen-cli-frame-support" / "src", source / "snapshots", source / "tests" / "visual_baseline", live / "tests" / "visual_baseline"):
            directory.mkdir(parents=True, exist_ok=True)
        for relative, contents in (
            (Path("tools/frozen_cli_extensions.json"), "{}"),
            (Path("tools/frozen_cli_extensions.rs"), "extension"),
            (Path("tools/frozen_cli_pair.patch"), "patch"),
            (Path("tools/frozen-cli-frame-support/src/lib.rs"), "helper"),
        ):
            target = repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(contents)
        for relative, contents in ((Path("Cargo.lock"), "lock"), (Path("rust-toolchain.toml"), "toolchain")):
            (source / relative).write_text(contents)
        (source / "Cargo.toml").write_text("[workspace]\n")
        (source / "tests" / "visual_baseline" / "main.rs").write_text("harness")
        (live / "tests" / "visual_baseline" / "main.rs").write_text("patched harness")
        (root / "provenance.json").write_text("{}")
        (root / "live-source.json").write_text("{}")
        extension = "termrock_extensions::showcase::table_sort::fixture::80x24::truecolor"
        inventory = {
            "showcase::pages::fixture::80x24::truecolor": True,
            extension: True,
            "source_only::fixture": False,
            "modes::showcase_boot_sets_terminal_modes": False,
        }
        live_record = {"test_inventory_sha256": runner.inventory_digest(inventory), "module_sha256": "a" * 64}
        cohort = {
            "cohort_id": "fixture-cohort",
            "cohort_registry_sha256": "b" * 64,
            "review_registry_sha256": "c" * 64,
            "decision_sha256": "d" * 64,
            "evidence_index_sha256": "e" * 64,
            "status": runner.CAPTURE_ELIGIBLE_COHORT_STATUS,
            "source": {"commit": runner.BASELINE_COMMIT},
            "tools": {},
            "fresh_baseline_capture_eligible": True,
            "baseline_replay_eligible": True,
            "candidate_comparison_eligible": False,
            "checkpoint_reference_approval": "none",
            "product_acceptance_eligible": False,
            "acceptance_complete": False,
            "binaries": {app: {"path": str(root / "bins" / app), "sha256": "f" * 64} for app in runner.APPS},
        }
        manifest = {"generated_case_ids": [extension], "checkpoint_obligations": {extension: ["initial", "done"]}}
        original_inventory = {name: ignored for name, ignored in inventory.items() if name != extension}
        with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "validate_extension_inventory", return_value=(original_inventory, [extension])), patch.object(runner, "ordinary_test_groups", return_value=( ["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"] )), patch.object(runner, "fresh_clean_env", return_value={"LANG": "C", "TERM": "xterm-256color", "CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"}), patch.object(runner, "fresh_tool_observation", return_value={"cargo": {"path": "/pinned/cargo", "sha256": "1" * 64}}):
            plan = runner.build_fresh_plan_payload(
                repo=repo, root=root, cohort=cohort, live_record=live_record, inventory=inventory,
                extension_manifest=manifest, chunk_size=chunk_size, term_profile=None, plan_id="1" * 32,
                created_at="2026-10-21T00:00:00+00:00", advance_budget_seconds=advance_budget_seconds,
                execution=execution,
            )
        return plan

    def _owned_plan_cli_inputs(self, root: Path, stack: ExitStack, executions: list[runner.PrepareExecution], nextest):
        plan = self._fresh_plan_fixture(root)
        inventory = {row["name"]: row["ignored"] for row in plan["inventory"]["full_sorted"]}
        extension = plan["inventory"]["visual_extensions"][0]
        manifest = {"generated_case_ids": [extension], "checkpoint_obligations": {extension: ["initial", "done"]}}
        live_record = {"test_inventory_sha256": runner.inventory_digest(inventory), "module_sha256": "a" * 64}
        stack.enter_context(patch.object(runner, "EXPECTED_SCENARIOS", 1))
        stack.enter_context(patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2))
        original_inventory = {name: ignored for name, ignored in inventory.items() if name != extension}
        stack.enter_context(patch.object(runner, "validate_extension_inventory", return_value=(original_inventory, [extension])))
        stack.enter_context(patch.object(runner, "ordinary_test_groups", return_value=( ["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"] )))

        def verify_source(_root: Path, _source: Path, *, check_work: Callable[[], None] | None = None) -> str:
            self.assertIsNotNone(check_work)
            self.assertIsInstance(check_work.__self__, runner.PrepareExecution)
            executions.append(check_work.__self__)
            check_work()
            return "frozen-source"

        def capture(_repo: Path, _cohort: str, _bins: Path, *, check_work: Callable[[], None] | None = None):
            self.assertIsNotNone(check_work)
            self.assertIs(check_work.__self__, executions[-1])
            check_work()
            cohort = dict(plan["cohort"])
            cohort["cohort_id"] = runner.PREPARE_LIVE_OWNED_COHORT
            return cohort

        def verify_live(_repo: Path, _root: Path, _source: Path, _timeout: int, *, fresh_cohort_id: str, baseline_bin_dir: Path, execution: runner.PrepareExecution | None = None):
            self.assertEqual(fresh_cohort_id, runner.PREPARE_LIVE_OWNED_COHORT)
            self.assertIsNotNone(execution)
            execution.check_work()
            executions.append(execution)
            return root / "live-source", live_record

        def load_manifest(_path: Path, *, check_work: Callable[[], None] | None = None):
            self.assertIsNotNone(check_work)
            check_work()
            return manifest

        def observe(_source: Path, _env: dict[str, str], *, execution: runner.PrepareExecution | None = None):
            self.assertTrue(executions)
            self.assertIs(execution, executions[-1])
            execution.check_work()
            return {"cargo": {"path": "/pinned/cargo", "sha256": "1" * 64}}

        stack.enter_context(patch.object(runner, "verify_source", side_effect=verify_source))
        stack.enter_context(patch.object(runner, "require_capture_eligible_cohort", side_effect=capture))
        stack.enter_context(patch.object(runner, "verify_live_source", side_effect=verify_live))
        stack.enter_context(patch.object(runner, "load_extension_manifest", side_effect=load_manifest))
        stack.enter_context(patch.object(runner, "fresh_tool_observation", side_effect=observe))
        stack.enter_context(patch.object(runner, "nextest_inventory", side_effect=lambda *_args, execution=None, **_kwargs: nextest(execution, inventory)))
        return plan, inventory

    def _fresh_advance_fixture(self, base: Path, *, chunk_size: int = 1) -> dict[str, object]:
        base = base.resolve()
        root = base / "runner-root"
        root.mkdir()
        plan = self._fresh_plan_fixture(root, chunk_size=chunk_size)
        plan_path = root / runner.FRESH_PLAN_FILE
        creation_sha256 = runner._atomic_create_plan(plan_path, plan)
        bins = root / "bins"
        bins.mkdir()
        for app in runner.APPS:
            (bins / app).write_bytes(f"fixture-{app}".encode())
        key_dir = base / "custody"
        key_dir.mkdir(mode=0o700)
        key_path = key_dir / "run.key"
        repo = root / "repo"
        source = root / "source"
        stdout = io.StringIO()
        with (
            patch.object(runner, "EXPECTED_SCENARIOS", 1),
            patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2),
            patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])),
            patch.object(runner, "verify_fresh_plan", return_value=plan),
            patch.object(runner, "paths", return_value=(repo, root, source)),
            patch.object(runner, "_git_metadata_paths", return_value=set()),
            patch.object(runner.sys, "stdout", stdout),
        ):
            manifest = runner.create_fresh_run(repo, root, "fixture-cohort", bins, 30, creation_sha256, key_path)
        return {
            "base": base,
            "root": root,
            "repo": repo,
            "source": source,
            "live": root / "live-source",
            "plan": plan,
            "plan_path": plan_path,
            "creation_sha256": creation_sha256,
            "bins": bins,
            "key_path": key_path,
            "manifest": manifest,
            "run_root": root / "fresh-runs" / plan["plan_id"],
        }

    def _advance_fresh_fixture(self, fixture: dict[str, object], run_live_chunk, **patches):
        context = ExitStack()
        stdout = io.StringIO()
        fixture["advance_stdout"] = stdout
        context.enter_context(patch.object(runner.sys, "stdout", stdout))
        context.enter_context(patch.object(runner, "EXPECTED_SCENARIOS", 1))
        context.enter_context(patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2))
        context.enter_context(patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])))
        context.enter_context(patch.object(runner, "paths", return_value=(fixture["repo"], fixture["root"], fixture["source"])))
        context.enter_context(patch.object(runner, "verify_fresh_run", return_value=fixture["manifest"]))
        context.enter_context(patch.object(runner, "verify_live_source", return_value=(fixture["live"], {"source_sha256": fixture["manifest"]["source"]["source_sha256"]})))
        context.enter_context(patch.object(runner, "phase_binary_hashes", return_value=fixture["manifest"]["binary_hashes"]))
        context.enter_context(patch.object(runner, "verify_source", return_value=fixture["manifest"]["source"]["source_sha256"]))
        context.enter_context(patch.object(runner, "load_extension_manifest", return_value={"generated_case_ids": [], "checkpoint_obligations": {}}))
        if run_live_chunk is not None:
            context.enter_context(patch.object(runner, "run_live_chunk", side_effect=run_live_chunk))
        for name, value in patches.items():
            context.enter_context(patch.object(runner, name, side_effect=value) if callable(value) else patch.object(runner, name, value=value))
        return context

    @staticmethod
    def _passing_live_chunk(_live, _root, runner_dir, phase, _baseline_bin_dir, expected_root, selected, chunk_index, *_args, **_kwargs):
        expected_root.mkdir(parents=True, exist_ok=True)
        (expected_root / "frame.json").write_text('{"frame":"fixture"}\n')
        report = runner_dir / phase / f"chunk-{chunk_index:05}" / "report.json"
        report.parent.mkdir(parents=True)
        report.write_text(json.dumps({"status": "passed", "selected_cases": selected}) + "\n")
        return {
            "status": "passed",
            "checkpoint_count": len(selected),
            "successful_test_events": len(selected),
            "process_receipt_count": 1,
            "process_pid_count": 1,
            "process_receipt_inventory_sha256": "a" * 64,
            "checkpoint_inventory_sha256": "b" * 64,
            "artifacts_sha256": "c" * 64,
        }

    @staticmethod
    def _fake_cargo(path: Path, *, timeout: bool) -> Path:
        path.mkdir()
        cargo = path / "cargo"
        script = "#!/usr/bin/python3\nimport os, sys, time\nopen(os.environ['FAKE_CARGO_PID_FILE'], 'w').write(str(os.getpid()))\nprint('fake nextest stdout')\nprint('fake nextest stderr', file=sys.stderr)\n"
        script += "time.sleep(5)\n" if timeout else "sys.exit(17)\n"
        cargo.write_text(script)
        cargo.chmod(0o755)
        return cargo

    def _assert_failed_attempt_preserved(self, fixture: dict[str, object], *, phase: str = "baseline-capture") -> Path:
        phase_root = Path(fixture["plan"]["execution"]["artifact_roots"][phase.replace("-", "_")])
        attempts = sorted(phase_root.glob(".attempt-*"))
        self.assertEqual(len(attempts), 1, f"attempts={attempts} phase_exists={phase_root.exists()} error={fixture.get('advance_error')} stdout={fixture['advance_stdout'].getvalue()!r}")
        attempt = attempts[0]
        marker_path = attempt / "untrusted-failure.json"
        self.assertTrue(marker_path.is_file(), marker_path)
        marker = json.loads(marker_path.read_text())
        self.assertEqual(marker["status"], "untrusted-failed-attempt")
        self.assertIn(marker["attempt_root_original_mode"], {"0555", "0700"})
        self.assertEqual(marker["attempt_root_mode_for_marker"], "0700")
        self.assertFalse((phase_root / "chunk-00000").exists())
        journal = Path(fixture["run_root"]) / "journal"
        self.assertFalse(journal.exists() and list(journal.iterdir()))
        envelopes = runner._fresh_journal_envelopes(Path(fixture["run_root"]))
        manifest_envelope, _ = runner._read_stable_json(Path(fixture["run_root"]) / runner.FRESH_RUN_MANIFEST_FILE, "fresh run manifest")
        with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
            progress = runner.validate_fresh_run_journal(envelopes, fixture["key_path"].read_bytes(), manifest_envelope, fixture["plan_path"], fixture["creation_sha256"])
        self.assertEqual(progress["completed_steps"], 0)
        self.assertFalse(progress["candidate_comparison_eligible"])
        return attempt

    def test_fresh_advance_failed_child_and_timeout_keep_attempt_and_retry_same_ordinal(self) -> None:
        for failure in ("child", "timeout"):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                fixture = self._fresh_advance_fixture(Path(temp))
                fake_bin = Path(temp) / "fake-bin"
                self._fake_cargo(fake_bin, timeout=failure == "timeout")
                pid_path = Path(temp) / "fake-cargo.pid"
                run_patch = {}
                if failure == "timeout":
                    original_run = runner.run
                    run_patch["run"] = lambda command, **kwargs: original_run(command, **{**kwargs, "timeout": 1})
                with self._advance_fresh_fixture(fixture, None, clean_env=lambda: {"PATH": str(fake_bin), "HOME": str(Path(temp) / "home"), "FAKE_CARGO_PID_FILE": str(pid_path)}, **run_patch):
                    with self.assertRaises((runner.RunnerError, runner.subprocess.TimeoutExpired)) as raised:
                        runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
                    fixture["advance_error"] = str(raised.exception)
                attempt = self._assert_failed_attempt_preserved(fixture)
                report = attempt / "runner" / "baseline-capture" / "chunk-00000" / "report.json"
                self.assertTrue(report.is_file())
                self.assertEqual(json.loads(report.read_text())["status"], "failed")
                logs = report.parent / "logs"
                self.assertTrue((logs / "stderr.log").is_file())
                self.assertTrue((logs / "stdout.jsonl.gz").is_file())
                self.assertNotIn('"journal_ordinal"', fixture["advance_stdout"].getvalue())
                self.assertTrue(pid_path.is_file())
                child_pid = int(pid_path.read_text())
                if failure == "timeout":
                    with self.assertRaises(ProcessLookupError):
                        os.kill(child_pid, 0)

                with self._advance_fresh_fixture(fixture, self._passing_live_chunk):
                    output = runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
                self.assertEqual(output["journal_ordinal"], 0)
                self.assertEqual(output["phase"], "baseline-capture")
                self.assertTrue((Path(fixture["run_root"]) / "journal" / "step-00000000.json").is_file())

    def test_fresh_advance_mutated_artifact_is_retained_as_failed_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp))
            original_verify = runner._verify_fresh_journal_artifacts
            mutated_hashes: list[str] = []

            def mutate_then_verify(plan, envelopes):
                if not envelopes:
                    return original_verify(plan, envelopes)
                phase_root = Path(plan["execution"]["artifact_roots"]["baseline_capture"])
                frame = phase_root / "chunk-00000" / "expected" / "frame.json"
                frame.chmod(0o644)
                frame.write_text('{"frame":"mutated after digest"}\n')
                frame.chmod(0o444)
                mutated_hashes.append(runner.sha256(frame))
                return original_verify(plan, envelopes)

            with self._advance_fresh_fixture(fixture, self._passing_live_chunk, _verify_fresh_journal_artifacts=mutate_then_verify):
                with self.assertRaisesRegex(runner.RunnerError, "changed|immutable"):
                    runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
            attempt = self._assert_failed_attempt_preserved(fixture)
            frame = attempt / "expected" / "frame.json"
            self.assertEqual(runner.sha256(frame), mutated_hashes[0])
            self.assertEqual(fixture["advance_stdout"].getvalue(), "")

    def test_fresh_advance_postrename_verification_failure_rolls_back_to_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp))
            phase_root = Path(fixture["plan"]["execution"]["artifact_roots"]["baseline_capture"])

            original_verify = runner._verify_fresh_journal_artifacts
            committed_hashes: list[str] = []

            def fail_verification(_plan, envelopes):
                if not envelopes:
                    return original_verify(_plan, envelopes)
                final_chunk = phase_root / "chunk-00000"
                self.assertTrue(final_chunk.is_dir())
                committed_hashes.append(runner._fresh_expected_tree_sha256(final_chunk / "runner"))
                raise runner.RunnerError("injected postrename verification failure")

            with self._advance_fresh_fixture(fixture, self._passing_live_chunk, _verify_fresh_journal_artifacts=fail_verification):
                with self.assertRaisesRegex(runner.RunnerError, "postrename verification"):
                    runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
            attempt = self._assert_failed_attempt_preserved(fixture)
            self.assertEqual(runner._fresh_expected_tree_sha256(attempt / "runner"), committed_hashes[0])
            self.assertEqual(fixture["advance_stdout"].getvalue(), "")

    def test_fresh_advance_journal_failure_rolls_back_and_keeps_ordinal_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp))
            published_hashes: list[str] = []

            def fail_append(*_args, **_kwargs):
                final_chunk = Path(fixture["plan"]["execution"]["artifact_roots"]["baseline_capture"]) / "chunk-00000"
                published_hashes.append(runner._fresh_expected_tree_sha256(final_chunk / "runner"))
                raise runner.RunnerError("injected journal publication failure")

            with self._advance_fresh_fixture(fixture, self._passing_live_chunk, _fresh_journal_append=fail_append):
                with self.assertRaisesRegex(runner.RunnerError, "journal publication"):
                    runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
            attempt = self._assert_failed_attempt_preserved(fixture)
            self.assertTrue((attempt / "runner" / "baseline-capture" / "chunk-00000" / "report.json").is_file())
            self.assertEqual(runner._fresh_expected_tree_sha256(attempt / "runner"), published_hashes[0])
            self.assertEqual(fixture["advance_stdout"].getvalue(), "")

    def test_fresh_advance_exception_after_receipt_publication_keeps_committed_step(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            fixture = self._fresh_advance_fixture(Path(temp))
            original_append = runner._fresh_journal_append

            def append_then_raise(run_root, envelope, ordinal):
                original_append(run_root, envelope, ordinal)
                raise runner.RunnerError("injected exception after immutable receipt publication")

            with self._advance_fresh_fixture(fixture, self._passing_live_chunk, _fresh_journal_append=append_then_raise):
                with self.assertRaisesRegex(runner.RunnerError, "immutable ordinal 0 was published.*verified resume"):
                    runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
            journal_path = Path(fixture["run_root"]) / "journal" / "step-00000000.json"
            self.assertTrue(journal_path.is_file())
            committed = Path(fixture["plan"]["execution"]["artifact_roots"]["baseline_capture"]) / "chunk-00000"
            self.assertTrue(committed.is_dir())
            receipt_sha = runner.sha256(journal_path)
            chunk_sha = runner._fresh_expected_tree_sha256(committed)
            failure_markers = list((Path(fixture["run_root"]) / "failed-attempts").glob("ordinal-*.json"))
            self.assertEqual(len(failure_markers), 1)
            self.assertEqual(json.loads(failure_markers[0].read_text())["status"], "untrusted-failed-publication")
            envelopes = runner._fresh_journal_envelopes(Path(fixture["run_root"]))
            key = fixture["key_path"].read_bytes()
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                result = runner.validate_fresh_run_journal(envelopes, key, runner._read_stable_json(Path(fixture["run_root"]) / runner.FRESH_RUN_MANIFEST_FILE, "fresh run manifest")[0], fixture["plan_path"], fixture["creation_sha256"])
            self.assertEqual(result["completed_steps"], 1)
            self.assertFalse(result["candidate_comparison_eligible"])
            retry = unittest.mock.Mock(side_effect=self._passing_live_chunk)
            with self._advance_fresh_fixture(fixture, retry):
                with self.assertRaisesRegex(runner.RunnerError, "unresolved post-publication failure"):
                    runner.advance_fresh_run(fixture["repo"], fixture["root"], "fixture-cohort", fixture["bins"], 30, fixture["creation_sha256"], fixture["key_path"])
            retry.assert_not_called()
            self.assertEqual(runner.sha256(journal_path), receipt_sha)
            self.assertEqual(runner._fresh_expected_tree_sha256(committed), chunk_sha)
            self.assertFalse((committed.parent / "chunk-00001").exists())

    def test_fresh_plan_is_complete_capture_only_inventory_and_verifiable(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plan = self._fresh_plan_fixture(root)
            self.assertEqual(plan["inventory"]["visual_count"], 2)
            self.assertEqual(plan["inventory"]["ordinary_count"], 2)
            self.assertEqual([phase["name"] for phase in plan["execution"]["phases"]], ["baseline-capture", "baseline-replay", "baseline-ordinary-checks"])
            self.assertFalse(plan["execution"]["candidate_comparison_eligible"])
            self.assertIsNone(plan["execution"]["candidate_phase"])
            self.assertEqual(plan["cohort"]["checkpoint_reference_approval"], "none")
            self.assertEqual(plan["inventory"]["checkpoint_obligations"][plan["inventory"]["visual_extensions"][0]]["checkpoints"], ["initial", "done"])
            path = root / runner.FRESH_PLAN_FILE
            runner._atomic_create_plan(path, plan)
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                digest = runner.sha256(path)
                self.assertEqual(runner._read_self_sealed_plan(path, digest), plan)
                with self.assertRaisesRegex(runner.RunnerError, "externally retained creation digest"):
                    runner._read_self_sealed_plan(path, "0" * 64)
            with self.assertRaisesRegex(runner.RunnerError, "refusing to replace"):
                runner._atomic_create_plan(path, plan)

    def test_fresh_plan_rejects_duplicate_zero_chunk_privilege_and_digest_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            plan = self._fresh_plan_fixture(Path(temp))
            mutations = []
            duplicate = json.loads(json.dumps(plan))
            duplicate["inventory"]["full_sorted"].append(duplicate["inventory"]["full_sorted"][0])
            mutations.append(duplicate)
            zero_chunk = json.loads(json.dumps(plan))
            zero_chunk["execution"]["phases"][0]["chunks"] = []
            mutations.append(zero_chunk)
            privilege = json.loads(json.dumps(plan))
            privilege["execution"]["candidate_comparison_eligible"] = True
            mutations.append(privilege)
            old_tuple = json.loads(json.dumps(plan))
            old_tuple["legacy_reference_decision"] = {"sha256": runner.REFERENCE_DECISIONS_SHA256}
            mutations.append(old_tuple)
            digest = json.loads(json.dumps(plan))
            digest["plan_sha256"] = "0" * 64
            mutations.append(digest)
            for altered in mutations:
                with self.subTest(mutation=altered), self.assertRaises(runner.RunnerError):
                    runner.validate_fresh_plan_document(altered)
            coherent = json.loads(json.dumps(plan))
            coherent["execution"]["chunk_size"] = 2
            visual = coherent["inventory"]["visual_selected"]
            coherent["execution"]["phases"][0]["chunks"] = runner._chunk_records(visual, 2)
            coherent["execution"]["phases"][1]["chunks"] = runner._chunk_records(visual, 2)
            coherent["execution"]["phases"][2]["groups"] = [
                {"name": "source-only", "chunks": runner._chunk_records(coherent["inventory"]["ordinary_source_only"], 2)},
                {"name": "baseline-cli", "chunks": runner._chunk_records(coherent["inventory"]["ordinary_baseline_cli"], 2)},
            ]
            coherent["plan_sha256"] = runner.canonical_json_digest({key: value for key, value in coherent.items() if key != "plan_sha256"})
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                runner.validate_fresh_plan_document(coherent)
            with tempfile.TemporaryDirectory() as temp:
                path = Path(temp) / runner.FRESH_PLAN_FILE
                runner._atomic_create_plan(path, coherent)
                with self.assertRaisesRegex(runner.RunnerError, "externally retained creation digest"):
                    original_creation_digest = runner.hashlib.sha256((json.dumps(plan, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()).hexdigest()
                    runner._read_self_sealed_plan(path, original_creation_digest)

    def test_fresh_plan_reader_rejects_symlink_and_duplicate_json_keys(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plan = self._fresh_plan_fixture(root)
            path = root / runner.FRESH_PLAN_FILE
            runner._atomic_create_plan(path, plan)
            alias = root / "plan-link.json"
            alias.symlink_to(path)
            with self.assertRaisesRegex(runner.RunnerError, "symlinked or non-regular"):
                runner._read_self_sealed_plan(alias)
            duplicate = root / "plan-duplicate.json"
            duplicate.write_text('{"schema":"x","schema":"y"}')
            with self.assertRaisesRegex(runner.RunnerError, "duplicate fresh plan JSON field"):
                runner._read_self_sealed_plan(duplicate)

    def test_fresh_environment_binds_locale_color_timezone_and_tool_bytes(self) -> None:
        baseline = {key: "stable" for key in runner.FRESH_RUNTIME_ENV_KEYS}
        baseline.pop("LC_ALL")
        expected = runner.fresh_environment_fingerprint(baseline)
        for key, value in (("LANG", "fr_FR.UTF-8"), ("LC_ALL", "C"), ("TZ", "UTC"), ("NO_COLOR", "1"), ("COLORTERM", "truecolor")):
            changed = dict(baseline)
            changed[key] = value
            self.assertNotEqual(runner.fresh_environment_fingerprint(changed), expected, key)
        env = {"PATH": "/tools", "RUSTC": "/tools/rustc"}
        observation = {"path": "/tools/cargo", "sha256": "a" * 64, "version": "cargo 1"}
        with patch.object(runner, "_selected_executable_observation", side_effect=[observation] * 4), patch.object(runner, "toolchain_provenance", return_value={"selected": True}):
            first = runner.fresh_tool_observation(Path("/src"), env)
        changed_observation = {"path": "/tools/cargo", "sha256": "b" * 64, "version": "cargo 1"}
        with patch.object(runner, "_selected_executable_observation", side_effect=[changed_observation] + [observation] * 3), patch.object(runner, "toolchain_provenance", return_value={"selected": True}):
            second = runner.fresh_tool_observation(Path("/src"), env)
        self.assertNotEqual(first, second)

    def test_fresh_plan_cli_defaults_to_bounded_timeout_and_external_anchor_required(self) -> None:
        args = runner.parser().parse_args(["fresh-plan", "--cohort", "fixture", "--baseline-bin-dir", "/bins"])
        self.assertEqual(args.timeout, runner.FRESH_PLAN_MAX_TIMEOUT)
        self.assertFalse(args.owned_lifecycle)
        self.assertIsNone(args.owned_budget)
        verify = runner.parser().parse_args(["verify-fresh-plan", "--cohort", "fixture", "--baseline-bin-dir", "/bins", "--expected-plan-sha256", "a" * 64])
        self.assertEqual(verify.expected_plan_sha256, "a" * 64)
        self.assertFalse(verify.owned_lifecycle)
        self.assertIsNone(verify.owned_budget)
        with self.assertRaises(SystemExit):
            runner.parser().parse_args(["verify-fresh-plan", "--cohort", "fixture", "--baseline-bin-dir", "/bins"])

    def test_owned_fresh_plan_budget_is_200_and_legacy_plan_budget_remains_225(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            legacy = self._fresh_plan_fixture(root / "legacy")
            with runner.PrepareExecution(200) as execution:
                owned = self._fresh_plan_fixture(root / "owned", execution=execution, advance_budget_seconds=execution.work_budget)
            self.assertEqual(legacy["execution"]["timeout_seconds_per_advance"], 225)
            self.assertEqual(owned["execution"]["timeout_seconds_per_advance"], 200)
            with self.assertRaisesRegex(runner.RunnerError, "advance budget must be"):
                self._fresh_plan_fixture(root / "invalid", advance_budget_seconds=226)

    def test_owned_fresh_plan_cli_create_and_verify_share_budgeted_scope(self) -> None:
        for command in ("fresh-plan", "verify-fresh-plan"):
            with self.subTest(command=command), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve()
                repo = root / "repo"
                executions: list[runner.PrepareExecution] = []
                inventory_holder: dict[str, bool] = {}

                def inventory(execution, expected):
                    self.assertIsNotNone(execution)
                    execution.check_work()
                    executions.append(execution)
                    inventory_holder.update(expected)
                    return expected

                with ExitStack() as stack:
                    plan_fixture, expected_inventory = self._owned_plan_cli_inputs(root, stack, executions, inventory)
                    if command == "fresh-plan":
                        argv = [
                            "frozen_baseline.py", "--repo", str(repo), "--root", str(root), command,
                            "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                            "--owned-lifecycle", "--owned-budget", "20",
                        ]
                    else:
                        # Creation in this synthetic fixture uses the same CLI path and budget.
                        create_argv = [
                            "frozen_baseline.py", "--repo", str(repo), "--root", str(root), "fresh-plan",
                            "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                            "--owned-lifecycle", "--owned-budget", "20",
                        ]
                        with patch.object(sys, "argv", create_argv), redirect_stdout(io.StringIO()):
                            self.assertEqual(runner.main(), 0)
                        plan = runner._read_self_sealed_plan(root / runner.FRESH_PLAN_FILE)
                        digest = runner.sha256(root / runner.FRESH_PLAN_FILE)
                        self.assertEqual(plan["execution"]["timeout_seconds_per_advance"], 20)
                        argv = [
                            "frozen_baseline.py", "--repo", str(repo), "--root", str(root), command,
                            "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                            "--expected-plan-sha256", digest, "--owned-lifecycle", "--owned-budget", "20",
                        ]
                    with patch.object(sys, "argv", argv), redirect_stdout(io.StringIO()) as output:
                        self.assertEqual(runner.main(), 0)
                    self.assertIn('"status": "prepared-not-executed"' if command == "fresh-plan" else '"verified": true', output.getvalue())
                    self.assertTrue(executions)
                    self.assertTrue(all(item.work_budget == 20 for item in executions))
                    self.assertEqual(inventory_holder, expected_inventory)
                    if command == "fresh-plan":
                        plan = runner._read_self_sealed_plan(root / runner.FRESH_PLAN_FILE)
                        self.assertEqual(plan["execution"]["timeout_seconds_per_advance"], 20)

    def test_owned_fresh_plan_nextest_deadline_reaps_child_and_publishes_no_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            repo = root / "repo"
            executions: list[runner.PrepareExecution] = []

            def hanging_inventory(execution, _expected):
                self.assertIsNotNone(execution)
                executions.append(execution)
                execution.run([sys.executable, "-c", "import time; time.sleep(60)"], cwd=root, quiet=True)
                self.fail("owned nextest child should be stopped at work deadline")

            with ExitStack() as stack:
                self._owned_plan_cli_inputs(root, stack, executions, hanging_inventory)
                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(root), "fresh-plan",
                    "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                    "--owned-lifecycle", "--owned-budget", "1",
                ]
                with patch.object(sys, "argv", argv), redirect_stdout(io.StringIO()) as output, redirect_stderr(io.StringIO()) as errors:
                    started = time.monotonic()
                    self.assertEqual(runner.main(), 1)
                    elapsed = time.monotonic() - started
            self.assertLess(elapsed, 16)
            self.assertIn("work deadline expired", errors.getvalue())
            self.assertNotIn('"status": "prepared-not-executed"', output.getvalue())
            self.assertFalse((root / runner.FRESH_PLAN_FILE).exists())
            self.assertTrue(executions[-1]._children == {})

    def test_fresh_plan_owned_budget_flag_requires_owned_lifecycle(self) -> None:
        for budget in (20, runner.PREPARE_OWNED_MAX_BUDGET):
            with self.subTest(budget=budget), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve()
                argv = [
                    "frozen_baseline.py", "--repo", str(root / "repo"), "--root", str(root), "fresh-plan",
                    "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                    "--owned-budget", str(budget),
                ]
                with patch.object(sys, "argv", argv), redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(runner.main(), 1)
                self.assertIn("--owned-budget requires --owned-lifecycle", errors.getvalue())
                self.assertFalse((root / runner.FRESH_PLAN_FILE).exists())

    def test_owned_fresh_plan_deadline_during_hashing_prevents_plan_and_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            repo = root / "repo"
            executions: list[runner.PrepareExecution] = []
            original_digest = runner.digest_source

            def late_digest(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
                if path == root / "source":
                    time.sleep(1.05)
                return original_digest(path, check_work=check_work)

            def inventory(execution, expected):
                self.assertIsNotNone(execution)
                executions.append(execution)
                execution.check_work()
                return expected

            with ExitStack() as stack:
                self._owned_plan_cli_inputs(root, stack, executions, inventory)
                stack.enter_context(patch.object(runner, "digest_source", side_effect=late_digest))
                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(root), "fresh-plan",
                    "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                    "--owned-lifecycle", "--owned-budget", "1",
                ]
                with patch.object(sys, "argv", argv), redirect_stdout(io.StringIO()) as output, redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(runner.main(), 1)
            self.assertIn("work deadline expired", errors.getvalue())
            self.assertNotIn('"status": "prepared-not-executed"', output.getvalue())
            self.assertFalse((root / runner.FRESH_PLAN_FILE).exists())

    def test_owned_fresh_plan_post_publication_deadline_keeps_immutable_plan_without_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            repo = root / "repo"
            executions: list[runner.PrepareExecution] = []
            original_publish = runner._atomic_create_plan

            def late_publication(path: Path, document: dict[str, object], *, check_work: Callable[[], None] | None = None) -> str:
                digest = original_publish(path, document, check_work=check_work)
                time.sleep(1.05)
                assert check_work is not None
                check_work()
                return digest

            def inventory(execution, expected):
                self.assertIsNotNone(execution)
                executions.append(execution)
                execution.check_work()
                return expected

            with ExitStack() as stack:
                self._owned_plan_cli_inputs(root, stack, executions, inventory)
                stack.enter_context(patch.object(runner, "_atomic_create_plan", side_effect=late_publication))
                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(root), "fresh-plan",
                    "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                    "--owned-lifecycle", "--owned-budget", "1",
                ]
                with patch.object(sys, "argv", argv), redirect_stdout(io.StringIO()) as output, redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(runner.main(), 1)
            self.assertIn("work deadline expired", errors.getvalue())
            self.assertNotIn('"status": "prepared-not-executed"', output.getvalue())
            self.assertTrue((root / runner.FRESH_PLAN_FILE).is_file())
            self.assertEqual(list(root.glob(".fresh-capture-plan.json.*")), [])

    def test_owned_fresh_plan_final_output_is_flushed_then_postchecked(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            repo = root / "repo"
            executions: list[runner.PrepareExecution] = []
            original_print = print

            def late_success(*args: object, **kwargs: object) -> None:
                if args and isinstance(args[0], str) and '"status": "prepared-not-executed"' in args[0]:
                    self.assertTrue(kwargs.get("flush"))
                    original_print(*args, **kwargs)
                    signal.raise_signal(signal.SIGTERM)
                    return
                original_print(*args, **kwargs)

            def inventory(execution, expected):
                self.assertIsNotNone(execution)
                executions.append(execution)
                execution.check_work()
                return expected

            with ExitStack() as stack:
                self._owned_plan_cli_inputs(root, stack, executions, inventory)
                stack.enter_context(patch("builtins.print", side_effect=late_success))
                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(root), "fresh-plan",
                    "--cohort", runner.PREPARE_LIVE_OWNED_COHORT, "--baseline-bin-dir", str(root / "bins"),
                    "--owned-lifecycle", "--owned-budget", "20",
                ]
                with patch.object(sys, "argv", argv), redirect_stdout(io.StringIO()) as output, redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(runner.main(), 1)
            self.assertIn('"status": "prepared-not-executed"', output.getvalue())
            self.assertIn("cancelled by signal", errors.getvalue())
            self.assertTrue((root / runner.FRESH_PLAN_FILE).is_file())

    def test_fresh_plan_creation_digest_is_precomputed_before_publish_race(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            plan = self._fresh_plan_fixture(root)
            inventory = {row["name"]: row["ignored"] for row in plan["inventory"]["full_sorted"]}
            repo = root / "repo"
            source = root / "source"
            live = root / "live-source"
            path = root / runner.FRESH_PLAN_FILE
            original_reader = runner._read_self_sealed_plan

            def rewrite_after_stable_read(plan_path: Path, expected_sha256: str | None = None) -> dict[str, object]:
                parsed = original_reader(plan_path, expected_sha256)
                changed = json.loads(json.dumps(plan))
                changed["execution"]["chunk_size"] = 2
                visual = changed["inventory"]["visual_selected"]
                changed["execution"]["phases"][0]["chunks"] = runner._chunk_records(visual, 2)
                changed["execution"]["phases"][1]["chunks"] = runner._chunk_records(visual, 2)
                changed["execution"]["phases"][2]["groups"] = [
                    {"name": "source-only", "chunks": runner._chunk_records(changed["inventory"]["ordinary_source_only"], 2)},
                    {"name": "baseline-cli", "chunks": runner._chunk_records(changed["inventory"]["ordinary_baseline_cli"], 2)},
                ]
                changed["plan_sha256"] = runner.canonical_json_digest({key: value for key, value in changed.items() if key != "plan_sha256"})
                plan_path.chmod(0o600)
                plan_path.write_text(json.dumps(changed, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n")
                return parsed

            stdout = io.StringIO()
            with (
                patch.object(runner, "EXPECTED_SCENARIOS", 1),
                patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2),
                patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])),
                patch.object(runner, "paths", return_value=(repo, root, source)),
                patch.object(runner, "verify_source"),
                patch.object(runner, "require_capture_eligible_cohort", return_value=plan["cohort"]),
                patch.object(runner, "verify_live_source", return_value=(live, {"test_inventory_sha256": runner.inventory_digest(inventory)})),
                patch.object(runner, "load_extension_manifest", return_value={"generated_case_ids": plan["inventory"]["visual_extensions"], "checkpoint_obligations": {}}),
                patch.object(runner, "fresh_clean_env", return_value={"LANG": "C"}),
                patch.object(runner, "nextest_inventory", return_value=inventory),
                patch.object(runner, "require_inventory_matches"),
                patch.object(runner, "build_fresh_plan_payload", return_value=plan),
                patch.object(runner, "_read_self_sealed_plan", side_effect=rewrite_after_stable_read),
                patch.object(sys, "stdout", stdout),
            ):
                result = runner.create_fresh_plan(repo, root, "fixture-cohort", root / "bins", 10, 1, None)
            self.assertEqual(result, plan)
            report = json.loads(stdout.getvalue())
            original_bytes = (json.dumps(plan, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
            self.assertEqual(report["creation_file_sha256"], runner.hashlib.sha256(original_bytes).hexdigest())
            with self.assertRaisesRegex(runner.RunnerError, "externally retained creation digest"):
                original_reader(path, report["creation_file_sha256"])

    def test_fresh_run_prepared_manifest_is_authenticated_and_nonaccepting(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve() / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            plan_path = root / runner.FRESH_PLAN_FILE
            creation_digest = runner._atomic_create_plan(plan_path, plan)
            key_path = Path(temp).resolve() / "private-custody" / "run.key"
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(Path(temp) / "repo", root, root / "source")), patch.object(runner, "_git_metadata_paths", return_value=set()), patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(['source_only::fixture'], ['modes::showcase_boot_sets_terminal_modes'])):
                manifest = runner.create_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                self.assertFalse(manifest["executed"])
                self.assertFalse(manifest["candidate_comparison_eligible"])
                self.assertFalse(manifest["product_acceptance"])
                self.assertEqual(manifest["checkpoint_reference_approval"], "none")
                self.assertEqual(key_path.stat().st_mode & 0o777, 0o600)
                key = key_path.read_bytes()
                self.assertEqual(len(key), 32)
                self.assertNotIn(key.hex(), (root / "fresh-runs" / plan["plan_id"] / runner.FRESH_RUN_MANIFEST_FILE).read_text())
                verified = runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                self.assertEqual(verified, manifest)
                key_path.parent.chmod(0o755)
                with self.assertRaisesRegex(runner.RunnerError, "owned by this user and mode 0700"):
                    runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                self.assertEqual(key_path.parent.stat().st_mode & 0o777, 0o755)
                key_path.parent.chmod(0o700)
                with self.assertRaisesRegex(runner.RunnerError, "already exists"):
                    runner.create_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)

    def test_fresh_run_rejects_tampered_stale_or_untrusted_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve() / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            creation_digest = runner._atomic_create_plan(root / runner.FRESH_PLAN_FILE, plan)
            key_path = Path(temp).resolve() / "custody" / "run.key"
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(Path(temp) / "repo", root, root / "source")), patch.object(runner, "_git_metadata_paths", return_value=set()):
                runner.create_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                manifest_path = root / "fresh-runs" / plan["plan_id"] / runner.FRESH_RUN_MANIFEST_FILE
                original = manifest_path.read_bytes()
                envelope = json.loads(original)
                envelope["manifest"]["executed"] = True
                manifest_path.chmod(0o600)
                manifest_path.write_bytes(runner.canonical_receipt_json(envelope))
                with self.assertRaisesRegex(runner.RunnerError, "authentication failed"):
                    runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                manifest_path.write_bytes(original)
                with self.assertRaisesRegex(runner.RunnerError, "externally retained creation digest"):
                    with patch.object(runner, "verify_fresh_plan", side_effect=runner.RunnerError("fresh plan does not match externally retained creation digest")):
                        runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, "0" * 64, key_path)
                wrong_key = Path(temp).resolve() / "custody" / "wrong.key"
                wrong_key.write_bytes(b"x" * 32)
                wrong_key.chmod(0o600)
                with self.assertRaisesRegex(runner.RunnerError, "authentication failed"):
                    runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, wrong_key)
                alias = Path(temp).resolve() / "custody" / "key-link"
                alias.symlink_to(key_path)
                with self.assertRaisesRegex(runner.RunnerError, "private 0600 regular file"):
                    runner.verify_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, alias)

    def test_fresh_run_key_cannot_live_inside_source_or_artifact_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve() / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            digest = runner._atomic_create_plan(root / runner.FRESH_PLAN_FILE, plan)
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(Path(temp) / "repo", root, root / "source")), patch.object(runner, "_git_metadata_paths", return_value=set()):
                for key_path in (root / "private.key", Path(plan["source"]["source_path"]) / "private.key", Path(plan["execution"]["artifact_roots"]["run_root"]) / "private.key"):
                    with self.subTest(key_path=key_path), self.assertRaisesRegex(runner.RunnerError, "outside Git, source, binaries"):
                        runner.create_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, digest, key_path)

    def test_fresh_run_key_requires_owned_private_custody_directory_without_chmod(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            unsafe = base / "unsafe-custody"
            unsafe.mkdir(mode=0o755)
            unsafe.chmod(0o755)
            key_path = unsafe.resolve() / "run.key"
            with self.assertRaisesRegex(runner.RunnerError, "owned by this user and mode 0700"):
                runner._stable_secret_key(key_path, create=True)
            self.assertEqual(unsafe.stat().st_mode & 0o777, 0o755)
            self.assertFalse(key_path.exists())
            secure = base / "secure-custody"
            secure.mkdir(mode=0o700)
            alias = base / "custody-alias"
            alias.symlink_to(secure, target_is_directory=True)
            with self.assertRaisesRegex(runner.RunnerError, "symlinked ancestor"):
                runner._stable_secret_key(alias / "run.key", create=True)
            root = base / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            digest = runner._atomic_create_plan(root / runner.FRESH_PLAN_FILE, plan)
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(base / "repo", root, root / "source")), patch.object(runner, "_git_metadata_paths", return_value=set()):
                with self.assertRaisesRegex(runner.RunnerError, "symlinked ancestor"):
                    runner.create_fresh_run(base / "repo", root, "fixture-cohort", root / "bins", 30, digest, alias / "run.key")

    def test_fresh_run_key_rejects_linked_worktree_and_git_common_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            main = base / "main"
            linked = base / "linked"
            main.mkdir()
            subprocess = runner.subprocess
            subprocess.run(["git", "init", str(main)], check=True, capture_output=True, timeout=20)
            subprocess.run(["git", "-C", str(main), "config", "user.email", "test@example.invalid"], check=True, capture_output=True, timeout=20)
            subprocess.run(["git", "-C", str(main), "config", "user.name", "fixture"], check=True, capture_output=True, timeout=20)
            (main / "README").write_text("fixture\n")
            subprocess.run(["git", "-C", str(main), "add", "README"], check=True, capture_output=True, timeout=20)
            subprocess.run(["git", "-C", str(main), "commit", "-m", "fixture"], check=True, capture_output=True, timeout=20)
            subprocess.run(["git", "-C", str(main), "worktree", "add", str(linked), "HEAD"], check=True, capture_output=True, timeout=20)
            metadata = runner._git_metadata_paths(linked)
            self.assertEqual(len(metadata), 2)
            common = next(path for path in metadata if path == (main / ".git").resolve())
            linked_git = next(path for path in metadata if path != common)
            root = base / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            digest = runner._atomic_create_plan(root / runner.FRESH_PLAN_FILE, plan)
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(linked, root, root / "source")):
                for git_path in (common, linked_git):
                    with self.subTest(git_path=git_path), self.assertRaisesRegex(runner.RunnerError, "outside Git, source, binaries"):
                        runner.create_fresh_run(linked, root, "fixture-cohort", root / "bins", 30, digest, git_path / "secret" / "run.key")

    def test_fresh_run_journal_requires_ordered_authenticated_nonzero_complete_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve() / "runner-root"
            root.mkdir()
            plan = self._fresh_plan_fixture(root)
            creation_digest = runner._atomic_create_plan(root / runner.FRESH_PLAN_FILE, plan)
            key_path = Path(temp).resolve() / "custody" / "run.key"
            with patch.object(runner, "verify_fresh_plan", return_value=plan), patch.object(runner, "paths", return_value=(Path(temp) / "repo", root, root / "source")), patch.object(runner, "_git_metadata_paths", return_value=set()):
                manifest = runner.create_fresh_run(Path(temp) / "repo", root, "fixture-cohort", root / "bins", 30, creation_digest, key_path)
                manifest_path = root / "fresh-runs" / plan["plan_id"] / runner.FRESH_RUN_MANIFEST_FILE
                manifest_envelope = json.loads(manifest_path.read_text())
            key = key_path.read_bytes()
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                plan_path = root / runner.FRESH_PLAN_FILE
                loaded_plan = runner._read_self_sealed_plan(plan_path, creation_digest)
                planned = runner._fresh_plan_steps(loaded_plan)
            def signed(step: dict[str, object], ordinal: int, previous: str | None, **changes: object) -> dict[str, object]:
                expected_hash = "a" * 64
                receipt = {
                    "schema": "termrock.frozen-cli-fresh-run-receipt-v2", "ordinal": ordinal,
                    **step, "status": "passed", "checkpoint_count": 1 if step["phase"] != "baseline-ordinary-checks" else 0,
                    "ordinary_check_count": 1 if step["phase"] == "baseline-ordinary-checks" else 0,
                    "process_receipt_count": 1 if step["phase"] != "baseline-ordinary-checks" else 0,
                    "process_receipt_inventory_sha256": "b" * 64,
                    "process_pid_count": 1 if step["phase"] != "baseline-ordinary-checks" else 0,
                    "checkpoint_inventory_sha256": "c" * 64,
                    "artifacts_sha256": "d" * 64,
                    "committed_chunk_sha256": "e" * 64,
                    "expected_chunk_sha256": expected_hash,
                    "run_id": manifest["run_id"], "plan_creation_sha256": creation_digest,
                    "plan_self_sha256": plan["plan_sha256"], "cohort_id": manifest["cohort_id"],
                    "binary_hashes": manifest["binary_hashes"], "source_sha256": manifest["source"]["source_sha256"],
                    "runtime_sha256": runner.canonical_json_digest(manifest["runtime"]), "previous_receipt_sha256": previous,
                }
                receipt.update(changes)
                return {"receipt": receipt, "hmac_sha256": runner.hmac.new(key, runner.canonical_receipt_json(receipt), runner.hashlib.sha256).hexdigest()}
            chain = []
            previous = None
            visual_planned = [step for step in planned if step["phase"] != "baseline-ordinary-checks"]
            for ordinal, step in enumerate(visual_planned):
                envelope = signed(step, ordinal, previous)
                chain.append(envelope)
                previous = runner.hashlib.sha256(runner.canonical_receipt_json(envelope)).hexdigest()
            first, second, third = chain[:3]
            altered_replay_hash = signed(visual_planned[2], 2, runner.hashlib.sha256(runner.canonical_receipt_json(second)).hexdigest(), expected_chunk_sha256="0" * 64)
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                empty = runner.validate_fresh_run_journal([], key, manifest_envelope, plan_path, creation_digest)
                self.assertFalse(empty["complete"])
                with self.assertRaises(TypeError):
                    runner.validate_fresh_run_journal([], key, manifest_envelope, plan_path, creation_digest, [])
                forged_manifest = json.loads(json.dumps(manifest_envelope))
                forged_manifest["manifest"]["inventory_sha256"] = "0" * 64
                with self.assertRaisesRegex(runner.RunnerError, "manifest authentication failed"):
                    runner.validate_fresh_run_journal([], key, forged_manifest, plan_path, creation_digest)
                prefix = runner.validate_fresh_run_journal([first], key, manifest_envelope, plan_path, creation_digest)
                complete = runner.validate_fresh_run_journal(chain, key, manifest_envelope, plan_path, creation_digest)
                ordinary_receipt = signed(planned[len(chain)], len(chain), previous)
                with self.assertRaisesRegex(runner.RunnerError, "unsupported phase"):
                    runner.validate_fresh_run_journal([*chain, ordinary_receipt], key, manifest_envelope, plan_path, creation_digest)
                forged = json.loads(json.dumps(loaded_plan))
                forged["execution"]["phases"][0]["chunks"][0]["selected_cases"] = ["arbitrary::not-in-inventory"]
                forged["plan_sha256"] = runner.canonical_json_digest({field: value for field, value in forged.items() if field != "plan_sha256"})
                forged_path = root / "forged-plan.json"
                forged_digest = runner._atomic_create_plan(forged_path, forged)
                with self.assertRaisesRegex(runner.RunnerError, "chunk membership differs"):
                    runner.validate_fresh_run_journal([first], key, manifest_envelope, forged_path, forged_digest)
                other_plan = json.loads(json.dumps(loaded_plan))
                other_plan["plan_id"] = "2" * 32
                other_plan["plan_sha256"] = runner.canonical_json_digest({field: value for field, value in other_plan.items() if field != "plan_sha256"})
                other_path = root / "different-plan.json"
                other_digest = runner._atomic_create_plan(other_path, other_plan)
                with self.assertRaisesRegex(runner.RunnerError, "differs from the authenticated manifest"):
                    runner.validate_fresh_run_journal([first], key, manifest_envelope, other_path, other_digest)
            self.assertFalse(prefix["complete"])
            self.assertFalse(prefix["candidate_comparison_eligible"])
            self.assertTrue(complete["visual_capture_replay_complete"])
            self.assertFalse(complete["complete"])
            for bad in (
                [chain[1], chain[0], *chain[2:]],
                [chain[0], chain[0], *chain[2:]],
                [chain[0], *chain[2:]],
                [*chain, chain[-1]],
                [{"receipt": first["receipt"], "hmac_sha256": "0" * 64}],
                [signed(planned[0], 0, None, status="skipped")],
                [signed(planned[0], 0, None, checkpoint_count=0)],
                [first, second, altered_replay_hash],
                [signed(planned[0], 0, None, run_id="stale-run")],
                [signed(planned[0], 1, None)],
            ):
                with self.subTest(bad=bad), self.assertRaises(runner.RunnerError):
                    with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 2), patch.object(runner, "ordinary_test_groups", return_value=(["source_only::fixture"], ["modes::showcase_boot_sets_terminal_modes"])):
                        runner.validate_fresh_run_journal(bad, key, manifest_envelope, plan_path, creation_digest)

    def test_fresh_prepare_mode_does_not_consult_legacy_build_or_tuple_approval(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            source = root / "source"
            for directory in (repo / "tools" / "frozen-cli-frame-support" / "src", source / "snapshots", source / "tests" / "visual_baseline" / "scenarios"):
                directory.mkdir(parents=True)
            patch_path = repo / "tools" / "frozen_cli_pair.patch"
            helper_path = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
            extension_manifest_path = repo / runner.EXTENSION_MANIFEST
            extension_source_path = repo / runner.EXTENSION_SOURCE
            for path in (patch_path, helper_path, extension_manifest_path, extension_source_path):
                path.write_text("fixture")
            expected_changes = {
                "tests/visual_baseline/main.rs", "tests/visual_baseline/support.rs",
                "tests/visual_baseline/scenarios/mod.rs", "tests/visual_baseline/scenarios/interact.rs",
                "tests/visual_baseline/frozen_cli_pair.rs", "tests/visual_baseline/frozen_cli_extensions.rs",
                "tests/visual_baseline/frozen_cli_extensions.json",
            }
            initial_manifest = {}
            final_manifest = {name: "fixture" for name in expected_changes}
            extension_case = "termrock_extensions::showcase::table_sort::fixture::80x24::truecolor"
            inventory = {"showcase::pages::fixture::80x24::truecolor": True, extension_case: True}
            capture = {"cohort_id": "fixture-cohort", "fresh_baseline_capture_eligible": True}

            def fake_digest(path: Path, **_kwargs: object) -> str:
                if path == patch_path:
                    return runner.EXTENSION_PATCH_SHA256
                if path == extension_manifest_path:
                    return runner.EXTENSION_MANIFEST_SHA256
                if path == extension_source_path:
                    return runner.EXTENSION_SOURCE_SHA256
                if path == helper_path:
                    return "a" * 64
                return "b" * 64

            with (
                patch.object(runner, "load_reference_decisions", side_effect=AssertionError("legacy tuple/build loader must not be called")) as legacy,
                patch.object(runner, "_fresh_cohort_source_record", return_value=capture),
                patch.object(runner, "load_extension_manifest", return_value={"generated_case_ids": [extension_case], "checkpoint_obligations": {extension_case: ["initial"]}}),
                patch.object(runner, "file_manifest", side_effect=[initial_manifest, final_manifest]),
                patch.object(runner, "digest_tree", return_value="c" * 64),
                patch.object(runner, "digest_source", return_value="d" * 64),
                patch.object(runner, "freeze_tree_readonly"),
                patch.object(runner, "run"),
                patch.object(runner, "nextest_inventory", return_value=inventory),
                patch.object(runner, "validate_extension_inventory", return_value=(inventory, [extension_case])),
                patch.object(runner, "sha256", side_effect=fake_digest),
                patch.object(runner, "fresh_clean_env", return_value={"PATH": "/tools", "LANG": "C"}),
                patch.object(runner, "fresh_tool_observation", return_value={"cargo": "fixture"}),
            ):
                result = runner.prepare_live_source(repo, root, source, 10, fresh_cohort_id="fixture-cohort", baseline_bin_dir=root / "sealed")
            legacy.assert_not_called()
            self.assertEqual(result["fresh_capture_cohort"], capture)
            self.assertIsNone(result["reference_decisions_sha256"])

    def test_fresh_live_verification_does_not_load_legacy_tuple_decision(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            source = root / "source"
            live = root / "live-source"
            for directory in (source / "snapshots", live / "snapshots", live / "tests" / "visual_baseline"):
                directory.mkdir(parents=True, exist_ok=True)
            helper = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
            patch_file = repo / "tools" / "frozen_cli_pair.patch"
            extension_manifest = repo / runner.EXTENSION_MANIFEST
            extension_source = repo / runner.EXTENSION_SOURCE
            for path in (helper, patch_file, extension_manifest, extension_source):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("fixture")
            live_module = live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"
            live_extension_module = live / "tests" / "visual_baseline" / "frozen_cli_extensions.rs"
            live_manifest_copy = live / "tests" / "visual_baseline" / "frozen_cli_extensions.json"
            for path in (live_module, live_extension_module, live_manifest_copy):
                path.write_text("fixture")
            capture = {"cohort_id": "fixture-cohort", "binary_sha256": {app: "a" * 64 for app in runner.APPS}}
            fresh_env = {"PATH": "/tools", "LANG": "C", "CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"}
            record = {
                "frozen_source_sha256": "frozen", "live_source_sha256": "live",
                "patch_sha256": runner.EXTENSION_PATCH_SHA256, "helper_sha256": "helper",
                "module_sha256": "module", "extension_manifest_sha256": runner.EXTENSION_MANIFEST_SHA256,
                "extension_source_sha256": runner.EXTENSION_SOURCE_SHA256,
                "extension_module_sha256": "extension-module", "extension_manifest_copy_sha256": "manifest-copy",
                "reference_decisions_sha256": None, "fresh_capture_cohort": capture,
                "frozen_snapshots_sha256": "snapshots", "live_snapshots_sha256": "snapshots",
                "preparation_environment": runner.fresh_environment_fingerprint(fresh_env),
                "preparation_tool_observation": {"cargo": "fixture"},
            }
            (root / "live-source.json").write_text(json.dumps(record))

            active_execution: list[runner.PrepareExecution] = []
            checked_methods: list[Callable[[], None]] = []
            observed_executions: list[runner.PrepareExecution | None] = []

            def observe_check(check_fn: Callable[[], None] | None) -> None:
                self.assertIsNotNone(check_fn)
                self.assertIs(check_fn.__self__, active_execution[-1])
                checked_methods.append(check_fn)
                check_fn()

            def fake_sha(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
                if active_execution:
                    observe_check(check_work)
                return {
                    patch_file: runner.EXTENSION_PATCH_SHA256,
                    helper: "helper",
                    live_module: "module",
                    extension_manifest: runner.EXTENSION_MANIFEST_SHA256,
                    extension_source: runner.EXTENSION_SOURCE_SHA256,
                    live_extension_module: "extension-module",
                    live_manifest_copy: "manifest-copy",
                }.get(path, "unused")

            def fake_source_digest(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
                if active_execution:
                    observe_check(check_work)
                return "frozen" if path == source else "live"

            def fake_tree_digest(_path: Path, *, check_work: Callable[[], None] | None = None) -> str:
                if active_execution:
                    observe_check(check_work)
                return "snapshots"

            def observe_tools(_source: Path, _env: dict[str, str], *, execution: runner.PrepareExecution | None = None) -> dict[str, str]:
                observed_executions.append(execution)
                return {"cargo": "fixture"}

            def observe_cohort(_repo: Path, _cohort: str, _bins: Path, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
                if active_execution:
                    observe_check(check_work)
                return capture

            with (
                patch.object(runner, "load_reference_decisions", side_effect=AssertionError("fresh source must not consult old tuple/build provenance")) as legacy,
                patch.object(runner, "_fresh_cohort_source_record", side_effect=observe_cohort),
                patch.object(runner, "digest_source", side_effect=fake_source_digest),
                patch.object(runner, "digest_tree", side_effect=fake_tree_digest),
                patch.object(runner, "sha256", side_effect=fake_sha),
                patch.object(runner, "fresh_clean_env", return_value={"PATH": "/tools", "LANG": "C"}),
                patch.object(runner, "fresh_tool_observation", side_effect=observe_tools),
            ):
                verified_live, verified_record = runner.verify_live_source(
                    repo, root, source, 10, fresh_cohort_id="fixture-cohort", baseline_bin_dir=root / "sealed",
                )
                with runner.PrepareExecution(20) as execution:
                    active_execution.append(execution)
                    owned_live, owned_record = runner.verify_live_source(
                        repo, root, source, 10, fresh_cohort_id="fixture-cohort",
                        baseline_bin_dir=root / "sealed", execution=execution,
                    )
                    execution.check_work()
            legacy.assert_not_called()
            self.assertEqual(verified_live, live)
            self.assertEqual(verified_record["fresh_capture_cohort"], capture)
            self.assertEqual(owned_live, live)
            self.assertEqual(owned_record["fresh_capture_cohort"], capture)
            self.assertTrue(checked_methods)
            self.assertTrue(all(method.__self__ is active_execution[-1] for method in checked_methods))
            self.assertEqual(observed_executions, [None, active_execution[-1]])

    def test_verify_live_owned_budget_requires_lifecycle_flag(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            frozen = root / "frozen"
            repo.mkdir()
            (frozen / "source").mkdir(parents=True)
            (frozen / "source" / "Cargo.toml").write_text("[workspace]\n")
            argv = [
                "frozen_baseline.py", "--repo", str(repo), "--root", str(frozen),
                "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
                "--baseline-bin-dir", str(root / "sealed"), "--owned-budget", "20",
            ]
            with patch.object(sys, "argv", argv), redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(runner.main(), 1)
            self.assertIn("--owned-budget requires --owned-lifecycle", errors.getvalue())

    def test_verify_live_owned_cli_shares_execution_through_final_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            frozen = root / "frozen"
            source = frozen / "source"
            repo.mkdir()
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("[workspace]\n")
            live = frozen / "live-source"
            checks: list[str] = []
            active_execution: list[runner.PrepareExecution] = []
            original_enter = runner.PrepareExecution.__enter__

            def enter_execution(execution: runner.PrepareExecution) -> runner.PrepareExecution:
                entered = original_enter(execution)
                active_execution.append(execution)
                return entered

            def resolve_paths(_repo: Path, _root: Path) -> tuple[Path, Path, Path]:
                self.assertTrue(active_execution, "owned path resolution must run inside PrepareExecution")
                active_execution[-1].check_work()
                checks.append("paths")
                return repo, frozen, source

            def verify_frozen(_root: Path, _source: Path, *, check_work: Callable[[], None] | None = None) -> None:
                self.assertIsNotNone(check_work)
                self.assertIs(check_work.__self__, active_execution[-1])
                check_work()
                checks.append("frozen")

            def verify_live(
                _repo: Path,
                _root: Path,
                _source: Path,
                _timeout: int,
                *,
                fresh_cohort_id: str,
                baseline_bin_dir: Path,
                execution: Execution | None = None,
            ) -> tuple[Path, dict[str, object]]:
                self.assertEqual(fresh_cohort_id, runner.PREPARE_LIVE_OWNED_COHORT)
                self.assertEqual(baseline_bin_dir, root / "sealed")
                self.assertIs(execution, active_execution[-1])
                execution.check_work()
                checks.append("live")
                return live, {"verified": True}

            argv = [
                "frozen_baseline.py", "--repo", str(repo), "--root", str(frozen),
                "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
                "--baseline-bin-dir", str(root / "sealed"), "--owned-lifecycle", "--owned-budget", "20",
            ]
            with (
                patch.object(sys, "argv", argv),
                patch.object(runner.PrepareExecution, "__enter__", enter_execution),
                patch.object(runner, "paths", side_effect=resolve_paths),
                patch.object(runner, "verify_source", side_effect=verify_frozen) as frozen_verify,
                patch.object(runner, "verify_live_source", side_effect=verify_live) as live_verify,
                redirect_stdout(io.StringIO()) as output,
            ):
                self.assertEqual(runner.main(), 0)
            frozen_verify.assert_called_once()
            live_verify.assert_called_once()
            self.assertEqual(checks, ["paths", "frozen", "live"])
            self.assertIn('"verified": true', output.getvalue())

    def test_verify_live_owned_path_failure_stops_before_verification(self) -> None:
        original_enter = runner.PrepareExecution.__enter__
        for failure in ("deadline", "cancel"):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                repo = root / "repo"
                frozen = root / "frozen"
                source = frozen / "source"
                repo.mkdir()
                source.mkdir(parents=True)
                (source / "Cargo.toml").write_text("[workspace]\n")
                active_execution: list[runner.PrepareExecution] = []

                def enter_execution(execution: runner.PrepareExecution) -> runner.PrepareExecution:
                    entered = original_enter(execution)
                    active_execution.append(execution)
                    return entered

                def resolving_paths(_repo: Path, _root: Path) -> tuple[Path, Path, Path]:
                    self.assertTrue(active_execution)
                    if failure == "deadline":
                        time.sleep(1.05)
                    else:
                        signal.raise_signal(signal.SIGTERM)
                    return repo, frozen, source

                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(frozen),
                    "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
                    "--baseline-bin-dir", str(root / "sealed"), "--owned-lifecycle", "--owned-budget", "1",
                ]
                with (
                    patch.object(sys, "argv", argv),
                    patch.object(runner.PrepareExecution, "__enter__", enter_execution),
                    patch.object(runner, "paths", side_effect=resolving_paths),
                    patch.object(runner, "verify_source", side_effect=AssertionError("verification must not run")),
                    patch.object(runner, "verify_live_source", side_effect=AssertionError("verification must not run")),
                    redirect_stderr(io.StringIO()) as errors,
                ):
                    self.assertEqual(runner.main(), 1)
                self.assertIn("owned prepare", errors.getvalue())

    def test_verify_live_owned_final_output_deadline_and_cancellation_fail_closed(self) -> None:
        original_enter = runner.PrepareExecution.__enter__
        original_print = print
        for failure in ("deadline", "cancel"):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                repo = root / "repo"
                frozen = root / "frozen"
                source = frozen / "source"
                repo.mkdir()
                source.mkdir(parents=True)
                (source / "Cargo.toml").write_text("[workspace]\n")
                active_execution: list[runner.PrepareExecution] = []

                def enter_execution(execution: runner.PrepareExecution) -> runner.PrepareExecution:
                    entered = original_enter(execution)
                    active_execution.append(execution)
                    return entered

                def emit(*args: object, **kwargs: object) -> None:
                    if args and isinstance(args[0], str) and '"verified": true' in args[0]:
                        self.assertTrue(kwargs.get("flush"), "owned success output must flush inside its deadline scope")
                        self.assertTrue(active_execution)
                        if failure == "deadline":
                            time.sleep(1.05)
                        else:
                            signal.raise_signal(signal.SIGTERM)
                    original_print(*args, **kwargs)

                argv = [
                    "frozen_baseline.py", "--repo", str(repo), "--root", str(frozen),
                    "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
                    "--baseline-bin-dir", str(root / "sealed"), "--owned-lifecycle", "--owned-budget", "1",
                ]
                with (
                    patch.object(sys, "argv", argv),
                    patch.object(runner.PrepareExecution, "__enter__", enter_execution),
                    patch.object(runner, "paths", return_value=(repo, frozen, source)),
                    patch.object(runner, "verify_source"),
                    patch.object(runner, "verify_live_source", return_value=(frozen / "live-source", {"verified": True})),
                    patch("builtins.print", side_effect=emit),
                    redirect_stdout(io.StringIO()),
                    redirect_stderr(io.StringIO()) as errors,
                ):
                    self.assertEqual(runner.main(), 1)
                if failure == "deadline":
                    self.assertIn("work deadline expired", errors.getvalue())
                else:
                    self.assertIn("cancelled by signal", errors.getvalue())

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
            (root / "receipts.actual").mkdir()
            (root / "receipts.semantic-actual").mkdir()
            with self.assertRaisesRegex(runner.RunnerError, "missing checkpoint receipts"):
                runner.read_receipts(receipts, ["showcase/pages/overview"], expected)

    def test_chunk_receipts_validate_global_expected_store_as_union(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            expected = root / "runtime-expected"
            expected.mkdir()
            pairs = [("holla::files::120x40::truecolor", "holla/files/120x40/truecolor"), ("tablepro::files::120x40::truecolor", "tablepro/files/120x40/truecolor")]
            semantic = self.semantic_frame()
            ansi = b"frame\x1b[?25h"
            for index, (test_id, case_id) in enumerate(pairs):
                phase = root / "baseline-capture"
                chunk = phase / f"chunk-{index:05}"
                receipts = chunk / "receipts"
                receipt_dir = receipts / runner.hashlib.sha256(case_id.encode()).hexdigest()
                receipt_dir.mkdir(parents=True)
                stem = f"checkpoint-00000000-{runner.checkpoint_identity(case_id, 0, case_id)}"
                for suffix, content in ((".ansi", ansi), (".frame.json", semantic)):
                    (expected / f"{stem}{suffix}").write_bytes(content)
                (receipts.with_name(receipts.name + ".actual")).mkdir()
                actual = receipts.with_name(receipts.name + ".actual")
                (actual / f"{stem}.ansi").write_bytes(ansi)
                semantic_actual = receipts.with_name(receipts.name + ".semantic-actual")
                semantic_actual.mkdir()
                (semantic_actual / f"{stem}.frame.json").write_bytes(semantic)
                record = {"case_id": case_id, "ordinal": 0, "logical_name": case_id, "sha256": runner.hashlib.sha256(ansi).hexdigest(), "semantic_frame_sha256": runner.hashlib.sha256(semantic).hexdigest(), "process_receipt_sha256": "f" * 64}
                (receipt_dir / f"{stem}.json").write_text(json.dumps(record))
                (chunk / "report.json").write_text(json.dumps({"selected_cases": [test_id]}))
                self.assertEqual(len(runner.read_receipts(receipts, [case_id], expected)), 1)
            combined = runner.phase_receipts(root, "baseline-capture", [test_id for test_id, _ in pairs], expected)
            self.assertEqual(len(combined), 2)
            (expected / "unregistered.ansi").write_bytes(ansi)
            with self.assertRaisesRegex(runner.RunnerError, "runtime expected frame inventory mismatch"):
                runner.phase_receipts(root, "baseline-capture", [test_id for test_id, _ in pairs], expected)

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

    def test_extension_manifest_rejects_missing_middle_checkpoint(self) -> None:
        manifest = self.extension_manifest()
        case = next(case for case in manifest["generated_cases"] if case["cols"] == 120 and case["rows"] == 40)
        case["checkpoints"] = ["initial", "cleared"]
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "extensions.json"
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(runner.RunnerError, "exact journey checkpoints"):
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
        case = name
        manifest = {"generated_case_ids": [name], "checkpoint_obligations": {name: ["before", "sorted"]}}
        receipts = {(case, 0, name + "/before"): "1" * 64, (case, 1, name + "/sorted"): "2" * 64}
        self.assertEqual(runner.scenario_receipt_id(name, manifest), name)
        self.assertEqual(runner.scenario_receipt_id("showcase::overview::120x40::truecolor", manifest), "showcase/overview/120x40/truecolor")
        self.assertEqual(runner.validate_extension_checkpoint_receipts([name], receipts, manifest), [])
        with self.assertRaisesRegex(runner.RunnerError, "checkpoint obligations differ"):
            runner.validate_extension_checkpoint_receipts([name], {(case, 0, name + "/sorted"): "1" * 64}, manifest)

    def test_registered_cli_green_does_not_claim_overall_acceptance(self) -> None:
        report = runner.cli_coverage_report_fields(True)
        self.assertEqual(report["scope"], "registered_cli_scenarios_and_ordinary_checks")
        self.assertTrue(report["registered_cli_complete"])
        self.assertFalse(report["overall_acceptance_complete"])
        self.assertFalse(report["acceptance_complete"])
        self.assertFalse(runner.cli_coverage_report_fields(False)["registered_cli_complete"])

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

    def test_executable_cohort_registry_is_pinned_and_diagnostic_only(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        document = runner.load_executable_cohorts(repo)
        cohort = document["cohorts"][0]
        self.assertEqual(cohort["status"], runner.DIAGNOSTIC_COHORT_STATUS)
        self.assertFalse(cohort["build"]["build_admitted"])
        self.assertFalse(cohort["build"]["reference_frames_approved"])
        reviews, _digest = runner.load_cohort_review_registry(repo, document, runner.EXECUTABLE_COHORTS_SHA256)
        review = reviews[cohort["id"]]
        self.assertTrue(review["fresh_baseline_capture_eligible"])
        self.assertTrue(review["baseline_replay_eligible"])
        self.assertFalse(review["candidate_comparison_eligible"])
        self.assertEqual(review["checkpoint_reference_approval"], "none")
        self.assertFalse(review["product_acceptance_eligible"])
        self.assertEqual(review["evidence"][0]["report_sha256"], runner.CAPTURE_REVIEW_DECISION_SHA256)
        decision, decision_digest, _index, index_digest, contract, contract_digest = runner._capture_review_documents(repo)
        self.assertEqual(decision_digest, review["decision_sha256"])
        self.assertEqual(index_digest, review["evidence_index_sha256"])
        self.assertEqual(contract_digest, runner.CAPTURE_REVIEW_CONTRACT_SHA256)
        self.assertEqual(decision["cohort_id"], cohort["id"])
        self.assertEqual(decision["cohort_registry_sha256"], runner.EXECUTABLE_COHORTS_SHA256)
        self.assertEqual(decision["source"], cohort["source"])
        self.assertEqual(decision["binary_sha256"], {app: cohort["binaries"][app]["sha256"] for app in runner.APPS})
        self.assertEqual(contract["status"], "proposed-implementation-contract-not-executed")
        self.assertEqual(runner.sha256(repo / runner.REFERENCE_DECISIONS), runner.REFERENCE_DECISIONS_SHA256)

    def test_fresh_capture_review_does_not_transfer_old_checkpoint_exception(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        reference_path = repo / runner.REFERENCE_DECISIONS
        payload = reference_path.read_bytes()
        self.assertEqual(runner.hashlib.sha256(payload).hexdigest(), runner.REFERENCE_DECISIONS_SHA256)
        legacy = json.loads(payload)
        self.assertEqual(legacy["decision_scope"], "one_exact_stale_approval_live_pair_reference")
        self.assertEqual(len(legacy["decisions"]), 1)
        stale = legacy["decisions"][0]
        self.assertEqual(stale["tuple"]["test_id"], runner.REFERENCE_CASE)
        self.assertEqual(stale["legacy_approval"]["verdict"], "known_stale_cells_differ")
        self.assertEqual(stale["legacy_approval"]["paired_exception"], "never edits or replaces this approval; its standalone legacy verdict stays red")
        reviews, _ = runner.load_cohort_review_registry(repo, runner.load_executable_cohorts(repo), runner.EXECUTABLE_COHORTS_SHA256)
        capture = reviews["frozen-20261021-serial-nonincr-r1-r2"]
        self.assertEqual(capture["checkpoint_reference_approval"], "none")
        self.assertFalse(capture["candidate_comparison_eligible"])
        self.assertFalse(capture["product_acceptance_eligible"])

    def test_capture_review_decision_evidence_and_contract_are_hash_pinned(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for relative in (runner.CAPTURE_REVIEW_DECISION, runner.CAPTURE_REVIEW_EVIDENCE_INDEX, runner.CAPTURE_REVIEW_CONTRACT, Path("docs/audits/2026-10-21-fresh-baseline-capture-evidence-recheck.json")):
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((repo / relative).read_bytes())
            for relative in (runner.CAPTURE_REVIEW_DECISION, runner.CAPTURE_REVIEW_EVIDENCE_INDEX, runner.CAPTURE_REVIEW_CONTRACT):
                original = (root / relative).read_bytes()
                (root / relative).write_bytes(original + b" ")
                with self.assertRaisesRegex(runner.RunnerError, "is unpinned"):
                    runner._capture_review_documents(root)
                (root / relative).write_bytes(original)

    def test_executable_cohort_schema_rejects_duplicate_unknown_and_bool_version(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        original = (repo / runner.EXECUTABLE_COHORTS).read_bytes()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            path = root / runner.EXECUTABLE_COHORTS
            path.parent.mkdir(parents=True)
            for payload, error in (
                (b'{"schema":"termrock.frozen-cli-build-cohorts","schema":"bad","schema_version":1,"cohorts":[]}', "duplicate executable cohort JSON field"),
                (json.dumps({**json.loads(original), "unknown": True}).encode(), "fields differ from schema"),
                (json.dumps({**json.loads(original), "schema_version": True}).encode(), "unsupported executable cohort registry schema"),
            ):
                path.write_bytes(payload)
                with patch.object(runner, "EXECUTABLE_COHORTS_SHA256", runner.hashlib.sha256(payload).hexdigest()):
                    with self.assertRaisesRegex(runner.RunnerError, error):
                        runner.load_executable_cohorts(root)

    def test_executable_cohort_boolean_claims_reject_integer_aliases(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        document = runner.load_executable_cohorts(repo)
        for field, replacement in (("incremental", 0), ("stage_hashes_match", 1), ("build_admitted", 0), ("reference_frames_approved", 0)):
            altered = json.loads(json.dumps(document))
            altered["cohorts"][0]["build"][field] = replacement
            with self.assertRaisesRegex(runner.RunnerError, "boolean type"):
                runner.validate_executable_cohorts(altered)

    def test_executable_cohort_rejects_mixed_hashes_and_invented_run_evidence(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        document = runner.load_executable_cohorts(repo)
        for mutate, error in (
            (lambda c: c["binaries"]["holla"].update({"sha256": "0" * 64}), "stage binary inventory is invalid"),
            (lambda c: c["runs"][0].update({"complete_target_archive": True}), "archive/retry limitations"),
            (lambda c: c["runs"].pop(), "exactly two complete run records"),
            (lambda c: c["source"].update({"tag": "moving-tag"}), "source/tool pins changed"),
        ):
            altered = json.loads(json.dumps(document))
            mutate(altered["cohorts"][0])
            with self.assertRaisesRegex(runner.RunnerError, error):
                runner.validate_executable_cohorts(altered)

    def test_diagnostic_binary_check_rejects_symlinks_mismatch_and_path_escape(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        cohort = runner.load_executable_cohorts(repo)["cohorts"][0]
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in runner.APPS:
                path = root / name
                path.write_bytes(b"not-the-sealed-reference-binary")
                path.chmod(0o755)
            with self.assertRaisesRegex(runner.RunnerError, "hash mismatch"):
                runner.verify_diagnostic_cohort_binaries(repo, cohort["id"], root)
            (root / "showcase").unlink()
            (root / "showcase").symlink_to(root / "tablepro")
            with self.assertRaisesRegex(runner.RunnerError, "private regular executable"):
                runner.verify_diagnostic_cohort_binaries(repo, cohort["id"], root)
            altered = json.loads(json.dumps(cohort))
            altered["binaries"]["showcase"]["path"] = "../showcase"
            with self.assertRaises(TypeError):
                runner.verify_diagnostic_cohort_binaries(altered, root)

    def test_diagnostic_binary_check_rejects_forged_hash_inventory(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        cohort = runner.load_executable_cohorts(repo)["cohorts"][0]
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            altered = json.loads(json.dumps(cohort))
            for app in runner.APPS:
                data = f"unrelated-{app}".encode()
                (root / app).write_bytes(data)
                altered["binaries"][app]["sha256"] = runner.hashlib.sha256(data).hexdigest()
            with self.assertRaises(TypeError):
                runner.verify_diagnostic_cohort_binaries(altered, root)

    def test_capture_eligibility_is_separate_pinned_review_and_never_approves_frames(self) -> None:
        repo = Path(runner.__file__).resolve().parent.parent
        document = runner.load_executable_cohorts(repo)
        cohort = json.loads(json.dumps(document["cohorts"][0]))
        cohort["id"] = "test-cohort-fixture"
        binary_bytes = {app: f"fixture-binary-{app}".encode() for app in runner.APPS}
        binary_hashes = {app: runner.hashlib.sha256(data).hexdigest() for app, data in binary_bytes.items()}
        cohort["binaries"] = {app: {"path": app, "sha256": binary_hashes[app]} for app in runner.APPS}
        for run in cohort["runs"]:
            run["stage1_hashes"] = binary_hashes.copy()
            run["final_hashes"] = binary_hashes.copy()
        fixture_registry = {"schema": runner.EXECUTABLE_COHORT_SCHEMA, "schema_version": runner.EXECUTABLE_COHORT_VERSION, "cohorts": [cohort]}
        registry_bytes = (json.dumps(fixture_registry, indent=2) + "\n").encode()
        registry_digest = runner.hashlib.sha256(registry_bytes).hexdigest()
        decision = json.loads((repo / runner.CAPTURE_REVIEW_DECISION).read_text())
        decision["cohort_id"] = cohort["id"]
        decision["cohort_registry_sha256"] = registry_digest
        decision["binary_sha256"] = binary_hashes.copy()
        decision["source"] = cohort["source"]
        decision_bytes = (json.dumps(decision, indent=2) + "\n").encode()
        decision_digest = runner.hashlib.sha256(decision_bytes).hexdigest()
        index = json.loads((repo / runner.CAPTURE_REVIEW_EVIDENCE_INDEX).read_text())
        index["cohort_id"] = cohort["id"]
        index["decision_sha256"] = decision_digest
        index["reviewed_integration_commit"] = decision["integration_commit_reviewed"]
        index["source"] = decision["source"]
        index["binary_sha256"] = binary_hashes.copy()
        index["artifacts"]["capture_eligibility_decision"]["sha256"] = decision_digest
        index["artifacts"]["capture_eligibility_decision"]["bytes"] = len(decision_bytes)
        index_bytes = (json.dumps(index, indent=2) + "\n").encode()
        index_digest = runner.hashlib.sha256(index_bytes).hexdigest()
        review = {
            "cohort_id": cohort["id"],
            "status": runner.CAPTURE_ELIGIBLE_COHORT_STATUS,
            "cohort_registry_sha256": registry_digest,
            "binary_sha256": binary_hashes,
            "source": cohort["source"],
            "evidence": [{"reviewer": decision["reviewer"], "scope": decision["scope"], "report_sha256": decision_digest}],
            "decision_sha256": decision_digest,
            "evidence_index_sha256": index_digest,
            "integration_commit_reviewed": decision["integration_commit_reviewed"],
            "fresh_baseline_capture_eligible": True,
            "baseline_replay_eligible": True,
            "candidate_comparison_eligible": False,
            "checkpoint_reference_approval": "none",
            "product_acceptance_eligible": False,
        }
        review_bytes = (json.dumps({"schema": runner.COHORT_REVIEWS_SCHEMA, "schema_version": runner.COHORT_REVIEWS_VERSION, "reviews": [review]}, indent=2) + "\n").encode()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / runner.EXECUTABLE_COHORTS).parent.mkdir(parents=True)
            (root / runner.EXECUTABLE_COHORTS).write_bytes(registry_bytes)
            (root / runner.COHORT_REVIEWS).write_bytes(review_bytes)
            (root / runner.CAPTURE_REVIEW_EVIDENCE_INDEX).parent.mkdir(parents=True, exist_ok=True)
            (root / runner.CAPTURE_REVIEW_EVIDENCE_INDEX).write_bytes(index_bytes)
            for artifact in index["artifacts"].values():
                artifact_path = Path(artifact["repository_path"])
                destination = root / artifact_path
                destination.parent.mkdir(parents=True, exist_ok=True)
                if artifact_path == runner.CAPTURE_REVIEW_DECISION:
                    destination.write_bytes(decision_bytes)
                else:
                    destination.write_bytes((repo / artifact_path).read_bytes())
            bins = root / "sealed-bin"
            bins.mkdir()
            for app, data in binary_bytes.items():
                path = bins / app
                path.write_bytes(data)
                path.chmod(0o755)
            with patch.object(runner, "EXECUTABLE_COHORTS_SHA256", registry_digest), patch.object(runner, "COHORT_REVIEWS_SHA256", runner.hashlib.sha256(review_bytes).hexdigest()), patch.object(runner, "CAPTURE_REVIEW_DECISION_SHA256", decision_digest), patch.object(runner, "CAPTURE_REVIEW_EVIDENCE_INDEX_SHA256", index_digest):
                admitted = runner.require_capture_eligible_cohort(root, cohort["id"], bins)
                self.assertEqual(admitted["status"], runner.CAPTURE_ELIGIBLE_COHORT_STATUS)
                self.assertTrue(admitted["fresh_baseline_capture_eligible"])
                self.assertTrue(admitted["baseline_replay_eligible"])
                self.assertFalse(admitted["candidate_comparison_eligible"])
                self.assertEqual(admitted["checkpoint_reference_approval"], "none")
                self.assertFalse(admitted["product_acceptance_eligible"])
                self.assertFalse(admitted["acceptance_complete"])
                source = root / "source"
                source.mkdir()
                (source / "Cargo.lock").write_text("fixture lock")
                (source / "rust-toolchain.toml").write_text("fixture toolchain")
                candidate_repo = root / "candidate-repo"
                candidate_repo.mkdir()
                candidate_bins = root / "candidate-bin"
                candidate_bins.mkdir()
                candidate_hashes = {}
                candidate_files = {}
                for app in runner.APPS:
                    path = candidate_bins / app
                    path.write_bytes(f"candidate-{app}".encode())
                    path.chmod(0o755)
                    candidate_hashes[app], candidate_files[app] = runner.stable_private_binary(path, f"candidate binary {app}")
                candidate_record = {
                    "repo": str(candidate_repo), "revision": "candidate-revision",
                    "dirty_fingerprint": "b" * 64, "dirty_files": [],
                    "target_dir": str(root / "candidate-target"), "bin_dir": str(candidate_bins),
                    "binaries": candidate_hashes, "binary_files": candidate_files,
                }
                (root / "candidate-build.json").write_text(json.dumps(candidate_record))
                real_sha256 = runner.sha256
                def fixture_sha256(path: Path) -> str:
                    if path == source / "Cargo.lock":
                        return cohort["source"]["cargo_lock_sha256"]
                    if path == source / "rust-toolchain.toml":
                        return cohort["source"]["rust_toolchain_sha256"]
                    return real_sha256(path)
                def read_records():
                    with patch.object(runner, "verify_source", return_value="frozen-source-sha"), patch.object(runner, "git_state", return_value=("candidate-revision", "b" * 64, [])), patch.object(runner, "run", return_value="candidate-tree"), patch.object(runner, "sha256", side_effect=fixture_sha256):
                        return runner.baseline_and_candidate_records(
                            candidate_repo, root, bins, candidate_bins, 30, baseline_cohort_id=cohort["id"], cohort_repo=root
                        )
                baseline_record, checked_candidate = read_records()
                self.assertEqual(baseline_record["executable_cohort"]["cohort_id"], cohort["id"])
                self.assertEqual(baseline_record["binaries"], binary_hashes)
                self.assertNotIn("build_provenance_sha256", baseline_record)
                self.assertFalse(baseline_record["candidate_comparison_eligible"])
                self.assertFalse(baseline_record["acceptance_complete"])
                app = runner.APPS[0]
                candidate_path = candidate_bins / app
                baseline_path = bins / app
                with self.assertRaisesRegex(runner.RunnerError, "private regular executable"):
                    candidate_path.unlink()
                    candidate_path.symlink_to(baseline_path)
                    read_records()
                candidate_path.unlink()
                candidate_path.write_bytes(f"candidate-{app}".encode())
                candidate_path.chmod(0o755)
                with self.assertRaisesRegex(runner.RunnerError, "private regular executable"):
                    candidate_path.unlink()
                    runner.os.link(baseline_path, candidate_path)
                    read_records()
                candidate_path.unlink()
                candidate_path.write_bytes(baseline_path.read_bytes())
                candidate_path.chmod(0o755)
                with self.assertRaisesRegex(runner.RunnerError, "identity changed"):
                    read_records()
                for alteration in (
                    {**review, "candidate_comparison_eligible": True},
                    {**review, "fresh_baseline_capture_eligible": False},
                    {**review, "baseline_replay_eligible": 0},
                    {**review, "product_acceptance_eligible": True},
                    {**review, "checkpoint_reference_approval": "approved"},
                    {**review, "cohort_registry_sha256": "0" * 64},
                    {**review, "binary_sha256": {**binary_hashes, "holla": "0" * 64}},
                    {**review, "source": {**cohort["source"], "commit": "different-source"}},
                    {**review, "decision_sha256": "0" * 64},
                    {**review, "evidence_index_sha256": "0" * 64},
                    {**review, "integration_commit_reviewed": "different-commit"},
                    {**review, "evidence": []},
                ):
                    altered = (json.dumps({"schema": runner.COHORT_REVIEWS_SCHEMA, "schema_version": runner.COHORT_REVIEWS_VERSION, "reviews": [alteration]}, indent=2) + "\n").encode()
                    (root / runner.COHORT_REVIEWS).write_bytes(altered)
                    with patch.object(runner, "COHORT_REVIEWS_SHA256", runner.hashlib.sha256(altered).hexdigest()):
                        with self.assertRaises(runner.RunnerError):
                            runner.require_capture_eligible_cohort(root, cohort["id"], bins)

    def test_registry_hash_and_parse_use_one_pinned_file_read(self) -> None:
        actual_path = Path(runner.__file__).resolve().parent.parent / runner.EXECUTABLE_COHORTS
        original = actual_path.read_bytes()
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            path = root / runner.EXECUTABLE_COHORTS
            path.parent.mkdir(parents=True)
            path.write_bytes(original)

            def replace_after_hash(target: Path) -> str:
                digest = runner.hashlib.sha256(target.read_bytes()).hexdigest()
                altered = json.loads(target.read_text())
                altered["cohorts"][0]["status"] = "forged-status"
                target.write_text(json.dumps(altered))
                return digest

            with patch.object(runner, "EXECUTABLE_COHORTS_SHA256", runner.hashlib.sha256(original).hexdigest()), patch.object(runner, "sha256", side_effect=replace_after_hash) as separate_hash:
                loaded = runner.load_executable_cohorts(root)
                separate_hash.assert_not_called()
            self.assertEqual(loaded["cohorts"][0]["status"], runner.DIAGNOSTIC_COHORT_STATUS)
            self.assertEqual(path.read_bytes(), original)

    def test_owned_prepare_cli_is_opt_in_and_budget_is_bounded(self) -> None:
        legacy = runner.parser().parse_args(["prepare"])
        owned = runner.parser().parse_args(["prepare", "--owned-lifecycle", "--owned-budget", "37"])
        verify_legacy = runner.parser().parse_args([
            "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
            "--baseline-bin-dir", "/sealed/bin",
        ])
        verify_owned = runner.parser().parse_args([
            "verify-live", "--fresh-cohort", runner.PREPARE_LIVE_OWNED_COHORT,
            "--baseline-bin-dir", "/sealed/bin", "--owned-lifecycle", "--owned-budget", "37",
        ])
        self.assertFalse(legacy.owned_lifecycle)
        self.assertEqual(legacy.timeout, runner.MAX_TIMEOUT)
        self.assertEqual(owned.owned_budget, 37)
        self.assertFalse(verify_legacy.owned_lifecycle)
        self.assertEqual(verify_legacy.owned_budget, runner.PREPARE_OWNED_MAX_BUDGET)
        self.assertTrue(verify_owned.owned_lifecycle)
        self.assertEqual(verify_owned.owned_budget, 37)
        with self.assertRaisesRegex(runner.RunnerError, "owned prepare budget"):
            runner.PrepareExecution(runner.PREPARE_OWNED_MAX_BUDGET + 1)

    def test_owned_prepare_synthetic_git_archive_and_provenance_match_legacy(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            repo.mkdir()
            (repo / "Cargo.lock").write_text('source = "fixture-revision"\n')
            (repo / "src").mkdir()
            (repo / "src" / "main.rs").write_text("fn main() {}\n")
            (repo / "snapshots").mkdir()
            (repo / "snapshots" / "frame.ansi").write_bytes(b"synthetic-frame\0")

            def git(*args: str) -> str:
                return subprocess.run(
                    ["git", "-C", str(repo), *args],
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=10,
                ).stdout.strip()

            git("init")
            git("config", "user.email", "fixture@example.invalid")
            git("config", "user.name", "fixture")
            git("add", "Cargo.lock", "src/main.rs", "snapshots/frame.ansi")
            git("commit", "-m", "fixture")
            git("tag", "-a", "fixture", "-m", "fixture")
            commit = git("rev-parse", "fixture^{commit}")
            tag_object = git("rev-parse", "fixture^{tag}")
            tree = git("rev-parse", f"{commit}^{{tree}}")
            with patch.object(runner, "BASELINE_TAG", "fixture"), patch.object(runner, "BASELINE_TAG_OBJECT", tag_object), patch.object(runner, "BASELINE_COMMIT", commit), patch.object(runner, "BASELINE_TREE", tree), patch.object(runner, "TUISCOTTI_REV", "fixture-revision"):
                legacy_root = root / "legacy"
                owned_root = root / "owned"
                with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                    runner.prepare(repo, legacy_root, 10)
                    with runner.PrepareExecution(20) as execution:
                        runner.prepare(repo, owned_root, 20, execution=execution)
                self.assertEqual((legacy_root / "provenance.json").read_bytes(), (owned_root / "provenance.json").read_bytes())
                self.assertEqual(runner.digest_source(legacy_root / "source"), runner.digest_source(owned_root / "source"))
                self.assertEqual(runner.digest_tree(legacy_root / "source" / "snapshots"), runner.digest_tree(owned_root / "source" / "snapshots"))

    def test_owned_prepare_command_preserves_captured_streams_and_cleans_group(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            with runner.PrepareExecution(10) as execution:
                stdout, stderr = execution.run(
                    [sys.executable, "-c", "import sys; print('captured-out'); print('captured-err', file=sys.stderr)"],
                    cwd=root,
                    quiet=True,
                )
            self.assertEqual(stdout, "captured-out\n")
            self.assertEqual(stderr, "captured-err\n")

    def test_owned_prepare_archive_stream_is_byte_exact(self) -> None:
        payload = b"git-archive-equivalent\0\xff\n"
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive = tempfile.TemporaryFile()
            try:
                with runner.PrepareExecution(10) as execution:
                    stdout, _stderr = execution.run(
                        [sys.executable, "-c", "import os,sys; os.write(1, bytes.fromhex(sys.argv[1]))", payload.hex()],
                        cwd=root,
                        stdout_file=archive,
                        capture_stdout=False,
                        quiet=True,
                        show_command=False,
                    )
                self.assertEqual(stdout, "")
                archive.seek(0)
                self.assertEqual(archive.read(), payload)
            finally:
                archive.close()

    def test_owned_prepare_checked_digest_and_extraction_preserve_results(self) -> None:
        checks: list[int] = []

        def check_work() -> None:
            checks.append(1)

        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive_path = root / "input.tar"
            original = root / "original"
            checked = root / "checked"
            original.mkdir()
            checked.mkdir()
            with tarfile.open(archive_path, "w") as archive:
                payload = root / "payload.txt"
                payload.write_text("pinned source bytes\n")
                archive.add(payload, arcname="nested/payload.txt")
            for destination in (original, checked):
                with tarfile.open(archive_path, "r:") as archive:
                    runner.safe_extract(archive, destination, check_work=check_work if destination == checked else None)
            self.assertGreater(len(checks), 0)
            self.assertEqual(runner.digest_source(original), runner.digest_source(checked))
            counted: list[int] = []
            checked_digest = runner.digest_source(checked, check_work=lambda: counted.append(1))
            self.assertEqual(checked_digest, runner.digest_source(checked))
            self.assertGreater(len(counted), 0)

    def test_owned_prepare_extraction_preserves_delayed_directory_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive_path = root / "directory-metadata.tar"
            old_mtime = 1_234_567_000
            with tarfile.open(archive_path, "w") as archive:
                directory = tarfile.TarInfo("restricted")
                directory.type = tarfile.DIRTYPE
                directory.mode = 0o000
                directory.mtime = old_mtime
                archive.addfile(directory)
                payload = b"nested synthetic content\n"
                file_member = tarfile.TarInfo("restricted/payload.txt")
                file_member.size = len(payload)
                file_member.mode = 0o400
                file_member.mtime = old_mtime + 1
                archive.addfile(file_member, io.BytesIO(payload))

            expected = root / "tarfile-reference"
            expected.mkdir()
            with tarfile.open(archive_path, "r:") as archive:
                archive.extractall(expected)
            expected_dir = expected / "restricted"
            expected_metadata = (stat.S_IMODE(expected_dir.stat().st_mode), int(expected_dir.stat().st_mtime))

            destinations = (root / "legacy", root / "owned")
            check_calls: list[int] = []
            for destination in destinations:
                destination.mkdir()
                with tarfile.open(archive_path, "r:") as archive:
                    runner.safe_extract(
                        archive,
                        destination,
                        check_work=(lambda: check_calls.append(1)) if destination == destinations[1] else None,
                    )

            for destination in destinations:
                extracted_dir = destination / "restricted"
                self.assertEqual((extracted_dir / "payload.txt").read_bytes(), payload)
                self.assertEqual(
                    (stat.S_IMODE(extracted_dir.stat().st_mode), int(extracted_dir.stat().st_mtime)),
                    expected_metadata,
                )
                self.assertEqual(int(extracted_dir.stat().st_mtime), old_mtime)
            self.assertGreater(len(check_calls), 0)

    def test_owned_group_observer_uses_absolute_system_ps_and_rejects_bad_results(self) -> None:
        valid = subprocess.CompletedProcess(
            [runner.OWNED_PROCESS_LIST_COMMAND],
            0,
            stdout="123 123 S\n",
            stderr="",
        )
        with patch.object(runner.subprocess, "run", return_value=valid) as run_process:
            self.assertEqual(runner._owned_group_members(123, 123, 1), {123})
        self.assertEqual(run_process.call_args.args[0][0], runner.OWNED_PROCESS_LIST_COMMAND)
        self.assertTrue(Path(runner.OWNED_PROCESS_LIST_COMMAND).is_absolute())

        malformed = subprocess.CompletedProcess(
            [runner.OWNED_PROCESS_LIST_COMMAND], 0, stdout="not a process row\n", stderr=""
        )
        with patch.object(runner.subprocess, "run", return_value=malformed):
            with self.assertRaisesRegex(runner.RunnerError, "membership output was malformed"):
                runner._owned_group_members(123, 123, 1)

        timeout = subprocess.TimeoutExpired([runner.OWNED_PROCESS_LIST_COMMAND], timeout=0.1)
        with patch.object(runner.subprocess, "run", side_effect=timeout):
            with self.assertRaisesRegex(runner.RunnerError, "could not verify anchored process-group membership"):
                runner._owned_group_members(123, 123, 1)

    def test_owned_prepare_eof_with_observation_failure_kills_anchored_group(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            stdout_path = root / "stdout"
            stderr_path = root / "stderr"
            status_path = root / "status.json"
            ready_path = root / "anchor-ready.json"
            spec_path = root / "spec.json"
            child_pid_path = root / "child.pid"
            child_ready_path = root / "child.ready"
            stopped_path = root / "child.stopped"
            helper = (
                "import pathlib,signal,sys,time; "
                "signal.signal(signal.SIGTERM, lambda *_: (pathlib.Path(sys.argv[2]).write_text('stopped'), sys.exit(0))); "
                "pathlib.Path(sys.argv[1]).write_text('ready'); "
                "time.sleep(30)"
            )
            launch = (
                "import pathlib,subprocess,sys; "
                "child=subprocess.Popen([sys.executable,'-c',sys.argv[2],sys.argv[3],sys.argv[4]]); "
                "ready=pathlib.Path(sys.argv[3]); "
                "deadline=__import__('time').monotonic()+5; "
                "exec('while not ready.exists() and __import__(\"time\").monotonic()<deadline: __import__(\"time\").sleep(0.005)'); "
                "assert ready.exists(), 'child signal handler did not become ready'; "
                "pathlib.Path(sys.argv[1]).write_text(str(child.pid))"
            )
            spec_path.write_text(json.dumps({
                "argv": [sys.executable, "-c", launch, str(child_pid_path), helper, str(child_ready_path), str(stopped_path)],
                "cwd": str(root),
                "stdout_fd": 1,
                "stderr_fd": 2,
                "status_path": str(status_path),
                "ready_path": str(ready_path),
            }))
            control_read, control_write = os.pipe()
            os.set_inheritable(control_read, True)
            import_path = Path(runner.__file__).resolve().parent
            broker_script = (
                "import pathlib,sys; sys.path.insert(0,sys.argv[3]); import frozen_baseline as r; "
                "r.PREPARE_OWNED_TERM_GRACE=0.2; "
                "r._owned_group_members=lambda *_: (_ for _ in ()).throw(r.RunnerError('injected observer failure')); "
                "raise SystemExit(r._owned_broker_main(pathlib.Path(sys.argv[1]),int(sys.argv[2])))"
            )
            stdout_stream = stdout_path.open("wb")
            stderr_stream = stderr_path.open("wb")
            broker = subprocess.Popen(
                [sys.executable, "-c", broker_script, str(spec_path), str(control_read), str(import_path)],
                stdin=subprocess.DEVNULL,
                stdout=stdout_stream,
                stderr=stderr_stream,
                close_fds=True,
                pass_fds=(control_read,),
                start_new_session=True,
            )
            os.close(control_read)
            try:
                deadline = time.monotonic() + 5
                while not status_path.exists() and broker.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(status_path.exists(), "broker did not finish its synthetic owned command")
                self.assertTrue(child_pid_path.is_file())
                self.assertTrue(child_ready_path.is_file(), "helper signal handler was not ready before EOF")
                os.close(control_write)
                control_write = -1
                self.assertEqual(broker.wait(timeout=5), -signal.SIGKILL)
                self.assertTrue(stopped_path.exists(), "owned child did not receive TERM before fail-closed KILL")
                child_pid = int(child_pid_path.read_text())
                with self.assertRaises(ProcessLookupError):
                    os.kill(child_pid, 0)
            finally:
                if control_write >= 0:
                    os.close(control_write)
                if broker.poll() is None:
                    os.killpg(broker.pid, signal.SIGKILL)
                    broker.wait(timeout=5)
                stdout_stream.close()
                stderr_stream.close()

    def test_owned_prepare_removes_cooperative_same_group_helper_before_release(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            helper = "import signal,sys,time; signal.signal(signal.SIGTERM, lambda *_: sys.exit(0)); time.sleep(30)"
            command = [
                sys.executable,
                "-c",
                "import subprocess,sys; subprocess.Popen([sys.executable, '-c', sys.argv[1]])",
                helper,
            ]
            with runner.PrepareExecution(10) as execution:
                stdout, _stderr = execution.run(command, cwd=root, quiet=True)
            self.assertEqual(stdout, "")

    def test_owned_prepare_cancellation_is_recorded_and_child_gets_default_term(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            marker = root / "started"
            original_handler = signal.getsignal(signal.SIGTERM)
            calls: list[int] = []
            original_killpg = runner.os.killpg
            unrelated = subprocess.Popen(
                [sys.executable, "-c", "import time; time.sleep(30)"],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )

            def killpg(pgid: int, signum: int) -> None:
                calls.append(signum)
                original_killpg(pgid, signum)

            def signal_runner() -> None:
                deadline = time.monotonic() + 5
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                if marker.exists():
                    os.kill(os.getpid(), signal.SIGTERM)

            sender = threading.Thread(target=signal_runner)
            sender.start()
            try:
                with patch.object(runner.os, "killpg", side_effect=killpg):
                    with self.assertRaisesRegex(runner.RunnerError, "cancelled by signal"):
                        with runner.PrepareExecution(10) as execution:
                            execution.run(
                                [sys.executable, "-c", "import pathlib,sys,time; pathlib.Path(sys.argv[1]).touch(); time.sleep(30)", str(marker)],
                                cwd=root,
                                quiet=True,
                            )
                sender.join(timeout=5)
                self.assertFalse(sender.is_alive())
                unrelated.poll()
                self.assertIsNone(unrelated.returncode, "owned cleanup signalled an unrelated session")
                self.assertIn(signal.SIGTERM, calls)
                self.assertNotIn(signal.SIGKILL, calls)
                self.assertIs(signal.getsignal(signal.SIGTERM), original_handler)
            finally:
                sender.join(timeout=5)
                if unrelated.poll() is None:
                    unrelated.terminate()
                unrelated.wait(timeout=5)

    def test_owned_prepare_cancellation_during_broker_registration_is_deferred(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            marker = root / "command-started"
            original_handler = signal.getsignal(signal.SIGTERM)
            original_popen = runner.subprocess.Popen
            calls: list[int] = []
            original_killpg = runner.os.killpg

            def popen(*args: object, **kwargs: object) -> subprocess.Popen[bytes]:
                child = original_popen(*args, **kwargs)
                argv = args[0] if args else kwargs.get("args")
                if isinstance(argv, list) and "--_owned-prepare-broker" in argv:
                    os.kill(os.getpid(), signal.SIGTERM)
                return child

            def killpg(pgid: int, signum: int) -> None:
                calls.append(signum)
                original_killpg(pgid, signum)

            with patch.object(runner.subprocess, "Popen", side_effect=popen), patch.object(runner.os, "killpg", side_effect=killpg):
                with self.assertRaisesRegex(runner.RunnerError, "cancelled by signal"):
                    with runner.PrepareExecution(10) as execution:
                        execution.run(
                            [sys.executable, "-c", "import pathlib,sys,time; pathlib.Path(sys.argv[1]).touch(); time.sleep(30)", str(marker)],
                            cwd=root,
                            quiet=True,
                        )
            self.assertNotIn(signal.SIGTERM, calls)
            self.assertNotIn(signal.SIGKILL, calls)
            self.assertFalse(marker.exists(), "cancellation before anchor readiness must prevent command launch")
            self.assertIs(signal.getsignal(signal.SIGTERM), original_handler)

    def test_owned_prepare_work_deadline_uses_reserved_cleanup(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            calls: list[int] = []
            original_killpg = runner.os.killpg

            def killpg(pgid: int, signum: int) -> None:
                calls.append(signum)
                original_killpg(pgid, signum)

            with patch.object(runner.os, "killpg", side_effect=killpg):
                with self.assertRaisesRegex(runner.RunnerError, "work deadline expired"):
                    with runner.PrepareExecution(3) as execution:
                        execution.run([sys.executable, "-c", "import time; time.sleep(30)"], cwd=root, quiet=True)
            self.assertIn(signal.SIGTERM, calls)
            self.assertNotIn(signal.SIGKILL, calls)

    def test_owned_prepare_term_resistant_command_fails_cleanup_without_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            marker = root / "term-ignored"
            calls: list[int] = []
            original_killpg = runner.os.killpg

            def killpg(pgid: int, signum: int) -> None:
                calls.append(signum)
                original_killpg(pgid, signum)

            def cancel_after_handler_installed() -> None:
                deadline = time.monotonic() + 5
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                if marker.exists():
                    os.kill(os.getpid(), signal.SIGTERM)

            sender = threading.Thread(target=cancel_after_handler_installed)
            sender.start()
            with patch.object(runner.os, "killpg", side_effect=killpg):
                with self.assertRaisesRegex(runner.RunnerError, "cleanup unverified after SIGKILL"):
                    with runner.PrepareExecution(10) as execution:
                        execution.run(
                            [sys.executable, "-c", "import pathlib,signal,sys,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); pathlib.Path(sys.argv[1]).touch(); time.sleep(30)", str(marker)],
                            cwd=root,
                            quiet=True,
                        )
            sender.join(timeout=5)
            self.assertFalse(sender.is_alive())
            self.assertIn(signal.SIGTERM, calls)
            self.assertIn(signal.SIGKILL, calls)
            self.assertEqual(calls[-1], signal.SIGKILL)

    def test_owned_prepare_spawn_failure_is_recorded_and_reaped(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            with runner.PrepareExecution(10) as execution:
                with self.assertRaisesRegex(runner.RunnerError, "could not start"):
                    execution.run([str(root / "missing-command")], cwd=root, quiet=True)
                self.assertFalse(execution._children)

    def test_owned_prepare_parent_pipe_eof_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            stdout_path = root / "stdout"
            stderr_path = root / "stderr"
            status_path = root / "status.json"
            ready_path = root / "anchor-ready.json"
            spec_path = root / "spec.json"
            with stdout_path.open("w+b") as stdout, stderr_path.open("w+b") as stderr:
                helper = "import signal,sys,time; signal.signal(signal.SIGTERM, lambda *_: sys.exit(0)); time.sleep(30)"
                spec_path.write_text(json.dumps({
                    "argv": [sys.executable, "-c", "import subprocess,sys; subprocess.Popen([sys.executable, '-c', sys.argv[1]])", helper],
                    "cwd": str(root),
                    "stdout_fd": stdout.fileno(),
                    "stderr_fd": stderr.fileno(),
                    "status_path": str(status_path),
                    "ready_path": str(ready_path),
                }))
                control_read, control_write = os.pipe()
                os.set_inheritable(control_read, True)
                broker = subprocess.Popen(
                    [sys.executable, str(Path(runner.__file__).resolve()), "--_owned-prepare-broker", str(spec_path), str(control_read)],
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    close_fds=True,
                    pass_fds=(control_read, stdout.fileno(), stderr.fileno()),
                    start_new_session=True,
                )
                os.close(control_read)
                try:
                    deadline = time.monotonic() + 5
                    while not status_path.exists() and broker.poll() is None and time.monotonic() < deadline:
                        time.sleep(0.01)
                    self.assertTrue(status_path.exists(), "broker did not publish its bounded command status")
                    os.close(control_write)
                    control_write = -1
                    self.assertEqual(broker.wait(timeout=5), 2)
                finally:
                    if control_write >= 0:
                        os.close(control_write)
                    if broker.poll() is None:
                        os.killpg(broker.pid, signal.SIGKILL)
                        broker.wait(timeout=5)

    def test_owned_broker_observer_failure_grace_boundary_still_sends_kill(self) -> None:
        signals: list[tuple[int, int]] = []

        def record_killpg(pgid: int, signum: int) -> None:
            signals.append((pgid, signum))

        clock_values = iter([0.0, 0.0, 0.0, 1.99, 2.01])
        with patch.object(runner.os, "getpid", return_value=1234), \
             patch.object(runner.os, "getpgrp", return_value=1234), \
             patch.object(runner.os, "killpg", side_effect=record_killpg), \
             patch.object(runner, "_owned_group_members", side_effect=runner.RunnerError("observer failure")), \
             patch.object(runner.time, "monotonic", side_effect=lambda: next(clock_values)), \
             patch.object(runner.time, "sleep") as sleep:
            runner._owned_broker_orphan_cleanup(None)

        self.assertEqual(signals, [(1234, signal.SIGTERM), (1234, signal.SIGKILL)])
        self.assertEqual(len(sleep.call_args_list), 1)
        self.assertAlmostEqual(sleep.call_args.args[0], 0.01)

    def _make_owned_live_fixture(self, root: Path) -> tuple[Path, Path, Path, Path, Path]:
        repo = root / "repo"
        frozen_root = root / "frozen"
        source = frozen_root / "source"
        (source / "tests" / "visual_baseline" / "scenarios").mkdir(parents=True)
        (source / "snapshots").mkdir()
        (source / "Cargo.toml").write_text("[workspace]\n")
        (source / "snapshots" / "frame.ansi").write_bytes(b"immutable reference input\n")
        for relative in (
            "tests/visual_baseline/main.rs",
            "tests/visual_baseline/support.rs",
            "tests/visual_baseline/scenarios/mod.rs",
            "tests/visual_baseline/scenarios/interact.rs",
        ):
            path = source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("frozen\n")
        repo.mkdir()
        (repo / "tools" / "frozen-cli-frame-support" / "src").mkdir(parents=True)
        (repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs").write_text("helper\n")
        (repo / runner.EXTENSION_MANIFEST).parent.mkdir(parents=True, exist_ok=True)
        (repo / runner.EXTENSION_MANIFEST).write_text("{}\n")
        (repo / runner.EXTENSION_SOURCE).write_text("extension\n")
        patch_lines = []
        for relative in (
            "tests/visual_baseline/main.rs",
            "tests/visual_baseline/support.rs",
            "tests/visual_baseline/scenarios/mod.rs",
            "tests/visual_baseline/scenarios/interact.rs",
        ):
            patch_lines.extend((
                f"--- a/{relative}\n",
                f"+++ b/{relative}\n",
                "@@ -1 +1 @@\n",
                "-frozen\n",
                "+live\n",
            ))
        (repo / "tools").mkdir(exist_ok=True)
        patch_path = repo / "tools" / "frozen_cli_pair.patch"
        patch_path.write_text("".join(patch_lines))
        return repo, frozen_root, source, patch_path, repo / runner.EXTENSION_MANIFEST

    def _owned_live_patches(self, repo: Path, patch_path: Path, manifest_path: Path):
        extension_source = repo / runner.EXTENSION_SOURCE
        helper = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
        return {
            "extension_manifest": patch.object(runner, "EXTENSION_MANIFEST", manifest_path.relative_to(repo)),
            "extension_source": patch.object(runner, "EXTENSION_SOURCE", extension_source.relative_to(repo)),
            "patch_hash": patch.object(runner, "EXTENSION_PATCH_SHA256", runner.sha256(patch_path)),
            "manifest_hash": patch.object(runner, "EXTENSION_MANIFEST_SHA256", runner.sha256(manifest_path)),
            "source_hash": patch.object(runner, "EXTENSION_SOURCE_SHA256", runner.sha256(extension_source)),
            "load_manifest": patch.object(runner, "load_extension_manifest", return_value={"generated_case_ids": []}),
            "cohort": patch.object(runner, "_fresh_cohort_source_record", return_value={"cohort_id": "fixture", "candidate_comparison_eligible": False, "checkpoint_reference_approval": "none", "product_acceptance_eligible": False}),
        }

    def test_prepare_live_owned_uses_one_execution_and_writes_manifest_only_after_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo, frozen_root, source, patch_path, manifest_path = self._make_owned_live_fixture(root)
            extensions = self._owned_live_patches(repo, patch_path, manifest_path)
            execution_seen: list[runner.PrepareExecution] = []

            def observe(_source: Path, _env: dict[str, str], *, execution: runner.PrepareExecution | None = None) -> dict[str, object]:
                self.assertIsNotNone(execution)
                execution_seen.append(execution)
                return {"synthetic": True}

            def inventory(_source: Path, _timeout: int, env: dict[str, str], _manifest: dict[str, object], *, execution: runner.PrepareExecution | None = None) -> dict[str, bool]:
                self.assertIsNotNone(execution)
                execution_seen.append(execution)
                return {"scenario::listed": True, "ordinary::listed": False}

            contexts = [
                *extensions.values(),
                patch.object(runner, "fresh_clean_env", return_value={"PATH": os.environ["PATH"], "VISUAL_TOOL": "must-be-removed"}),
                patch.object(runner, "fresh_tool_observation", side_effect=observe),
                patch.object(runner, "nextest_inventory", side_effect=inventory),
                patch.object(runner, "validate_extension_inventory", side_effect=lambda rows, _manifest: (rows, [])),
                patch.object(runner, "EXPECTED_SCENARIOS", 1),
                patch.object(runner, "EXPECTED_ORDINARY_TESTS", 1),
            ]
            with ExitStack() as stack:
                for context in contexts:
                    stack.enter_context(context)
                with runner.PrepareExecution(30) as execution, redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                    record = runner.prepare_live_source(repo, frozen_root, source, 20, fresh_cohort_id=runner.PREPARE_LIVE_OWNED_COHORT, baseline_bin_dir=root / "bins", execution=execution)
            self.assertEqual(execution_seen, [execution, execution])
            self.assertEqual(record["changed_files"], sorted({
                "tests/visual_baseline/main.rs",
                "tests/visual_baseline/support.rs",
                "tests/visual_baseline/scenarios/mod.rs",
                "tests/visual_baseline/scenarios/interact.rs",
                "tests/visual_baseline/frozen_cli_pair.rs",
                "tests/visual_baseline/frozen_cli_extensions.rs",
                "tests/visual_baseline/frozen_cli_extensions.json",
            }))
            self.assertEqual(record["registered_scenarios"], 1)
            self.assertEqual(record["ordinary_checks"], 1)
            self.assertTrue((frozen_root / "live-source.json").is_file())
            self.assertEqual((source / "snapshots" / "frame.ansi").stat().st_mode & 0o777, 0o644)
            self.assertEqual((frozen_root / "live-source" / "snapshots" / "frame.ansi").stat().st_mode & 0o222, 0)

    def test_prepare_live_owned_inventory_failure_preserves_partial_and_no_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo, frozen_root, source, patch_path, manifest_path = self._make_owned_live_fixture(root)
            extensions = self._owned_live_patches(repo, patch_path, manifest_path)
            contexts = [
                *extensions.values(),
                patch.object(runner, "fresh_clean_env", return_value={"PATH": os.environ["PATH"]}),
                patch.object(runner, "fresh_tool_observation", return_value={"synthetic": True}),
                patch.object(runner, "nextest_inventory", side_effect=runner.RunnerError("synthetic inventory failure")),
                patch.object(runner, "EXPECTED_SCENARIOS", 1),
                patch.object(runner, "EXPECTED_ORDINARY_TESTS", 1),
            ]
            with ExitStack() as stack:
                for context in contexts:
                    stack.enter_context(context)
                with runner.PrepareExecution(30) as execution, redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                    with self.assertRaisesRegex(runner.RunnerError, "synthetic inventory failure"):
                        runner.prepare_live_source(repo, frozen_root, source, 20, fresh_cohort_id=runner.PREPARE_LIVE_OWNED_COHORT, baseline_bin_dir=root / "bins", execution=execution)
            self.assertTrue((frozen_root / "live-source").is_dir())
            self.assertFalse((frozen_root / "live-source.json").exists())

    def test_live_manifest_publication_rejects_symlinks_and_preserves_partial_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = root / "live-source.json"
            dangling_target = root.parent / f"outside-{root.name}"
            temporary = root / ".live-source.json.tmp"
            temporary.symlink_to(dangling_target)
            with self.assertRaisesRegex(runner.RunnerError, "temporary entry"):
                runner._publish_live_source_manifest(root, manifest, {"safe": True})
            self.assertTrue(temporary.is_symlink())
            self.assertFalse(dangling_target.exists())
            self.assertFalse(manifest.exists())

            temporary.unlink()
            manifest.symlink_to(dangling_target)
            with self.assertRaisesRegex(runner.RunnerError, "manifest entry"):
                runner._publish_live_source_manifest(root, manifest, {"safe": True})
            self.assertTrue(manifest.is_symlink())
            self.assertFalse(dangling_target.exists())

            manifest.unlink()
            manifest.write_text("user-owned\n")
            with self.assertRaisesRegex(runner.RunnerError, "manifest entry"):
                runner._publish_live_source_manifest(root, manifest, {"safe": True})
            self.assertEqual(manifest.read_text(), "user-owned\n")

            manifest.unlink()

            def expired() -> None:
                raise runner.RunnerError("synthetic deadline expired")

            with self.assertRaisesRegex(runner.RunnerError, "synthetic deadline expired"):
                runner._publish_live_source_manifest(root, manifest, {"safe": True}, check_work=expired)
            self.assertTrue(temporary.is_file())
            self.assertFalse(manifest.exists())
            self.assertEqual(json.loads(temporary.read_text()), {"safe": True})

            temporary.unlink()
            runner._publish_live_source_manifest(root, manifest, {"safe": True})
            self.assertTrue(stat.S_ISREG(manifest.lstat().st_mode))
            self.assertEqual(json.loads(manifest.read_text()), {"safe": True})
            self.assertFalse(temporary.exists())


    def test_owned_probe_nonzero_and_local_timeout_are_optional_but_scope_failures_propagate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            env = dict(os.environ)
            with runner.PrepareExecution(20) as execution:
                self.assertIsNone(runner._captured_output([sys.executable, "-c", "raise SystemExit(9)"], cwd=root, env=env, execution=execution))
                self.assertIsNone(runner._captured_output([sys.executable, "-c", "import time; time.sleep(4)"], cwd=root, env=env, timeout=0.1, execution=execution))
                execution.cancel_signal = signal.SIGINT
                with self.assertRaisesRegex(runner.RunnerError, "cancelled by signal"):
                    runner._captured_output([sys.executable, "-c", "pass"], cwd=root, env=env, execution=execution)
            with runner.PrepareExecution(20) as execution:
                execution.work_deadline = time.monotonic() - 1
                with self.assertRaisesRegex(runner.RunnerError, "work deadline expired"):
                    runner._captured_output([sys.executable, "-c", "pass"], cwd=root, env=env, execution=execution)

    def test_owned_optional_probe_cannot_hide_unverified_timeout_cleanup(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            with runner.PrepareExecution(15) as execution:
                with self.assertRaisesRegex(runner.RunnerError, "cleanup unverified after SIGKILL"):
                    runner._captured_output(
                        [sys.executable, "-c", "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(10)"],
                        cwd=root,
                        env=dict(os.environ),
                        timeout=0.75,
                        execution=execution,
                    )

    def test_owned_optional_probe_timeout_does_not_hide_final_broker_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp, runner.PrepareExecution(15) as execution:
            root = Path(temp)
            cleanup = execution._cleanup_group

            def cleanup_then_publish_error(broker: subprocess.Popen[bytes], control_write: int, status_path: Path) -> None:
                cleanup(broker, control_write, status_path)
                status_path.write_text(json.dumps({
                    "schema": 1,
                    "returncode": None,
                    "spawn_failure": False,
                    "error": "synthetic delayed broker RuntimeError",
                }))

            with patch.object(execution, "_cleanup_group", side_effect=cleanup_then_publish_error):
                with self.assertRaisesRegex(runner.RunnerError, "synthetic delayed broker RuntimeError"):
                    runner._captured_output(
                        [sys.executable, "-c", "import time; time.sleep(4)"],
                        cwd=root,
                        env=dict(os.environ),
                        timeout=0.75,
                        execution=execution,
                    )

    def test_owned_optional_probe_timeout_is_capped_at_twelve_seconds(self) -> None:
        with tempfile.TemporaryDirectory() as temp, runner.PrepareExecution(30) as execution:
            seen: dict[str, object] = {}

            def fake_run(_command: list[str], **kwargs: object) -> tuple[str, str]:
                seen.update(kwargs)
                return "probe version\n", ""

            with patch.object(execution, "run", side_effect=fake_run):
                result = runner._captured_output(["fixture-tool", "--version"], cwd=Path(temp), env={"PATH": "/tools"}, timeout=99, execution=execution)
            self.assertEqual(result, "probe version")
            self.assertEqual(seen["command_timeout"], 12.0)
            self.assertTrue(seen["optional_failure"])

    def test_nextest_owned_inventory_lists_without_visual_environment_or_run_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            tools = root / "bin"
            tools.mkdir()
            driver = root / "driver"
            driver.write_text(
                "#!" + sys.executable + "\n"
                "import os,sys,pathlib\n"
                "pathlib.Path(os.environ['DRIVER_CALLS']).write_text(' '.join(sys.argv[1:]))\n"
                "if '--list' not in sys.argv or '--format' not in sys.argv: pathlib.Path(os.environ['BODY_RAN']).touch()\n"
                "print('scenario::listed: test')\nprint('ordinary::listed: test')\n"
            )
            driver.chmod(0o755)
            cargo = tools / "cargo"
            cargo.write_text(
                "#!" + sys.executable + "\n"
                "import json,os,subprocess,sys,pathlib\n"
                "if len(sys.argv) < 3 or sys.argv[1:3] != ['nextest','list']: sys.exit(87)\n"
                "listed=subprocess.run([os.environ['DRIVER'], '--list','--format','terse'],check=True,capture_output=True,text=True).stdout\n"
                "pathlib.Path(os.environ['LISTED_NAMES']).write_text(listed)\n"
                "if 'VISUAL_TOOL' in os.environ: sys.exit(88)\n"
                "print(json.dumps({'rust-suites':[{'testcases':{'scenario::listed':{'ignored':True},'ordinary::listed':{'ignored':False}}}]}))\n"
            )
            cargo.chmod(0o755)
            calls = root / "calls"
            names = root / "names"
            body = root / "body-ran"
            env = {"PATH": f"{tools}:{os.environ['PATH']}", "DRIVER": str(driver), "DRIVER_CALLS": str(calls), "LISTED_NAMES": str(names), "BODY_RAN": str(body), "VISUAL_TOOL": "must-be-removed"}
            with patch.object(runner, "EXPECTED_SCENARIOS", 1), patch.object(runner, "EXPECTED_ORDINARY_TESTS", 1), runner.PrepareExecution(20) as execution:
                inventory = runner.nextest_inventory(root, 10, env, execution=execution)
            self.assertEqual(inventory, {"scenario::listed": True, "ordinary::listed": False})
            self.assertEqual(calls.read_text(), "--list --format terse")
            self.assertIn("scenario::listed", names.read_text())
            self.assertFalse(body.exists())

    def test_prepare_live_owned_flags_are_opt_in_and_require_fixed_cohort(self) -> None:
        legacy = runner.parser().parse_args(["prepare-live"])
        owned = runner.parser().parse_args([
            "prepare-live", "--owned-lifecycle", "--owned-budget", "37",
            "--fresh-cohort", "frozen-20261021-serial-nonincr-r1-r2",
            "--baseline-bin-dir", "/sealed/bin",
        ])
        self.assertFalse(legacy.owned_lifecycle)
        self.assertEqual(legacy.owned_budget, runner.PREPARE_OWNED_MAX_BUDGET)
        self.assertTrue(owned.owned_lifecycle)
        self.assertEqual(owned.owned_budget, 37)

    def test_prepare_live_owned_cli_rejects_missing_fixed_cohort_before_preparation(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            source = root / "root" / "source"
            repo.mkdir()
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("[workspace]\n")
            argv = [
                "frozen_baseline.py", "--repo", str(repo), "--root", str(source.parent),
                "prepare-live", "--owned-lifecycle",
            ]
            with patch.object(sys, "argv", argv), redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(runner.main(), 1)
            self.assertIn("requires explicit --fresh-cohort and --baseline-bin-dir", errors.getvalue())


if __name__ == "__main__":
    unittest.main()
