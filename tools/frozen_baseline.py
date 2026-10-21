#!/usr/bin/env python3
"""Build the frozen CLI oracle and run its PTY suite against paired binaries."""

from __future__ import annotations

import argparse
import fcntl
from datetime import datetime, timezone
import gzip
import hashlib
import hmac
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import secrets
import select
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
from typing import Any, Callable


BASELINE_TAG = "visual-baseline"
BASELINE_TAG_OBJECT = "2718a6b8127a522a3d351d0b4bdb7542fc8da542"
BASELINE_COMMIT = "4169783ae883788186d161af09ba9c481bd84e5c"
BASELINE_TREE = "ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297"
TUISCOTTI_REV = "a1ea27f757d75d07b05499276cd932e9f8e067ad"
TEST_TARGET = "visual_baseline"
TEST_PACKAGE = "junie-tui"
APPS = ("showcase", "tablepro", "jackin-preview", "holla")
MAX_TIMEOUT = 230
PREPARE_OWNED_MAX_BUDGET = 200
PREPARE_OWNED_CLEANUP_RESERVE = 15
PREPARE_OWNED_HARD_MAX = 220
PREPARE_OWNED_TERM_GRACE = 2.0
PREPARE_LIVE_OWNED_COHORT = "frozen-20261021-serial-nonincr-r1-r2"
OWNED_PROCESS_LIST_COMMAND = "/bin/ps"
EXPECTED_SCENARIOS = 28_895
EXPECTED_ORDINARY_TESTS = 20
EXTENSION_MANIFEST = Path("tools/frozen_cli_extensions.json")
EXTENSION_SOURCE = Path("tools/frozen_cli_extensions.rs")
EXTENSION_CASE_PREFIX = "termrock_extensions::showcase::table_sort::"
EXTENSION_MANIFEST_SHA256 = "b995c00f0dc14d4e0a81d1d56140e1c10ba7936a043e0135196f8fe8421cabb8"
EXTENSION_SOURCE_SHA256 = "332bd4eac4dc65be1a1478f97177cc658113a063900d8b9c6a0ba2ddc6dc67d9"
EXTENSION_PATCH_SHA256 = "17de30ad69fea9c7f514cd6530727283637d090958020ebe13a311aad4506ff4"
REFERENCE_DECISIONS = Path("tools/frozen_cli_references.json")
REFERENCE_DECISIONS_SHA256 = "2a43fb118c268300c8bbd6439ea11bb9a24ffff03dc3c9f4601893eee26a096b"
REFERENCE_CASE = "showcase::resize::overview_grown::120x40::truecolor"
REFERENCE_TUPLE = ("showcase/resize/overview_grown/120x40/truecolor", 0, "showcase/resize/overview_grown/120x40/truecolor")
EXECUTABLE_COHORTS = Path("tools/frozen_cli_build_cohorts.json")
EXECUTABLE_COHORTS_SHA256 = "96a83bee520e148bbecf7115644d88f8057a74438029c12c4a3a83253fa0b0b4"
EXECUTABLE_COHORT_SCHEMA = "termrock.frozen-cli-build-cohorts"
EXECUTABLE_COHORT_VERSION = 1
DIAGNOSTIC_COHORT_STATUS = "reproduced-diagnostic-not-reference-approved"
CAPTURE_ELIGIBLE_COHORT_STATUS = "reviewed-capture-eligible"
COHORT_REVIEWS = Path("tools/frozen_cli_build_cohort_reviews.json")
COHORT_REVIEWS_SHA256 = "65d1343138d5c817186bbd9486e6149ef554fc0fd4b3a96bf70cb916f0883718"
COHORT_REVIEWS_SCHEMA = "termrock.frozen-cli-build-cohort-reviews"
COHORT_REVIEWS_VERSION = 2
CAPTURE_REVIEW_DECISION = Path("docs/audits/2026-10-21-fresh-baseline-capture-eligibility.json")
CAPTURE_REVIEW_DECISION_SHA256 = "446273ce18d3caaea939263b37687be5aec770cf1ab54c62110f6c945ec9c2f6"
CAPTURE_REVIEW_EVIDENCE_INDEX = Path("docs/audits/2026-10-21-fresh-baseline-capture-evidence-index.json")
CAPTURE_REVIEW_EVIDENCE_INDEX_SHA256 = "7333d762ddbbc25f7fafbc78fd49fe016358b787123dbfdabce422172121ba2c"
CAPTURE_REVIEW_CONTRACT = Path("docs/audits/2026-10-21-fresh-capture-resumable-runner-contract.json")
CAPTURE_REVIEW_CONTRACT_SHA256 = "81df800e4b16a8240acf7c569a9c5c9472fa1678c96aa78947928ca7a082ebfb"
FRESH_PLAN_SCHEMA = "termrock.frozen-cli-fresh-capture-plan"
FRESH_PLAN_VERSION = 1
FRESH_PLAN_FILE = "fresh-capture-plan.json"
FRESH_PLAN_MAX_TIMEOUT = 225
FRESH_RUN_MANIFEST_SCHEMA = "termrock.frozen-cli-fresh-run-manifest"
FRESH_RUN_MANIFEST_VERSION = 1
FRESH_RUN_MANIFEST_FILE = "run-manifest.json"
FRESH_RUNTIME_ENV_KEYS = tuple(sorted({
    "PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUSTC", "RUSTDOC", "CARGO_TARGET_DIR",
    "NEXTEST_EXPERIMENTAL_LIBTEST_JSON",
    "RUSTC_BOOTSTRAP", "CARGO_BUILD_JOBS", "CARGO_NET_OFFLINE", "CARGO_HTTP_MULTIPLEXING",
    "CC", "CXX", "AR", "CFLAGS", "CXXFLAGS", "LDFLAGS", "LIBRARY_PATH", "CPATH",
    "PKG_CONFIG_PATH", "SDKROOT", "DEVELOPER_DIR", "MACOSX_DEPLOYMENT_TARGET", "DYLD_LIBRARY_PATH",
    "LANG", "LC_ALL", "LC_CTYPE", "TZ", "TERM", "COLORTERM", "NO_COLOR", "CLICOLOR", "CLICOLOR_FORCE",
    "TERMINFO", "TERMINFO_DIRS", "TMPDIR", "TMP", "TEMP", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME",
}))
BUILD_PROVENANCE_FILE = "build-provenance.json"
BUILD_PROVENANCE_SCHEMA = "termrock.frozen-build-provenance"
BUILD_PROVENANCE_VERSION = 1
PROVENANCE_ENV_KEYS = (
    "PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUSTC",
    "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET",
    "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CC", "CXX", "AR",
    "CFLAGS", "CXXFLAGS", "LDFLAGS", "LIBRARY_PATH", "CPATH", "PKG_CONFIG_PATH",
    "SDKROOT", "DEVELOPER_DIR", "MACOSX_DEPLOYMENT_TARGET", "DYLD_LIBRARY_PATH",
    "VISUAL_BIN_DIR", "VISUAL_FULL", "VISUAL_REVIEW",
)
EXTENSION_SIZES = ((72, 20), (80, 24), (100, 30), (120, 40), (160, 50), (40, 12), (60, 18))
EXTENSION_COLORS = ("truecolor", "256", "16", "none", "nocolor")
EXPECTED_CLI_ORDINARY_TESTS = frozenset({
    "isolation::env_isolation",
    "modes::showcase_boot_sets_terminal_modes",
    "modes::tablepro_boot_sets_terminal_modes",
    "modes::holla_boot_sets_terminal_modes",
    "modes::jackin_boot_sets_terminal_modes",
    "startup_input::showcase_discards_keys_typed_before_start",
    "startup_input::tablepro_discards_keys_typed_before_start",
    "startup_resize::a_resize_queued_with_a_key_still_reaches_the_app",
    "startup_resize::a_resize_right_after_the_first_frame_repaints",
})
DEFAULT_CHUNK_SIZE = 32


class RunnerError(RuntimeError):
    pass


def validate_executable_cohorts(document: object) -> dict[str, object]:
    """Validate the additive binary-cohort registry without approving frames."""
    if not isinstance(document, dict) or set(document) != {"schema", "schema_version", "cohorts"}:
        raise RunnerError("executable cohort registry fields differ from schema version 1")
    if document["schema"] != EXECUTABLE_COHORT_SCHEMA or type(document["schema_version"]) is not int or document["schema_version"] != EXECUTABLE_COHORT_VERSION:
        raise RunnerError("unsupported executable cohort registry schema")
    cohorts = document["cohorts"]
    if not isinstance(cohorts, list) or len(cohorts) != 1:
        raise RunnerError("executable cohort registry must contain exactly the recorded cohort")
    cohort = cohorts[0]
    fields = {"id", "status", "source", "tools", "build", "binaries", "runs", "limitations"}
    if not isinstance(cohort, dict) or set(cohort) != fields:
        raise RunnerError("executable cohort fields differ from schema version 1")
    if not isinstance(cohort["id"], str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{2,127}", cohort["id"]) or cohort["status"] != DIAGNOSTIC_COHORT_STATUS:
        raise RunnerError("executable cohort is not the exact diagnostic cohort")
    source = cohort["source"]
    if not isinstance(source, dict) or set(source) != {"tag", "tag_object", "commit", "tree", "tuiscotti_rev", "cargo_lock_sha256", "rust_toolchain_sha256"}:
        raise RunnerError("executable cohort source identity fields are invalid")
    expected_source = {
        "tag": BASELINE_TAG, "tag_object": BASELINE_TAG_OBJECT, "commit": BASELINE_COMMIT,
        "tree": BASELINE_TREE, "tuiscotti_rev": TUISCOTTI_REV,
        "cargo_lock_sha256": "46c1c71d85de829b277feffc959e6e8feab99deb171b1e48521c12d5cf14c2ed",
        "rust_toolchain_sha256": "007a7e2abbd4afe192a3ef190dfa54914800621e221196747cff9e172735dc00",
    }
    if source != expected_source:
        raise RunnerError("executable cohort frozen source/tool pins changed")
    tools = cohort["tools"]
    tool_fields = {"recipe_sha256", "run1_archive_sha256", "run2_archive_sha256", "run2_comparison_sha256", "run1_provenance_sha256", "run2_provenance_sha256", "run1_live_source_sha256", "run2_live_source_sha256", "run1_stage1_receipt_sha256", "run1_stage2_receipt_sha256", "run2_stage1_receipt_sha256", "run2_stage2_receipt_sha256", "run2_sealed_manifest_sha256"}
    if not isinstance(tools, dict) or set(tools) != tool_fields:
        raise RunnerError("executable cohort evidence digest fields are invalid")
    for key, digest in tools.items():
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise RunnerError(f"executable cohort evidence digest is invalid: {key}")
    if tools["recipe_sha256"] != "849900e9c848d892824bd2b6636a17ba35d57c9be57859abf136c42f7f314bac":
        raise RunnerError("executable cohort build recipe changed")
    build = cohort["build"]
    if not isinstance(build, dict) or set(build) != {"target", "incremental", "profile", "stage_hashes_match", "build_admitted", "reference_frames_approved"}:
        raise RunnerError("executable cohort build record is invalid")
    for field in ("incremental", "stage_hashes_match", "build_admitted", "reference_frames_approved"):
        if type(build[field]) is not bool:
            raise RunnerError(f"executable cohort build field must have boolean type: {field}")
    if build != {"target": "aarch64-apple-darwin", "incremental": False, "profile": "frozen manifest profiles with targeted root-package codegen-units=256", "stage_hashes_match": True, "build_admitted": False, "reference_frames_approved": False}:
        raise RunnerError("executable cohort diagnostic build claims changed")
    binaries = cohort["binaries"]
    if not isinstance(binaries, dict) or set(binaries) != set(APPS):
        raise RunnerError("executable cohort must pin the complete application binary inventory")
    if any(not isinstance(record, dict) or set(record) != {"path", "sha256"} or record["path"] != name or not isinstance(record["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", record["sha256"]) for name, record in binaries.items()):
        raise RunnerError("executable cohort binary paths or hashes are invalid")
    expected_binaries = {name: record["sha256"] for name, record in binaries.items()}
    runs = cohort["runs"]
    run_fields = {"id", "archive_sha256", "stage1_hashes", "final_hashes", "complete_target_archive", "operational_retry"}
    if not isinstance(runs, list) or len(runs) != 2 or any(not isinstance(run, dict) or set(run) != run_fields for run in runs):
        raise RunnerError("executable cohort must contain exactly two complete run records")
    expected_run_ids = ["serial-run-1", "serial-run-2"]
    expected_archives = [tools["run1_archive_sha256"], tools["run2_archive_sha256"]]
    previous_hashes: tuple[dict[str, str], dict[str, str]] | None = None
    expected_archive_flags = [(False, True), (True, False)]
    for run, expected_id, expected_archive, (complete_target, operational_retry) in zip(runs, expected_run_ids, expected_archives, expected_archive_flags, strict=True):
        if run["id"] != expected_id or not isinstance(run["final_hashes"], dict) or not isinstance(run["stage1_hashes"], dict) or set(run["final_hashes"]) != set(APPS) or set(run["stage1_hashes"]) != set(APPS):
            raise RunnerError("executable cohort run identity or binary hashes differ")
        if any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) for hashes in (run["stage1_hashes"], run["final_hashes"]) for value in hashes.values()) or run["final_hashes"] != expected_binaries:
            raise RunnerError("executable cohort stage binary inventory is invalid")
        if previous_hashes is not None and previous_hashes != (run["stage1_hashes"], run["final_hashes"]):
            raise RunnerError("executable cohort independent runs do not reproduce identical binary stages")
        previous_hashes = (run["stage1_hashes"], run["final_hashes"])
        if run["archive_sha256"] != expected_archive:
            raise RunnerError("executable cohort run archive digest is invalid")
        if type(run["complete_target_archive"]) is not bool or type(run["operational_retry"]) is not bool:
            raise RunnerError("executable cohort run evidence flags are invalid")
        if (run["complete_target_archive"], run["operational_retry"]) != (complete_target, operational_retry):
            raise RunnerError("executable cohort archive/retry limitations were altered")
    limitations = cohort["limitations"]
    if limitations != [
        "This cohort is diagnostic and is not admitted for paired capture.",
        "Run 1 archive omits the complete controlled target tree; sealed application binaries and receipts are preserved.",
        "Run 1 required an operational prepare-live retry after an initial missing historical loader manifest; both attempts are preserved.",
        "The historical build.json loader copy is not fresh provenance and is excluded as cohort evidence.",
        "The result reproduces at one fixed physical build path on this host; it is not a hermetic or path-independent build claim.",
        "No new frozen baseline frames or checkpoint reference decisions have been approved.",
    ]:
        raise RunnerError("executable cohort limitations changed")
    return document


def load_executable_cohorts(repo: Path) -> dict[str, object]:
    path = repo / EXECUTABLE_COHORTS
    try:
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
            raise RunnerError("executable cohort registry is missing, symlinked, or non-regular")
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            opened = os.fstat(fd)
            chunks: list[bytes] = []
            while chunk := os.read(fd, 1024 * 1024):
                chunks.append(chunk)
            after_fd = os.fstat(fd)
        finally:
            os.close(fd)
        after = path.lstat()
    except OSError as error:
        raise RunnerError(f"executable cohort registry cannot be safely read: {error}") from error
    identity = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)
    if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after):
        raise RunnerError("executable cohort registry changed during read")
    payload = b"".join(chunks)
    if hashlib.sha256(payload).hexdigest() != EXECUTABLE_COHORTS_SHA256:
        raise RunnerError("executable cohort registry is unpinned")
    def strict_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise RunnerError(f"duplicate executable cohort JSON field: {key}")
            result[key] = value
        return result
    try:
        return validate_executable_cohorts(json.loads(payload, object_pairs_hook=strict_object))
    except (OSError, json.JSONDecodeError) as error:
        raise RunnerError(f"executable cohort registry cannot be read: {error}") from error


def _read_pinned_json(repo: Path, relative: Path, expected_sha256: str, label: str, *, check_work: Callable[[], None] | None = None) -> tuple[object, str]:
    path = repo / relative
    try:
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
            raise RunnerError(f"{label} is missing, symlinked, or non-regular")
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            opened = os.fstat(fd)
            chunks: list[bytes] = []
            while chunk := os.read(fd, 1024 * 1024):
                if check_work is not None:
                    check_work()
                chunks.append(chunk)
            after_fd = os.fstat(fd)
        finally:
            os.close(fd)
        after = path.lstat()
    except OSError as error:
        raise RunnerError(f"{label} cannot be safely read: {error}") from error
    identity = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)
    if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after):
        raise RunnerError(f"{label} changed during read")
    payload = b"".join(chunks)
    digest = hashlib.sha256(payload).hexdigest()
    if digest != expected_sha256:
        raise RunnerError(f"{label} is unpinned")
    def strict_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise RunnerError(f"duplicate {label} JSON field: {key}")
            result[key] = value
        return result
    try:
        return json.loads(payload, object_pairs_hook=strict_object), digest
    except json.JSONDecodeError as error:
        raise RunnerError(f"{label} JSON is invalid: {error}") from error


