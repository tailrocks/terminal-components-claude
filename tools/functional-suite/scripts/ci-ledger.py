#!/usr/bin/env python3
"""Initialize, validate, and reconcile Common functional-suite CI evidence.

This helper never starts Cargo, Nextest, or an application process. The public
process runner is ``run-ci.sh``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import tomllib
import uuid
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, NoReturn


PLAN_ID = "PR38-CI-RUN-LEDGER-TP1-R4"
PLAN_SHA256 = "578da771c7080481a51c1ef5ab84d653bdbb093a2cedfdc8e4bf1d2056cb095d"
ACCEPTED_FUNCTIONAL_SHA256 = "6bad4294a37e86e1cfa79f9133ac319fde8e7500d34831660ae1a1c728b3efa7"
ACCEPTED_CASES_SHA256 = "469f87b6a83f3bfc23f1a4e6ccae214b5d513535c25ef763cb16ee85d516671f"
ACCEPTED_NEXTEST_CONFIG_SHA256 = "cbe976f7a29c4d1650f9cfc7aece915ba3db39745c2a2371728701c9aecce58b"
ACCEPTED_CARGO_TOML_SHA256 = "e0297bbeaf04149cc8356cae89f02364effcedd137b90f9ecdd7978f91a5949f"
ACCEPTED_CARGO_LOCK_SHA256 = "b0e56056f43985b15bc5695060a4c6fe5d7f97790b2f0dd9fbbf2fbc81ec9f42"
ACCEPTED_ENGINE_PINS_FILE_SHA256 = "b790e0d55cd2d2d192090aeb5aebc6dfa9d1c7e2b892dd71e8afd81e2ab09b00"
ACCEPTED_TUI_TEST_PATCH_SHA256 = "1c827f5074a28a8d129fa93308c1a39e57a163bfb405d2d440d6b1494a9a399d"
ACCEPTED_TUISCOTTI_PATCH_SHA256 = "ed4c9ac1d03b6aac8347e7018311aebad100d37a1a8fea400378ee434038955d"
TUI_TEST_PATCH_PATH = "engine-patches/tui-test-rs-base-to-v7.patch"
TUISCOTTI_PATCH_PATH = "engine-patches/tuiscotti-ac5f70d-to-pixel-gap.patch"
PACKAGE_VERSION = "0.1.0"
NEXTEST_PROFILE = "ci"
CARGO_PROFILE = "dev"
EXPECTED_TARGET_TRIPLE = "aarch64-apple-darwin"
EXPECTED_PR26_SHA = "5b3dbd7ae799f325efbb4f4cc11b2f5518dec948"
EXPECTED_APPROVAL_SHA256 = "a62e1eeedbab3c114f833a7d52dd45e5e8b99ce7bb40018016d22d086d7386ec"
EXPECTED_SAMPLE_MANIFEST_SHA256 = "a5074c6fd2860d8c612379fc1a324baccc0a245082271c00ac1afe59e5cc8a53"
EXPECTED_SAMPLE_COUNT = 42
SUITE_ID = "termrock-functional-suite-v3"
PACKAGE_NAME = "termrock-functional-suite"
BINARY_NAME = "functional"
BINARY_ID = f"{PACKAGE_NAME}::{BINARY_NAME}"
ENGINE_PINS = {
    "tuiscotti-tui-test": "7abeac8478aa9c05ec7bef93afb10209cc348bec",
    "tuiscotti": "d8d54155947f6303a41dfd6079b53c0abc16f29a",
}
TUI_TEST_FILES = (
    "crates/tui-test/src/engine.rs",
    "crates/tui-test/src/launch.rs",
    "crates/tui-test/src/lib.rs",
    "crates/tui-test/src/record.rs",
    "crates/tui-test/src/record/worker.rs",
    "crates/tui-test/src/runtime.rs",
    "crates/tui-test/src/session.rs",
    "crates/tui-test/src/snapshot.rs",
    "crates/tui-test/src/terminal/alacritty.rs",
    "crates/tui-test/src/terminal/emu.rs",
    "crates/tui-test/src/terminal/pty.rs",
)
TUISCOTTI_FILES = (
    "crates/tuiscotti/docs/check.md",
    "crates/tuiscotti/docs/report.md",
    "crates/tuiscotti/src/check.rs",
    "crates/tuiscotti/src/report.rs",
)
LIST_TOP_LEVEL_KEYS = {"rust-build-meta", "test-count", "rust-suites"}
LIST_BUILD_META_KEYS = {
    "target-directory",
    "build-directory",
    "base-output-directories",
    "non-test-binaries",
    "build-script-out-dirs",
    "build-script-info",
    "linked-paths",
    "platforms",
    "target-platforms",
    "target-platform",
}
LIST_HOST_PLATFORM_KEYS = {"triple", "target-features"}
LIST_PLATFORMS_KEYS = {"host", "targets"}
LIST_SUITE_KEYS = {
    "package-name",
    "binary-id",
    "binary-name",
    "package-id",
    "kind",
    "binary-path",
    "build-platform",
    "cwd",
    "status",
    "testcases",
}
LIST_TESTCASE_KEYS = {"kind", "ignored", "filter-match"}
LIST_MISMATCH_REASONS = {
    "not-benchmark",
    "ignored",
    "string",
    "expression",
    "partition",
    "rerun-already-passed",
    "default-filter",
}
CASE_RECEIPT_REQUIRED_KEYS = {
    "schema_version",
    "suite_id",
    "run_id",
    "case_started_at",
    "case_ended_at",
    "test_name",
    "case_id",
    "engine_pins",
    "fault_control",
    "actions",
    "planned_checkpoint_names",
    "not_run_checkpoints",
    "status",
    "failure_kind",
    "comparison_failure_kind",
    "stage",
    "launched",
    "error",
}
CASE_RECEIPT_OPTIONAL_KEYS = {"phases"}
ACTION_RECEIPT_KEYS = {
    "action_id",
    "case_id",
    "phase_name",
    "checkpoint_name",
    "sequence",
    "action_kind",
    "attempted_at",
    "dispatch_completed_at",
    "dispatch_state",
    "observation_at",
    "observation_state",
}
PHASE_RECEIPT_KEYS = {
    "name",
    "motion",
    "cleanup",
    "planned_checkpoint_names",
    "status",
    "comparison_failure_kind",
    "launched",
    "cleanup_actions_attempted",
    "wait_exit_succeeded",
    "exit_status",
    "exit_status_zero",
    "exit_signal",
    "no_signal",
    "eof_drained",
    "post_drain_exit_confirmed",
    "raw_pty_sha256",
    "raw_checkpoint_prefixes",
    "terminal_restored",
    "cleanup_snapshot_file",
    "cleanup_snapshot_sha256",
    "cleanup_raw_prefix_sha256",
    "error",
}
NOT_RUN_CHECKPOINT_KEYS = {"name", "status", "reason"}
CASE_STATUSES = {"pending", "passed", "failed"}
CASE_FAILURE_KINDS = {"setup", "application", "execution", "capture", "comparison"}
COMPARISON_FAILURE_KINDS = {"missing_baseline", "test_id", "policy", "state", "contract", "dimensions", "pixels"}
CASE_STAGES = {"manifest", "plan", "fixture", "launch", "journey", "finalize"}
PHASE_STATUSES = {"not_run", "setup_failed", "launched", "passed", "failed"}
ACTION_KINDS = {"key", "write", "resize", "mouse_click"}
ACTION_DISPATCH_STATES = {"attempted", "dispatched", "failed", "suppressed_by_control"}
ACTION_OBSERVATION_STATES = {"matched", "mismatched"}
PHASE_CLEANUP_ACTION_CHECKPOINTS = {
    "cleanup_dialog": "Ctrl+Q",
    "cleanup_selected_quit": "Right",
}
STATIC_EXPECTED_FILES = {"BASELINE-APPROVAL.json", "README.md"}
TERMINAL_ROW_STATES = {"passed", "failed", "setup_failed", "not_run"}
SIDE_NAMES = ("pr26", "candidate")
RUN_ID_RE = re.compile(r"^[A-Za-z0-9_.-]{1,128}$")
RFC3339_RE = re.compile(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z$")
CASE_NAME_RE = re.compile(r"^[A-Za-z0-9_-]+$")
CASE_ID_RE = re.compile(r"^[A-Za-z0-9_.-]+$")


class LedgerError(Exception):
    """A fail-closed evidence or identity error."""


def fail(message: str) -> NoReturn:
    print(f"ci-ledger: {message}", file=sys.stderr)
    raise SystemExit(2)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise LedgerError(message)


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def safe_run_id(value: str) -> bool:
    return bool(RUN_ID_RE.fullmatch(value))


def check_no_symlink_components(path: Path, *, allow_missing_leaf: bool = False) -> None:
    absolute = Path(os.path.abspath(path))
    cursor = Path(absolute.anchor)
    parts = absolute.parts[1:]
    for index, part in enumerate(parts):
        cursor = cursor / part
        if allow_missing_leaf and index == len(parts) - 1 and not cursor.exists():
            continue
        try:
            mode = cursor.lstat().st_mode
        except FileNotFoundError:
            continue
        require(not stat.S_ISLNK(mode), f"symlink path component refused: {cursor}")


def atomic_write(path: Path, data: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    require(not path.is_symlink(), f"symlink output refused: {path}")
    fd, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temp_path = Path(temporary)
    try:
        with os.fdopen(fd, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        os.chmod(temp_path, mode)
        os.replace(temp_path, path)
    finally:
        if temp_path.exists():
            temp_path.unlink()


def atomic_json(path: Path, value: Any) -> None:
    data = (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()
    atomic_write(path, data)


def read_json(path: Path, label: str) -> Any:
    require(path.is_file() and not path.is_symlink(), f"{label} is missing or not a regular file: {path}")

    def unique_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON key: {key!r}")
            result[key] = value
        return result

    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_pairs)
    except (OSError, UnicodeError, ValueError) as error:
        raise LedgerError(f"{label} is invalid JSON: {error}") from error


def safe_relpath(value: str, label: str) -> str:
    require(isinstance(value, str), f"{label} is not a string")
    require(value and "\\" not in value and not value.startswith("/"), f"bad {label}: {value!r}")
    path = Path(value)
    require(all(part not in ("", ".", "..") for part in path.parts), f"unsafe {label}: {value!r}")
    return path.as_posix()


def normalize_case(case: dict[str, Any], index: int) -> dict[str, Any]:
    name = case.get("name")
    require(isinstance(name, str) and CASE_NAME_RE.fullmatch(name), f"cases[{index}].name is invalid")
    case_id = case.get("case_id", name)
    require(
        isinstance(case_id, str)
        and case_id not in {".", ".."}
        and CASE_ID_RE.fullmatch(case_id),
        f"cases[{index}].case_id is invalid",
    )
    names: list[str] = []
    planned_phases: list[dict[str, Any]] = []
    if isinstance(case.get("steps"), list):
        names.extend(step.get("name") for step in case["steps"] if isinstance(step, dict))
    if isinstance(case.get("phases"), list):
        for phase in case["phases"]:
            if isinstance(phase, dict) and isinstance(phase.get("steps"), list):
                phase_names = [step.get("name") for step in phase["steps"] if isinstance(step, dict)]
                names.extend(phase_names)
                planned_phases.append({
                    "name": phase.get("name"),
                    "motion": phase.get("motion"),
                    "cleanup": phase.get("cleanup"),
                    "planned_checkpoint_names": phase_names,
                })
    if not names and case.get("native_steps") == ["help", "return", "exit"]:
        names.extend(("opened", "help", "returned", "exited"))
    if names:
        require(all(isinstance(item, str) and CASE_NAME_RE.fullmatch(item) for item in names),
                f"case {name} has an invalid checkpoint name")
        require(len(set(names)) == len(names), f"case {name} has duplicate checkpoint names")
    phase_names = [phase["name"] for phase in planned_phases]
    if planned_phases:
        require(all(isinstance(item, str) and CASE_NAME_RE.fullmatch(item) for item in phase_names),
                f"case {name} has an invalid phase name")
        require(len(set(phase_names)) == len(phase_names), f"case {name} has duplicate phase names")
    return {
        "test_name": name,
        "case_id": case_id,
        "planned_checkpoint_names": names if names else None,
        "planned_checkpoint_count": len(names) if names else None,
        "planned_phase_names": phase_names,
        "planned_phases": planned_phases,
    }


def load_inventory(suite_root: Path) -> tuple[list[dict[str, Any]], dict[str, str], str]:
    required_files = {
        "Cargo.toml": ACCEPTED_CARGO_TOML_SHA256,
        "Cargo.lock": ACCEPTED_CARGO_LOCK_SHA256,
        ".config/nextest.toml": ACCEPTED_NEXTEST_CONFIG_SHA256,
        "engine-pins.json": ACCEPTED_ENGINE_PINS_FILE_SHA256,
        TUI_TEST_PATCH_PATH: ACCEPTED_TUI_TEST_PATCH_SHA256,
        TUISCOTTI_PATCH_PATH: ACCEPTED_TUISCOTTI_PATCH_SHA256,
    }
    for relative, expected_sha in required_files.items():
        path = suite_root / relative
        require(path.is_file() and not path.is_symlink(), f"accepted Suite source is missing or unsafe: {relative}")
        actual_sha = sha256_file(path)
        require(actual_sha == expected_sha,
                f"accepted Suite source SHA mismatch for {relative}: {actual_sha}")
    manifest_path = suite_root / "cases" / "cases.toml"
    require(manifest_path.is_file() and not manifest_path.is_symlink(), f"manifest missing or unsafe: {manifest_path}")
    try:
        manifest_bytes = manifest_path.read_bytes()
        manifest = tomllib.loads(manifest_bytes.decode("utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise LedgerError(f"cannot parse manifest: {error}") from error
    functional_path = suite_root / "tests" / "functional.rs"
    require(functional_path.is_file() and not functional_path.is_symlink(),
            f"accepted functional harness is missing or unsafe: {functional_path}")
    functional_sha = sha256_file(functional_path)
    require(functional_sha == ACCEPTED_FUNCTIONAL_SHA256,
            f"functional harness is not the accepted freeze: {functional_sha}")
    manifest_sha = sha256_bytes(manifest_bytes)
    require(manifest_sha == ACCEPTED_CASES_SHA256,
            f"case manifest is not the accepted freeze: {manifest_sha}")
    raw_cases = manifest.get("cases")
    require(isinstance(raw_cases, list) and len(raw_cases) == 89, "manifest must contain exactly 89 case rows")
    cases = [normalize_case(case, index) for index, case in enumerate(raw_cases)]
    names = [case["test_name"] for case in cases]
    ids = [case["case_id"] for case in cases]
    require(len(set(names)) == len(names), "manifest case names are not unique")
    require(len(set(ids)) == len(ids), "manifest case IDs are not unique")
    pins = manifest.get("engine_pins")
    require(isinstance(pins, dict), "manifest engine_pins is missing")
    require(pins == ENGINE_PINS, "manifest engine pins differ from the accepted exact revisions")
    suite_id = manifest.get("version", {}).get("suite_id")
    require(suite_id == SUITE_ID, f"manifest suite ID differs: {suite_id!r}")
    return cases, pins, manifest_sha


def suite_source_identity(suite_root: Path) -> dict[str, str]:
    paths = {
        "Cargo.toml": "Cargo.toml",
        "Cargo.lock": "Cargo.lock",
        "functional.rs": "tests/functional.rs",
        "cases.toml": "cases/cases.toml",
        "nextest_config": ".config/nextest.toml",
        "engine-pins.json": "engine-pins.json",
        "tui_test_patch": TUI_TEST_PATCH_PATH,
        "tuiscotti_patch": TUISCOTTI_PATCH_PATH,
    }
    return {identity: sha256_file(suite_root / relative) for identity, relative in paths.items()}


def initial_row(case: dict[str, Any]) -> dict[str, Any]:
    return {
        **case,
        "selected": None,
        "status": "pending",
        "reason": "case has not been finalized",
        "reached_checkpoint_count": 0,
        "compared_checkpoint_count": 0,
        "matched_checkpoint_count": 0,
        "junit_status": None,
        "case_result_path": None,
        "case_result_sha256": None,
    }


def participant_dir(run_root: Path, name: str) -> Path:
    require(name in SIDE_NAMES, f"unknown participant: {name}")
    return run_root / "participants" / name


def load_run(run_root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    invocation_path = run_root / "invocation.json"
    invocation = read_json(invocation_path, "invocation")
    inventory_path = run_root / "inventory.json"
    inventory = read_json(inventory_path, "inventory")
    require(isinstance(invocation, dict), "invocation root is not an object")
    require(isinstance(inventory, dict) and set(inventory) == {
        "schema_version", "run_id", "manifest_sha256", "case_count", "cases",
    }, "inventory has an unsupported shape")
    require(invocation.get("run_id") == inventory.get("run_id"), "invocation and inventory IDs differ")
    execution_root_raw = invocation.get("execution_root")
    require(isinstance(execution_root_raw, str) and Path(execution_root_raw).is_absolute(),
            "invocation execution root is missing or not absolute")
    try:
        execution_root = Path(execution_root_raw).resolve(strict=False)
    except (OSError, RuntimeError) as error:
        raise LedgerError("invocation execution root cannot be resolved") from error
    require(str(execution_root) == execution_root_raw,
            "invocation execution root is not in canonical absolute form")
    require(type(inventory.get("schema_version")) is int and inventory["schema_version"] == 1,
            "inventory schema version is unsupported")
    require(type(inventory.get("case_count")) is int and inventory["case_count"] == 89,
            "inventory case count is not 89")
    require(isinstance(inventory.get("cases"), list) and len(inventory["cases"]) == 89,
            "inventory does not contain all 89 cases")
    suite_root_raw = invocation.get("suite_root")
    require(isinstance(suite_root_raw, str), "invocation Suite root is missing")
    cases, pins, manifest_sha = load_inventory(Path(suite_root_raw).resolve(strict=True))
    require(manifest_sha == ACCEPTED_CASES_SHA256
            and inventory.get("manifest_sha256") == manifest_sha
            and invocation.get("manifest_sha256") == manifest_sha,
            "invocation/inventory manifest identity differs from the accepted frozen case plan")
    require(invocation.get("engine_pins") == pins and inventory.get("cases") == cases,
            "invocation/inventory rows differ from the accepted frozen case plan")
    return invocation, inventory["cases"]


def create_invocation(args: argparse.Namespace) -> None:
    suite_root = args.suite_root.resolve(strict=True)
    run_root = Path(os.path.abspath(args.run_root))
    require(safe_run_id(args.run_id), "run ID is not a safe single component")
    check_no_symlink_components(run_root, allow_missing_leaf=True)
    require(not run_root.exists(), f"run directory already exists: {run_root}")
    run_root.mkdir(parents=True, mode=0o700)
    try:
        execution_context = json.loads(args.execution_context_json)
    except json.JSONDecodeError as error:
        raise LedgerError(f"execution context must be JSON: {error}") from error
    base = {
        "schema_version": 1,
        "plan_id": PLAN_ID,
        "plan_sha256": PLAN_SHA256,
        "run_id": args.run_id,
        "started_at": utc_now(),
        "suite_root": str(suite_root),
        "execution_root": str(run_root.resolve(strict=False)),
        "status": "initializing",
        "report_complete": False,
        "success": False,
        "diagnostic_filter": args.diagnostic_filter,
        "execution_mode": args.execution_mode,
        "execution_context": execution_context,
        "dependency_sources_ready": False,
        "dependency_sources_error": None,
        "expected_store_verified": False,
        "export_verified": False,
        "interruption": {"signal": None, "raw_child_exit": None, "child_pid": None, "child_process_group": None},
        "commands": [],
        "participants": {},
    }
    try:
        cases, pins, manifest_sha = load_inventory(suite_root)
    except LedgerError as error:
        atomic_json(run_root / "inventory-unavailable.json", {
            "schema_version": 1,
            "run_id": args.run_id,
            "created_at": utc_now(),
            "manifest_path": str(suite_root / "cases" / "cases.toml"),
            "error": str(error),
            "inventory_available": False,
        })
        base.update({"status": "inventory_unavailable", "inventory_available": False})
        atomic_json(run_root / "invocation.json", base)
        print(f"inventory unavailable: {error}", file=sys.stderr)
        raise SystemExit(2)

    base.update({
        "status": "initialized",
        "inventory_available": True,
        "manifest_sha256": manifest_sha,
        "case_count": len(cases),
        "engine_pins": pins,
    })
    inventory = {
        "schema_version": 1,
        "run_id": args.run_id,
        "manifest_sha256": manifest_sha,
        "case_count": len(cases),
        "cases": cases,
    }
    atomic_json(run_root / "inventory.json", inventory)
    for side in SIDE_NAMES:
        side_root = participant_dir(run_root, side)
        (side_root / "out").mkdir(parents=True, mode=0o700)
        (side_root / "commands").mkdir(mode=0o700)
        side_id = f"{args.run_id}-{side}"
        require(safe_run_id(side_id), f"derived side run ID is unsafe: {side_id}")
        state = {
            "schema_version": 1,
            "participant": side,
            "run_id": side_id,
            "created_at": utc_now(),
            "built": False,
            "build_exit": None,
            "build_target_root": None,
            "run_launched": False,
            "run_exit": None,
            "run_signal": None,
            "run_pid": None,
            "run_process_group": None,
            "never_launched_reason": None,
            "junit_path": None,
            "selection_count": None,
        }
        atomic_json(side_root / "state.json", state)
        atomic_json(side_root / "ledger.json", {
            "schema_version": 1,
            "participant": side,
            "run_id": side_id,
            "created_at": utc_now(),
            "selection_known": False,
            "report_complete": False,
            "cases": [initial_row(case) for case in cases],
        })
        base["participants"][side] = {"run_id": side_id, "report_complete": False, "success": False}
    atomic_json(run_root / "invocation.json", base)
    print(run_root)


def enumerate_tree(root: Path) -> list[dict[str, Any]]:
    require(root.is_dir() and not root.is_symlink(), f"export root is missing or unsafe: {root}")
    records: list[dict[str, Any]] = []
    lower_paths: dict[str, str] = {}
    excluded = {".git", "__pycache__", "forks", "target"}
    for current, dirnames, filenames in os.walk(root, topdown=True, followlinks=False):
        directory = Path(current)
        kept: list[str] = []
        for name in sorted(dirnames):
            child = directory / name
            if child.is_symlink():
                raise LedgerError(f"symlink directory in export: {child.relative_to(root).as_posix()}")
            relative = child.relative_to(root).as_posix()
            if relative.split("/", 1)[0] in excluded:
                continue
            kept.append(name)
        dirnames[:] = kept
        for name in sorted(filenames):
            path = directory / name
            relative = path.relative_to(root).as_posix()
            if relative == ".termrock-export-manifest.json":
                continue
            if relative.split("/", 1)[0] in excluded:
                continue
            mode = path.lstat().st_mode
            if stat.S_ISLNK(mode):
                raise LedgerError(f"symlink file in export: {relative}")
            require(stat.S_ISREG(mode), f"non-regular file in export: {relative}")
            folded = relative.casefold()
            require(folded not in lower_paths, f"case-alias paths in export: {lower_paths.get(folded)} and {relative}")
            lower_paths[folded] = relative
            records.append({
                "path": relative,
                "sha256": sha256_file(path),
                "mode": stat.S_IMODE(mode),
            })
    records.sort(key=lambda record: record["path"])
    return records


def sample_inventory(expected_root: Path) -> tuple[list[dict[str, str]], str]:
    require(expected_root.is_dir() and not expected_root.is_symlink(), f"expected store missing or unsafe: {expected_root}")
    records: list[dict[str, str]] = []
    lower_paths: dict[str, str] = {}
    actual_files: set[str] = set()
    actual_directories: set[str] = set()

    def register_path(relative: str) -> None:
        folded = relative.casefold()
        require(folded not in lower_paths,
                f"case-alias paths in expected store: {lower_paths.get(folded)} and {relative}")
        lower_paths[folded] = relative

    for current, dirnames, filenames in os.walk(expected_root, topdown=True, followlinks=False):
        directory = Path(current)
        for name in list(dirnames):
            path = directory / name
            require(not path.is_symlink(), f"symlink directory in expected store: {path.relative_to(expected_root)}")
            relative = path.relative_to(expected_root).as_posix()
            register_path(relative)
            actual_directories.add(relative)
        for name in filenames:
            path = directory / name
            relative = path.relative_to(expected_root).as_posix()
            require(not path.is_symlink(), f"symlink file in expected store: {relative}")
            mode = path.lstat().st_mode
            require(stat.S_ISREG(mode), f"non-regular expected-store file: {relative}")
            register_path(relative)
            actual_files.add(relative)
            if relative.endswith(".tui.json"):
                records.append({"path": relative, "sha256": sha256_file(path)})

    sample_paths = {row["path"] for row in records}
    expected_files = sample_paths | STATIC_EXPECTED_FILES
    if actual_files != expected_files:
        missing = sorted(expected_files - actual_files)
        extra = sorted(actual_files - expected_files)
        raise LedgerError(f"expected-store file inventory differs; missing={missing}; extra={extra}")
    expected_directories: set[str] = set()
    for relative in expected_files:
        parent = Path(relative).parent
        while parent != Path("."):
            expected_directories.add(parent.as_posix())
            parent = parent.parent
    if actual_directories != expected_directories:
        missing = sorted(expected_directories - actual_directories)
        extra = sorted(actual_directories - expected_directories)
        raise LedgerError(f"expected-store directory inventory differs; missing={missing}; extra={extra}")

    records.sort(key=lambda record: record["path"])
    manifest = "".join(f"{row['sha256']}  {row['path']}\n" for row in records).encode()
    return records, sha256_bytes(manifest)


def verify_approval(export_root: Path) -> tuple[list[dict[str, str]], str, str]:
    approval_path = export_root / "expected" / "BASELINE-APPROVAL.json"
    require(approval_path.is_file() and not approval_path.is_symlink(), "expected approval record missing or unsafe")
    approval_sha = sha256_file(approval_path)
    require(approval_sha == EXPECTED_APPROVAL_SHA256,
            f"expected approval record SHA mismatch: {approval_sha}")
    approval = read_json(approval_path, "expected approval record")
    current = approval.get("current_expected_store_state")
    require(isinstance(current, dict), "approval has no current expected-store state")
    expected_count = current.get("accepted_reference_sample_count")
    expected_digest = current.get("sample_inventory_manifest_sha256")
    require(expected_count == EXPECTED_SAMPLE_COUNT, f"approved expected sample count changed: {expected_count!r}")
    require(expected_digest == EXPECTED_SAMPLE_MANIFEST_SHA256,
            f"approval sample inventory digest changed: {expected_digest!r}")
    samples, digest = sample_inventory(export_root / "expected")
    require(len(samples) == EXPECTED_SAMPLE_COUNT,
            f"expected store sample count mismatch: got {len(samples)}, expected {EXPECTED_SAMPLE_COUNT}")
    require(digest == EXPECTED_SAMPLE_MANIFEST_SHA256,
            f"expected store inventory digest mismatch: got {digest}")
    require(not (export_root / "expected" / ".tuiscotti-approval.lock").exists(),
            "runtime approval lock must not be in the source export")
    return samples, digest, approval_sha


def git_output(argv: list[str], cwd: Path | None = None) -> str:
    result = subprocess.run(argv, cwd=cwd, check=False, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode:
        raise LedgerError(f"command failed ({result.returncode}): {' '.join(argv)}: {result.stderr.strip()}")
    return result.stdout.strip()


def verify_prepared_engines(export_root: Path) -> None:
    pins_path = export_root / "engine-pins.json"
    require(pins_path.is_file() and not pins_path.is_symlink(), "engine pin file is missing or unsafe")
    require(sha256_file(pins_path) == ACCEPTED_ENGINE_PINS_FILE_SHA256,
            "engine pin file is not the accepted immutable pin record")
    pins = read_json(pins_path, "engine pins")
    require(isinstance(pins, dict) and set(pins) == {
        "schema_version", "ci_integration", "tuiscotti", "tuiscotti_tui_test",
    }, "engine pin file has an unsupported root shape")
    require(pins.get("schema_version") == 2, "engine pin schema version is unsupported")
    require(pins.get("tuiscotti_tui_test", {}).get("base_commit") == ENGINE_PINS["tuiscotti-tui-test"],
            "engine pin file has wrong tui-test base commit")
    require(pins.get("tuiscotti", {}).get("commit") == ENGINE_PINS["tuiscotti"],
            "engine pin file has wrong tuiscotti commit")
    tui_test_pin = pins["tuiscotti_tui_test"]
    tuiscotti_pin = pins["tuiscotti"]
    tui_test_sources = tui_test_pin.get("source_sha256")
    tuiscotti_sources = tuiscotti_pin.get("source_sha256")
    require(isinstance(tui_test_sources, dict) and set(tui_test_sources) == set(TUI_TEST_FILES),
            "engine pin tui-test patched-file inventory differs from the accepted path set")
    require(isinstance(tuiscotti_sources, dict) and set(tuiscotti_sources) == set(TUISCOTTI_FILES),
            "engine pin tuiscotti patched-file inventory differs from the accepted path set")
    expected = [
        (export_root / "forks" / "tui-test-upstream", tui_test_pin["base_commit"], tui_test_sources),
        (export_root / "forks" / "tuiscotti-upstream", tuiscotti_pin["commit"], tuiscotti_sources),
    ]
    for repository, commit, sources in expected:
        require(repository.is_dir() and not repository.is_symlink(), f"engine checkout is missing or unsafe: {repository}")
        head = git_output(["git", "-C", str(repository), "rev-parse", "HEAD"])
        require(head == commit, f"engine checkout HEAD differs: {repository.name} is {head}, expected {commit}")
        require(not git_output(["git", "-C", str(repository), "diff", "--cached", "--name-only", commit]),
                f"engine checkout has staged changes: {repository.name}")
        status = git_output(["git", "-C", str(repository), "status", "--porcelain", "--untracked-files=all"])
        status_lines = status.splitlines() if status else []
        require(all(line.startswith(" M ") for line in status_lines),
                f"engine checkout has unexpected worktree state: {repository.name}")
        changed = git_output(["git", "-C", str(repository), "diff", "--name-only", commit])
        require(set(changed.splitlines()) == set(sources), f"engine patch path set differs: {repository.name}")
        for relative, wanted_hash in sources.items():
            source = repository / safe_relpath(relative, "engine source path")
            require(source.is_file() and not source.is_symlink(), f"engine source missing or unsafe: {relative}")
            actual_hash = sha256_file(source)
            require(actual_hash == wanted_hash,
                    f"engine source SHA mismatch {repository.name}:{relative}: {actual_hash}")
    patch_specs = (
        (pins["tuiscotti_tui_test"]["patch_file"], pins["tuiscotti_tui_test"]["patch_sha256"],
         TUI_TEST_PATCH_PATH, ACCEPTED_TUI_TEST_PATCH_SHA256),
        (pins["tuiscotti"]["pixel_gap_patch_file"], pins["tuiscotti"]["pixel_gap_patch_sha256"],
         TUISCOTTI_PATCH_PATH, ACCEPTED_TUISCOTTI_PATCH_SHA256),
    )
    for relative, recorded_hash, accepted_path, accepted_hash in patch_specs:
        require(relative == accepted_path, f"engine pin references an unexpected retained patch: {relative}")
        require(recorded_hash == accepted_hash, f"engine pin patch digest differs from accepted patch: {relative}")
        patch = export_root / safe_relpath(relative, "engine patch path")
        require(patch.is_file() and not patch.is_symlink(), f"retained engine patch missing or unsafe: {relative}")
        require(sha256_file(patch) == accepted_hash, f"retained engine patch SHA mismatch: {relative}")


def write_export_manifest(args: argparse.Namespace) -> None:
    root = Path(args.export_dir).resolve(strict=True)
    require(not (root / ".termrock-export-manifest.json").exists(),
            "export inventory already exists; refusing to reseal an existing export")
    load_inventory(root)
    records = enumerate_tree(root)
    sample_records, expected_digest, approval_sha = verify_approval(root)
    verify_prepared_engines(root)
    source_identity = suite_source_identity(root)
    lines = "".join(f"{record['sha256']}  {record['path']}\n" for record in records).encode()
    manifest = {
        "schema_version": 1,
        "created_at": utc_now(),
        "source_file_count": len(records),
        "source_manifest_sha256": sha256_bytes(lines),
        "files_manifest_sha256": sha256_bytes(
            json.dumps(records, separators=(",", ":"), sort_keys=True).encode()
        ),
        "files": records,
        "expected_sample_count": len(sample_records),
        "expected_sample_inventory_sha256": expected_digest,
        "expected_samples": sample_records,
        "approval_record_sha256": approval_sha,
        "engine_pins": ENGINE_PINS,
        "suite_source_identity": source_identity,
    }
    atomic_json(root / ".termrock-export-manifest.json", manifest)
    print(f"source files={len(records)} manifest_sha256={manifest['source_manifest_sha256']}")
    print(f"expected samples={len(sample_records)} digest={expected_digest}")


def verify_export(export_root: Path) -> dict[str, Any]:
    root = export_root.resolve(strict=True)
    require(root.is_dir() and not root.is_symlink(), f"export root is missing or unsafe: {root}")
    load_inventory(root)
    source_identity = suite_source_identity(root)
    receipt_path = root / ".termrock-export-manifest.json"
    receipt = read_json(receipt_path, "export inventory receipt")
    require(receipt.get("schema_version") == 1, "export inventory schema version is unsupported")
    expected_files = receipt.get("files")
    require(isinstance(expected_files, list), "export inventory receipt has no files array")
    actual_files = enumerate_tree(root)
    expected_paths = [row.get("path") for row in expected_files if isinstance(row, dict)]
    actual_paths = [row["path"] for row in actual_files]
    require(len(expected_paths) == len(expected_files), "export inventory has malformed file rows")
    if set(expected_paths) != set(actual_paths):
        missing = sorted(set(expected_paths) - set(actual_paths))
        extra = sorted(set(actual_paths) - set(expected_paths))
        raise LedgerError(f"export file inventory differs; missing={missing}; extra={extra}")
    expected_map = {row["path"]: row for row in expected_files}
    actual_map = {row["path"]: row for row in actual_files}
    for path, expected in expected_map.items():
        actual = actual_map[path]
        require(expected.get("sha256") == actual["sha256"], f"export file digest mismatch: {path}")
        require(expected.get("mode") == actual["mode"], f"export file mode mismatch: {path}")
    lines = "".join(f"{record['sha256']}  {record['path']}\n" for record in actual_files).encode()
    actual_source_digest = sha256_bytes(lines)
    require(actual_source_digest == receipt.get("source_manifest_sha256"),
            f"export source manifest SHA mismatch: {actual_source_digest}")
    files_hash = sha256_bytes(json.dumps(actual_files, separators=(",", ":"), sort_keys=True).encode())
    require(files_hash == receipt.get("files_manifest_sha256"), f"export file receipt digest mismatch: {files_hash}")
    sample_records, expected_digest, approval_sha = verify_approval(root)
    require(receipt.get("expected_samples") == sample_records,
            "expected sample path/hash inventory differs from the recorded export receipt")
    require(receipt.get("expected_sample_count") == EXPECTED_SAMPLE_COUNT,
            "expected sample count in export receipt differs from the approved inventory")
    require(receipt.get("expected_sample_inventory_sha256") == expected_digest,
            "expected sample digest in export receipt differs from the approved inventory")
    require(receipt.get("approval_record_sha256") == approval_sha,
            "approval record SHA in export receipt differs from the frozen approval")
    require(receipt.get("engine_pins") == ENGINE_PINS, "engine pins in export receipt differ")
    require(receipt.get("suite_source_identity") == source_identity,
            "accepted Suite source identities differ from the immutable export receipt")
    verify_prepared_engines(root)
    return {
        "root": str(root),
        "source_file_count": len(actual_files),
        "source_manifest_sha256": actual_source_digest,
        "expected_sample_count": len(sample_records),
        "expected_sample_inventory_sha256": expected_digest,
        "approval_record_sha256": approval_sha,
        "engine_pins": ENGINE_PINS,
        "suite_source_identity": source_identity,
    }


def verify_export_command(args: argparse.Namespace) -> None:
    try:
        summary = verify_export(Path(args.export_dir))
    except LedgerError as error:
        fail(str(error))
    print(json.dumps(summary, indent=2, sort_keys=True))


def write_junit_config(args: argparse.Namespace) -> None:
    base = Path(args.base_config).resolve(strict=True)
    output = Path(args.output_config)
    require(safe_run_id(args.run_id), "JUnit run ID is invalid")
    require(sha256_file(base) == ACCEPTED_NEXTEST_CONFIG_SHA256,
            "base Nextest config is not the accepted CI profile")
    require(not output.exists() and not output.is_symlink(),
            f"refusing to overwrite an existing JUnit config: {output}")
    source = base.read_text(encoding="utf-8")
    require(source.count('path = "junit.xml"') == 1,
            "base Nextest config must contain exactly one standard junit.xml path")
    destination = f'path = "junit-{args.run_id}.xml"'
    atomic_write(output, source.replace('path = "junit.xml"', destination).encode(), mode=0o600)
    print(output)


def parse_filter_match(value: Any, label: str) -> tuple[bool | None, str | None]:
    require(isinstance(value, dict), f"{label} filter-match is not an object")
    status = value.get("status")
    if status == "matches":
        require(set(value) == {"status"}, f"{label} matches status has unknown fields")
        return True, None
    if status == "mismatch":
        require(set(value) == {"status", "reason"}, f"{label} mismatch status has unknown fields")
        reason = value.get("reason")
        require(isinstance(reason, str) and reason in LIST_MISMATCH_REASONS,
                f"{label} has an unknown filter mismatch reason: {reason!r}")
        return False, reason
    raise LedgerError(f"{label} has an unknown filter-match status: {status!r}")


def find_command_record(invocation: dict[str, Any], run_root: Path, name: str,
                        participant: str) -> dict[str, Any]:
    commands = invocation.get("commands")
    require(isinstance(commands, list), "invocation command ledger is missing")
    matches = [item for item in commands if isinstance(item, dict)
               and item.get("name") == name and item.get("participant") == participant]
    require(len(matches) == 1,
            f"invocation must contain exactly one {participant}/{name} command record")
    command = matches[0]
    require(isinstance(command.get("argv"), list)
            and all(isinstance(value, str) for value in command["argv"]),
            f"invocation {participant}/{name} argv is malformed")
    command_root = (run_root / "global" if participant == "global"
                    else participant_dir(run_root, participant))
    argv_path = command_root / "commands" / f"{name}.argv.json"
    require(read_json(argv_path, f"{participant}/{name} argv receipt") == command["argv"],
            f"invocation {participant}/{name} argv differs from its copied command-input receipt")
    if participant in SIDE_NAMES:
        side_record = read_json(command_root / "commands" / f"{name}.json",
                                f"{participant}/{name} command receipt")
        require(side_record == command,
                f"invocation {participant}/{name} differs from its copied command receipt")
    return command


def verify_command_logs(run_root: Path, command: dict[str, Any], label: str) -> dict[str, Path]:
    logs = command.get("logs")
    require(isinstance(logs, dict) and set(logs) == {"stdout", "stderr"},
            f"{label} stdout/stderr log receipt is missing or malformed")
    verified: dict[str, Path] = {}
    for stream in ("stdout", "stderr"):
        record = logs.get(stream)
        require(isinstance(record, dict) and set(record) == {"path", "sha256", "size"},
                f"{label} {stream} log receipt has an unsupported shape")
        relative = safe_relpath(record.get("path"), f"{label} {stream} log path")
        path = run_root / relative
        check_no_symlink_components(path)
        require(path.is_file() and not path.is_symlink(),
                f"{label} {stream} log is missing or unsafe")
        validate_sha256_value(record.get("sha256"), f"{label} {stream} log SHA-256")
        require(type(record.get("size")) is int and record["size"] >= 0,
                f"{label} {stream} log size is invalid")
        require(sha256_file(path) == record["sha256"] and path.stat().st_size == record["size"],
                f"{label} {stream} log differs from its command receipt")
        verified[stream] = path
    return verified


EXECUTION_CONTEXT_KEYS = {
    "guard_path",
    "cargo_home",
    "github_actions",
    "github_workspace",
    "github_run_id",
    "github_run_attempt",
    "runner_os",
    "runner_temp",
    "home",
    "cargo_build_jobs",
    "nextest_test_threads",
}


def canonical_context_path(raw: Any, label: str, *, strict: bool) -> Path:
    require(isinstance(raw, str) and Path(raw).is_absolute(),
            f"execution context {label} must be an absolute path")
    try:
        resolved = Path(raw).resolve(strict=strict)
    except (OSError, RuntimeError) as error:
        raise LedgerError(f"execution context {label} cannot be resolved") from error
    require(str(resolved) == raw, f"execution context {label} is not canonical")
    return resolved


def validate_execution_context(invocation: dict[str, Any], run_root: Path | None = None,
                               *, require_actual_execution_root: bool = False) -> dict[str, Any]:
    mode = invocation.get("execution_mode")
    require(mode in {"local-guarded", "velnor-native"},
            "execution mode is missing or unsupported")
    context = invocation.get("execution_context")
    require(isinstance(context, dict) and set(context) == EXECUTION_CONTEXT_KEYS,
            "execution context has an unsupported shape")
    cargo_home = canonical_context_path(context.get("cargo_home"), "Cargo home", strict=False)
    suite_root_raw = invocation.get("suite_root")
    require(isinstance(suite_root_raw, str), "invocation Suite root is missing")
    suite_root = Path(suite_root_raw).resolve(strict=True)
    app_root = suite_root.parent.parent

    if mode == "local-guarded":
        guard = canonical_context_path(context.get("guard_path"), "rust-build-guard", strict=True)
        require(guard.name == "rust-build-guard" and guard.is_file() and os.access(guard, os.X_OK),
                "local execution requires an executable rust-build-guard")
        for key in EXECUTION_CONTEXT_KEYS - {"guard_path", "cargo_home"}:
            value = context.get(key)
            require(value is None or isinstance(value, str),
                    f"local execution context {key} must be a string or null")
        return {"mode": mode, "cargo_home": cargo_home, "guard_path": guard}

    require(context.get("guard_path") is None,
            "native execution must use the explicit unguarded native route")
    require(context.get("github_actions") == "true",
            "native execution requires the GitHub Actions runtime identity")
    require(context.get("runner_os") == "macOS",
            "native execution requires the macOS runner contract")
    run_id = context.get("github_run_id")
    attempt = context.get("github_run_attempt")
    require(isinstance(run_id, str) and re.fullmatch(r"[0-9]+", run_id) is not None
            and int(run_id) > 0, "native execution GitHub run ID is invalid")
    require(isinstance(attempt, str) and re.fullmatch(r"[0-9]+", attempt) is not None
            and int(attempt) > 0, "native execution GitHub run attempt is invalid")
    runner_temp = canonical_context_path(context.get("runner_temp"), "runner temporary directory", strict=False)
    workspace = canonical_context_path(context.get("github_workspace"), "GitHub workspace", strict=False)
    home = canonical_context_path(context.get("home"), "temporary home", strict=False)
    require(workspace == app_root,
            "native execution GitHub workspace differs from the repository root")
    task_root = runner_temp / f"velnor-task-{run_id}-{attempt}-functional-suite"
    require(home == task_root / "home",
            "native execution HOME is not the isolated Velnor task home")
    expected_cargo_home = task_root / "cargo"
    require(cargo_home == expected_cargo_home,
            "native execution Cargo home is not the isolated Velnor task Cargo home")
    require(context.get("cargo_build_jobs") == "1",
            "native execution requires CARGO_BUILD_JOBS=1")
    require(context.get("nextest_test_threads") == "1",
            "native execution requires NEXTEST_TEST_THREADS=1")
    execution_root = canonical_context_path(invocation.get("execution_root"),
                                            "execution evidence directory", strict=False)
    invocation_run_id = invocation.get("run_id")
    require(isinstance(invocation_run_id, str) and safe_run_id(invocation_run_id),
            "native execution invocation run ID is invalid")
    expected_run_root = runner_temp / "termrock-functional-suite-ci" / run_id / invocation_run_id
    require(execution_root == expected_run_root,
            "native execution evidence directory does not match its GitHub run identity")
    if require_actual_execution_root:
        require(run_root is not None,
                "native execution context check requires its live run directory")
        require(runner_temp.is_dir() and workspace.is_dir() and home.is_dir() and cargo_home.is_dir(),
                "native execution temporary directories are missing")
        try:
            actual_run_root = run_root.resolve(strict=True)
        except (OSError, RuntimeError) as error:
            raise LedgerError("native execution run directory cannot be resolved") from error
        require(actual_run_root == execution_root,
                "native execution context was checked from a copied artifact directory")
    return {"mode": mode, "cargo_home": cargo_home, "guard_path": None,
            "app_root": app_root, "runner_temp": runner_temp}


def validate_execution_context_command(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    invocation, _ = load_run(run_root)
    context = validate_execution_context(invocation, run_root, require_actual_execution_root=True)
    print(json.dumps({"execution_mode": context["mode"],
                      "cargo_home": str(context["cargo_home"])}, sort_keys=True))


def validate_cargo_route_argv(argv: list[str], invocation: dict[str, Any],
                              expected_env: list[str], cargo_arguments: list[str],
                              label: str) -> None:
    context = validate_execution_context(invocation)
    app_root = Path(invocation["suite_root"]).resolve(strict=True).parent.parent
    env_prefix = [
        "env",
        "MISE_DISABLE_TOOLS=java",
        f"MISE_CONFIG_FILE={app_root / 'mise.toml'}",
        f"CARGO_HOME={context['cargo_home']}",
    ] + expected_env
    require(argv[:len(env_prefix)] == env_prefix,
            f"{label} environment differs from the frozen execution contract")
    route = ([str(context["guard_path"]), "--", "mise", "exec", "--", "cargo"]
             if context["mode"] == "local-guarded"
             else ["mise", "exec", "--", "cargo"])
    start = len(env_prefix)
    require(argv[start:start + len(route)] == route,
            f"{label} does not use the required {context['mode']} MBX/Cargo route")
    require(argv[start + len(route):] == cargo_arguments,
            f"{label} Cargo arguments differ from the frozen contract")


def validate_nextest_argv(argv: list[str], invocation: dict[str, Any], run_root: Path,
                          export_root: Path, target_root: Path,
                          participant: str, *, listing: bool,
                          side_root: Path | None = None) -> None:
    run_id = invocation.get("run_id")
    require(isinstance(run_id, str) and safe_run_id(run_id), "invocation run ID is invalid")
    suite_root_raw = invocation.get("suite_root")
    require(isinstance(suite_root_raw, str), "invocation Suite root is missing")
    suite_root = Path(suite_root_raw).resolve(strict=True)
    execution_root_raw = invocation.get("execution_root")
    require(isinstance(execution_root_raw, str), "invocation execution root is missing")
    execution_root = Path(execution_root_raw).resolve(strict=False)
    app_root = suite_root.parent.parent
    run_suffix = "pr26" if listing else participant
    participant_root = execution_root / "participants" / "pr26"
    if not listing:
        require(participant in SIDE_NAMES and side_root is not None,
                "Nextest RUN participant identity is invalid")
        participant_root = side_root
    bin_root = participant_root / "bin"
    out_root = participant_root / "out"
    validate_execution_context(invocation, run_root)
    if listing:
        expected_options = [
            "nextest", "list", "--test", BINARY_NAME,
            "--locked", "--offline",
            "--profile", NEXTEST_PROFILE,
            "--cargo-profile", CARGO_PROFILE,
            "--config-file", str(export_root / ".config" / "nextest.toml"),
            "--user-config-file", "none",
            "--build-jobs", "1",
            "--ignore-default-filter",
            "--message-format", "json",
        ]
    else:
        require(side_root is not None, "Nextest RUN side root is missing")
        expected_options = [
            "nextest", "run", "--test", BINARY_NAME,
            "--locked", "--offline",
            "--profile", NEXTEST_PROFILE,
            "--cargo-profile", CARGO_PROFILE,
            "--config-file", str(side_root / "nextest.toml"),
            "--user-config-file", "none",
            "--build-jobs", "1",
            "--test-threads", "1",
            "--retries", "0",
            "--no-fail-fast",
            "--no-tests", "fail",
            "--ignore-default-filter",
        ]
    diagnostic_filter = invocation.get("diagnostic_filter")
    require(diagnostic_filter is None or isinstance(diagnostic_filter, str),
            "invocation diagnostic filter has the wrong type")
    if diagnostic_filter:
        expected_options.extend(["--filter-expr", diagnostic_filter])
    validate_cargo_route_argv(argv, invocation,
                              [f"CARGO_TARGET_DIR={target_root.resolve(strict=False)}",
                               "CARGO_BUILD_JOBS=1",
                               f"TERMROCK_PARITY_BIN_DIR={bin_root}",
                               f"TERMROCK_PARITY_OUT_DIR={out_root}",
                               f"TERMROCK_PARITY_RUN_ID={run_id}-{run_suffix}"],
                              expected_options,
                              f"Nextest {'LIST' if listing else 'RUN'} command")


def validate_dependency_fetches(run_root: Path, invocation: dict[str, Any]) -> None:
    context = validate_execution_context(invocation, run_root)
    app_root = Path(invocation["suite_root"]).resolve(strict=True).parent.parent
    export = invocation.get("export_identity")
    sources = invocation.get("source_identity")
    require(isinstance(export, dict) and isinstance(export.get("root"), str),
            "prepared Suite export identity is missing for Cargo dependency validation")
    require(isinstance(sources, dict)
            and isinstance(sources.get("candidate"), dict)
            and isinstance(sources.get("pr26"), dict),
            "candidate/PR26 source identities are missing for Cargo dependency validation")
    candidate = sources["candidate"]
    pr26 = sources["pr26"]
    require(isinstance(candidate.get("root"), str) and Path(candidate["root"]).is_absolute(),
            "candidate workspace path is missing or invalid")
    require(isinstance(pr26.get("root"), str) and Path(pr26["root"]).is_absolute(),
            "PR26 workspace path is missing or invalid")
    require(isinstance(export.get("root"), str) and Path(export["root"]).is_absolute(),
            "prepared Suite export path is invalid")
    require(candidate.get("available") is True
            and candidate.get("root") == str(app_root)
            and Path(candidate["root"]).resolve(strict=True) == app_root,
            "candidate workspace identity is unavailable or differs from the repository root")
    require(pr26.get("available") is True
            and pr26.get("root") == str(Path(pr26["root"]).resolve(strict=True))
            and pr26.get("git_head") == EXPECTED_PR26_SHA
            and pr26.get("expected_sha") == EXPECTED_PR26_SHA,
            "PR26 workspace identity is unavailable or differs from its pinned revision")
    workspaces = (
        ("cargo-fetch-suite", Path(export["root"])),
        ("cargo-fetch-pr26", Path(pr26["root"])),
        ("cargo-fetch-candidate", Path(candidate["root"])),
    )
    for name, workspace in workspaces:
        command = find_command_record(invocation, run_root, name, "global")
        try:
            command_cwd = Path(command.get("cwd", "")).resolve(strict=True)
            expected_cwd = workspace.resolve(strict=True)
        except (OSError, RuntimeError) as error:
            raise LedgerError(f"{name} workspace is missing or cannot be resolved") from error
        require(command_cwd == expected_cwd,
                f"{name} command cwd differs from its recorded workspace identity")
        require(type(command.get("exit_status")) is int and command["exit_status"] == 0
                and command.get("signal") is None,
                f"{name} did not complete successfully")
        verify_command_logs(run_root, command, name)
        validate_cargo_route_argv(
            command["argv"], invocation, ["CARGO_BUILD_JOBS=1"], ["fetch", "--locked"], name
        )
    require(invocation.get("dependency_sources_ready") is True,
            "runner did not record successful locked dependency preparation")
    require(invocation.get("dependency_sources_error") is None,
            "runner recorded a Cargo dependency preparation error")


def validate_app_build_command(run_root: Path, invocation: dict[str, Any], side: str,
                               state: dict[str, Any]) -> None:
    require(side in SIDE_NAMES, "application build participant is invalid")
    target_raw = state.get("build_target_root")
    require(isinstance(target_raw, str) and Path(target_raw).is_absolute(),
            f"{side} application build target directory is missing or invalid")
    target_root = Path(target_raw).resolve(strict=False)
    require(str(target_root) == target_raw,
            f"{side} application build target directory is not canonical")
    source_identity = invocation.get("source_identity")
    require(isinstance(source_identity, dict)
            and isinstance(source_identity.get(side), dict)
            and isinstance(source_identity[side].get("root"), str),
            f"{side} application source identity is missing")
    source_root = Path(source_identity[side]["root"]).resolve(strict=True)
    command = find_command_record(invocation, run_root, "app-build", side)
    try:
        command_cwd = Path(command.get("cwd", "")).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise LedgerError(f"{side} application build cwd cannot be resolved") from error
    require(command_cwd == source_root,
            f"{side} application build cwd differs from its source identity")
    state_exit = state.get("build_exit")
    require(type(state_exit) is int and command.get("exit_status") == state_exit
            and command.get("signal") is None,
            f"{side} application build command result differs from participant state")
    verify_command_logs(run_root, command, f"{side} application build")
    validate_cargo_route_argv(
        command["argv"], invocation,
        [f"CARGO_TARGET_DIR={target_root}", "CARGO_BUILD_JOBS=1"],
        ["build", "--locked", "--offline", "--jobs", "1",
         "-p", "showcase", "-p", "holla", "-p", "jackin-preview", "-p", "tablepro", "--bins"],
        f"{side} application build",
    )


def validate_list_command(invocation: dict[str, Any], run_root: Path, export_root: Path,
                          target_root: Path) -> dict[str, Any]:
    command = find_command_record(invocation, run_root, "nextest-list", "global")
    try:
        command_cwd = Path(command.get("cwd", "")).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise LedgerError("Nextest LIST command cwd is missing or cannot be resolved") from error
    require(command_cwd == export_root, "Nextest LIST command cwd differs from the source export")
    require(type(command.get("exit_status")) is int and command["exit_status"] == 0
            and command.get("signal") is None,
            "Nextest LIST command did not complete successfully")
    command_logs = verify_command_logs(run_root, command, "Nextest LIST")
    require(command_logs["stdout"].relative_to(run_root).as_posix()
            == "global/logs/nextest-list.stdout.log",
            "Nextest LIST stdout path differs from the runner's captured stream")
    validate_nextest_argv(command["argv"], invocation, run_root, export_root, target_root,
                          "global", listing=True)
    return command


def validate_run_command(run_root: Path, invocation: dict[str, Any], side: str,
                         state: dict[str, Any]) -> tuple[str | None, dict[str, Any] | None]:
    if state.get("run_launched") is not True:
        return None, None
    try:
        export_root_raw = invocation.get("list_export_root")
        target_root_raw = invocation.get("list_target_root")
        require(isinstance(export_root_raw, str) and isinstance(target_root_raw, str),
                "Nextest RUN export/target identity is missing")
        export_root = Path(export_root_raw).resolve(strict=True)
        target_root = Path(target_root_raw).resolve(strict=False)
        side_root = participant_dir(run_root, side)
        execution_root_raw = invocation.get("execution_root")
        require(isinstance(execution_root_raw, str), "invocation execution root is missing")
        original_side_root = Path(execution_root_raw).resolve(strict=False) / "participants" / side
        command = find_command_record(invocation, run_root, "nextest-run", side)
        try:
            command_cwd = Path(command.get("cwd", "")).resolve(strict=True)
        except (OSError, RuntimeError) as error:
            raise LedgerError(f"{side} Nextest RUN cwd is missing or cannot be resolved") from error
        require(command_cwd == export_root,
                f"{side} Nextest RUN cwd differs from the source export")
        require(command.get("exit_status") == state.get("run_exit")
                and command.get("signal") == state.get("run_signal"),
                f"{side} Nextest RUN command result differs from participant state")
        verify_command_logs(run_root, command, f"{side} Nextest RUN")
        validate_nextest_argv(command["argv"], invocation, run_root,
                              export_root, target_root, side,
                              listing=False, side_root=original_side_root)

        side_run_id = f"{invocation['run_id']}-{side}"
        config_command = find_command_record(invocation, run_root, "nextest-config", side)
        expected_config_argv = [
            "python3",
            str(Path(invocation["suite_root"]).resolve(strict=True) / "scripts" / "ci-ledger.py"),
            "write-junit-config",
            "--base-config", str(export_root / ".config" / "nextest.toml"),
            "--output-config", str(original_side_root / "nextest.toml"),
            "--run-id", side_run_id,
        ]
        try:
            config_cwd = Path(config_command.get("cwd", "")).resolve(strict=True)
        except (OSError, RuntimeError) as error:
            raise LedgerError(f"{side} Nextest config command cwd is missing or invalid") from error
        require(config_cwd == export_root
                and config_command.get("exit_status") == 0
                and config_command.get("signal") is None
                and config_command.get("argv") == expected_config_argv,
                f"{side} unique JUnit configuration command is missing or invalid")
        base_config = export_root / ".config" / "nextest.toml"
        output_config = side_root / "nextest.toml"
        require(base_config.is_file() and not base_config.is_symlink()
                and sha256_file(base_config) == ACCEPTED_NEXTEST_CONFIG_SHA256,
                "accepted Nextest base config is missing or changed")
        source_text = base_config.read_text(encoding="utf-8")
        require(source_text.count('path = "junit.xml"') == 1,
                "accepted Nextest base config does not have one standard JUnit path")
        expected_config = source_text.replace(
            'path = "junit.xml"', f'path = "junit-{side_run_id}.xml"'
        ).encode()
        require(output_config.is_file() and not output_config.is_symlink()
                and output_config.read_bytes() == expected_config,
                f"{side} JUnit config differs from the unique accepted profile")

        junit_source = target_root / "nextest" / NEXTEST_PROFILE / f"junit-{side_run_id}.xml"
        junit_source_copy = side_root / "junit-source-original.xml"
        junit_copy = side_root / "junit.xml"
        receipt_path = side_root / "junit-source.json"
        require(junit_source_copy.is_file() and not junit_source_copy.is_symlink()
                and junit_copy.is_file() and not junit_copy.is_symlink()
                and receipt_path.is_file() and not receipt_path.is_symlink(),
                f"{side} JUnit source/copy receipt is missing or unsafe")
        junit_receipt = read_json(receipt_path, f"{side} JUnit source receipt")
        require(isinstance(junit_receipt, dict) and set(junit_receipt) == {"schema_version", "files"}
                and type(junit_receipt.get("schema_version")) is int
                and junit_receipt["schema_version"] == 1
                and isinstance(junit_receipt.get("files"), list)
                and len(junit_receipt["files"]) == 3,
                f"{side} JUnit source receipt has an unsupported schema")
        source_sha = sha256_file(junit_source_copy)
        source_size = junit_source_copy.stat().st_size
        expected_files = [
            {"path": str(junit_source), "sha256": source_sha, "size": source_size},
            {"path": str(original_side_root / "junit-source-original.xml"),
             "sha256": source_sha, "size": source_size},
            {"path": str(original_side_root / "junit.xml"), "sha256": sha256_file(junit_copy), "size": junit_copy.stat().st_size},
        ]
        require(junit_receipt["files"] == expected_files,
                f"{side} JUnit source receipt does not bind the unique report and its copy")
        require(expected_files[0]["sha256"] == expected_files[2]["sha256"],
                f"{side} JUnit copied report differs from its archived Nextest output")
        if junit_source.exists() or junit_source.is_symlink():
            require(junit_source.is_file() and not junit_source.is_symlink()
                    and sha256_file(junit_source) == source_sha
                    and junit_source.stat().st_size == source_size,
                    f"{side} live Nextest JUnit report differs from its archived copy")
        return None, command
    except (LedgerError, OSError, UnicodeError, KeyError) as error:
        return str(error), None


def parse_nextest_list(
    path: Path,
    cases: list[dict[str, Any]],
    expected_export_root: Path | None,
    expected_target_root: Path | None,
    *,
    binary_evidence: Path | None = None,
    expected_binary_sha256: str | None = None,
) -> dict[str, Any]:
    require(expected_export_root is not None and expected_target_root is not None,
            "Nextest LIST export and target identities are missing")
    export_root = expected_export_root.resolve(strict=True)
    target_root = expected_target_root.resolve(strict=False)
    data = read_json(path, "Nextest LIST output")
    require(isinstance(data, dict), "Nextest LIST root is not an object")
    require(set(data) == LIST_TOP_LEVEL_KEYS, f"Nextest LIST has unexpected top-level keys: {sorted(set(data) ^ LIST_TOP_LEVEL_KEYS)}")
    build_meta = data.get("rust-build-meta")
    require(isinstance(build_meta, dict) and set(build_meta) == LIST_BUILD_META_KEYS,
            "Nextest rust-build-meta has an unsupported shape")
    require(build_meta.get("target-directory") == str(target_root),
            "Nextest rust-build-meta target directory does not match CARGO_TARGET_DIR")
    require(build_meta.get("build-directory") == str(target_root),
            "Nextest rust-build-meta build directory does not match CARGO_TARGET_DIR")
    require(build_meta.get("base-output-directories") == ["debug"],
            "Nextest LIST was not produced from the expected dev/Cargo debug profile")
    for key in ("non-test-binaries", "build-script-out-dirs", "build-script-info"):
        require(isinstance(build_meta.get(key), dict), f"Nextest rust-build-meta {key} is not an object")
    require(isinstance(build_meta.get("linked-paths"), list),
            "Nextest rust-build-meta linked-paths is not an array")
    platforms = build_meta.get("platforms")
    require(isinstance(platforms, dict) and set(platforms) == LIST_PLATFORMS_KEYS,
            "Nextest rust-build-meta platform inventory is invalid")
    host = platforms.get("host")
    require(isinstance(host, dict) and set(host) == {"platform", "libdir"},
            "Nextest host platform entry is invalid")
    host_platform = host.get("platform")
    require(isinstance(host_platform, dict) and set(host_platform) == LIST_HOST_PLATFORM_KEYS,
            "Nextest host platform descriptor is invalid")
    host_triple = host_platform.get("triple")
    host_features = host_platform.get("target-features")
    require(isinstance(host_triple, str) and host_triple,
            "Nextest host platform triple is missing")
    require(host_triple == EXPECTED_TARGET_TRIPLE,
            f"Nextest host platform is not {EXPECTED_TARGET_TRIPLE}: {host_triple!r}")
    require(isinstance(host_features, str), "Nextest host target-features is invalid")
    require(isinstance(host.get("libdir"), dict)
            and set(host["libdir"]) == {"status", "path"}
            and isinstance(host["libdir"].get("status"), str)
            and host["libdir"].get("status") in {"available", "unavailable"}
            and (host["libdir"].get("path") is None or isinstance(host["libdir"].get("path"), str)),
            "Nextest host libdir descriptor is invalid")
    require(platforms.get("targets") == [], "Nextest LIST unexpectedly contains cross-target platforms")
    require(build_meta.get("target-platforms") == [{"triple": host_triple, "target-features": host_features}],
            "Nextest target platform does not match the host platform")
    require(build_meta.get("target-platform") is None,
            "Nextest LIST unexpectedly selected a cross-compilation target")
    suites = data.get("rust-suites")
    require(isinstance(suites, dict) and set(suites) == {BINARY_ID},
            f"Nextest LIST must contain exactly {BINARY_ID}")
    suite = suites[BINARY_ID]
    require(isinstance(suite, dict), "Nextest functional suite entry is not an object")
    require(set(suite) == LIST_SUITE_KEYS,
            f"Nextest suite has unexpected fields: {sorted(set(suite) ^ LIST_SUITE_KEYS)}")
    require(suite.get("package-name") == PACKAGE_NAME, "Nextest package-name mismatch")
    require(suite.get("binary-name") == BINARY_NAME, "Nextest binary-name mismatch")
    require(suite.get("binary-id") == BINARY_ID, "Nextest binary-id mismatch")
    require(suite.get("package-id") == f"path+{export_root.as_uri()}#{PACKAGE_NAME}@{PACKAGE_VERSION}",
            f"Nextest package-id mismatch: {suite.get('package-id')!r}")
    require(suite.get("kind") == "test", f"Nextest functional target kind mismatch: {suite.get('kind')!r}")
    require(suite.get("build-platform") == "target",
            f"Nextest build platform is not the native target: {suite.get('build-platform')!r}")
    require(suite.get("cwd") == str(export_root), f"Nextest suite cwd mismatch: {suite.get('cwd')!r}")
    binary_path_raw = suite.get("binary-path")
    require(isinstance(binary_path_raw, str) and Path(binary_path_raw).is_absolute(),
            "Nextest binary-path is missing or not absolute")
    binary_path = Path(binary_path_raw)
    require(not binary_path.is_symlink(), f"Nextest functional binary is a symlink: {binary_path}")
    expected_binary_directory = (target_root / "debug" / "deps").resolve(strict=False)
    resolved_binary = binary_path.resolve(strict=False)
    require(resolved_binary.parent == expected_binary_directory,
            f"Nextest functional binary is outside the expected dev-profile deps directory: {binary_path}")
    require(re.fullmatch(r"functional-[0-9a-f]+", binary_path.name) is not None,
            f"Nextest binary-path names an unexpected executable: {binary_path.name!r}")
    if binary_path.exists():
        require(binary_path.is_file(), f"Nextest functional binary is not a regular file: {binary_path}")
        binary_sha256 = sha256_file(resolved_binary)
        binary_size = resolved_binary.stat().st_size
    else:
        require(binary_evidence is not None and expected_binary_sha256 is not None,
                f"Nextest functional binary is missing or unsafe: {binary_path}")
        require(re.fullmatch(r"[0-9a-f]{64}", expected_binary_sha256) is not None,
                "stored Nextest functional binary SHA-256 is invalid")
        require(binary_evidence.is_file() and not binary_evidence.is_symlink(),
                "preserved Nextest functional binary evidence is missing or unsafe")
        binary_sha256 = sha256_file(binary_evidence)
        require(binary_sha256 == expected_binary_sha256,
                "preserved Nextest functional binary evidence digest differs from the LIST receipt")
        binary_size = binary_evidence.stat().st_size
    if expected_binary_sha256 is not None:
        require(binary_sha256 == expected_binary_sha256,
                "Nextest functional binary digest differs from the LIST receipt")
    require(suite.get("status") == "listed", f"Nextest suite is not listed: {suite.get('status')!r}")
    testcases = suite.get("testcases")
    require(isinstance(testcases, dict), "Nextest testcases is not an object")
    expected_names = {case["test_name"] for case in cases}
    actual_names = set(testcases)
    missing = sorted(expected_names - actual_names)
    extra = sorted(actual_names - expected_names)
    require(not missing and not extra, f"Nextest LIST names differ; missing={missing}; extra={extra}")
    require(type(data.get("test-count")) is int and data["test-count"] == 89,
            f"Nextest LIST test-count is not 89: {data.get('test-count')!r}")
    matches: dict[str, dict[str, Any]] = {}
    for case in cases:
        name = case["test_name"]
        item = testcases[name]
        require(isinstance(item, dict) and set(item) == LIST_TESTCASE_KEYS,
                f"Nextest testcase {name} has an unsupported shape")
        require(item.get("kind") == "test", f"Nextest testcase {name} is not a test")
        require(isinstance(item.get("ignored"), bool), f"Nextest testcase {name} has invalid ignored flag")
        selected, reason = parse_filter_match(item.get("filter-match"), f"Nextest testcase {name}")
        require(selected is not None, f"Nextest testcase {name} selection is unknown")
        matches[name] = {"selected": selected, "reason": reason, "ignored": item["ignored"]}
    return {"data": data, "matches": matches, "test_count": len(testcases), "binary_id": BINARY_ID,
            "package_id": suite["package-id"], "binary_path": str(resolved_binary),
            "binary_sha256": binary_sha256, "binary_size": binary_size,
            "export_root": str(export_root), "target_root": str(target_root)}


def stage_list_binary(run_root: Path, source: Path, expected_sha256: str) -> tuple[str, int]:
    relative = "functional-test-binary"
    destination = run_root / relative
    require(not destination.exists() and not destination.is_symlink(),
            "functional-test binary evidence path already exists")
    mode = stat.S_IMODE(source.stat().st_mode)
    fd, temporary = tempfile.mkstemp(prefix=".functional-test-binary.", dir=run_root)
    temporary_path = Path(temporary)
    try:
        with source.open("rb") as input_stream, os.fdopen(fd, "wb") as output_stream:
            shutil.copyfileobj(input_stream, output_stream, length=1024 * 1024)
            output_stream.flush()
            os.fsync(output_stream.fileno())
        os.chmod(temporary_path, mode)
        require(sha256_file(temporary_path) == expected_sha256,
                "staged functional-test binary digest differs from the LIST receipt")
        os.replace(temporary_path, destination)
    finally:
        if temporary_path.exists():
            temporary_path.unlink()
    return relative, destination.stat().st_size


def record_list(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    invocation, cases = load_run(run_root)
    try:
        diagnostic_filter = invocation.get("diagnostic_filter")
        require(diagnostic_filter is None or isinstance(diagnostic_filter, str),
                "invocation diagnostic filter has an invalid type")
        require((args.mode == "normal" and diagnostic_filter is None)
                or (args.mode == "diagnostic" and isinstance(diagnostic_filter, str) and diagnostic_filter),
                "Nextest LIST mode disagrees with the invocation filter")
        export_root = Path(args.export_root).resolve(strict=True)
        target_root = Path(args.target_root).resolve(strict=True)
        validate_list_command(invocation, run_root, export_root, target_root)
        list_input = Path(args.list_json)
        require(list_input.is_file() and not list_input.is_symlink(),
                "Nextest LIST input is missing or unsafe")
        parsed = parse_nextest_list(Path(args.list_json), cases, export_root, target_root)
        require(list_input.read_bytes() == (run_root / "global/logs/nextest-list.stdout.log").read_bytes(),
                "Nextest LIST input differs from the runner's captured stdout")
        identity_ok = True
        error = None
        if args.mode == "normal":
            mismatched = [name for name, value in parsed["matches"].items() if not value["selected"]]
            ignored = [name for name, value in parsed["matches"].items() if value["ignored"]]
            if mismatched or ignored:
                identity_ok = False
                error = f"normal unfiltered LIST selected fewer than all cases; mismatched={mismatched}; ignored={ignored}"
    except LedgerError as error_obj:
        parsed = None
        identity_ok = False
        error = str(error_obj)
    destination = run_root / "nextest-list.json"
    if Path(args.list_json).is_file() and not Path(args.list_json).is_symlink():
        shutil.copyfile(args.list_json, destination)
    if parsed is not None:
        try:
            binary_evidence_path, staged_binary_size = stage_list_binary(
                run_root, Path(parsed["binary_path"]), parsed["binary_sha256"]
            )
        except (LedgerError, OSError) as error_obj:
            parsed = None
            identity_ok = False
            error = str(error_obj)
    if parsed is not None:
        invocation.update({
            "list_mode": args.mode,
            "list_valid": identity_ok,
            "list_sha256": sha256_file(destination),
            "list_test_count": parsed["test_count"],
            "list_identity": parsed["binary_id"],
            "list_export_root": parsed["export_root"],
            "list_target_root": parsed["target_root"],
            "list_package_id": parsed["package_id"],
            "list_binary_path": parsed["binary_path"],
            "list_binary_sha256": parsed["binary_sha256"],
            "list_binary_size": parsed["binary_size"],
            "list_binary_evidence_path": binary_evidence_path,
            "list_binary_evidence_size": staged_binary_size,
            "list_error": error,
        })
        for side in SIDE_NAMES:
            ledger_path = participant_dir(run_root, side) / "ledger.json"
            ledger = read_json(ledger_path, f"{side} ledger")
            for row in ledger["cases"]:
                match = parsed["matches"][row["test_name"]]
                row["selected"] = match["selected"]
                if not match["selected"]:
                    row["status"] = "not_run"
                    row["reason"] = f"not selected by filter: {match['reason']}"
                else:
                    row["status"] = "pending"
                    row["reason"] = "selected by Nextest LIST; awaiting final case receipt"
            ledger["selection_known"] = True
            ledger["selected_count"] = sum(1 for value in parsed["matches"].values() if value["selected"])
            ledger["list_sha256"] = invocation.get("list_sha256")
            ledger["list_error"] = error
            atomic_json(ledger_path, ledger)
            side_state_path = participant_dir(run_root, side) / "state.json"
            side_state = read_json(side_state_path, f"{side} state")
            side_state["selection_count"] = ledger["selected_count"]
            atomic_json(side_state_path, side_state)
    else:
        invocation.update({"list_mode": args.mode, "list_valid": False,
                           "list_sha256": None, "list_error": error})
        for side in SIDE_NAMES:
            ledger_path = participant_dir(run_root, side) / "ledger.json"
            ledger = read_json(ledger_path, f"{side} ledger")
            ledger["selection_known"] = False
            for row in ledger["cases"]:
                row["selected"] = None
                row["status"] = "incomplete"
                row["reason"] = "selection unknown: Nextest LIST identity is missing or invalid"
            ledger["list_error"] = error
            atomic_json(ledger_path, ledger)
    atomic_json(run_root / "invocation.json", invocation)
    if not identity_ok:
        fail(error or "Nextest LIST is invalid")
    print(f"validated {parsed['test_count']} Nextest test names; selected={sum(v['selected'] for v in parsed['matches'].values())}")


def verify_list_binary(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    invocation, cases = load_run(run_root)
    target_raw = invocation.get("list_target_root")
    recorded_sha = invocation.get("list_binary_sha256")
    require(isinstance(target_raw, str), "Nextest LIST target identity is missing")
    require(isinstance(recorded_sha, str) and re.fullmatch(r"[0-9a-f]{64}", recorded_sha) is not None,
            "Nextest LIST binary digest is missing or invalid")
    binary_path_raw = invocation.get("list_binary_path")
    require(isinstance(binary_path_raw, str) and Path(binary_path_raw).is_absolute(),
            "recorded functional test binary path is missing or invalid")
    current_binary = Path(binary_path_raw)
    require(current_binary.is_file() and not current_binary.is_symlink(),
            "current Nextest functional binary is missing or unsafe before RUN")
    require(sha256_file(current_binary) == recorded_sha,
            "current Nextest functional binary changed after LIST")
    recorded_size = invocation.get("list_binary_size")
    require(type(recorded_size) is int and current_binary.stat().st_size == recorded_size,
            "current Nextest functional binary size changed after LIST")
    target_root = Path(args.target_root).resolve(strict=True)
    require(str(target_root) == target_raw, "requested LIST target root differs from the recorded target")
    parsed = parse_saved_nextest_list(run_root, invocation, cases)
    require(parsed["binary_sha256"] == recorded_sha, "current LIST binary differs from the recorded digest")
    print(json.dumps({"binary_path": parsed["binary_path"], "binary_sha256": recorded_sha,
                      "binary_size": parsed["binary_size"],
                      "evidence_path": invocation["list_binary_evidence_path"]}, sort_keys=True))


def set_value(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    require(args.side in SIDE_NAMES, f"unknown participant: {args.side}")
    allowed = {
        "built", "build_exit", "build_target_root", "run_launched", "run_exit", "run_signal", "run_pid",
        "run_process_group", "never_launched_reason", "junit_path", "run_duration_seconds",
    }
    require(args.key in allowed, f"participant field is not writable: {args.key}")
    try:
        value = json.loads(args.value)
    except json.JSONDecodeError as error:
        raise LedgerError(f"participant value must be JSON: {error}") from error
    state_path = participant_dir(run_root, args.side) / "state.json"
    state = read_json(state_path, f"{args.side} state")
    state[args.key] = value
    state["updated_at"] = utc_now()
    atomic_json(state_path, state)


def record_command(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    invocation_path = run_root / "invocation.json"
    invocation = read_json(invocation_path, "invocation")
    require(args.participant == "global" or args.participant in SIDE_NAMES,
            f"unknown command participant: {args.participant}")
    argv = read_json(Path(args.argv_json), "command argv")
    require(isinstance(argv, list) and all(isinstance(item, str) for item in argv), "command argv is invalid")
    log_paths: dict[str, dict[str, Any]] = {}
    for label, raw in (("stdout", args.stdout), ("stderr", args.stderr)):
        if raw:
            path = Path(raw).resolve(strict=True)
            log_paths[label] = {"path": os.path.relpath(path, run_root), "sha256": sha256_file(path), "size": path.stat().st_size}
    item = {
        "name": args.name,
        "participant": args.participant,
        "argv": argv,
        "cwd": args.cwd,
        "started_at": args.started_at,
        "ended_at": args.ended_at,
        "duration_seconds": args.duration_seconds,
        "exit_status": args.exit_status,
        "signal": args.signal,
        "pid": args.pid,
        "process_group": args.process_group,
        "logs": log_paths,
    }
    invocation.setdefault("commands", []).append(item)
    atomic_json(invocation_path, invocation)
    if args.participant in SIDE_NAMES:
        side_path = participant_dir(run_root, args.participant) / "commands" / f"{args.name}.json"
        atomic_json(side_path, item)


def copy_capture_counts(case_root: Path, row: dict[str, Any]) -> tuple[int | None, int | None, int | None]:
    if row.get("planned_checkpoint_names") is None:
        return 0, 0, 0
    indexes: list[Path] = []
    if case_root.is_dir() and not case_root.is_symlink():
        for current, dirnames, filenames in os.walk(case_root, topdown=True, followlinks=False):
            directory = Path(current)
            for name in list(dirnames):
                path = directory / name
                require(not path.is_symlink(), f"symlink in case output: {path}")
            if "capture-index.json" in filenames:
                index_path = directory / "capture-index.json"
                require(not index_path.is_symlink(), f"capture index is symlink: {index_path}")
                indexes.append(index_path)
    if not indexes:
        return 0, 0, 0
    expected_names = set(row["planned_checkpoint_names"])
    reached_names: set[str] = set()
    compared_names: set[str] = set()
    matched_names: set[str] = set()
    for index_path in indexes:
        index = read_json(index_path, "capture index")
        require(index.get("test_name") == row["test_name"],
                f"capture index test name mismatch for {row['test_name']}")
        require(index.get("case_id") == row["case_id"],
                f"capture index case ID mismatch for {row['test_name']}")
        attempts = index.get("checkpoint_attempts", [])
        checkpoints = index.get("checkpoints", [])
        require(isinstance(attempts, list) and isinstance(checkpoints, list),
                f"capture index arrays malformed for {row['test_name']}")
        for attempt in attempts:
            require(isinstance(attempt, dict) and isinstance(attempt.get("name"), str),
                    f"capture attempt malformed for {row['test_name']}")
            name = attempt["name"]
            require(name in expected_names, f"capture index has unknown checkpoint {name!r}")
            if attempt.get("reached") is True:
                reached_names.add(name)
        for checkpoint in checkpoints:
            require(isinstance(checkpoint, dict) and isinstance(checkpoint.get("name"), str),
                    f"captured checkpoint malformed for {row['test_name']}")
            name = checkpoint["name"]
            require(name in expected_names, f"capture index has unknown captured checkpoint {name!r}")
            comparison = checkpoint.get("comparison")
            require(isinstance(comparison, dict) and isinstance(comparison.get("passed"), bool),
                    f"checkpoint comparison malformed for {row['test_name']}:{name}")
            compared_names.add(name)
            if comparison["passed"]:
                matched_names.add(name)
    return len(reached_names), len(compared_names), len(matched_names)


def parse_saved_nextest_list(run_root: Path, invocation: dict[str, Any],
                             cases: list[dict[str, Any]]) -> dict[str, Any]:
    export_root = invocation.get("list_export_root")
    target_root = invocation.get("list_target_root")
    evidence_relative = invocation.get("list_binary_evidence_path")
    binary_sha256 = invocation.get("list_binary_sha256")
    require(isinstance(export_root, str) and isinstance(target_root, str),
            "saved Nextest LIST export/target identity is missing")
    require(evidence_relative == "functional-test-binary",
            "saved Nextest LIST binary evidence path is missing or unexpected")
    require(isinstance(binary_sha256, str) and re.fullmatch(r"[0-9a-f]{64}", binary_sha256) is not None,
            "saved Nextest LIST binary digest is missing or invalid")
    evidence = run_root / safe_relpath(evidence_relative, "LIST binary evidence path")
    require(evidence.is_file() and not evidence.is_symlink()
            and evidence.stat().st_size == invocation.get("list_binary_evidence_size"),
            "saved Nextest LIST binary evidence size/path is invalid")
    validate_list_command(invocation, run_root, Path(export_root), Path(target_root))
    list_path = run_root / "nextest-list.json"
    parsed = parse_nextest_list(
        list_path, cases, Path(export_root), Path(target_root),
        binary_evidence=evidence, expected_binary_sha256=binary_sha256,
    )
    require(invocation.get("list_sha256") == sha256_file(list_path),
            "saved Nextest LIST digest differs from its invocation identity")
    require(list_path.read_bytes() == (run_root / "global/logs/nextest-list.stdout.log").read_bytes(),
            "saved Nextest LIST differs from the runner's captured stdout")
    require(type(invocation.get("list_valid")) is bool,
            "saved Nextest LIST validity flag is missing or invalid")
    diagnostic_filter = invocation.get("diagnostic_filter")
    require(diagnostic_filter is None or isinstance(diagnostic_filter, str),
            "saved invocation diagnostic filter is invalid")
    expected_mode = "diagnostic" if diagnostic_filter else "normal"
    require(invocation.get("list_mode") == expected_mode,
            "saved Nextest LIST mode disagrees with the invocation filter")
    require(invocation.get("list_error") is None or isinstance(invocation.get("list_error"), str),
            "saved Nextest LIST error field is invalid")
    expected_identity = {
        "list_test_count": parsed["test_count"],
        "list_identity": parsed["binary_id"],
        "list_export_root": parsed["export_root"],
        "list_target_root": parsed["target_root"],
        "list_package_id": parsed["package_id"],
        "list_binary_path": parsed["binary_path"],
        "list_binary_sha256": parsed["binary_sha256"],
        "list_binary_size": parsed["binary_size"],
    }
    for key, expected in expected_identity.items():
        require(invocation.get(key) == expected,
                f"saved Nextest LIST field {key} differs from the raw LIST/binary identity")
    return parsed


def artifact_integrity_scope(integrity: dict[str, Any] | None,
                             cases: list[dict[str, Any]]) -> dict[str, Any]:
    scope: dict[str, Any] = {
        "list_untrusted": False,
        "all_untrusted": False,
        "side_untrusted": set(),
        "case_untrusted": set(),
        "junit_untrusted": set(),
        "reasons": {},
    }
    if integrity is None:
        return scope
    if not isinstance(integrity, dict):
        scope["all_untrusted"] = True
        scope["list_untrusted"] = True
        scope["reasons"]["*"] = "artifact integrity record is not an object"
        return scope
    status = integrity.get("status")
    mismatches = integrity.get("mismatched_paths")
    if status == "verified":
        return scope
    if status not in {"mismatch", "unavailable"} or not isinstance(mismatches, dict):
        scope["all_untrusted"] = True
        scope["list_untrusted"] = True
        scope["reasons"]["*"] = "artifact integrity record is unavailable or malformed"
        return scope
    if status == "unavailable":
        scope["all_untrusted"] = True
        scope["list_untrusted"] = True
        errors = integrity.get("errors", [])
        detail = "; ".join(item for item in errors if isinstance(item, str)) if isinstance(errors, list) else ""
        scope["reasons"]["*"] = detail or "artifact manifest could not be verified"
        return scope
    cases_by_id = {case["case_id"]: case["test_name"] for case in cases}
    ids_by_name = {case["test_name"]: case["case_id"] for case in cases}
    for relative, reason in mismatches.items():
        if not isinstance(relative, str) or not isinstance(reason, str):
            scope["all_untrusted"] = True
            scope["list_untrusted"] = True
            scope["reasons"]["*"] = "artifact mismatch inventory is malformed"
            continue
        scope["reasons"][relative] = reason
        if relative in {"nextest-list.json", "global/logs/nextest-list.stdout.log"} \
                or relative.startswith("global/commands/nextest-list."):
            scope["list_untrusted"] = True
            continue
        if relative in {"invocation.json", "inventory.json"} or relative.startswith("global/"):
            scope["all_untrusted"] = True
            if relative in {"invocation.json", "inventory.json"} \
                    or relative.startswith("global/commands/nextest-list.") \
                    or relative.startswith("global/logs/nextest-list."):
                scope["list_untrusted"] = True
            continue
        match = re.match(r"^participants/(pr26|candidate)/out/v3/([^/]+)/", relative)
        if match and match.group(2) in cases_by_id:
            scope["case_untrusted"].add((match.group(1), match.group(2)))
            continue
        match = re.match(r"^participants/(pr26|candidate)/out/v3/case-results/([^/]+)\.json$", relative)
        if match and match.group(2) in ids_by_name:
            scope["case_untrusted"].add((match.group(1), ids_by_name[match.group(2)]))
            continue
        match = re.match(r"^participants/(pr26|candidate)/(.+)$", relative)
        if match:
            side, suffix = match.groups()
            scope["side_untrusted"].add(side)
            if suffix in {"junit.xml", "junit-source-original.xml", "junit-source.json"}:
                scope["junit_untrusted"].add(side)
            continue
        scope["all_untrusted"] = True
    return scope


def load_junit(path: Path, list_matches: dict[str, dict[str, Any]]) -> tuple[dict[str, str], str | None]:
    if not path.is_file() or path.is_symlink():
        return {}, "JUnit report missing"
    raw = path.read_bytes()
    require(len(raw) <= 64 * 1024 * 1024, "JUnit report exceeds 64 MiB")
    upper = raw.upper()
    require(b"<!DOCTYPE" not in upper and b"<!ENTITY" not in upper,
            "JUnit report contains a prohibited DTD/entity declaration")
    try:
        root = ET.fromstring(raw)
    except ET.ParseError as error:
        return {}, f"JUnit XML invalid: {error}"
    if root.tag == "testsuite":
        suites = [root]
    elif root.tag == "testsuites":
        suites = [child for child in root if child.tag == "testsuite"]
    else:
        return {}, f"JUnit root element is not testsuite(s): {root.tag!r}"
    rows: dict[str, str] = {}
    for suite in suites:
        for testcase in suite.findall("testcase"):
            name = testcase.attrib.get("name")
            if not name:
                return {}, "JUnit testcase has no name"
            status_nodes = [child for child in testcase if child.tag in {"failure", "error", "skipped"}]
            if len(status_nodes) > 1:
                return {}, f"JUnit testcase has multiple terminal result nodes: {name}"
            status = "passed" if not status_nodes else {
                "failure": "failed",
                "error": "error",
                "skipped": "skipped",
            }[status_nodes[0].tag]
            if name in rows:
                return {}, f"JUnit has duplicate testcase name: {name}"
            rows[name] = status
    listed_names = set(list_matches)
    unexpected = sorted(set(rows) - listed_names)
    if unexpected:
        return rows, f"JUnit contains names absent from the 89-name Nextest LIST: {unexpected}"
    missing = sorted(listed_names - set(rows))
    if missing:
        return rows, f"JUnit is missing names from the 89-name Nextest LIST: {missing}"
    for name, match in list_matches.items():
        selected = match.get("selected")
        if selected is True and rows[name] == "skipped":
            return rows, f"JUnit marks selected testcase as skipped: {name}"
        if selected is False and rows[name] != "skipped":
            return rows, f"JUnit ran or failed an unselected testcase: {name} ({rows[name]})"
    return rows, None


def case_receipt_path(out_root: Path, test_name: str) -> Path:
    require(CASE_NAME_RE.fullmatch(test_name) is not None, f"unsafe test name in inventory: {test_name}")
    return out_root / "v3" / "case-results" / f"{test_name}.json"


def parse_utc_timestamp(value: Any, label: str) -> datetime:
    require(isinstance(value, str) and RFC3339_RE.fullmatch(value) is not None,
            f"{label} is missing or malformed")
    try:
        parsed = datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError as error:
        raise LedgerError(f"{label} is not a valid calendar timestamp") from error
    require(parsed.tzinfo is not None and parsed.utcoffset() == timezone.utc.utcoffset(parsed),
            f"{label} is not UTC")
    return parsed


def validate_sha256_value(value: Any, label: str, *, nullable: bool = False) -> None:
    if nullable and value is None:
        return
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None,
            f"{label} is not a lowercase SHA-256")


def validate_nullable(value: Any, expected_type: type, label: str) -> None:
    require(value is None or type(value) is expected_type, f"{label} has the wrong type")


def validate_not_run_checkpoints(value: Any, planned_names: list[str], label: str) -> None:
    require(isinstance(value, list), f"{label} is not an array")
    seen: set[str] = set()
    for item in value:
        require(isinstance(item, dict) and set(item) == NOT_RUN_CHECKPOINT_KEYS,
                f"{label} has an unsupported item shape")
        name = item.get("name")
        require(isinstance(name, str) and name in planned_names,
                f"{label} contains an unknown checkpoint: {name!r}")
        require(name not in seen, f"{label} contains a duplicate checkpoint: {name}")
        seen.add(name)
        require(item.get("status") == "not-run", f"{label} has a non-not-run status")
        require(isinstance(item.get("reason"), str) and item["reason"].strip(),
                f"{label} has an empty reason")


def validate_phase_receipt(value: Any, expected: dict[str, Any], case_id: str) -> None:
    require(isinstance(value, dict) and set(value) == PHASE_RECEIPT_KEYS,
            f"phase receipt {expected['name']} has an unsupported field set")
    for key in ("name", "motion", "cleanup"):
        require(isinstance(value.get(key), str) and value[key] == expected[key],
                f"phase receipt {expected['name']} {key} differs from the frozen plan")
    planned_names = expected["planned_checkpoint_names"]
    require(value.get("planned_checkpoint_names") == planned_names,
            f"phase receipt {expected['name']} checkpoint plan differs from the frozen plan")
    status = value.get("status")
    require(isinstance(status, str) and status in PHASE_STATUSES,
            f"phase receipt {expected['name']} status is invalid")
    comparison_kind = value.get("comparison_failure_kind")
    require(comparison_kind is None or (isinstance(comparison_kind, str)
            and comparison_kind in COMPARISON_FAILURE_KINDS),
            f"phase receipt {expected['name']} comparison failure kind is invalid")
    require(type(value.get("launched")) is bool,
            f"phase receipt {expected['name']} launched flag is invalid")
    cleanup_actions = value.get("cleanup_actions_attempted")
    require(isinstance(cleanup_actions, list)
            and all(isinstance(action, str) and action for action in cleanup_actions),
            f"phase receipt {expected['name']} cleanup actions are invalid")

    nullable_fields = {
        "wait_exit_succeeded": bool,
        "exit_status_zero": bool,
        "no_signal": bool,
        "eof_drained": bool,
        "post_drain_exit_confirmed": bool,
        "terminal_restored": bool,
        "exit_status": int,
        "exit_signal": str,
        "raw_pty_sha256": str,
        "cleanup_snapshot_file": str,
        "cleanup_snapshot_sha256": str,
        "cleanup_raw_prefix_sha256": str,
        "error": str,
    }
    for key, expected_type in nullable_fields.items():
        validate_nullable(value.get(key), expected_type, f"phase receipt {expected['name']} {key}")
    if value.get("exit_status") is not None:
        require(-(2**31) <= value["exit_status"] < 2**31,
                f"phase receipt {expected['name']} exit status is outside i32")
    for key in ("raw_pty_sha256", "cleanup_snapshot_sha256", "cleanup_raw_prefix_sha256"):
        validate_sha256_value(value.get(key), f"phase receipt {expected['name']} {key}", nullable=True)
    prefixes = value.get("raw_checkpoint_prefixes")
    require(isinstance(prefixes, dict), f"phase receipt {expected['name']} raw checkpoint prefixes are invalid")
    require(set(prefixes).issubset(set(planned_names)),
            f"phase receipt {expected['name']} has an unknown raw checkpoint prefix")
    for name, digest in prefixes.items():
        validate_sha256_value(digest, f"phase receipt {expected['name']} raw prefix {name}")

    snapshot_file = value.get("cleanup_snapshot_file")
    snapshot_sha = value.get("cleanup_snapshot_sha256")
    require((snapshot_file is None) == (snapshot_sha is None),
            f"phase receipt {expected['name']} cleanup snapshot path/hash are incomplete")
    if snapshot_file is not None:
        snapshot_parts = snapshot_file.split("/")
        require(len(snapshot_parts) >= 5
                and not snapshot_file.startswith("/") and "\\" not in snapshot_file
                and all(part not in ("", ".") for part in snapshot_parts)
                and snapshot_parts.count("..") == 1
                and snapshot_parts[0] == ".."
                and snapshot_parts[1] == case_id
                and snapshot_parts[2] == "phases"
                and snapshot_parts[3] == expected["name"],
                f"phase receipt {expected['name']} cleanup snapshot path is unsafe")

    if value["exit_status_zero"] is not None and value["exit_status"] is not None:
        require(value["exit_status_zero"] is (value["exit_status"] == 0),
                f"phase receipt {expected['name']} exit status boolean disagrees with its status")
    if value["exit_signal"] is not None and value["no_signal"] is not None:
        require(value["no_signal"] is (value["exit_signal"] is None),
                f"phase receipt {expected['name']} signal evidence is inconsistent")
    if status == "passed":
        require(value["launched"] is True and value["wait_exit_succeeded"] is True
                and value["exit_status"] == 0 and value["exit_status_zero"] is True
                and value["exit_signal"] is None and value["no_signal"] is True
                and value["eof_drained"] is True and value["post_drain_exit_confirmed"] is True
                and value["terminal_restored"] is True and value["raw_pty_sha256"] is not None
                and comparison_kind is None and value["error"] is None,
                f"phase receipt {expected['name']} passed without complete lifecycle evidence")
    elif status == "setup_failed":
        require(value["launched"] is False and isinstance(value["error"], str) and value["error"].strip(),
                f"phase receipt {expected['name']} setup failure lacks explicit prelaunch evidence")
    elif status == "not_run":
        require(value["launched"] is False,
                f"phase receipt {expected['name']} not-run status claims launch")
    elif status == "launched":
        require(value["launched"] is True,
                f"phase receipt {expected['name']} launched status lacks a launch")
    elif status == "failed":
        require(value["launched"] is True
                and ((isinstance(value["error"], str) and value["error"].strip())
                     or comparison_kind is not None),
                f"phase receipt {expected['name']} failed status lacks failure evidence")

def validate_case_receipt(path: Path, row: dict[str, Any], state: dict[str, Any]) -> tuple[dict[str, Any] | None, str | None]:
    if not path.is_file() or path.is_symlink():
        return None, "case receipt missing"
    try:
        receipt = read_json(path, f"case receipt {row['test_name']}")
        require(isinstance(receipt, dict), "case receipt root is not an object")
        keys = set(receipt)
        require(keys in (CASE_RECEIPT_REQUIRED_KEYS, CASE_RECEIPT_REQUIRED_KEYS | CASE_RECEIPT_OPTIONAL_KEYS),
                "case receipt has missing or unknown fields")
        require(type(receipt.get("schema_version")) is int and receipt["schema_version"] == 2,
                "case receipt schema is not version 2")
        status = receipt.get("status")
        require(isinstance(status, str) and status in CASE_STATUSES,
                "case receipt status is missing or invalid")
        initial_pending = (status == "pending" and receipt.get("stage") == "manifest"
                           and receipt.get("case_ended_at") is None)
        suite_id = receipt.get("suite_id")
        require(suite_id == SUITE_ID or (initial_pending and suite_id is None),
                "case receipt suite ID is missing or mismatched")
        run_id = state.get("run_id")
        require(isinstance(run_id, str) and safe_run_id(run_id)
                and receipt.get("run_id") == run_id,
                "case receipt run ID is missing or mismatched")
        require(receipt.get("test_name") == row["test_name"], "case receipt test name mismatch")
        case_identity = receipt.get("case_id")
        require(case_identity == row["case_id"] or (initial_pending and case_identity is None),
                "case receipt case ID is missing or mismatched")
        pins = receipt.get("engine_pins")
        require(pins == ENGINE_PINS or (initial_pending and pins == {}),
                "case receipt engine pins are missing or mismatched")
        if case_identity is None:
            require(initial_pending and pins == {} and suite_id is None
                    and receipt.get("actions") == []
                    and receipt.get("planned_checkpoint_names") == []
                    and receipt.get("not_run_checkpoints") == []
                    and receipt.get("phases", []) == [],
                    "initial pending case receipt contains evidence before manifest identity")
        planned_names = row.get("planned_checkpoint_names") or []
        receipt_planned_names = receipt.get("planned_checkpoint_names")
        require(isinstance(receipt_planned_names, list)
                and (receipt_planned_names == planned_names
                     or (initial_pending and case_identity is None and receipt_planned_names == [])),
                "case receipt planned checkpoint names differ from the frozen plan")
        require(len(set(receipt_planned_names)) == len(receipt_planned_names)
                and all(isinstance(name, str) for name in receipt_planned_names),
                "case receipt planned checkpoint names are malformed")
        planned_names = receipt_planned_names
        started = parse_utc_timestamp(receipt.get("case_started_at"), "case receipt start timestamp")
        ended_raw = receipt.get("case_ended_at")
        ended = (None if ended_raw is None
                 else parse_utc_timestamp(ended_raw, "case receipt end timestamp"))
        if ended is not None:
            require(ended >= started, "case receipt end timestamp precedes its start")
        require(type(receipt.get("launched")) is bool, "case receipt launched flag has the wrong type")
        require((status == "pending") == (ended is None),
                "case receipt pending status and end timestamp disagree")
        failure_kind = receipt.get("failure_kind")
        require(failure_kind is None or (isinstance(failure_kind, str) and failure_kind in CASE_FAILURE_KINDS),
                "case receipt failure kind is invalid")
        comparison_kind = receipt.get("comparison_failure_kind")
        require(comparison_kind is None or (isinstance(comparison_kind, str)
                and comparison_kind in COMPARISON_FAILURE_KINDS),
                "case receipt comparison failure kind is invalid")
        stage = receipt.get("stage")
        require(isinstance(stage, str) and stage in CASE_STAGES,
                "case receipt stage is missing or invalid")
        error_text = receipt.get("error")
        require(error_text is None or isinstance(error_text, str),
                "case receipt error field has the wrong type")
        fault_control = receipt.get("fault_control")
        require(isinstance(fault_control, dict) and set(fault_control) == {"active", "spec"}
                and type(fault_control.get("active")) is bool
                and (fault_control.get("spec") is None or isinstance(fault_control.get("spec"), str)),
                "case receipt fault-control object has an invalid schema")
        require(fault_control == {"active": False, "spec": None},
                "ordinary CI case receipt must have fault control inactive")

        not_run = receipt.get("not_run_checkpoints")
        validate_not_run_checkpoints(not_run, planned_names, "case receipt not_run_checkpoints")
        actions = receipt.get("actions")
        require(isinstance(actions, list), "case receipt actions are missing or not an array")
        planned_phases = row.get("planned_phases") or []
        phase_names = {phase["name"] for phase in planned_phases}
        checkpoints_by_phase = {
            phase["name"]: set(phase["planned_checkpoint_names"])
            for phase in planned_phases
        }
        for sequence, action in enumerate(actions, start=1):
            require(isinstance(action, dict) and set(action) == ACTION_RECEIPT_KEYS,
                    f"case receipt action {sequence} has missing or unknown fields")
            require(action.get("case_id") == row["case_id"],
                    f"case receipt action {sequence} case ID mismatch")
            phase_name = action.get("phase_name")
            require(phase_name is None or (isinstance(phase_name, str) and phase_name in phase_names),
                    f"case receipt action {sequence} phase is outside the frozen plan")
            checkpoint_name = action.get("checkpoint_name")
            cleanup_action_checkpoint = (isinstance(checkpoint_name, str)
                                         and checkpoint_name in PHASE_CLEANUP_ACTION_CHECKPOINTS)
            require(isinstance(checkpoint_name, str)
                    and (checkpoint_name in set(planned_names) | {"exit"}
                         or (cleanup_action_checkpoint and phase_name is not None)),
                    f"case receipt action {sequence} checkpoint is outside the frozen plan")
            checkpoint_phases = {
                name for name, checkpoints in checkpoints_by_phase.items()
                if checkpoint_name in checkpoints
            }
            if phase_name is None:
                require(not checkpoint_phases and not cleanup_action_checkpoint,
                        f"case receipt action {sequence} omits its required phase identity")
            else:
                require(checkpoint_name == "exit" or cleanup_action_checkpoint
                        or checkpoint_name in checkpoints_by_phase[phase_name],
                        f"case receipt action {sequence} checkpoint does not belong to phase {phase_name}")
            require(type(action.get("sequence")) is int and action["sequence"] == sequence,
                    f"case receipt action sequence {sequence} is invalid")
            expected_id = (f"{run_id}/{row['case_id']}/{phase_name if phase_name is not None else 'null'}"
                           f"/{checkpoint_name}/{sequence:04}")
            require(action.get("action_id") == expected_id,
                    f"case receipt action {sequence} ID does not match its identity fields")
            require(isinstance(action.get("action_kind"), str)
                    and action["action_kind"] in ACTION_KINDS,
                    f"case receipt action {sequence} kind is invalid")
            attempted = parse_utc_timestamp(action.get("attempted_at"),
                                            f"case receipt action {sequence} attempt timestamp")
            require(started <= attempted and (ended is None or attempted <= ended),
                    f"case receipt action {sequence} attempt timestamp is outside the case")
            dispatch_state = action.get("dispatch_state")
            require(isinstance(dispatch_state, str) and dispatch_state in ACTION_DISPATCH_STATES,
                    f"case receipt action {sequence} dispatch state is invalid")
            dispatch_at_raw = action.get("dispatch_completed_at")
            if dispatch_state == "dispatched":
                dispatch_at = parse_utc_timestamp(dispatch_at_raw,
                                                  f"case receipt action {sequence} dispatch timestamp")
                require(attempted <= dispatch_at and (ended is None or dispatch_at <= ended),
                        f"case receipt action {sequence} dispatch timestamp is out of order")
            else:
                require(dispatch_at_raw is None,
                        f"case receipt action {sequence} has a completion time without a dispatched state")
            if dispatch_state == "suppressed_by_control":
                raise LedgerError(f"case receipt action {sequence} was suppressed while fault control is inactive")
            observation_state = action.get("observation_state")
            observation_at_raw = action.get("observation_at")
            require(observation_state is None or (isinstance(observation_state, str)
                    and observation_state in ACTION_OBSERVATION_STATES),
                    f"case receipt action {sequence} observation state is invalid")
            if observation_state is None:
                require(observation_at_raw is None,
                        f"case receipt action {sequence} has an observation time without an outcome")
            else:
                observed_at = parse_utc_timestamp(observation_at_raw,
                                                  f"case receipt action {sequence} observation timestamp")
                require(started <= attempted <= observed_at
                        and (ended is None or observed_at <= ended),
                        f"case receipt action {sequence} observation timestamp is out of order")
                require(dispatch_state == "dispatched",
                        f"case receipt action {sequence} has an observation without successful dispatch")
                if dispatch_state == "dispatched":
                    require(dispatch_at <= observed_at,
                            f"case receipt action {sequence} observation precedes dispatch completion")

        phases = receipt.get("phases", [])
        require(isinstance(phases, list), "case receipt phases field is not an array")
        require(len(phases) == len(planned_phases)
                or (initial_pending and case_identity is None and phases == []),
                "case receipt phase inventory differs from the frozen plan")
        for phase_value, phase_plan in zip(phases, planned_phases):
            validate_phase_receipt(phase_value, phase_plan, row["case_id"])
            snapshot_relative = phase_value.get("cleanup_snapshot_file")
            if snapshot_relative is not None:
                case_root = path.parent.parent / row["case_id"]
                snapshot_path = path.parent / snapshot_relative
                check_no_symlink_components(snapshot_path)
                require(snapshot_path.is_file() and not snapshot_path.is_symlink(),
                        f"phase {phase_plan['name']} cleanup snapshot is missing or unsafe")
                try:
                    snapshot_path.resolve(strict=True).relative_to(case_root.resolve(strict=True))
                except (OSError, RuntimeError, ValueError) as error:
                    raise LedgerError(
                        f"phase {phase_plan['name']} cleanup snapshot escapes its case output"
                    ) from error
                require(sha256_file(snapshot_path) == phase_value["cleanup_snapshot_sha256"],
                        f"phase {phase_plan['name']} cleanup snapshot digest mismatch")

        if status == "pending":
            # The Suite writes its receipt before launch and persists action updates as
            # they happen. A surviving pending receipt is evidence of an interrupted
            # case, not a terminal failure or pass.
            return receipt, None

        if status == "passed":
            require(receipt["launched"] is True and stage == "finalize"
                    and failure_kind is None and comparison_kind is None and error_text is None
                    and not not_run,
                    "passed case receipt violates terminal status invariants")
            require(all(phase.get("status") == "passed" and phase.get("launched") is True
                        for phase in phases),
                    "passed case receipt has a phase without successful lifecycle evidence")
            phases_by_name = {phase["name"]: phase for phase in phases}
            for action in actions:
                checkpoint_name = action["checkpoint_name"]
                if checkpoint_name in PHASE_CLEANUP_ACTION_CHECKPOINTS:
                    phase = phases_by_name[action["phase_name"]]
                    expected_cleanup_action = PHASE_CLEANUP_ACTION_CHECKPOINTS[checkpoint_name]
                    require(action["action_kind"] == "key"
                            and action["dispatch_state"] == "dispatched"
                            and action["observation_state"] is None
                            and action["observation_at"] is None
                            and expected_cleanup_action in phase["cleanup_actions_attempted"],
                            f"passed case phase cleanup action {checkpoint_name} lacks dispatch/lifecycle evidence")
                    if checkpoint_name == "cleanup_selected_quit":
                        require(phase["cleanup_snapshot_file"] is not None
                                and phase["cleanup_snapshot_sha256"] is not None,
                                "passed selected-quit cleanup action lacks its phase snapshot")
                else:
                    require(action["dispatch_state"] == "dispatched"
                            and action["observation_state"] == "matched",
                            "passed case receipt has an undispatched or unobserved action")
        else:
            require(failure_kind is not None,
                    "failed case receipt has no explicit failure kind")
            require(isinstance(error_text, str) and error_text.strip(),
                    "failed case receipt has no explicit error evidence")
            if failure_kind == "setup":
                if receipt["launched"] is False:
                    require(all(phase.get("launched") is False for phase in phases),
                            "prelaunch setup failure receipt contains a launched phase")
                else:
                    require(any(phase.get("status") == "setup_failed"
                                and phase.get("launched") is False for phase in phases)
                            and any(phase.get("launched") is True for phase in phases),
                            "aggregate setup failure after launch lacks mixed phase evidence")
            else:
                require(receipt["launched"] is True,
                        "non-setup failure receipt has no application launch")
        return receipt, None
    except LedgerError as error:
        return None, str(error)


def finalize_side(run_root: Path, side: str, cases: list[dict[str, Any]], list_info: dict[str, Any] | None,
                  list_error: str | None, integrity_scope: dict[str, Any] | None = None) -> dict[str, Any]:
    side_root = participant_dir(run_root, side)
    invocation = read_json(run_root / "invocation.json", "invocation")
    state_path = side_root / "state.json"
    state = read_json(state_path, f"{side} state")
    ledger_path = side_root / "ledger.json"
    ledger = read_json(ledger_path, f"{side} ledger")
    require(len(ledger.get("cases", [])) == len(cases), f"{side} ledger row count changed")
    build_command_error = None
    if state.get("build_exit") is not None:
        try:
            validate_app_build_command(run_root, invocation, side, state)
        except (LedgerError, OSError, UnicodeError, KeyError) as error:
            build_command_error = str(error)
    run_command_error, _ = validate_run_command(run_root, invocation, side, state)
    integrity_scope = integrity_scope or artifact_integrity_scope(None, cases)
    list_matches = list_info["matches"] if list_info is not None else {}
    junit_path = side_root / "junit.xml"
    junit_is_untrusted = (integrity_scope["all_untrusted"]
                          or side in integrity_scope["side_untrusted"]
                          or side in integrity_scope["junit_untrusted"])
    if junit_is_untrusted:
        junit, junit_error = {}, "JUnit/execution evidence differs from the sealed artifact inventory"
    else:
        junit, junit_error = (load_junit(junit_path, list_matches) if state.get("run_launched")
                              else ({}, "Nextest RUN was not launched"))
    rows_by_name = {row["test_name"]: row for row in ledger["cases"]}
    for case in cases:
        row = rows_by_name[case["test_name"]]
        row["junit_status"] = junit.get(case["test_name"])
        row["case_result_path"] = None
        row["case_result_sha256"] = None
        if list_info is None:
            row.update({"selected": None, "status": "incomplete", "reason": list_error or "selection unknown"})
            row.update({"reached_checkpoint_count": 0, "compared_checkpoint_count": 0, "matched_checkpoint_count": 0})
            continue
        match = list_info["matches"].get(case["test_name"])
        if match is None:
            row.update({"selected": None, "status": "incomplete", "reason": "selection unknown: LIST name is absent"})
            continue
        row["selected"] = match["selected"]
        if not match["selected"]:
            row.update({"status": "not_run", "reason": f"not selected by filter: {match['reason']}"})
            row.update({"reached_checkpoint_count": 0, "compared_checkpoint_count": 0, "matched_checkpoint_count": 0})
            continue
        path = case_receipt_path(side_root / "out", case["test_name"])
        if path.is_file() and not path.is_symlink():
            row["case_result_path"] = path.relative_to(run_root).as_posix()
            row["case_result_sha256"] = sha256_file(path)
        integrity_reason = None
        if integrity_scope["all_untrusted"]:
            integrity_reason = integrity_scope["reasons"].get("*", "artifact integrity could not be verified")
        elif side in integrity_scope["side_untrusted"]:
            integrity_reason = "participant evidence differs from the sealed artifact inventory"
        elif (side, case["case_id"]) in integrity_scope["case_untrusted"]:
            integrity_reason = "case evidence differs from the sealed artifact inventory"
        if integrity_reason:
            row.update({"status": "incomplete", "reason": integrity_reason})
            row.update({"reached_checkpoint_count": None, "compared_checkpoint_count": None,
                        "matched_checkpoint_count": None})
            continue
        if state.get("never_launched_reason"):
            row.update({
                "status": "incomplete",
                "reason": f"selected by LIST but this participant never launched: {state['never_launched_reason']}",
            })
            row.update({"reached_checkpoint_count": 0, "compared_checkpoint_count": 0, "matched_checkpoint_count": 0})
            continue
        if list_error:
            row.update({
                "status": "incomplete",
                "reason": f"selected by LIST but RUN was not admitted: {list_error}",
            })
            continue
        if run_command_error:
            row.update({
                "status": "incomplete",
                "reason": f"selected by LIST but RUN command evidence is invalid: {run_command_error}",
            })
            continue
        receipt, receipt_error = validate_case_receipt(path, case, state)
        if receipt is None:
            row.update({"status": "incomplete", "reason": receipt_error or "case receipt unavailable"})
            if not path.exists():
                row.update({"reached_checkpoint_count": 0, "compared_checkpoint_count": 0, "matched_checkpoint_count": 0})
            else:
                row.update({"reached_checkpoint_count": None, "compared_checkpoint_count": None, "matched_checkpoint_count": None})
            continue
        case_root = side_root / "out" / "v3" / case["case_id"]
        try:
            counts = copy_capture_counts(case_root, case)
        except LedgerError as error:
            row.update({"status": "incomplete", "reason": str(error)})
            row.update({"reached_checkpoint_count": None, "compared_checkpoint_count": None, "matched_checkpoint_count": None})
            continue
        row["reached_checkpoint_count"], row["compared_checkpoint_count"], row["matched_checkpoint_count"] = counts
        junit_status = junit.get(case["test_name"])
        if junit_status is None:
            row.update({"status": "incomplete", "reason": junit_error or "selected case has no JUnit testcase"})
            continue
        if receipt["status"] == "pending":
            row.update({
                "status": "incomplete",
                "reason": "case receipt remained pending when the Nextest invocation ended",
            })
            continue
        if receipt["status"] == "passed":
            if junit_status != "passed":
                row.update({"status": "incomplete", "reason": f"passed receipt disagrees with JUnit {junit_status}"})
                continue
            if case["planned_checkpoint_names"] is not None:
                planned = case["planned_checkpoint_count"]
                if counts != (planned, planned, planned):
                    row.update({"status": "incomplete", "reason": f"passed receipt has incomplete capture counts {counts}"})
                    continue
            row.update({"status": "passed", "reason": None})
        elif receipt.get("failure_kind") == "setup" and receipt.get("launched") is False:
            if junit_status not in {"failed", "error"}:
                row.update({"status": "incomplete", "reason": f"setup receipt disagrees with JUnit {junit_status}"})
                continue
            row.update({"status": "setup_failed", "reason": receipt.get("error") or "explicit Suite setup failure"})
        else:
            if junit_status not in {"failed", "error"}:
                row.update({"status": "incomplete", "reason": f"failed receipt disagrees with JUnit {junit_status}"})
                continue
            row.update({"status": "failed", "reason": receipt.get("error") or "explicit Suite case failure"})
    selected_count = sum(1 for row in ledger["cases"] if row.get("selected") is True)
    state["junit_path"] = "participants/" + side + "/junit.xml" if junit_path.is_file() else None
    state["selected_count"] = selected_count
    atomic_json(state_path, state)
    terminal = all(row.get("status") in TERMINAL_ROW_STATES for row in ledger["cases"])
    report_complete = bool(
        list_info is not None
        and not list_error
        and state.get("built") is True
        and state.get("run_launched") is True
        and state.get("run_exit") is not None
        and state.get("run_signal") is None
        and build_command_error is None
        and run_command_error is None
        and selected_count > 0
        and not junit_error
        and terminal
    )
    ledger.update({
        "selection_known": list_info is not None and not list_error,
        "selected_count": selected_count,
        "junit_sha256": sha256_file(junit_path) if junit_path.is_file() and not junit_path.is_symlink() else None,
        "junit_error": junit_error,
        "build_command_error": build_command_error,
        "run_command_error": run_command_error,
        "report_complete": report_complete,
        "completed_at": utc_now(),
    })
    atomic_json(ledger_path, ledger)
    return {"report_complete": report_complete, "selected_count": selected_count,
            "terminal_case_count": sum(row.get("status") in TERMINAL_ROW_STATES for row in ledger["cases"]),
            "case_count": len(ledger["cases"]), "states": {status: sum(row.get("status") == status for row in ledger["cases"]) for status in TERMINAL_ROW_STATES}}


def update_final_report(run_root: Path) -> None:
    invocation_path = run_root / "invocation.json"
    invocation = read_json(invocation_path, "invocation")
    if not invocation.get("inventory_available"):
        invocation.update({"status": "incomplete", "report_complete": False, "success": False, "completed_at": utc_now()})
        atomic_json(invocation_path, invocation)
        return
    invocation, cases_raw = load_run(run_root)
    context_error = None
    try:
        validate_execution_context(invocation, run_root)
    except (LedgerError, OSError, UnicodeError, KeyError) as error:
        context_error = str(error)
    dependency_error = None
    try:
        validate_dependency_fetches(run_root, invocation)
    except (LedgerError, OSError, UnicodeError, KeyError) as error:
        dependency_error = str(error)
    invocation["dependency_sources_ready"] = dependency_error is None
    invocation["dependency_sources_error"] = dependency_error
    atomic_json(invocation_path, invocation)
    integrity_scope = artifact_integrity_scope(invocation.get("artifact_integrity"), cases_raw)
    try:
        if context_error:
            raise LedgerError(context_error)
        list_info = parse_saved_nextest_list(run_root, invocation, cases_raw)
        list_error = None if invocation.get("list_valid") else invocation.get("list_error") or "LIST identity invalid"
    except (LedgerError, OSError, UnicodeError, KeyError) as error:
        list_info = None
        list_error = str(error)
    if integrity_scope["list_untrusted"]:
        list_info = None
        if list_error is None:
            reasons = [reason for path, reason in integrity_scope["reasons"].items()
                       if path == "*" or path == "nextest-list.json"
                       or path == "global/logs/nextest-list.stdout.log"
                       or path.startswith("global/commands/nextest-list.")]
            list_error = "; ".join(reasons) or "Nextest LIST evidence differs from the sealed artifact inventory"
    sides = {}
    for side in SIDE_NAMES:
        sides[side] = finalize_side(run_root, side, cases_raw, list_info, list_error, integrity_scope)
    participant_summaries = {}
    for side in SIDE_NAMES:
        state = read_json(participant_dir(run_root, side) / "state.json", f"{side} state")
        ledger = read_json(participant_dir(run_root, side) / "ledger.json", f"{side} ledger")
        side_success = bool(
            sides[side]["report_complete"]
            and state.get("run_exit") == 0
            and all(row.get("status") == "passed" for row in ledger["cases"])
            and len(ledger["cases"]) == 89
        )
        participant_summaries[side] = {**sides[side], "success": side_success,
                                       "run_exit": state.get("run_exit"), "built": state.get("built")}
    interrupted = invocation.get("interruption", {}).get("signal") is not None
    integrity_record = invocation.get("artifact_integrity")
    integrity_ok = (integrity_record is None
                    or (isinstance(integrity_record, dict) and integrity_record.get("status") == "verified"))
    pair_complete = all(item["report_complete"] for item in participant_summaries.values()) \
        and not interrupted and integrity_ok and dependency_error is None
    export_verified = invocation.get("export_verified") is True
    expected_verified = invocation.get("expected_store_verified") is True
    success = pair_complete and all(item["success"] for item in participant_summaries.values()) \
        and export_verified and expected_verified and invocation.get("interruption", {}).get("signal") is None
    invocation.update({
        "participants": participant_summaries,
        "report_complete": pair_complete,
        "success": success,
        "status": "complete" if pair_complete else "incomplete",
        "list_error": list_error,
        "completed_at": utc_now(),
    })
    atomic_json(invocation_path, invocation)


def set_invocation_value(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    path = run_root / "invocation.json"
    invocation = read_json(path, "invocation")
    allowed = {
        "export_verified", "expected_store_verified", "source_identity", "export_identity",
        "toolchain_identity", "interruption", "status", "inventory_available",
        "list_valid", "list_error", "dependency_sources_ready", "dependency_sources_error",
        "artifact_path", "artifact_manifest_sha256",
    }
    require(args.key in allowed, f"invocation field is not writable: {args.key}")
    try:
        value = json.loads(args.value)
    except json.JSONDecodeError as error:
        raise LedgerError(f"invocation value must be JSON: {error}") from error
    invocation[args.key] = value
    atomic_json(path, invocation)


def reconcile_existing(args: argparse.Namespace) -> None:
    run_root = Path(args.run_root).resolve(strict=True)
    invocation, _ = load_run(run_root)
    manifest_error = None
    try:
        parent_manifest_sha256, mismatches = verify_copied_artifact(run_root, invocation)
    except (LedgerError, OSError, UnicodeError) as error:
        parent_manifest_sha256 = None
        mismatches = {}
        manifest_error = str(error)

    previous = invocation.get("artifact_integrity")
    previous_errors: list[str] = []
    previous_paths: dict[str, str] = {}
    if isinstance(previous, dict) and previous.get("status") in {"mismatch", "unavailable"}:
        old_errors = previous.get("errors", [])
        old_paths = previous.get("mismatched_paths", {})
        if isinstance(old_errors, list) and all(isinstance(item, str) for item in old_errors):
            previous_errors = old_errors
        if isinstance(old_paths, dict) and all(isinstance(k, str) and isinstance(v, str)
                                               for k, v in old_paths.items()):
            previous_paths = old_paths
        else:
            manifest_error = manifest_error or "prior artifact integrity record is malformed"
    all_paths = {**previous_paths, **mismatches}
    all_errors = list(dict.fromkeys(previous_errors + ([manifest_error] if manifest_error else [])))
    if manifest_error:
        integrity_status = "unavailable"
    elif all_paths:
        integrity_status = "mismatch"
    elif isinstance(previous, dict) and previous.get("status") in {"mismatch", "unavailable"}:
        # A previous reconciliation may already have resealed the changed copy.
        # Its parent-link finding remains part of the evidence history.
        integrity_status = previous["status"]
    else:
        integrity_status = "verified"
    invocation["artifact_integrity"] = {
        "status": integrity_status,
        "parent_manifest_sha256": parent_manifest_sha256 or (previous.get("parent_manifest_sha256")
                                                               if isinstance(previous, dict) else None),
        "mismatched_paths": all_paths,
        "errors": all_errors,
    }
    atomic_json(run_root / "invocation.json", invocation)

    update_final_report(run_root)
    result = read_json(run_root / "invocation.json", "invocation")
    atomic_json(run_root / "reconciliation.json", {
        "schema_version": 1,
        "created_at": utc_now(),
        "mode": "reconcile-existing",
        "parent_artifact_manifest_sha256": parent_manifest_sha256,
        "artifact_integrity": result.get("artifact_integrity"),
        "report_complete": result.get("report_complete"),
        "success": result.get("success"),
        "list_error": result.get("list_error"),
        "participants": result.get("participants"),
    })
    reseal_reconciled_artifact(run_root, parent_manifest_sha256)
    if not result.get("report_complete"):
        fail(result.get("list_error") or "existing evidence is incomplete")
    if not result.get("success"):
        fail("existing evidence is complete but unsuccessful")
    print("existing evidence reconciled successfully")


def seal_artifact(args: argparse.Namespace) -> None:
    root = Path(args.artifact_root).resolve(strict=True)
    records = enumerate_tree(root)
    lines = "".join(f"{row['sha256']}  {row['path']}\n" for row in records).encode()
    manifest = {
        "schema_version": 1,
        "created_at": utc_now(),
        "file_count": len(records),
        "files_sha256": sha256_bytes(lines),
        "files": records,
    }
    atomic_json(root / "artifact-manifest.json", manifest)
    invocation = read_json(Path(args.run_root) / "invocation.json", "invocation")
    invocation["artifact_path"] = str(root)
    invocation["artifact_manifest_sha256"] = sha256_file(root / "artifact-manifest.json")
    atomic_json(Path(args.run_root) / "invocation.json", invocation)
    print(f"artifact files={len(records)} manifest_sha256={invocation['artifact_manifest_sha256']}")


def verify_copied_artifact(run_root: Path, invocation: dict[str, Any]) -> tuple[str | None, dict[str, str]]:
    execution_root_raw = invocation.get("execution_root")
    require(isinstance(execution_root_raw, str), "invocation execution root is missing")
    execution_root = Path(execution_root_raw).resolve(strict=False)
    if run_root.resolve(strict=True) == execution_root:
        return None, {}
    manifest_path = run_root / "artifact-manifest.json"
    manifest = read_json(manifest_path, "copied artifact manifest")
    base_keys = {"schema_version", "created_at", "file_count", "files_sha256", "files"}
    parent_key = "parent_artifact_manifest_sha256"
    require(isinstance(manifest, dict)
            and set(manifest) in (base_keys, base_keys | {parent_key}),
            "copied artifact manifest has an unsupported shape")
    require(type(manifest.get("schema_version")) is int and manifest["schema_version"] == 1,
            "copied artifact manifest schema is unsupported")
    parse_utc_timestamp(manifest.get("created_at"), "copied artifact manifest timestamp")
    require(type(manifest.get("file_count")) is int,
            "copied artifact file count has the wrong type")
    validate_sha256_value(manifest.get("files_sha256"), "copied artifact manifest digest")
    if parent_key in manifest:
        validate_sha256_value(manifest[parent_key], "parent artifact manifest SHA-256")
        parent_path = run_root / f"parent-artifact-manifest-{manifest[parent_key]}.json"
        require(parent_path.is_file() and not parent_path.is_symlink()
                and sha256_file(parent_path) == manifest[parent_key],
                "copied artifact parent manifest link is missing or mismatched")
    files = manifest.get("files")
    require(isinstance(files, list) and len(files) == manifest["file_count"],
            "copied artifact file inventory is malformed")
    expected_by_path: dict[str, dict[str, Any]] = {}
    folded_paths: set[str] = set()
    for row in files:
        require(isinstance(row, dict) and set(row) == {"path", "sha256", "mode"}
                and isinstance(row.get("path"), str)
                and type(row.get("mode")) is int and row["mode"] >= 0,
                "copied artifact file row has an unsupported shape")
        relative = safe_relpath(row["path"], "copied artifact file path")
        require(relative != "artifact-manifest.json", "artifact manifest must not include itself")
        folded = relative.casefold()
        require(folded not in folded_paths, f"case-alias paths in copied artifact manifest: {relative}")
        folded_paths.add(folded)
        validate_sha256_value(row.get("sha256"), "copied artifact file SHA-256")
        expected_by_path[relative] = row
    require(list(expected_by_path) == [row["path"] for row in files],
            "copied artifact manifest paths are not unique")
    lines = "".join(f"{row['sha256']}  {row['path']}\n" for row in files).encode()
    require(sha256_bytes(lines) == manifest["files_sha256"],
            "copied artifact manifest's own file-list digest is invalid")
    actual_files = [row for row in enumerate_tree(run_root)
                    if row["path"] != "artifact-manifest.json"]
    actual_by_path = {row["path"]: row for row in actual_files}
    mismatches: dict[str, str] = {}
    for relative in sorted(set(expected_by_path) | set(actual_by_path)):
        expected = expected_by_path.get(relative)
        actual = actual_by_path.get(relative)
        if expected is None:
            mismatches[relative] = "file is not present in the sealed artifact inventory"
        elif actual is None:
            mismatches[relative] = "sealed artifact file is missing from the copy"
        elif expected != actual:
            mismatches[relative] = "copied file digest or mode differs from the sealed artifact inventory"
    return sha256_file(manifest_path), mismatches


def reseal_reconciled_artifact(run_root: Path, parent_manifest_sha256: str | None) -> None:
    if parent_manifest_sha256 is None:
        return
    current_manifest = run_root / "artifact-manifest.json"
    validate_sha256_value(parent_manifest_sha256, "parent artifact manifest SHA-256")
    parent_copy = run_root / f"parent-artifact-manifest-{parent_manifest_sha256}.json"
    if parent_copy.exists() or parent_copy.is_symlink():
        require(parent_copy.is_file() and not parent_copy.is_symlink()
                and sha256_file(parent_copy) == parent_manifest_sha256,
                "existing parent artifact manifest copy differs")
    else:
        require(current_manifest.is_file() and not current_manifest.is_symlink()
                and sha256_file(current_manifest) == parent_manifest_sha256,
                "current artifact manifest changed before recovery reseal")
        shutil.copyfile(current_manifest, parent_copy)
    records = [row for row in enumerate_tree(run_root)
               if row["path"] != "artifact-manifest.json"]
    lines = "".join(f"{row['sha256']}  {row['path']}\n" for row in records).encode()
    manifest = {
        "schema_version": 1,
        "created_at": utc_now(),
        "file_count": len(records),
        "files_sha256": sha256_bytes(lines),
        "parent_artifact_manifest_sha256": parent_manifest_sha256,
        "files": records,
    }
    atomic_json(current_manifest, manifest)


def command_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    new_id = commands.add_parser("new-run-id")
    new_id.set_defaults(func=lambda args: print(uuid.uuid4()))

    init = commands.add_parser("init")
    init.add_argument("--suite-root", required=True)
    init.add_argument("--run-root", required=True)
    init.add_argument("--run-id", required=True)
    init.add_argument("--diagnostic-filter")
    init.add_argument("--execution-mode", choices=("local-guarded", "velnor-native"), required=True)
    init.add_argument("--execution-context-json", required=True)
    init.set_defaults(func=create_invocation)

    context_check = commands.add_parser("validate-execution-context")
    context_check.add_argument("--run-root", required=True)
    context_check.set_defaults(func=validate_execution_context_command)

    manifest = commands.add_parser("write-export-manifest")
    manifest.add_argument("--export-dir", required=True)
    manifest.set_defaults(func=write_export_manifest)

    verify = commands.add_parser("verify-export")
    verify.add_argument("--export-dir", required=True)
    verify.set_defaults(func=verify_export_command)

    junit = commands.add_parser("write-junit-config")
    junit.add_argument("--base-config", required=True)
    junit.add_argument("--output-config", required=True)
    junit.add_argument("--run-id", required=True)
    junit.set_defaults(func=write_junit_config)

    listing = commands.add_parser("record-list")
    listing.add_argument("--run-root", required=True)
    listing.add_argument("--list-json", required=True)
    listing.add_argument("--export-root", required=True)
    listing.add_argument("--target-root", required=True)
    listing.add_argument("--mode", choices=("normal", "diagnostic"), required=True)
    listing.set_defaults(func=record_list)

    verify_binary = commands.add_parser("verify-list-binary")
    verify_binary.add_argument("--run-root", required=True)
    verify_binary.add_argument("--target-root", required=True)
    verify_binary.set_defaults(func=verify_list_binary)

    setter = commands.add_parser("set-participant")
    setter.add_argument("--run-root", required=True)
    setter.add_argument("--side", required=True)
    setter.add_argument("--key", required=True)
    setter.add_argument("--value", required=True)
    setter.set_defaults(func=set_value)

    inv_setter = commands.add_parser("set-invocation")
    inv_setter.add_argument("--run-root", required=True)
    inv_setter.add_argument("--key", required=True)
    inv_setter.add_argument("--value", required=True)
    inv_setter.set_defaults(func=set_invocation_value)

    command = commands.add_parser("record-command")
    command.add_argument("--run-root", required=True)
    command.add_argument("--participant", choices=("global", *SIDE_NAMES), required=True)
    command.add_argument("--name", required=True)
    command.add_argument("--argv-json", required=True)
    command.add_argument("--cwd", required=True)
    command.add_argument("--started-at", required=True)
    command.add_argument("--ended-at", required=True)
    command.add_argument("--duration-seconds", type=float, required=True)
    command.add_argument("--exit-status", type=int)
    command.add_argument("--signal")
    command.add_argument("--pid", type=int)
    command.add_argument("--process-group", type=int)
    command.add_argument("--stdout")
    command.add_argument("--stderr")
    command.set_defaults(func=record_command)

    finalize = commands.add_parser("finalize")
    finalize.add_argument("--run-root", required=True)
    finalize.set_defaults(func=lambda args: update_final_report(Path(args.run_root).resolve(strict=True)))

    reconcile = commands.add_parser("reconcile-existing")
    reconcile.add_argument("--run-root", required=True)
    reconcile.set_defaults(func=reconcile_existing)

    seal = commands.add_parser("seal-artifact")
    seal.add_argument("--run-root", required=True)
    seal.add_argument("--artifact-root", required=True)
    seal.set_defaults(func=seal_artifact)
    return parser


def main() -> None:
    parser = command_parser()
    args = parser.parse_args()
    try:
        args.func(args)
    except LedgerError as error:
        fail(str(error))
    except OSError as error:
        fail(f"filesystem operation failed: {error}")


if __name__ == "__main__":
    main()
