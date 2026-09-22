#!/usr/bin/env python3
"""Focused tests for shipped accounting functions."""

from __future__ import annotations

import json
import os
import sys
import tempfile
import threading
import types
import unittest
from pathlib import Path


PROOF = Path(__file__).resolve().parents[1]


def _install_packages() -> None:
    if "refactor_proof" in sys.modules:
        return
    package = types.ModuleType("refactor_proof")
    package.__path__ = [str(PROOF)]
    sys.modules["refactor_proof"] = package
    sys.modules["refactor_proof.runner"] = types.ModuleType("refactor_proof.runner")
    sys.modules["refactor_proof.runner"].__path__ = [str(PROOF / "runner")]
    sys.modules["refactor_proof.accounting"] = types.ModuleType("refactor_proof.accounting")
    sys.modules["refactor_proof.accounting"].__path__ = [str(PROOF / "accounting")]


_install_packages()

from refactor_proof.accounting.dispatch import run_account_tests  # noqa: E402
from refactor_proof.accounting.extension import validate_extension_event  # noqa: E402
from refactor_proof.accounting.fixture import (  # noqa: E402
    build_outputs,
    validate_event,
    validate_inventory_integrity,
)
from refactor_proof.accounting.identity import (  # noqa: E402
    canonical_identity,
    original_name,
    resolve_required_names,
)
from refactor_proof.accounting.qualification import (  # noqa: E402
    validate_qualification_oracle_namespace,
    is_qualification_schema,
    validate_qualification_schema,
)
from refactor_proof.runner.context import Reject  # noqa: E402
from refactor_proof.runner.json_util import canonical, sha256_bytes  # noqa: E402


class IdentityTests(unittest.TestCase):
    def test_canonical_identity_is_sorted_json(self) -> None:
        left = {"name": "test_future", "package": "tiny-a", "profile": "primary"}
        right = {"profile": "primary", "package": "tiny-a", "name": "test_future"}
        self.assertEqual(canonical_identity(left), canonical_identity(right))

    def test_relocations_preserve_required_names(self) -> None:
        inventory = {
            "required": ["test_closed", "test_draw", "test_future", "test_update"],
            "relocations": {"test_draw": "test_renamed"},
        }
        self.assertEqual(
            resolve_required_names(inventory),
            ["test_closed", "test_future", "test_renamed", "test_update"],
        )
        self.assertEqual(original_name("test_renamed", inventory["relocations"]), "test_draw")
        self.assertEqual(original_name("test_future", inventory["relocations"]), "test_future")