def _capture_review_documents(repo: Path, *, check_work: Callable[[], None] | None = None) -> tuple[dict[str, object], str, dict[str, object], str, dict[str, object], str]:
    decision, decision_sha256 = _read_pinned_json(repo, CAPTURE_REVIEW_DECISION, CAPTURE_REVIEW_DECISION_SHA256, "fresh capture eligibility decision", check_work=check_work)
    index, index_sha256 = _read_pinned_json(repo, CAPTURE_REVIEW_EVIDENCE_INDEX, CAPTURE_REVIEW_EVIDENCE_INDEX_SHA256, "fresh capture evidence index", check_work=check_work)
    contract, contract_sha256 = _read_pinned_json(repo, CAPTURE_REVIEW_CONTRACT, CAPTURE_REVIEW_CONTRACT_SHA256, "fresh capture resumable-runner contract", check_work=check_work)
    if not isinstance(decision, dict) or decision.get("schema") != "termrock.independent-executable-cohort-capture-review" or type(decision.get("schema_version")) is not int or decision["schema_version"] != 1:
        raise RunnerError("fresh capture eligibility decision schema is unsupported")
    if decision.get("decision") != "eligible-for-fresh-baseline-capture-and-baseline-replay-only" or decision.get("scope") != "executable-cohort-capture-eligibility":
        raise RunnerError("fresh capture eligibility decision has an unexpected scope")
    permitted = decision.get("permitted")
    forbidden = decision.get("not_permitted")
    if not isinstance(permitted, dict) or permitted != {
        "fresh_baseline_capture": True,
        "baseline_replay_against_fresh_immutable_capture": True,
        "collect_reference_conflict_evidence_without_prior_tuple_approval": True,
    } or any(type(value) is not bool or value is not True for value in permitted.values()):
        raise RunnerError("fresh capture decision does not authorize only the recorded baseline phases")
    if not isinstance(forbidden, dict) or forbidden != {
        "candidate_comparison_eligible": False,
        "checkpoint_reference_approval": "none",
        "old_v1_exception_transferred": False,
        "product_acceptance": False,
        "unconditional_general_build_admission": False,
    }:
        raise RunnerError("fresh capture decision contains forbidden candidate or reference approval")
    if any(type(forbidden[key]) is not bool for key in ("candidate_comparison_eligible", "old_v1_exception_transferred", "product_acceptance", "unconditional_general_build_admission")):
        raise RunnerError("fresh capture decision boolean claims have invalid types")
    if decision.get("legacy_decision", {}).get("path") != REFERENCE_DECISIONS.as_posix() or decision.get("legacy_decision", {}).get("sha256") != REFERENCE_DECISIONS_SHA256:
        raise RunnerError("fresh capture decision is not isolated from the pinned legacy checkpoint decision")
    if decision.get("current_state") != {
        "production_review_registry_empty": True,
        "no_capture_executed_in_this_review": True,
        "candidate_acceptance_blocked": True,
    }:
        raise RunnerError("fresh capture decision historical-state record differs from the reviewed state")
    if not isinstance(index, dict) or index.get("schema") != "termrock.frozen-cli-capture-eligibility-evidence-index" or type(index.get("schema_version")) is not int or index["schema_version"] != 1:
        raise RunnerError("fresh capture evidence index schema is unsupported")
    if set(index) != {"schema", "schema_version", "record_kind", "cohort_id", "decision_sha256", "reviewed_integration_commit", "source", "binary_sha256", "artifacts", "external_build_evidence", "capture_scope", "historical_state_note", "evidence_limitations"}:
        raise RunnerError("fresh capture evidence index fields differ from schema version 1")
    if index.get("decision_sha256") != decision_sha256 or index.get("cohort_id") != decision.get("cohort_id") or index.get("reviewed_integration_commit") != decision.get("integration_commit_reviewed") or index.get("source") != decision.get("source") or index.get("binary_sha256") != decision.get("binary_sha256"):
        raise RunnerError("fresh capture evidence index is not bound to the reviewed decision")
    artifacts = index.get("artifacts")
    if not isinstance(artifacts, dict) or not artifacts:
        raise RunnerError("fresh capture evidence index has no artifact map")
    for key, item in artifacts.items():
        if not isinstance(key, str) or not isinstance(item, dict) or set(item) != {"repository_path", "source_path", "sha256", "bytes", "copied_byte_for_byte"}:
            raise RunnerError("fresh capture evidence index artifact entry is malformed")
        relative = PurePosixPath(item["repository_path"]) if isinstance(item["repository_path"], str) else PurePosixPath("/")
        if relative.is_absolute() or len(relative.parts) < 2 or ".." in relative.parts or not isinstance(item["source_path"], str) or not Path(item["source_path"]).is_absolute() or not isinstance(item["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", item["sha256"]) or type(item["bytes"]) is not int or item["bytes"] < 1 or item["copied_byte_for_byte"] is not True:
            raise RunnerError("fresh capture evidence index artifact identity is invalid")
        _artifact, artifact_digest = _read_pinned_json(repo, Path(relative.as_posix()), item["sha256"], f"fresh capture evidence artifact {key}", check_work=check_work)
        if (repo / relative).stat().st_size != item["bytes"] or artifact_digest != item["sha256"]:
            raise RunnerError(f"fresh capture evidence artifact differs from its index: {key}")
    for key, path, digest in (
        ("capture_eligibility_decision", CAPTURE_REVIEW_DECISION, decision_sha256),
        ("evidence_recheck", Path("docs/audits/2026-10-21-fresh-baseline-capture-evidence-recheck.json"), decision.get("evidence", {}).get("recheck_sha256")),
        ("resumable_runner_contract", CAPTURE_REVIEW_CONTRACT, contract_sha256),
    ):
        item = artifacts.get(key)
        if not isinstance(item, dict) or item.get("repository_path") != path.as_posix() or item.get("sha256") != digest or item.get("copied_byte_for_byte") is not True:
            raise RunnerError(f"fresh capture evidence index artifact binding is invalid: {key}")
    _recheck, recheck_digest = _read_pinned_json(
        repo,
        Path("docs/audits/2026-10-21-fresh-baseline-capture-evidence-recheck.json"),
        str(decision.get("evidence", {}).get("recheck_sha256")),
        "fresh capture evidence recheck",
        check_work=check_work,
    )
    if recheck_digest != decision.get("evidence", {}).get("recheck_sha256"):
        raise RunnerError("fresh capture evidence recheck digest differs from the decision")
    if not isinstance(contract, dict) or contract.get("schema") != "termrock.reviewed-fresh-capture-resumable-contract" or type(contract.get("schema_version")) is not int or contract["schema_version"] != 1 or contract.get("status") != "proposed-implementation-contract-not-executed":
        raise RunnerError("fresh capture runner contract is not the reviewed unexecuted contract")
    if contract.get("basis_commit") != decision.get("integration_commit_reviewed") or contract.get("inventory", {}).get("visual_scenarios_total") != EXPECTED_SCENARIOS + 174 or contract.get("inventory", {}).get("ordinary_checks") != EXPECTED_ORDINARY_TESTS:
        raise RunnerError("fresh capture runner contract differs from the reviewed source or inventory")
    scope = index.get("capture_scope")
    expected_scope = {
        "fresh_baseline_capture": True,
        "baseline_replay": True,
        "collect_reference_conflicts_without_prior_tuple_approval": True,
        "candidate_comparison_eligible": False,
        "checkpoint_reference_approval": "none",
        "product_acceptance": False,
    }
    if scope != expected_scope or any(type(scope[key]) is not bool for key in ("fresh_baseline_capture", "baseline_replay", "collect_reference_conflicts_without_prior_tuple_approval", "candidate_comparison_eligible", "product_acceptance")):
        raise RunnerError("fresh capture evidence index grants an invalid scope")
    return decision, decision_sha256, index, index_sha256, contract, contract_sha256


def validate_cohort_reviews(document: object, registry_sha256: str, cohorts: dict[str, object], decision: dict[str, object], decision_sha256: str, evidence_index_sha256: str) -> dict[str, dict[str, object]]:
    if not isinstance(document, dict) or set(document) != {"schema", "schema_version", "reviews"}:
        raise RunnerError("cohort review registry fields differ from schema version 1")
    if document["schema"] != COHORT_REVIEWS_SCHEMA or type(document["schema_version"]) is not int or document["schema_version"] != COHORT_REVIEWS_VERSION:
        raise RunnerError("unsupported cohort review registry schema")
    reviews = document["reviews"]
    if not isinstance(reviews, list):
        raise RunnerError("cohort review registry reviews must be an array")
    by_id: dict[str, dict[str, object]] = {}
    for review in reviews:
        fields = {"cohort_id", "status", "cohort_registry_sha256", "binary_sha256", "source", "evidence", "decision_sha256", "evidence_index_sha256", "integration_commit_reviewed", "fresh_baseline_capture_eligible", "baseline_replay_eligible", "candidate_comparison_eligible", "checkpoint_reference_approval", "product_acceptance_eligible"}
        if not isinstance(review, dict) or set(review) != fields:
            raise RunnerError("cohort review fields differ from schema version 1")
        cohort_id = review["cohort_id"]
        cohort = cohorts.get(cohort_id) if isinstance(cohort_id, str) else None
        if cohort is None or cohort_id in by_id:
            raise RunnerError("cohort review identifies an unknown or duplicate cohort")
        if review["status"] != CAPTURE_ELIGIBLE_COHORT_STATUS or review["cohort_registry_sha256"] != registry_sha256:
            raise RunnerError("cohort review is not bound to the pinned capture-eligible registry")
        if (review["decision_sha256"], review["evidence_index_sha256"], review["integration_commit_reviewed"]) != (decision_sha256, evidence_index_sha256, decision.get("integration_commit_reviewed")):
            raise RunnerError("cohort review does not bind the pinned decision, evidence index, and reviewed integration commit")
        expected_hashes = {app: cohort["binaries"][app]["sha256"] for app in APPS}
        if review["binary_sha256"] != expected_hashes:
            raise RunnerError("cohort review binary inventory differs from the pinned cohort")
        if review["source"] != cohort["source"]:
            raise RunnerError("cohort review source identity differs from the pinned cohort")
        if decision.get("cohort_id") != cohort_id or decision.get("binary_sha256") != expected_hashes or decision.get("source") != cohort["source"]:
            raise RunnerError("pinned independent decision source or binary identity differs from the reviewed cohort")
        evidence = review["evidence"]
        expected_evidence = [{"reviewer": decision.get("reviewer"), "scope": decision.get("scope"), "report_sha256": decision_sha256}]
        if evidence != expected_evidence:
            raise RunnerError("capture eligibility evidence differs from the pinned independent decision")
        reviewers: set[str] = set()
        for item in evidence:
            if not isinstance(item, dict) or set(item) != {"reviewer", "scope", "report_sha256"}:
                raise RunnerError("cohort review evidence fields are invalid")
            if not isinstance(item["reviewer"], str) or not item["reviewer"].strip() or item["reviewer"] in reviewers:
                raise RunnerError("cohort review evidence has an empty or duplicate reviewer")
            if item["scope"] != "executable-cohort-capture-eligibility" or not isinstance(item["report_sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", item["report_sha256"]):
                raise RunnerError("cohort review evidence scope or digest is invalid")
            reviewers.add(item["reviewer"])
        for field in ("fresh_baseline_capture_eligible", "baseline_replay_eligible", "candidate_comparison_eligible", "product_acceptance_eligible"):
            if type(review[field]) is not bool:
                raise RunnerError(f"cohort review {field} claim has an invalid boolean type")
        if review["fresh_baseline_capture_eligible"] is not True or review["baseline_replay_eligible"] is not True:
            raise RunnerError("cohort review does not authorize fresh capture and baseline replay")
        if type(review["candidate_comparison_eligible"]) is not bool or review["candidate_comparison_eligible"] is not False:
            raise RunnerError("fresh baseline capture review cannot approve candidate comparison")
        if review["checkpoint_reference_approval"] != "none" or review["product_acceptance_eligible"] is not False:
            raise RunnerError("fresh baseline capture review cannot approve checkpoint references")
        by_id[cohort_id] = review
    return by_id


def load_cohort_review_registry(repo: Path, cohort_document: dict[str, object], cohort_registry_sha256: str, *, check_work: Callable[[], None] | None = None) -> tuple[dict[str, dict[str, object]], str]:
    document, digest = _read_pinned_json(repo, COHORT_REVIEWS, COHORT_REVIEWS_SHA256, "cohort review registry", check_work=check_work)
    decision, decision_digest, _index, index_digest, _contract, _contract_digest = _capture_review_documents(repo, check_work=check_work)
    cohorts = {cohort["id"]: cohort for cohort in cohort_document["cohorts"]}
    if decision.get("cohort_registry_sha256") != cohort_registry_sha256:
        raise RunnerError("fresh capture decision is bound to a different executable cohort registry")
    return validate_cohort_reviews(document, cohort_registry_sha256, cohorts, decision, decision_digest, index_digest), digest


def _verify_cohort_binaries(cohort: dict[str, object], binary_root: Path, context: str, *, check_work: Callable[[], None] | None = None) -> dict[str, Path]:
    if cohort.get("status") != DIAGNOSTIC_COHORT_STATUS:
        raise RunnerError(f"{context} verification only accepts the recorded diagnostic cohort")
    expected = cohort.get("binaries")
    if not isinstance(expected, dict) or set(expected) != set(APPS):
        raise RunnerError(f"{context} binary inventory is incomplete")
    verified: dict[str, Path] = {}
    for app in APPS:
        if check_work is not None:
            check_work()
        record = expected[app]
        if not isinstance(record, dict) or set(record) != {"path", "sha256"}:
            raise RunnerError(f"{context} binary path is invalid: {app}")
        relative = PurePosixPath(record["path"])
        if relative.is_absolute() or len(relative.parts) != 1 or relative.parts[0] in ("", ".", ".."):
            raise RunnerError(f"{context} binary path escapes the supplied root: {app}")
        if record["path"] != app:
            raise RunnerError(f"{context} binary path is invalid: {app}")
        path = binary_root / relative.parts[0]
        actual, _identity = stable_private_binary(path, f"{context} binary {app}", check_work=check_work) if check_work is not None else stable_private_binary(path, f"{context} binary {app}")
        if actual != record["sha256"]:
            raise RunnerError(f"{context} binary hash mismatch: {app}")
        verified[app] = path
    return verified


def verify_diagnostic_cohort_binaries(repo: Path, cohort_id: str, binary_root: Path) -> dict[str, Path]:
    """Verify pinned diagnostic binaries; callers cannot supply cohort claims."""
    document = load_executable_cohorts(repo)
    cohort = next((item for item in document["cohorts"] if item["id"] == cohort_id), None)
    if cohort is None or cohort["status"] != DIAGNOSTIC_COHORT_STATUS:
        raise RunnerError("binary verification only accepts the pinned diagnostic cohort")
    return _verify_cohort_binaries(cohort, binary_root, "diagnostic cohort")


def require_capture_eligible_cohort(repo: Path, cohort_id: str, binary_root: Path, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
    """Resolve a separately reviewed cohort for fresh baseline capture only."""
    cohort_document, registry_sha256 = _read_pinned_json(repo, EXECUTABLE_COHORTS, EXECUTABLE_COHORTS_SHA256, "executable cohort registry", check_work=check_work)
    cohort_document = validate_executable_cohorts(cohort_document)
    review_map, reviews_sha256 = load_cohort_review_registry(repo, cohort_document, registry_sha256, check_work=check_work)
    cohort = next((item for item in cohort_document["cohorts"] if item["id"] == cohort_id), None)
    review = review_map.get(cohort_id)
    if cohort is None or review is None:
        raise RunnerError("executable cohort has no separately reviewed fresh-capture eligibility")
    binaries = _verify_cohort_binaries(cohort, binary_root, "capture-eligible cohort", check_work=check_work)
    return {
        "cohort_id": cohort_id,
        "cohort_registry_sha256": registry_sha256,
        "review_registry_sha256": reviews_sha256,
        "status": review["status"],
        "decision_sha256": review["decision_sha256"],
        "evidence_index_sha256": review["evidence_index_sha256"],
        "source": cohort["source"],
        "tools": cohort["tools"],
        "binaries": {app: {"path": str(path.resolve()), "sha256": cohort["binaries"][app]["sha256"]} for app, path in binaries.items()},
        "fresh_baseline_capture_eligible": True,
        "baseline_replay_eligible": True,
        "candidate_comparison_eligible": False,
        "checkpoint_reference_approval": "none",
        "product_acceptance_eligible": False,
        "acceptance_complete": False,
    }


def run(command: list[str], *, cwd: Path, timeout: int, env: dict[str, str] | None = None, quiet: bool = False, include_stderr: bool = False, stdout_path: Path | None = None, stderr_path: Path | None = None) -> str:
    if not 1 <= timeout <= MAX_TIMEOUT:
        raise RunnerError(f"timeout must be 1..{MAX_TIMEOUT} seconds")
    print("+", " ".join(command), flush=True)
    process = subprocess.Popen(
        command,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )
    try:
        output, errors = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired as error:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            output, errors = process.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            output, errors = process.communicate()
        if stdout_path is not None:
            stdout_path.parent.mkdir(parents=True, exist_ok=True)
            stdout_path.write_text(output)
        if stderr_path is not None:
            stderr_path.parent.mkdir(parents=True, exist_ok=True)
            stderr_path.write_text(errors)
        raise RunnerError(f"command timed out after {timeout}s: {' '.join(command)}\n{output}\n{errors}") from error
    if stdout_path is not None:
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_text(output)
    if stderr_path is not None:
        stderr_path.parent.mkdir(parents=True, exist_ok=True)
        stderr_path.write_text(errors)
    if not quiet:
        print(errors, end="", file=sys.stderr, flush=True)
        print(output, end="", flush=True)
    if process.returncode:
        detail = (output + "\n" + errors)[-6000:]
        raise RunnerError(f"command exited {process.returncode}: {' '.join(command)}\n{detail}")
    return output + ("\n" + errors if include_stderr else "")


def _owned_group_members(pgid: int, anchor_pid: int, timeout: float) -> set[int]:
    """Observe one anchored process group; caller must retain its unreaped anchor."""
    if timeout <= 0:
        raise RunnerError("owned process-group observation exceeded the cleanup deadline")
    try:
        result = subprocess.run(
            [OWNED_PROCESS_LIST_COMMAND, "-axo", "pid=,pgid=,stat="],
            check=False,
            capture_output=True,
            text=True,
            timeout=min(timeout, 1.0),
            start_new_session=True,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise RunnerError(f"could not verify anchored process-group membership: {error}") from error
    if result.returncode:
        raise RunnerError(f"process-group membership command failed ({result.returncode})")
    members: set[int] = set()
    for line in result.stdout.splitlines():
        fields = line.split()
        if len(fields) != 3:
            raise RunnerError("process-group membership output was malformed")
        try:
            pid, observed_pgid = map(int, fields[:2])
        except ValueError:
            raise RunnerError("process-group membership output was malformed") from None
        if observed_pgid == pgid:
            if pid == anchor_pid and fields[2].startswith("Z"):
                raise RunnerError("owned process-group anchor exited before cleanup verification")
            if not fields[2].startswith("Z"):
                members.add(pid)
    if anchor_pid not in members or pgid != anchor_pid:
        raise RunnerError("owned process-group anchor disappeared before cleanup verification")
    return members


def _write_owned_status(path: Path, document: dict[str, object]) -> None:
    payload = json.dumps(document, sort_keys=True, separators=(",", ":")).encode()
    if len(payload) > 4096:
        raise RunnerError("owned subprocess status exceeded its fixed size limit")
    temporary = path.with_suffix(".tmp")
    with temporary.open("xb") as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)


def _owned_broker_main(spec_path: Path, control_fd: int) -> int:
    """Launch one command in this anchored session and await parent cleanup."""
    term_requested = False

    def record_term(_signum: int, _frame: object) -> None:
        nonlocal term_requested
        term_requested = True

    signal.signal(signal.SIGTERM, record_term)
    signal.signal(signal.SIGINT, record_term)
    ready_path: Path | None = None
    result_path: Path | None = None
    process: subprocess.Popen[bytes] | None = None
    command_spawn_pending = False
    status: dict[str, object]
    try:
        spec = json.loads(spec_path.read_text())
        if not isinstance(spec, dict) or set(spec) != {"argv", "cwd", "stdout_fd", "stderr_fd", "status_path", "ready_path"}:
            raise RunnerError("owned subprocess specification fields were invalid")
        argv, cwd = spec["argv"], spec["cwd"]
        stdout_fd, stderr_fd = spec["stdout_fd"], spec["stderr_fd"]
        result_path = Path(spec["status_path"])
        ready_path = Path(spec["ready_path"])
        if (
            not isinstance(argv, list)
            or not argv
            or any(not isinstance(part, str) or "\0" in part for part in argv)
            or not isinstance(cwd, str)
            or type(stdout_fd) is not int
            or type(stderr_fd) is not int
            or stdout_fd < 0
            or stderr_fd < 0
        ):
            raise RunnerError("owned subprocess specification values were invalid")
        _write_owned_status(ready_path, {"schema": 1, "anchor_pid": os.getpid()})
        ready, _, _ = select.select([control_fd], [], [], 0)
        if ready and os.read(control_fd, 1) == b"":
            return 2
        started = time.monotonic()
        command_spawn_pending = True
        process = subprocess.Popen(
            argv,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=stdout_fd,
            stderr=stderr_fd,
            close_fds=True,
            start_new_session=False,
        )
        command_spawn_pending = False
        # Caught handlers reset to SIG_DFL on exec. Thus the broker can remain
        # an anchor while the owned command still receives ordinary SIGTERM.
        if term_requested:
            # A TERM can arrive while Popen constructs the child. Re-send after
            # registration so the newly created child cannot miss cancellation.
            os.killpg(os.getpid(), signal.SIGTERM)
        while process.poll() is None:
            ready, _, _ = select.select([control_fd], [], [], 0.05)
            if ready and os.read(control_fd, 1) == b"":
                _owned_broker_orphan_cleanup(process)
                return 2
        status = {"schema": 1, "returncode": process.returncode, "elapsed_seconds": time.monotonic() - started}
    except BaseException as error:
        status = {
            "schema": 1,
            "returncode": None,
            "error": f"{type(error).__name__}: {str(error)[:512]}",
        }
        if command_spawn_pending and isinstance(error, OSError):
            status["spawn_failure"] = True
    try:
        if result_path is None:
            raise RunnerError("owned subprocess result path was not established")
        _write_owned_status(result_path, status)
        while True:
            ready, _, _ = select.select([control_fd], [], [], 0.1)
            if not ready:
                continue
            command = os.read(control_fd, 1)
            if command == b"R":
                return 0
            if command == b"":
                _owned_broker_orphan_cleanup(process)
                return 2
            raise RunnerError("owned subprocess parent sent an invalid release command")
    except BaseException:
        _owned_broker_orphan_cleanup(process)
        return 2


def _owned_broker_orphan_cleanup(process: subprocess.Popen[bytes] | None) -> None:
    """Fail closed on parent loss; never claim descendant cleanup as success."""
    anchor_pid = os.getpid()
    deadline = time.monotonic() + PREPARE_OWNED_TERM_GRACE
    term_sent = False
    while True:
        if process is not None:
            process.poll()
        remaining = deadline - time.monotonic()
        try:
            members = _owned_group_members(anchor_pid, anchor_pid, max(0.01, remaining))
        except RunnerError:
            # Observation failure cannot release the live command. This broker
            # is its own session and process-group leader, so while that
            # invariant still holds the exact group id is anchored against
            # reuse. TERM gets a bounded cooperative window; KILL is the
            # fail-closed last resort and intentionally kills this broker too.
            if os.getpgrp() != anchor_pid:
                raise RunnerError("owned broker lost its process-group anchor; cleanup cannot be verified")
            try:
                os.killpg(anchor_pid, signal.SIGTERM)
            except ProcessLookupError as error:
                raise RunnerError("owned broker group disappeared during fail-closed cleanup") from error
            term_deadline = time.monotonic() + PREPARE_OWNED_TERM_GRACE
            while True:
                grace_remaining = term_deadline - time.monotonic()
                if grace_remaining <= 0:
                    break
                time.sleep(min(0.02, grace_remaining))
            if os.getpgrp() != anchor_pid:
                raise RunnerError("owned broker lost its process-group anchor before final cleanup")
            try:
                os.killpg(anchor_pid, signal.SIGKILL)
            except ProcessLookupError as error:
                raise RunnerError("owned broker group disappeared before fail-closed KILL") from error
            return
        if members == {anchor_pid}:
            if process is not None:
                try:
                    process.wait(timeout=0.5)
                except subprocess.TimeoutExpired:
                    pass
            return
        if not term_sent:
            try:
                os.killpg(anchor_pid, signal.SIGTERM)
            except ProcessLookupError:
                return
            term_sent = True
        elif remaining <= 0:
            try:
                os.killpg(anchor_pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            return
        time.sleep(min(0.02, max(0.001, remaining)))


class PrepareExecution:
    """Opt-in prepare lifecycle deadline and anchored subprocess ownership.

    This scope is for the canonical fixed Git commands. A child that starts a
    new session escapes the owned process group and is outside this contract.
    """

    def __init__(self, work_budget: int, *, cleanup_reserve: int = PREPARE_OWNED_CLEANUP_RESERVE):
        if type(work_budget) is not int or not 1 <= work_budget <= PREPARE_OWNED_MAX_BUDGET:
            raise RunnerError(f"owned prepare budget must be 1..{PREPARE_OWNED_MAX_BUDGET} seconds")
        if cleanup_reserve != PREPARE_OWNED_CLEANUP_RESERVE:
            raise RunnerError("owned prepare cleanup reserve is fixed")
        if work_budget + cleanup_reserve > PREPARE_OWNED_HARD_MAX:
            raise RunnerError("owned prepare budget exceeds its hard supervisor deadline")
        self.started = time.monotonic()
        self.work_budget = work_budget
        self.work_deadline = self.started + work_budget
        self.hard_deadline = self.work_deadline + cleanup_reserve
        self.cancel_signal: int | None = None
        self._previous_handlers: dict[int, object] = {}
        self._children: dict[int, subprocess.Popen[bytes]] = {}
        self._retired_anchors: set[int] = set()

    def __enter__(self) -> PrepareExecution:
        if threading.current_thread() is not threading.main_thread():
            raise RunnerError("owned prepare signal handling requires the main thread")
        try:
            for signum in (signal.SIGINT, signal.SIGTERM):
                self._previous_handlers[signum] = signal.getsignal(signum)
                signal.signal(signum, self._record_cancellation)
            self.check_work()
        except BaseException:
            for signum, handler in self._previous_handlers.items():
                signal.signal(signum, handler)
            self._previous_handlers.clear()
            raise
        return self

    def __exit__(self, _exc_type: object, _exc: object, _traceback: object) -> None:
        for signum, handler in self._previous_handlers.items():
            signal.signal(signum, handler)
        self._previous_handlers.clear()

    def _record_cancellation(self, signum: int, _frame: object) -> None:
        if self.cancel_signal is None:
            self.cancel_signal = signum

    def check_work(self) -> None:
        if self.cancel_signal is not None:
            raise RunnerError(f"owned prepare cancelled by signal {self.cancel_signal}")
        if time.monotonic() >= self.work_deadline:
            raise RunnerError("owned prepare work deadline expired")

    def _remaining_hard(self) -> float:
        remaining = self.hard_deadline - time.monotonic()
        if remaining <= 0:
            raise RunnerError("owned prepare cleanup deadline expired")
        return remaining

    @property
    def remaining_work(self) -> float:
        self.check_work()
        return self.work_deadline - time.monotonic()

    def run(self, command: list[str], *, cwd: Path, stdout_file: object | None = None, capture_stdout: bool = True, env: dict[str, str] | None = None, quiet: bool = False, show_command: bool = True, command_timeout: float | None = None, optional_failure: bool = False) -> tuple[str, str] | None:
        self.check_work()
        if command_timeout is not None and (not isinstance(command_timeout, (int, float)) or command_timeout <= 0):
            raise RunnerError("owned command timeout must be positive")
        if command_timeout is not None and command_timeout > 12:
            raise RunnerError("owned optional probe timeout exceeds 12 seconds")
        if not command or any(not isinstance(part, str) or "\0" in part for part in command):
            raise RunnerError("owned prepare command argv was invalid")
        status_dir = Path(tempfile.mkdtemp(prefix="termrock-prepare-owned-"))
        stdout_tmp = tempfile.NamedTemporaryFile(mode="w+b", prefix="stdout-", dir=status_dir, delete=False) if stdout_file is None else stdout_file
        stderr_tmp = tempfile.NamedTemporaryFile(mode="w+b", prefix="stderr-", dir=status_dir, delete=False)
        owns_stdout = stdout_file is None
        control_read, control_write = os.pipe()
        os.set_inheritable(control_read, True)
        status_path = status_dir / "status.json"
        ready_path = status_dir / "anchor-ready.json"
        spec_path = status_dir / "spec.json"
        spec = {
            "argv": command,
            "cwd": str(cwd),
            "stdout_fd": stdout_tmp.fileno(),
            "stderr_fd": stderr_tmp.fileno(),
            "status_path": str(status_path),
            "ready_path": str(ready_path),
        }
        spec_path.write_text(json.dumps(spec, separators=(",", ":")))
        broker_argv = [sys.executable, str(Path(__file__).resolve()), "--_owned-prepare-broker", str(spec_path), str(control_read)]
        broker: subprocess.Popen[bytes] | None = None
        registered = False
        released = False
        succeeded = False
        cleanup_error: RunnerError | None = None
        result: dict[str, object] | None = None
        local_timeout_hit = False
        command_deadline = min(self.work_deadline, time.monotonic() + command_timeout) if command_timeout is not None else self.work_deadline
        try:
            broker = subprocess.Popen(
                broker_argv,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                close_fds=True,
                pass_fds=(control_read, stdout_tmp.fileno(), stderr_tmp.fileno()),
                start_new_session=True,
                env=env,
            )
            # Signal handlers only record cancellation, so this registration
            # immediately follows Popen without a signal-interruptible gap.
            self._children[broker.pid] = broker
            registered = True
            os.close(control_read)
            control_read = -1
            if show_command:
                print("+", " ".join(command), flush=True)
            while not ready_path.exists():
                if self.cancel_signal is not None or time.monotonic() >= self.work_deadline:
                    # Before the anchor-ready receipt, EOF makes the broker
                    # exit before it starts the Git child. No group operation
                    # is authorized until the receipt exists.
                    os.close(control_write)
                    control_write = -1
                    try:
                        broker.wait(timeout=self._remaining_hard())
                    except subprocess.TimeoutExpired as error:
                        raise RunnerError("owned prepare broker did not stop before its anchor became ready") from error
                    self._retired_anchors.add(broker.pid)
                    if self.cancel_signal is not None:
                        raise RunnerError(f"owned prepare cancelled by signal {self.cancel_signal}")
                    raise RunnerError("owned prepare work deadline expired")
                if time.monotonic() >= command_deadline:
                    os.close(control_write)
                    control_write = -1
                    try:
                        broker.wait(timeout=self._remaining_hard())
                    except subprocess.TimeoutExpired as error:
                        raise RunnerError("owned prepare broker did not stop after local probe timeout") from error
                    self._retired_anchors.add(broker.pid)
                    if broker.returncode != 2:
                        raise RunnerError("owned prepare broker failed before anchor readiness")
                    if status_path.exists():
                        status = self._read_status(status_path)
                        if not (status.get("spawn_failure") is True and status.get("returncode") is None):
                            raise RunnerError("owned prepare broker reported an error before anchor readiness")
                    if self.cancel_signal is not None:
                        raise RunnerError(f"owned prepare cancelled by signal {self.cancel_signal}")
                    if time.monotonic() >= self.work_deadline:
                        raise RunnerError("owned prepare work deadline expired")
                    if optional_failure:
                        self.check_work()
                        succeeded = True
                        return None
                    raise RunnerError("owned command timed out before its anchor became ready")
                if not self._anchor_exists(broker.pid):
                    raise RunnerError("owned prepare broker exited before its anchor-ready receipt")
                time.sleep(min(0.01, max(0.0, self.work_deadline - time.monotonic())))
            while not status_path.exists():
                if self.cancel_signal is not None or time.monotonic() >= self.work_deadline:
                    break
                if time.monotonic() >= command_deadline:
                    local_timeout_hit = command_timeout is not None
                    break
                if not self._anchor_exists(broker.pid):
                    break
                time.sleep(min(0.02, max(0.0, self.work_deadline - time.monotonic())))
            if status_path.exists():
                result = self._read_status(status_path)
            try:
                self._cleanup_group(broker, control_write, status_path)
                released = True
                control_write = -1
            except RunnerError as error:
                cleanup_error = error
            if cleanup_error is not None:
                raise cleanup_error
            # Cleanup may be the point where a delayed broker failure becomes
            # observable. Classify its final receipt before treating a local
            # optional-probe timeout as an ordinary missing result.
            if status_path.exists():
                result = self._read_status(status_path)
            if self.cancel_signal is not None:
                raise RunnerError(f"owned prepare cancelled by signal {self.cancel_signal}")
            if time.monotonic() >= self.work_deadline:
                raise RunnerError("owned prepare work deadline expired")
            if result is None:
                raise RunnerError("owned prepare broker exited without a bounded status record")
            if result.get("returncode") is None:
                if optional_failure:
                    if result.get("spawn_failure") is True:
                        self.check_work()
                        succeeded = True
                        return None
                    raise RunnerError(f"owned prepare broker failed before command completion: {result.get('error', 'unknown broker error')}")
                raise RunnerError(f"owned prepare command could not start: {result.get('error', 'unknown broker error')}")
            returncode = result.get("returncode")
            if type(returncode) is not int:
                raise RunnerError("owned prepare broker returned no command status")
            if local_timeout_hit:
                if optional_failure:
                    self.check_work()
                    succeeded = True
                    return None
                raise RunnerError("owned command timed out")
            stdout_tmp.seek(0)
            stderr_tmp.seek(0)
            output = stdout_tmp.read().decode(errors="replace") if capture_stdout else ""
            errors = stderr_tmp.read().decode(errors="replace")
            if not quiet:
                print(errors, end="", file=sys.stderr, flush=True)
                print(output, end="", flush=True)
            if returncode:
                if optional_failure:
                    self.check_work()
                    succeeded = True
                    return None
                detail = (output + "\n" + errors)[-6000:]
                raise RunnerError(f"command exited {returncode}: {' '.join(command)}\n{detail}")
            succeeded = True
            self.check_work()
            return output, errors
        finally:
            if broker is not None and registered and not released:
                try:
                    if broker.pid not in self._retired_anchors:
                        self._cleanup_group(broker, control_write, status_path)
                    released = True
                    control_write = -1
                except RunnerError:
                    pass
            if control_read >= 0:
                os.close(control_read)
            if control_write >= 0:
                os.close(control_write)
                control_write = -1
            if broker is not None and registered:
                try:
                    remaining = max(0.0, self.hard_deadline - time.monotonic())
                    broker.wait(timeout=remaining)
                except subprocess.TimeoutExpired:
                    # No group signal follows this point unless the live
                    # anchor was verified by _cleanup_group above.
                    pass
                self._children.pop(broker.pid, None)
            if owns_stdout:
                stdout_tmp.close()
            stderr_tmp.close()
            if succeeded:
                shutil.rmtree(status_dir)
            else:
                print(f"owned prepare failure logs preserved: {status_dir}", file=sys.stderr, flush=True)

    @staticmethod
    def _anchor_exists(anchor_pid: int) -> bool:
        try:
            os.kill(anchor_pid, 0)
        except ProcessLookupError:
            return False
        return True

    @staticmethod
    def _read_status(path: Path) -> dict[str, object]:
        try:
            info = path.lstat()
            if not stat.S_ISREG(info.st_mode) or info.st_size > 4096:
                raise RunnerError("owned prepare broker status was unsafe or oversized")
            status = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError) as error:
            raise RunnerError(f"owned prepare broker status could not be read: {error}") from error
        if not isinstance(status, dict) or status.get("schema") != 1:
            raise RunnerError("owned prepare broker status schema was invalid")
        return status

    def _cleanup_group(self, broker: subprocess.Popen[bytes], control_write: int, status_path: Path) -> None:
        anchor_pid = broker.pid
        if anchor_pid in self._retired_anchors:
            raise RunnerError("owned prepare anchor was already reaped; no further group operations are allowed")
        term_sent = False
        term_deadline = self.hard_deadline
        while True:
            remaining = self._remaining_hard()
            if not self._anchor_exists(anchor_pid):
                try:
                    broker.wait(timeout=remaining)
                except subprocess.TimeoutExpired as error:
                    raise RunnerError("owned prepare broker could not be reaped within cleanup reserve") from error
                self._retired_anchors.add(anchor_pid)
                raise RunnerError("owned prepare anchor exited before cleanup could be verified")
            members = _owned_group_members(anchor_pid, anchor_pid, remaining)
            if members == {anchor_pid} and status_path.exists():
                try:
                    os.write(control_write, b"R")
                except OSError as error:
                    raise RunnerError(f"owned prepare broker release failed: {error}") from error
                try:
                    broker_code = broker.wait(timeout=self._remaining_hard())
                except subprocess.TimeoutExpired as error:
                    raise RunnerError("owned prepare broker did not exit after verified cleanup") from error
                self._retired_anchors.add(anchor_pid)
                if broker_code != 0:
                    raise RunnerError(f"owned prepare broker rejected its cleanup release ({broker_code})")
                if status_path.exists():
                    self._read_status(status_path)
                return
            if not term_sent:
                try:
                    os.killpg(anchor_pid, signal.SIGTERM)
                except ProcessLookupError as error:
                    raise RunnerError("owned prepare process group disappeared before TERM") from error
                term_sent = True
                term_deadline = min(self.hard_deadline, time.monotonic() + PREPARE_OWNED_TERM_GRACE)
            elif time.monotonic() >= term_deadline:
                # This signal is sent only while the unreaped anchor still
                # owns the PGID. Cleanup after this point is unverified, so
                # caller must fail and publish no successful preparation.
                try:
                    os.killpg(anchor_pid, signal.SIGKILL)
                except ProcessLookupError as error:
                    raise RunnerError("owned prepare process group disappeared before KILL") from error
                try:
                    broker.wait(timeout=self._remaining_hard())
                except subprocess.TimeoutExpired as error:
                    raise RunnerError("owned prepare broker KILL could not be reaped within cleanup reserve") from error
                self._retired_anchors.add(anchor_pid)
                raise RunnerError("owned prepare cleanup unverified after SIGKILL")
            time.sleep(min(0.02, max(0.0, term_deadline - time.monotonic()), remaining))


def sha256(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
    if check_work is not None:
        check_work()
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            if check_work is not None:
                check_work()
            h.update(chunk)
    if check_work is not None:
        check_work()
    return h.hexdigest()


def stable_private_binary(path: Path, label: str, *, check_work: Callable[[], None] | None = None) -> tuple[str, dict[str, int]]:
    """Hash one unchanged, unaliased executable through a no-follow file descriptor."""
    if check_work is not None:
        check_work()
    try:
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or not before.st_mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH):
            raise RunnerError(f"{label} must be a private regular executable file")
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            opened = os.fstat(fd)
            digest = hashlib.sha256()
            while chunk := os.read(fd, 1024 * 1024):
                if check_work is not None:
                    check_work()
                digest.update(chunk)
            after_fd = os.fstat(fd)
        finally:
            os.close(fd)
        after = path.lstat()
    except OSError as error:
        raise RunnerError(f"{label} cannot be safely opened: {error}") from error
    identity = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)
    if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after):
        raise RunnerError(f"{label} changed during verification")
    return digest.hexdigest(), {
        "device": opened.st_dev,
        "inode": opened.st_ino,
        "mode": opened.st_mode,
        "size": opened.st_size,
        "mtime_ns": opened.st_mtime_ns,
        "ctime_ns": opened.st_ctime_ns,
    }


def phase_binary_hashes(bins: Path) -> dict[str, str]:
    if bins.is_symlink() or not bins.is_dir():
        raise RunnerError(f"paired phase binary directory is missing or unsafe: {bins}")
    result = {}
    for app in APPS:
        binary = bins / app
        if binary.is_symlink() or not binary.is_file() or binary.stat().st_nlink != 1:
            raise RunnerError(f"paired phase executable is missing or unsafe: {binary}")
        result[app] = sha256(binary)
    return result


def _digest_file_bytes(hasher: Any, path: Path, check_work: Callable[[], None] | None = None) -> None:
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            if check_work is not None:
                check_work()
            hasher.update(chunk)


def digest_tree(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
    h = hashlib.sha256()
    files = []
    for item in path.rglob("*"):
        if check_work is not None:
            check_work()
        if item.is_file():
            files.append(item)
    for file in sorted(files):
        if check_work is not None:
            check_work()
        h.update(file.relative_to(path).as_posix().encode())
        h.update(b"\0")
        _digest_file_bytes(h, file, check_work)
        h.update(b"\0")
    return h.hexdigest()


def sealed_tree(path: Path) -> tuple[str, set[str]]:
    if path.is_symlink() or not path.is_dir():
        raise RunnerError(f"sealed store root must be a real directory: {path}")
    entries: set[str] = set()
    digest = hashlib.sha256()
    for current, directories, files in os.walk(path, followlinks=False):
        current_path = Path(current)
        for name in directories:
            item = current_path / name
            if item.is_symlink() or not stat.S_ISDIR(item.lstat().st_mode):
                raise RunnerError(f"sealed store contains non-directory or symlink: {item}")
            relative = item.relative_to(path).as_posix() + "/"
            entries.add(relative)
            digest.update(b"dir\0" + relative.encode() + b"\0")
        for name in files:
            item = current_path / name
            if item.is_symlink() or not stat.S_ISREG(item.lstat().st_mode):
                raise RunnerError(f"sealed store contains non-regular file or symlink: {item}")
            relative = item.relative_to(path).as_posix()
            entries.add(relative)
            digest.update(b"file\0" + relative.encode() + b"\0")
            digest.update(bytes.fromhex(sha256(item)))
            digest.update(b"\0")
    return digest.hexdigest(), entries


def flat_sealed_store(path: Path, label: str) -> tuple[str, set[str]]:
    digest, entries = sealed_tree(path)
    directories = sorted(name for name in entries if name.endswith("/"))
    if directories:
        raise RunnerError(f"{label} store contains unexpected directory: {directories[:3]}")
    return digest, entries


def digest_source(path: Path, *, check_work: Callable[[], None] | None = None) -> str:
    h = hashlib.sha256()
    files = []
    for item in path.rglob("*"):
        if check_work is not None:
            check_work()
        if item.is_file() and not {"target", "snapshots"}.intersection(item.relative_to(path).parts):
            files.append(item)
    for file in sorted(files):
        if check_work is not None:
            check_work()
        h.update(file.relative_to(path).as_posix().encode())
        h.update(b"\0")
        _digest_file_bytes(h, file, check_work)
        h.update(b"\0")
    return h.hexdigest()


def file_manifest(path: Path, *, check_work: Callable[[], None] | None = None) -> dict[str, str]:
    files = []
    for file in path.rglob("*"):
        if check_work is not None:
            check_work()
        if file.is_file() and not {"target", "snapshots"}.intersection(file.relative_to(path).parts):
            files.append(file)
    return {
        file.relative_to(path).as_posix(): sha256(file, check_work=check_work) if check_work is not None else sha256(file)
        for file in sorted(files)
    }


def freeze_tree_readonly(path: Path, *, check_work: Callable[[], None] | None = None) -> None:
    if path.is_symlink() or not path.is_dir():
        raise RunnerError(f"frozen copy must be a real directory: {path}")
    directories = []
    for current, child_dirs, files in os.walk(path, followlinks=False):
        if check_work is not None:
            check_work()
        current_path = Path(current)
        directories.append(current_path)
        for name in child_dirs:
            item = current_path / name
            if item.is_symlink() or not stat.S_ISDIR(item.lstat().st_mode):
                raise RunnerError(f"frozen copy contains non-directory or symlink: {item}")
        for name in files:
            if check_work is not None:
                check_work()
            item = current_path / name
            if item.is_symlink() or not stat.S_ISREG(item.lstat().st_mode):
                raise RunnerError(f"frozen copy contains non-regular file or symlink: {item}")
            item.chmod(0o444)
    for directory in reversed(directories):
        if check_work is not None:
            check_work()
        directory.chmod(0o555)
        if check_work is not None:
            check_work()


def safe_extract(tar: tarfile.TarFile, destination: Path, *, check_work: Callable[[], None] | None = None) -> None:
    root = destination.resolve()
    members = []
    for member in tar:
        if check_work is not None:
            check_work()
        name = PurePosixPath(member.name)
        if name.is_absolute() or ".." in name.parts:
            raise RunnerError(f"unsafe archive path: {member.name}")
        target = (destination / Path(*name.parts)).resolve()
        if target != root and root not in target.parents:
            raise RunnerError(f"archive path escapes destination: {member.name}")
        if member.issym() or member.islnk():
            link = PurePosixPath(member.linkname)
            resolved = (target.parent / Path(*link.parts)).resolve()
            if link.is_absolute() or ".." in link.parts or (resolved != root and root not in resolved.parents):
                raise RunnerError(f"unsafe archive link: {member.name}")
        members.append(member)
    if check_work is None:
        # Keep the legacy tarfile behavior, including delayed directory mode
        # and timestamp application after children have been extracted.
        tar.extractall(destination, members=members)
        return

    def checked_members() -> Any:
        for member in members:
            check_work()
            yield member

    check_work()
    # Wrap the member iterator instead of overriding tarfile's extraction
    # filter. This preserves the runtime's legacy default filter and its
    # delayed directory metadata handling while checking between members.
    tar.extractall(destination, members=checked_members())
    check_work()


def paths(repo: Path, root: Path) -> tuple[Path, Path, Path]:
    repo = repo.resolve()
    root = root.resolve()
    source = root / "source"
    workspace = source / "Cargo.toml"
    if not workspace.is_file():
        raise RunnerError(f"frozen source missing; run prepare first: {source}")
    return repo, root, source


def prepare(repo: Path, root: Path, timeout: int, *, execution: PrepareExecution | None = None) -> None:
    repo = repo.resolve()
    root = root.resolve()
    source = root / "source"
    if source.exists():
        raise RunnerError(f"refusing to replace frozen source: {source}")
    def prepare_run(command: list[str], *, stdout_file: object | None = None, capture_stdout: bool = True, show_command: bool = True, quiet: bool = False) -> str:
        if execution is None:
            if stdout_file is not None:
                raise RunnerError("legacy prepare cannot direct command output to an external file")
            return run(command, cwd=repo, timeout=timeout).strip()
        output, _errors = execution.run(command, cwd=repo, stdout_file=stdout_file, capture_stdout=capture_stdout, show_command=show_command, quiet=quiet)
        execution.check_work()
        return output.strip()

    tag_obj = prepare_run(["git", "rev-parse", f"{BASELINE_TAG}^{{tag}}"])
    commit = prepare_run(["git", "rev-parse", f"{BASELINE_TAG}^{{commit}}"])
    if tag_obj != BASELINE_TAG_OBJECT or commit != BASELINE_COMMIT:
        raise RunnerError(f"tag identity mismatch: object={tag_obj}, commit={commit}")
    root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryFile() as bundle:
        if execution is None:
            archive = subprocess.Popen(
                ["git", "archive", "--format=tar", BASELINE_COMMIT],
                cwd=repo,
                stdout=bundle,
                stderr=subprocess.PIPE,
                start_new_session=True,
            )
            try:
                _, stderr = archive.communicate(timeout=timeout)
            except subprocess.TimeoutExpired as error:
                os.killpg(archive.pid, signal.SIGTERM)
                try:
                    archive.communicate(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(archive.pid, signal.SIGKILL)
                    archive.communicate()
                raise RunnerError(f"git archive timed out after {timeout}s") from error
            if archive.returncode:
                raise RunnerError(f"git archive failed ({archive.returncode}): {stderr.decode(errors='replace')}")
        else:
            prepare_run(["git", "archive", "--format=tar", BASELINE_COMMIT], stdout_file=bundle, capture_stdout=False, show_command=False, quiet=True)
        source.mkdir()
        bundle.seek(0)
        with tarfile.open(fileobj=bundle, mode="r:") as tar:
            safe_extract(tar, source, check_work=execution.check_work if execution is not None else None)
    lock = (source / "Cargo.lock").read_text()
    if execution is not None:
        execution.check_work()
    if f'rev = "{TUISCOTTI_REV}"' not in lock and TUISCOTTI_REV not in lock:
        raise RunnerError(f"frozen Cargo.lock does not record tuiscotti {TUISCOTTI_REV}")
    tree = prepare_run(["git", "rev-parse", f"{commit}^{{tree}}"])
    if tree != BASELINE_TREE:
        raise RunnerError(f"frozen tree mismatch: {tree}")
    if execution is not None:
        execution.check_work()
    manifest = {
        "tag": BASELINE_TAG,
        "tag_object": tag_obj,
        "source_commit": commit,
        "source_tree": tree,
        "tuiscotti_rev": TUISCOTTI_REV,
        "source_sha256": digest_source(source, check_work=execution.check_work if execution is not None else None),
        "snapshots_sha256": digest_tree(source / "snapshots", check_work=execution.check_work if execution is not None else None),
    }
    (root / "provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


def verify_source(root: Path, source: Path, *, check_work: Callable[[], None] | None = None) -> str:
    if check_work is not None:
        check_work()
    manifest = json.loads((root / "provenance.json").read_text())
    if check_work is not None:
        check_work()
    if manifest.get("source_commit") != BASELINE_COMMIT or manifest.get("tag_object") != BASELINE_TAG_OBJECT or manifest.get("source_tree") != BASELINE_TREE:
        raise RunnerError("frozen provenance identity mismatch")
    current = digest_source(source, check_work=check_work) if check_work is not None else digest_source(source)
    if current != manifest.get("source_sha256"):
        raise RunnerError("frozen source was modified after extraction")
    current_snapshots = digest_tree(source / "snapshots", check_work=check_work) if check_work is not None else digest_tree(source / "snapshots")
    if current_snapshots != manifest.get("snapshots_sha256"):
        raise RunnerError("frozen snapshots were modified after extraction")
    return current


def _fresh_cohort_source_record(repo: Path, cohort_id: str, binary_dir: Path, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
    if check_work is not None:
        check_work()
    capture = require_capture_eligible_cohort(repo, cohort_id, binary_dir, check_work=check_work)
    hashes: dict[str, str] = {}
    identities: dict[str, dict[str, int]] = {}
    for app in APPS:
        if check_work is not None:
            check_work()
        hashes[app], identities[app] = stable_private_binary(binary_dir / app, f"fresh-capture baseline binary {app}", check_work=check_work)
    if hashes != {app: item["sha256"] for app, item in capture["binaries"].items()}:
        raise RunnerError("capture cohort binary hashes changed during source preparation")
    return {
        "cohort_id": cohort_id,
        "cohort_registry_sha256": capture["cohort_registry_sha256"],
        "review_registry_sha256": capture["review_registry_sha256"],
        "decision_sha256": capture["decision_sha256"],
        "evidence_index_sha256": capture["evidence_index_sha256"],
        "binary_sha256": hashes,
        "binary_file_identity": identities,
        "binary_dir": str(binary_dir.resolve()),
        "candidate_comparison_eligible": False,
        "checkpoint_reference_approval": "none",
        "product_acceptance_eligible": False,
    }


def _path_entry_exists(path: Path) -> bool:
    try:
        path.lstat()
    except FileNotFoundError:
        return False
    return True


def _publish_live_source_manifest(root: Path, manifest_path: Path, info: dict[str, object], *, check_work: Callable[[], None] | None = None) -> None:
    temporary = root / ".live-source.json.tmp"
    if _path_entry_exists(temporary):
        raise RunnerError("refusing to replace live source manifest temporary entry")
    if _path_entry_exists(manifest_path):
        raise RunnerError("refusing to replace live source manifest entry")

    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        raise RunnerError("platform lacks no-follow support for live source manifest creation")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | nofollow
    try:
        descriptor = os.open(temporary, flags, 0o644)
    except FileExistsError as error:
        raise RunnerError("refusing to replace live source manifest temporary entry") from error
    try:
        created = os.fstat(descriptor)
        if not stat.S_ISREG(created.st_mode):
            raise RunnerError("live source manifest temporary was not a regular file")
        with os.fdopen(descriptor, "wb") as stream:
            descriptor = -1
            stream.write((json.dumps(info, indent=2) + "\n").encode())
            stream.flush()
            os.fsync(stream.fileno())
        if check_work is not None:
            check_work()
        temporary_entry = temporary.lstat()
        if not stat.S_ISREG(temporary_entry.st_mode) or (temporary_entry.st_dev, temporary_entry.st_ino) != (created.st_dev, created.st_ino):
            raise RunnerError("live source manifest temporary entry changed during publication")
        try:
            # A same-filesystem hard-link publishes atomically and refuses an
            # existing file or symlink, including a dangling symlink.
            os.link(temporary, manifest_path, follow_symlinks=False)
        except FileExistsError as error:
            raise RunnerError("refusing to replace live source manifest entry") from error
        published = manifest_path.lstat()
        if not stat.S_ISREG(published.st_mode) or (published.st_dev, published.st_ino) != (created.st_dev, created.st_ino):
            raise RunnerError("published live source manifest was not the owned regular file")
        temporary_after = temporary.lstat()
        if not stat.S_ISREG(temporary_after.st_mode) or (temporary_after.st_dev, temporary_after.st_ino) != (created.st_dev, created.st_ino):
            raise RunnerError("live source manifest temporary entry changed before cleanup")
        temporary.unlink()
    except BaseException:
        if descriptor >= 0:
            os.close(descriptor)
        raise


def prepare_live_source(
    repo: Path,
    root: Path,
    source: Path,
    timeout: int,
    *,
    fresh_cohort_id: str | None = None,
    baseline_bin_dir: Path | None = None,
    execution: PrepareExecution | None = None,
) -> dict[str, object]:
    live = root / "live-source"
    manifest_path = root / "live-source.json"
    if execution is not None and (fresh_cohort_id is None or baseline_bin_dir is None):
        raise RunnerError("owned prepare-live requires explicit fresh cohort and baseline binaries")
    if execution is not None and fresh_cohort_id != PREPARE_LIVE_OWNED_COHORT:
        raise RunnerError("owned prepare-live requires the pinned frozen-20261021-serial-nonincr-r1-r2 cohort")
    if _path_entry_exists(live) or _path_entry_exists(manifest_path):
        raise RunnerError(f"refusing to replace prepared live source: {live}")
    check_work = execution.check_work if execution is not None else None
    if check_work is not None:
        check_work()
    patch_path = repo / "tools" / "frozen_cli_pair.patch"
    helper_path = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    extension_manifest_path = repo / EXTENSION_MANIFEST
    extension_source_path = repo / EXTENSION_SOURCE
    if not patch_path.is_file() or not helper_path.is_file() or not extension_manifest_path.is_file() or not extension_source_path.is_file():
        raise RunnerError("live harness patch, frame helper, or visual extension inputs missing from candidate checkout")
    patch_digest = sha256(patch_path, check_work=check_work) if check_work is not None else sha256(patch_path)
    if patch_digest != EXTENSION_PATCH_SHA256:
        raise RunnerError("visual extension test patch differs from the pinned extension revision")
    extension_manifest_digest = sha256(extension_manifest_path, check_work=check_work) if check_work is not None else sha256(extension_manifest_path)
    extension_source_digest = sha256(extension_source_path, check_work=check_work) if check_work is not None else sha256(extension_source_path)
    if extension_manifest_digest != EXTENSION_MANIFEST_SHA256 or extension_source_digest != EXTENSION_SOURCE_SHA256:
        raise RunnerError("visual extension manifest or Rust module differs from the pinned extension revision")
    extension_manifest = load_extension_manifest(extension_manifest_path)
    if fresh_cohort_id is None:
        if baseline_bin_dir is not None:
            raise RunnerError("baseline binary directory requires explicit fresh cohort mode")
        _, reference_decision_hash = load_reference_decisions(repo, root, source)
        fresh_cohort = None
    else:
        if baseline_bin_dir is None:
            raise RunnerError("fresh cohort mode requires its sealed baseline binary directory")
        fresh_cohort = _fresh_cohort_source_record(repo, fresh_cohort_id, baseline_bin_dir.resolve(), check_work=check_work) if check_work is not None else _fresh_cohort_source_record(repo, fresh_cohort_id, baseline_bin_dir.resolve())
        reference_decision_hash = None

    def checked_copy2(src: str, dst: str, *, follow_symlinks: bool = True) -> str:
        if check_work is not None:
            check_work()
        copied = shutil.copy2(src, dst, follow_symlinks=follow_symlinks)
        if check_work is not None:
            check_work()
        return copied

    def checked_copyfile(src: str | os.PathLike[str], dst: str | os.PathLike[str]) -> str:
        if check_work is not None:
            check_work()
        copied = shutil.copyfile(src, dst)
        if check_work is not None:
            check_work()
        return copied

    before = file_manifest(source, check_work=check_work) if check_work is not None else file_manifest(source)
    if check_work is not None:
        check_work()
    shutil.copytree(source, live, ignore=shutil.ignore_patterns("target"), symlinks=True, copy_function=checked_copy2)
    if check_work is not None:
        check_work()
    snapshot_hash = digest_tree(source / "snapshots", check_work=check_work) if check_work is not None else digest_tree(source / "snapshots")
    copied_snapshot_hash = digest_tree(live / "snapshots", check_work=check_work) if check_work is not None else digest_tree(live / "snapshots")
    if copied_snapshot_hash != snapshot_hash:
        raise RunnerError("live source snapshot copy differs from frozen snapshots")
    freeze_tree_readonly(live / "snapshots", check_work=check_work)
    if execution is None:
        run(["patch", "--batch", "--forward", "-V", "none", "-p1", "-i", str(patch_path.resolve())], cwd=live, timeout=timeout, quiet=True)
    else:
        execution.run(["patch", "--batch", "--forward", "-V", "none", "-p1", "-i", str(patch_path.resolve())], cwd=live, env=fresh_clean_env(), quiet=True)
        execution.check_work()
    module = live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"
    extension_module = live / "tests" / "visual_baseline" / "frozen_cli_extensions.rs"
    extension_manifest_copy = live / "tests" / "visual_baseline" / "frozen_cli_extensions.json"
    checked_copyfile(helper_path, module)
    checked_copyfile(extension_source_path, extension_module)
    checked_copyfile(extension_manifest_path, extension_manifest_copy)
    expected = {"tests/visual_baseline/main.rs", "tests/visual_baseline/support.rs", "tests/visual_baseline/scenarios/mod.rs", "tests/visual_baseline/scenarios/interact.rs", "tests/visual_baseline/frozen_cli_pair.rs", "tests/visual_baseline/frozen_cli_extensions.rs", "tests/visual_baseline/frozen_cli_extensions.json"}
    after = file_manifest(live, check_work=check_work) if check_work is not None else file_manifest(live)
    changed = {name for name in set(before) | set(after) if before.get(name) != after.get(name)}
    if changed != expected:
        raise RunnerError(f"live harness patch changed unexpected frozen source files: {sorted(changed)}")
    live_hash = digest_source(live, check_work=check_work) if check_work is not None else digest_source(live)
    frozen_source_hash = digest_source(source, check_work=check_work) if check_work is not None else digest_source(source)
    patch_file_hash = sha256(patch_path, check_work=check_work) if check_work is not None else sha256(patch_path)
    helper_file_hash = sha256(helper_path, check_work=check_work) if check_work is not None else sha256(helper_path)
    module_file_hash = sha256(module, check_work=check_work) if check_work is not None else sha256(module)
    extension_manifest_file_hash = sha256(extension_manifest_path, check_work=check_work) if check_work is not None else sha256(extension_manifest_path)
    extension_manifest_copy_hash = sha256(extension_manifest_copy, check_work=check_work) if check_work is not None else sha256(extension_manifest_copy)
    extension_source_file_hash = sha256(extension_source_path, check_work=check_work) if check_work is not None else sha256(extension_source_path)
    extension_module_file_hash = sha256(extension_module, check_work=check_work) if check_work is not None else sha256(extension_module)
    live_snapshot_hash = digest_tree(live / "snapshots", check_work=check_work) if check_work is not None else digest_tree(live / "snapshots")
    info = {
        "frozen_source_sha256": frozen_source_hash,
        "live_source_sha256": live_hash,
        "patch_sha256": patch_file_hash,
        "helper_sha256": helper_file_hash,
        "module_sha256": module_file_hash,
        "extension_manifest_sha256": extension_manifest_file_hash,
        "extension_manifest_copy_sha256": extension_manifest_copy_hash,
        "extension_source_sha256": extension_source_file_hash,
        "extension_module_sha256": extension_module_file_hash,
        "extension_case_count": len(extension_manifest["generated_case_ids"]),
        "reference_decisions_sha256": reference_decision_hash,
        "fresh_capture_cohort": fresh_cohort,
        "extension_inventory_sha256": hashlib.sha256(json.dumps(extension_manifest["generated_case_ids"], separators=(",", ":")).encode()).hexdigest(),
        "frozen_snapshots_sha256": snapshot_hash,
        "live_snapshots_sha256": live_snapshot_hash,
        "changed_files": sorted(changed),
    }
    env = fresh_clean_env() if fresh_cohort is not None else clean_env()
    env["CARGO_TARGET_DIR"] = str(root / "live-target")
    if fresh_cohort is not None:
        env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"] = "1"
        info["preparation_environment"] = fresh_environment_fingerprint(env)
        info["preparation_tool_observation"] = fresh_tool_observation(live, env, execution=execution) if execution is not None else fresh_tool_observation(live, env)
    inventory = nextest_inventory(live, timeout, env, extension_manifest, execution=execution) if execution is not None else nextest_inventory(live, timeout, env, extension_manifest)
    original_inventory, _ = validate_extension_inventory(inventory, extension_manifest)
    info["test_inventory_sha256"] = inventory_digest(inventory)
    info["original_test_inventory_sha256"] = inventory_digest(original_inventory)
    info["registered_scenarios"] = EXPECTED_SCENARIOS
    info["extension_scenarios"] = len(extension_manifest["generated_case_ids"])
    info["ordinary_checks"] = EXPECTED_ORDINARY_TESTS
    if check_work is not None:
        check_work()
    _publish_live_source_manifest(root, manifest_path, info, check_work=check_work)
    return info


def verify_live_source(
    repo: Path,
    root: Path,
    source: Path,
    timeout: int,
    *,
    fresh_cohort_id: str | None = None,
    baseline_bin_dir: Path | None = None,
    execution: PrepareExecution | None = None,
) -> tuple[Path, dict[str, object]]:
    check_work = execution.check_work if execution is not None else None
    if check_work is not None:
        check_work()
    live = root / "live-source"
    manifest_path = root / "live-source.json"
    if check_work is not None:
        check_work()
    if not live.is_dir() or not manifest_path.is_file():
        raise RunnerError("live source missing; run prepare-live first")
    recorded = json.loads(manifest_path.read_text())
    if check_work is not None:
        check_work()
    recorded_fresh = recorded.get("fresh_capture_cohort")
    if fresh_cohort_id is None:
        if recorded_fresh is not None or baseline_bin_dir is not None:
            raise RunnerError("prepared source requires its explicit fresh cohort verification mode")
    else:
        if baseline_bin_dir is None or not isinstance(recorded_fresh, dict) or recorded_fresh.get("cohort_id") != fresh_cohort_id:
            raise RunnerError("prepared source does not match the requested fresh cohort")
    patch = repo / "tools" / "frozen_cli_pair.patch"
    helper = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    extension_manifest = repo / EXTENSION_MANIFEST
    extension_source = repo / EXTENSION_SOURCE
    reference_path = repo / REFERENCE_DECISIONS

    def checked_digest(path: Path) -> str:
        return sha256(path, check_work=check_work) if check_work is not None else sha256(path)

    def checked_source_digest(path: Path) -> str:
        return digest_source(path, check_work=check_work) if check_work is not None else digest_source(path)

    def checked_tree_digest(path: Path) -> str:
        return digest_tree(path, check_work=check_work) if check_work is not None else digest_tree(path)

    if fresh_cohort_id is not None and baseline_bin_dir is not None:
        if check_work is not None:
            check_work()
        fresh_cohort = (
            _fresh_cohort_source_record(repo, fresh_cohort_id, baseline_bin_dir.resolve(), check_work=check_work)
            if check_work is not None
            else _fresh_cohort_source_record(repo, fresh_cohort_id, baseline_bin_dir.resolve())
        )
    else:
        fresh_cohort = None
    reference_decision_hash = checked_digest(reference_path) if fresh_cohort_id is None and reference_path.is_file() and not reference_path.is_symlink() else None
    if check_work is not None:
        check_work()
    current = {
        "frozen_source_sha256": checked_source_digest(source),
        "live_source_sha256": checked_source_digest(live),
        "patch_sha256": checked_digest(patch),
        "helper_sha256": checked_digest(helper),
        "module_sha256": checked_digest(live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"),
        "extension_manifest_sha256": checked_digest(extension_manifest),
        "extension_source_sha256": checked_digest(extension_source),
        "extension_module_sha256": checked_digest(live / "tests" / "visual_baseline" / "frozen_cli_extensions.rs"),
        "extension_manifest_copy_sha256": checked_digest(live / "tests" / "visual_baseline" / "frozen_cli_extensions.json"),
        "reference_decisions_sha256": reference_decision_hash,
        "fresh_capture_cohort": fresh_cohort,
        "frozen_snapshots_sha256": checked_tree_digest(source / "snapshots"),
        "live_snapshots_sha256": checked_tree_digest(live / "snapshots"),
    }
    if check_work is not None:
        check_work()
    if fresh_cohort_id is not None:
        env = fresh_clean_env()
        env["CARGO_TARGET_DIR"] = str(root / "live-target")
        env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"] = "1"
        current["preparation_environment"] = fresh_environment_fingerprint(env)
        current["preparation_tool_observation"] = fresh_tool_observation(live, env, execution=execution) if execution is not None else fresh_tool_observation(live, env)
    for key, value in current.items():
        if check_work is not None:
            check_work()
        if recorded.get(key) != value:
            raise RunnerError(f"live source provenance changed: {key}")
    if fresh_cohort_id is None:
        _, validated_reference_hash = load_reference_decisions(repo, root, source)
        if validated_reference_hash != current["reference_decisions_sha256"]:
            raise RunnerError("reference decision provenance changed")
    elif current["reference_decisions_sha256"] is not None:
        raise RunnerError("fresh cohort source unexpectedly depends on legacy checkpoint decisions")
    if current["extension_manifest_sha256"] != EXTENSION_MANIFEST_SHA256 or current["extension_source_sha256"] != EXTENSION_SOURCE_SHA256:
        raise RunnerError("visual extension inputs differ from the pinned extension revision")
    if current["patch_sha256"] != EXTENSION_PATCH_SHA256:
        raise RunnerError("visual extension test patch differs from the pinned extension revision")
    if check_work is not None:
        check_work()
    return live, recorded


def git_state(repo: Path, timeout: int) -> tuple[str, str, list[str]]:
    revision = run(["git", "-C", str(repo), "rev-parse", "HEAD"], cwd=repo, timeout=timeout).strip()
    tracked = run(["git", "-C", str(repo), "diff", "--name-only", "-z", "HEAD"], cwd=repo, timeout=timeout, quiet=True)
    untracked = run(["git", "-C", str(repo), "ls-files", "--others", "--exclude-standard", "-z"], cwd=repo, timeout=timeout, quiet=True)
    files = sorted(set(filter(None, tracked.split("\0") + untracked.split("\0"))))
    digest = hashlib.sha256()
    for name in files:
        path = repo / name
        digest.update(name.encode())
        digest.update(b"\0")
        if path.is_symlink():
            digest.update(b"link\0" + os.readlink(path).encode())
        elif path.is_file():
            digest.update(bytes.fromhex(sha256(path)))
        else:
            digest.update(b"missing")
        digest.update(b"\0")
    return revision, digest.hexdigest(), files


def canonical_json_digest(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def build_environment_fingerprint(env: dict[str, str]) -> dict[str, object]:
    observed = {
        key: {"present": key in env, "value_sha256": hashlib.sha256(env[key].encode()).hexdigest() if key in env else None}
        for key in PROVENANCE_ENV_KEYS
    }
    return {
        "policy": "hash-only-explicit-build-environment-allowlist",
        "allowlisted_values": observed,
        "fingerprint_sha256": canonical_json_digest(observed),
        "unknown_environment_values_recorded": False,
        "effective_env_was_runner_cleaned": True,
        "hermetic": False,
        "hermeticity_note": "clean_env inherits unreviewed host variables and PATH; this is observation, not a hermetic build claim",
    }


def _captured_output(command: list[str], *, cwd: Path, env: dict[str, str], timeout: int = 12, execution: PrepareExecution | None = None) -> str | None:
    if execution is not None:
        bounded_timeout = min(float(timeout), 12.0, execution.remaining_work)
        result = execution.run(command, cwd=cwd, env=env, quiet=True, command_timeout=bounded_timeout, optional_failure=True)
        return result[0].strip() if result is not None else None
    try:
        result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def _executable_metadata(path: str | None, *, env: dict[str, str], version_args: list[str], cwd: Path, execution: PrepareExecution | None = None) -> dict[str, object]:
    if execution is not None:
        execution.check_work()
    if not path:
        return {"available": False, "path": None, "sha256": None, "version": None}
    entry = Path(path)
    if not entry.is_absolute():
        entry = Path(shutil.which(path, path=env.get("PATH")) or path)
    try:
        resolved = entry.resolve(strict=True)
    except OSError:
        return {"available": False, "path": str(entry), "sha256": None, "version": None}
    if not resolved.is_file():
        return {"available": False, "path": str(resolved), "sha256": None, "version": None}
    executable_hash = sha256(resolved, check_work=execution.check_work) if execution is not None else sha256(resolved)
    version = _captured_output([str(resolved), *version_args], cwd=cwd, env=env, execution=execution) if execution is not None else _captured_output([str(resolved), *version_args], cwd=cwd, env=env)
    return {
        "available": True,
        "path": str(resolved),
        "sha256": executable_hash,
        "version": version,
    }


def toolchain_provenance(source: Path, env: dict[str, str], *, execution: PrepareExecution | None = None) -> dict[str, object]:
    def probe(command: list[str] | None) -> str | None:
        if execution is not None:
            execution.check_work()
        if not command:
            return None
        return _captured_output(command, cwd=source, env=env, execution=execution) if execution is not None else _captured_output(command, cwd=source, env=env)

    cargo_entry = shutil.which("cargo", path=env.get("PATH"))
    rustup_entry = shutil.which("rustup", path=env.get("PATH"))
    rustc_entry = env.get("RUSTC") or shutil.which("rustc", path=env.get("PATH"))
    if rustup_entry:
        selected_rustc = probe([rustup_entry, "which", "rustc"])
        selected_cargo = probe([rustup_entry, "which", "cargo"])
        if selected_rustc:
            rustc_entry = selected_rustc
    else:
        selected_cargo = None
    cargo_selected = selected_cargo or cargo_entry
    compiler = shutil.which("cc", path=env.get("PATH"))
    clang = shutil.which("clang", path=env.get("PATH"))
    linker = shutil.which("ld", path=env.get("PATH"))
    xcrun = shutil.which("xcrun", path=env.get("PATH"))
    xcode_select = shutil.which("xcode-select", path=env.get("PATH"))
    sdk_path = probe([xcrun, "--show-sdk-path"] if xcrun else None)
    sdk_version = probe([xcrun, "--show-sdk-version"] if xcrun else None)
    selected_xcode = probe([xcode_select, "-p"] if xcode_select else None)
    active_toolchain = probe([rustup_entry, "show", "active-toolchain"] if rustup_entry else None)
    rustc_metadata = _executable_metadata(rustc_entry, env=env, version_args=["-vV"], cwd=source, execution=execution)
    rustc_version = rustc_metadata.get("version")
    host_triple = None
    llvm_version = None
    if isinstance(rustc_version, str):
        for line in rustc_version.splitlines():
            if line.startswith("host: "):
                host_triple = line.partition(": ")[2]
            elif line.startswith("LLVM version: "):
                llvm_version = line.partition(": ")[2]
    sysroot = probe([rustc_entry, "--print", "sysroot"] if rustc_entry else None)
    sysroot_files = []
    if sysroot:
        for relative in ("lib/rustlib/components", "lib/rustlib/manifest-rustc", "lib/rustlib/multirust-channel-manifest.toml"):
            if execution is not None:
                execution.check_work()
            path = Path(sysroot) / relative
            if path.is_file() and not path.is_symlink():
                sysroot_hash = sha256(path, check_work=execution.check_work) if execution is not None else sha256(path)
                sysroot_files.append({"path": str(path), "sha256": sysroot_hash})
    return {
        "cargo_path_entry": _executable_metadata(cargo_entry, env=env, version_args=["--version", "--verbose"], cwd=source, execution=execution),
        "cargo_selected_toolchain": _executable_metadata(cargo_selected, env=env, version_args=["--version", "--verbose"], cwd=source, execution=execution),
        "cargo_selection_method": "rustup which from frozen-source cwd" if selected_cargo else "PATH entry only; toolchain selection unresolved",
        "rustup": _executable_metadata(rustup_entry, env=env, version_args=["--version"], cwd=source, execution=execution),
        "active_toolchain": active_toolchain,
        "rustc_selected_toolchain": rustc_metadata,
        "rustc_selection_method": "RUSTC environment path" if env.get("RUSTC") else "rustup which from frozen-source cwd" if rustup_entry else "PATH entry only; toolchain selection unresolved",
        "rustc_host_triple": host_triple,
        "llvm_version": llvm_version,
        "rustc_sysroot": sysroot,
        "rustc_sysroot_identity_files": sysroot_files,
        "cc": _executable_metadata(compiler, env=env, version_args=["--version"], cwd=source, execution=execution),
        "clang": _executable_metadata(clang, env=env, version_args=["--version"], cwd=source, execution=execution),
        "linker": _executable_metadata(linker, env=env, version_args=["-v"], cwd=source, execution=execution),
        "xcrun": _executable_metadata(xcrun, env=env, version_args=["--version"], cwd=source, execution=execution),
        "xcode_select": _executable_metadata(xcode_select, env=env, version_args=["-version"], cwd=source, execution=execution),
        "sdk_path": sdk_path,
        "sdk_version": sdk_version,
        "xcode_developer_directory": selected_xcode,
        "link_invocation": "not captured; cargo config and available linker identities are recorded as observations",
    }


def _selected_executable_observation(name: str, env: dict[str, str], version_args: list[str], cwd: Path, override: str | None = None, *, execution: PrepareExecution | None = None) -> dict[str, object]:
    if execution is not None:
        execution.check_work()
    entry = override or shutil.which(name, path=env.get("PATH"))
    if not entry:
        raise RunnerError(f"fresh execution tool is unavailable: {name}")
    resolved = Path(entry).resolve(strict=True)
    digest, identity = stable_private_binary(resolved, f"selected fresh execution tool {name}", check_work=execution.check_work) if execution is not None else stable_private_binary(resolved, f"selected fresh execution tool {name}")
    version = _captured_output([str(resolved), *version_args], cwd=cwd, env=env, execution=execution) if execution is not None else _captured_output([str(resolved), *version_args], cwd=cwd, env=env)
    if not version:
        raise RunnerError(f"selected fresh execution tool did not report its version: {name}")
    return {"path": str(resolved), "sha256": digest, "file_identity": identity, "version": version}


def fresh_tool_observation(source: Path, env: dict[str, str], *, execution: PrepareExecution | None = None) -> dict[str, object]:
    """Capture selected executable identities and compiler/linker metadata without storing binary bytes."""
    invocation = {
        "cargo": _selected_executable_observation("cargo", env, ["--version", "--verbose"], source, execution=execution) if execution is not None else _selected_executable_observation("cargo", env, ["--version", "--verbose"], source),
        "rustc_path_entry": _selected_executable_observation("rustc", env, ["-vV"], source, env.get("RUSTC"), execution=execution) if execution is not None else _selected_executable_observation("rustc", env, ["-vV"], source, env.get("RUSTC")),
        "cargo_nextest": _selected_executable_observation("cargo-nextest", env, ["--version"], source, execution=execution) if execution is not None else _selected_executable_observation("cargo-nextest", env, ["--version"], source),
        "python": _selected_executable_observation("python", env, ["--version"], source, sys.executable, execution=execution) if execution is not None else _selected_executable_observation("python", env, ["--version"], source, sys.executable),
    }
    return {
        "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
        "invocation_tools": invocation,
        "selected_toolchain": toolchain_provenance(source, env, execution=execution) if execution is not None else toolchain_provenance(source, env),
    }


def _cargo_config_inputs(source: Path, env: dict[str, str]) -> list[dict[str, str]]:
    candidates = []
    for directory in (source, *source.parents):
        candidates.extend((directory / ".cargo" / "config", directory / ".cargo" / "config.toml"))
    cargo_home = Path(env.get("CARGO_HOME", str(Path(env.get("HOME", str(Path.home()))) / ".cargo")))
    candidates.extend((cargo_home / "config", cargo_home / "config.toml"))
    result = []
    for path in candidates:
        if path.is_file() and not path.is_symlink():
            result.append({"path": str(path.resolve()), "sha256": sha256(path)})
    return result


def _application_binary_hashes(bin_dir: Path) -> dict[str, str]:
    hashes = {}
    for app in APPS:
        binary = bin_dir / app
        if not binary.is_file():
            raise RunnerError(f"missing frozen binary: {binary}")
        hashes[app] = sha256(binary)
    return hashes


def build_provenance_payload(repo: Path, source: Path, source_hash: str, target: Path, binaries: dict[str, str], stages: list[dict[str, object]], env: dict[str, str], timeout: int) -> dict[str, object]:
    archive = subprocess.run(
        ["git", "-C", str(repo), "archive", "--format=tar", BASELINE_COMMIT],
        cwd=repo,
        env=env,
        capture_output=True,
        timeout=min(timeout, 225),
        check=False,
    )
    if archive.returncode != 0:
        raise RunnerError("could not fingerprint the fixed frozen-source git archive")
    runner_path = Path(__file__).resolve()
    patch_path = repo / "tools" / "frozen_cli_pair.patch"
    helper_manifest = repo / "tools" / "frozen-cli-frame-support" / "Cargo.toml"
    helper_lock = repo / "tools" / "frozen-cli-frame-support" / "Cargo.lock"
    helper_path = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    extension_manifest = repo / EXTENSION_MANIFEST
    extension_source = repo / EXTENSION_SOURCE
    reference_path = repo / REFERENCE_DECISIONS
    tool_inputs = {}
    for name, path in (("runner", runner_path), ("harness_patch", patch_path), ("frame_helper_manifest", helper_manifest), ("frame_helper_lock", helper_lock), ("frame_helper", helper_path), ("extension_manifest", extension_manifest), ("extension_source", extension_source), ("reference_decisions", reference_path)):
        tool_inputs[name] = sha256(path) if path.is_file() and not path.is_symlink() else None
    rust_toolchain = source / "rust-toolchain.toml"
    now = datetime.now(timezone.utc).isoformat()
    stages_with_env = []
    for stage in stages:
        stages_with_env.append({**stage, "effective_environment": build_environment_fingerprint(stage["env"])})
    toolchain = toolchain_provenance(source, env)
    payload = {
        "schema": BUILD_PROVENANCE_SCHEMA,
        "schema_version": BUILD_PROVENANCE_VERSION,
        "status": "observed-not-hermetic",
        "observed_at_utc": now,
        "reference": {
            "tag": BASELINE_TAG,
            "tag_object": BASELINE_TAG_OBJECT,
            "commit": BASELINE_COMMIT,
            "tree": BASELINE_TREE,
            "source_digest": source_hash,
            "archive_sha256": hashlib.sha256(archive.stdout).hexdigest(),
            "archive_argv": ["git", "-C", str(repo), "archive", "--format=tar", BASELINE_COMMIT],
            "cargo_lock_sha256": sha256(source / "Cargo.lock"),
            "rust_toolchain_toml_sha256": sha256(rust_toolchain) if rust_toolchain.is_file() else None,
            "tuiscotti_rev": TUISCOTTI_REV,
        },
        "runner_inputs": tool_inputs,
        "host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine(), "python": platform.python_version()},
        "target_dir": str(target.resolve()),
        "config_inputs": _cargo_config_inputs(source, env),
        "toolchain": toolchain,
        "build": {
            "stage_order": [stage["name"] for stage in stages_with_env],
            "stages": [{key: value for key, value in stage.items() if key != "env"} for stage in stages_with_env],
            "environment_policy": "per-stage hash-only explicit allowlist of the actual env mapping passed to the runner; clean_env strips RUSTFLAGS, wrappers and profile overrides; unknown values are omitted and host inheritance remains uncontrolled",
            "profile": "Cargo default dev profile",
            "features": "default features; no explicit --features or --no-default-features",
            "target_triple": toolchain.get("rustc_host_triple"),
            "target_selection": "observed rustc host triple; Cargo config inputs are hash-recorded but not interpreted, so config target overrides remain unresolved",
            "final_application_binary_sha256": binaries,
            "final_hash_point": "after cargo build --bins and cargo nextest --no-run stages",
        },
        "admission": {"eligible": False, "reason": "provenance observation only; no reference decision or binary admission is performed"},
    }
    return payload


def seal_build_provenance(payload: dict[str, object]) -> dict[str, object]:
    sealed = dict(payload)
    cohort = {key: value for key, value in sealed.items() if key not in {"observed_at_utc", "cohort_fingerprint_sha256", "provenance_sha256"}}
    sealed["cohort_fingerprint_sha256"] = canonical_json_digest(cohort)
    sealed["provenance_sha256"] = canonical_json_digest(sealed)
    return sealed


def validate_build_provenance(manifest: dict[str, object]) -> None:
    required = {"schema", "schema_version", "status", "observed_at_utc", "reference", "runner_inputs", "host", "target_dir", "config_inputs", "toolchain", "build", "admission", "cohort_fingerprint_sha256", "provenance_sha256"}
    if set(manifest) != required or manifest.get("schema") != BUILD_PROVENANCE_SCHEMA or manifest.get("schema_version") != BUILD_PROVENANCE_VERSION:
        raise RunnerError("frozen build provenance schema or required fields are invalid")
    if manifest.get("status") != "observed-not-hermetic" or manifest.get("admission") != {"eligible": False, "reason": "provenance observation only; no reference decision or binary admission is performed"}:
        raise RunnerError("frozen build provenance must remain diagnostic-only and not hermetic")
    reference = manifest.get("reference")
    reference_required = {"tag", "tag_object", "commit", "tree", "source_digest", "archive_sha256", "archive_argv", "cargo_lock_sha256", "rust_toolchain_toml_sha256", "tuiscotti_rev"}
    if not isinstance(reference, dict) or set(reference) != reference_required or reference.get("tag") != BASELINE_TAG or reference.get("tag_object") != BASELINE_TAG_OBJECT or reference.get("commit") != BASELINE_COMMIT or reference.get("tree") != BASELINE_TREE or reference.get("tuiscotti_rev") != TUISCOTTI_REV:
        raise RunnerError("frozen build provenance reference identity is invalid")
    if any(not isinstance(reference.get(key), str) or not re.fullmatch(r"[0-9a-f]{64}", reference[key]) for key in ("source_digest", "archive_sha256", "cargo_lock_sha256")):
        raise RunnerError("frozen build provenance source or archive digest is invalid")
    archive_argv = reference.get("archive_argv")
    if not isinstance(archive_argv, list) or len(archive_argv) != 6 or archive_argv[:2] != ["git", "-C"] or archive_argv[2] == "" or archive_argv[3:] != ["archive", "--format=tar", BASELINE_COMMIT]:
        raise RunnerError("frozen build provenance archive command is invalid")
    runner_inputs = manifest.get("runner_inputs")
    runner_input_names = {"runner", "harness_patch", "frame_helper_manifest", "frame_helper_lock", "frame_helper", "extension_manifest", "extension_source", "reference_decisions"}
    if not isinstance(runner_inputs, dict) or set(runner_inputs) != runner_input_names or any(value is not None and (not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value)) for value in runner_inputs.values()):
        raise RunnerError("frozen build provenance runner input hashes are invalid")
    build = manifest.get("build")
    build_required = {"stage_order", "stages", "environment_policy", "profile", "features", "target_triple", "target_selection", "final_application_binary_sha256", "final_hash_point"}
    if not isinstance(build, dict) or set(build) != build_required or not isinstance(build.get("final_application_binary_sha256"), dict) or set(build["final_application_binary_sha256"]) != set(APPS):
        raise RunnerError("frozen build provenance application binary inventory is invalid")
    final_hashes = build["final_application_binary_sha256"]
    if any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) for value in final_hashes.values()):
        raise RunnerError("frozen build provenance final application binary hash is invalid")
    toolchain = manifest.get("toolchain")
    toolchain_required = {"cargo_path_entry", "cargo_selected_toolchain", "cargo_selection_method", "rustup", "active_toolchain", "rustc_selected_toolchain", "rustc_selection_method", "rustc_host_triple", "llvm_version", "rustc_sysroot", "rustc_sysroot_identity_files", "cc", "clang", "linker", "xcrun", "xcode_select", "sdk_path", "sdk_version", "xcode_developer_directory", "link_invocation"}
    if not isinstance(toolchain, dict) or set(toolchain) != toolchain_required:
        raise RunnerError("frozen build provenance toolchain metadata is incomplete")
    stages = build.get("stages")
    if not isinstance(stages, list) or build.get("stage_order") != ["build-frozen-binaries", "build-frozen-nextest-harness"] or build.get("stage_order") != [stage.get("name") for stage in stages if isinstance(stage, dict)]:
        raise RunnerError("frozen build provenance stage order is invalid")
    for stage in stages:
        stage_fields = {"name", "argv", "effective_environment", "output_application_binary_sha256"}
        if not isinstance(stage, dict) or set(stage) != stage_fields or not isinstance(stage.get("argv"), list):
            raise RunnerError("frozen build provenance stage record is incomplete")
        output_hashes = stage.get("output_application_binary_sha256")
        if not isinstance(output_hashes, dict) or set(output_hashes) != set(APPS) or any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) for value in output_hashes.values()):
            raise RunnerError("frozen build provenance stage binary inventory is invalid")
        environment = stage.get("effective_environment")
        if not isinstance(environment, dict) or not isinstance(environment.get("allowlisted_values"), dict) or set(environment["allowlisted_values"]) != set(PROVENANCE_ENV_KEYS) or environment.get("fingerprint_sha256") != canonical_json_digest(environment["allowlisted_values"]):
            raise RunnerError("frozen build provenance environment fingerprint is invalid")
        if environment.get("unknown_environment_values_recorded") is not False or environment.get("hermetic") is not False:
            raise RunnerError("frozen build provenance must omit unknown environment values and remain non-hermetic")
    if stages[-1]["output_application_binary_sha256"] != final_hashes:
        raise RunnerError("frozen build provenance final binaries differ from the last recorded build stage")
    unsigned = {key: value for key, value in manifest.items() if key != "provenance_sha256"}
    if manifest.get("provenance_sha256") != canonical_json_digest(unsigned):
        raise RunnerError("frozen build provenance content digest mismatch")
    cohort = {key: value for key, value in manifest.items() if key not in {"observed_at_utc", "cohort_fingerprint_sha256", "provenance_sha256"}}
    if manifest.get("cohort_fingerprint_sha256") != canonical_json_digest(cohort):
        raise RunnerError("frozen build provenance cohort fingerprint mismatch")


def build(repo: Path, root: Path, timeout: int) -> None:
    repo, root, source = paths(repo, root)
    source_hash = verify_source(root, source)
    target = root / "cargo-target"
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(target)
    build_command = ["cargo", "build", "--locked", "--bins", "--manifest-path", str(source / "Cargo.toml")]
    build_env = dict(env)
    run(build_command, cwd=source, timeout=timeout, env=build_env)
    initial_hashes = _application_binary_hashes(target / "debug")
    env["VISUAL_BIN_DIR"] = str(target / "debug")
    env["VISUAL_FULL"] = "1"
    env["VISUAL_REVIEW"] = "0"
    nextest_command = ["cargo", "nextest", "run", "--no-run", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET]
    nextest_env = dict(env)
    run(nextest_command, cwd=source, timeout=timeout, env=nextest_env)
    hashes = _application_binary_hashes(target / "debug")
    stages = [
        {"name": "build-frozen-binaries", "argv": build_command, "env": build_env, "output_application_binary_sha256": initial_hashes},
        {"name": "build-frozen-nextest-harness", "argv": nextest_command, "env": nextest_env, "output_application_binary_sha256": hashes},
    ]
    provenance = seal_build_provenance(build_provenance_payload(repo, source, source_hash, target, hashes, stages, nextest_env, timeout))
    validate_build_provenance(provenance)
    provenance_path = root / BUILD_PROVENANCE_FILE
    provenance_path.write_text(json.dumps(provenance, indent=2) + "\n")
    record = {"source_sha256": source_hash, "binaries": hashes, "build_provenance_sha256": provenance["provenance_sha256"]}
    (root / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"build": record, "build_provenance": provenance}, indent=2))


