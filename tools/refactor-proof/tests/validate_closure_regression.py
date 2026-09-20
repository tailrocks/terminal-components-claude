#!/usr/bin/env python3
"""Focused closure-path regressions for the proof runner validator."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

# Keep the regression command runnable from the repository root, matching the
# documented native qualification command. The bundled worker is not an
# installed Python package; import its source tree explicitly for this test.
PROOF_ROOT = Path(__file__).resolve().parents[1]
if str(PROOF_ROOT) not in sys.path:
    sys.path.insert(0, str(PROOF_ROOT))

from runner.context import ALLOWED_V1_KEYS, Reject
from runner.validate import validate_index_close_outputs


def _write_json(path: Path, value: dict[str, object]) -> str:
    raw = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()
    path.write_bytes(raw)
    return hashlib.sha256(raw).hexdigest()


def _fixture(base_dir: Path | None = None) -> tuple[Path, dict[str, object], dict[str, object], Path]:
    root = Path(
        tempfile.mkdtemp(
            prefix="tc-proof-closure-",
            dir=str(base_dir) if base_dir is not None else None,
        )
    )
    contexts_dir = root / "contexts"
    results_dir = root / "results"
    output_dir = root / "outputs"
    contexts_dir.mkdir()
    results_dir.mkdir()
    output_dir.mkdir()
    task_id = "TASK-001"
    worktree_commit = "a" * 40
    scope_base = "b" * 40

    contexts: list[dict[str, str]] = []
    results: list[dict[str, str]] = []
    context_hashes: dict[str, str] = {}
    for check_id in ("CHK-001", "CHK-002"):
        context: dict[str, object] = {key: None for key in ALLOWED_V1_KEYS}
        context.update(
            {
                "schema": "tc-proof-context/v1",
                "run_id": str(root),
                "task_id": task_id,
                "check_id": check_id,
                "worktree_commit": worktree_commit,
                "scope_base": scope_base,
                "operation": "direct",
                "observer_sequence": ["direct"],
                "qualification": {},
            }
        )
        context_path = contexts_dir / f"{check_id}.json"
        context_hash = _write_json(context_path, context)
        context_hashes[check_id] = context_hash
        contexts.append({"check_id": check_id, "path": str(context_path), "sha256": context_hash})

        preparation = {
            "schema": "tc-proof-preparation-result/v1",
            "task_id": task_id,
            "check_id": check_id,
            "run_id": str(root),
            "worktree_commit": worktree_commit,
            "scope_base": scope_base,
            "context_sha256": context_hash,
            "status": "ready",
        }
        result_path = results_dir / f"{check_id}.json"
        result_hash = _write_json(result_path, preparation)
        results.append({"check_id": check_id, "path": str(result_path), "sha256": result_hash})

    observer = {
        "schema": "tc-proof-observer-capability/v1",
        "task_id": task_id,
        "run_id": str(root),
        "worktree_commit": worktree_commit,
        "scope_base": scope_base,
        "transport": "inherited-pipe/v1",
        "nonce_sha256": "c" * 64,
        "sequences": {check_id: ["direct"] for check_id in ("CHK-001", "CHK-002")},
        "provider": {"path": "/usr/bin/true", "sha256": "e" * 64},
    }
    observer_path = root / "observer.json"
    observer_hash = _write_json(observer_path, observer)
    index = {
        "schema": "tc-proof-context-index/v1",
        "task_id": task_id,
        "run_id": str(root),
        "worktree_commit": worktree_commit,
        "scope_base": scope_base,
        "contexts": contexts,
        "results": results,
        "observer_sequences": {check_id: ["direct"] for check_id in ("CHK-001", "CHK-002")},
        "observer": {"path": str(observer_path), "sha256": observer_hash},
    }

    output = {
        "schema": "tc-proof-runner-result/v1",
        "run_id": str(root),
        "operation": "direct",
        "context_sha256": context_hashes["CHK-002"],
        "status": "passed",
        "category": None,
        "observation_digests": ["d" * 64],
        "outputs": {},
    }
    output_path = output_dir / "CHK-002.result.json"
    output_hash = _write_json(output_path, output)
    event = {
        "records": [
            {
                "check_id": "CHK-002",
                "context_sha256": context_hashes["CHK-002"],
                "result_sha256": output_hash,
                "status": "passed",
            }
        ]
    }
    return root, index, event, output_dir


class ClosurePathTests(unittest.TestCase):
    def test_valid_closure_is_accepted(self) -> None:
        root, index, event, output_dir = _fixture()
        try:
            validate_index_close_outputs(event, index, output_dir, "CHK-001")
        finally:
            for path in sorted(root.rglob("*"), reverse=True):
                if path.is_symlink() or path.is_file():
                    path.unlink()
                elif path.is_dir():
                    path.rmdir()
            root.rmdir()

    def test_symlinked_output_and_trust_directories_are_rejected(self) -> None:
        for directory_name in ("outputs", "contexts", "results"):
            with self.subTest(directory_name=directory_name):
                root, index, event, output_dir = _fixture()
                try:
                    directory = root / directory_name
                    real_directory = root / f"{directory_name}-real"
                    directory.rename(real_directory)
                    directory.symlink_to(real_directory, target_is_directory=True)
                    with self.assertRaises(Reject) as raised:
                        validate_index_close_outputs(event, index, output_dir, "CHK-001")
                    self.assertEqual(raised.exception.category, "CLOSURE")
                finally:
                    for path in sorted(root.rglob("*"), reverse=True):
                        if path.is_symlink() or path.is_file():
                            path.unlink()
                        elif path.is_dir():
                            path.rmdir()
                    root.rmdir()

    def test_symlinked_ancestor_of_run_and_trust_paths_is_rejected(self) -> None:
        real_parent = Path(tempfile.mkdtemp(prefix="tc-proof-closure-parent-"))
        alias_parent = real_parent.with_name(real_parent.name + "-alias")
        alias_parent.symlink_to(real_parent, target_is_directory=True)
        root = None
        try:
            root, index, event, output_dir = _fixture(alias_parent)
            with self.assertRaises(Reject) as raised:
                validate_index_close_outputs(event, index, output_dir, "CHK-001")
            self.assertEqual(raised.exception.category, "CLOSURE")
        finally:
            if root is not None:
                for path in sorted(root.rglob("*"), reverse=True):
                    if path.is_symlink() or path.is_file():
                        path.unlink()
                    elif path.is_dir():
                        path.rmdir()
                root.rmdir()
            alias_parent.unlink()
            real_parent.rmdir()

    def test_allowed_tmp_and_var_aliases_remain_accepted(self) -> None:
        for base_dir in (Path("/tmp"), Path("/var/tmp")):
            with self.subTest(base_dir=base_dir):
                root, index, event, output_dir = _fixture(base_dir)
                try:
                    validate_index_close_outputs(event, index, output_dir, "CHK-001")
                finally:
                    for path in sorted(root.rglob("*"), reverse=True):
                        if path.is_symlink() or path.is_file():
                            path.unlink()
                        elif path.is_dir():
                            path.rmdir()
                    root.rmdir()

    def test_hardlinked_trust_and_output_artifacts_are_rejected(self) -> None:
        for relative in (
            "contexts/CHK-002.json",
            "results/CHK-002.json",
            "outputs/CHK-002.result.json",
        ):
            with self.subTest(relative=relative):
                root, index, event, output_dir = _fixture()
                try:
                    source = root / relative
                    alias = root / f"alias-{source.name}"
                    os.link(source, alias)
                    with self.assertRaises(Reject) as raised:
                        validate_index_close_outputs(event, index, output_dir, "CHK-001")
                    self.assertEqual(raised.exception.category, "CLOSURE")
                finally:
                    for path in sorted(root.rglob("*"), reverse=True):
                        if path.is_symlink() or path.is_file():
                            path.unlink()
                        elif path.is_dir():
                            path.rmdir()
                    root.rmdir()


if __name__ == "__main__":
    unittest.main()