class FixtureTests(unittest.TestCase):
    def test_closed_and_future_overlap_is_integrity(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_inventory_integrity({"closed": ["test_closed"], "future": {"test_closed": "owner"}})
        self.assertEqual(raised.exception.category, "INTEGRITY")

    def test_build_outputs_keeps_unresolved_and_closed_contribution(self) -> None:
        events = [{"operation": "account-tests", "payload": {"results": []}}]
        inventory = {
            "future": {"test_future": "terminal-components/completion/999"},
            "future_scenarios": {"tiny-shell": "terminal-components/completion/999"},
            "contributions": [
                {
                    "id": "tiny-shell.route",
                    "previously_closed": True,
                    "required_for_closure": True,
                }
            ],
        }
        self.assertEqual(
            build_outputs(events, inventory),
            {
                "observations": events,
                "unresolved_tests": ["test_future"],
                "closed_contributions": ["tiny-shell.route"],
                "unresolved_scenarios": ["tiny-shell"],
            },
        )

    def test_unknown_failure_is_test_accounting(self) -> None:
        inventory = {
            "required": ["test_closed", "test_draw", "test_future", "test_update"],
            "relocations": {},
            "closed": ["test_closed"],
            "future": {"test_future": "terminal-components/completion/999"},
            "package": "tiny",
            "target": "unit",
            "profile": "primary",
            "contributions": [],
        }
        event = {
            "exit": 0,
            "payload": {
                "discovered": ["test_closed", "test_draw", "test_future", "test_update"],
                "results": [
                    {"id": "test_closed", "status": "passed"},
                    {"id": "test_draw", "status": "passed"},
                    {"id": "test_future", "status": "failed"},
                    {"id": "test_update", "status": "passed"},
                    {"id": "test_unknown", "status": "failed"},
                ],
                "package": "tiny",
                "target": "unit",
                "profile": "primary",
                "scenarios": [{"state": {}, "frame": {}}],
            },
        }
        with self.assertRaises(Reject) as raised:
            validate_event(event, inventory, {})
        self.assertEqual(raised.exception.category, "TEST_ACCOUNTING")


class QualificationSchemaTests(unittest.TestCase):
    def test_native_extension_accepts_synthetic_cli_namespace(self) -> None:
        validate_qualification_oracle_namespace(
            "synthetic",
            {"schema": "tc-proof-runner-extension/v1", "family": "native"},
        )

    def test_non_extension_native_rejects_synthetic_cli_namespace(self) -> None:
        with self.assertRaises(Reject) as raised:
            validate_qualification_oracle_namespace(
                "synthetic", {"schema": "other", "family": "native"}
            )
        self.assertEqual(raised.exception.category, "PROTOCOL")

    def test_runner_context_v1_is_qualification(self) -> None:
        context = {
            "schema": "tc-proof-runner-context/v1",
            "run_id": "r",
            "operation": "account-tests",
            "tree": "0" * 40,
            "oracle_commit": "1" * 40,
            "oracle_tree": "2" * 40,
            "bundle": "/tmp/oracle.bundle",
            "bundle_sha256": "a" * 64,
            "tool": {"path": "/usr/bin/python3", "sha256": "b" * 64},
            "dependencies": [{"accepted": True, "integrated": True}],
            "adapter": {"changes": []},
            "lane": "direct",
            "axes": {"lanes": ["direct"], "widths": [8], "palettes": ["blue"]},
            "members": ["tiny/direct/8/blue"],
            "inventory": {},
            "evidence": [],
            "configuration": {},
        }
        self.assertTrue(is_qualification_schema(context))
        validate_qualification_schema(context)

    def test_native_context_is_not_qualification(self) -> None:
        self.assertFalse(is_qualification_schema({"schema": "tc-proof-context/v1"}))


class ExtensionTests(unittest.TestCase):
    def test_borrowed_owner_is_rejected(self) -> None:
        identity_a = {
            "package": "tiny-a",
            "target": "unit",
            "profile": "primary",
            "source_commit": "c" * 40,
            "name": "test_future",
        }
        identity_b = dict(identity_a)
        identity_b["package"] = "tiny-b"
        identity_b["target"] = "integration"
        identity_b["profile"] = "msrv"
        contract = {
            "original": {
                "source_sha256": "s" * 64,
                "non_test_sha256": "n" * 64,
                "assertions": {"test_compatible": "x", "test_future": "y"},
            },
            "mode": "production",
            "accepted_inventory": True,
            "accepted_disposition": True,
            "required": [
                {**identity_a, "name": "test_compatible"},
                identity_a,
                {**identity_b, "name": "test_compatible"},
                identity_b,
            ],
            "future": [
                {
                    "identity": identity_a,
                    "owner": "terminal-components/completion/991",
                    "classification": "failed",
                },
                {
                    "identity": identity_b,
                    "owner": "terminal-components/completion/991",
                    "classification": "failed",
                },
            ],
            "closed": [],
        }
        event = {
            "payload": {
                "source": {
                    "assertions": contract["original"]["assertions"],
                    "non_test_sha256": "n" * 64,
                },
                "archive_sha256": "s" * 64,
                "runs": [],
            }
        }
        with self.assertRaises(Reject) as raised:
            validate_extension_event(event, contract, run_id="run", source_commit="c" * 40)
        self.assertEqual(raised.exception.category, "TEST_ACCOUNTING")


class DispatchTests(unittest.TestCase):
    def test_hash_mismatch_rejects_integrity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "context.json"
            context = {
                "schema": "tc-proof-runner-context/v1",
                "run_id": "run",
                "operation": "account-tests",
                "tree": "0" * 40,
                "oracle_commit": "1" * 40,
                "oracle_tree": "2" * 40,
                "bundle": "bundle",
                "bundle_sha256": "a" * 64,
                "tool": {"path": sys.executable, "sha256": "b" * 64},
                "dependencies": [{"accepted": True, "integrated": True}],
                "adapter": {"changes": []},
                "lane": "direct",
                "axes": {"lanes": ["direct", "pty"], "widths": [8, 12], "palettes": ["blue", "yellow"]},
                "members": [],
                "inventory": {"required": [], "closed": [], "future": {}},
                "evidence": [],
                "configuration": {},
            }
            raw = canonical(context)
            path.write_bytes(raw)
            result_path = Path(directory) / "result.json"
            env = {
                "TC_PROOF_CONTEXT_SHA256": "0" * 64,
                "TC_PROOF_RUN_ID": "run",
                "TC_PROOF_ORACLE_COMMIT": "1" * 40,
                "TC_PROOF_SOURCE_TREE": "0" * 40,
                "TC_PROOF_RESULT": str(result_path),
            }
            saved = {key: os.environ.get(key) for key in env}
            os.environ.update(env)
            try:
                code = run_account_tests(path)
            finally:
                for key, value in saved.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            self.assertEqual(code, 1)
            report = json.loads(result_path.read_text())
            self.assertEqual(report["status"], "rejected")
            self.assertEqual(report["category"], "INTEGRITY")
            self.assertEqual(report["observation_digests"], [])

    def test_executed_inventory_failure_is_accounted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "context.json"
            inventory = {
                "source_commit": "1" * 40,
                "package": "tiny",
                "target": "unit",
                "profile": "primary",
                "required": ["test_closed", "test_draw", "test_future", "test_update"],
                "closed": ["test_closed"],
                "future": {"test_future": "terminal-components/completion/999"},
                "relocations": {},
                "contributions": [
                    {
                        "id": "tiny-shell.route",
                        "previously_closed": True,
                        "required_for_closure": True,
                        "state_key": "route_value",
                        "expected": 8,
                    }
                ],
                "scenario_expectations": {"tiny-shell": {"custom_art": "<>"}},
                "future_scenarios": {"tiny-shell": "terminal-components/completion/999"},
            }
            context = {
                "schema": "tc-proof-runner-context/v1",
                "run_id": "run",
                "operation": "account-tests",
                "tree": "0" * 40,
                "oracle_commit": "1" * 40,
                "oracle_tree": "2" * 40,
                "bundle": "bundle",
                "bundle_sha256": "a" * 64,
                "tool": {"path": sys.executable, "sha256": "b" * 64},
                "dependencies": [{"accepted": True, "integrated": True}],
                "adapter": {"changes": []},
                "lane": "direct",
                "axes": {"lanes": ["direct", "pty"], "widths": [8, 12], "palettes": ["blue", "yellow"]},
                "members": [
                    "tiny/direct/8/blue",
                    "tiny/direct/8/yellow",
                    "tiny/direct/12/blue",
                    "tiny/direct/12/yellow",
                    "tiny/pty/8/blue",
                    "tiny/pty/8/yellow",
                    "tiny/pty/12/blue",
                    "tiny/pty/12/yellow",
                ],
                "inventory": inventory,
                "evidence": [],
                "configuration": {"seed": 7, "palette": "blue"},
            }
            raw = canonical(context)
            path.write_bytes(raw)
            digest = sha256_bytes(raw)
            event = {
                "exit": 0,
                "payload": {
                    "discovered": ["test_closed", "test_draw", "test_future", "test_update"],
                    "results": [
                        {"id": "test_closed", "status": "passed"},
                        {"id": "test_draw", "status": "passed"},
                        {"id": "test_future", "status": "failed"},
                        {"id": "test_update", "status": "passed"},
                    ],
                    "package": "tiny",
                    "target": "unit",
                    "profile": "primary",
                    "scenarios": [
                        {
                            "id": "tiny-shell",
                            "state": {"route_value": 8},
                            "frame": {"custom_art": "*"},
                        }
                    ],
                },
            }
            request_read, request_write = os.pipe()
            response_read, response_write = os.pipe()

            def serve() -> None:
                with os.fdopen(request_read, "rb") as incoming, os.fdopen(response_write, "wb") as outgoing:
                    line = incoming.readline(4097)
                    self.assertTrue(line.endswith(b"\n"))
                    payload = json.loads(line)
                    self.assertEqual(
                        set(payload),
                        {"schema", "nonce", "operation", "source_commit", "tree"},
                    )
                    outgoing.write(canonical(event) + b"\n")
                    outgoing.flush()

            thread = threading.Thread(target=serve)
            thread.start()
            result_path = Path(directory) / "result.json"
            env = {
                "TC_PROOF_CONTEXT_SHA256": digest,
                "TC_PROOF_RUN_ID": "run",
                "TC_PROOF_ORACLE_COMMIT": "1" * 40,
                "TC_PROOF_SOURCE_TREE": "0" * 40,
                "TC_PROOF_RESULT": str(result_path),
                "TC_PROOF_OBSERVER_NONCE": "nonce",
                "TC_PROOF_OBSERVER_REQUEST_FD": str(request_write),
                "TC_PROOF_OBSERVER_RESPONSE_FD": str(response_read),
            }
            saved = {key: os.environ.get(key) for key in env}
            os.environ.update(env)
            try:
                code = run_account_tests(path)
            finally:
                thread.join(timeout=5)
                for key, value in saved.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            self.assertEqual(code, 0)
            report = json.loads(result_path.read_text())
            self.assertEqual(report["status"], "passed")
            self.assertEqual(report["outputs"]["unresolved_tests"], ["test_future"])
            self.assertEqual(report["outputs"]["closed_contributions"], ["tiny-shell.route"])
            self.assertEqual(report["outputs"]["unresolved_scenarios"], ["tiny-shell"])


if __name__ == "__main__":
    unittest.main()