def build_candidate(repo: Path, root: Path, target: Path, timeout: int) -> None:
    repo = repo.resolve()
    root = root.resolve()
    if target.is_symlink():
        raise RunnerError("candidate build target cannot be a symlink")
    target = target.resolve()
    if not (repo / "Cargo.toml").is_file():
        raise RunnerError(f"candidate workspace manifest missing: {repo / 'Cargo.toml'}")
    baseline_target = (root / "cargo-target").resolve()
    if target == baseline_target or baseline_target in target.parents or target in baseline_target.parents:
        raise RunnerError("candidate target directory overlaps immutable frozen build directory")
    if target.exists():
        raise RunnerError("candidate build target must be absent to prevent reuse of unverified cached artifacts")
    target.parent.mkdir(parents=True, exist_ok=True)
    try:
        target.mkdir()
    except FileExistsError as error:
        raise RunnerError("candidate build target was claimed before the isolated build") from error
    revision, dirty_fingerprint, dirty_files = git_state(repo, timeout)
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(target)
    run(["cargo", "build", "--locked", "--bins", "--manifest-path", str(repo / "Cargo.toml")], cwd=repo, timeout=timeout, env=env)
    after_revision, after_fingerprint, after_files = git_state(repo, timeout)
    if (after_revision, after_fingerprint, after_files) != (revision, dirty_fingerprint, dirty_files):
        raise RunnerError("candidate source changed while binaries were building")
    hashes = {}
    binary_files = {}
    for app in APPS:
        binary = target / "debug" / app
        hashes[app], binary_files[app] = stable_private_binary(binary, f"candidate binary {app}")
    record = {"repo": str(repo), "revision": revision, "dirty_fingerprint": dirty_fingerprint, "dirty_files": dirty_files, "target_dir": str(target), "bin_dir": str(target / "debug"), "binaries": hashes, "binary_files": binary_files}
    root.mkdir(parents=True, exist_ok=True)
    (root / "candidate-build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))


def nextest_selection(source: Path, root: Path, selector: str | None, all_cases: bool, timeout: int, env: dict[str, str]) -> tuple[str, int]:
    if all_cases == (selector is not None):
        raise RunnerError("choose exactly one: --all or --case <exact test name>")
    command = ["cargo", "nextest", "list", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--message-format", "json"]
    listing = run(command, cwd=source, timeout=timeout, env=env, quiet=True)
    try:
        data = json.loads(listing)
        suite_data = data["rust-suites"]
        cases = suite_data.values() if isinstance(suite_data, dict) else suite_data
        if not all(isinstance(suite.get("testcases"), dict) for suite in cases):
            raise TypeError("testcases is not a name-keyed map")
    except (ValueError, KeyError, TypeError) as error:
        raise RunnerError("could not parse nextest JSON test inventory") from error
    cases = suite_data.values() if isinstance(suite_data, dict) else suite_data
    ignored = [name for suite in cases for name, test in suite["testcases"].items() if test.get("ignored")]
    selected = ignored if all_cases else [name for suite in cases for name in suite["testcases"] if name == selector]
    if not selected:
        raise RunnerError(f"selected zero baseline tests: {selector!r}")
    if selector is not None and len(selected) != 1:
        raise RunnerError(f"selector matched {len(selected)} tests; provide an exact name")
    # The selector becomes a nextest regex after exact inventory validation.
    expression = "all()" if all_cases else f"test(={selector})"
    # Keep ignored scenario rows in the inventory and execute them deliberately.
    selected_ignored = [name for suite in cases for name, test in suite["testcases"].items() if name in selected and test.get("ignored")]
    if not all_cases and len(selected_ignored) != len(selected):
        raise RunnerError("selected test is not an ignored visual scenario")
    return expression, len(selected)


def nextest_inventory(source: Path, timeout: int, env: dict[str, str], extension_manifest: dict[str, object] | None = None, *, execution: PrepareExecution | None = None) -> dict[str, bool]:
    command = ["cargo", "nextest", "list", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--message-format", "json"]
    if execution is None:
        listing = run(command, cwd=source, timeout=timeout, env=env, quiet=True)
    else:
        execution.check_work()
        inventory_env = dict(env)
        inventory_env.pop("VISUAL_TOOL", None)
        owned_output = execution.run(command, cwd=source, env=inventory_env, quiet=True)
        assert owned_output is not None
        listing = owned_output[0]
        execution.check_work()
    try:
        data = json.loads(listing)
        suites = data["rust-suites"]
        suites = suites.values() if isinstance(suites, dict) else suites
        inventory: dict[str, bool] = {}
        for suite in suites:
            if execution is not None:
                execution.check_work()
            testcases = suite["testcases"]
            if not isinstance(testcases, dict):
                raise TypeError("testcases is not a name-keyed map")
            for name, record in testcases.items():
                if execution is not None:
                    execution.check_work()
                if name in inventory:
                    raise RunnerError(f"duplicate Nextest case name: {name}")
                inventory[name] = bool(record.get("ignored"))
    except (ValueError, KeyError, TypeError) as error:
        raise RunnerError("could not parse complete Nextest JSON test inventory") from error
    if execution is not None:
        execution.check_work()
    if extension_manifest is None:
        scenarios = sum(inventory.values())
        ordinary = len(inventory) - scenarios
        if (scenarios, ordinary) != (EXPECTED_SCENARIOS, EXPECTED_ORDINARY_TESTS):
            raise RunnerError(f"registered coverage changed: {scenarios} scenarios and {ordinary} ordinary; expected {EXPECTED_SCENARIOS} and {EXPECTED_ORDINARY_TESTS}")
    else:
        validate_extension_inventory(inventory, extension_manifest, check_work=execution.check_work if execution is not None else None)
    return inventory


def validate_reference_decision_document(document: object) -> dict[str, object]:
    if not isinstance(document, dict) or set(document) != {"schema_version", "decision_scope", "decisions"}:
        raise RunnerError("reference decision document fields differ from schema version 1")
    if document["schema_version"] != 1 or document["decision_scope"] != "one_exact_stale_approval_live_pair_reference":
        raise RunnerError("unsupported reference decision scope")
    decisions = document["decisions"]
    if not isinstance(decisions, list) or len(decisions) != 1:
        raise RunnerError("reference decisions must contain exactly the reviewed single checkpoint")
    decision = decisions[0]
    required = {"decision_id", "tuple", "baseline", "control", "environment", "legacy_approval", "live_frame", "independent_replays", "independent_control"}
    if not isinstance(decision, dict) or set(decision) != required:
        raise RunnerError("reference decision fields differ from schema version 1")
    expected_tuple = {
        "test_id": REFERENCE_CASE,
        "case_id": REFERENCE_TUPLE[0],
        "ordinal": REFERENCE_TUPLE[1],
        "logical_name": REFERENCE_TUPLE[2],
        "application": "showcase",
        "cols": 120,
        "rows": 40,
        "color_mode": "truecolor",
    }
    if decision["tuple"] != expected_tuple or decision["decision_id"] != "visual-baseline-overview-grown-120x40-2026-10-20":
        raise RunnerError("reference decision tuple or identity changed")
    baseline = decision["baseline"]
    expected_baseline = {
        "tag": BASELINE_TAG,
        "tag_object": BASELINE_TAG_OBJECT,
        "commit": BASELINE_COMMIT,
        "source_tree": BASELINE_TREE,
        "source_sha256": "f74646a92505bd9baff29c89398af4d31a4812dd7d51acc84d755be61a383850",
        "visual_test_tree_sha256": "bdd772e6d049efd700912bc5d7b4464d11bb80e69a5d18b0f175b9421546af05",
        "inventory_sha256": "d260d2f0644d63a3a1b4c1611c0d7c61777e59e61249e3ca513d1d6217b0639c",
        "tuiscotti_rev": TUISCOTTI_REV,
        "showcase_binary_sha256": "21e8b1c1af30b47b331aed033525c4682726b99de5fe1238d4507e58f84359d1",
        "binary_sha256": {"showcase": "21e8b1c1af30b47b331aed033525c4682726b99de5fe1238d4507e58f84359d1", "tablepro": "f23caa1d97cecc501e8588079e4f38529f834cbd8bb8b57750a427c5a5f993dc", "jackin-preview": "8fccb21faa2459d4b22e5e527f7c1cc29eff64cd1df025957aaed92da35c2a2e", "holla": "6dd87a4e21dff2c69bc2b102234d0f37a5ddcd5c3f2ecd42114a825b2037e01c"},
    }
    if baseline != expected_baseline:
        raise RunnerError("reference decision frozen source/test/binary pins changed")
    control = decision["control"]
    expected_control = {
        "boot_args": ["--page", "overview"],
        "start_size": [80, 24],
        "resize_to": [120, 40],
        "inputs_after_resize": ["ctrl-l"],
        "post_input_delay_ms": 700,
        "readiness_text": "Design system  Overview",
        "quiet_settle_ms": 400,
        "checkpoint_order": [REFERENCE_TUPLE[2]],
    }
    if control != expected_control:
        raise RunnerError("reference decision interaction controls changed")
    environment = decision["environment"]
    expected_environment = {
        "policy": "runner-clean-env-with-fixed-values",
        "values": {"VISUAL_FULL": "1", "VISUAL_REVIEW": "0", "TERM": "dumb", "NO_COLOR": "1", "COLORTERM": "", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
        "terminal_profile": "tuiscotti-default",
    }
    if environment != expected_environment:
        raise RunnerError("reference decision environment contract changed")
    legacy = decision["legacy_approval"]
    if legacy != {"path": "showcase/resize/overview_grown/120x40/truecolor.ansi", "sha256": "7769a6686dfc5cb5c69facb03f54a0fa57e963b35fcf315b1c4625745e2b7708", "bytes": 18474, "verdict": "known_stale_cells_differ", "paired_exception": "never edits or replaces this approval; its standalone legacy verdict stays red"}:
        raise RunnerError("reference decision legacy approval record changed")
    live_frame = decision["live_frame"]
    if live_frame != {"ansi_sha256": "f2e07ed284673b1d543441767460193e987034e163ca7ea7d3d4496473a0d442", "ansi_bytes": 19454, "semantic_frame_sha256": "01e9f7d349fc8075df7b64188db97a05705792aac8c781184db14a6d1ecc253a", "cols": 120, "rows": 40, "cell_count": 4800, "cursor": {"x": 0, "y": 0, "visible": False, "style": "Bar", "blinking": False}}:
        raise RunnerError("reference decision live frame hashes or cursor changed")
    repeats = decision["independent_replays"]
    if not isinstance(repeats, list) or len(repeats) != 2:
        raise RunnerError("reference decision requires two independent frozen replays")
    for index, repeat in enumerate(repeats, 1):
        if not isinstance(repeat, dict) or set(repeat) != {"run_id", "command_sha256", "log_sha256", "ansi_sha256", "semantic_frame_sha256"} or repeat["run_id"] != f"repeat-{index}" or repeat["ansi_sha256"] != live_frame["ansi_sha256"] or repeat["semantic_frame_sha256"] != live_frame["semantic_frame_sha256"]:
            raise RunnerError("reference decision repeat does not reproduce the reviewed frame")
        if any(not isinstance(repeat[key], str) or not re.fullmatch(r"[0-9a-f]{64}", repeat[key]) for key in ("command_sha256", "log_sha256")):
            raise RunnerError("reference decision repeat evidence hash is invalid")
    control_proof = decision["independent_control"]
    if control_proof != {"test_id": "showcase::pages::overview::120x40::truecolor", "ansi_sha256": "9d07df1b6a32e33d2af4f66d5eff80ff08d55759782f28a42fa7f286f53e5915", "semantic_frame_sha256": "0340c1d42acbadea6cfd6b05534dec4a0ebdb2ca25a0b581ec7719274641e4de"}:
        raise RunnerError("reference decision independent control changed")
    return decision


def load_reference_decisions(repo: Path, root: Path, source: Path) -> tuple[list[dict[str, object]], str]:
    path = repo / REFERENCE_DECISIONS
    if not path.is_file() or path.is_symlink() or sha256(path) != REFERENCE_DECISIONS_SHA256:
        raise RunnerError("reference decision file is missing or differs from the reviewed pin")
    try:
        document = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise RunnerError("could not parse reference decision file") from error
    decision = validate_reference_decision_document(document)
    baseline_source = decision["baseline"]
    build = json.loads((root / "build.json").read_text())
    if digest_source(source) != baseline_source["source_sha256"] or digest_tree(source / "tests" / "visual_baseline") != baseline_source["visual_test_tree_sha256"]:
        raise RunnerError("reference decision source or test suite pin does not match extracted frozen source")
    if build.get("source_sha256") != baseline_source["source_sha256"] or build.get("binaries") != baseline_source["binary_sha256"]:
        raise RunnerError("reference decision baseline binaries are not the reviewed frozen application build")
    approval = source / "snapshots" / Path(str(decision["legacy_approval"]["path"]))
    if not approval.is_file() or sha256(approval) != decision["legacy_approval"]["sha256"] or approval.stat().st_size != decision["legacy_approval"]["bytes"]:
        raise RunnerError("reference decision legacy approval file changed")
    return [decision], REFERENCE_DECISIONS_SHA256


def load_extension_manifest(path: Path, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
    if check_work is not None:
        check_work()
    try:
        manifest = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise RunnerError(f"could not read extension manifest: {path}") from error
    if not isinstance(manifest, dict) or manifest.get("schema_version") != 1:
        raise RunnerError("unsupported extension manifest schema")
    if set(manifest) != {"schema_version", "extension_id_prefix", "journeys", "sizes", "color_modes", "generated_cases"}:
        raise RunnerError("extension manifest fields differ from schema version 1")
    if manifest.get("extension_id_prefix") != EXTENSION_CASE_PREFIX:
        raise RunnerError("extension manifest ID prefix changed")
    journeys = manifest.get("journeys")
    sizes = manifest.get("sizes")
    colors = manifest.get("color_modes")
    generated = manifest.get("generated_cases")
    if not isinstance(journeys, list) or not journeys or not isinstance(sizes, list) or not sizes or not isinstance(colors, list) or not colors or not isinstance(generated, list) or not generated:
        raise RunnerError("extension manifest has empty or invalid journeys, sizes, color modes, or generated cases")
    journey_checkpoints: dict[str, list[str]] = {}
    journey_ids: set[str] = set()
    for journey in journeys:
        if check_work is not None:
            check_work()
        if not isinstance(journey, dict) or set(journey) != {"id", "title", "checkpoints"} or not isinstance(journey.get("id"), str) or not journey["id"] or not isinstance(journey.get("title"), str):
            raise RunnerError("invalid extension journey")
        journey_id = journey["id"]
        if journey_id in journey_ids:
            raise RunnerError(f"duplicate extension journey: {journey_id}")
        journey_ids.add(journey_id)
        checkpoints = journey.get("checkpoints")
        if not isinstance(checkpoints, list) or not checkpoints or any(not isinstance(item, str) or not item for item in checkpoints) or len(set(checkpoints)) != len(checkpoints):
            raise RunnerError(f"invalid ordered checkpoints for extension journey: {journey_id}")
        journey_checkpoints[journey_id] = checkpoints
    size_pairs: set[tuple[int, int]] = set()
    narrow_sizes = {(40, 12), (60, 18)}
    for size in sizes:
        if not isinstance(size, dict) or set(size) != {"cols", "rows", "allow_narrow_fallback"} or type(size.get("cols")) is not int or type(size.get("rows")) is not int or size["cols"] < 1 or size["rows"] < 1 or not isinstance(size.get("allow_narrow_fallback"), bool):
            raise RunnerError("invalid extension terminal size")
        pair = (size["cols"], size["rows"])
        if pair in size_pairs:
            raise RunnerError(f"duplicate extension terminal size: {pair}")
        if size["allow_narrow_fallback"] != (pair in narrow_sizes):
            raise RunnerError(f"incorrect narrow fallback policy for extension terminal size: {pair}")
        size_pairs.add(pair)
    if size_pairs != set(EXTENSION_SIZES):
        raise RunnerError("extension terminal-size inventory changed")
    if any(not isinstance(color, str) for color in colors) or len(set(colors)) != len(colors) or any(color not in {"truecolor", "256", "16", "none", "nocolor"} for color in colors):
        raise RunnerError("invalid or duplicate extension color mode")
    if tuple(colors) != EXTENSION_COLORS:
        raise RunnerError("extension color-mode inventory or order changed")
    generated_ids: set[str] = set()
    generated_combinations: set[tuple[str, int, int, str]] = set()
    for case in generated:
        if check_work is not None:
            check_work()
        if not isinstance(case, dict) or set(case) != {"id", "journey_id", "cols", "rows", "color_mode", "checkpoints"} or not isinstance(case.get("id"), str) or not case["id"].startswith(EXTENSION_CASE_PREFIX) or not isinstance(case.get("journey_id"), str):
            raise RunnerError("extension case ID is missing the reserved prefix")
        case_id = case["id"]
        if case_id in generated_ids:
            raise RunnerError(f"duplicate extension case ID: {case_id}")
        generated_ids.add(case_id)
        journey_id = case.get("journey_id")
        if journey_id not in journey_checkpoints:
            raise RunnerError(f"extension case references unknown journey: {journey_id}")
        if type(case.get("cols")) is not int or type(case.get("rows")) is not int or (case["cols"], case["rows"]) not in size_pairs:
            raise RunnerError(f"extension case references unknown terminal size: {case_id}")
        if case.get("color_mode") not in colors:
            raise RunnerError(f"extension case references unknown color mode: {case_id}")
        width, height = case["cols"], case["rows"]
        expected_suffix = f"{journey_id}::{width}x{height}::{case['color_mode']}"
        if case_id != EXTENSION_CASE_PREFIX + expected_suffix:
            raise RunnerError(f"extension case ID does not encode its declared inputs: {case_id}")
        generated_combinations.add((journey_id, width, height, case["color_mode"]))
        expected_checkpoints = journey_checkpoints[journey_id]
        if width < 120:
            if not expected_checkpoints or expected_checkpoints[0] != "initial":
                raise RunnerError(f"sub-120 extension journey must begin with initial checkpoint: {journey_id}")
            reveal_checkpoint = "narrow_revealed_120x40" if (width, height) in narrow_sizes else "table_revealed_120x40"
            allowed_checkpoints = ["initial", reveal_checkpoint, *expected_checkpoints[1:]]
        else:
            reveal_checkpoint = None
            allowed_checkpoints = expected_checkpoints
        case_checkpoints = case.get("checkpoints")
        if not isinstance(case_checkpoints, list) or not case_checkpoints or any(not isinstance(item, str) or not item for item in case_checkpoints) or len(set(case_checkpoints)) != len(case_checkpoints):
            raise RunnerError(f"invalid case checkpoint obligations: {case_id}")
        if case_checkpoints != allowed_checkpoints:
            raise RunnerError(f"extension case checkpoint obligations differ from exact journey checkpoints: {case_id}")
    expected_combinations = {
        (journey_id, width, height, color)
        for journey_id in journey_ids
        for width, height in EXTENSION_SIZES
        for color in (("truecolor", "nocolor") if (width, height) in narrow_sizes else EXTENSION_COLORS)
    }
    if generated_combinations != expected_combinations:
        raise RunnerError(f"extension case matrix incomplete: missing={sorted(expected_combinations - generated_combinations)[:5]}, extra={sorted(generated_combinations - expected_combinations)[:5]}")
    if check_work is not None:
        check_work()
    return {**manifest, "generated_case_ids": sorted(generated_ids), "checkpoint_obligations": {case["id"]: case["checkpoints"] for case in generated}}


def validate_extension_inventory(inventory: dict[str, bool], manifest: dict[str, object], *, check_work: Callable[[], None] | None = None) -> tuple[dict[str, bool], list[str]]:
    extension_ids = set(manifest["generated_case_ids"])
    if len(inventory) != EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS + len(extension_ids):
        raise RunnerError("registered inventory count differs from original inventory plus declared extensions")
    extension_inventory: dict[str, bool] = {}
    for name in inventory:
        if check_work is not None:
            check_work()
        if name in extension_ids:
            extension_inventory[name] = inventory[name]
    missing = extension_ids - set(extension_inventory)
    if missing:
        raise RunnerError(f"extension manifest cases missing from Nextest inventory: {sorted(missing)[:5]}")
    if any(not ignored for ignored in extension_inventory.values()):
        raise RunnerError("extension visual cases must be ignored Nextest scenarios")
    original: dict[str, bool] = {}
    for name, ignored in inventory.items():
        if check_work is not None:
            check_work()
        if name not in extension_ids:
            original[name] = ignored
    if len(original) != EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS:
        raise RunnerError("extension IDs overlap or alter the original registered inventory")
    scenarios = sum(original.values())
    ordinary = len(original) - scenarios
    if (scenarios, ordinary) != (EXPECTED_SCENARIOS, EXPECTED_ORDINARY_TESTS):
        raise RunnerError(f"original registered coverage changed: {scenarios} scenarios and {ordinary} ordinary")
    return original, sorted(extension_ids)


def inventory_digest(inventory: dict[str, bool], *, check_work: Callable[[], None] | None = None) -> str:
    ordered: list[tuple[str, bool]] = []
    for name, ignored in sorted(inventory.items()):
        if check_work is not None:
            check_work()
        ordered.append((name, ignored))
    serialized = json.dumps(ordered, separators=(",", ":")).encode()
    return hashlib.sha256(serialized).hexdigest()


def require_inventory_matches(inventory: dict[str, bool], expected_digest: str, *, check_work: Callable[[], None] | None = None) -> None:
    actual = inventory_digest(inventory, check_work=check_work)
    if actual != expected_digest:
        raise RunnerError(f"Nextest inventory hash mismatch: expected {expected_digest}, got {actual}")


def selected_expression(names: list[str]) -> str:
    if not names:
        raise RunnerError("cannot build an empty exact Nextest selection")
    return " | ".join(f"test(={name})" for name in names)


def scenario_case_id(test_name: str) -> str:
    return test_name.replace("::", "/")


def scenario_receipt_id(test_name: str, extension_manifest: dict[str, object] | None = None) -> str:
    if extension_manifest is not None and test_name in extension_manifest["generated_case_ids"]:
        return test_name
    return scenario_case_id(test_name)


def nextest_events(stdout: str, selected: list[str]) -> dict[str, str]:
    prefix = f"{TEST_PACKAGE}::{TEST_TARGET}$"
    expected = {prefix + name for name in selected}
    events: dict[str, str] = {}
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except ValueError as error:
            raise RunnerError("invalid JSON event on Nextest stdout") from error
        if event.get("type") != "test" or event.get("event") not in {"ok", "failed", "ignored", "timeout", "exec-failed"}:
            continue
        name = event.get("name")
        if name not in expected:
            if event["event"] == "ignored":
                continue
            raise RunnerError(f"Nextest emitted an unselected terminal test event: {name}")
        if name in events:
            raise RunnerError(f"Nextest emitted duplicate terminal test event: {name}")
        events[name] = event["event"]
    if set(events) != expected:
        missing = sorted(expected - set(events))
        extra = sorted(set(events) - expected)
        raise RunnerError(f"Nextest terminal event coverage mismatch: missing={missing[:3]}, extra={extra[:3]}")
    failed = sorted(name for name, result in events.items() if result != "ok")
    if failed:
        raise RunnerError(f"Nextest selected tests did not all pass: {failed[:5]}")
    return events


def clean_env() -> dict[str, str]:
    blocked = {"BLESS", "COMBO_FILTER", "MIDTEST_BUILD", "PRINT_PAGES", "REACH_DUMP", "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET"}
    blocked_prefixes = ("VISUAL_", "TERMROCK_PAIR_", "UPDATE_", "NEXTEST_", "NEXTTEST_", "DUMP_", "SHOWCASE_", "INSTA_", "SNAPSHOT", "TUISNAP", "TUISCOTTI", "JUNIE_TUI_", "CARGO_PROFILE_")
    return {key: value for key, value in os.environ.items() if key not in blocked and not key.startswith(blocked_prefixes)}


def fresh_clean_env() -> dict[str, str]:
    """Use only the explicit fresh-runner environment allowlist."""
    base = clean_env()
    return {key: value for key, value in base.items() if key in FRESH_RUNTIME_ENV_KEYS}


def fresh_environment_fingerprint(env: dict[str, str]) -> dict[str, object]:
    observed = {
        key: {"present": key in env, "value_sha256": hashlib.sha256(env[key].encode()).hexdigest() if key in env else None}
        for key in FRESH_RUNTIME_ENV_KEYS
    }
    return {
        "policy": "only-explicit-allowlisted-keys-passed; omitted-parent-keys-are-removed",
        "keys": list(FRESH_RUNTIME_ENV_KEYS),
        "values": observed,
        "sha256": canonical_json_digest(observed),
        "raw_values_recorded": False,
        "unknown_parent_environment_values_recorded": False,
    }


TERM_PROFILES = {"xterm-256color", "linux", "dumb", "unset"}


def apply_term_profile(env: dict[str, str], term_profile: str | None) -> dict[str, object]:
    if term_profile is not None:
        if term_profile not in TERM_PROFILES:
            raise RunnerError(f"unsupported TERM profile: {term_profile}")
        if term_profile == "unset":
            env.pop("TERM", None)
        else:
            env["TERM"] = term_profile
    child_term = None if term_profile == "unset" else (term_profile or "xterm-256color")
    effective = {
        "profile": term_profile or "harness-default",
        "runner_term_present": "TERM" in env,
        "runner_term": env.get("TERM"),
        "child_term_present": child_term is not None,
        "child_term": child_term,
    }
    effective["sha256"] = hashlib.sha256(json.dumps(effective, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return effective


def validate_reference_term_profile(decisions: list[dict[str, object]], selected: list[str], term_profile: str | None) -> None:
    for decision in decisions:
        if reference_case_selected(decision, selected):
            if term_profile is not None:
                raise RunnerError("the pinned grown-reference exception was reviewed with the harness-default child TERM; profiled runs must exclude that case")


def tree_digest_or_empty(path: Path) -> str:
    return digest_tree(path) if path.exists() else hashlib.sha256(b"").hexdigest()


def archive_artifacts(source: Path, run_root: Path, phase: str) -> Path:
    dest = run_root / phase
    if dest.exists():
        raise RunnerError(f"refusing to overwrite saved {phase} artifacts: {dest}")
    dest.mkdir(parents=True)
    for name in ("actual", "diff"):
        source_path = source / "target" / "tuiscotti" / name
        if source_path.exists():
            shutil.copytree(source_path, dest / name)
    return dest


def archive_live_artifacts(source: Path, chunk_root: Path, receipt_root: Path) -> Path:
    dest = chunk_root / "artifacts"
    if dest.exists():
        raise RunnerError(f"refusing to overwrite saved live artifacts: {dest}")
    dest.mkdir(parents=True)
    candidates = (
        ("harness-actual", source / "target" / "tuiscotti" / "actual"),
        ("harness-diff", source / "target" / "tuiscotti" / "diff"),
        ("runtime-actual", receipt_root.with_name(receipt_root.name + ".actual")),
        ("runtime-semantic-actual", receipt_root.with_name(receipt_root.name + ".semantic-actual")),
        ("runtime-diff", receipt_root.with_name(receipt_root.name + ".diff")),
        ("original-actual", receipt_root.with_name(receipt_root.name + ".original-actual")),
        ("original-diff", receipt_root.with_name(receipt_root.name + ".original-diff")),
    )
    for name, path in candidates:
        if path.exists() or path.is_symlink():
            if path.is_symlink():
                raise RunnerError(f"artifact root is a symlink: {path}")
            shutil.copytree(path, dest / name, symlinks=True)
    return dest


def frame_artifacts(path: Path) -> dict[str, bytes]:
    result = {}
    actual = path / "actual"
    for file in actual.rglob("*") if actual.is_dir() else ():
        if file.is_file() and (file.name.endswith(".ansi") or file.name.endswith(".frame.json")):
            result[file.relative_to(actual).as_posix()] = file.read_bytes()
    return result


def exact_frame_differences(baseline: Path, candidate: Path) -> list[str]:
    left = frame_artifacts(baseline)
    right = frame_artifacts(candidate)
    differences = []
    for name in sorted(set(left) | set(right)):
        if name not in left:
            differences.append(f"candidate-only:{name}")
        elif name not in right:
            differences.append(f"baseline-only:{name}")
        elif left[name] != right[name]:
            differences.append(f"frame-bytes-differ:{name}")
    if not left or not right:
        differences.append("missing-canonical-frame-artifacts")
    return differences


def frame_digest(path: Path) -> str:
    h = hashlib.sha256()
    for name, content in sorted(frame_artifacts(path).items()):
        h.update(name.encode())
        h.update(b"\0")
        h.update(content)
        h.update(b"\0")
    return h.hexdigest()


def valid_color(value: object) -> bool:
    if value == "Default":
        return True
    if not isinstance(value, dict) or len(value) != 1:
        return False
    if "Indexed" in value:
        return type(value["Indexed"]) is int and 0 <= value["Indexed"] <= 255
    if "Rgb" in value:
        rgb = value["Rgb"]
        return isinstance(rgb, dict) and set(rgb) == {"r", "g", "b"} and all(type(rgb[channel]) is int and 0 <= rgb[channel] <= 255 for channel in ("r", "g", "b"))
    return False


def validate_semantic_frame_file(path: Path, digest: str) -> dict[str, object]:
    if path.is_symlink() or not path.is_file() or path.stat().st_nlink != 1:
        raise RunnerError(f"semantic frame missing or not an isolated regular file: {path}")
    payload = path.read_bytes()
    if digest and hashlib.sha256(payload).hexdigest() != digest:
        raise RunnerError(f"semantic frame hash mismatch: {path}")
    try:
        frame = json.loads(payload)
    except (UnicodeDecodeError, ValueError) as error:
        raise RunnerError(f"invalid semantic frame JSON: {path}") from error
    if not isinstance(frame, dict) or set(frame) != {"version", "cols", "rows", "cells", "cursor", "provenance"}:
        raise RunnerError(f"semantic frame fields differ from Frame JSON schema: {path}")
    cols, rows = frame["cols"], frame["rows"]
    if type(frame["version"]) is not int or frame["version"] != 3 or type(cols) is not int or type(rows) is not int or cols < 1 or rows < 1:
        raise RunnerError(f"invalid semantic frame dimensions or version: {path}")
    cells = frame["cells"]
    if not isinstance(cells, list) or len(cells) != cols * rows:
        raise RunnerError(f"semantic frame cell inventory mismatch: {path}")
    for index, cell in enumerate(cells):
        x, y = index % cols, index // cols
        required_cell_fields = {"x", "y", "symbol", "width", "continuation", "fg", "bg", "mods"}
        allowed_cell_fields = (required_cell_fields, required_cell_fields | {"underline_color"})
        if not isinstance(cell, dict) or set(cell) not in allowed_cell_fields or type(cell.get("x")) is not int or type(cell.get("y")) is not int or cell.get("x") != x or cell.get("y") != y or not isinstance(cell.get("symbol"), str) or type(cell.get("width")) is not int or not isinstance(cell.get("continuation"), bool) or not valid_color(cell.get("fg")) or not valid_color(cell.get("bg")) or not isinstance(cell.get("mods"), dict) or ("underline_color" in cell and not valid_color(cell["underline_color"])):
            raise RunnerError(f"invalid semantic frame cell {index}: {path}")
    cursor = frame["cursor"]
    if not isinstance(cursor, dict) or set(cursor) != {"x", "y", "visible", "style", "blinking"} or type(cursor.get("x")) is not int or type(cursor.get("y")) is not int or not isinstance(cursor.get("visible"), bool) or not isinstance(cursor.get("style"), str) or not isinstance(cursor.get("blinking"), bool):
        raise RunnerError(f"invalid semantic frame cursor: {path}")
    if cursor["x"] >= cols or cursor["y"] >= rows or not isinstance(frame["provenance"], dict):
        raise RunnerError(f"invalid semantic frame cursor bounds or provenance: {path}")
    return frame


def actual_semantic_digest(path: Path) -> str:
    frame = validate_semantic_frame_file(path, "")
    frame["provenance"] = {
        "tool": "tuiscotti",
        "tool_version": "0.2.0",
        "profile": "tuiscotti-default",
        "source": "screen",
        "argv": [],
        "created_unix": 0,
    }
    canonical = json.dumps(frame, ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(canonical).hexdigest()


def read_receipts(receipt_root: Path, selected: list[str], expected_root: Path) -> dict[tuple[str, int, str], tuple[str, str, str | None]]:
    _, receipt_entries = sealed_tree(receipt_root)
    checkpoint_entries = {entry for entry in receipt_entries if not entry.startswith("process/") and entry != "process/"}
    actual_root = receipt_root.with_name(receipt_root.name + ".actual")
    _, actual_entries = flat_sealed_store(actual_root, "runtime actual")
    semantic_actual_root = receipt_root.with_name(receipt_root.name + ".semantic-actual")
    _, semantic_actual_entries = flat_sealed_store(semantic_actual_root, "runtime semantic actual")
    receipts: dict[tuple[str, int, str], tuple[str, str, str | None]] = {}
    expected_receipt_entries: set[str] = set()
    expected_actual_entries: set[str] = set()
    expected_semantic_actual_entries: set[str] = set()
    expected_cases = set(selected)
    for path in sorted(receipt_root.rglob("*.json")):
        if path.relative_to(receipt_root).parts[0] == "process":
            continue
        try:
            record = json.loads(path.read_text())
            case = record["case_id"]
            ordinal = record["ordinal"]
            logical = record["logical_name"]
            ansi_digest = record["sha256"]
            semantic_digest = record["semantic_frame_sha256"]
            process_digest = record["process_receipt_sha256"]
            reference_digest = record.get("reference_decision_sha256")
            if not isinstance(case, str) or type(ordinal) is not int or ordinal < 0 or not isinstance(logical, str) or not isinstance(ansi_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", ansi_digest) or not isinstance(semantic_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", semantic_digest) or not isinstance(process_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", process_digest) or (reference_digest is not None and (not isinstance(reference_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", reference_digest))):
                raise TypeError("invalid receipt field")
            if set(record) not in ({"case_id", "ordinal", "logical_name", "sha256", "semantic_frame_sha256", "process_receipt_sha256"}, {"case_id", "ordinal", "logical_name", "sha256", "semantic_frame_sha256", "process_receipt_sha256", "reference_decision_sha256"}):
                raise TypeError("unexpected receipt field")
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise RunnerError(f"invalid checkpoint receipt: {path}") from error
        if case not in expected_cases:
            raise RunnerError(f"receipt belongs to an unselected scenario: {case}")
        key = (case, ordinal, logical)
        if key in receipts:
            raise RunnerError(f"duplicate checkpoint receipt: {key}")
        case_hash = hashlib.sha256(case.encode()).hexdigest()
        checkpoint = f"checkpoint-{ordinal:08}-{checkpoint_identity(case, ordinal, logical)}"
        expected_receipt = receipt_root / case_hash / f"{checkpoint}.json"
        if path != expected_receipt:
            raise RunnerError(f"checkpoint receipt path does not match its identity: {path}")
        expected_receipt_entries.add(path.relative_to(receipt_root).as_posix())
        expected_ansi = expected_root / f"{checkpoint}.ansi"
        if expected_ansi.is_symlink() or not expected_ansi.is_file() or expected_ansi.stat().st_nlink != 1 or sha256(expected_ansi) != ansi_digest:
            raise RunnerError(f"receipt ANSI frame missing or hash mismatch: {expected_ansi}")
        expected_semantic = expected_root / f"{checkpoint}.frame.json"
        validate_semantic_frame_file(expected_semantic, semantic_digest)
        actual_semantic = actual_root / f"{checkpoint}.frame.json"
        if actual_semantic.exists() or actual_semantic.is_symlink():
            if actual_semantic.is_symlink() or not actual_semantic.is_file() or actual_semantic.stat().st_nlink != 1 or actual_semantic_digest(actual_semantic) != semantic_digest:
                raise RunnerError(f"actual semantic frame hash mismatch or unsafe: {actual_semantic}")
        semantic_actual = semantic_actual_root / f"{checkpoint}.frame.json"
        validate_semantic_frame_file(semantic_actual, semantic_digest)
        actual_ansi = actual_root / f"{checkpoint}.ansi"
        if actual_ansi.is_symlink() or not actual_ansi.is_file() or actual_ansi.stat().st_nlink != 1 or sha256(actual_ansi) != ansi_digest:
            raise RunnerError(f"actual ANSI frame missing or hash mismatch: {actual_ansi}")
        expected_actual_entries.add(actual_ansi.name)
        if actual_semantic.is_file():
            expected_actual_entries.add(actual_semantic.name)
        expected_semantic_actual_entries.add(semantic_actual.name)
        receipts[key] = (ansi_digest, semantic_digest, reference_digest)
    counts = {case: 0 for case in selected}
    for case, _, _ in receipts:
        counts[case] += 1
    missing = sorted(case for case, count in counts.items() if count == 0)
    if missing:
        raise RunnerError(f"successful scenarios missing checkpoint receipts: {missing[:5]}")
    expected_receipt_dirs = {hashlib.sha256(case.encode()).hexdigest() + "/" for case, _, _ in receipts}
    expected_receipt_entries.update(expected_receipt_dirs)
    if checkpoint_entries != expected_receipt_entries:
        raise RunnerError(f"checkpoint receipt inventory mismatch: extra={sorted(checkpoint_entries - expected_receipt_entries)[:3]}, missing={sorted(expected_receipt_entries - checkpoint_entries)[:3]}")
    actual_frame_entries = {entry for entry in actual_entries if entry.endswith(".ansi")}
    if actual_frame_entries != {entry for entry in expected_actual_entries if entry.endswith(".ansi")}:
        raise RunnerError(f"runtime actual ANSI inventory mismatch: extra={sorted(actual_frame_entries - expected_actual_entries)[:3]}, missing={sorted({entry for entry in expected_actual_entries if entry.endswith('.ansi')} - actual_frame_entries)[:3]}")
    optional_raw_frame_entries = {entry for entry in actual_entries if entry.endswith(".frame.json")}
    expected_raw_frame_entries = {entry for entry in expected_actual_entries if entry.endswith(".frame.json")}
    if optional_raw_frame_entries - expected_raw_frame_entries:
        raise RunnerError(f"runtime actual Frame inventory mismatch: extra={sorted(optional_raw_frame_entries - expected_raw_frame_entries)[:3]}")
    if semantic_actual_entries != expected_semantic_actual_entries:
        raise RunnerError(f"runtime semantic actual inventory mismatch: extra={sorted(semantic_actual_entries - expected_semantic_actual_entries)[:3]}, missing={sorted(expected_semantic_actual_entries - semantic_actual_entries)[:3]}")
    if not receipts:
        raise RunnerError("phase produced no checkpoint receipts")
    return receipts


def canonical_receipt_json(record: dict[str, object]) -> bytes:
    return json.dumps(record, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def process_app_for_case(case: str, extension_manifest: dict[str, object] | None) -> str:
    if extension_manifest is not None and case in extension_manifest["generated_case_ids"]:
        if not case.startswith(EXTENSION_CASE_PREFIX):
            raise RunnerError(f"extension process case is outside the pinned app prefix: {case}")
        return "showcase"
    app = case.split("::", 1)[0]
    if app == "jackin":
        app = "jackin-preview"
    if app not in APPS:
        raise RunnerError(f"process receipt case has an unregistered application: {case}")
    return app


def verify_process_receipts(
    receipt_root: Path,
    selected: list[str],
    phase: str,
    bins: Path,
    expected_hashes: dict[str, str],
    extension_manifest: dict[str, object] | None,
    run_id: str,
    key_hex: str,
    checkpoint_receipts: dict[tuple[str, int, str], tuple[str, str, str | None]],
) -> tuple[dict[str, list[str]], dict[str, object]]:
    if not re.fullmatch(r"[0-9a-f]{64}", key_hex):
        raise RunnerError("paired process receipt key is invalid")
    key = bytes.fromhex(key_hex)
    process_root = receipt_root / "process"
    _, entries = sealed_tree(process_root)
    selected_ids = [scenario_receipt_id(name, extension_manifest) for name in selected]
    expected = {case: process_app_for_case(name, extension_manifest) for name, case in zip(selected, selected_ids, strict=True)}
    launches: dict[str, list[tuple[int, dict[str, object], str]]] = {case: [] for case in selected_ids}
    closures: dict[str, list[tuple[int, dict[str, object]]]] = {case: [] for case in selected_ids}
    known_launch_fields = {
        "app", "argv_provenance", "case_id", "constructed_argv_hmac_sha256", "constructed_env_hmac_sha256",
        "constructed_env_keys", "expected_file_identity", "expected_path", "expected_sha256", "image_hash_source", "observed_file_identity", "observed_path",
        "observed_sha256", "process_start_identity", "previous_receipt_sha256", "run_id", "phase", "pid", "ordinal", "schema", "hmac_sha256",
    }
    known_close_fields = {"case_id", "launch_receipt_sha256", "ordinal", "phase", "run_id", "schema", "hmac_sha256"}

    def read_signed(path: Path, fields: set[str]) -> dict[str, object]:
        if path.is_symlink() or not path.is_file() or path.stat().st_nlink != 1:
            raise RunnerError(f"process receipt is not an isolated regular file: {path}")
        raw = path.read_bytes()
        try:
            record = json.loads(raw)
        except (UnicodeDecodeError, ValueError) as error:
            raise RunnerError(f"process receipt is invalid JSON: {path}") from error
        if not isinstance(record, dict) or set(record) != fields:
            raise RunnerError(f"process receipt fields differ from schema: {path}")
        signature = record.get("hmac_sha256")
        if not isinstance(signature, str) or not re.fullmatch(r"[0-9a-f]{64}", signature):
            raise RunnerError(f"process receipt signature is missing or malformed: {path}")
        payload = dict(record)
        payload.pop("hmac_sha256")
        expected_signature = hmac.new(key, canonical_receipt_json(payload), hashlib.sha256).hexdigest()
        if not hmac.compare_digest(signature, expected_signature):
            raise RunnerError(f"process receipt authentication failed: {path}")
        return record

    expected_entries: set[str] = set()
    for case in selected_ids:
        case_hash = hashlib.sha256(case.encode()).hexdigest()
        launch_dir = process_root / case_hash
        for kind in ("launch", "closed"):
            matches = sorted(launch_dir.glob(f"{kind}-*.json")) if launch_dir.is_dir() and not launch_dir.is_symlink() else []
            if not matches:
                raise RunnerError(f"process receipt inventory is missing {kind} record for {case}")
            for path in matches:
                relative = path.relative_to(process_root).as_posix()
                match = re.fullmatch(rf"{kind}-(\d{{8}})\.json", path.name)
                if not match:
                    raise RunnerError(f"process receipt filename is malformed: {path}")
                ordinal = int(match.group(1))
                record = read_signed(path, known_launch_fields if kind == "launch" else known_close_fields)
                if record.get("case_id") != case or record.get("phase") != phase or record.get("run_id") != run_id or record.get("schema") != 1 or record.get("ordinal") != ordinal:
                    raise RunnerError(f"process receipt identity or ordering mismatch: {path}")
                expected_entries.add(relative)
                if kind == "launch":
                    app = expected[case]
                    expected_path = str((bins / app).resolve())
                    if record.get("app") != app or record.get("expected_path") != expected_path or record.get("observed_path") != expected_path:
                        raise RunnerError(f"process executable path does not match {phase} phase: {path}")
                    if record.get("expected_sha256") != expected_hashes.get(app) or record.get("observed_sha256") != expected_hashes.get(app):
                        raise RunnerError(f"process executable hash does not match {phase} phase: {path}")
                    if not isinstance(record.get("expected_file_identity"), str) or record.get("observed_file_identity") != record.get("expected_file_identity"):
                        raise RunnerError(f"process executable file identity changed during spawn: {path}")
                    if type(record.get("pid")) is not int or record["pid"] <= 0:
                        raise RunnerError(f"process receipt PID is invalid: {path}")
                    process_start = record.get("process_start_identity")
                    start_pattern = r"[0-9]+" if sys.platform.startswith("linux") else r"[0-9]+:[0-9]+" if sys.platform == "darwin" else r"(?!)"
                    if not isinstance(process_start, str) or re.fullmatch(start_pattern, process_start) is None:
                        raise RunnerError(f"process start identity is missing: {path}")
                    if record.get("argv_provenance") != "tuiscotti-builder-input" or not isinstance(record.get("constructed_argv_hmac_sha256"), str) or not re.fullmatch(r"[0-9a-f]{64}", record["constructed_argv_hmac_sha256"]):
                        raise RunnerError(f"constructed argv receipt is invalid: {path}")
                    if not isinstance(record.get("constructed_env_hmac_sha256"), str) or not re.fullmatch(r"[0-9a-f]{64}", record["constructed_env_hmac_sha256"]):
                        raise RunnerError(f"constructed environment receipt is invalid: {path}")
                    env_keys = record.get("constructed_env_keys")
                    if not isinstance(env_keys, list) or any(not isinstance(name, str) for name in env_keys) or env_keys != sorted(set(env_keys)) or any(re.search(r"(SECRET|TOKEN|PASSWORD|CREDENTIAL|PRIVATE|API_KEY)", name, re.IGNORECASE) for name in env_keys):
                        raise RunnerError(f"constructed environment key inventory is unsafe: {path}")
                    allowed_probe = "linux-proc-exe-open-fd" if sys.platform.startswith("linux") else "macos-proc-pidpath-file" if sys.platform == "darwin" else None
                    if allowed_probe is None or record.get("image_hash_source") != allowed_probe:
                        raise RunnerError("paired process image probing is unsupported or weaker than the platform contract")
                    if not isinstance(record.get("previous_receipt_sha256"), (str, type(None))) or (record.get("previous_receipt_sha256") is not None and not re.fullmatch(r"[0-9a-f]{64}", record["previous_receipt_sha256"])):
                        raise RunnerError(f"process receipt chain link is invalid: {path}")
                    receipt_digest = sha256(path)
                    launches[case].append((ordinal, record, receipt_digest))
                else:
                    closures[case].append((ordinal, record))

    expected_tree_entries = expected_entries | {f"{hashlib.sha256(case.encode()).hexdigest()}/" for case in selected_ids}
    if entries != expected_tree_entries:
        unexpected = sorted(entries - expected_tree_entries)
        missing = sorted(expected_tree_entries - entries)
        raise RunnerError(f"process receipt inventory mismatch: extra={unexpected[:3]}, missing={missing[:3]}")

    process_links: dict[str, list[str]] = {}
    all_pids: set[int] = set()
    for case, records in launches.items():
        records.sort(key=lambda row: row[0])
        closed = sorted(closures[case], key=lambda row: row[0])
        if not records or [ordinal for ordinal, _, _ in records] != list(range(len(records))):
            raise RunnerError(f"process launch sequence is incomplete for {case}")
        if [ordinal for ordinal, _ in closed] != list(range(len(records))):
            raise RunnerError(f"process session completion sequence is incomplete for {case}")
        previous = None
        hashes = []
        for ordinal, record, digest in records:
            if record.get("previous_receipt_sha256") != previous:
                raise RunnerError(f"process launch chain is broken for {case} ordinal {ordinal}")
            pid = record["pid"]
            if pid in all_pids:
                raise RunnerError(f"paired child PID was reused within the chunk: {pid}")
            all_pids.add(pid)
            close_record = closed[ordinal][1]
            if close_record.get("launch_receipt_sha256") != digest:
                raise RunnerError(f"process completion does not bind launch receipt for {case} ordinal {ordinal}")
            previous = digest
            hashes.append(digest)
        process_links[case] = hashes

    linked_checkpoints: set[tuple[str, int, str]] = set()
    for path in sorted(receipt_root.rglob("*.json")):
        if path.relative_to(receipt_root).parts[0] == "process":
            continue
        record = json.loads(path.read_text())
        case = record.get("case_id")
        logical = record.get("logical_name")
        ordinal = record.get("ordinal")
        if not isinstance(case, str) or not isinstance(logical, str) or type(ordinal) is not int:
            raise RunnerError(f"checkpoint process link identity is invalid: {path}")
        linked_checkpoints.add((case, ordinal, logical))
        if record.get("process_receipt_sha256") not in process_links.get(case, []):
            raise RunnerError(f"checkpoint receipt is not linked to its launched process: {path}")
    if not checkpoint_receipts or linked_checkpoints != set(checkpoint_receipts):
        raise RunnerError("process receipts cannot pass without the complete checkpoint receipt inventory")
    digest, _ = sealed_tree(process_root)
    return process_links, {"receipt_count": sum(map(len, process_links.values())), "inventory_sha256": digest, "pid_count": len(all_pids)}


def clear_runtime_artifacts(source: Path) -> None:
    runtime = source / "target" / "tuiscotti"
    for name in ("actual", "diff"):
        path = runtime / name
        if path.exists():
            shutil.rmtree(path)


def paired_report(source_hash: str, snapshots_hash: str, snapshots_tree_hash: str, harness_hash: str, expected_path: Path, selection: str, expected_count: int, candidate: dict[str, object], phases: list[dict[str, object]]) -> dict[str, object]:
    return {
        "tag_object": BASELINE_TAG_OBJECT,
        "baseline_commit": BASELINE_COMMIT,
        "tuiscotti_rev": TUISCOTTI_REV,
        "baseline_source_sha256": source_hash,
        "expected_snapshot_sha256": snapshots_hash,
        "snapshot_tree_sha256": snapshots_tree_hash,
        "expected_snapshot": str(expected_path),
        "harness_sha256": harness_hash,
        "case_selection": selection,
        "selection_completeness": "partial",
        "acceptance_complete": False,
        "selected_test_count": expected_count,
        "candidate_revision": candidate["revision"],
        "candidate_dirty_fingerprint": candidate["dirty_fingerprint"],
        "candidate_dirty_files": candidate["dirty_files"],
        "candidate_binaries": candidate["binaries"],
        "phases": phases,
    }


def compare(repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, selector: str | None, all_cases: bool, timeout: int) -> None:
    repo, root, source = paths(repo, root)
    source_hash = verify_source(root, source)
    build_record = json.loads((root / "build.json").read_text())
    if build_record.get("source_sha256") != source_hash:
        raise RunnerError("frozen build provenance does not match extracted source")
    if baseline_bins.resolve() != (root / "cargo-target" / "debug").resolve():
        raise RunnerError("baseline binaries must be the isolated binaries built by this runner")
    if build_record.get("binaries") != {app: sha256(baseline_bins / app) for app in APPS}:
        raise RunnerError("frozen baseline binaries changed after the recorded build")
    candidate_record_path = root / "candidate-build.json"
    if not candidate_record_path.is_file():
        raise RunnerError("candidate build provenance missing; run build-candidate")
    candidate = json.loads(candidate_record_path.read_text())
    if Path(candidate["bin_dir"]).resolve() != candidate_bins.resolve():
        raise RunnerError("candidate binary directory does not match candidate-build.json")
    candidate_revision, candidate_dirty_hash, candidate_dirty_files = git_state(repo, timeout)
    if candidate_revision != candidate.get("revision") or candidate_dirty_hash != candidate.get("dirty_fingerprint") or candidate_dirty_files != candidate.get("dirty_files"):
        raise RunnerError("candidate source changed after its recorded build")
    expected_candidate_hashes = candidate.get("binaries", {})
    if expected_candidate_hashes != {app: sha256(candidate_bins / app) for app in APPS}:
        raise RunnerError("candidate binaries changed after their recorded build")
    if baseline_bins.resolve() == candidate_bins.resolve():
        raise RunnerError("baseline and candidate binary directories alias")
    for app in APPS:
        for directory in (baseline_bins, candidate_bins):
            binary = directory / app
            if not binary.is_file():
                raise RunnerError(f"missing binary: {binary}")
    if not candidate_bins.is_dir():
        raise RunnerError(f"candidate bin directory missing: {candidate_bins}")
    env = clean_env()
    env.update({"VISUAL_FULL": "1", "VISUAL_REVIEW": "0", "VISUAL_BIN_DIR": str(baseline_bins)})
    env["CARGO_TARGET_DIR"] = str(root / "cargo-target")
    expression, expected_count = nextest_selection(source, root, selector, all_cases, timeout, env)
    if all_cases:
        raise RunnerError("--all is incomplete: the frozen store has pending scenarios without approved snapshots; select approved cases with --case")
    expected_path = source / "snapshots" / f"{selector.replace('::', '/')}.ansi"
    if not expected_path.is_file():
        raise RunnerError(f"selected frozen scenario has no committed ANSI snapshot: {expected_path}")
    snapshots_hash = sha256(expected_path)
    snapshots_tree_hash = digest_tree(source / "snapshots")
    harness_hash = digest_tree(source / "tests" / "visual_baseline")
    run_pair = ["cargo", "nextest", "run", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--run-ignored", "only", "--filter-expr", expression, "--no-fail-fast"]
    phases = []
    selection_name = "all ignored scenario tests" if all_cases else selector or ""
    run_id = f"{time.time_ns()}-{hashlib.sha256(selection_name.encode()).hexdigest()[:12]}"
    run_root = root / "paired" / run_id
    run_root.mkdir(parents=True, exist_ok=False)
    for phase, bins in (("baseline", baseline_bins), ("candidate", candidate_bins)):
        env["VISUAL_BIN_DIR"] = str(bins.resolve())
        clear_runtime_artifacts(source)
        started = time.monotonic()
        failure = None
        try:
            output = run(run_pair, cwd=source, timeout=timeout, env=env, include_stderr=True)
            actual_count = int(re.search(r"\b(\d+) tests? run\b", output).group(1)) if re.search(r"\b(\d+) tests? run\b", output) else 0
            if actual_count != expected_count:
                raise RunnerError(f"{phase} executed {actual_count} tests; expected {expected_count}")
        except RunnerError as error:
            failure = str(error)
        artifacts = archive_artifacts(source, run_root, phase)
        phases.append({"phase": phase, "status": "failed" if failure else "passed", "failure": failure, "bin_dir": str(bins.resolve()), "artifact_dir": str(artifacts), "elapsed_seconds": round(time.monotonic() - started, 2), "selected_tests": expected_count, "complete_artifact_sha256": digest_tree(artifacts), "canonical_frame_sha256": frame_digest(artifacts)})
        if digest_tree(source / "snapshots") != snapshots_tree_hash or sha256(expected_path) != snapshots_hash or digest_tree(source / "tests" / "visual_baseline") != harness_hash:
            phases[-1]["status"] = "failed"
            phases[-1]["failure"] = "selected frozen snapshot or test harness changed during phase"
            failure = "selected frozen snapshot or test harness changed"
        report = paired_report(source_hash, snapshots_hash, snapshots_tree_hash, harness_hash, expected_path, selection_name, expected_count, candidate, phases)
        dest = run_root / "report.json"
        if phase == "candidate" and (run_root / "baseline").is_dir():
            report["direct_frame_differences"] = exact_frame_differences(run_root / "baseline", artifacts)
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(json.dumps(report, indent=2) + "\n")
        if failure:
            raise RunnerError(f"{phase} failed; artifacts and run record saved to {artifacts}: {failure}")
    frame_differences = exact_frame_differences(run_root / "baseline", run_root / "candidate")
    report = paired_report(source_hash, snapshots_hash, snapshots_tree_hash, harness_hash, expected_path, selection_name, expected_count, candidate, phases)
    report["direct_frame_differences"] = frame_differences
    if frame_differences:
        phases[-1]["status"] = "failed"
        phases[-1]["failure"] = "baseline and candidate canonical frame artifacts differ"
        dest = run_root / "report.json"
        dest.write_text(json.dumps(report, indent=2) + "\n")
        raise RunnerError(f"baseline and candidate frames differ; see {dest}: {frame_differences}")
    report["acceptance_complete"] = False
    dest = run_root / "report.json"
    dest.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


def baseline_and_candidate_records(repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, timeout: int, *, baseline_cohort_id: str | None = None, cohort_repo: Path | None = None) -> tuple[dict[str, object], dict[str, object]]:
    source = root / "source"
    source_hash = verify_source(root, source)
    baseline_hashes: dict[str, str] = {}
    baseline_files: dict[str, dict[str, int]] = {}
    if baseline_cohort_id is None:
        build_record = json.loads((root / "build.json").read_text())
        if build_record.get("source_sha256") != source_hash:
            raise RunnerError("frozen build provenance does not match extracted source")
        if baseline_bins.resolve() != (root / "cargo-target" / "debug").resolve():
            raise RunnerError("baseline binaries must be the isolated binaries built by this runner")
        for app in APPS:
            baseline_hashes[app], baseline_files[app] = stable_private_binary(baseline_bins / app, f"frozen baseline binary {app}")
        if build_record.get("binaries") != baseline_hashes:
            raise RunnerError("frozen baseline binaries changed after the recorded build")
    else:
        capture = require_capture_eligible_cohort(cohort_repo or repo, baseline_cohort_id, baseline_bins)
        for app in APPS:
            baseline_hashes[app], baseline_files[app] = stable_private_binary(baseline_bins / app, f"capture-eligible frozen baseline binary {app}")
        actual = baseline_hashes
        expected_source = capture["source"]
        if expected_source != {
            "tag": BASELINE_TAG, "tag_object": BASELINE_TAG_OBJECT, "commit": BASELINE_COMMIT,
            "tree": BASELINE_TREE, "tuiscotti_rev": TUISCOTTI_REV,
            "cargo_lock_sha256": sha256(source / "Cargo.lock"),
            "rust_toolchain_sha256": sha256(source / "rust-toolchain.toml"),
        }:
            raise RunnerError("capture-eligible cohort source identity differs from extracted frozen source")
        expected = {app: item["sha256"] for app, item in capture["binaries"].items()}
        if actual != expected:
            raise RunnerError("capture-eligible frozen binaries changed after cohort verification")
        build_record = {
            "source_sha256": source_hash,
            "binaries": actual,
            "provenance_kind": "separately reviewed executable cohort evidence; not build-provenance.json",
            "executable_cohort": capture,
            "candidate_comparison_eligible": False,
            "checkpoint_reference_approval": "none",
            "acceptance_complete": False,
        }
    candidate = json.loads((root / "candidate-build.json").read_text())
    if Path(candidate["bin_dir"]).resolve() != candidate_bins.resolve():
        raise RunnerError("candidate binary directory does not match candidate-build.json")
    revision, dirty_hash, dirty_files = git_state(repo, timeout)
    if (revision, dirty_hash, dirty_files) != (candidate.get("revision"), candidate.get("dirty_fingerprint"), candidate.get("dirty_files")):
        raise RunnerError("candidate source changed after its recorded build")
    candidate_hashes: dict[str, str] = {}
    candidate_files: dict[str, dict[str, int]] = {}
    for app in APPS:
        candidate_hashes[app], candidate_files[app] = stable_private_binary(candidate_bins / app, f"candidate binary {app}")
    if candidate.get("binary_files") != candidate_files:
        raise RunnerError("candidate binary file identity changed after its recorded build")
    if candidate.get("binaries") != candidate_hashes:
        raise RunnerError("candidate binaries changed after their recorded build")
    for app in APPS:
        if (baseline_files[app]["device"], baseline_files[app]["inode"]) == (candidate_files[app]["device"], candidate_files[app]["inode"]):
            raise RunnerError(f"candidate binary aliases the frozen baseline executable: {app}")
    candidate["source_tree"] = run(["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"], cwd=repo, timeout=timeout).strip()
    if baseline_bins.resolve() == candidate_bins.resolve() or not candidate_bins.is_dir():
        raise RunnerError("baseline and candidate binary directories must be distinct existing directories")
    return build_record, candidate


def checkpoint_identity(case: str, ordinal: int, logical_name: str) -> str:
    identity = hashlib.sha256()
    identity.update(b"termrock-paired-checkpoint-v1\0")
    identity.update(case.encode())
    identity.update(b"\0")
    identity.update(ordinal.to_bytes(8, "big"))
    identity.update(b"\0")
    identity.update(logical_name.encode())
    return identity.hexdigest()


def exact_receipt_differences(left: dict[tuple[str, int, str], tuple[str, str, str | None]], right: dict[tuple[str, int, str], tuple[str, str, str | None]]) -> list[str]:
    differences = []
    for key in sorted(set(left) | set(right)):
        if key not in left:
            differences.append(f"candidate-only:{key}")
        elif key not in right:
            differences.append(f"baseline-only:{key}")
        else:
            if left[key][0] != right[key][0]:
                differences.append(f"ansi-digest-differs:{key}")
            if left[key][1] != right[key][1]:
                differences.append(f"semantic-frame-digest-differs:{key}")
    return differences


def validate_extension_checkpoint_receipts(selected: list[str], receipts: dict[tuple[str, int, str], str], extension_manifest: dict[str, object]) -> list[str]:
    obligations = extension_manifest["checkpoint_obligations"]
    extension_ids = set(extension_manifest["generated_case_ids"])
    for test_name in sorted(set(selected) & extension_ids):
        case = scenario_receipt_id(test_name, extension_manifest)
        actual = [(ordinal, logical) for receipt_case, ordinal, logical in receipts if receipt_case == case]
        actual.sort()
        expected = list(enumerate(f"{test_name}/{stage}" for stage in obligations[test_name]))
        if actual != expected:
            raise RunnerError(f"extension checkpoint obligations differ for {test_name}: expected={expected}, got={actual}")
    return []


def cli_coverage_report_fields(registered_cli_complete: bool) -> dict[str, object]:
    return {
        "scope": "registered_cli_scenarios_and_ordinary_checks",
        "registered_cli_complete": registered_cli_complete,
        "overall_acceptance_complete": False,
        "acceptance_complete": False,
    }


def scenario_chunks(names: list[str], size: int, *, check_work: Callable[[], None] | None = None) -> list[list[str]]:
    if not 1 <= size <= 512:
        raise RunnerError("chunk size must be 1..512 cases")
    reference_names = []
    ordinary = []
    for name in names:
        if check_work is not None:
            check_work()
        (reference_names if name == REFERENCE_CASE else ordinary).append(name)
    if len(reference_names) > 1:
        raise RunnerError("reference checkpoint scenario is duplicated in selection")
    chunks = [ordinary[start : start + size] for start in range(0, len(ordinary), size)]
    if reference_names:
        chunks.append(reference_names)
    return chunks


def _visual_case_conditions(names: list[str], *, check_work: Callable[[], None] | None = None) -> dict[str, dict[str, object]]:
    conditions: dict[str, dict[str, object]] = {}
    for name in names:
        if check_work is not None:
            check_work()
        parts = name.rsplit("::", 2)
        if len(parts) != 3 or not re.fullmatch(r"[0-9]+x[0-9]+", parts[1]) or parts[2] not in {"truecolor", "256", "16", "none", "nocolor"}:
            raise RunnerError(f"visual scenario does not encode a supported terminal size and color mode: {name}")
        cols, rows = (int(value) for value in parts[1].split("x", 1))
        if cols < 1 or rows < 1:
            raise RunnerError(f"visual scenario encodes a zero-sized terminal: {name}")
        conditions[name] = {"cols": cols, "rows": rows, "color_mode": parts[2]}
    return conditions


def _chunk_records(names: list[str], size: int, *, check_work: Callable[[], None] | None = None) -> list[dict[str, object]]:
    return [
        {"index": index, "selected_cases": chunk}
        for index, chunk in enumerate(scenario_chunks(names, size, check_work=check_work))
    ]


def build_fresh_plan_payload(
    *,
    repo: Path,
    root: Path,
    cohort: dict[str, object],
    live_record: dict[str, object],
    inventory: dict[str, bool],
    extension_manifest: dict[str, object],
    chunk_size: int,
    term_profile: str | None,
    plan_id: str,
    created_at: str,
    advance_budget_seconds: int = FRESH_PLAN_MAX_TIMEOUT,
    execution: PrepareExecution | None = None,
) -> dict[str, object]:
    check_work = execution.check_work if execution is not None else None
    if check_work is not None:
        check_work()
    if type(advance_budget_seconds) is not int or not 1 <= advance_budget_seconds <= FRESH_PLAN_MAX_TIMEOUT:
        raise RunnerError(f"fresh plan advance budget must be 1..{FRESH_PLAN_MAX_TIMEOUT} seconds")
    if not 1 <= chunk_size <= DEFAULT_CHUNK_SIZE:
        raise RunnerError(f"fresh plan chunk size must be 1..{DEFAULT_CHUNK_SIZE}")
    original_inventory, extension_ids = validate_extension_inventory(inventory, extension_manifest, check_work=check_work) if check_work is not None else validate_extension_inventory(inventory, extension_manifest)
    require_inventory_matches(inventory, str(live_record.get("test_inventory_sha256")), check_work=check_work)
    visual = []
    for name, ignored in inventory.items():
        if check_work is not None:
            check_work()
        if ignored:
            visual.append(name)
    visual.sort()
    source_only, baseline_cli = ordinary_test_groups(inventory, check_work=check_work) if check_work is not None else ordinary_test_groups(inventory)
    if len(visual) != EXPECTED_SCENARIOS + len(extension_ids) or len(source_only) + len(baseline_cli) != EXPECTED_ORDINARY_TESTS:
        raise RunnerError("fresh plan selection is incomplete or includes a zero-case group")
    conditions = _visual_case_conditions(visual, check_work=check_work) if check_work is not None else _visual_case_conditions(visual)
    extension_obligations = extension_manifest["checkpoint_obligations"]
    if set(extension_obligations) != set(extension_ids):
        raise RunnerError("fresh plan extension checkpoint obligations are incomplete")
    obligations: dict[str, dict[str, object]] = {}
    for name in visual:
        if check_work is not None:
            check_work()
        if name in extension_obligations:
            obligations[name] = {"kind": "exact", "checkpoints": extension_obligations[name]}
        else:
            obligations[name] = {
                "kind": "pinned-harness-nonempty-ordered-receipts",
                "harness_sha256": live_record["module_sha256"],
                "scenario_test_id": name,
            }
    visual_chunks = _chunk_records(visual, chunk_size, check_work=check_work) if check_work is not None else _chunk_records(visual, chunk_size)
    phases = [
        {"name": "baseline-capture", "chunks": visual_chunks},
        {"name": "baseline-replay", "chunks": visual_chunks},
        {
            "name": "baseline-ordinary-checks",
            "groups": [
                {"name": "source-only", "chunks": _chunk_records(source_only, chunk_size, check_work=check_work) if check_work is not None else _chunk_records(source_only, chunk_size)},
                {"name": "baseline-cli", "chunks": _chunk_records(baseline_cli, chunk_size, check_work=check_work) if check_work is not None else _chunk_records(baseline_cli, chunk_size)},
            ],
        },
    ]
    live = root / "live-source"
    env = fresh_clean_env()
    env["CARGO_TARGET_DIR"] = str(root / "live-target")
    env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"] = "1"
    if check_work is not None:
        check_work()
    runtime_env = {
        "policy": "fresh-clean-env-with-explicit-runtime-allowlist",
        "effective_environment": fresh_environment_fingerprint(env),
        "selected_tools": fresh_tool_observation(live, env, execution=execution) if execution is not None else fresh_tool_observation(live, env),
        "overrides": {
            "VISUAL_FULL": "1",
            "VISUAL_REVIEW": "0",
            "VISUAL_BIN_DIR": cohort["binaries"][APPS[0]]["path"].rsplit(os.sep, 1)[0],
        },
        "child_time_policy": "fixed by pinned frozen scenario harness; no clock override bytes are stored in plan",
        "controlled_clock": {
            "extension_cases": {"TUISCOTTI_FIXED_TIME": "2026-01-01T00:00:00Z"},
            "original_cases": "use only the clock policy declared by the pinned frozen scenario harness",
        },
        "terminal_profile": apply_term_profile(clean_env(), term_profile),
        "font_profile": {"name": "pinned-tuiscotti-VENDORED_FACES", "tuiscotti_rev": TUISCOTTI_REV},
        "unknown_parent_environment_values_recorded": False,
    }
    source = root / "source"
    live = root / "live-source"
    extension_path = repo / EXTENSION_MANIFEST
    patch_path = repo / "tools" / "frozen_cli_pair.patch"
    helper_path = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    runner_path = Path(__file__).resolve()
    full_sorted = []
    visual_original = []
    ordinary_selected = []
    for name, ignored in sorted(inventory.items()):
        if check_work is not None:
            check_work()
        full_sorted.append({"name": name, "ignored": ignored})
    for name, ignored in original_inventory.items():
        if check_work is not None:
            check_work()
        if ignored:
            visual_original.append(name)
    visual_original.sort()
    for name, ignored in inventory.items():
        if check_work is not None:
            check_work()
        if not ignored:
            ordinary_selected.append(name)
    ordinary_selected.sort()
    payload: dict[str, object] = {
        "schema": FRESH_PLAN_SCHEMA,
        "schema_version": FRESH_PLAN_VERSION,
        "status": "prepared-not-executed",
        "plan_id": plan_id,
        "created_at_utc": created_at,
        "cohort": cohort,
        "source": {
            "tag": BASELINE_TAG,
            "tag_object": BASELINE_TAG_OBJECT,
            "commit": BASELINE_COMMIT,
            "tree": BASELINE_TREE,
            "tuiscotti_rev": TUISCOTTI_REV,
            "source_path": str(source.resolve()),
            "source_sha256": digest_source(source, check_work=check_work) if check_work is not None else digest_source(source),
            "source_tree_sha256": digest_tree(source, check_work=check_work) if check_work is not None else digest_tree(source),
            "cargo_lock_sha256": sha256(source / "Cargo.lock", check_work=check_work) if check_work is not None else sha256(source / "Cargo.lock"),
            "rust_toolchain_sha256": sha256(source / "rust-toolchain.toml", check_work=check_work) if check_work is not None else sha256(source / "rust-toolchain.toml"),
            "visual_test_tree_sha256": digest_tree(source / "tests" / "visual_baseline", check_work=check_work) if check_work is not None else digest_tree(source / "tests" / "visual_baseline"),
            "snapshots_sha256": digest_tree(source / "snapshots", check_work=check_work) if check_work is not None else digest_tree(source / "snapshots"),
            "live_source_path": str(live.resolve()),
            "live_source_sha256": digest_source(live, check_work=check_work) if check_work is not None else digest_source(live),
            "live_test_tree_sha256": digest_tree(live / "tests" / "visual_baseline", check_work=check_work) if check_work is not None else digest_tree(live / "tests" / "visual_baseline"),
        },
        "runner_inputs": {
            "runner_sha256": sha256(runner_path, check_work=check_work) if check_work is not None else sha256(runner_path),
            "harness_patch_sha256": sha256(patch_path, check_work=check_work) if check_work is not None else sha256(patch_path),
            "frame_helper_sha256": sha256(helper_path, check_work=check_work) if check_work is not None else sha256(helper_path),
            "extension_manifest_sha256": sha256(extension_path, check_work=check_work) if check_work is not None else sha256(extension_path),
            "extension_source_sha256": sha256(repo / EXTENSION_SOURCE, check_work=check_work) if check_work is not None else sha256(repo / EXTENSION_SOURCE),
            "frozen_test_harness_sha256": digest_tree(live / "tests" / "visual_baseline", check_work=check_work) if check_work is not None else digest_tree(live / "tests" / "visual_baseline"),
            "frozen_source_manifest_sha256": sha256(root / "provenance.json", check_work=check_work) if check_work is not None else sha256(root / "provenance.json"),
            "live_source_manifest_sha256": sha256(root / "live-source.json", check_work=check_work) if check_work is not None else sha256(root / "live-source.json"),
            "capture_decision_sha256": cohort["decision_sha256"],
            "capture_evidence_index_sha256": cohort["evidence_index_sha256"],
            "cohort_registry_sha256": cohort["cohort_registry_sha256"],
            "cohort_review_registry_sha256": cohort["review_registry_sha256"],
            "resumable_contract_sha256": CAPTURE_REVIEW_CONTRACT_SHA256,
        },
        "runtime": runtime_env,
        "inventory": {
            "sha256": inventory_digest(inventory, check_work=check_work),
            "full_sorted": full_sorted,
            "visual_original": visual_original,
            "visual_extensions": extension_ids,
            "visual_selected": visual,
            "ordinary_selected": ordinary_selected,
            "ordinary_source_only": source_only,
            "ordinary_baseline_cli": baseline_cli,
            "visual_count": len(visual),
            "original_visual_count": EXPECTED_SCENARIOS,
            "extension_visual_count": len(extension_ids),
            "ordinary_count": len(source_only) + len(baseline_cli),
            "case_terminal_inputs": conditions,
            "checkpoint_obligations": obligations,
        },
        "execution": {
            "timeout_seconds_per_advance": advance_budget_seconds,
            "chunk_size": chunk_size,
            "phases": phases,
            "candidate_phase": None,
            "candidate_comparison_eligible": False,
            "checkpoint_reference_approval": "none",
            "product_acceptance": False,
            "artifact_roots": {
                "run_root": str((root / "fresh-capture" / plan_id).resolve()),
                "baseline_capture": str((root / "fresh-capture" / plan_id / "baseline-capture").resolve()),
                "baseline_replay": str((root / "fresh-capture" / plan_id / "baseline-replay").resolve()),
                "ordinary_checks": str((root / "fresh-capture" / plan_id / "baseline-ordinary-checks").resolve()),
            },
            "receipt_secret": "not stored in plan; generated only by a future per-step process and never passed to application env",
        },
    }
    if check_work is not None:
        check_work()
    plan = {**payload, "plan_sha256": canonical_json_digest(payload)}
    validate_fresh_plan_document(plan, check_work=check_work)
    if check_work is not None:
        check_work()
    return plan


def validate_fresh_plan_document(document: object, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
    fields = {"schema", "schema_version", "status", "plan_id", "created_at_utc", "cohort", "source", "runner_inputs", "runtime", "inventory", "execution", "plan_sha256"}
    if not isinstance(document, dict) or set(document) != fields:
        raise RunnerError("fresh plan fields differ from schema version 1")
    if document["schema"] != FRESH_PLAN_SCHEMA or type(document["schema_version"]) is not int or document["schema_version"] != FRESH_PLAN_VERSION or document["status"] != "prepared-not-executed":
        raise RunnerError("fresh plan schema or execution status is invalid")
    if not isinstance(document["plan_id"], str) or not re.fullmatch(r"[0-9a-f]{32}", document["plan_id"]):
        raise RunnerError("fresh plan id is invalid")
    cohort = document["cohort"]
    if not isinstance(cohort, dict) or set(cohort) != {"cohort_id", "cohort_registry_sha256", "review_registry_sha256", "decision_sha256", "evidence_index_sha256", "status", "source", "tools", "binaries", "fresh_baseline_capture_eligible", "baseline_replay_eligible", "candidate_comparison_eligible", "checkpoint_reference_approval", "product_acceptance_eligible", "acceptance_complete"}:
        raise RunnerError("fresh plan cohort identity fields are invalid")
    if cohort.get("fresh_baseline_capture_eligible") is not True or cohort.get("baseline_replay_eligible") is not True:
        raise RunnerError("fresh plan cohort is not eligible for capture and replay")
    if cohort.get("candidate_comparison_eligible") is not False or cohort.get("checkpoint_reference_approval") != "none" or cohort.get("product_acceptance_eligible") is not False or cohort.get("acceptance_complete") is not False:
        raise RunnerError("fresh plan cohort grants forbidden candidate or reference privileges")
    for field in ("cohort_registry_sha256", "review_registry_sha256", "decision_sha256", "evidence_index_sha256"):
        if not isinstance(cohort.get(field), str) or not re.fullmatch(r"[0-9a-f]{64}", cohort[field]):
            raise RunnerError(f"fresh plan cohort digest is invalid: {field}")
    if not isinstance(document["source"], dict) or set(document["source"]) != {"tag", "tag_object", "commit", "tree", "tuiscotti_rev", "source_path", "source_sha256", "source_tree_sha256", "cargo_lock_sha256", "rust_toolchain_sha256", "visual_test_tree_sha256", "snapshots_sha256", "live_source_path", "live_source_sha256", "live_test_tree_sha256"}:
        raise RunnerError("fresh plan source identity fields are invalid")
    source = document["source"]
    if (source.get("tag"), source.get("tag_object"), source.get("commit"), source.get("tree"), source.get("tuiscotti_rev")) != (BASELINE_TAG, BASELINE_TAG_OBJECT, BASELINE_COMMIT, BASELINE_TREE, TUISCOTTI_REV):
        raise RunnerError("fresh plan source pins differ from the fixed reference")
    if not isinstance(document["runner_inputs"], dict) or set(document["runner_inputs"]) != {"runner_sha256", "harness_patch_sha256", "frame_helper_sha256", "extension_manifest_sha256", "extension_source_sha256", "frozen_test_harness_sha256", "frozen_source_manifest_sha256", "live_source_manifest_sha256", "capture_decision_sha256", "capture_evidence_index_sha256", "cohort_registry_sha256", "cohort_review_registry_sha256", "resumable_contract_sha256"}:
        raise RunnerError("fresh plan runner input fields are invalid")
    for field, digest in document["runner_inputs"].items():
        if check_work is not None:
            check_work()
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise RunnerError(f"fresh plan runner input digest is invalid: {field}")
    if not isinstance(document["runtime"], dict) or set(document["runtime"]) != {"policy", "effective_environment", "selected_tools", "overrides", "child_time_policy", "controlled_clock", "terminal_profile", "font_profile", "unknown_parent_environment_values_recorded"}:
        raise RunnerError("fresh plan runtime fields are invalid")
    inventory = document["inventory"]
    if not isinstance(inventory, dict) or set(inventory) != {"sha256", "full_sorted", "visual_original", "visual_extensions", "visual_selected", "ordinary_selected", "ordinary_source_only", "ordinary_baseline_cli", "visual_count", "original_visual_count", "extension_visual_count", "ordinary_count", "case_terminal_inputs", "checkpoint_obligations"}:
        raise RunnerError("fresh plan inventory fields are invalid")
    rows = inventory["full_sorted"]
    if not isinstance(rows, list) or not rows:
        raise RunnerError("fresh plan inventory rows are invalid")
    names = []
    row_inventory: dict[str, bool] = {}
    for row in rows:
        if check_work is not None:
            check_work()
        if not isinstance(row, dict) or set(row) != {"name", "ignored"} or not isinstance(row["name"], str) or type(row["ignored"]) is not bool:
            raise RunnerError("fresh plan inventory rows are invalid")
        names.append(row["name"])
        row_inventory[row["name"]] = row["ignored"]
    if names != sorted(names) or len(names) != len(set(names)) or inventory["sha256"] != inventory_digest(row_inventory, check_work=check_work):
        raise RunnerError("fresh plan inventory is unsorted, duplicated, or has a false digest")
    visual = []
    ordinary = []
    for row in rows:
        if check_work is not None:
            check_work()
        (visual if row["ignored"] else ordinary).append(row["name"])
    if visual != inventory["visual_selected"] or ordinary != inventory["ordinary_selected"]:
        raise RunnerError("fresh plan inventory selection differs from complete registered inventory")
    extensions = inventory["visual_extensions"]
    if not isinstance(extensions, list) or extensions != sorted(set(extensions)) or any(name not in visual for name in extensions):
        raise RunnerError("fresh plan extension case inventory is invalid")
    if len(visual) != EXPECTED_SCENARIOS + len(extensions) or len(ordinary) != EXPECTED_ORDINARY_TESTS:
        raise RunnerError("fresh plan visual or ordinary inventory count is incomplete")
    if inventory["visual_original"] != sorted(set(visual) - set(extensions)):
        raise RunnerError("fresh plan original visual selection differs from extension inventory")
    if (inventory["visual_count"], inventory["original_visual_count"], inventory["extension_visual_count"], inventory["ordinary_count"]) != (
        len(visual), EXPECTED_SCENARIOS, len(extensions), len(ordinary)
    ):
        raise RunnerError("fresh plan declared counts differ from the complete inventory")
    source_only = inventory["ordinary_source_only"]
    baseline_cli = inventory["ordinary_baseline_cli"]
    if not isinstance(source_only, list) or not isinstance(baseline_cli, list) or source_only != sorted(set(source_only)) or baseline_cli != sorted(set(baseline_cli)):
        raise RunnerError("fresh plan ordinary groups are invalid")
    if sorted(source_only + baseline_cli) != ordinary or set(source_only) & set(baseline_cli):
        raise RunnerError("fresh plan ordinary groups omit or duplicate registered checks")
    if set(inventory["case_terminal_inputs"]) != set(visual) or set(inventory["checkpoint_obligations"]) != set(visual):
        raise RunnerError("fresh plan case inputs or checkpoint obligations are incomplete")
    for name, condition in inventory["case_terminal_inputs"].items():
        if check_work is not None:
            check_work()
        if not isinstance(condition, dict) or set(condition) != {"cols", "rows", "color_mode"} or type(condition["cols"]) is not int or type(condition["rows"]) is not int or min(condition["cols"], condition["rows"]) < 1 or condition["color_mode"] not in {"truecolor", "256", "16", "none", "nocolor"}:
            raise RunnerError(f"fresh plan has invalid terminal inputs for {name}")
    execution = document["execution"]
    if not isinstance(execution, dict) or set(execution) != {"timeout_seconds_per_advance", "chunk_size", "phases", "candidate_phase", "candidate_comparison_eligible", "checkpoint_reference_approval", "product_acceptance", "artifact_roots", "receipt_secret"} or execution.get("candidate_phase") is not None or execution.get("candidate_comparison_eligible") is not False or execution.get("checkpoint_reference_approval") != "none" or execution.get("product_acceptance") is not False:
        raise RunnerError("fresh plan execution scope enables a forbidden phase")
    if type(execution.get("chunk_size")) is not int or not 1 <= execution["chunk_size"] <= DEFAULT_CHUNK_SIZE or type(execution.get("timeout_seconds_per_advance")) is not int or not 1 <= execution["timeout_seconds_per_advance"] <= FRESH_PLAN_MAX_TIMEOUT:
        raise RunnerError("fresh plan chunk or timeout bounds are invalid")
    if not isinstance(execution.get("phases"), list) or len(execution["phases"]) != 3 or any(not isinstance(phase, dict) for phase in execution["phases"]):
        raise RunnerError("fresh plan phase records are malformed")
    phase_names = [phase.get("name") for phase in execution["phases"]]
    if phase_names != ["baseline-capture", "baseline-replay", "baseline-ordinary-checks"]:
        raise RunnerError("fresh plan phase inventory is incomplete or out of order")
    visual_chunks = _chunk_records(visual, execution.get("chunk_size"), check_work=check_work)
    for phase in execution["phases"][:2]:
        if phase != {"name": phase["name"], "chunks": visual_chunks}:
            raise RunnerError("fresh plan visual phase chunk membership differs from inventory")
    ordinary_groups = execution["phases"][2].get("groups") if isinstance(execution["phases"][2], dict) else None
    if not isinstance(ordinary_groups, list) or [group.get("name") for group in ordinary_groups if isinstance(group, dict)] != ["source-only", "baseline-cli"]:
        raise RunnerError("fresh plan ordinary group order is invalid")
    if ordinary_groups != [
        {"name": "source-only", "chunks": _chunk_records(source_only, execution["chunk_size"], check_work=check_work)},
        {"name": "baseline-cli", "chunks": _chunk_records(baseline_cli, execution["chunk_size"], check_work=check_work)},
    ]:
        raise RunnerError("fresh plan ordinary chunk membership differs from inventory")
    runtime = document["runtime"]
    if not isinstance(runtime, dict) or runtime.get("font_profile") != {"name": "pinned-tuiscotti-VENDORED_FACES", "tuiscotti_rev": TUISCOTTI_REV}:
        raise RunnerError("fresh plan font or Tuiscotti profile is invalid")
    if runtime.get("policy") != "fresh-clean-env-with-explicit-runtime-allowlist" or runtime.get("unknown_parent_environment_values_recorded") is not False or not isinstance(runtime.get("effective_environment"), dict) or not isinstance(runtime.get("selected_tools"), dict):
        raise RunnerError("fresh plan environment disclosure policy is invalid")
    if runtime.get("child_time_policy") != "fixed by pinned frozen scenario harness; no clock override bytes are stored in plan":
        raise RunnerError("fresh plan time control policy is invalid")
    if runtime.get("controlled_clock") != {
        "extension_cases": {"TUISCOTTI_FIXED_TIME": "2026-01-01T00:00:00Z"},
        "original_cases": "use only the clock policy declared by the pinned frozen scenario harness",
    }:
        raise RunnerError("fresh plan controlled clock inputs are invalid")
    payload = {key: value for key, value in document.items() if key != "plan_sha256"}
    if not isinstance(document["plan_sha256"], str) or document["plan_sha256"] != canonical_json_digest(payload):
        raise RunnerError("fresh plan content digest mismatch")
    return document


def _read_self_sealed_plan(path: Path, expected_sha256: str | None = None, *, check_work: Callable[[], None] | None = None) -> dict[str, object]:
    if check_work is not None:
        check_work()
    try:
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
            raise RunnerError("fresh plan is symlinked or non-regular")
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            opened = os.fstat(fd)
            chunks: list[bytes] = []
            while chunk := os.read(fd, 1024 * 1024):
                if check_work is not None:
                    check_work()
                chunks.append(chunk)
            after_fd = os.fstat(fd)
        finally:
            os.close(fd)
        after = path.lstat()
    except OSError as error:
        raise RunnerError(f"fresh plan cannot be safely read: {error}") from error
    identity = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)
    if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after):
        raise RunnerError("fresh plan changed during read")
    def strict_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise RunnerError(f"duplicate fresh plan JSON field: {key}")
            result[key] = value
        return result
    raw = b"".join(chunks)
    if check_work is not None:
        check_work()
    if expected_sha256 is not None:
        if not re.fullmatch(r"[0-9a-f]{64}", expected_sha256) or hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise RunnerError("fresh plan does not match the externally retained creation digest")
    try:
        parsed = json.loads(raw, object_pairs_hook=strict_object)
    except json.JSONDecodeError as error:
        raise RunnerError("fresh plan is not valid JSON") from error
    result = validate_fresh_plan_document(parsed, check_work=check_work)
    if check_work is not None:
        check_work()
    return result


def _atomic_create_plan(path: Path, document: dict[str, object], *, check_work: Callable[[], None] | None = None) -> str:
    if check_work is not None:
        check_work()
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = (json.dumps(document, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        if check_work is not None:
            check_work()
        os.fchmod(fd, 0o400)
        with os.fdopen(fd, "wb") as stream:
            stream.write(payload)
            stream.flush()
            if check_work is not None:
                check_work()
            os.fsync(stream.fileno())
        if check_work is not None:
            check_work()
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError as error:
            raise RunnerError(f"refusing to replace immutable fresh plan: {path}") from error
        directory_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
        if check_work is not None:
            check_work()
    finally:
        temporary.unlink(missing_ok=True)
    return hashlib.sha256(payload).hexdigest()


def create_fresh_plan(
    repo: Path,
    root: Path,
    cohort_id: str,
    baseline_bin_dir: Path,
    timeout: int,
    chunk_size: int,
    term_profile: str | None,
    *,
    execution: PrepareExecution | None = None,
) -> dict[str, object]:
    check_work = execution.check_work if execution is not None else None
    if check_work is not None:
        check_work()
    if timeout > FRESH_PLAN_MAX_TIMEOUT:
        raise RunnerError(f"fresh plan command timeout must be <= {FRESH_PLAN_MAX_TIMEOUT} seconds")
    repo, root, source = paths(repo, root)
    if check_work is not None:
        check_work()
        verify_source(root, source, check_work=check_work)
        capture = require_capture_eligible_cohort(repo, cohort_id, baseline_bin_dir.resolve(), check_work=check_work)
    else:
        verify_source(root, source)
        capture = require_capture_eligible_cohort(repo, cohort_id, baseline_bin_dir.resolve())
    if execution is not None:
        live, live_record = verify_live_source(repo, root, source, timeout, fresh_cohort_id=cohort_id, baseline_bin_dir=baseline_bin_dir, execution=execution)
    else:
        live, live_record = verify_live_source(repo, root, source, timeout, fresh_cohort_id=cohort_id, baseline_bin_dir=baseline_bin_dir)
    extension_manifest = load_extension_manifest(repo / EXTENSION_MANIFEST, check_work=check_work) if check_work is not None else load_extension_manifest(repo / EXTENSION_MANIFEST)
    env = fresh_clean_env()
    env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
    inventory = nextest_inventory(live, timeout, env, extension_manifest, execution=execution) if execution is not None else nextest_inventory(live, timeout, env, extension_manifest)
    require_inventory_matches(inventory, str(live_record.get("test_inventory_sha256")), check_work=check_work)
    run_id = secrets.token_hex(16)
    created_at = datetime.now(timezone.utc).isoformat()
    plan = build_fresh_plan_payload(
        repo=repo, root=root, cohort=capture, live_record=live_record, inventory=inventory,
        extension_manifest=extension_manifest, chunk_size=chunk_size, term_profile=term_profile,
        plan_id=run_id, created_at=created_at,
        advance_budget_seconds=execution.work_budget if execution is not None else FRESH_PLAN_MAX_TIMEOUT,
        execution=execution,
    )
    plan_path = root / FRESH_PLAN_FILE
    if check_work is not None:
        check_work()
    creation_file_sha256 = _atomic_create_plan(plan_path, plan, check_work=check_work) if check_work is not None else _atomic_create_plan(plan_path, plan)
    _read_self_sealed_plan(plan_path, creation_file_sha256, check_work=check_work) if check_work is not None else _read_self_sealed_plan(plan_path, creation_file_sha256)
    success = json.dumps({"plan_path": str(plan_path), "plan_sha256": plan["plan_sha256"], "creation_file_sha256": creation_file_sha256, "status": plan["status"], "visual_cases": plan["inventory"]["visual_count"], "ordinary_checks": plan["inventory"]["ordinary_count"]}, indent=2)
    if check_work is not None:
        check_work()
        print(success, flush=True)
        check_work()
    else:
        print(success)
    return plan


def verify_fresh_plan(repo: Path, root: Path, cohort_id: str, baseline_bin_dir: Path, timeout: int, expected_plan_sha256: str, *, execution: PrepareExecution | None = None) -> dict[str, object]:
    check_work = execution.check_work if execution is not None else None
    if check_work is not None:
        check_work()
    if timeout > FRESH_PLAN_MAX_TIMEOUT:
        raise RunnerError(f"fresh plan verification timeout must be <= {FRESH_PLAN_MAX_TIMEOUT} seconds")
    repo, root, source = paths(repo, root)
    plan_path = root / FRESH_PLAN_FILE
    plan = _read_self_sealed_plan(plan_path, expected_plan_sha256, check_work=check_work) if check_work is not None else _read_self_sealed_plan(plan_path, expected_plan_sha256)
    if plan["cohort"].get("cohort_id") != cohort_id:
        raise RunnerError("fresh plan belongs to another reviewed executable cohort")
    if execution is not None and plan["execution"]["timeout_seconds_per_advance"] != execution.work_budget:
        raise RunnerError("fresh plan advance budget differs from --owned-budget")
    if check_work is not None:
        verify_source(root, source, check_work=check_work)
        capture = require_capture_eligible_cohort(repo, cohort_id, baseline_bin_dir.resolve(), check_work=check_work)
    else:
        verify_source(root, source)
        capture = require_capture_eligible_cohort(repo, cohort_id, baseline_bin_dir.resolve())
    if execution is not None:
        live, live_record = verify_live_source(repo, root, source, timeout, fresh_cohort_id=cohort_id, baseline_bin_dir=baseline_bin_dir, execution=execution)
    else:
        live, live_record = verify_live_source(repo, root, source, timeout, fresh_cohort_id=cohort_id, baseline_bin_dir=baseline_bin_dir)
    extension_manifest = load_extension_manifest(repo / EXTENSION_MANIFEST, check_work=check_work) if check_work is not None else load_extension_manifest(repo / EXTENSION_MANIFEST)
    env = fresh_clean_env()
    env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
    inventory = nextest_inventory(live, timeout, env, extension_manifest, execution=execution) if execution is not None else nextest_inventory(live, timeout, env, extension_manifest)
    regenerated = build_fresh_plan_payload(
        repo=repo, root=root, cohort=capture, live_record=live_record, inventory=inventory,
        extension_manifest=extension_manifest, chunk_size=plan["execution"]["chunk_size"],
        term_profile=None if plan["runtime"]["terminal_profile"]["profile"] == "harness-default" else plan["runtime"]["terminal_profile"]["profile"],
        plan_id=plan["plan_id"], created_at=plan["created_at_utc"],
        advance_budget_seconds=plan["execution"]["timeout_seconds_per_advance"], execution=execution,
    )
    if check_work is not None:
        check_work()
    if check_work is not None:
        check_work()
    if regenerated != plan:
        raise RunnerError("fresh plan no longer matches source, tool, cohort, environment, or complete test inventory pins")
    success = json.dumps({"plan_sha256": plan["plan_sha256"], "creation_file_sha256": expected_plan_sha256, "status": plan["status"], "verified": True}, indent=2)
    if check_work is not None:
        check_work()
        print(success, flush=True)
        check_work()
    else:
        print(success)
    return plan


def _open_directory_chain(path: Path) -> tuple[int, os.stat_result]:
    """Open a canonical directory path component-by-component without following links."""
    canonical = path.resolve(strict=True)
    fd = os.open(os.sep, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        for component in canonical.parts[1:]:
            child = os.open(component, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0), dir_fd=fd)
            os.close(fd)
            fd = child
        return fd, os.fstat(fd)
    except OSError:
        os.close(fd)
        raise


def _private_custody_directory(path: Path, *, create: bool) -> tuple[int, os.stat_result, Path]:
    """Open an owned 0700 custody directory; existing unsafe directories are never chmodded."""
    path = path.absolute()
    parent = path.parent
    try:
        canonical_parent = parent.resolve(strict=True)
        if canonical_parent != parent:
            raise RunnerError("fresh run custody path contains a symlinked ancestor")
        parent_fd, _parent_info = _open_directory_chain(canonical_parent)
    except OSError as error:
        raise RunnerError(f"fresh run custody parent is unsafe or missing: {error}") from error
    try:
        if create:
            created = False
            try:
                os.mkdir(path.name, mode=0o700, dir_fd=parent_fd)
                created = True
            except FileExistsError:
                pass
            except OSError as error:
                raise RunnerError(f"fresh run custody directory cannot be created: {error}") from error
            if created:
                created_fd = os.open(path.name, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0), dir_fd=parent_fd)
                try:
                    os.fchmod(created_fd, 0o700)
                finally:
                    os.close(created_fd)
    finally:
        os.close(parent_fd)
    try:
        canonical = path.resolve(strict=True)
        if canonical != path:
            raise RunnerError("fresh run custody path contains a symlinked ancestor")
        fd, info = _open_directory_chain(canonical)
        if create and created:
            os.fchmod(fd, 0o700)
            info = os.fstat(fd)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) != 0o700:
            os.close(fd)
            raise RunnerError("fresh run custody directory must be owned by this user and mode 0700")
        return fd, info, canonical
    except OSError as error:
        raise RunnerError(f"fresh run custody directory is unsafe: {error}") from error


def _stable_secret_key(path: Path, *, create: bool = False) -> tuple[bytes, tuple[int, int, int, int], Path]:
    """Read or exclusively create a private per-run HMAC key relative to a stable 0700 dirfd."""
    dir_fd, dir_info, custody_path = _private_custody_directory(path.parent, create=create)
    name = path.name
    try:
        if create:
            flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
            try:
                fd = os.open(name, flags, 0o600, dir_fd=dir_fd)
            except FileExistsError as error:
                raise RunnerError("fresh run key already exists") from error
            key = secrets.token_bytes(32)
            try:
                view = memoryview(key)
                while view:
                    view = view[os.write(fd, view):]
                os.fsync(fd)
            finally:
                os.close(fd)
        try:
            before = os.stat(name, dir_fd=dir_fd, follow_symlinks=False)
            if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or stat.S_IMODE(before.st_mode) != 0o600 or before.st_uid != os.geteuid():
                raise RunnerError("fresh run key must be an owned private 0600 regular file")
            fd = os.open(name, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0), dir_fd=dir_fd)
            try:
                opened = os.fstat(fd)
                chunks: list[bytes] = []
                while chunk := os.read(fd, 4096):
                    chunks.append(chunk)
                after_fd = os.fstat(fd)
            finally:
                os.close(fd)
            after = os.stat(name, dir_fd=dir_fd, follow_symlinks=False)
            path_fd, path_info = _open_directory_chain(custody_path)
            os.close(path_fd)
        except OSError as error:
            raise RunnerError(f"fresh run key cannot be safely read: {error}") from error
        identity = lambda item: (item.st_dev, item.st_ino, item.st_mode, item.st_size, item.st_mtime_ns, item.st_ctime_ns, item.st_nlink, item.st_uid)
        dir_identity = lambda item: (item.st_dev, item.st_ino, item.st_mode, item.st_uid)
        if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after) or dir_identity(dir_info) != dir_identity(path_info):
            raise RunnerError("fresh run key or custody directory changed during read")
        key = b"".join(chunks)
        if len(key) != 32:
            raise RunnerError("fresh run key must contain exactly 32 bytes")
        return key, (after.st_dev, after.st_ino, after.st_size, stat.S_IMODE(after.st_mode)), custody_path / name
    finally:
        os.close(dir_fd)


def _git_metadata_paths(repo: Path) -> set[Path]:
    """Return resolved worktree and common Git metadata roots, including linked worktrees."""
    paths: set[Path] = set()
    for option in ("--absolute-git-dir", "--git-common-dir"):
        try:
            result = subprocess.run(["git", "-C", str(repo), "rev-parse", "--path-format=absolute", option], check=True, capture_output=True, text=True, timeout=10)
        except (OSError, subprocess.SubprocessError) as error:
            raise RunnerError(f"cannot identify Git metadata path for key-custody check: {error}") from error
        value = result.stdout.strip()
        if not value:
            raise RunnerError("Git returned an empty metadata path")
        paths.add(Path(value).resolve())
    return paths


def _atomic_create_bytes(path: Path, payload: bytes, mode: int = 0o400) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        os.fchmod(fd, mode)
        with os.fdopen(fd, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError as error:
            raise RunnerError(f"refusing to replace immutable fresh run metadata: {path}") from error
        dir_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
    finally:
        temporary.unlink(missing_ok=True)


def _read_stable_json(path: Path, label: str) -> tuple[dict[str, object], bytes]:
    try:
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
            raise RunnerError(f"{label} must be an unaliased regular file")
        fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            opened = os.fstat(fd)
            chunks: list[bytes] = []
            while chunk := os.read(fd, 1024 * 1024):
                chunks.append(chunk)
            after_fd = os.fstat(fd)
        finally:
            os.close(fd)
        after = path.lstat()
    except OSError as error:
        raise RunnerError(f"{label} cannot be safely read: {error}") from error
    identity = lambda item: (item.st_dev, item.st_ino, item.st_mode, item.st_size, item.st_mtime_ns, item.st_ctime_ns, item.st_nlink)
    if identity(before) != identity(opened) or identity(opened) != identity(after_fd) or identity(after_fd) != identity(after):
        raise RunnerError(f"{label} changed during read")
    raw = b"".join(chunks)
    def strict_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise RunnerError(f"duplicate {label} JSON field: {key}")
            result[key] = value
        return result
    try:
        value = json.loads(raw, object_pairs_hook=strict_object)
    except json.JSONDecodeError as error:
        raise RunnerError(f"{label} is not valid JSON") from error
    if not isinstance(value, dict):
        raise RunnerError(f"{label} must be a JSON object")
    return value, raw


def _authenticate_fresh_run_manifest(envelope: object, key: bytes) -> dict[str, object]:
    if not isinstance(envelope, dict) or set(envelope) != {"manifest", "hmac_sha256"} or not isinstance(envelope["manifest"], dict) or not isinstance(envelope["hmac_sha256"], str):
        raise RunnerError("fresh run manifest envelope is invalid")
    manifest = envelope["manifest"]
    fields = {"schema", "schema_version", "status", "run_id", "plan_id", "plan_creation_sha256", "plan_self_sha256", "cohort_id", "binary_hashes", "source", "runner_inputs", "runtime", "inventory_sha256", "visual_count", "ordinary_count", "candidate_comparison_eligible", "checkpoint_reference_approval", "product_acceptance", "executed", "key_custody"}
    if set(manifest) != fields or manifest.get("schema") != FRESH_RUN_MANIFEST_SCHEMA or type(manifest.get("schema_version")) is not int or manifest["schema_version"] != FRESH_RUN_MANIFEST_VERSION:
        raise RunnerError("fresh run manifest schema is invalid")
    expected_signature = hmac.new(key, canonical_receipt_json(manifest), hashlib.sha256).hexdigest()
    if not hmac.compare_digest(expected_signature, envelope["hmac_sha256"]):
        raise RunnerError("fresh run manifest authentication failed")
    if manifest.get("status") != "prepared-metadata-only" or any(type(manifest.get(field)) is not bool or manifest[field] is not False for field in ("candidate_comparison_eligible", "product_acceptance", "executed")) or manifest.get("checkpoint_reference_approval") != "none":
        raise RunnerError("fresh run manifest is stale or grants forbidden privileges")
    return manifest


def create_fresh_run(repo: Path, root: Path, cohort_id: str, baseline_bin_dir: Path, timeout: int, expected_plan_sha256: str, key_path: Path) -> dict[str, object]:
    """Create authenticated PREPARED metadata only; this command never launches an application."""
    plan = verify_fresh_plan(repo, root, cohort_id, baseline_bin_dir, timeout, expected_plan_sha256)
    repo, root, _source = paths(repo, root)
    key_path = key_path.absolute()
    run_root = (root / "fresh-runs" / str(plan["plan_id"])).absolute()
    protected = [repo.resolve(), root.resolve(), Path(plan["source"]["source_path"]).resolve(), Path(plan["source"]["live_source_path"]).resolve(), Path(plan["execution"]["artifact_roots"]["run_root"]).resolve(), Path(plan["cohort"]["binaries"][APPS[0]]["path"]).resolve().parent]
    if any(key_path == item or item in key_path.parents for item in protected):
        raise RunnerError("fresh run key custody must be outside Git, source, binaries, and artifact roots")
    git_paths = _git_metadata_paths(repo)
    protected.extend(git_paths)
    if any(key_path == item or item in key_path.parents for item in protected):
        raise RunnerError("fresh run key custody must be outside Git, source, binaries, and artifact roots")
    custody_fd, _custody_info, key_path_parent = _private_custody_directory(key_path.parent, create=True)
    os.close(custody_fd)
    key_path = key_path_parent / key_path.name
    if any(key_path == item or item in key_path.parents for item in protected):
        raise RunnerError("fresh run key custody must be outside Git, source, binaries, and artifact roots")
    try:
        run_root.mkdir(parents=True, exist_ok=False, mode=0o700)
    except FileExistsError as error:
        raise RunnerError("fresh run directory already exists; refusing to reuse or clean evidence") from error
    key, key_identity, key_path = _stable_secret_key(key_path, create=True)
    manifest: dict[str, object] = {
        "schema": FRESH_RUN_MANIFEST_SCHEMA,
        "schema_version": FRESH_RUN_MANIFEST_VERSION,
        "status": "prepared-metadata-only",
        "run_id": secrets.token_hex(16),
        "plan_id": plan["plan_id"],
        "plan_creation_sha256": expected_plan_sha256,
        "plan_self_sha256": plan["plan_sha256"],
        "cohort_id": plan["cohort"]["cohort_id"],
        "binary_hashes": {app: plan["cohort"]["binaries"][app]["sha256"] for app in APPS},
        "source": plan["source"],
        "runner_inputs": plan["runner_inputs"],
        "runtime": plan["runtime"],
        "inventory_sha256": plan["inventory"]["sha256"],
        "visual_count": plan["inventory"]["visual_count"],
        "ordinary_count": plan["inventory"]["ordinary_count"],
        "candidate_comparison_eligible": False,
        "checkpoint_reference_approval": "none",
        "product_acceptance": False,
        "executed": False,
        "key_custody": {"path": str(key_path), "device": key_identity[0], "inode": key_identity[1], "bytes": key_identity[2], "mode": key_identity[3]},
    }
    body = canonical_receipt_json(manifest)
    envelope = {"manifest": manifest, "hmac_sha256": hmac.new(key, body, hashlib.sha256).hexdigest()}
    _atomic_create_bytes(run_root / FRESH_RUN_MANIFEST_FILE, canonical_receipt_json(envelope))
    print(json.dumps({"run_root": str(run_root), "run_id": manifest["run_id"], "status": manifest["status"], "executed": False}, indent=2))
    return manifest


def verify_fresh_run(repo: Path, root: Path, cohort_id: str, baseline_bin_dir: Path, timeout: int, expected_plan_sha256: str, key_path: Path) -> dict[str, object]:
    """Revalidate current plan/source/tool inputs and authenticate PREPARED metadata."""
    plan = verify_fresh_plan(repo, root, cohort_id, baseline_bin_dir, timeout, expected_plan_sha256)
    _repo, root, _source = paths(repo, root)
    run_root = (root / "fresh-runs" / str(plan["plan_id"])).absolute()
    key_path = key_path.absolute()
    git_paths = _git_metadata_paths(repo)
    protected = [repo.resolve(), root.resolve(), Path(plan["source"]["source_path"]).resolve(), Path(plan["source"]["live_source_path"]).resolve(), Path(plan["execution"]["artifact_roots"]["run_root"]).resolve(), Path(plan["cohort"]["binaries"][APPS[0]]["path"]).resolve().parent, *git_paths]
    if any(key_path == item or item in key_path.parents for item in protected):
        raise RunnerError("fresh run key custody must be outside Git, source, binaries, and artifact roots")
    key, key_identity, key_path = _stable_secret_key(key_path)
    if any(key_path == item or item in key_path.parents for item in protected):
        raise RunnerError("fresh run key custody must be outside Git, source, binaries, and artifact roots")
    envelope, _raw = _read_stable_json(run_root / FRESH_RUN_MANIFEST_FILE, "fresh run manifest")
    manifest = _authenticate_fresh_run_manifest(envelope, key)
    expected_manifest = {
        "schema": FRESH_RUN_MANIFEST_SCHEMA, "schema_version": FRESH_RUN_MANIFEST_VERSION,
        "status": "prepared-metadata-only", "plan_id": plan["plan_id"],
        "plan_creation_sha256": expected_plan_sha256, "plan_self_sha256": plan["plan_sha256"],
        "cohort_id": plan["cohort"]["cohort_id"], "source": plan["source"], "runner_inputs": plan["runner_inputs"],
        "binary_hashes": {app: plan["cohort"]["binaries"][app]["sha256"] for app in APPS},
        "runtime": plan["runtime"], "inventory_sha256": plan["inventory"]["sha256"],
        "visual_count": plan["inventory"]["visual_count"], "ordinary_count": plan["inventory"]["ordinary_count"],
        "candidate_comparison_eligible": False, "checkpoint_reference_approval": "none",
        "product_acceptance": False, "executed": False,
    }
    if any(type(manifest.get(field)) is not type(value) or manifest.get(field) != value for field, value in expected_manifest.items()):
        raise RunnerError("fresh run manifest is stale or grants forbidden privileges")
    custody = manifest["key_custody"]
    if custody != {"path": str(key_path.absolute()), "device": key_identity[0], "inode": key_identity[1], "bytes": key_identity[2], "mode": key_identity[3]}:
        raise RunnerError("fresh run key custody identity changed")
    envelopes = _fresh_journal_envelopes(run_root)
    progress = validate_fresh_run_journal(envelopes, key, envelope, root / FRESH_PLAN_FILE, expected_plan_sha256)
    _verify_fresh_journal_artifacts(plan, envelopes)
    print(json.dumps({"run_id": manifest["run_id"], "status": manifest["status"], "verified": True, "executed": False, "completed_steps": progress["completed_steps"], "visual_capture_replay_complete": progress["visual_capture_replay_complete"], "complete": False, "candidate_comparison_eligible": False, "product_acceptance": False}, indent=2))
    return manifest


def _fresh_run_receipt_body(envelope: object, key: bytes) -> dict[str, object]:
    if not isinstance(envelope, dict) or set(envelope) != {"receipt", "hmac_sha256"}:
        raise RunnerError("fresh run receipt envelope is malformed")
    receipt = envelope["receipt"]
    if not isinstance(receipt, dict) or not isinstance(envelope["hmac_sha256"], str):
        raise RunnerError("fresh run receipt is malformed")
    expected = hmac.new(key, canonical_receipt_json(receipt), hashlib.sha256).hexdigest()
    if not hmac.compare_digest(expected, envelope["hmac_sha256"]):
        raise RunnerError("fresh run receipt authentication failed")
    fields = {"schema", "ordinal", "phase", "chunk", "selected_cases", "status", "checkpoint_count", "ordinary_check_count", "process_receipt_count", "process_receipt_inventory_sha256", "process_pid_count", "checkpoint_inventory_sha256", "artifacts_sha256", "committed_chunk_sha256", "expected_chunk_sha256", "run_id", "plan_creation_sha256", "plan_self_sha256", "cohort_id", "binary_hashes", "source_sha256", "runtime_sha256", "previous_receipt_sha256"}
    if set(receipt) != fields or receipt.get("schema") != "termrock.frozen-cli-fresh-run-receipt-v2":
        raise RunnerError("fresh run receipt schema is invalid")
    if receipt.get("status") != "passed":
        raise RunnerError("fresh run journal cannot trust failed, skipped, or unfinished work")
    if type(receipt.get("ordinal")) is not int or receipt["ordinal"] < 0 or type(receipt.get("chunk")) is not int or receipt["chunk"] < 0:
        raise RunnerError("fresh run receipt ordinal or chunk is invalid")
    if receipt.get("phase") not in {"baseline-capture", "baseline-replay", "baseline-ordinary-checks"}:
        raise RunnerError("fresh run receipt phase is invalid")
    cases = receipt.get("selected_cases")
    if not isinstance(cases, list) or not cases or any(not isinstance(case, str) for case in cases) or len(cases) != len(set(cases)):
        raise RunnerError("fresh run receipt case set is empty or duplicated")
    for field in ("checkpoint_count", "ordinary_check_count", "process_receipt_count", "process_pid_count"):
        if type(receipt.get(field)) is not int or receipt[field] < 0:
            raise RunnerError(f"fresh run receipt {field} is invalid")
    if receipt["phase"] == "baseline-ordinary-checks":
        if receipt["ordinary_check_count"] == 0 or receipt["checkpoint_count"] != 0:
            raise RunnerError("ordinary-check receipt must prove checks and cannot claim visual checkpoints")
    elif receipt["checkpoint_count"] < len(cases) or receipt["ordinary_check_count"] != 0 or receipt["process_receipt_count"] == 0 or receipt["process_pid_count"] == 0:
        raise RunnerError("visual receipt must have nonzero checkpoints and cannot claim ordinary checks")
    for field in ("plan_creation_sha256", "plan_self_sha256", "source_sha256", "runtime_sha256", "expected_chunk_sha256", "process_receipt_inventory_sha256", "checkpoint_inventory_sha256", "artifacts_sha256", "committed_chunk_sha256"):
        if not isinstance(receipt.get(field), str) or not re.fullmatch(r"[0-9a-f]{64}", receipt[field]):
            raise RunnerError(f"fresh run receipt {field} is invalid")
    if not isinstance(receipt.get("binary_hashes"), dict) or set(receipt["binary_hashes"]) != set(APPS) or any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) for value in receipt["binary_hashes"].values()):
        raise RunnerError("fresh run receipt binary inventory is invalid")
    previous = receipt.get("previous_receipt_sha256")
    if previous is not None and (not isinstance(previous, str) or not re.fullmatch(r"[0-9a-f]{64}", previous)):
        raise RunnerError("fresh run receipt chain link is invalid")
    return receipt


def _fresh_plan_steps(plan: dict[str, object]) -> list[dict[str, object]]:
    phases = plan["execution"]["phases"]
    steps: list[dict[str, object]] = []
    for phase in phases:
        if phase["name"] in {"baseline-capture", "baseline-replay"}:
            for chunk in phase["chunks"]:
                steps.append({"phase": phase["name"], "chunk": chunk["index"], "selected_cases": chunk["selected_cases"]})
        elif phase["name"] == "baseline-ordinary-checks":
            for group in phase["groups"]:
                for chunk in group["chunks"]:
                    steps.append({"phase": phase["name"], "chunk": chunk["index"], "selected_cases": chunk["selected_cases"]})
    if not steps or not any(step["phase"] == "baseline-capture" for step in steps) or not any(step["phase"] == "baseline-replay" for step in steps) or not any(step["phase"] == "baseline-ordinary-checks" for step in steps):
        raise RunnerError("fresh run plan has an empty required phase inventory")
    return steps


def validate_fresh_run_journal(envelopes: list[object], key: bytes, manifest_envelope: object, plan_path: Path, expected_plan_sha256: str) -> dict[str, object]:
    """Validate an authenticated completed prefix against the externally anchored immutable plan."""
    manifest = _authenticate_fresh_run_manifest(manifest_envelope, key)
    plan = _read_self_sealed_plan(plan_path, expected_plan_sha256)
    if plan["plan_id"] != manifest.get("plan_id") or plan["plan_sha256"] != manifest.get("plan_self_sha256") or plan["inventory"]["sha256"] != manifest.get("inventory_sha256"):
        raise RunnerError("fresh run plan or inventory differs from the authenticated manifest")
    planned_steps = _fresh_plan_steps(plan)
    if not isinstance(envelopes, list) or len(envelopes) > len(planned_steps):
        raise RunnerError("fresh run journal has extra receipts")
    prior_digest: str | None = None
    captured_chunks: dict[int, str] = {}
    for ordinal, (envelope, expected_step) in enumerate(zip(envelopes, planned_steps, strict=False)):
        receipt = _fresh_run_receipt_body(envelope, key)
        if receipt["ordinal"] != ordinal or expected_step != {"phase": receipt["phase"], "chunk": receipt["chunk"], "selected_cases": receipt["selected_cases"]}:
            raise RunnerError("fresh run receipt order or planned chunk differs")
        expected_previous = prior_digest
        if receipt["previous_receipt_sha256"] != expected_previous:
            raise RunnerError("fresh run receipt chain is stale, missing, or reordered")
        expected_claims = {
            "run_id": manifest["run_id"], "plan_creation_sha256": manifest["plan_creation_sha256"],
            "plan_self_sha256": manifest["plan_self_sha256"], "cohort_id": manifest["cohort_id"],
            "binary_hashes": manifest["binary_hashes"], "source_sha256": manifest["source"]["source_sha256"],
            "runtime_sha256": canonical_json_digest(manifest["runtime"]),
        }
        if any(receipt.get(field) != value for field, value in expected_claims.items()):
            raise RunnerError("fresh run receipt is stale or bound to different source, binary, environment, or plan")
        if receipt["phase"] == "baseline-capture":
            if receipt["chunk"] in captured_chunks:
                raise RunnerError("fresh run journal duplicates a baseline capture chunk")
            captured_chunks[receipt["chunk"]] = receipt["expected_chunk_sha256"]
        elif receipt["phase"] == "baseline-replay":
            if captured_chunks.get(receipt["chunk"]) != receipt["expected_chunk_sha256"]:
                raise RunnerError("baseline replay does not bind its immutable captured expected chunk")
        else:
            raise RunnerError("fresh run journal contains an unsupported phase; ordinary checks are not implemented")
        prior_digest = hashlib.sha256(canonical_receipt_json(envelope)).hexdigest()
    visual_steps = sum(step["phase"] in {"baseline-capture", "baseline-replay"} for step in planned_steps)
    visual_complete = len(envelopes) >= visual_steps and all(step["phase"] in {"baseline-capture", "baseline-replay"} for step in planned_steps[:len(envelopes)])
    return {"completed_steps": len(envelopes), "planned_steps": len(planned_steps), "visual_capture_replay_complete": visual_complete, "complete": False, "executed": bool(envelopes), "candidate_comparison_eligible": False, "product_acceptance": False}


def _fresh_run_lock(run_root: Path):
    """Take a nonblocking advisory lock without exposing secrets to child processes."""
    try:
        root_info = run_root.lstat()
    except OSError as error:
        raise RunnerError(f"fresh run root is missing or unsafe: {error}") from error
    if stat.S_ISLNK(root_info.st_mode) or not stat.S_ISDIR(root_info.st_mode) or root_info.st_uid != os.geteuid() or stat.S_IMODE(root_info.st_mode) != 0o700:
        raise RunnerError("fresh run root must be an owned mode-0700 directory")
    path = run_root / ".advance.lock"
    flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
    fd: int | None = None
    try:
        fd = os.open(path, flags, 0o600)
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) != 0o600:
            raise RunnerError("fresh run lock is unsafe")
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as error:
        if fd is not None:
            os.close(fd)
        raise RunnerError("another fresh run chunk advance is active") from error
    except OSError as error:
        if fd is not None:
            os.close(fd)
        raise RunnerError(f"fresh run lock cannot be safely acquired: {error}") from error
    except RunnerError:
        if fd is not None:
            os.close(fd)
        raise
    return fd


def _fresh_journal_envelopes(run_root: Path) -> list[object]:
    journal = run_root / "journal"
    if not journal.exists() and not journal.is_symlink():
        return []
    info = journal.lstat()
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode) or info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise RunnerError("fresh run journal directory is unsafe")
    paths = sorted(journal.iterdir(), key=lambda item: item.name)
    expected_names = [f"step-{index:08}.json" for index in range(len(paths))]
    if [path.name for path in paths] != expected_names:
        raise RunnerError("fresh run journal contains a missing, duplicate, reordered, or unrecognized receipt")
    return [_read_stable_json(path, "fresh run journal receipt")[0] for path in paths]


def _fresh_expected_tree_sha256(path: Path) -> str:
    """Hash every expected frame plus type and permission mode; reject aliases and links."""
    if path.is_symlink() or not path.is_dir():
        raise RunnerError(f"expected chunk root is missing or unsafe: {path}")
    digest = hashlib.sha256()
    for current, directories, files in os.walk(path, followlinks=False):
        current_path = Path(current)
        for name in sorted(directories):
            item = current_path / name
            info = item.lstat()
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                raise RunnerError(f"expected chunk contains a non-directory or symlink: {item}")
            relative = item.relative_to(path).as_posix() + "/"
            digest.update(b"directory\0" + relative.encode() + b"\0" + f"{stat.S_IMODE(info.st_mode):04o}".encode() + b"\0")
        for name in sorted(files):
            item = current_path / name
            before = item.lstat()
            if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
                raise RunnerError(f"expected chunk contains an aliased or non-regular file: {item}")
            fd = os.open(item, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
            try:
                opened = os.fstat(fd)
                content = b""
                chunks = []
                while chunk := os.read(fd, 1024 * 1024):
                    chunks.append(chunk)
                after = os.fstat(fd)
            finally:
                os.close(fd)
            final = item.lstat()
            identity = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_size, value.st_mtime_ns, value.st_ctime_ns, value.st_nlink)
            if identity(before) != identity(opened) or identity(opened) != identity(after) or identity(after) != identity(final):
                raise RunnerError(f"expected chunk file changed while hashing: {item}")
            content = b"".join(chunks)
            relative = item.relative_to(path).as_posix()
            digest.update(b"file\0" + relative.encode() + b"\0" + f"{stat.S_IMODE(before.st_mode):04o}".encode() + b"\0" + hashlib.sha256(content).digest() + b"\0")
    return digest.hexdigest()


def _verify_fresh_journal_artifacts(plan: dict[str, object], envelopes: list[object]) -> None:
    roots = plan["execution"]["artifact_roots"]
    capture_root = Path(roots["baseline_capture"])
    for envelope in envelopes:
        receipt = envelope["receipt"]
        phase = receipt["phase"]
        chunk = receipt["chunk"]
        committed = Path(roots[phase.replace("-", "_")]) / f"chunk-{chunk:05}"
        if committed.is_symlink() or not committed.is_dir() or _fresh_expected_tree_sha256(committed) != receipt["committed_chunk_sha256"]:
            raise RunnerError("fresh run committed chunk is missing, changed, or not immutable")
        captured_expected = capture_root / f"chunk-{chunk:05}" / "expected"
        if _fresh_expected_tree_sha256(captured_expected) != receipt["expected_chunk_sha256"]:
            raise RunnerError("fresh run immutable expected frame chunk changed")


def _verify_fresh_staged_chunk(plan: dict[str, object], envelope: dict[str, object], stage: Path) -> None:
    """Validate a frozen attempt before publishing its directory under the final chunk name."""
    receipt = envelope["receipt"]
    if stage.is_symlink() or not stage.is_dir():
        raise RunnerError("fresh run staged chunk is missing or unsafe")
    if _fresh_expected_tree_sha256(stage) != receipt["committed_chunk_sha256"]:
        raise RunnerError("fresh run staged chunk changed before publication")
    phase = receipt["phase"]
    chunk = receipt["chunk"]
    if phase == "baseline-capture":
        expected_root = stage / "expected"
    elif phase == "baseline-replay":
        capture_root = Path(plan["execution"]["artifact_roots"]["baseline_capture"])
        expected_root = capture_root / f"chunk-{chunk:05}" / "expected"
    else:
        raise RunnerError("fresh run staged chunk has an unsupported phase")
    if _fresh_expected_tree_sha256(expected_root) != receipt["expected_chunk_sha256"]:
        raise RunnerError("fresh run staged chunk expected frames changed before publication")


def _fresh_journal_append(run_root: Path, envelope: dict[str, object], ordinal: int) -> Path:
    journal = run_root / "journal"
    try:
        journal.mkdir(mode=0o700)
    except FileExistsError:
        pass
    info = journal.lstat()
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode) or info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise RunnerError("fresh run journal directory is unsafe")
    dir_fd, opened = _open_directory_chain(journal)
    try:
        if (opened.st_dev, opened.st_ino, opened.st_mode) != (info.st_dev, info.st_ino, info.st_mode):
            raise RunnerError("fresh run journal directory changed while opening")
        name = f"step-{ordinal:08}.json"
        temporary = f".{name}.{secrets.token_hex(8)}"
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o400, dir_fd=dir_fd)
        try:
            os.fchmod(fd, 0o400)
            payload = canonical_receipt_json(envelope)
            view = memoryview(payload)
            while view:
                written = os.write(fd, view)
                view = view[written:]
            os.fsync(fd)
        finally:
            os.close(fd)
        try:
            os.link(temporary, name, src_dir_fd=dir_fd, dst_dir_fd=dir_fd, follow_symlinks=False)
        except FileExistsError as error:
            raise RunnerError(f"refusing to replace immutable fresh run journal receipt: {name}") from error
        finally:
            os.unlink(temporary, dir_fd=dir_fd)
        os.fsync(dir_fd)
    except OSError as error:
        raise RunnerError(f"fresh run receipt cannot be atomically published: {error}") from error
    finally:
        os.close(dir_fd)
    return journal / name


def advance_fresh_run(
    repo: Path,
    root: Path,
    cohort_id: str,
    baseline_bin_dir: Path,
    timeout: int,
    expected_plan_sha256: str,
    key_path: Path,
    *,
    expected_ordinal: int | None = None,
    expected_phase: str | None = None,
) -> dict[str, object]:
    """Run exactly the next planned baseline capture or replay chunk; never candidate-check or finalize."""
    if not 1 <= timeout <= FRESH_PLAN_MAX_TIMEOUT:
        raise RunnerError(f"fresh run advance timeout must be 1..{FRESH_PLAN_MAX_TIMEOUT} seconds")
    if (expected_ordinal is None) != (expected_phase is None):
        raise RunnerError("expected fresh run ordinal and phase must be supplied together")
    if expected_ordinal is not None and (type(expected_ordinal) is not int or expected_ordinal < 0):
        raise RunnerError("expected fresh run ordinal must be a nonnegative integer")
    if expected_phase is not None and (
        type(expected_phase) is not str
        or expected_phase not in {"baseline-capture", "baseline-replay"}
    ):
        raise RunnerError("expected fresh run phase must be baseline-capture or baseline-replay")
    repo, root, _source = paths(repo, root)
    plan_path = root / FRESH_PLAN_FILE
    plan = _read_self_sealed_plan(plan_path, expected_plan_sha256)
    run_root = root / "fresh-runs" / str(plan["plan_id"])
    lock_fd = _fresh_run_lock(run_root)
    stage: Path | None = None
    attempt_path: Path | None = None
    final_chunk: Path | None = None
    chunk_published = False
    deadline = time.monotonic() + timeout

    def remaining() -> int:
        seconds = int(deadline - time.monotonic())
        if seconds < 1:
            raise RunnerError("fresh run advance exceeded its single-chunk time budget")
        return seconds

    try:
        failed_publications = run_root / "failed-attempts"
        if failed_publications.exists() or failed_publications.is_symlink():
            if failed_publications.is_symlink() or not failed_publications.is_dir():
                raise RunnerError("fresh run failed-publication evidence path is unsafe")
            if any(failed_publications.iterdir()):
                raise RunnerError("fresh run has an unresolved post-publication failure; verify the committed ordinal before resuming")
        manifest = verify_fresh_run(repo, root, cohort_id, baseline_bin_dir, remaining(), expected_plan_sha256, key_path)
        manifest_envelope, _ = _read_stable_json(run_root / FRESH_RUN_MANIFEST_FILE, "fresh run manifest")
        key, _key_identity, _key_path = _stable_secret_key(key_path.absolute())
        envelopes = _fresh_journal_envelopes(run_root)
        prefix = validate_fresh_run_journal(envelopes, key, manifest_envelope, plan_path, expected_plan_sha256)
        _verify_fresh_journal_artifacts(plan, envelopes)
        planned = _fresh_plan_steps(plan)
        ordinal = prefix["completed_steps"]
        if ordinal >= len(planned):
            raise RunnerError("fresh run has no remaining planned chunks; finalization is not implemented")
        step = planned[ordinal]
        phase = step["phase"]
        if phase not in {"baseline-capture", "baseline-replay"}:
            raise RunnerError("ordinary-check advancement is not implemented; no phase was skipped")
        if expected_ordinal is not None and (ordinal != expected_ordinal or phase != expected_phase):
            raise RunnerError(
                "fresh run next step differs from the expected ordinal and phase "
                f"({expected_ordinal}, {expected_phase}); actual ({ordinal}, {phase})"
            )
        chunk_index = step["chunk"]
        selected = list(step["selected_cases"])
        if not selected:
            raise RunnerError("fresh run planned an empty visual chunk")
        expected_capture_root = Path(plan["execution"]["artifact_roots"][phase.replace("-", "_")]).resolve()
        root_artifacts = Path(plan["execution"]["artifact_roots"]["run_root"]).resolve()
        if expected_capture_root != root_artifacts / phase or root_artifacts != root / "fresh-capture" / str(plan["plan_id"]):
            raise RunnerError("fresh run artifact paths differ from the anchored plan")
        phase_root = expected_capture_root
        if phase_root.is_symlink():
            raise RunnerError("fresh run phase artifact root is symlinked")
        phase_root.mkdir(parents=True, exist_ok=True, mode=0o700)
        phase_info = phase_root.lstat()
        if not stat.S_ISDIR(phase_info.st_mode) or phase_info.st_uid != os.geteuid() or stat.S_IMODE(phase_info.st_mode) != 0o700:
            raise RunnerError("fresh run phase artifact root must be an owned mode-0700 directory")
        final_chunk = phase_root / f"chunk-{chunk_index:05}"
        if final_chunk.exists() or final_chunk.is_symlink():
            raise RunnerError("fresh run chunk output already exists without a matching journal receipt")
        stage = phase_root / f".attempt-{ordinal:08}-{secrets.token_hex(8)}"
        attempt_path = stage
        stage.mkdir(mode=0o700, exist_ok=False)
        live, _live_record = verify_live_source(repo, root, root / "source", remaining(), fresh_cohort_id=cohort_id, baseline_bin_dir=baseline_bin_dir)
        binary_hashes = phase_binary_hashes(baseline_bin_dir)
        expected_hashes = {app: manifest["binary_hashes"][app] for app in APPS}
        if binary_hashes != expected_hashes:
            raise RunnerError("fresh run baseline binaries differ from the authenticated manifest")
        expected_root = stage / "expected" if phase == "baseline-capture" else root_artifacts / "baseline-capture" / f"chunk-{chunk_index:05}" / "expected"
        expected_chunk_hash = None
        if phase == "baseline-replay":
            capture_step = next((envelopes[index]["receipt"] for index, prior in enumerate(planned[:len(envelopes)]) if prior["phase"] == "baseline-capture" and prior["chunk"] == chunk_index), None)
            if not isinstance(capture_step, dict):
                raise RunnerError("baseline replay has no authenticated completed capture chunk")
            expected_chunk_hash = capture_step["expected_chunk_sha256"]
            if _fresh_expected_tree_sha256(expected_root) != expected_chunk_hash:
                raise RunnerError("captured expected chunk changed before replay")
        extension_manifest = load_extension_manifest(repo / EXTENSION_MANIFEST)
        result = run_live_chunk(
            live, root, stage / "runner", phase, baseline_bin_dir, expected_root, selected, chunk_index,
            remaining(), expected_hashes, extension_manifest,
            term_profile=None if plan["runtime"]["terminal_profile"]["profile"] == "harness-default" else plan["runtime"]["terminal_profile"]["profile"],
        )
        if result["status"] != "passed" or result["checkpoint_count"] < len(selected) or result["successful_test_events"] != len(selected):
            raise RunnerError("Tuiscotti chunk did not produce successful tests and nonzero checkpoints for every selected case")
        if result["process_receipt_count"] == 0 or result["process_pid_count"] == 0:
            raise RunnerError("Tuiscotti chunk has no verified child launch and close receipts")
        if phase == "baseline-capture":
            if Path(expected_root).is_symlink() or not Path(expected_root).is_dir():
                raise RunnerError("baseline capture did not create an expected frame store")
            expected_chunk_hash = _fresh_expected_tree_sha256(expected_root)
            freeze_tree_readonly(expected_root)
            expected_chunk_hash = _fresh_expected_tree_sha256(expected_root)
        elif _fresh_expected_tree_sha256(expected_root) != expected_chunk_hash:
            raise RunnerError("baseline replay changed its immutable expected chunk")
        verify_fresh_run(repo, root, cohort_id, baseline_bin_dir, remaining(), expected_plan_sha256, key_path)
        after_hashes = phase_binary_hashes(baseline_bin_dir)
        if after_hashes != expected_hashes:
            raise RunnerError("baseline binaries changed during fresh chunk execution")
        source_digest = verify_source(root, root / "source")
        if source_digest != manifest["source"]["source_sha256"]:
            raise RunnerError("frozen baseline source changed during fresh chunk execution")
        remaining()
        runner_chunk = stage / "runner" / phase / f"chunk-{chunk_index:05}" / "report.json"
        chunk_report = json.loads(runner_chunk.read_text())
        chunk_report["receipt_root"] = f"runner/{phase}/chunk-{chunk_index:05}/receipts"
        chunk_report["artifact_dir"] = f"runner/{phase}/chunk-{chunk_index:05}/artifacts"
        runner_chunk.write_text(json.dumps(chunk_report, indent=2) + "\n")
        previous = hashlib.sha256(canonical_receipt_json(envelopes[-1])).hexdigest() if envelopes else None
        receipt = {
            "schema": "termrock.frozen-cli-fresh-run-receipt-v2", "ordinal": ordinal,
            **step, "status": "passed", "checkpoint_count": result["checkpoint_count"],
            "ordinary_check_count": 0, "process_receipt_count": result["process_receipt_count"],
            "process_receipt_inventory_sha256": result["process_receipt_inventory_sha256"],
            "process_pid_count": result["process_pid_count"],
            "checkpoint_inventory_sha256": result["checkpoint_inventory_sha256"],
            "artifacts_sha256": result["artifacts_sha256"],
            "expected_chunk_sha256": expected_chunk_hash,
            "run_id": manifest["run_id"], "plan_creation_sha256": manifest["plan_creation_sha256"],
            "plan_self_sha256": manifest["plan_self_sha256"], "cohort_id": manifest["cohort_id"],
            "binary_hashes": manifest["binary_hashes"], "source_sha256": source_digest,
            "runtime_sha256": canonical_json_digest(manifest["runtime"]), "previous_receipt_sha256": previous,
        }
        freeze_tree_readonly(stage)
        receipt["committed_chunk_sha256"] = _fresh_expected_tree_sha256(stage)
        remaining()
        envelope = {"receipt": receipt, "hmac_sha256": hmac.new(key, canonical_receipt_json(receipt), hashlib.sha256).hexdigest()}
        summary = validate_fresh_run_journal([*envelopes, envelope], key, manifest_envelope, plan_path, expected_plan_sha256)
        _verify_fresh_staged_chunk(plan, envelope, stage)
        chunk_published = True
        os.rename(stage, final_chunk)
        _verify_fresh_journal_artifacts(plan, [*envelopes, envelope])
        remaining()
        try:
            _fresh_journal_append(run_root, envelope, ordinal)
        except Exception as append_error:
            receipt_path = run_root / "journal" / f"step-{ordinal:08}.json"
            if not receipt_path.exists() and not receipt_path.is_symlink():
                raise
            published_envelope, _raw = _read_stable_json(receipt_path, "fresh run journal receipt")
            if published_envelope != envelope:
                raise RunnerError("fresh run append failed after a different immutable receipt was published")
            summary = validate_fresh_run_journal([*envelopes, published_envelope], key, manifest_envelope, plan_path, expected_plan_sha256)
            _verify_fresh_journal_artifacts(plan, [*envelopes, published_envelope])
            raise RunnerError(f"journal append raised after immutable ordinal {ordinal} was published; its evidence is preserved and requires verified resume: {append_error}") from append_error
        stage = None
        chunk_published = False
        output = {"run_id": manifest["run_id"], "phase": phase, "chunk": chunk_index, "selected_cases": selected, "checkpoint_count": result["checkpoint_count"], "expected_chunk_sha256": expected_chunk_hash, "journal_ordinal": ordinal, "visual_capture_replay_complete": summary["visual_capture_replay_complete"], "complete": False, "candidate_comparison_eligible": False, "checkpoint_reference_approval": "none", "product_acceptance": False}
        print(json.dumps(output, indent=2))
        return output
    except Exception as error:
        if chunk_published and final_chunk is not None and final_chunk.exists() and final_chunk.is_dir():
            receipt_path = run_root / "journal" / f"step-{ordinal:08}.json"
            if not receipt_path.exists() and not receipt_path.is_symlink() and attempt_path is not None and not attempt_path.exists():
                try:
                    os.rename(final_chunk, attempt_path)
                    stage = attempt_path
                    chunk_published = False
                except OSError as rollback_error:
                    error = RunnerError(f"{error}; failed to restore unreceipted chunk to its attempt directory: {rollback_error}")
        if stage is not None and stage.exists() and stage.is_dir() and not stage.is_symlink():
            try:
                stage_info = stage.lstat()
                if stage_info.st_uid != os.geteuid():
                    raise RunnerError("fresh run failed attempt is not owned by this user")
                original_mode = stat.S_IMODE(stage_info.st_mode)
                if not (stage_info.st_mode & 0o200):
                    stage.chmod(0o700)
                failure = {"status": "untrusted-failed-attempt", "error": str(error), "timestamp_utc": datetime.now(timezone.utc).isoformat(), "attempt_root_original_mode": f"{original_mode:04o}", "attempt_root_mode_for_marker": f"{stat.S_IMODE(stage.lstat().st_mode):04o}"}
                _atomic_create_bytes(stage / "untrusted-failure.json", canonical_receipt_json(failure))
            except (OSError, RunnerError) as marker_error:
                raise RunnerError(f"{error}; failed to preserve untrusted attempt evidence: {marker_error}") from error
        elif chunk_published:
            failure_root = run_root / "failed-attempts"
            try:
                failure_root.mkdir(mode=0o700, exist_ok=True)
                failure_info = failure_root.lstat()
                if stat.S_ISLNK(failure_info.st_mode) or not stat.S_ISDIR(failure_info.st_mode) or failure_info.st_uid != os.geteuid() or stat.S_IMODE(failure_info.st_mode) != 0o700:
                    raise RunnerError("fresh run failure evidence directory is unsafe")
                _atomic_create_bytes(failure_root / f"ordinal-{ordinal:08}-{secrets.token_hex(8)}.json", canonical_receipt_json({"status": "untrusted-failed-publication", "error": str(error), "timestamp_utc": datetime.now(timezone.utc).isoformat()}))
            except (OSError, RunnerError) as marker_error:
                raise RunnerError(f"{error}; failed to preserve publication failure evidence: {marker_error}") from error
        raise
    finally:
        fcntl.flock(lock_fd, fcntl.LOCK_UN)
        os.close(lock_fd)


def validate_chunk_coverage(selected: list[str], records: list[dict[str, object]], size: int) -> None:
    expected = scenario_chunks(selected, size)
    actual_ids = [record.get("chunk") for record in records]
    if actual_ids != list(range(len(expected))):
        raise RunnerError(f"chunk partition identities incomplete or duplicated: expected {list(range(len(expected)))}, got {actual_ids}")
    actual = [record.get("selected_cases") for record in records]
    if actual != expected:
        raise RunnerError("chunk partition case inventory differs from the exact expected partition")
    failed = [record.get("chunk") for record in records if record.get("status") != "passed"]
    if failed:
        raise RunnerError(f"chunk partition contains failed or unfinished chunks: {failed}")


def reference_case_selected(decision: dict[str, object], selected: list[str]) -> bool:
    return decision["tuple"]["test_id"] in selected or decision["tuple"]["case_id"] in selected


def validate_reference_receipts(receipts: dict[tuple[str, int, str], tuple[str, str, str | None]], selected: list[str], phase: str, decision: dict[str, object], decision_sha256: str) -> None:
    reference = decision["tuple"]
    key = (reference["case_id"], reference["ordinal"], reference["logical_name"])
    matched = receipts.get(key)
    if reference_case_selected(decision, selected):
        if matched is None:
            raise RunnerError(f"selected stale-approval reference checkpoint is missing: {key}")
        if matched[0] != decision["live_frame"]["ansi_sha256"] or matched[1] != decision["live_frame"]["semantic_frame_sha256"]:
            raise RunnerError(f"reviewed live-frame reference mismatch at {key}")
        expected_decision_hash = decision_sha256 if phase == "baseline-capture" else None
        if matched[2] != expected_decision_hash:
            raise RunnerError(f"reference decision receipt binding mismatch at {key}: {matched[2]}")
    elif any(value[2] is not None for value in receipts.values()):
        raise RunnerError("unselected or unrelated checkpoint claims a reference decision")


def paired_reference_report(decisions: list[dict[str, object]], selected: list[str], decision_sha256: str) -> list[dict[str, object]]:
    output = []
    for decision in decisions:
        if reference_case_selected(decision, selected):
            output.append({
                "decision_id": decision["decision_id"],
                "decision_sha256": decision_sha256,
                "test_id": decision["tuple"]["test_id"],
                "checkpoint": [decision["tuple"]["case_id"], decision["tuple"]["ordinal"], decision["tuple"]["logical_name"]],
                "legacy_approval_conflict_preserved": True,
                "legacy_approval_sha256": decision["legacy_approval"]["sha256"],
                "live_reference_ansi_sha256": decision["live_frame"]["ansi_sha256"],
                "live_reference_semantic_frame_sha256": decision["live_frame"]["semantic_frame_sha256"],
                "scope": "baseline-capture exact checkpoint only",
            })
    return output


def run_live_chunk(live: Path, root: Path, run_root: Path, phase: str, bins: Path, expected_root: Path, selected: list[str], index: int, timeout: int, expected_binary_hashes: dict[str, str], extension_manifest: dict[str, object] | None = None, reference_decisions: list[dict[str, object]] | None = None, reference_decision_sha256: str | None = None, term_profile: str | None = None) -> dict[str, object]:
    chunk_name = f"chunk-{index:05}"
    chunk_root = run_root / phase / chunk_name
    receipt_root = chunk_root / "receipts"
    logs = chunk_root / "logs"
    receipt_root.mkdir(parents=True, exist_ok=False)
    initial_binary_hashes = phase_binary_hashes(bins)
    if initial_binary_hashes != expected_binary_hashes:
        raise RunnerError(f"{phase} binaries changed before paired chunk: {bins}")
    process_receipt_key = secrets.token_hex(32)
    process_receipt_run_id = f"{run_root.name}:{phase}:{chunk_name}"
    env = clean_env()
    env.update({
        "CARGO_TARGET_DIR": str(root / "live-target"),
        "VISUAL_BIN_DIR": str(bins.resolve()),
        "VISUAL_FULL": "1",
        "VISUAL_REVIEW": "0",
        "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1",
        "TERMROCK_PAIR_PHASE": phase,
        "TERMROCK_PAIR_EXPECTED_ROOT": str(expected_root.resolve()),
        "TERMROCK_PAIR_RECEIPT_ROOT": str(receipt_root.resolve()),
        "TERMROCK_PAIR_PROCESS_RECEIPT_KEY": process_receipt_key,
        "TERMROCK_PAIR_PROCESS_RECEIPT_RUN_ID": process_receipt_run_id,
    })
    controlled_environment = apply_term_profile(env, term_profile)
    if term_profile is not None:
        env["TERMROCK_PAIR_TERM_PROFILE"] = term_profile
    reference_decisions = reference_decisions or []
    active_references = [decision for decision in reference_decisions if reference_case_selected(decision, selected)]
    if phase == "baseline-capture" and active_references:
        if len(active_references) != 1 or reference_decision_sha256 != REFERENCE_DECISIONS_SHA256:
            raise RunnerError("invalid or unpinned paired-reference decision")
        decision = active_references[0]
        for name, value in decision["environment"]["values"].items():
            env[name] = value
        controlled_environment = apply_term_profile(env, term_profile)
        ref_tuple = decision["tuple"]
        env.update({
            "TERMROCK_PAIR_REFERENCE_CASE": ref_tuple["case_id"],
            "TERMROCK_PAIR_REFERENCE_ORDINAL": str(ref_tuple["ordinal"]),
            "TERMROCK_PAIR_REFERENCE_LOGICAL": ref_tuple["logical_name"],
            "TERMROCK_PAIR_REFERENCE_ANSI_SHA256": decision["live_frame"]["ansi_sha256"],
            "TERMROCK_PAIR_REFERENCE_SEMANTIC_FRAME_SHA256": decision["live_frame"]["semantic_frame_sha256"],
            "TERMROCK_PAIR_REFERENCE_DECISION_SHA256": reference_decision_sha256,
        })
    elif any(name.startswith("TERMROCK_PAIR_REFERENCE_") for name in env):
        raise RunnerError("reference env unexpectedly survived clean environment")
    expression = selected_expression(selected)
    command = ["cargo", "nextest", "run", "--locked", "--manifest-path", str(live / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--run-ignored", "only", "--filter-expr", expression, "--message-format", "libtest-json-plus", "--no-fail-fast"]
    clear_runtime_artifacts(live)
    started = time.monotonic()
    failure = None
    events: dict[str, str] = {}
    try:
        stdout_path, stderr_path = logs / "stdout.jsonl", logs / "stderr.log"
        stdout = run(command, cwd=live, timeout=timeout, env=env, quiet=True, stdout_path=stdout_path, stderr_path=stderr_path)
        events = nextest_events(stdout, selected)
    except RunnerError as error:
        failure = str(error)
    try:
        final_binary_hashes = phase_binary_hashes(bins)
        if final_binary_hashes != initial_binary_hashes or final_binary_hashes != expected_binary_hashes:
            raise RunnerError(f"{phase} binaries changed during paired chunk: {bins}")
    except RunnerError as error:
        failure = f"{failure}; {error}" if failure else str(error)
    stdout_path = logs / "stdout.jsonl"
    stdout_gzip = logs / "stdout.jsonl.gz"
    if stdout_path.is_file():
        with stdout_path.open("rb") as source_stream, gzip.open(stdout_gzip, "wb", compresslevel=6) as compressed:
            shutil.copyfileobj(source_stream, compressed)
        stdout_path.unlink()
    try:
        artifacts = archive_live_artifacts(live, chunk_root, receipt_root)
    except RunnerError as error:
        failure = f"{failure}; {error}" if failure else str(error)
        artifacts = chunk_root / "artifacts"
    receipts: dict[tuple[str, int, str], tuple[str, str, str | None]] = {}
    process_summary: dict[str, object] = {"receipt_count": 0, "inventory_sha256": None, "pid_count": 0}
    if failure is None:
        try:
            receipts = read_receipts(receipt_root, [scenario_receipt_id(name, extension_manifest) for name in selected], expected_root)
            _, process_summary = verify_process_receipts(
                receipt_root, selected, phase, bins, expected_binary_hashes, extension_manifest,
                process_receipt_run_id, process_receipt_key, receipts,
            )
            for decision in reference_decisions:
                validate_reference_receipts(receipts, [scenario_receipt_id(name, extension_manifest) for name in selected], phase, decision, reference_decision_sha256 or "")
        except RunnerError as error:
            failure = str(error)
    artifact_digest = None
    if artifacts.is_dir() and not artifacts.is_symlink():
        try:
            artifact_digest = sealed_tree(artifacts)[0]
        except RunnerError as error:
            failure = f"{failure}; {error}" if failure else str(error)
    record = {
        "phase": phase,
        "chunk": index,
        "controlled_environment": controlled_environment,
        "status": "failed" if failure else "passed",
        "failure": failure,
        "selected_cases": selected,
        "successful_test_events": len(events),
        "checkpoint_count": len(receipts),
        "process_receipt_count": process_summary["receipt_count"],
        "process_receipt_inventory_sha256": process_summary["inventory_sha256"],
        "process_pid_count": process_summary["pid_count"],
        "phase_binary_hashes": final_binary_hashes if "final_binary_hashes" in locals() else None,
        "checkpoint_inventory_sha256": hashlib.sha256(json.dumps([[*key, *value] for key, value in sorted(receipts.items())], separators=(",", ":")).encode()).hexdigest(),
        "receipt_root": str(receipt_root),
        "artifact_dir": str(artifacts),
        "elapsed_seconds": round(time.monotonic() - started, 2),
        "artifacts_sha256": artifact_digest,
        "stdout_sha256": sha256(stdout_gzip) if stdout_gzip.is_file() else None,
        "stderr_sha256": sha256(logs / "stderr.log") if (logs / "stderr.log").is_file() else None,
    }
    chunk_root.mkdir(parents=True, exist_ok=True)
    (chunk_root / "report.json").write_text(json.dumps(record, indent=2) + "\n")
    if failure:
        raise RunnerError(f"{phase} {chunk_name} failed; report and artifacts saved: {failure}")
    return record


def phase_receipts(run_root: Path, phase: str, selected: list[str], expected_root: Path, extension_manifest: dict[str, object] | None = None) -> dict[tuple[str, int, str], tuple[str, str, str | None]]:
    combined: dict[tuple[str, int, str], tuple[str, str, str | None]] = {}
    for chunk in sorted((run_root / phase).glob("chunk-*")):
        names = json.loads((chunk / "report.json").read_text())["selected_cases"]
        chunk_cases = [scenario_receipt_id(name, extension_manifest) for name in names]
        chunk_receipts = read_receipts(chunk / "receipts", chunk_cases, expected_root)
        overlap = set(combined) & set(chunk_receipts)
        if overlap:
            raise RunnerError(f"duplicate phase checkpoint identities: {sorted(overlap)[:3]}")
        combined.update(chunk_receipts)
    counts = {scenario_receipt_id(name, extension_manifest): 0 for name in selected}
    for case, _, _ in combined:
        if case not in counts:
            raise RunnerError(f"phase contains unselected scenario receipt: {case}")
        counts[case] += 1
    missing = sorted(name for name, count in counts.items() if count == 0)
    if missing:
        raise RunnerError(f"phase scenario coverage incomplete: {missing[:5]}")
    expected_files = {name for case, ordinal, logical in combined for name in (f"checkpoint-{ordinal:08}-{checkpoint_identity(case, ordinal, logical)}.ansi", f"checkpoint-{ordinal:08}-{checkpoint_identity(case, ordinal, logical)}.frame.json")}
    _, actual_files = flat_sealed_store(expected_root, "runtime expected")
    if actual_files != expected_files:
        raise RunnerError(f"runtime expected frame inventory mismatch: missing={sorted(expected_files - actual_files)[:3]}, extra={sorted(actual_files - expected_files)[:3]}")
    if extension_manifest is not None:
        validate_extension_checkpoint_receipts(selected, combined, extension_manifest)
    return combined


def ordinary_test_groups(inventory: dict[str, bool], *, check_work: Callable[[], None] | None = None) -> tuple[list[str], list[str]]:
    selected = []
    for name, ignored in inventory.items():
        if check_work is not None:
            check_work()
        if not ignored:
            selected.append(name)
    selected.sort()
    if len(selected) != EXPECTED_ORDINARY_TESTS:
        raise RunnerError(f"ordinary test inventory changed: {len(selected)}; expected {EXPECTED_ORDINARY_TESTS}")
    cli = []
    for name in selected:
        if check_work is not None:
            check_work()
        if name == "isolation::env_isolation" or name.startswith(("modes::", "startup_input::", "startup_resize::")):
            cli.append(name)
    cli.sort()
    if set(cli) != EXPECTED_CLI_ORDINARY_TESTS:
        raise RunnerError(f"CLI ordinary-check inventory changed: expected {sorted(EXPECTED_CLI_ORDINARY_TESTS)}, got {cli}")
    return [name for name in selected if name not in EXPECTED_CLI_ORDINARY_TESTS], cli


def run_ordinary_checks(live: Path, root: Path, run_root: Path, inventory: dict[str, bool], baseline_bins: Path, candidate_bins: Path, timeout: int) -> dict[str, object]:
    source_only, cli = ordinary_test_groups(inventory)
    suites = (("source-only", source_only, None), ("baseline-cli", cli, baseline_bins), ("candidate-cli", cli, candidate_bins))
    results: dict[str, object] = {}
    for suite_name, selected, bins in suites:
        env = clean_env()
        if any(key.startswith("TERMROCK_PAIR_") for key in env):
            raise RunnerError("pair runtime flags leaked into ordinary checks")
        env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
        if bins is not None:
            env["VISUAL_BIN_DIR"] = str(bins.resolve())
        command = ["cargo", "nextest", "run", "--locked", "--manifest-path", str(live / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--filter-expr", selected_expression(selected), "--message-format", "libtest-json-plus", "--no-fail-fast"]
        output_dir = run_root / "ordinary" / suite_name
        output_dir.mkdir(parents=True, exist_ok=False)
        failure = None
        events: dict[str, str] = {}
        try:
            stdout = run(command, cwd=live, timeout=timeout, env=env, quiet=True, stdout_path=output_dir / "stdout.jsonl", stderr_path=output_dir / "stderr.log")
            events = nextest_events(stdout, selected)
        except RunnerError as error:
            failure = str(error)
        stdout_path = output_dir / "stdout.jsonl"
        stdout_gzip = output_dir / "stdout.jsonl.gz"
        if stdout_path.is_file():
            with stdout_path.open("rb") as source_stream, gzip.open(stdout_gzip, "wb", compresslevel=6) as compressed:
                shutil.copyfileobj(source_stream, compressed)
            stdout_path.unlink()
        result = {"status": "failed" if failure else "passed", "failure": failure, "selected_cases": selected, "binary_dir": str(bins.resolve()) if bins is not None else None, "passed_test_events": len(events), "stdout_sha256": sha256(stdout_gzip) if stdout_gzip.is_file() else None, "stderr_sha256": sha256(output_dir / "stderr.log") if (output_dir / "stderr.log").is_file() else None}
        (output_dir / "report.json").write_text(json.dumps(result, indent=2) + "\n")
        results[suite_name] = result
        if failure:
            raise RunnerError(f"ordinary {suite_name} checks failed; report saved: {output_dir}: {failure}")
    return results


def compare_live(repo: Path, candidate_repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, selectors: list[str] | None, all_cases: bool, chunk_size: int, timeout: int, term_profile: str | None = None) -> None:
    repo, root, source = paths(repo, root)
    candidate_repo = candidate_repo.resolve()
    baseline_build, candidate = baseline_and_candidate_records(candidate_repo, root, baseline_bins, candidate_bins, timeout)
    live, live_record = verify_live_source(repo, root, source, timeout)
    reference_decisions, reference_decision_sha256 = load_reference_decisions(repo, root, source)
    extension_manifest_path = repo / EXTENSION_MANIFEST
    extension_manifest = load_extension_manifest(extension_manifest_path)
    extension_manifest_hash = sha256(extension_manifest_path)
    if live_record.get("extension_manifest_sha256") != extension_manifest_hash:
        raise RunnerError("extension manifest changed after live-source preparation")
    env = clean_env()
    env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
    inventory = nextest_inventory(live, timeout, env, extension_manifest)
    require_inventory_matches(inventory, str(live_record.get("test_inventory_sha256")))
    original_inventory, extension_ids = validate_extension_inventory(inventory, extension_manifest)
    if inventory_digest(original_inventory) != live_record.get("original_test_inventory_sha256"):
        raise RunnerError("original 28,895-scenario/20-check inventory changed")
    all_names = sorted(name for name, ignored in inventory.items() if ignored)
    if all_cases == (selectors is not None):
        raise RunnerError("choose exactly one: --all or one or more --case selectors")
    if all_cases:
        selected = all_names
    else:
        selected = sorted(set(selectors or []))
        if not selected:
            raise RunnerError("select at least one exact scenario case")
        for name in selected:
            if name not in inventory:
                raise RunnerError(f"selected zero baseline tests: {name!r}")
            if not inventory[name]:
                raise RunnerError(f"selected test is not an ignored visual scenario: {name}")
    validate_reference_term_profile(reference_decisions, selected, term_profile)
    controlled_env = clean_env()
    controlled_environment = apply_term_profile(controlled_env, term_profile)
    chunks = scenario_chunks(selected, chunk_size)
    original_snapshots_hash = digest_tree(source / "snapshots")
    harness_hash = digest_tree(source / "tests" / "visual_baseline")
    selection_digest = hashlib.sha256("\n".join(selected).encode()).hexdigest()[:12]
    run_id = f"{time.time_ns()}-{selection_digest}"
    run_root = root / "live-paired" / run_id
    run_root.mkdir(parents=True, exist_ok=False)
    expected_root = run_root / "runtime-expected"
    expected_root.mkdir()
    phases = []
    phase_bins = (("baseline-capture", baseline_bins), ("baseline-check", baseline_bins), ("candidate-check", candidate_bins))
    for phase, bins in phase_bins:
        phase_dir = run_root / phase
        phase_dir.mkdir(parents=True, exist_ok=False)
        if phase != "baseline-capture":
            expected_before = flat_sealed_store(expected_root, "runtime expected")[0]
        else:
            expected_before = None
        started = time.monotonic()
        phase_chunks = []
        expected_binary_hashes = baseline_build["binaries"] if phase in {"baseline-capture", "baseline-check"} else candidate["binaries"]
        for index, names in enumerate(chunks):
            try:
                phase_chunks.append(run_live_chunk(live, root, run_root, phase, bins, expected_root, names, index, timeout, expected_binary_hashes, extension_manifest, reference_decisions, reference_decision_sha256, term_profile))
            except RunnerError as error:
                phase_chunks.append({"chunk": index, "status": "failed", "failure": str(error), "selected_cases": names, "controlled_environment": controlled_environment})
                report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "controlled_environment": controlled_environment, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "complete registered scenarios pending integrity checks" if all_cases else "partial", **cli_coverage_report_fields(False), "selected_case_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "remaining_chunks": len(chunks) - index - 1}
                (run_root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
                raise RunnerError(f"{phase} stopped at {index + 1}/{len(chunks)} chunks; evidence saved at {run_root}: {error}")
        try:
            validate_chunk_coverage(selected, phase_chunks, chunk_size)
        except RunnerError as error:
            failed_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "controlled_environment": controlled_environment, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", **cli_coverage_report_fields(False), "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "failure": str(error)}
            (run_root / "report.json").write_text(json.dumps(failed_report, indent=2) + "\n")
            raise RunnerError(f"{phase} chunk partition incomplete; report saved at {run_root}: {error}")
        phase_summary: dict[str, object] = {"phase": phase, "status": "passed", "selected_cases": len(selected), "chunks": phase_chunks, "elapsed_seconds": round(time.monotonic() - started, 2)}
        try:
            if phase != "baseline-capture" and flat_sealed_store(expected_root, "runtime expected")[0] != expected_before:
                raise RunnerError(f"{phase} modified runtime expected frames")
            phase_summary["receipts"] = phase_receipts(run_root, phase, selected, expected_root, extension_manifest)
            phase_summary["runtime_expected_sha256"] = flat_sealed_store(expected_root, "runtime expected")[0]
            verify_source(root, source)
            current_live, current_live_record = verify_live_source(repo, root, source, timeout)
            if current_live != live or current_live_record != live_record:
                raise RunnerError("live harness patch, helper, or source changed during paired run")
        except RunnerError as error:
            phase_summary["status"] = "failed"
            phase_summary["failure"] = str(error)
            phase_reports = [{key: value for key, value in item.items() if key != "receipts"} for item in [*phases, phase_summary]]
            failure_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "controlled_environment": controlled_environment, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", **cli_coverage_report_fields(False), "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phases": phase_reports, "artifacts_root": str(run_root)}
            (run_root / "report.json").write_text(json.dumps(failure_report, indent=2) + "\n")
            raise RunnerError(f"{phase} phase incomplete; evidence saved at {run_root}: {error}")
        phases.append(phase_summary)
        phase_reports = [{key: value for key, value in item.items() if key != "receipts"} for item in phases]
        (run_root / "report.json").write_text(json.dumps({"phase": phase, "phases": phase_reports}, indent=2) + "\n")
    receipts = [phase["receipts"] for phase in phases]
    differences = exact_receipt_differences(receipts[0], receipts[1]) + exact_receipt_differences(receipts[0], receipts[2])
    if differences:
        mismatch_report = {
            "tag_object": BASELINE_TAG_OBJECT,
            "baseline_commit": BASELINE_COMMIT,
            "tuiscotti_rev": TUISCOTTI_REV,
            "live_source": live_record,
            "candidate_revision": candidate["revision"],
            "candidate_source_tree": candidate["source_tree"],
            "candidate_dirty_fingerprint": candidate["dirty_fingerprint"],
            "candidate_binaries": candidate["binaries"],
            "controlled_environment": controlled_environment,
            "case_selection": "all registered scenarios" if all_cases else "explicit cases",
            "selection_completeness": "complete" if all_cases else "partial",
            "selected_scenario_count": len(selected),
            "original_scenario_count": EXPECTED_SCENARIOS,
            "extension_scenario_count": len(extension_ids),
            "extension_manifest_sha256": extension_manifest_hash,
            "test_inventory_sha256": inventory_digest(inventory),
            "checkpoint_differences": differences,
            "checkpoint_receipt_sha256": [hashlib.sha256(json.dumps([[*key, *value] for key, value in sorted(receipt.items())], separators=(",", ":")).encode()).hexdigest() for receipt in receipts],
            "phases": [{key: value for key, value in phase.items() if key != "receipts"} for phase in phases],
            "ordinary_checks": "not run because paired checkpoints differed",
            **cli_coverage_report_fields(False),
            "artifacts_root": str(run_root),
        }
        (run_root / "report.json").write_text(json.dumps(mismatch_report, indent=2) + "\n")
        raise RunnerError(f"live checkpoint mismatch; see {run_root}: {differences[:5]}")
    ordinary = None
    if all_cases:
        try:
            ordinary = run_ordinary_checks(live, root, run_root, inventory, baseline_bins, candidate_bins, timeout)
        except RunnerError as error:
            failure_report = {
                "tag_object": BASELINE_TAG_OBJECT,
                "baseline_commit": BASELINE_COMMIT,
                "tuiscotti_rev": TUISCOTTI_REV,
                "live_source": live_record,
                "candidate_revision": candidate["revision"],
                "candidate_source_tree": candidate["source_tree"],
                "candidate_dirty_fingerprint": candidate["dirty_fingerprint"],
                "candidate_binaries": candidate["binaries"],
                "controlled_environment": controlled_environment,
                "case_selection": "all registered scenarios",
                "selection_completeness": "incomplete",
                **cli_coverage_report_fields(False),
                "selected_scenario_count": len(selected),
                "test_inventory_sha256": inventory_digest(inventory),
                "ordinary_failure": str(error),
                "phases": [{key: value for key, value in phase.items() if key != "receipts"} for phase in phases],
                "artifacts_root": str(run_root),
            }
            (run_root / "report.json").write_text(json.dumps(failure_report, indent=2) + "\n")
            raise RunnerError(f"ordinary checks incomplete; report saved at {run_root / 'report.json'}: {error}") from error
    source_only, ordinary_cli = ordinary_test_groups(inventory)
    coverage_complete = (
        all_cases
        and ordinary is not None
        and ordinary["source-only"]["status"] == "passed"
        and ordinary["source-only"]["selected_cases"] == source_only
        and ordinary["baseline-cli"]["status"] == "passed"
        and ordinary["baseline-cli"]["selected_cases"] == ordinary_cli
        and ordinary["candidate-cli"]["status"] == "passed"
        and ordinary["candidate-cli"]["selected_cases"] == ordinary_cli
    )
    report = {
        "tag_object": BASELINE_TAG_OBJECT,
        "baseline_commit": BASELINE_COMMIT,
        "tuiscotti_rev": TUISCOTTI_REV,
        "baseline_source_sha256": verify_source(root, source),
        "baseline_source_tree": BASELINE_TREE,
        "baseline_binaries": json.loads((root / "build.json").read_text())["binaries"],
        "original_snapshots_sha256": original_snapshots_hash,
        "original_harness_sha256": harness_hash,
        "live_source": live_record,
        "reference_decisions_sha256": reference_decision_sha256,
        "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256),
        "candidate_revision": candidate["revision"],
        "candidate_source_tree": candidate["source_tree"],
        "candidate_dirty_fingerprint": candidate["dirty_fingerprint"],
        "candidate_dirty_files": candidate["dirty_files"],
        "candidate_binaries": candidate["binaries"],
        "controlled_environment": controlled_environment,
        "case_selection": "all registered scenarios" if all_cases else "explicit cases",
        "selection_completeness": "complete" if coverage_complete else "partial",
        "expected_registered_scenarios": EXPECTED_SCENARIOS,
        "expected_extension_scenarios": len(extension_ids),
        "extension_manifest_sha256": extension_manifest_hash,
        "extension_inventory_sha256": live_record["extension_inventory_sha256"],
        "selected_original_scenario_count": sum(name in original_inventory and inventory[name] for name in selected),
        "selected_extension_scenario_count": sum(name in extension_ids for name in selected),
        "test_inventory_sha256": inventory_digest(inventory),
        "selected_scenario_count": len(selected),
        "ordinary_checks": ordinary,
        "phases": [{key: value for key, value in phase.items() if key != "receipts"} for phase in phases],
        "checkpoint_count": len(receipts[0]),
        "checkpoint_receipt_sha256": [hashlib.sha256(json.dumps([[*key, *value] for key, value in sorted(receipt.items())], separators=(",", ":")).encode()).hexdigest() for receipt in receipts],
        "checkpoint_differences": differences,
        **cli_coverage_report_fields(coverage_complete and not differences),
        "artifacts_root": str(run_root),
    }
    (run_root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    if all_cases and not coverage_complete:
        raise RunnerError(f"registered scenario comparison incomplete; report saved: {run_root / 'report.json'}")
    print(json.dumps(report, indent=2))


def parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--repo", type=Path, default=Path.cwd())
    ap.add_argument("--root", type=Path, default=Path("target/frozen-baseline"))
    ap.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    sub = ap.add_subparsers(dest="command", required=True)
    prepare_parser = sub.add_parser("prepare")
    prepare_parser.add_argument("--owned-lifecycle", action="store_true", help="use one cancellation-safe stage-one deadline and anchored child cleanup")
    prepare_parser.add_argument("--owned-budget", type=int, default=PREPARE_OWNED_MAX_BUDGET, help="work seconds for --owned-lifecycle; cleanup reserve is additional and fixed")
    prepare_live = sub.add_parser("prepare-live")
    prepare_live.add_argument("--owned-lifecycle", action="store_true", help="use one cancellation-safe deadline and anchored child cleanup for live-source preparation")
    prepare_live.add_argument("--owned-budget", type=int, default=PREPARE_OWNED_MAX_BUDGET, help="work seconds for --owned-lifecycle; cleanup reserve is additional and fixed")
    prepare_live.add_argument("--fresh-cohort", help="explicit reviewed fresh-capture cohort; bypasses legacy build.json/tuple approval path")
    prepare_live.add_argument("--baseline-bin-dir", type=Path, help="sealed application directory required with --fresh-cohort")
    verify_live = sub.add_parser("verify-live")
    verify_live.add_argument("--fresh-cohort", required=True)
    verify_live.add_argument("--baseline-bin-dir", type=Path, required=True)
    verify_live.add_argument("--owned-lifecycle", action="store_true", help="use one cancellation-safe deadline and anchored child cleanup for live-source verification")
    verify_live.add_argument("--owned-budget", type=int, default=PREPARE_OWNED_MAX_BUDGET, help="work seconds for --owned-lifecycle; cleanup reserve is additional and fixed")
    plan = sub.add_parser("fresh-plan")
    plan.add_argument("--cohort", required=True)
    plan.add_argument("--baseline-bin-dir", type=Path, required=True)
    plan.add_argument("--chunk-size", type=int, default=DEFAULT_CHUNK_SIZE)
    plan.add_argument("--term-profile", choices=sorted(TERM_PROFILES), default=None)
    plan.add_argument("--owned-lifecycle", action="store_true", help="use one cancellation-safe deadline and anchored child cleanup for plan creation")
    plan.add_argument("--owned-budget", type=int, help="work seconds for --owned-lifecycle; cleanup reserve is additional and fixed")
    verify_plan = sub.add_parser("verify-fresh-plan")
    verify_plan.set_defaults(timeout=min(MAX_TIMEOUT, FRESH_PLAN_MAX_TIMEOUT))
    verify_plan.add_argument("--cohort", required=True)
    verify_plan.add_argument("--baseline-bin-dir", type=Path, required=True)
    verify_plan.add_argument("--expected-plan-sha256", required=True, help="digest retained from fresh-plan creation; self-digest alone is not an approval anchor")
    verify_plan.add_argument("--owned-lifecycle", action="store_true", help="use one cancellation-safe deadline and anchored child cleanup for plan verification")
    verify_plan.add_argument("--owned-budget", type=int, help="work seconds for --owned-lifecycle; cleanup reserve is additional and fixed")
    fresh_run_init = sub.add_parser("fresh-run-init", help="create authenticated PREPARED metadata only; launches no application")
    fresh_run_init.add_argument("--cohort", required=True)
    fresh_run_init.add_argument("--baseline-bin-dir", type=Path, required=True)
    fresh_run_init.add_argument("--expected-plan-sha256", required=True)
    fresh_run_init.add_argument("--key-path", type=Path, required=True, help="private 0600 key path outside Git, source, binaries, and artifacts")
    fresh_run_verify = sub.add_parser("fresh-run-verify", help="verify authenticated PREPARED metadata only; launches no application")
    fresh_run_verify.add_argument("--cohort", required=True)
    fresh_run_verify.add_argument("--baseline-bin-dir", type=Path, required=True)
    fresh_run_verify.add_argument("--expected-plan-sha256", required=True)
    fresh_run_verify.add_argument("--key-path", type=Path, required=True)
    for command_parser in (fresh_run_init, fresh_run_verify):
        command_parser.set_defaults(timeout=min(MAX_TIMEOUT, FRESH_PLAN_MAX_TIMEOUT))
    fresh_run_advance = sub.add_parser("fresh-run-advance", help="run exactly the next planned baseline capture or replay chunk; no candidate comparison or finalization")
    fresh_run_advance.set_defaults(timeout=min(MAX_TIMEOUT, FRESH_PLAN_MAX_TIMEOUT))
    fresh_run_advance.add_argument("--cohort", required=True)
    fresh_run_advance.add_argument("--baseline-bin-dir", type=Path, required=True)
    fresh_run_advance.add_argument("--expected-plan-sha256", required=True)
    fresh_run_advance.add_argument("--key-path", type=Path, required=True)
    fresh_run_advance.add_argument("--expected-ordinal", type=int)
    fresh_run_advance.add_argument("--expected-phase", choices=("baseline-capture", "baseline-replay"))
    plan.set_defaults(timeout=min(MAX_TIMEOUT, FRESH_PLAN_MAX_TIMEOUT))
    sub.add_parser("build")
    candidate_build = sub.add_parser("build-candidate")
    candidate_build.add_argument("--target-dir", type=Path)
    pair = sub.add_parser("compare")
    pair.add_argument("--candidate-bin-dir", type=Path, required=True)
    selection = pair.add_mutually_exclusive_group(required=True)
    selection.add_argument("--case", help="exact ignored visual scenario test name")
    selection.add_argument("--all", action="store_true", help="run the complete frozen visual scenario suite")
    live_pair = sub.add_parser("live-compare")
    live_pair.add_argument("--candidate-bin-dir", type=Path, required=True)
    live_pair.add_argument("--candidate-repo", type=Path, help="checkout used to build candidate binaries; defaults to --repo")
    live_selection = live_pair.add_mutually_exclusive_group(required=True)
    live_selection.add_argument("--case", action="append", help="exact ignored scenario test name; may be repeated")
    live_selection.add_argument("--all", action="store_true", help="run all registered ignored visual scenarios")
    live_pair.add_argument("--chunk-size", type=int, default=DEFAULT_CHUNK_SIZE)
    live_pair.add_argument("--term-profile", choices=sorted(TERM_PROFILES), default=None,
                           help="set runner and spawned-child TERM for every phase (default: harness child default; 'unset' removes TERM)")
    return ap


def main() -> int:
    if len(sys.argv) == 4 and sys.argv[1] == "--_owned-prepare-broker":
        try:
            return _owned_broker_main(Path(sys.argv[2]), int(sys.argv[3]))
        except (RunnerError, OSError, subprocess.SubprocessError, json.JSONDecodeError, ValueError) as error:
            print(f"frozen-baseline owned broker: {error}", file=sys.stderr)
            return 2
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            if args.owned_lifecycle:
                with PrepareExecution(args.owned_budget) as execution:
                    prepare(args.repo, args.root, args.timeout, execution=execution)
            else:
                if args.owned_budget != PREPARE_OWNED_MAX_BUDGET:
                    raise RunnerError("--owned-budget requires --owned-lifecycle")
                prepare(args.repo, args.root, args.timeout)
        elif args.command == "prepare-live":
            repo, root, source = paths(args.repo, args.root)
            if args.fresh_cohort is None and args.baseline_bin_dir is not None:
                raise RunnerError("--baseline-bin-dir requires --fresh-cohort")
            if args.fresh_cohort is not None and args.baseline_bin_dir is None:
                raise RunnerError("--fresh-cohort requires --baseline-bin-dir")
            if args.owned_lifecycle:
                if args.fresh_cohort is None or args.baseline_bin_dir is None:
                    raise RunnerError("--owned-lifecycle requires explicit --fresh-cohort and --baseline-bin-dir")
                with PrepareExecution(args.owned_budget) as execution:
                    verify_source(root, source, check_work=execution.check_work)
                    record = prepare_live_source(repo, root, source, args.timeout, fresh_cohort_id=args.fresh_cohort, baseline_bin_dir=args.baseline_bin_dir, execution=execution)
            else:
                if args.owned_budget != PREPARE_OWNED_MAX_BUDGET:
                    raise RunnerError("--owned-budget requires --owned-lifecycle")
                verify_source(root, source)
                record = prepare_live_source(repo, root, source, args.timeout, fresh_cohort_id=args.fresh_cohort, baseline_bin_dir=args.baseline_bin_dir)
            print(json.dumps(record, indent=2))
        elif args.command == "verify-live":
            if args.owned_lifecycle:
                if args.fresh_cohort != PREPARE_LIVE_OWNED_COHORT:
                    raise RunnerError(
                        "--owned-lifecycle requires the pinned frozen-20261021-serial-nonincr-r1-r2 cohort"
                    )
                with PrepareExecution(args.owned_budget) as execution:
                    execution.check_work()
                    repo, root, source = paths(args.repo, args.root)
                    execution.check_work()
                    verify_source(root, source, check_work=execution.check_work)
                    live, record = verify_live_source(
                        repo,
                        root,
                        source,
                        args.timeout,
                        fresh_cohort_id=args.fresh_cohort,
                        baseline_bin_dir=args.baseline_bin_dir,
                        execution=execution,
                    )
                    success = json.dumps({"live_source": str(live), "verified": True, "record": record}, indent=2)
                    execution.check_work()
                    print(success, flush=True)
                    execution.check_work()
            else:
                if args.owned_budget != PREPARE_OWNED_MAX_BUDGET:
                    raise RunnerError("--owned-budget requires --owned-lifecycle")
                repo, root, source = paths(args.repo, args.root)
                verify_source(root, source)
                live, record = verify_live_source(
                    repo,
                    root,
                    source,
                    args.timeout,
                    fresh_cohort_id=args.fresh_cohort,
                    baseline_bin_dir=args.baseline_bin_dir,
                )
                print(json.dumps({"live_source": str(live), "verified": True, "record": record}, indent=2))
        elif args.command == "fresh-plan":
            if args.owned_lifecycle:
                if args.cohort != PREPARE_LIVE_OWNED_COHORT:
                    raise RunnerError("--owned-lifecycle requires the pinned fresh-capture cohort")
                owned_budget = PREPARE_OWNED_MAX_BUDGET if args.owned_budget is None else args.owned_budget
                with PrepareExecution(owned_budget) as execution:
                    create_fresh_plan(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.chunk_size, args.term_profile, execution=execution)
            else:
                if args.owned_budget is not None:
                    raise RunnerError("--owned-budget requires --owned-lifecycle")
                create_fresh_plan(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.chunk_size, args.term_profile)
        elif args.command == "verify-fresh-plan":
            if args.owned_lifecycle:
                if args.cohort != PREPARE_LIVE_OWNED_COHORT:
                    raise RunnerError("--owned-lifecycle requires the pinned fresh-capture cohort")
                owned_budget = PREPARE_OWNED_MAX_BUDGET if args.owned_budget is None else args.owned_budget
                with PrepareExecution(owned_budget) as execution:
                    verify_fresh_plan(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.expected_plan_sha256, execution=execution)
            else:
                if args.owned_budget is not None:
                    raise RunnerError("--owned-budget requires --owned-lifecycle")
                verify_fresh_plan(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.expected_plan_sha256)
        elif args.command == "fresh-run-init":
            create_fresh_run(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.expected_plan_sha256, args.key_path)
        elif args.command == "fresh-run-verify":
            verify_fresh_run(args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout, args.expected_plan_sha256, args.key_path)
        elif args.command == "fresh-run-advance":
            if (args.expected_ordinal is None) != (args.expected_phase is None):
                raise RunnerError("--expected-ordinal and --expected-phase must be supplied together")
            advance_fresh_run(
                args.repo, args.root, args.cohort, args.baseline_bin_dir, args.timeout,
                args.expected_plan_sha256, args.key_path,
                expected_ordinal=args.expected_ordinal, expected_phase=args.expected_phase,
            )
        elif args.command == "build":
            build(args.repo, args.root, args.timeout)
        elif args.command == "build-candidate":
            root = args.root.resolve()
            default_target = root / "candidate-targets" / f"build-{time.time_ns()}"
            build_candidate(args.repo, root, args.target_dir or default_target, args.timeout)
        elif args.command == "live-compare":
            root = args.root.resolve()
            baseline_bins = root / "cargo-target" / "debug"
            compare_live(args.repo, args.candidate_repo or args.repo, root, baseline_bins, args.candidate_bin_dir.resolve(), args.case, args.all, args.chunk_size, args.timeout, args.term_profile)
        else:
            root = args.root.resolve()
            baseline_bins = root / "cargo-target" / "debug"
            compare(args.repo, root, baseline_bins, args.candidate_bin_dir.resolve(), args.case, args.all, args.timeout)
    except (RunnerError, OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"frozen-baseline: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
