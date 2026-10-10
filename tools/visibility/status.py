#!/usr/bin/env python3
"""Render Termrock visibility facts and bounded, source-pinned observations."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import shlex
import stat
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Mapping, Optional, Sequence

ROOT = Path(__file__).resolve().parents[2]
FACTS_PATH = ROOT / "tools/visibility/source-facts.json"
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"
STATUS_PATH = ROOT / "STATUS.md"
REPOSITORY = "tailrocks/terminal-components-claude"
TAG_OBJECT = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
TAG_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
MAX_EXECUTION_EVIDENCE_BYTES = 8 * 1024 * 1024
EVIDENCE_TMP_ROOT = Path("/private/tmp")
STATUS_EVIDENCE_ARCHIVE_ROOT = Path(
    "docs/implementation/visibility/evidence/reports/status-source-archive-20261009"
)
STATUS_EVIDENCE_ARCHIVE_RAW_ROOT = STATUS_EVIDENCE_ARCHIVE_ROOT / "raw"
ARCHIVED_EXTERNAL_EVIDENCE_PATHS: Optional[frozenset[str]] = None
ARCHIVED_REPOSITORY_EVIDENCE_PATHS: Optional[frozenset[str]] = None
PRIORITY = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
CURRENT_CI_WORKFLOW = ".github/workflows/ci.yml"
WORKFLOW_SIZE_ANNOTATION = (
    "Workflow file exceeds the maximum allowed size of 500 KB. See "
    "https://docs.github.com/actions/reference/limits#workflow-file-size "
    "for more information."
)

COMMANDS = (
    ("RUN-01", "cargo run --release --bin showcase"),
    ("RUN-02", "cargo run --release --bin jackin-preview"),
    ("RUN-03", "cargo run --release --bin holla"),
    ("RUN-04", "cargo run --release --bin tablepro"),
    ("RUN-05", "cargo run --release --bin tablepro -- --connect Production"),
    ("RUN-06", "cargo run --release --bin jackin-preview -- --scenario accounts-mixed"),
    ("RUN-07", "cargo run --release --bin holla -- --scenario remote-host"),
)
APPS = ("Showcase", "Jackin Preview", "Holla", "TablePro")
HOLLA_CHECKPOINTS = ("00-boot", "01-help", "02-finder", "03-help-again")
HOLLA_INTERACTION_ASSERTIONS = {
    "00-boot": (
        "boot.disk_cursor", "boot.finder_footer", "boot.placeholder", "boot.world",
    ),
    "01-help": (
        "help.finder_footer_absent", "help.finder_row", "help.footer",
        "help.quit_row", "help.title_here",
    ),
    "02-finder": ("finder.placeholder_restored", "finder.quit_row_absent"),
    "03-help-again": ("reopen.title_here",),
}
HOLLA_ARTIFACT_FORMATS = (
    "ansi", "ascii", "ascii_loss_json", "frame_json", "html", "manifest_json",
    "observations_json", "png", "png_fidelity_json", "txt",
)
STATUS_ORDER = ("PASS", "FAIL", "BLOCKED", "NOT_RUN", "NOT_APPLICABLE")
EXECUTION_ATTEMPT_HISTORY_SCHEMA = "termrock-status-execution-attempt-history/v1"
PRODUCT_PHASE_OBSERVATIONS_SCHEMA = "termrock-status-product-phase-observations/v1"
PRODUCT_PHASE_ARCHIVE_SCHEMA = "termrock-vis13-raw-attempt-archive/v1"
PRODUCT_PHASE_ARCHIVE_PREFIX = (
    "docs/implementation/visibility/evidence/reports/"
)
PRODUCT_PHASE_ARCHIVE_MAX_MEMBERS = 512
PRODUCT_PHASE_ARCHIVE_MAX_BYTES = 64 * 1024 * 1024
PROVIDER_OBSERVATIONS_SCHEMA = "termrock-status-provider-observations/v1"
# Provider captures use a stricter, provider-specific manifest shape than the
# shared VIS-13 raw-attempt archive consumed by product-phase observations.
PROVIDER_ARCHIVE_SCHEMA = "termrock-status-provider-capture-archive/v1"
PROVIDER_POLL_OBSERVATIONS_SCHEMA = "termrock-status-provider-observations/v2"
PROVIDER_POLL_VARIANT = "workflow_run_poll"
PROVIDER_POLL_ARCHIVE_SCHEMA = "termrock-status-provider-poll-archive/v1"
PROVIDER_POLL_CAPTURE_FILENAMES = {
    "summary": "observation.json",
    "checksums": "SHA256SUMS",
    "pull_request": "pull-17.json",
    "candidate_ref": "head-termrock-implementation.json",
    "commit": "commit-object.json",
    "run_api": "run-api.json",
    "run_view": "run-view.json",
    "jobs_api": "run-jobs-api.json",
    "jobs_api_stderr": "run-jobs-api.stderr",
    "artifacts_api": "run-artifacts-api.json",
    "run_log_stdout": "run-log.txt",
    "run_log_stderr": "run-log.stderr",
    "dco_checks": "check-runs-retry.json",
}
PROVIDER_POLL_JSON_ROLES = {
    "summary", "pull_request", "candidate_ref", "commit", "run_api",
    "run_view", "jobs_api", "artifacts_api", "dco_checks",
}
PROVIDER_CAPTURE_ROLES = {
    "workflow", "run", "check_suites", "suite_check_runs", "check_runs",
    "jobs", "artifacts", "run_logs_headers", "run_logs_body",
}
PROVIDER_JSON_ROLES = {
    "workflow", "run", "check_suites", "suite_check_runs", "check_runs",
    "jobs", "artifacts",
}
PROVIDER_CAPTURE_FILENAMES = {
    "workflow": "workflow.json",
    "run": "run.json",
    "check_suites": "check-suites.json",
    "suite_check_runs": "suite-check-runs.json",
    "check_runs": "check-runs.json",
    "jobs": "jobs.json",
    "artifacts": "artifacts.json",
    "run_logs_headers": "run-logs.headers",
    "run_logs_body": "run-logs.body",
}
PRODUCT_PHASE_NAMES = (
    "build", "launch", "first_frame", "interaction", "exit", "restoration",
    "visual", "ownership", "cleanup",
)
PRODUCT_PHASE_DISPLAY_NAMES = PRODUCT_PHASE_NAMES[:-1]
EXECUTION_ATTEMPT_LANES = {
    "ci_provider": "candidate_commit",
    "paired_nextest": "candidate_reference_pair",
    "tag_capture": "immutable_visual_tag",
    "tool_control": "tool_source",
    "observer_ingest": "tool_source",
}
EXECUTION_ATTEMPT_COUNT_FIELDS = (
    "inventory", "selected", "started", "passed", "failed", "filtered", "incomplete",
)
EXECUTION_ATTEMPT_STATE_FIELDS = {
    "test": {"PASS", "FAIL", "INCOMPLETE", "NOT_RUN"},
    "child": {"PASS", "FAIL", "NOT_RUN", "NOT_APPLICABLE"},
    "wrapper": {"PASS", "FAIL", "INVALID", "INCOMPLETE", "NOT_RUN", "NOT_APPLICABLE"},
    "collector": {"PASS", "FAIL", "INCOMPLETE", "NOT_RUN", "NOT_APPLICABLE"},
    "postflight": {"PASS", "FAIL", "UNVERIFIED", "NOT_RUN", "NOT_APPLICABLE"},
    "cleanup": {"PASS", "FAIL", "UNVERIFIED", "NOT_RUN", "NOT_APPLICABLE"},
    "product_qualification": {"BLOCKED", "NOT_QUALIFIED", "NOT_RUN", "NOT_APPLICABLE"},
    "capture": {"COMPLETE", "INCOMPLETE", "NOT_RUN", "NOT_APPLICABLE"},
    "admission": {"ADMITTED", "NOT_RUN", "NOT_APPLICABLE"},
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def require_repository_relative_source_paths(value: Any) -> None:
    if isinstance(value, str):
        require(not value.startswith(EVIDENCE_TMP_ROOT.as_posix() + "/"),
                "source-facts paths must be repository-relative, not temporary-root locators")
    elif isinstance(value, list):
        for item in value:
            require_repository_relative_source_paths(item)
    elif isinstance(value, dict):
        for item in value.values():
            require_repository_relative_source_paths(item)


def sha(value: Any, label: str) -> str:
    require(isinstance(value, str) and SHA.fullmatch(value) is not None,
            "{} must be a full lowercase SHA-1".format(label))
    return value


def sha256(value: Any, label: str) -> str:
    require(isinstance(value, str) and SHA256.fullmatch(value) is not None,
            "{} must be a full lowercase SHA-256".format(label))
    return value


def timestamp(value: Any, label: str) -> str:
    require(isinstance(value, str), "{} must be text".format(label))
    try:
        datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError as error:
        raise ValueError("{} must be UTC RFC3339 seconds".format(label)) from error
    return value


def timestamp_with_offset(value: Any, label: str) -> str:
    require(isinstance(value, str), "{} must be text".format(label))
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError("{} must be RFC3339 with a timezone".format(label)) from error
    require(parsed.utcoffset() is not None,
            "{} must include a timezone offset".format(label))
    return value


def validate_source_evidence(evidence: Any, label: str) -> Mapping[str, Any]:
    require(isinstance(evidence, dict), "{} is required".format(label))
    require(isinstance(evidence.get("path"), str) and evidence["path"].strip(),
            "{} path is required".format(label))
    sha256(evidence.get("sha256"), "{}.sha256".format(label))
    return evidence



def pinned_evidence_location(raw_path: Path, label: str) -> tuple[Path, Path]:
    if raw_path.is_absolute():
        try:
            relative_path = raw_path.relative_to(EVIDENCE_TMP_ROOT)
        except ValueError as error:
            raise ValueError(
                "{} absolute evidence is outside the archived source root".format(label)
            ) from error
        original_relative_path = relative_path.as_posix()
        require(ARCHIVED_EXTERNAL_EVIDENCE_PATHS is not None
                and original_relative_path in ARCHIVED_EXTERNAL_EVIDENCE_PATHS,
                "{} absolute evidence is not in the pinned repository archive".format(label))
        evidence_root = ROOT.resolve() / STATUS_EVIDENCE_ARCHIVE_RAW_ROOT
    else:
        evidence_root = ROOT.resolve()
        relative_path = raw_path
        archive_relative_path = relative_path.as_posix()
        if archive_relative_path.startswith(STATUS_EVIDENCE_ARCHIVE_RAW_ROOT.as_posix() + "/"):
            require(ARCHIVED_REPOSITORY_EVIDENCE_PATHS is not None
                    and archive_relative_path in ARCHIVED_REPOSITORY_EVIDENCE_PATHS,
                    "{} repository archive path is not listed in its pinned manifest".format(label))
    return evidence_root, relative_path


def canonical_evidence_path(value: Any, label: str) -> str:
    require(isinstance(value, str) and value.strip(),
            "{} evidence path is required".format(label))
    raw_path = Path(value)
    if raw_path.is_absolute():
        try:
            relative_path = raw_path.relative_to(EVIDENCE_TMP_ROOT)
        except ValueError as error:
            raise ValueError(
                "{} absolute evidence is outside the archived source root".format(label)
            ) from error
        original_relative_path = relative_path.as_posix()
        require(ARCHIVED_EXTERNAL_EVIDENCE_PATHS is not None
                and original_relative_path in ARCHIVED_EXTERNAL_EVIDENCE_PATHS,
                "{} absolute evidence is not in the pinned repository archive".format(label))
        return (STATUS_EVIDENCE_ARCHIVE_RAW_ROOT / relative_path).as_posix()
    return raw_path.as_posix()


def read_pinned_bytes(evidence: Any, label: str) -> bytes:
    validate_source_evidence(evidence, label)
    raw_path = Path(evidence["path"])
    evidence_root, relative_path = pinned_evidence_location(raw_path, label)
    path_parts = relative_path.parts
    require(bool(path_parts) and all(part not in {"", ".", ".."} for part in path_parts),
            "{} evidence path must stay below its allowed root".format(label))
    require(hasattr(os, "O_DIRECTORY") and hasattr(os, "O_NOFOLLOW")
            and hasattr(os, "O_NONBLOCK"),
            "{} secure component-wise evidence reads are unavailable".format(label))
    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    root_descriptor = -1
    descriptor = -1
    try:
        root_descriptor = os.open(os.sep, directory_flags)
        for component in evidence_root.parts[1:]:
            next_descriptor = os.open(component, directory_flags, dir_fd=root_descriptor)
            previous_descriptor = root_descriptor
            root_descriptor = next_descriptor
            os.close(previous_descriptor)
        for component in path_parts[:-1]:
            next_descriptor = os.open(
                component, directory_flags, dir_fd=root_descriptor
            )
            previous_descriptor = root_descriptor
            root_descriptor = next_descriptor
            os.close(previous_descriptor)
        descriptor = os.open(
            path_parts[-1],
            os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
            dir_fd=root_descriptor,
        )
    except OSError as error:
        raise ValueError("{} cannot be opened without following symlinks: {}".format(
            label, error
        )) from error
    finally:
        if root_descriptor >= 0:
            os.close(root_descriptor)
    try:
        with os.fdopen(descriptor, "rb") as stream:
            file_stat = os.fstat(stream.fileno())
            if not stat.S_ISREG(file_stat.st_mode):
                raise ValueError("{} must be a regular file".format(label))
            require(file_stat.st_size <= MAX_EXECUTION_EVIDENCE_BYTES,
                    "{} exceeds the bounded evidence-file size".format(label))
            raw = stream.read(MAX_EXECUTION_EVIDENCE_BYTES + 1)
            require(len(raw) <= MAX_EXECUTION_EVIDENCE_BYTES,
                    "{} exceeds the bounded evidence-file size".format(label))
    except OSError as error:
        raise ValueError("{} cannot be read: {}".format(label, error)) from error
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    require(actual_sha256 == evidence["sha256"],
            "{} bytes do not match the recorded SHA-256".format(label))
    return raw


def read_pinned_json(
    evidence: Any, label: str, queue_module: Any
) -> Any:
    validate_source_evidence(evidence, label)
    raw_path = Path(evidence["path"])
    evidence_root, relative_path = pinned_evidence_location(raw_path, label)
    path_parts = relative_path.parts
    require(bool(path_parts) and all(part not in {"", ".", ".."} for part in path_parts),
            "{} evidence path must stay below its allowed root".format(label))
    require(hasattr(os, "O_DIRECTORY") and hasattr(os, "O_NOFOLLOW")
            and hasattr(os, "O_NONBLOCK"),
            "{} secure component-wise evidence reads are unavailable".format(label))
    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    root_descriptor = -1
    try:
        root_descriptor = os.open(os.sep, directory_flags)
        for component in evidence_root.parts[1:]:
            next_descriptor = os.open(component, directory_flags, dir_fd=root_descriptor)
            previous_descriptor = root_descriptor
            root_descriptor = next_descriptor
            os.close(previous_descriptor)
        for component in path_parts[:-1]:
            next_descriptor = os.open(
                component, directory_flags, dir_fd=root_descriptor
            )
            previous_descriptor = root_descriptor
            root_descriptor = next_descriptor
            os.close(previous_descriptor)
        descriptor = os.open(
            path_parts[-1],
            os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
            dir_fd=root_descriptor,
        )
    except OSError as error:
        raise ValueError("{} cannot be opened without following symlinks: {}".format(
            label, error
        )) from error
    finally:
        if root_descriptor >= 0:
            os.close(root_descriptor)
    try:
        with os.fdopen(descriptor, "rb") as stream:
            file_stat = os.fstat(stream.fileno())
            if not stat.S_ISREG(file_stat.st_mode):
                raise ValueError("{} must be a regular file".format(label))
            require(file_stat.st_size <= MAX_EXECUTION_EVIDENCE_BYTES,
                    "{} exceeds the bounded evidence-file size".format(label))
            raw = stream.read(MAX_EXECUTION_EVIDENCE_BYTES + 1)
            require(len(raw) <= MAX_EXECUTION_EVIDENCE_BYTES,
                    "{} exceeds the bounded evidence-file size".format(label))
    except OSError as error:
        raise ValueError("{} cannot be read: {}".format(label, error)) from error
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    require(actual_sha256 == evidence["sha256"],
            "{} bytes do not match the recorded SHA-256".format(label))
    try:
        value = queue_module.strict_json_loads(raw)
    except Exception as error:
        raise ValueError("{} is not strict JSON: {}".format(label, error)) from error
    require(isinstance(value, (dict, list)),
            "{} must contain a JSON object or array".format(label))
    return value


def validate_source_evidence_archive(
    pin: Any, queue_module: Any, *, required: bool
) -> Optional[Mapping[str, Any]]:
    global ARCHIVED_EXTERNAL_EVIDENCE_PATHS, ARCHIVED_REPOSITORY_EVIDENCE_PATHS
    ARCHIVED_EXTERNAL_EVIDENCE_PATHS = frozenset()
    ARCHIVED_REPOSITORY_EVIDENCE_PATHS = frozenset()
    if pin is None:
        require(not required,
                "current status observations require a repository-relative source evidence archive")
        return None
    require(isinstance(pin, dict)
            and pin.get("schema") == "termrock-status-source-evidence-archive-pin/v1",
            "unsupported source evidence archive pin")
    manifest_pin = pin.get("manifest")
    require(isinstance(manifest_pin, dict)
            and manifest_pin.get("path") ==
            (STATUS_EVIDENCE_ARCHIVE_ROOT / "MANIFEST.json").as_posix(),
            "source evidence archive manifest must use the pinned repository-relative path")
    manifest = read_pinned_json(
        manifest_pin, "source evidence archive manifest", queue_module,
    )
    require(isinstance(manifest, dict)
            and manifest.get("schema") == "termrock-status-source-evidence-archive/v1"
            and manifest.get("archive_root") == STATUS_EVIDENCE_ARCHIVE_ROOT.as_posix()
            and manifest.get("raw_root") == STATUS_EVIDENCE_ARCHIVE_RAW_ROOT.as_posix()
            and isinstance(manifest.get("inventory_method"), str)
            and manifest["inventory_method"].strip(),
            "source evidence archive manifest identity is malformed")
    files = manifest.get("files")
    require(isinstance(files, list) and 0 < len(files) <= 100
            and type(manifest.get("file_count")) is int
            and manifest["file_count"] == len(files)
            and type(manifest.get("total_bytes")) is int
            and 0 <= manifest["total_bytes"] <= 8 * 1024 * 1024,
            "source evidence archive count or total byte bound is invalid")
    require(all(isinstance(item, dict)
                and isinstance(item.get("original_relative_path"), str)
                and isinstance(item.get("path"), str)
                for item in files),
            "source evidence archive entries must have text paths")
    external_index = {item["original_relative_path"] for item in files}
    repository_index = {manifest_pin["path"]}
    repository_index.update(item["path"] for item in files)
    require(len(external_index) == len(files)
            and len(repository_index) == len(files) + 1,
            "source evidence archive repeats a path")
    ARCHIVED_EXTERNAL_EVIDENCE_PATHS = frozenset(external_index)
    ARCHIVED_REPOSITORY_EVIDENCE_PATHS = frozenset(repository_index)
    seen_external = set()
    seen_repository = {manifest_pin["path"]}
    total_bytes = 0
    for index, item in enumerate(files):
        label = "source evidence archive files[{}]".format(index)
        require(isinstance(item, dict), "{} must be an object".format(label))
        original = item.get("original_relative_path")
        archived = item.get("path")
        require(isinstance(original, str) and original.strip(),
                "{}.original_relative_path is required".format(label))
        original_parts = Path(original).parts
        require(not Path(original).is_absolute() and bool(original_parts)
                and all(part not in {"", ".", ".."} for part in original_parts),
                "{}.original_relative_path must be a safe relative path".format(label))
        require(original not in seen_external,
                "source evidence archive repeats an original path")
        seen_external.add(original)
        expected_archived = (
            STATUS_EVIDENCE_ARCHIVE_RAW_ROOT / Path(original)
        ).as_posix()
        require(archived == expected_archived,
                "{}.path does not preserve the source-relative archive layout".format(label))
        archived_parts = Path(archived).parts
        require(not Path(archived).is_absolute() and bool(archived_parts)
                and all(part not in {"", ".", ".."} for part in archived_parts),
                "{}.path must be a safe repository-relative path".format(label))
        require(archived not in seen_repository,
                "source evidence archive repeats an archived path")
        seen_repository.add(archived)
        count = item.get("bytes")
        require(type(count) is int and 0 <= count <= MAX_EXECUTION_EVIDENCE_BYTES,
                "{}.bytes is outside the evidence file bound".format(label))
        digest = sha256(item.get("sha256"), "{}.sha256".format(label))
        require(isinstance(item.get("read_kinds"), list)
                and item["read_kinds"]
                and all(kind in {"READ_JSON", "READ_BYTES"}
                        for kind in item["read_kinds"])
                and isinstance(item.get("labels"), list)
                and item["labels"]
                and all(isinstance(name, str) and name.strip()
                        for name in item["labels"]),
                "{}.read_kinds or labels are malformed".format(label))
        raw = read_pinned_bytes(
            {"path": archived, "sha256": digest}, "{} archived bytes".format(label),
        )
        require(len(raw) == count,
                "{} bytes do not match the archive manifest".format(label))
        total_bytes += count
    require(total_bytes == manifest["total_bytes"]
            and pin.get("file_count") == len(files)
            and pin.get("total_bytes") == total_bytes,
            "source evidence archive totals do not match the source-facts pin")
    require(pin.get("effect_on_product_qualification") == "NONE",
            "evidence archive cannot change product qualification")
    return manifest


def validate_historical_evidence_gaps(
    value: Any, history: Sequence[Mapping[str, Any]]
) -> Mapping[str, Any]:
    require(isinstance(value, dict)
            and value.get("schema") == "termrock-status-historical-evidence-gaps/v1"
            and value.get("status") == "RAW_CAPTURE_GAP",
            "historical raw-evidence gap record is required")
    records = value.get("records")
    require(isinstance(records, list) and len(records) == len(history),
            "historical evidence-gap records must match retained source history")
    for index, (gap, historical) in enumerate(zip(records, history)):
        source = historical["source_observation"]
        pair = source["candidate_remote"], source["reference_remote"]
        require(isinstance(gap, dict)
                and gap.get("history_index") == index
                and gap.get("candidate_sha") == pair[0]["head_sha"]
                and gap.get("reference_sha") == pair[1]["head_sha"]
                and gap.get("normalized_source_observation") == "RETAINED"
                and gap.get("normalized_ci_observation") == (
                    "NOT_RECORDED"
                    if historical.get("current_ci_observation") is None else "RETAINED"
                )
                and gap.get("raw_source_evidence") == "NOT_ARCHIVED"
                and gap.get("raw_ci_capture") == (
                    "NOT_RECORDED"
                    if historical.get("current_ci_observation") is None else "NOT_ARCHIVED"
                ),
                "historical evidence gap {} does not match its source record".format(index))
    require(value.get("report_readiness_effect")
            == "NONE; all existing report-level readiness values remain NOT_RUN",
            "historical evidence gaps cannot change report readiness")
    return value


def validate_product_evidence_packet(
    manifest_pin: Any, measurement: Mapping[str, Any], queue_module: Any
) -> Mapping[str, Any]:
    manifest = read_pinned_json(
        manifest_pin, "candidate measurement packet manifest", queue_module,
    )
    require(isinstance(manifest, dict)
            and manifest.get("schema") == "termrock-evidence-file-hashes/v1"
            and manifest.get("includes_self") is False,
            "candidate measurement packet manifest identity changed")
    files = manifest.get("files")
    require(isinstance(files, list) and len(files) == 22,
            "candidate measurement packet must enumerate its 22 files")
    manifest_path = Path(manifest_pin["path"])
    packet_root = manifest_path.parent
    seen = set()
    total_bytes = 0
    hashes = {}
    for index, item in enumerate(files):
        label = "candidate measurement packet files[{}]".format(index)
        require(isinstance(item, dict), "{} must be an object".format(label))
        relative = item.get("path")
        require(isinstance(relative, str) and relative.strip(),
                "{}.path is required".format(label))
        member = Path(relative)
        require(not member.is_absolute() and bool(member.parts)
                and all(part not in {"", ".", ".."} for part in member.parts),
                "{}.path must be a safe packet-relative path".format(label))
        member_name = member.as_posix()
        require(member_name not in seen,
                "candidate measurement packet repeats a member path")
        seen.add(member_name)
        digest = sha256(item.get("sha256"), "{}.sha256".format(label))
        count = item.get("bytes")
        require(type(count) is int and 0 <= count <= MAX_EXECUTION_EVIDENCE_BYTES,
                "{}.bytes is outside the evidence file bound".format(label))
        path = (packet_root / member).as_posix()
        raw = read_pinned_bytes(
            {"path": path, "sha256": digest}, "{} bytes".format(label),
        )
        require(len(raw) == count,
                "{} byte count does not match its manifest".format(label))
        hashes[member_name] = digest
        total_bytes += count
    require(total_bytes <= 8 * 1024 * 1024,
            "candidate measurement packet exceeds its total byte bound")
    runs_root = Path("runs/2026-10-09T03-04-14-975Z-10bae096")
    for field in ("receipt", "ledger", "review"):
        evidence = measurement.get(field)
        require(isinstance(evidence, dict),
                "candidate measurement {} pin is required".format(field))
        try:
            relative = Path(evidence["path"]).relative_to(packet_root).as_posix()
        except (KeyError, ValueError) as error:
            raise ValueError(
                "candidate measurement {} must be inside its pinned packet".format(field)
            ) from error
        expected_name = {
            "receipt": "receipt.json",
            "ledger": "ledger-record.json",
            "review": "actual-independent-review-luna.json",
        }[field]
        require(relative == (runs_root / expected_name).as_posix()
                and hashes.get(relative) == evidence.get("sha256"),
                "candidate measurement {} pin does not match its packet".format(field))
    return manifest


def source_pair_matches(recorded: Any, expected_candidate: str,
                        expected_reference: str, label: str) -> None:
    require(isinstance(recorded, dict), "{} source pair is required".format(label))
    require(sha(recorded.get("candidate_commit"),
                "{}.candidate_commit".format(label)) == expected_candidate,
            "{} candidate commit does not match the selected source".format(label))
    require(sha(recorded.get("reference_commit"),
                "{}.reference_commit".format(label)) == expected_reference,
            "{} reference commit does not match the selected source".format(label))


def load_deferred_execution(
    record: Any, candidate_sha: str, reference_sha: str, queue_module: Any
) -> Mapping[str, Any]:
    require(isinstance(record, dict), "deferred_nextest observation is required")
    seal = read_pinned_json(record.get("seal"), "deferred Nextest seal", queue_module)
    case_results = read_pinned_json(
        record.get("case_results"), "deferred Nextest case results", queue_module
    )
    case_map = read_pinned_json(
        record.get("case_map"), "deferred Nextest case map", queue_module
    )
    review = read_pinned_json(
        record.get("review"), "deferred Nextest independent review", queue_module
    )
    require(isinstance(seal, dict) and seal.get("schema")
            == "termrock-deferred-nextest-execution-v1"
            and seal.get("kind") == "actual-execution",
            "deferred Nextest seal schema or kind changed")
    source = seal.get("source")
    require(isinstance(source, dict), "deferred Nextest source identity is required")
    require(sha(source.get("commit"), "deferred Nextest source commit") == candidate_sha,
            "deferred Nextest candidate commit does not match the selected source")
    sha(source.get("tree"), "deferred Nextest source tree")
    sha256(source.get("registry_sha256"), "deferred Nextest registry SHA-256")
    require(type(source.get("source_inputs_rechecked_after_run")) is bool
            and source["source_inputs_rechecked_after_run"],
            "deferred Nextest source inputs were not rechecked after execution")
    execution = seal.get("execution")
    results = seal.get("results")
    require(isinstance(execution, dict) and isinstance(results, dict),
            "deferred Nextest execution and results are required")
    require(execution.get("build_completed") is True,
            "deferred Nextest build did not complete")
    require(execution.get("exit_code") == 100
            and execution.get("exit_classification")
            == "real assertion failure after test execution",
            "deferred Nextest run is not the reviewed assertion-failure execution")
    require(isinstance(case_results, list) and isinstance(case_map, list),
            "deferred Nextest case results and map must be arrays")
    result_events: dict[str, str] = {}
    for index, item in enumerate(case_results):
        require(isinstance(item, dict) and item.get("type") == "test"
                and item.get("event") in {"ok", "failed"},
                "deferred Nextest case result {} is not terminal".format(index))
        name = item.get("name")
        require(isinstance(name, str) and "$" in name,
                "deferred Nextest case result {} has no registry test name".format(index))
        registry_test = name.rsplit("$", 1)[1]
        require(registry_test not in result_events,
                "deferred Nextest contains a duplicate terminal test name")
        result_events[registry_test] = item["event"]
    mapped_tests: set[str] = set()
    failure_ids: list[str] = []
    failure_test_names: list[str] = []
    mapped_case_references = 0
    for index, item in enumerate(case_map):
        require(isinstance(item, dict),
                "deferred Nextest case map row {} must be an object".format(index))
        row_id = item.get("id")
        test_name = item.get("legacy_test")
        row_result = item.get("result")
        cases = item.get("cases")
        require(isinstance(row_id, str) and row_id.strip()
                and isinstance(test_name, str) and test_name.strip()
                and row_result in {"ok", "failed"}
                and isinstance(cases, list) and cases
                and all(isinstance(case, str) and case.strip() for case in cases),
                "deferred Nextest case map row {} is incomplete".format(index))
        require(test_name not in mapped_tests,
                "deferred Nextest case map contains a duplicate test name")
        mapped_tests.add(test_name)
        mapped_case_references += len(cases)
        require(result_events.get(test_name) == row_result,
                "deferred Nextest case map result disagrees with its terminal event")
        if row_result == "failed":
            failure_ids.append("{} / {} ({})".format(row_id, ", ".join(cases), test_name))
            failure_test_names.append(test_name)
    require(set(result_events) == mapped_tests,
            "deferred Nextest terminal events and case map do not match")
    passed = sum(event == "ok" for event in result_events.values())
    failed = sum(event == "failed" for event in result_events.values())
    total = len(result_events)
    require(total > 0 and passed + failed == total,
            "deferred Nextest terminal event counts are invalid")
    require(results.get("selected_filter_names") == total
            and results.get("test_started_events") == total
            and results.get("terminal_test_events") == total
            and results.get("passed") == passed
            and results.get("failed") == failed
            and results.get("registry_rows_mapped") == len(case_map)
            and results.get("registry_rows_missing") == 0
            and results.get("suite_event_counts_are_not_added_to_test_level_counts") is True,
            "deferred Nextest seal counts do not match the raw case files")
    require(source.get("registry_rows") == len(case_map)
            and source.get("registry_case_references") == mapped_case_references,
            "deferred Nextest source registry counts do not match the case map")
    require(Path(record["case_results"]["path"]).name
            == results.get("case_results_file")
            and Path(record["case_map"]["path"]).name
            == results.get("case_map_file"),
            "deferred Nextest seal file names do not match the pinned case files")

    require(isinstance(review, dict)
            and review.get("schema") == "termrock-independent-deferred-nextest-receipt-review-v1",
            "deferred Nextest review schema changed")
    review_receipt = review.get("receipt")
    findings = review.get("findings")
    review_events = findings.get("test_events") if isinstance(findings, dict) else None
    source_integrity = findings.get("source_integrity") if isinstance(findings, dict) else None
    require(isinstance(review_receipt, dict)
            and review_receipt.get("sha256") == record["seal"]["sha256"]
            and review_receipt.get("registry_sha256") == source["registry_sha256"]
            and review_receipt.get("registry_rows") == len(case_map),
            "deferred Nextest independent review is not bound to this seal")
    require(isinstance(review_events, dict)
            and review_events.get("started") == total
            and review_events.get("terminal") == total
            and review_events.get("ok") == passed
            and review_events.get("failed") == failed
            and review_events.get("case_map_result_vs_terminal_event_mismatches") == 0,
            "deferred Nextest independent review disagrees with raw test events")
    require(isinstance(source_integrity, dict)
            and source_integrity.get("commit_tree_registry_pins_match_receipt_and_source_hash_file") is True
            and source_integrity.get("execution_receipt_reports_inputs_rechecked_after_run") is True,
            "deferred Nextest independent review does not confirm source integrity")
    review_failure = findings.get("failure")
    require(isinstance(review_failure, dict)
            and len(failure_ids) == 1
            and review_failure.get("test") == failure_test_names[0],
            "deferred Nextest independent review does not match the failed row")
    timestamp(execution.get("start_utc"), "deferred Nextest start_utc")
    timestamp(execution.get("end_utc"), "deferred Nextest end_utc")
    return {
        "kind": "deferred_nextest",
        "run_root": seal.get("run_root"),
        "start_utc": execution.get("start_utc"),
        "end_utc": execution.get("end_utc"),
        "nextest_version": execution.get("nextest_version"),
        "passed": passed,
        "failed": failed,
        "total": total,
        "failure_ids": failure_ids,
        "seal": record["seal"],
        "review": record["review"],
    }


def load_holla_diagnostic(
    record: Any, candidate_sha: str, reference_sha: str, queue_module: Any
) -> Mapping[str, Any]:
    require(isinstance(record, dict), "holla_diagnostic observation is required")
    receipt = read_pinned_json(record.get("receipt"), "Holla paired receipt", queue_module)
    seal = read_pinned_json(record.get("seal"), "Holla journey execution seal", queue_module)
    review = read_pinned_json(record.get("review"), "Holla diagnostic review", queue_module)
    build_review = read_pinned_json(
        record.get("build_review"), "Holla paired build review", queue_module
    )
    require(isinstance(receipt, dict)
            and receipt.get("schema") == "termrock-spec/parity-run-receipt-v3",
            "Holla paired receipt schema changed")
    require(isinstance(receipt.get("run_id"), str) and receipt["run_id"].strip(),
            "Holla diagnostic run ID is required")
    suite = receipt.get("suite")
    require(isinstance(suite, dict) and suite.get("platform") == "macos"
            and suite.get("digest") == suite.get("compiled_digest"),
            "Holla diagnostic suite identity is invalid")
    pair_by_role: dict[str, str] = {}
    executable_hashes: dict[str, str] = {}
    builder_receipt_hashes: dict[str, str] = {}
    source_pair_rows = receipt.get("source_pair")
    require(isinstance(source_pair_rows, list), "Holla diagnostic source pair is required")
    for source in source_pair_rows:
        require(isinstance(source, dict)
                and source.get("role") in {"candidate", "reference"},
                "Holla diagnostic source pair role is invalid")
        role = source["role"]
        require(role not in pair_by_role,
                "Holla diagnostic contains a duplicate source role")
        pair_by_role[role] = sha(source.get("source_commit"),
                                 "Holla {} source commit".format(role))
        build = source.get("build")
        require(isinstance(build, dict)
                and build.get("target_triple") == "aarch64-apple-darwin",
                "Holla {} target triple is not the reviewed macOS target".format(role))
        executable = source.get("executable")
        builder_receipt = source.get("builder_receipt")
        require(isinstance(executable, dict) and isinstance(builder_receipt, dict),
                "Holla {} executable and builder receipt evidence are required".format(role))
        require(builder_receipt.get("source_commit") == pair_by_role[role],
                "Holla {} nested builder receipt source commit does not match the selected source".format(role))
        require(builder_receipt.get("target_triple") == build["target_triple"],
                "Holla {} nested builder receipt target triple does not match the build".format(role))
        expected_executable_sha = sha256(
            executable.get("expected_sha256"),
            "Holla {} expected executable SHA-256".format(role),
        )
        actual_executable_sha = sha256(
            executable.get("actual_sha256"),
            "Holla {} actual executable SHA-256".format(role),
        )
        nested_executable_sha = sha256(
            builder_receipt.get("executable_sha256"),
            "Holla {} builder receipt executable SHA-256".format(role),
        )
        require(expected_executable_sha == actual_executable_sha == nested_executable_sha,
                "Holla {} receipt executable SHA-256 values do not match".format(role))
        builder_receipt_sha = sha256(
            source.get("builder_receipt_sha256"),
            "Holla {} builder receipt SHA-256".format(role),
        )
        nested_builder_receipt_sha = sha256(
            builder_receipt.get("sha256"),
            "Holla {} nested builder receipt SHA-256".format(role),
        )
        require(builder_receipt_sha == nested_builder_receipt_sha,
                "Holla {} builder receipt SHA-256 values do not match".format(role))
        executable_hashes[role] = actual_executable_sha
        builder_receipt_hashes[role] = builder_receipt_sha
    require(pair_by_role == {"candidate": candidate_sha, "reference": reference_sha},
            "Holla diagnostic source pair does not match the selected pair")
    trust = receipt.get("trust")
    require(isinstance(trust, dict) and trust.get("status") == "accepted",
            "Holla diagnostic receipt trust status is not accepted")
    checks = receipt.get("checks")
    artifacts = receipt.get("artifacts")
    require(isinstance(checks, list) and isinstance(artifacts, list),
            "Holla diagnostic checks and artifacts must be arrays")
    status_counts: dict[str, int] = {}
    dimension_counts: dict[str, dict[str, int]] = {}
    role_dimension_counts: dict[str, dict[str, dict[str, int]]] = {
        "candidate": {}, "reference": {},
    }
    checkpoints: dict[str, set[str]] = {"candidate": set(), "reference": set()}
    unique_checks: set[tuple[str, str]] = set()
    actual_check_rows: set[tuple[str, str, str, str, str]] = set()
    expected_check_rows: set[tuple[str, str, str, str, str]] = set()
    for role in ("candidate", "reference"):
        for checkpoint in HOLLA_CHECKPOINTS:
            for dimension in (
                "build", "launch", "first_frame", "interaction", "visual",
                "exit", "restoration",
            ):
                expected_check_rows.add((
                    role,
                    "HELP-HOLLA-004",
                    checkpoint,
                    dimension,
                    "{}:HELP-HOLLA-004:{}:{}".format(role, checkpoint, dimension),
                ))
            for assertion in HOLLA_INTERACTION_ASSERTIONS[checkpoint]:
                expected_check_rows.add((
                    role,
                    "HELP-HOLLA-004",
                    checkpoint,
                    "interaction",
                    "HELP-HOLLA-004:{}:{}".format(checkpoint, assertion),
                ))
    for index, check in enumerate(checks):
        require(isinstance(check, dict), "Holla check {} must be an object".format(index))
        role = check.get("subject_role")
        check_id = check.get("id")
        dimension = check.get("dimension")
        status = check.get("status")
        case_id = check.get("case_id")
        checkpoint = check.get("checkpoint_id")
        require(role in checkpoints and isinstance(check_id, str) and check_id.strip()
                and case_id == "HELP-HOLLA-004"
                and isinstance(checkpoint, str) and checkpoint.strip()
                and dimension in {"build", "launch", "first_frame", "interaction",
                                  "visual", "exit", "restoration"}
                and status in {"PASS", "FAIL", "BLOCKED", "NOT_APPLICABLE", "NOT_RUN"},
                "Holla check {} has an unsupported role, case, dimension, or status".format(index))
        key = (role, check_id)
        require(key not in unique_checks,
                "Holla diagnostic has duplicate check IDs for a subject")
        unique_checks.add(key)
        actual_check_rows.add((role, case_id, checkpoint, dimension, check_id))
        checkpoints[role].add(checkpoint)
        status_counts[status] = status_counts.get(status, 0) + 1
        by_dimension = dimension_counts.setdefault(dimension, {})
        by_dimension[status] = by_dimension.get(status, 0) + 1
        role_dimension = role_dimension_counts[role].setdefault(dimension, {})
        role_dimension[status] = role_dimension.get(status, 0) + 1
    require(len(checks) == len(expected_check_rows)
            and actual_check_rows == expected_check_rows,
            "Holla check rows do not match the exact reviewed ID and dimension set")
    require(checkpoints["candidate"] == checkpoints["reference"]
            and checkpoints["candidate"] == set(HOLLA_CHECKPOINTS),
            "Holla diagnostic checkpoint sequence changed")
    environment = receipt.get("environment")
    common_environment = environment.get("common") if isinstance(environment, dict) else None
    require(isinstance(common_environment, dict)
            and common_environment.get("COLORTERM") == "truecolor",
            "Holla diagnostic is not bound to the reviewed truecolor environment")
    first_frame_rows = [check for check in checks if check["dimension"] == "first_frame"]
    require(len(first_frame_rows) == 8
            and all("120x40" in check.get("evidence", []) for check in first_frame_rows),
            "Holla first-frame evidence is not bound to the reviewed 120x40 viewport")
    expected_artifacts = {
        (role, "HELP-HOLLA-004", checkpoint, artifact_format)
        for role in ("candidate", "reference")
        for checkpoint in HOLLA_CHECKPOINTS
        for artifact_format in HOLLA_ARTIFACT_FORMATS
    }
    unique_artifacts: set[tuple[str, str, str, str]] = set()
    unique_artifact_paths: set[str] = set()
    artifact_count = 0
    for index, artifact in enumerate(artifacts):
        require(isinstance(artifact, dict)
                and artifact.get("subject_role") in checkpoints
                and artifact.get("case_id") == "HELP-HOLLA-004"
                and isinstance(artifact.get("checkpoint_id"), str)
                and isinstance(artifact.get("format"), str)
                and isinstance(artifact.get("path"), str)
                and artifact["path"].strip(),
                "Holla artifact {} is incomplete".format(index))
        artifact_key = (
            artifact["subject_role"], artifact["case_id"],
            artifact["checkpoint_id"], artifact["format"],
        )
        require(artifact_key not in unique_artifacts,
                "Holla diagnostic contains a duplicate artifact format row")
        unique_artifacts.add(artifact_key)
        require(artifact["path"] not in unique_artifact_paths,
                "Holla diagnostic contains a duplicate artifact path")
        unique_artifact_paths.add(artifact["path"])
        sha256(artifact.get("sha256"), "Holla artifact {} SHA-256".format(index))
        require(type(artifact.get("bytes")) is int and artifact["bytes"] > 0,
                "Holla artifact {} size is invalid".format(index))
        artifact_count += 1
    require(artifact_count == len(expected_artifacts)
            and unique_artifacts == expected_artifacts,
            "Holla artifact rows do not match the exact role, checkpoint, and format matrix")

    require(isinstance(seal, dict)
            and seal.get("record_type") == "termrock_holla_actual_paired_journey_seal_v1"
            and seal.get("state") == "sealed",
            "Holla journey execution seal schema or state changed")
    process = seal.get("process")
    selection = seal.get("selection")
    actual_receipt = seal.get("actual_receipt")
    require(isinstance(process, dict) and process.get("exit_code") == 100
            and process.get("timed_out") is False,
            "Holla diagnostic harness exit does not match the sealed run")
    require(isinstance(selection, dict) and selection.get("selected") == 1
            and selection.get("passed") == 0 and selection.get("failed") == 1
            and selection.get("skipped") == 1,
            "Holla diagnostic harness test counts changed")
    require(isinstance(actual_receipt, dict)
            and actual_receipt.get("sha256") == record["receipt"]["sha256"]
            and actual_receipt.get("run_id") == receipt["run_id"]
            and actual_receipt.get("check_count") == len(checks)
            and actual_receipt.get("status_counts") == status_counts,
            "Holla seal does not bind the paired receipt and its status counts")
    require(actual_receipt.get("qualification") in {"BLOCKED", "FAIL"},
            "Holla receipt qualification is neither BLOCKED nor FAIL")

    require(isinstance(review, dict)
            and review.get("record_type")
            == "independent_read_only_actual_holla_diagnostic_journey_review"
            and review.get("verdict") == "REVIEWED_DIAGNOSTIC_EVIDENCE_ONLY",
            "Holla diagnostic review verdict changed")
    reviewed_pair = review.get("source_pair_and_binaries")
    require(isinstance(reviewed_pair, dict)
            and all(isinstance(reviewed_pair.get(role), dict)
                    and reviewed_pair[role].get("source_commit") == pair_by_role[role]
                    for role in ("candidate", "reference")),
            "Holla diagnostic review source commits do not match the receipt")
    for role in ("candidate", "reference"):
        reviewed_subject = reviewed_pair[role]
        reviewed_executable_sha = sha256(
            reviewed_subject.get("executable_sha256_expected_and_rehashed"),
            "Holla diagnostic review {} executable SHA-256".format(role),
        )
        require(reviewed_executable_sha == executable_hashes[role],
                "Holla diagnostic review {} executable SHA-256 does not match the receipt".format(role))
        reviewed_builder_receipt_sha = sha256(
            reviewed_subject.get("builder_receipt_sha256"),
            "Holla diagnostic review {} builder receipt SHA-256".format(role),
        )
        require(reviewed_builder_receipt_sha == builder_receipt_hashes[role],
                "Holla diagnostic review {} builder receipt SHA-256 does not match the receipt".format(role))
    review_suite = review.get("suite_identity")
    require(isinstance(review_suite, dict)
            and review_suite.get("revision") == suite.get("revision")
            and review_suite.get("digest") == suite.get("digest"),
            "Holla diagnostic review suite identity does not match the receipt")
    subject = review.get("subject")
    review_execution = review.get("raw_execution")
    reviewed_results = review.get("checkpoint_and_assertion_results")
    artifact_audit = review.get("artifact_audit")
    require(isinstance(subject, dict)
            and subject.get("seal_sha256") == record["seal"]["sha256"]
            and subject.get("run_id") == Path(seal["executor"]["root"]).name
            and subject.get("nextest_run_id") == process.get("nextest_run_id"),
            "Holla diagnostic review is not bound to this sealed run")
    require(isinstance(review_execution, dict)
            and review_execution.get("exit_code") == process["exit_code"],
            "Holla diagnostic review disagrees with the sealed command exit")
    assertion_outcome = review_execution.get("assertion_outcome")
    execution_failure_statuses = {
        check["status"] for check in checks if check["status"] in {"FAIL", "BLOCKED"}
    }
    require(isinstance(assertion_outcome, str) and assertion_outcome.strip()
            and execution_failure_statuses
            and any(status in assertion_outcome.upper()
                    for status in execution_failure_statuses),
            "Holla harness failure cause does not match a recorded FAIL or BLOCKED result")
    reviewed_pass_dimensions = {
        "build_hash_checks": dimension_counts.get("build", {}).get("PASS", 0),
        "launch_checks": dimension_counts.get("launch", {}).get("PASS", 0),
        "first_frame": dimension_counts.get("first_frame", {}).get("PASS", 0),
        "interaction": dimension_counts.get("interaction", {}).get("PASS", 0),
    }
    reviewed_pass_dimensions = {
        name: count for name, count in reviewed_pass_dimensions.items() if count
    }
    require(isinstance(reviewed_results, dict)
            and reviewed_results.get("case_id") == "HELP-HOLLA-004"
            and reviewed_results.get("checks_total") == len(checks)
            and reviewed_results.get("status_counts") == status_counts
            and reviewed_results.get("pass_dimensions") == reviewed_pass_dimensions
            and reviewed_results.get("blocked_dimensions")
            == {dimension: counts["BLOCKED"] for dimension, counts in dimension_counts.items()
                if "BLOCKED" in counts}
            and reviewed_results.get("not_applicable_dimensions")
            == {dimension: counts["NOT_APPLICABLE"] for dimension, counts in dimension_counts.items()
                if "NOT_APPLICABLE" in counts},
            "Holla independent review disagrees with receipt checks")
    failed_dimensions = {
        dimension: counts["FAIL"] for dimension, counts in dimension_counts.items()
        if "FAIL" in counts
    }
    not_run_dimensions = {
        dimension: counts["NOT_RUN"] for dimension, counts in dimension_counts.items()
        if "NOT_RUN" in counts
    }
    if failed_dimensions:
        require(reviewed_results.get("failed_dimensions") == failed_dimensions,
                "Holla independent review does not bind the recorded FAIL dimensions")
    if not_run_dimensions:
        require(reviewed_results.get("not_run_dimensions") == not_run_dimensions,
                "Holla independent review does not bind the recorded NOT_RUN dimensions")
    interaction_breakdown = reviewed_results.get("interaction_breakdown")
    require(isinstance(interaction_breakdown, str)
            and "12 assertion IDs" in interaction_breakdown
            and "four checkpoints" in interaction_breakdown,
            "Holla interaction review does not preserve its assertion denominator")
    holla_interactions_per_role = {
        role: sum(role_dimension_counts[role].get("interaction", {}).values())
        for role in ("candidate", "reference")
    }
    require(holla_interactions_per_role == {"candidate": 16, "reference": 16},
            "Holla interaction check rows do not match the reviewed case scope")
    require(isinstance(artifact_audit, dict)
            and canonical_evidence_path(
                artifact_audit.get("actual_receipt_path"),
                "Holla artifact review receipt path",
            ) == canonical_evidence_path(
                record["receipt"]["path"], "Holla paired receipt path",
            )
            and artifact_audit.get("actual_receipt_sha256") == record["receipt"]["sha256"]
            and artifact_audit.get("artifact_observations") == artifact_count
            and artifact_audit.get("role_checkpoint_pairs") == 8
            and artifact_audit.get("unique_formats_per_pair") == 10
            and artifact_audit.get("formats") == list(HOLLA_ARTIFACT_FORMATS)
            and isinstance(artifact_audit.get("hash_and_size_verification"), str)
            and "all match" in artifact_audit["hash_and_size_verification"].lower(),
            "Holla artifact review does not bind this receipt")

    require(isinstance(build_review, dict)
            and build_review.get("record_type")
            == "independent_read_only_actual_holla_build_evidence_review"
            and build_review.get("verdict") == "REVIEWED_DIAGNOSTIC_BUILD_EVIDENCE_ONLY",
            "Holla paired build review changed")
    observations = build_review.get("observations")
    build_attempt = observations.get("attempt") if isinstance(observations, dict) else None
    build_nextest = observations.get("nextest") if isinstance(observations, dict) else None
    builds = observations.get("builds") if isinstance(observations, dict) else None
    build_roles: dict[str, Mapping[str, Any]] = {}
    require(isinstance(build_attempt, dict)
            and build_attempt.get("run_id") == receipt["run_id"]
            and build_attempt.get("process_exit_code") == 0
            and build_attempt.get("timed_out") is False
            and isinstance(build_nextest, dict)
            and build_nextest.get("passed") == 1
            and build_nextest.get("failed") == 0,
            "Holla paired build review does not record the successful build gate")
    require(isinstance(builds, list), "Holla paired build review has no build rows")
    for build in builds:
        require(isinstance(build, dict) and build.get("role") in {"candidate", "reference"},
                "Holla paired build review role is invalid")
        role = build["role"]
        require(role not in build_roles
                and build.get("source_commit") == pair_by_role[role]
                and build.get("target_triple") == "aarch64-apple-darwin"
                and build.get("exit_code") == 0,
                "Holla {} build review does not match the selected source and target".format(role))
        build_executable_sha = sha256(
            build.get("executable_sha256"),
            "Holla {} build review executable SHA-256".format(role),
        )
        require(build_executable_sha == executable_hashes[role],
                "Holla {} build review executable SHA-256 does not match the receipt".format(role))
        build_receipt_sha = sha256(
            build.get("builder_receipt_sha256"),
            "Holla {} build review builder receipt SHA-256".format(role),
        )
        require(build_receipt_sha == builder_receipt_hashes[role],
                "Holla {} build review builder receipt SHA-256 does not match the receipt".format(role))
        build_roles[role] = build
    require(set(build_roles) == {"candidate", "reference"},
            "Holla build review is missing a source role")

    pass_dimensions = reviewed_pass_dimensions
    blocked_dimensions = {
        dimension: counts["BLOCKED"] for dimension, counts in dimension_counts.items()
        if "BLOCKED" in counts
    }
    not_applicable_dimensions = {
        dimension: counts["NOT_APPLICABLE"] for dimension, counts in dimension_counts.items()
        if "NOT_APPLICABLE" in counts
    }
    return {
        "kind": "holla_diagnostic",
        "run_id": receipt["run_id"],
        "case_id": "HELP-HOLLA-004",
        "suite_revision": suite.get("revision"),
        "suite_digest": suite.get("digest"),
        "checks": len(checks),
        "artifact_count": artifact_count,
        "status_counts": status_counts,
        "pass_dimensions": pass_dimensions,
        "blocked_dimensions": blocked_dimensions,
        "not_applicable_dimensions": not_applicable_dimensions,
        "failed_dimensions": failed_dimensions,
        "not_run_dimensions": not_run_dimensions,
        "role_dimension_counts": role_dimension_counts,
        "candidate_assertions": sum(map(len, HOLLA_INTERACTION_ASSERTIONS.values())),
        "reference_assertions": sum(map(len, HOLLA_INTERACTION_ASSERTIONS.values())),
        "aggregate_interaction_checks_per_subject": len(HOLLA_CHECKPOINTS),
        "harness_selection": selection,
        "harness_assertion_outcome": assertion_outcome,
        "viewport": "120x40",
        "color_mode": "truecolor",
        "checkpoints_per_subject": len(checkpoints["candidate"]),
        "receipt": record["receipt"],
        "seal": record["seal"],
        "review": record["review"],
        "build_review": record["build_review"],
    }


def validate_execution_observations(
    value: Any, latest: Mapping[str, Any], queue_module: Any
) -> Sequence[Mapping[str, Any]]:
    if value is None:
        return []
    require(isinstance(value, dict)
            and value.get("schema") == "termrock-status-execution-observations-v1"
            and value.get("provisional_local_evidence") is True,
            "unsupported or nonlocal execution-observation record")
    candidate_sha = latest["candidate_remote"]["head_sha"]
    reference_sha = latest["reference_remote"]["head_sha"]
    source_pair_matches(value.get("source_pair"), candidate_sha, reference_sha,
                        "execution observations")
    deferred = load_deferred_execution(
        value.get("deferred_nextest"), candidate_sha, reference_sha, queue_module
    )
    holla = load_holla_diagnostic(
        value.get("holla_diagnostic"), candidate_sha, reference_sha, queue_module
    )
    return (deferred, holla)


def validate_historical_execution_observations(
    value: Any, history: Sequence[Mapping[str, Any]], queue_module: Any
) -> Sequence[Mapping[str, Any]]:
    if value is None:
        return []
    require(isinstance(value, list),
            "historical_execution_observations must be an array or null")
    validated = []
    for index, record in enumerate(value):
        label = "historical_execution_observations[{}]".format(index)
        require(isinstance(record, dict), "{} must be an object".format(label))
        source_pair = record.get("source_pair")
        require(isinstance(source_pair, dict), "{}.source_pair is required".format(label))
        candidate_sha = sha(source_pair.get("candidate_commit"),
                            "{}.source_pair.candidate_commit".format(label))
        reference_sha = sha(source_pair.get("reference_commit"),
                            "{}.source_pair.reference_commit".format(label))
        matches = [item for item in history
                   if item["source_observation"]["candidate_remote"]["head_sha"]
                   == candidate_sha
                   and item["source_observation"]["reference_remote"]["head_sha"]
                   == reference_sha]
        require(len(matches) == 1,
                "{} source pair must match exactly one superseded source observation"
                .format(label))
        source_record = matches[0]
        source = source_record["source_observation"]
        require(record.get("source_observed_at") == source["observed_at"]
                and record.get("superseded_at") == source_record["superseded_at"],
                "{} observation and supersession times do not match source history"
                .format(label))
        source_proxy = {
            "candidate_remote": {"head_sha": candidate_sha},
            "reference_remote": {"head_sha": reference_sha},
        }
        observations = validate_execution_observations(
            record.get("observations"), source_proxy, queue_module
        )
        validated.append({
            "source_pair": {"candidate_commit": candidate_sha,
                             "reference_commit": reference_sha},
            "source_observed_at": source["observed_at"],
            "superseded_at": source_record["superseded_at"],
            "observations": observations,
        })
    return validated


def validate_local_checkout_history(value: Any) -> Sequence[Mapping[str, Any]]:
    if value is None:
        return []
    require(isinstance(value, list), "local_checkout_history must be an array or null")
    validated = []
    seen = set()
    for index, record in enumerate(value):
        label = "local_checkout_history[{}]".format(index)
        require(isinstance(record, dict), "{} must be an object".format(label))
        checkout = record.get("checkout")
        dco = record.get("dco_trailer_observation")
        require(isinstance(checkout, dict) and isinstance(dco, dict),
                "{} checkout and DCO observations are required".format(label))
        timestamp(checkout.get("observed_at"), "{}.checkout.observed_at".format(label))
        superseded_at = timestamp(record.get("superseded_at"),
                                  "{}.superseded_at".format(label))
        require(checkout["observed_at"] < superseded_at,
                "{} supersession must follow its checkout observation".format(label))
        checkout_sha = sha(checkout.get("head_sha"), "{}.checkout.head_sha".format(label))
        require(checkout_sha not in seen, "local checkout history has a duplicate commit")
        seen.add(checkout_sha)
        sha(checkout.get("tree_sha"), "{}.checkout.tree_sha".format(label))
        require(checkout.get("branch") == "termrock-implementation"
                and checkout.get("signature_command") ==
                "git show -s --format='%G?' HEAD"
                and checkout.get("publication_status") in {
                    "unpublished_pending_correction", "unpublished", "published", "not_recorded",
                }
                and isinstance(checkout.get("qualification"), str)
                and checkout["qualification"].strip(),
                "{} checkout metadata is invalid".format(label))
        require(sha(dco.get("commit_sha"), "{}.DCO commit".format(label)) == checkout_sha
                and dco.get("expected_trailer")
                == "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
                and type(dco.get("trailer_present")) is bool
                and isinstance(dco.get("parsed_trailers"), list)
                and all(isinstance(item, str) for item in dco["parsed_trailers"])
                and dco.get("trailer_present") == any(
                    item == dco["expected_trailer"] for item in dco["parsed_trailers"]
                ),
                "{} DCO trailer observation does not match its checkout".format(label))
        validated.append({
            "checkout": checkout,
            "dco_trailer_observation": dco,
            "superseded_at": superseded_at,
        })
    return validated


LOCAL_E2E_COMMIT = "f61abd3dfba1b4f867a539ab18ed7e4760b24a18"
LOCAL_E2E_TREE = "15b0c9c2b24b773844a99c84b3839b4e98ef101f"
LOCAL_E2E_SUITE = "f307376a1e1d3dca04642875fdf98e9b43cdee47e6988ed0952c30c62c447361"
HISTORICAL_HOLLA_SUITE = "a914f8e34280f55d6a868bfee8777f771fb0354e96e5a2291a91b86c65a62ea0"


def load_local_e2e_suite_observation(
    value: Any, local: Mapping[str, Any], queue_module: Any,
    local_history: Sequence[Mapping[str, Any]] = (),
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict), "local E2E suite observation must be an object")
    timestamp(value.get("observed_at"), "local_e2e_suite_observation.observed_at")
    source_commit = sha(value.get("source_commit"), "local E2E source commit")
    source_tree = sha(value.get("source_tree_sha"), "local E2E source tree")
    if source_commit == local["head_sha"]:
        source_checkout = local
        is_historical = False
    else:
        matches = [record.get("checkout") for record in local_history
                   if isinstance(record, dict)
                   and isinstance(record.get("checkout"), dict)
                   and record["checkout"].get("head_sha") == source_commit
                   and record["checkout"].get("tree_sha") == source_tree]
        require(len(matches) == 1,
                "local E2E source commit does not match current or historical local checkout")
        source_checkout = matches[0]
        is_historical = True
    local_tree = sha(source_checkout.get("tree_sha"), "local E2E checkout tree SHA")
    require(source_commit == source_checkout["head_sha"] and source_tree == local_tree,
            "local E2E source identity does not match its checkout observation")
    require(source_commit == LOCAL_E2E_COMMIT and local_tree == LOCAL_E2E_TREE,
            "local E2E checkout identity is not the reviewed f61 source")
    require(value.get("revision") == "termrock-e2e-2026-10-08.1",
            "local E2E suite revision changed")
    require(type(value.get("file_count")) is int and value["file_count"] == 21,
            "local E2E suite file count changed")
    require(sha256(value.get("package_sha256"), "local E2E package SHA-256")
            == LOCAL_E2E_SUITE, "local E2E package digest changed")

    technical = read_pinned_json(
        value.get("technical_review"), "f307 package technical review", queue_module
    )
    require(isinstance(technical, dict)
            and technical.get("schema") == "termrock-vis02-proposed-package-technical-addendum-v1"
            and technical.get("verdict") == "TECHNICAL_READY_FOR_EXACT_SCOPED_PACKAGE_TREE",
            "f307 package technical review schema or verdict changed")
    repository = technical.get("repository")
    proposed_commit = technical.get("proposed_commit")
    require(isinstance(repository, dict)
            and sha(repository.get("proposed_tree_sha"), "f307 proposed source tree") == local_tree
            and isinstance(proposed_commit, dict)
            and sha256(proposed_commit.get("suite_sha256"), "f307 reviewed package SHA-256")
            == LOCAL_E2E_SUITE,
            "f307 package review does not bind the local tree and suite")

    gate = value.get("contract_gate")
    require(isinstance(gate, dict) and gate.get("result") == "PASS",
            "f307 non-product contract gate result is not PASS")
    receipt = read_pinned_json(
        gate.get("receipt"), "f307 contract gate receipt", queue_module
    )
    review = read_pinned_json(
        gate.get("independent_review"), "f307 contract gate review", queue_module
    )
    require(isinstance(receipt, dict)
            and receipt.get("schema") == "termrock-e2e/affected-holla-f307-rust-gate-result-v1"
            and receipt.get("status") == "PASS"
            and sha256(receipt.get("suite_sha256"), "f307 receipt suite SHA-256")
            == LOCAL_E2E_SUITE,
            "f307 contract gate receipt does not bind the reviewed suite")
    expected_stages = ["list-holla", "list-shared", "contract"]
    require(receipt.get("expected_stages") == expected_stages
            and receipt.get("completed_stages") == expected_stages,
            "f307 contract gate stages are incomplete")
    test_results = receipt.get("test_results")
    require(isinstance(test_results, list) and len(test_results) == 3,
            "f307 contract gate must contain the three reviewed stages")
    require(
        all(isinstance(item, dict)
            and item.get("stage") == stage
            and item.get("action") == action
            and item.get("status") == "PASS"
            for item, stage, action in zip(
                test_results, expected_stages, ("list", "list", "run")
            )),
        "f307 gate must contain two successful list stages and one successful run stage",
    )
    contract = test_results[2]
    counts = contract.get("counts") if isinstance(contract, dict) else None
    require(isinstance(contract, dict)
            and contract.get("stage") == "contract"
            and contract.get("action") == "run"
            and contract.get("status") == "PASS"
            and type(contract.get("exit_code")) is int
            and contract.get("exit_code") == 0
            and contract.get("source_unchanged") is True
            and contract.get("timed_out") is False
            and contract.get("expected_test")
            == "registry_contains_four_seed_cases_and_holla_checkpointed_preconditions"
            and contract.get("exact_filter")
            == "test(=registry_contains_four_seed_cases_and_holla_checkpointed_preconditions)"
            and isinstance(counts, dict)
            and all(type(counts.get(key)) is int for key in
                    ("selected", "executed", "passed", "failed", "skipped"))
            and counts.get("selected") == 1
            and counts.get("executed") == 1
            and counts.get("passed") == 1
            and counts.get("failed") == 0
            and counts.get("skipped") == 0,
            "f307 contract stage is not the reviewed single non-product test")
    require(isinstance(review, dict)
            and review.get("schema") == "termrock-e2e/affected-holla-f307-rust-gate-actual-review-v1"
            and review.get("verdict") == "READY_FOR_LOCAL_6_PATH_RECONCILIATION"
            and review.get("reviewed_at_utc") == value["observed_at"],
            "f307 contract gate independent review schema or verdict changed")
    actual = review.get("actual_run")
    selection = review.get("selection_and_results")
    source_closure = review.get("source_and_tool_closure")
    review_counts = selection.get("contract_counts") if isinstance(selection, dict) else None
    require(isinstance(actual, dict)
            and actual.get("receipt_sha256") == gate["receipt"]["sha256"]
            and actual.get("status") == "PASS"
            and actual.get("completed_all_stages") is True
            and actual.get("stages") == expected_stages
            and type(actual.get("timeouts")) is int
            and actual.get("timeouts") == 0
            and actual.get("source_suite_sha256") == LOCAL_E2E_SUITE
            and isinstance(review_counts, dict)
            and all(type(review_counts.get(key)) is int for key in
                    ("selected", "executed", "passed", "failed", "skipped"))
            and review_counts.get("selected") == 1
            and review_counts.get("executed") == 1
            and review_counts.get("passed") == 1
            and review_counts.get("failed") == 0
            and review_counts.get("skipped") == 0
            and isinstance(source_closure, dict)
            and type(source_closure.get("source_file_count")) is int
            and source_closure.get("source_file_count") == 21
            and source_closure.get("source_suite_sha256") == LOCAL_E2E_SUITE
            and source_closure.get("independently_recomputed_current_source_matches_freeze_and_receipt") is True
            and source_closure.get("all_three_stage_before_after_source_member_snapshots_match") is True
            and source_closure.get("compiled_output_suite_sha256") == LOCAL_E2E_SUITE
            and source_closure.get("compiled_rlib_rechecked_and_embeds_f307_digest") is True,
            "f307 independent review does not match the pinned one-test receipt")
    return {
        "observed_at": value["observed_at"],
        "source_commit": value["source_commit"],
        "source_tree_sha": value["source_tree_sha"],
        "is_historical": is_historical,
        "revision": value["revision"],
        "file_count": value["file_count"],
        "package_sha256": value["package_sha256"],
        "technical_review": value["technical_review"],
        "contract_gate": gate,
        "contract_receipt": receipt,
        "contract_review": review,
    }


def load_status_report_control_observation(
    value: Any, queue_module: Any
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict), "status report control observation must be an object")
    timestamp(value.get("observed_at"), "status_report_control_observation.observed_at")
    require(value.get("verdict") == "VERIFIED"
            and type(value.get("unique_test_identities")) is int
            and value["unique_test_identities"] == 35
            and value.get("composition")
            == "26 initial passing identities plus successive exact failed-test reruns of 7, 1, and 1 identities across reviewed test-source revisions; not one full green gate."
            and value.get("scope") == "Status generator/report controls only, not product readiness.",
            "status report control summary is not the reviewed cumulative result")
    review = read_pinned_json(
        value.get("independent_review"), "cumulative status-control review", queue_module
    )
    require(isinstance(review, dict)
            and review.get("schema") == "termrock-independent-status-r9-actual-review/v1"
            and review.get("verdict") == "VERIFIED",
            "cumulative status-control review schema or verdict changed")
    subject = review.get("r9_subject")
    require(isinstance(subject, dict)
            and subject.get("status_test_sha256")
            == "8111c3a2c9472627765843d88b687dacf281710fde35e294a7501c0a98bba2f7"
            and subject.get("production_status_py_sha256")
            == "b22e9cee9ea654cf65b55979bbed91afad1712dc265aec61e59d9b86f52f103a"
            and subject.get("source_facts_sha256")
            == "1253fb97e5f6ccdfabab8c9753082794af90bc6d9b12bfecb14baa1bba79c069",
            "cumulative review R9 subject changed")
    run = review.get("r9_run")
    require(isinstance(run, dict)
            and type(run.get("nextest_exit")) is int
            and run.get("nextest_exit") == 0
            and type(run.get("supervisor_exit")) is int
            and run.get("supervisor_exit") == 0
            and run.get("timed_out") is False
            and run.get("capture_errors") == []
            and run.get("nextest_result")
            == "1 test run: 1 passed, 34 skipped; exact selected case holla_valid_failure_and_not_run_rows_are_rendered_without_becoming_passes.",
            "cumulative review R9 run outcome changed")
    reconciliation = review.get("historical_test_identity_reconciliation")
    require(isinstance(reconciliation, dict),
            "cumulative status-control revision reconciliation is missing")
    expected_runs = {
        "r4": (
            "babcdda1d3ff6e92fbe9fa7829beb17a0fab726d3f15403159f89ee98404b36c",
            "556383b6512f296d3ec937045be2e69cf6dda2b7c9e15242b39ff907eeb5040a",
            "35 tests: 26 passed, 9 failed, 0 skipped.",
        ),
        "r6": (
            "0bb7f92b7d922b492ab92d8fddcfdf7620b276b0cdcd3303581be0c79dfec1cd",
            "b4ca700d29778d3aa51ed7228ae95c2b66f3e77b50d5b37bffdffa27006b9467",
            "Exact rerun of the nine R4 failed identities: 7 passed, 2 failed.",
        ),
        "r7": (
            "bab162c93c4cd41847bfc42adae327253707430bbec66371702de57606153b9a",
            "a9dac9a078e31099f85e0e1ea8d1ec4c5c2d40098402765701c75070cf78f544",
            "Exact rerun of the two R6 failed identities: 1 passed, 1 failed.",
        ),
        "r9": (
            "8111c3a2c9472627765843d88b687dacf281710fde35e294a7501c0a98bba2f7",
            "d3f9b7b286164dedef29c1f3cd64ffdaebeb9758642586066f1f46f07e4b6e5b",
            "Exact rerun of the remaining R7 failed identity: 1 passed.",
        ),
    }
    for name, (test_sha, stderr_sha, result) in expected_runs.items():
        item = reconciliation.get(name)
        require(isinstance(item, dict)
                and item.get("status_test_sha256") == test_sha
                and item.get("nextest_stderr_sha256") == stderr_sha
                and item.get("result") == result,
                "cumulative status-control {} run binding changed".format(name.upper()))
    require(reconciliation.get("reconciliation")
            == "The 26 R4-passing identities plus 7, 1, and 1 identities resolved by successive exact failed-test reruns total 35 unique test names. These outcomes span reviewed test-source revisions; they are not one complete green gate. All earlier failed raw receipts remain preserved.",
            "cumulative status-control identity reconciliation changed")
    return {
        "observed_at": value["observed_at"],
        "unique_test_identities": value["unique_test_identities"],
        "composition": value["composition"],
        "scope": value["scope"],
        "independent_review": value["independent_review"],
        "review": review,
    }


def validate_current_local_e2e_suite_observation(
    value: Any, local: Mapping[str, Any]
) -> Optional[Mapping[str, Any]]:
    """Validate a package-only current digest/control without product claims."""
    if value is None:
        return None
    require(isinstance(value, dict), "current local E2E suite observation is invalid")
    timestamp(value.get("observed_at"), "current E2E suite observation time")
    source_commit = sha(value.get("source_commit"), "current E2E source commit")
    source_tree = sha(value.get("source_tree_sha"), "current E2E source tree")
    require(source_commit == local["head_sha"] and source_tree == local["tree_sha"],
            "current E2E package source does not match the latest local checkout")
    require(value.get("scope") == "crates/termrock-e2e package files only",
            "current E2E package scope changed")
    require(value.get("revision") == "termrock-e2e-2026-10-09.2",
            "current E2E package revision changed")
    digest = sha256(value.get("package_sha256"), "current E2E package digest")
    files = value.get("files")
    require(isinstance(files, list) and bool(files)
            and len(files) == value.get("file_count"),
            "current E2E package file inventory is required")
    inventory_identity = hashlib.sha256()
    previous_path = ""
    for index, item in enumerate(files):
        label = "current E2E package files[{}]".format(index)
        require(isinstance(item, dict), "{} must be an object".format(label))
        path = item.get("path")
        require(isinstance(path, str) and path and not path.startswith("/")
                and path > previous_path and ".." not in Path(path).parts,
                "{} path must be safe, unique, and sorted".format(label))
        previous_path = path
        byte_count = item.get("bytes")
        require(type(byte_count) is int and byte_count >= 0,
                "{} byte count is invalid".format(label))
        file_digest = sha256(item.get("sha256"), "{}.sha256".format(label))
        item_identity = {
            "path": path, "bytes": byte_count, "sha256": file_digest,
        }
        encoded = json.dumps(item_identity, sort_keys=True,
                             separators=(",", ":"), ensure_ascii=False).encode("utf-8")
        inventory_identity.update(encoded)
        inventory_identity.update(b"\n")
    expected_inventory = sha256(value.get("inventory_sha256"),
                                "current E2E inventory identity")
    require(inventory_identity.hexdigest() == expected_inventory,
            "current E2E package file inventory identity changed")
    nextest = value.get("nextest")
    require(isinstance(nextest, dict), "current E2E Nextest control is required")
    require(nextest.get("source_suite_sha256") == digest,
            "current E2E Nextest control source digest changed")
    command = nextest.get("command")
    require(isinstance(command, str) and "cargo nextest" in command
            and "cargo test" not in command,
            "current E2E control must use cargo nextest")
    require(nextest.get("result") in {"PASS", "FAIL"},
            "current E2E control result must be PASS or FAIL")
    counts = {field: nextest.get(field) for field in (
        "selected", "executed", "passed", "failed", "skipped",
    )}
    require(all(type(count) is int and count >= 0 for count in counts.values()),
            "current E2E control counts must be nonnegative integers")
    require(counts["executed"] == counts["passed"] + counts["failed"]
            + counts["skipped"],
            "current E2E control counts do not reconcile")
    require(counts["selected"] == counts["executed"],
            "current E2E control selection does not reconcile")
    exit_code = nextest.get("exit_code")
    require(type(exit_code) is int, "current E2E control exit code is required")
    if nextest["result"] == "PASS":
        require(exit_code == 0 and counts["failed"] == 0 and counts["passed"] > 0,
                "current E2E PASS does not match its Nextest result")
    else:
        require(exit_code != 0 and counts["failed"] > 0,
                "current E2E FAIL does not match its Nextest result")
    require(isinstance(value.get("qualification"), str)
            and "not a paired run" in value["qualification"].lower(),
            "current E2E package control must deny paired-run qualification")
    return value


def validate_historical_current_status_observations(
    value: Any, latest: Mapping[str, Any], queue_module: Any,
) -> Sequence[Mapping[str, Any]]:
    if value is None:
        return []
    require(isinstance(value, dict)
            and value.get("schema")
            == "termrock-status-current-observations-history/v1",
            "unsupported historical current-status schema")
    records = value.get("records")
    require(isinstance(records, list) and bool(records),
            "historical current-status records are required")
    current_candidate = latest["candidate_remote"]["head_sha"]
    current_reference = latest["reference_remote"]["head_sha"]
    validated = []
    for index, record in enumerate(records):
        label = "historical_current_status_observations[{}]".format(index)
        require(isinstance(record, dict), "{} must be an object".format(label))
        superseded_at = timestamp(record.get("superseded_at"),
                                  "{}.superseded_at".format(label))
        observations = record.get("observations")
        require(isinstance(observations, dict)
                and observations.get("schema")
                == "termrock-status-current-observations-v1",
                "{} observations schema changed".format(label))
        pair = observations.get("fixed_comparison_pair")
        require(isinstance(pair, dict)
                and pair.get("candidate_commit") != current_candidate
                and pair.get("reference_commit") != current_reference,
                "{} must not use the current source pair".format(label))
        failure = observations.get("candidate_api_deferred_run")
        require(isinstance(failure, dict)
                and failure.get("measurement_status") == "FAILED"
                and failure.get("review_verdict") == "VERIFIED_FAILED_RUN"
                and failure.get("acceptance_decision") == "NOT_RECORDED",
                "{} must preserve its failed source-bound candidate run".format(label))
        receipt = read_pinned_json(failure.get("receipt"),
                                   "{} candidate receipt".format(label), queue_module)
        ledger = read_pinned_json(failure.get("ledger"),
                                  "{} candidate ledger".format(label), queue_module)
        review = read_pinned_json(failure.get("review"),
                                  "{} candidate review".format(label), queue_module)
        packet = validate_product_evidence_packet(
            failure.get("packet_manifest"), failure, queue_module,
        )
        require(isinstance(receipt, dict) and isinstance(ledger, dict)
                and isinstance(review, dict)
                and isinstance(packet, dict)
                and receipt.get("execution", {}).get("result")
                == failure["measurement_status"]
                and ledger.get("run", {}).get("receipt_sha256")
                == failure["receipt"]["sha256"]
                and review.get("verdict") == failure["review_verdict"],
                "{} candidate receipt, ledger, and review do not reconcile".format(label))
        readiness = observations.get("report_readiness")
        require(isinstance(readiness, dict) and all(
            readiness.get(field) == "NOT_RUN" for field in (
                "visibility_complete", "refactor_ready", "reference_qualified",
                "command_ready", "evidence_freshness",
            )
        ), "{} cannot promote readiness".format(label))
        validated.append({"superseded_at": superseded_at,
                          "observations": observations})
    return validated


def render_current_local_e2e_suite_observation(
    value: Optional[Mapping[str, Any]],
) -> str:
    if value is None:
        return ""
    run = value["nextest"]
    return "\n".join([
        "## Current local package control",
        "",
        "| Observation | Result | Scope |",
        "| --- | --- | --- |",
        "| Package `{}`; {} files | SHA-256 `{}` | Current source commit `{}`; revision `{}`; package files only. |".format(
            value["revision"], value["file_count"], value["package_sha256"],
            value["source_commit"], value["revision"],
        ),
        "| `cargo nextest` digest control | {} ({}/{} selected; {} passed, {} failed, {} skipped) | Run `{}`; compiled/current digest sensitivity only; NOT product execution and NOT a paired run. |".format(
            run["result"], run["executed"], run["selected"], run["passed"],
            run["failed"], run["skipped"], run["run_id"],
        ),
        "",
        value["qualification"],
        "",
    ])


def render_local_validation_observations(
    suite: Optional[Mapping[str, Any]],
    controls: Optional[Mapping[str, Any]],
    execution: Sequence[Mapping[str, Any]],
) -> str:
    if suite is None and controls is None:
        return ""
    require(suite is not None and controls is not None,
            "local suite and reporter-control observations must be recorded together")
    holla = next(
        (item for item in execution if item["kind"] == "holla_diagnostic"), None
    )
    require(isinstance(holla, dict)
            and holla.get("suite_digest") == HISTORICAL_HOLLA_SUITE
            and holla.get("suite_digest") != suite["package_sha256"],
            "Holla receipt suite identity is not the reviewed historical a914 package")
    return "\n".join([
        "## Local package and reporter controls",
        "",
        "| Observation | Result | Evidence |",
        "| --- | --- | --- |",
        "| {} E2E package | `{}`; tree `{}`; {} files; revision `{}`; package SHA-256 `{}` | Package review `{}` |".format(
            "Historical local" if suite["is_historical"] else "Local",
            suite["source_commit"], suite["source_tree_sha"], suite["file_count"],
            suite["revision"], suite["package_sha256"],
            suite["technical_review"]["sha256"],
        ),
        "| f307 contract control | 1 non-product registry/precondition test passed of 1 selected; no Holla journey ran | Receipt `{}`; independent review `{}` |".format(
            suite["contract_gate"]["receipt"]["sha256"],
            suite["contract_gate"]["independent_review"]["sha256"],
        ),
        "| Holla suite relation | Historical Holla receipt uses `{}`; it differs from local f307 `{}`. The f307 Holla product result remains NOT_RUN. | Holla receipt `{}` |".format(
            holla["suite_digest"], suite["package_sha256"],
            holla["receipt"]["sha256"],
        ),
        "| Status reporter controls | {} unique test identities reconciled as 26 + 7 + 1 + 1 across R4/R6/R7/R9 source revisions; this is not one full green run. | Cumulative review `{}` |".format(
            controls["unique_test_identities"],
            controls["independent_review"]["sha256"],
        ),
        "",
        "These package and reporter observations are local controls. A historical package remains bound to its recorded commit even when the local HEAD has advanced. They do not qualify product behavior or change Visibility / Complete, Refactor / Ready, or current paired product results.",
        "",
    ])


def summarize_status_counts(counts: Mapping[str, int]) -> str:
    return ", ".join(
        "{} {}".format(counts[status], status)
        for status in STATUS_ORDER
        if counts.get(status, 0)
    ) or "no recorded rows"


def summarize_dimension_statuses(counts: Mapping[str, int]) -> str:
    return "; ".join(
        "{} ({})".format(status, counts[status])
        for status in STATUS_ORDER
        if counts.get(status, 0)
    ) or "NOT_RUN"


def summarize_holla_dimensions(
    role_counts: Mapping[str, Mapping[str, Mapping[str, int]]]
) -> str:
    roles = []
    for role in ("candidate", "reference"):
        dimensions = role_counts[role]
        rendered = ", ".join(
            "{}: {}".format(
                dimension, summarize_dimension_statuses(dimensions.get(dimension, {}))
            )
            for dimension in (
                "build", "launch", "first_frame", "interaction", "visual", "exit",
                "restoration",
            )
        )
        roles.append("{} [{}]".format(role, rendered))
    return "; ".join(roles)


def summarize_holla_role(dimension_counts: Mapping[str, Mapping[str, int]]) -> str:
    return ", ".join(
        "{}: {}".format(
            dimension, summarize_dimension_statuses(dimension_counts.get(dimension, {}))
        )
        for dimension in (
            "build", "launch", "first_frame", "interaction", "visual", "exit",
            "restoration",
        )
    )


def summarize_holla_findings(
    dimension_counts: Mapping[str, Mapping[str, int]]
) -> str:
    findings = []
    for dimension in sorted(dimension_counts):
        for status in ("FAIL", "BLOCKED"):
            count = dimension_counts[dimension].get(status, 0)
            if count:
                findings.append("{}: {} ({})".format(dimension, status, count))
    return "; ".join(findings) or "no FAIL or BLOCKED rows"


def render_execution_observations(
    observations: Sequence[Mapping[str, Any]],
    *, historical: Optional[Mapping[str, Any]] = None,
) -> str:
    if not observations:
        return ""
    deferred = next(item for item in observations if item["kind"] == "deferred_nextest")
    holla = next(item for item in observations if item["kind"] == "holla_diagnostic")
    failed_rows = "; ".join(cell(item) for item in deferred["failure_ids"])
    status_counts = holla["status_counts"]
    status_summary = summarize_status_counts(status_counts)
    pass_dimensions = "; ".join(
        "{}: {}".format(cell(name), count)
        for name, count in sorted(holla["pass_dimensions"].items())
    )
    blocked_dimensions = "; ".join(
        "{}: {}".format(cell(name), count)
        for name, count in sorted(holla["blocked_dimensions"].items())
    ) or "none"
    not_applicable = "; ".join(
        "{}: {}".format(cell(name), count)
        for name, count in sorted(holla["not_applicable_dimensions"].items())
    ) or "none"
    role_summary = summarize_holla_dimensions(holla["role_dimension_counts"])
    finding_summary = "candidate: {}; reference: {}".format(
        summarize_holla_findings(holla["role_dimension_counts"]["candidate"]),
        summarize_holla_findings(holla["role_dimension_counts"]["reference"]),
    )
    holla_selection = holla["harness_selection"]
    if historical is None:
        heading = "## Bounded execution observations"
        scope_note = (
            "These local temporary records are bound to the selected source pair and reviewed as bounded evidence."
        )
    else:
        pair = historical["source_pair"]
        heading = "## Historical bounded execution observations"
        scope_note = (
            "These local temporary records are bound only to historical source pair `{}` / `{}`. "
            "That pair was observed at {} and superseded at {}. These results are not current for the selected source pair."
        ).format(pair["candidate_commit"], pair["reference_commit"],
                 historical["source_observed_at"], historical["superseded_at"])
    return "\n".join([
        heading,
        "",
        "{} They do not establish the complete required set, all applications, exact root commands, API, ownership, or readiness. Their raw evidence remains outside this repository, so this report is local-only until those artifacts are archived.".format(scope_note),
        "",
        "| Observation | Result | Evidence |",
        "| --- | --- | --- |",
        "| Deferred conformance subset | {} passed; {} failed of {} terminal tests; build completed; execution exited 100 after an assertion failure | Seal `{}`; review `{}` |".format(
            deferred["passed"], deferred["failed"], deferred["total"],
            deferred["seal"]["sha256"], deferred["review"]["sha256"],
        ),
        "| Holla {} diagnostic | {} checks: {} | Receipt `{}`; seal `{}`; review `{}` |".format(
            holla["case_id"], holla["checks"], status_summary,
            holla["receipt"]["sha256"], holla["seal"]["sha256"],
            holla["review"]["sha256"],
        ),
        "",
        "Per-role outcomes:",
        "",
        "| Role | Measured outcomes |",
        "| --- | --- |",
        "| Candidate | {} |".format(
            summarize_holla_role(holla["role_dimension_counts"]["candidate"]),
        ),
        "| Reference | {} |".format(
            summarize_holla_role(holla["role_dimension_counts"]["reference"]),
        ),
        "",
        "Aggregate status counts:",
        "",
        "| Scope | Count |",
        "| --- | --- |",
        "| Holla results | {} rows: {} |".format(holla["checks"], status_summary),
        "",
        "Aggregate dimension counts:",
        "",
        "| Status | Dimensions |",
        "| --- | --- |",
        "| PASS | {} |".format(pass_dimensions),
        "| BLOCKED | {} |".format(blocked_dimensions),
        "| NOT_APPLICABLE | {} |".format(not_applicable),
        "",
        "The deferred failure is {}.".format(failed_rows),
        "Holla recorded {} total result rows: {}.".format(holla["checks"], status_summary),
        "The harness selected {} test and exited 100: {} passed, {} failed, and {} skipped.".format(
            holla_selection["selected"], holla_selection["passed"],
            holla_selection["failed"], holla_selection["skipped"],
        ),
        "{}.".format(cell(holla["harness_assertion_outcome"]).rstrip(".")),
        "Recorded FAIL/BLOCKED rows: {}.".format(finding_summary),
        "The review records {} registry assertion IDs and {} aggregate interaction checks per subject across {} checkpoints.".format(
            holla["candidate_assertions"],
            holla["aggregate_interaction_checks_per_subject"],
            holla["checkpoints_per_subject"],
        ),
        "This covers one {} {} case only.".format(holla["viewport"], holla["color_mode"]),
        "The independent artifact review matched the hash and size of all {} expected artifact files.".format(
            holla["artifact_count"],
        ),
        "The candidate and reference release-build commands each exited with code 0.",
        "No API or ownership result is recorded.",
        "",
        "This partial case does not cover the 293-case, 421-checkpoint, or 7,550-snapshot/profile inventory. The proposed R5 denominator is not accepted as the active required set.",
        "",
    ])


def validate_source_observation(observation: Any, label: str) -> Mapping[str, Any]:
    require(isinstance(observation, dict), "{} is required".format(label))
    timestamp(observation.get("observed_at"), "{}.observed_at".format(label))
    method = observation.get("method")
    require(method in {
        "read-only GitHub branch API observation",
        "read-only GitHub branch API observation; local fetched refs verified afterward",
        "read-only exact-source comparison",
        "explicit HTTPS refs/heads fetch",
    }, "{} has an unknown observation method".format(label))
    candidate_remote = observation.get("candidate_remote")
    reference_remote = observation.get("reference_remote")
    require(isinstance(candidate_remote, dict) and isinstance(reference_remote, dict),
            "{} candidate and reference tips are required".format(label))
    require(candidate_remote.get("branch") == "termrock-implementation",
            "{} candidate branch changed".format(label))
    require(reference_remote.get("branch") == "visual-baseline",
            "{} reference branch changed".format(label))
    sha(candidate_remote.get("head_sha"), "{}.candidate_remote.head_sha".format(label))
    sha(reference_remote.get("head_sha"), "{}.reference_remote.head_sha".format(label))

    if method in {
        "read-only GitHub branch API observation",
        "read-only GitHub branch API observation; local fetched refs verified afterward",
    }:
        for role, remote in (("candidate", candidate_remote), ("reference", reference_remote)):
            if "updated_at" in remote:
                timestamp(remote["updated_at"], "{}.{}.updated_at".format(label, role))
                sources = observation.get("observation_sources")
                commands = sources.get("commands") if isinstance(sources, dict) else None
                require(isinstance(commands, list) and any(
                    isinstance(command, str)
                    and ".commit.commit.committer.date" in command
                    for command in commands
                ), "{} legacy updated_at must be documented as commit committer metadata"
                       .format(label))
            else:
                timestamp_with_offset(
                    remote.get("commit_committer_at"),
                    "{}.{}_remote.commit_committer_at".format(label, role),
                )
                provenance = observation.get("timestamp_provenance")
                require(isinstance(provenance, dict)
                        and provenance.get("committer_dates_are_branch_update_times") is False
                        and observation.get("branch_last_update_at") is None
                        and isinstance(provenance.get("branch_last_update_at_source"), str)
                        and provenance["branch_last_update_at_source"].strip(),
                        "{} API committer dates must remain separate from unknown branch update time"
                        .format(label))
                source_field = "{}_committer_date_source".format(role)
                require(isinstance(provenance.get(source_field), str)
                        and provenance[source_field].strip(),
                        "{} {} committer-date source is required".format(label, role))
        if method.endswith("local fetched refs verified afterward"):
            api = observation.get("api_observation")
            require(isinstance(api, dict),
                    "{} combined branch observation requires its API record".format(label))
            require(api.get("candidate_url") ==
                    "https://api.github.com/repos/{}/branches/termrock-implementation"
                    .format(REPOSITORY)
                    and api.get("reference_url") ==
                    "https://api.github.com/repos/{}/branches/visual-baseline"
                    .format(REPOSITORY),
                    "{} branch API URLs do not match the selected branches".format(label))
            require(api.get("raw_response_files_preserved") is False,
                    "{} branch API response preservation state changed".format(label))
            fetch = observation.get("fetch_provenance")
            require(isinstance(fetch, dict)
                    and fetch.get("evidence_kind") ==
                    "coordinator-reported fetch result with post-fetch local-ref verification"
                    and fetch.get("transport") == "HTTPS"
                    and fetch.get("operation") == "fetch"
                    and fetch.get("refspecs") == [
                        "refs/heads/termrock-implementation",
                        "refs/heads/visual-baseline",
                    ]
                    and type(fetch.get("session_id")) is int
                    and fetch["session_id"] > 0
                    and fetch.get("exit_code") == 0
                    and fetch.get("completed_at") is None
                    and fetch.get("raw_fetch_output_preserved") is False,
                    "{} post-observation fetch provenance is incomplete".format(label))
            query = fetch.get("post_fetch_ref_query")
            require(isinstance(query, dict),
                    "{} post-fetch ref query is required".format(label))
            timestamp(query.get("observed_at"),
                      "{}.fetch_provenance.post_fetch_ref_query.observed_at".format(label))
            require(query["observed_at"] >= observation["observed_at"],
                    "{} post-fetch query predates its API observation".format(label))
            require(sha(query.get("candidate_head_sha"),
                        "{}.post-fetch candidate SHA".format(label))
                    == candidate_remote["head_sha"]
                    and sha(query.get("reference_head_sha"),
                            "{}.post-fetch reference SHA".format(label))
                    == reference_remote["head_sha"],
                    "{} post-fetch refs do not match its branch API pair".format(label))
    elif method in {"read-only exact-source comparison", "explicit HTTPS refs/heads fetch"}:
        require("branch_last_update_at" in observation
                and observation["branch_last_update_at"] is None,
                "{} branch last-update time must be explicitly unknown".format(label))
        for role, remote in (("candidate", candidate_remote), ("reference", reference_remote)):
            if label == "latest_source_observation":
                # The current fixed-pair renderer reports commit dates as metadata.
                # Require and validate them here so malformed facts fail as a
                # schema error instead of raising KeyError during rendering.
                timestamp_with_offset(
                    remote.get("commit_committer_at"),
                    "{}.{}_remote.commit_committer_at".format(label, role),
                )
            elif "commit_committer_at" in remote:
                timestamp_with_offset(
                    remote["commit_committer_at"],
                    "{}.{}.commit_committer_at".format(label, role),
                )
        provenance = observation.get("timestamp_provenance")
        if method == "explicit HTTPS refs/heads fetch":
            require(isinstance(provenance, dict)
                    and provenance.get("committer_dates_are_branch_update_times") is False
                    and isinstance(provenance.get("branch_last_update_at_source"), str)
                    and provenance["branch_last_update_at_source"].strip(),
                    "{} must state commit-date and branch-update provenance".format(label))
            for role in ("candidate", "reference"):
                source_field = "{}_committer_date_source".format(role)
                require(isinstance(provenance.get(source_field), str)
                        and provenance[source_field].strip(),
                        "{} {} committer-date source is required".format(label, role))

            fetch = observation.get("fetch_provenance")
            require(isinstance(fetch, dict),
                    "{} fetch provenance is required".format(label))
            validate_source_evidence({
                "path": fetch.get("evidence_path"),
                "sha256": fetch.get("evidence_sha256"),
            }, "{}.fetch_provenance".format(label))
            if "command" in fetch:
                command = fetch.get("command")
                require(isinstance(command, str)
                        and "git fetch --no-tags https://github.com/"
                        "tailrocks/terminal-components-claude.git" in command
                        and "refs/heads/termrock-implementation" in command
                        and "refs/heads/visual-baseline" in command,
                        "{} legacy fetch command must identify both explicit refs".format(label))
            else:
                require(fetch.get("transport") == "HTTPS"
                        and fetch.get("operation") == "fetch",
                        "{} fetch transport and operation are required".format(label))
                refspecs = fetch.get("refspecs")
                require(isinstance(refspecs, list) and len(refspecs) == 2
                        and all(isinstance(item, str) for item in refspecs)
                        and set(refspecs) == {
                            "refs/heads/termrock-implementation",
                            "refs/heads/visual-baseline",
                        },
                        "{} fetch must identify the two explicit source refspecs".format(label))
                require(type(fetch.get("session_id")) is int
                        and fetch["session_id"] > 0,
                        "{} fetch session ID must be a positive integer".format(label))
                require(type(fetch.get("exit_code")) is int
                        and fetch["exit_code"] == 0,
                        "{} fetch must have a successful exit code".format(label))
                completed_at = timestamp(
                    fetch.get("completed_at"),
                    "{}.fetch_provenance.completed_at".format(label),
                )
                require(completed_at == observation["observed_at"],
                        "{} fetch completion must match the source observation time".format(label))
                require(type(fetch.get("raw_fetch_output_preserved")) is bool,
                        "{} raw fetch-output preservation state must be explicit".format(label))
                query = fetch.get("post_fetch_ref_query")
                require(isinstance(query, dict),
                        "{} post-fetch ref query evidence is required".format(label))
                require(isinstance(query.get("record_id"), str)
                        and query["record_id"].strip(),
                        "{} post-fetch ref query record ID is required".format(label))
                require("observed_at" in query,
                        "{} post-fetch ref query time must be present or null".format(label))
                if query["observed_at"] is not None:
                    timestamp(query["observed_at"],
                              "{}.fetch_provenance.post_fetch_ref_query.observed_at".format(label))
                query_candidate = sha(
                    query.get("candidate_head_sha"),
                    "{}.fetch_provenance.post_fetch_ref_query.candidate_head_sha".format(label),
                )
                query_reference = sha(
                    query.get("reference_head_sha"),
                    "{}.fetch_provenance.post_fetch_ref_query.reference_head_sha".format(label),
                )
                require(query_candidate == candidate_remote["head_sha"]
                        and query_reference == reference_remote["head_sha"],
                        "{} post-fetch ref query does not match the selected source pair".format(label))
    return observation


def validate_source_observation_history(
    facts: Mapping[str, Any], latest: Mapping[str, Any]
) -> None:
    has_history = "source_observation_history" in facts
    if latest["method"] != "explicit HTTPS refs/heads fetch" and not has_history:
        return
    history = facts.get("source_observation_history")
    require(isinstance(history, list),
            "source observation requires an ordered source-observation history array")
    sources = []
    evidence_identities = set()
    for index, record in enumerate(history):
        label = "source_observation_history[{}]".format(index)
        require(isinstance(record, dict), "{} must be an object".format(label))
        source = validate_source_observation(
            record.get("source_observation"), "{}.source_observation".format(label)
        )
        require("current_ci_observation" in record,
                "{}.current_ci_observation must be present; use null for an absent capture"
                .format(label))
        evidence = validate_source_evidence(record.get("source_evidence"),
                                            "{}.source_evidence".format(label))
        evidence_identity = (evidence["path"], evidence["sha256"])
        require(evidence_identity not in evidence_identities,
                "source observation history reuses a source-evidence identity")
        evidence_identities.add(evidence_identity)
        if source["method"] == "explicit HTTPS refs/heads fetch":
            fetch = source["fetch_provenance"]
            require(evidence["path"] == fetch["evidence_path"]
                    and evidence["sha256"] == fetch["evidence_sha256"],
                    "{}.source_evidence does not match its explicit-fetch provenance"
                    .format(label))
        ci_observation = record["current_ci_observation"]
        validated_ci = validate_current_ci_observation(source, ci_observation)
        timestamp(record.get("superseded_at"), "{}.superseded_at".format(label))
        sources.append(source)
        if validated_ci is not None:
            api_capture_at = validated_ci["api_capture"]["captured_at"]
            require(api_capture_at <= record["superseded_at"],
                    "{}.current_ci_observation was captured after supersession"
                    .format(label))
            provider = validated_ci.get("provider_annotation_capture")
            if provider is not None:
                require(provider["captured_at"] <= record["superseded_at"],
                        "{}.provider_annotation_capture was captured after supersession"
                        .format(label))

    successors = sources + [latest]
    source_pairs = set()
    for index, source in enumerate(successors):
        pair = (
            source["candidate_remote"]["head_sha"],
            source["reference_remote"]["head_sha"],
        )
        require(pair not in source_pairs,
                "source observation history contains a duplicate source pair")
        source_pairs.add(pair)
        if index == len(history):
            continue
        record = history[index]
        successor_at = successors[index + 1]["observed_at"]
        require(source["observed_at"] < successor_at,
                "source observations must be strictly ordered by observed_at")
        require(record["superseded_at"] == successor_at,
                "source_observation_history[{}].superseded_at must equal the next source observation time"
                .format(index))


def validate_current_ci_observation(
    latest: Mapping[str, Any], observation: Any
) -> Optional[Mapping[str, Any]]:
    if observation is None:
        return None
    require(isinstance(observation, dict), "current_ci_observation must be an object")
    api = observation.get("api_capture")
    require(isinstance(api, dict), "current CI API capture is required")
    timestamp(api.get("captured_at"), "current_ci_observation.api_capture.captured_at")
    require(isinstance(api.get("manifest_path"), str) and api["manifest_path"].strip(),
            "current CI API manifest path is required")
    sha256(api.get("manifest_sha256"), "current CI API manifest SHA-256")

    source_pair = api.get("source_pair")
    require(isinstance(source_pair, dict), "current CI source pair is required")
    candidate_sha = sha(source_pair.get("candidate_head_sha"),
                        "current CI candidate source SHA")
    reference_sha = sha(source_pair.get("reference_head_sha"),
                        "current CI reference source SHA")
    remote_candidate = latest["candidate_remote"]["head_sha"]
    remote_reference = latest["reference_remote"]["head_sha"]
    require(candidate_sha == remote_candidate,
            "current CI candidate source does not match the latest candidate tip")
    require(reference_sha == remote_reference,
            "current CI reference source does not match the latest reference tip")

    run = api.get("candidate_run")
    require(isinstance(run, dict), "current candidate workflow run is required")
    run_id = run.get("run_id")
    require(isinstance(run_id, str) and run_id.isdigit(),
            "current candidate run ID must be numeric text")
    run_sha = sha(run.get("head_sha"), "current candidate run head SHA")
    require(run_sha == candidate_sha,
            "current candidate run head does not match its source pair")
    require(run.get("workflow_path") == CURRENT_CI_WORKFLOW,
            "current candidate workflow path changed")
    require(run.get("status") in {"queued", "in_progress", "completed"},
            "unknown current candidate CI status")
    require(run.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown current candidate CI conclusion")
    timestamp(run.get("created_at"), "current candidate run creation time")
    query_count = run.get("head_sha_query_total_count")
    require(type(query_count) is int and query_count >= 1,
            "current candidate head-SHA query count must be positive")
    for field in ("job_count", "artifact_count"):
        require(type(run.get(field)) is int and run[field] >= 0,
                "current candidate CI {} must be a nonnegative integer".format(field))
    run_url = "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id)
    require(run.get("run_url") == run_url,
            "current candidate run URL does not match its ID")

    workflow = api.get("workflow_source")
    require(isinstance(workflow, dict), "current candidate workflow source is required")
    require(workflow.get("path") == run["workflow_path"],
            "current workflow source path does not match the run")
    require(sha(workflow.get("commit_sha"), "current workflow source commit SHA")
            == candidate_sha,
            "current workflow source commit does not match the candidate tip")
    sha(workflow.get("github_blob_sha"), "current workflow GitHub blob SHA")
    require(type(workflow.get("size_bytes")) is int and workflow["size_bytes"] >= 0,
            "current workflow source size must be a nonnegative integer")
    sha256(workflow.get("raw_sha256"), "current workflow source SHA-256")

    query = api.get("reference_query")
    if query is not None:
        require(isinstance(query, dict), "current reference query must be an object")
        query_sha = sha(query.get("head_sha"), "current reference query head SHA")
        require(query_sha == reference_sha,
                "current reference query head does not match its source pair")
        require(query.get("workflow_path") == CURRENT_CI_WORKFLOW,
                "current reference query workflow path changed")
        query_url = (
            "https://api.github.com/repos/{}/actions/runs?head_sha={}&per_page=100"
            .format(REPOSITORY, reference_sha)
        )
        require(query.get("request_url") == query_url,
                "current reference query URL does not match its source SHA")
        query_status = query.get("query_status")
        require(query_status in {"success", "error"},
                "unknown current reference query status")
        matching_ids = query.get("matching_run_ids")
        require(isinstance(matching_ids, list)
                and all(isinstance(item, str) and item.isdigit() for item in matching_ids),
                "current reference matching run IDs must be numeric text")
        if query_status == "success":
            total = query.get("query_total_count")
            require(type(total) is int and total >= 0,
                    "successful current reference query requires a nonnegative result count")
            require((total == 0) == (len(matching_ids) == 0),
                    "current reference query count and matching run IDs disagree")
        else:
            require(query.get("query_total_count") is None and not matching_ids,
                    "failed current reference query cannot claim a matching-run count")
            require(isinstance(query.get("error"), str) and query["error"].strip(),
                    "failed current reference query requires an error description")

    for role, expected_sha in (("candidate", candidate_sha), ("reference", reference_sha)):
        dco = api.get("{}_dco".format(role))
        require(isinstance(dco, dict), "current {} DCO check is required".format(role))
        check_id = dco.get("check_run_id")
        require(isinstance(check_id, str) and check_id.isdigit(),
                "current {} DCO check ID must be numeric text".format(role))
        require(dco.get("name") == "DCO", "current {} DCO check name changed".format(role))
        require(sha(dco.get("head_sha"), "current {} DCO head SHA".format(role))
                == expected_sha,
                "current {} DCO head does not match its source tip".format(role))
        require(dco.get("status") in {"queued", "in_progress", "completed"},
                "unknown current {} DCO status".format(role))
        require(dco.get("conclusion") in {
            "action_required", "cancelled", "failure", "neutral", "success",
            "skipped", "stale", "timed_out", None,
        }, "unknown current {} DCO conclusion".format(role))
        require(type(dco.get("affected_commit_count")) is int
                and dco["affected_commit_count"] >= 0,
                "current {} DCO finding count must be nonnegative".format(role))
        require(type(dco.get("annotations_count")) is int and dco["annotations_count"] >= 0,
                "current {} DCO annotation count must be nonnegative".format(role))
        result_url = dco.get("result_url")
        require(result_url is None or result_url ==
                "https://github.com/{}/runs/{}".format(REPOSITORY, check_id),
                "current {} DCO result URL does not match its ID".format(role))

    job_execution = api.get("job_execution_observation")
    if job_execution is not None:
        require(isinstance(job_execution, dict)
                and job_execution.get("schema")
                == "termrock-status-workflow-job-execution/v1",
                "unsupported current workflow job execution schema")
        timestamp(job_execution.get("observed_at"),
                  "current workflow job execution observed_at")
        require(job_execution.get("run_id") == run["run_id"],
                "current workflow job execution binds another run")
        require(job_execution.get("run_status") == run["status"],
                "current workflow job execution run status disagrees")
        total = job_execution.get("job_count")
        require(type(total) is int and total >= 0
                and total == run["job_count"],
                "current workflow job count does not match its run")
        buckets = {
            field: job_execution.get(field)
            for field in ("completed", "in_progress", "queued")
        }
        require(all(type(count) is int and count >= 0 for count in buckets.values()),
                "current workflow job state counts must be nonnegative integers")
        require(sum(buckets.values()) == total,
                "current workflow job state counts do not reconcile")
        skipped_count = job_execution.get("skipped_count", 0)
        outcome_counts = {
            field: job_execution.get(field)
            for field in ("success_count", "failure_count")
        }
        outcome_counts["skipped_count"] = skipped_count
        require(all(type(count) is int and count >= 0
                    for count in outcome_counts.values()),
                "current workflow job outcome counts must be nonnegative integers")
        require(buckets["completed"]
                == outcome_counts["success_count"]
                + outcome_counts["failure_count"]
                + outcome_counts["skipped_count"],
                "current workflow completed job outcomes do not reconcile")
        if run["status"] == "completed":
            require(buckets["completed"] == total
                    and buckets["in_progress"] == 0
                    and buckets["queued"] == 0,
                    "completed workflow run still has pending or partial job counts")
            if run["conclusion"] == "failure":
                require(outcome_counts["failure_count"] > 0,
                        "failed workflow run cannot record zero failed jobs")
        failures = job_execution.get("failed_jobs")
        require(isinstance(failures, list)
                and len(failures) == outcome_counts["failure_count"],
                "current workflow failed jobs do not match the failure count")
        skipped_jobs = job_execution.get("skipped_jobs", [])
        require(isinstance(skipped_jobs, list)
                and len(skipped_jobs) == outcome_counts["skipped_count"],
                "current workflow skipped jobs do not match the skipped count")
        seen_execution_job_ids = set()
        for kind, jobs in (("failed", failures), ("skipped", skipped_jobs)):
            for index, failure in enumerate(jobs):
                label = "current workflow {} jobs[{}]".format(kind, index)
                require(isinstance(failure, dict), "{} must be an object".format(label))
                job_id = failure.get("id")
                require(isinstance(job_id, str) and job_id.isdigit(),
                        "{} ID must be numeric text".format(label))
                require(job_id not in seen_execution_job_ids,
                        "current workflow outcome job IDs must be unique")
                seen_execution_job_ids.add(job_id)
                require(isinstance(failure.get("name"), str) and failure["name"].strip(),
                        "{} name is required".format(label))
                require(failure.get("html_url")
                        == "https://github.com/{}/actions/runs/{}/job/{}".format(
                            REPOSITORY, run["run_id"], job_id,
                        ),
                        "{} URL does not match its ID".format(label))
        require(job_execution.get("raw_api_responses_preserved") is False,
                "current workflow job capture preservation state must be explicit")
        require(job_execution.get("product_execution") == "NOT_RUN",
                "workflow job metadata cannot claim product execution")

    artifact_observation = api.get("artifact_observation")
    if artifact_observation is not None:
        require(isinstance(artifact_observation, dict)
                and artifact_observation.get("schema")
                == "termrock-status-workflow-artifact-count/v1",
                "unsupported current workflow artifact observation schema")
        timestamp(artifact_observation.get("observed_at"),
                  "current workflow artifact observation observed_at")
        require(artifact_observation.get("run_id") == run["run_id"],
                "current workflow artifact observation binds another run")
        artifact_count = artifact_observation.get("artifact_count")
        require(type(artifact_count) is int and artifact_count >= 0
                and artifact_count == run["artifact_count"],
                "current workflow artifact count does not match its run")
        expired_count = artifact_observation.get("expired_count")
        require(type(expired_count) is int and 0 <= expired_count <= artifact_count,
                "current workflow expired artifact count is invalid")
        require(artifact_observation.get("query_status") == "success",
                "current workflow artifact query must be successful for a count")
        require(type(artifact_observation.get("partial_snapshot")) is bool,
                "current workflow artifact snapshot state must be boolean")
        require(artifact_observation.get("raw_api_responses_preserved") is False,
                "current workflow artifact capture preservation state must be explicit")

    provider = observation.get("provider_annotation_capture")
    if provider is not None:
        require(isinstance(provider, dict),
                "current provider annotation capture must be an object")
        timestamp(provider.get("captured_at"), "provider annotation capture time")
        require(isinstance(provider.get("manifest_path"), str)
                and provider["manifest_path"].strip(),
                "provider annotation manifest path is required")
        sha256(provider.get("manifest_sha256"), "provider annotation manifest SHA-256")
        require(isinstance(provider.get("run_id"), str) and provider["run_id"].isdigit(),
                "provider annotation run ID must be numeric text")
        require(isinstance(provider.get("run_page_url"), str)
                and provider["run_page_url"].startswith("https://"),
                "provider annotation run page URL must be HTTPS")
        require(isinstance(provider.get("workflow_path"), str)
                and provider["workflow_path"].strip(),
                "provider annotation workflow path is required")
        sha(provider.get("workflow_commit_sha"), "provider workflow commit SHA")
        require(type(provider.get("workflow_source_size_bytes")) is int
                and provider["workflow_source_size_bytes"] >= 0,
                "provider workflow source size must be nonnegative")
        sha256(provider.get("workflow_source_sha256"),
              "provider workflow source SHA-256")
        require(type(provider.get("page_annotation_count")) is int
                and provider["page_annotation_count"] >= 0,
                "provider annotation count must be nonnegative")
        sha256(provider.get("raw_page_sha256"), "provider raw page SHA-256")
        require(provider.get("page_status") in {"Success", "Failure"},
                "unknown provider run-page status")
        require(isinstance(provider.get("annotation"), str),
                "provider annotation text is required")

    return observation


def validate_latest_ci_snapshot(
    latest: Mapping[str, Any], value: Any, queue_module: Any
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict)
            and value.get("schema") == "termrock-status-latest-ci-snapshot-v1",
            "unsupported latest CI snapshot")
    timestamp(value.get("captured_at"), "latest CI snapshot captured_at")
    interval = value.get("captured_interval_utc")
    require(isinstance(interval, list) and len(interval) == 2,
            "latest CI snapshot capture interval must have two timestamps")
    start = timestamp(interval[0], "latest CI snapshot interval start")
    end = timestamp(interval[1], "latest CI snapshot interval end")
    require(start <= end == value["captured_at"],
            "latest CI snapshot interval does not match its capture time")
    evidence = read_pinned_json(
        value.get("source_facts_evidence"), "latest CI source-facts record", queue_module
    )
    require(isinstance(evidence, dict)
            and evidence.get("record_type") ==
            "termrock-vis06-source-facts-and-build-only-status-v1",
            "latest CI evidence is not the reviewed source-facts record")
    require(evidence.get("latest_source_observation") == latest,
            "latest CI evidence does not bind the current source observation")
    require(value.get("raw_api_responses_preserved") is False,
            "latest CI raw-response preservation state must remain explicit")
    source_pair = value.get("source_pair")
    require(isinstance(source_pair, dict), "latest CI source pair is required")
    candidate_sha = sha(source_pair.get("candidate_commit"),
                        "latest CI candidate source SHA")
    reference_sha = sha(source_pair.get("reference_commit"),
                        "latest CI reference source SHA")
    require(candidate_sha == latest["candidate_remote"]["head_sha"]
            and reference_sha == latest["reference_remote"]["head_sha"],
            "latest CI source pair does not match the current source observation")

    run = value.get("candidate_run")
    require(isinstance(run, dict), "latest candidate run is required")
    run_id = run.get("run_id")
    require(isinstance(run_id, str) and run_id.isdigit(),
            "latest candidate run ID must be numeric text")
    require(sha(run.get("head_sha"), "latest candidate run head SHA") == candidate_sha,
            "latest candidate run is bound to another source commit")
    require(run.get("workflow_path") == CURRENT_CI_WORKFLOW,
            "latest candidate workflow path changed")
    require(run.get("status") in {"queued", "in_progress", "completed"},
            "unknown latest candidate CI status")
    require(run.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown latest candidate CI conclusion")
    timestamp(run.get("created_at"), "latest candidate run creation time")
    require(type(run.get("job_count")) is int and run["job_count"] >= 0,
            "latest candidate CI job count must be nonnegative")
    require(run.get("run_url") ==
            "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id),
            "latest candidate run URL does not match its ID")
    source_run = evidence.get("current_ci_observation", {}).get("api_capture", {}) \
        .get("candidate_run", {})
    require(source_run.get("run_id") == run_id
            and source_run.get("head_sha") == run["head_sha"]
            and source_run.get("job_count") == run["job_count"]
            and source_run.get("conclusion") == run["conclusion"],
            "latest candidate run fields do not match the pinned source facts")
    require(value.get("artifact_count") is None
            and value.get("artifact_query") == "NOT_QUERIED",
            "latest candidate artifact count must remain unknown when unqueried")
    require(value.get("failure_cause") == "UNKNOWN_NOT_CAPTURED",
            "latest candidate workflow failure cause must remain unknown")
    require(value.get("product_execution") == "NOT_RUN",
            "workflow metadata cannot qualify product execution")

    query = value.get("reference_query")
    require(isinstance(query, dict), "latest reference Actions query is required")
    query_sha = sha(query.get("head_sha"), "latest reference query head SHA")
    require(query_sha == reference_sha, "latest reference query binds another source commit")
    require(query.get("workflow_path") == CURRENT_CI_WORKFLOW,
            "latest reference query workflow path changed")
    require(query.get("query_status") in {"success", "error"},
            "unknown latest reference query status")
    require(query.get("request_url") ==
            "https://api.github.com/repos/{}/actions/runs?head_sha={}&per_page=100"
            .format(REPOSITORY, reference_sha),
            "latest reference query URL does not match its source commit")
    if query["query_status"] == "success":
        require(type(query.get("query_total_count")) is int
                and query["query_total_count"] >= 0,
                "successful latest reference query requires a nonnegative result count")
        require(isinstance(query.get("matching_run_ids"), list)
                and all(isinstance(item, str) and item.isdigit()
                        for item in query["matching_run_ids"])
                and (query["query_total_count"] == 0)
                == (len(query["matching_run_ids"]) == 0),
                "latest reference query count and matching IDs disagree")
    else:
        require(query.get("query_total_count") is None
                and query.get("matching_run_ids") == [],
                "failed latest reference query cannot claim a run count")
    source_query = evidence.get("current_ci_observation", {}).get("api_capture", {}) \
        .get("reference_query", {})
    require(source_query.get("query_status") == query.get("query_status")
            and source_query.get("head_sha") == query["head_sha"]
            and source_query.get("query_total_count") == query.get("query_total_count")
            and source_query.get("matching_run_ids") == query.get("matching_run_ids"),
            "latest reference query does not match the pinned source facts")

    candidate_dco = value.get("candidate_dco")
    require(isinstance(candidate_dco, dict), "latest candidate DCO check is required")
    check_id = candidate_dco.get("check_run_id")
    require(isinstance(check_id, str) and check_id.isdigit(),
            "latest candidate DCO ID must be numeric text")
    require(candidate_dco.get("name") == "DCO"
            and sha(candidate_dco.get("head_sha"), "latest candidate DCO head SHA")
            == candidate_sha,
            "latest candidate DCO check is bound to another source")
    require(candidate_dco.get("status") in {"queued", "in_progress", "completed"},
            "unknown latest candidate DCO status")
    require(candidate_dco.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown latest candidate DCO conclusion")
    missing_ids = candidate_dco.get("reported_missing_signoff_commit_ids")
    require(isinstance(missing_ids, list)
            and all(isinstance(item, str) for item in missing_ids),
            "latest DCO missing-signoff IDs are invalid")
    for item in missing_ids:
        sha(item, "latest DCO missing-signoff commit SHA")
    details_url = candidate_dco.get("details_url")
    require(details_url is None or details_url ==
            "https://github.com/{}/runs/{}".format(REPOSITORY, check_id),
            "latest candidate DCO URL does not match its ID")
    require(value.get("reference_dco") is None,
            "uncaptured reference DCO state must remain null")
    source_checks = evidence.get("current_ci_observation", {}).get(
        "separate_check_observations", []
    )
    source_dco = next((item for item in source_checks if item.get("name") == "DCO"), None)
    require(isinstance(source_dco, dict)
            and source_dco.get("check_run_id") == int(check_id)
            and source_dco.get("head_sha") == candidate_dco["head_sha"]
            and source_dco.get("reported_missing_signoff_commit_ids") == missing_ids,
            "latest candidate DCO details do not match the pinned source facts")
    return value


def validate_publication_ci_observations(
    value: Any, queue_module: Any
) -> Sequence[Mapping[str, Any]]:
    if value is None:
        return []
    require(isinstance(value, list), "publication_ci_observations must be an array or null")
    validated = []
    seen_runs = set()
    for index, record in enumerate(value):
        label = "publication_ci_observations[{}]".format(index)
        require(isinstance(record, dict), "{} must be an object".format(label))
        source_commit = sha(record.get("source_commit"), "{}.source_commit".format(label))
        manifest = read_pinned_json(
            record.get("manifest"), "{} manifest".format(label), queue_module
        )
        require(isinstance(manifest, dict), "{} manifest must be an object".format(label))
        if manifest.get("record_type") == "read_only_public_actions_run_refresh":
            require(len(manifest.get("requests", [])) == 2,
                    "{} must contain the run and jobs API observations".format(label))
            run_request = next((item for item in manifest["requests"]
                                if item.get("url", "").endswith("/actions/runs/37862074205")), None)
            jobs_request = next((item for item in manifest["requests"]
                                 if item.get("url", "").endswith(
                                     "/actions/runs/37862074205/jobs?per_page=100")), None)
            require(run_request is not None and jobs_request is not None,
                    "{} does not bind the exact P169 run and jobs endpoints".format(label))
            run_fields = run_request.get("selected_fields")
            jobs_fields = jobs_request.get("selected_fields")
            require(run_request.get("http_status") == 200
                    and jobs_request.get("http_status") == 200,
                    "{} P169 run and jobs requests were not successful".format(label))
            run_body = read_pinned_json({
                "path": run_request.get("body_path"),
                "sha256": run_request.get("body_sha256"),
            }, "{} P169 run response".format(label), queue_module)
            jobs_body = read_pinned_json({
                "path": jobs_request.get("body_path"),
                "sha256": jobs_request.get("body_sha256"),
            }, "{} P169 jobs response".format(label), queue_module)
            run_id = str(run_fields.get("id"))
            head_sha = sha(run_fields.get("head_sha"), "{}.run.head_sha".format(label))
            status = run_fields.get("status")
            conclusion = run_fields.get("conclusion")
            created_at = timestamp(run_fields.get("created_at"),
                                   "{}.run.created_at".format(label))
            job_count = jobs_fields.get("total_count")
            run_url = "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id)
            artifact_count = manifest.get("interpretation_limits", {}).get("artifact_count")
            failure_cause = manifest.get("interpretation_limits", {}).get("failure_cause")
            require(run_body.get("id") == run_fields.get("id")
                    and run_body.get("head_sha") == head_sha
                    and run_body.get("status") == status
                    and run_body.get("conclusion") == conclusion
                    and jobs_body.get("total_count") == job_count
                    and jobs_fields.get("jobs") == jobs_body.get("jobs"),
                    "{} P169 selected fields do not match the pinned API responses".format(label))
            require(source_commit == "169380c7d6a4cff9f1c43ecef592abc715e6df6a",
                    "{} legacy publication CI source commit changed".format(label))
            require(manifest.get("interpretation_limits", {}).get("product_execution")
                    == "NOT_RUN",
                    "{} cannot claim product execution".format(label))
        elif manifest.get("record_type") == "read_only_report_publication_ci_observation":
            run_fields = manifest.get("run")
            jobs_fields = manifest.get("jobs")
            require(isinstance(run_fields, dict) and isinstance(jobs_fields, dict),
                    "{} current publication run/jobs observations are incomplete".format(label))
            requests = manifest.get("raw_requests")
            require(isinstance(requests, list) and len(requests) == 2
                    and all(item.get("http_status") == 200 for item in requests),
                    "{} current publication run/jobs requests are incomplete".format(label))
            run_request = next((item for item in requests
                                if item.get("url", "").endswith(
                                    "/actions/runs/37864720297")), None)
            jobs_request = next((item for item in requests
                                 if item.get("url", "").endswith(
                                     "/actions/runs/37864720297/jobs?per_page=100")), None)
            require(run_request is not None and jobs_request is not None,
                    "{} does not bind the exact b9 run and jobs endpoints".format(label))
            run_body = read_pinned_json({
                "path": run_request.get("body_path"),
                "sha256": run_request.get("body_sha256"),
            }, "{} b9 run response".format(label), queue_module)
            jobs_body = read_pinned_json({
                "path": jobs_request.get("body_path"),
                "sha256": jobs_request.get("body_sha256"),
            }, "{} b9 jobs response".format(label), queue_module)
            run_id = str(run_fields.get("id"))
            head_sha = sha(run_fields.get("head_sha"), "{}.run.head_sha".format(label))
            status = run_fields.get("status")
            conclusion = run_fields.get("conclusion")
            created_at = timestamp(run_fields.get("created_at"),
                                   "{}.run.created_at".format(label))
            job_count = jobs_fields.get("total_count")
            run_url = run_fields.get("run_url")
            artifact_count = manifest.get("artifact_count")
            failure_cause = manifest.get("failure_cause")
            require(run_body.get("id") == run_fields.get("id")
                    and run_body.get("head_sha") == head_sha
                    and run_body.get("status") == status
                    and run_body.get("conclusion") == conclusion
                    and jobs_body.get("total_count") == job_count
                    and len(jobs_body.get("jobs", [])) == job_count,
                    "{} b9 selected fields do not match the pinned API responses".format(label))
            require(manifest.get("product_execution") == "NOT_RUN",
                    "{} cannot claim product execution".format(label))
            require(record.get("source_commit") ==
                    "b9e34b13ef47401865f1511b9babaab6e020eedc",
                    "{} latest publication source commit changed".format(label))
        else:
            raise ValueError("{} uses an unsupported publication CI manifest".format(label))
        require(run_id.isdigit() and run_id not in seen_runs,
                "{} has a missing or duplicate run ID".format(label))
        seen_runs.add(run_id)
        require(head_sha == source_commit,
                "{} CI run head does not match its publication commit".format(label))
        require(status == "completed" and conclusion == "failure",
                "{} publication workflow result changed".format(label))
        require(type(job_count) is int and job_count == 0,
                "{} publication workflow jobs must match the captured zero count".format(label))
        require(artifact_count is None and failure_cause == "UNKNOWN_NOT_CAPTURED",
                "{} unqueried artifact count and failure cause must remain unknown".format(label))
        require(run_url == "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id),
                "{} run URL does not match its ID".format(label))
        validated.append({
            "source_commit": source_commit,
            "run_id": run_id,
            "run_url": run_url,
            "created_at": created_at,
            "status": status,
            "conclusion": conclusion,
            "job_count": job_count,
            "artifact_count": artifact_count,
            "failure_cause": failure_cause,
            "manifest": record["manifest"],
        })
    require([item["source_commit"] for item in validated] == [
        "169380c7d6a4cff9f1c43ecef592abc715e6df6a",
        "b9e34b13ef47401865f1511b9babaab6e020eedc",
    ], "publication CI observations must retain P169 then b9 source order")
    return validated


def validate_report_publication_observation(
    value: Any, queue_module: Any
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict), "report_publication_observation must be an object")
    publication = read_pinned_json(
        value.get("publication_record"), "report publication record", queue_module
    )
    review = read_pinned_json(
        value.get("postcommit_review"), "report publication postcommit review", queue_module
    )
    require(isinstance(publication, dict)
            and publication.get("schema") == "termrock-publication/v1",
            "report publication record schema changed")
    commit = sha(publication.get("commit"), "report publication commit")
    parent = sha(publication.get("parent"), "report publication parent")
    tree = sha(publication.get("tree"), "report publication tree")
    require(commit == "b9e34b13ef47401865f1511b9babaab6e020eedc"
            and parent == "169380c7d6a4cff9f1c43ecef592abc715e6df6a",
            "report publication commit ancestry changed")
    push = publication.get("push")
    require(isinstance(push, dict)
            and push.get("branch") == "termrock-implementation"
            and push.get("normal_fast_forward") is True
            and push.get("exit_code") == 0
            and push.get("remote_api_confirmed") is True,
            "report publication record does not report normal fast-forward success")
    precommit_pin = publication.get("review")
    require(isinstance(precommit_pin, dict),
            "report publication precommit review pin is missing")
    precommit = read_pinned_json(precommit_pin, "report publication precommit review", queue_module)
    require(isinstance(precommit, dict)
            and precommit.get("verdict") == "READY_FOR_COMMIT_TREE",
            "report publication precommit review is not ready for the commit tree")
    require(isinstance(review, dict)
            and review.get("schema") == "termrock-postcommit-review/v1"
            and review.get("verdict") ==
            "LOCAL_COMMIT_MATCHES_REVIEWED_TREE; PUBLICATION_RECEIPT_MATCHES",
            "postcommit review verdict changed")
    observed = review.get("observed_commit")
    require(isinstance(observed, dict)
            and sha(observed.get("commit"), "reviewed publication commit") == commit
            and sha(observed.get("parent"), "reviewed publication parent") == parent
            and sha(observed.get("tree"), "reviewed publication tree") == tree,
            "postcommit review does not bind the publication commit tree")
    publication_pin = value["publication_record"]
    review_pin = value["postcommit_review"]
    review_receipt = review.get("publication_receipt")
    require(isinstance(review_receipt, dict)
            and canonical_evidence_path(
                review_receipt.get("path"), "postcommit publication receipt path",
            ) == canonical_evidence_path(
                publication_pin["path"], "publication record path",
            )
            and review_receipt.get("sha256") == publication_pin["sha256"],
            "postcommit review does not bind the publication record")
    review_binding = review.get("review_binding")
    require(isinstance(review_binding, dict)
            and canonical_evidence_path(
                review_binding.get("path"), "postcommit precommit-review path",
            ) == canonical_evidence_path(
                precommit_pin["path"], "precommit review path",
            )
            and review_binding.get("sha256") == precommit_pin["sha256"],
            "postcommit review does not bind its precommit review")
    local_state = review.get("local_state_observed", {})
    require(local_state.get("head_unchanged_from_prepublication")
            == publication.get("local_head")
            and local_state.get("source_worktree_remains_dirty") is True
            and local_state.get("local_origin_tracking_ref_was_not_refreshed") is True,
            "postcommit review local-state limitation changed")
    diff_paths = observed.get("diff_paths")
    require(isinstance(diff_paths, list)
            and all(isinstance(item, str) and item for item in diff_paths),
            "postcommit review changed-path inventory is invalid")
    return {
        "commit": commit,
        "parent": parent,
        "tree": tree,
        "changed_paths": observed.get("diff_paths"),
        "publication_record": publication_pin,
        "postcommit_review": review_pin,
        "push_session": push.get("session"),
        "remote_api_confirmed": push.get("remote_api_confirmed"),
        "remote_ref_independently_confirmed": False,
    }


def provider_annotation_is_correlated(
    observation: Mapping[str, Any]
) -> bool:
    provider = observation.get("provider_annotation_capture")
    if not isinstance(provider, dict):
        return False
    api = observation["api_capture"]
    run = api["candidate_run"]
    workflow = api["workflow_source"]
    return (
        provider.get("run_id") == run["run_id"]
        and provider.get("run_page_url") == run["run_url"]
        and provider.get("workflow_path") == run["workflow_path"]
        and provider.get("workflow_commit_sha") == run["head_sha"]
        and provider.get("workflow_commit_sha") == workflow["commit_sha"]
        and provider.get("workflow_path") == workflow["path"]
        and provider.get("workflow_source_size_bytes") == workflow["size_bytes"]
        and provider.get("workflow_source_sha256") == workflow["raw_sha256"]
        and provider.get("page_status") == "Failure"
        and provider.get("page_annotation_count") == 1
        and provider.get("annotation") == WORKFLOW_SIZE_ANNOTATION
    )


def dco_status_link(label: str, result_url: Optional[str]) -> str:
    if result_url:
        return "[{}]({})".format(label, result_url)
    return "{} (result URL unavailable)".format(label)


def render_job_execution_observation(api: Mapping[str, Any]) -> str:
    execution = api.get("job_execution_observation")
    if execution is None:
        return ""
    failures = execution["failed_jobs"]
    skipped_jobs = execution.get("skipped_jobs", [])
    failure_text = "; ".join(
        "[{}]({})".format(cell(item["name"]), item["html_url"])
        for item in failures
    ) or "none"
    skipped_text = "; ".join(
        "[{}]({})".format(cell(item["name"]), item["html_url"])
        for item in skipped_jobs
    ) or "none"
    skipped_count = execution.get("skipped_count", 0)
    snapshot_label = (
        "Final repository workflow job snapshot"
        if execution["run_status"] == "completed"
        else "Partial repository workflow job snapshot"
    )
    return (
        "| Candidate job execution snapshot | At `{observed_at}`: {total} jobs; "
        "{completed} completed, {in_progress} in progress, {queued} queued; "
        "{success_count} successful, {failure_count} failed, and {skipped_count} "
        "skipped. Failed jobs: {failures}. Skipped jobs: {skipped}. | "
        "{snapshot_label}; these job results do not qualify product checks and "
        "product readiness remains NOT_RUN. |\n"
    ).format(
        observed_at=execution["observed_at"],
        total=execution["job_count"],
        completed=execution["completed"],
        in_progress=execution["in_progress"],
        queued=execution["queued"],
        success_count=execution["success_count"],
        failure_count=execution["failure_count"],
        skipped_count=skipped_count,
        failures=failure_text,
        skipped=skipped_text,
        snapshot_label=snapshot_label,
    )


def render_e2e_subtree_parity_observation(value: Optional[Mapping[str, Any]]) -> str:
    if value is None:
        return ""
    equal = "equal" if value["equal"] else "NOT_EQUAL"
    return "\n".join([
        "## Current E2E subtree parity",
        "",
        "| Observation | Value | Scope |",
        "| --- | --- | --- |",
        "| `{path}` subtree identity | Candidate `{candidate_tree_sha}`; reference "
        "`{reference_tree_sha}`; **{parity}** | Structural `git rev-parse` comparison of "
        "candidate `{candidate_commit}` and reference `{reference_commit}` at "
        "`{observed_at}`. It is not a paired execution and does not establish visual, "
        "interaction, API, ownership, or product readiness. |",
        "",
    ]).format(parity=equal, **value)


def render_current_ci_observation(
    observation: Optional[Mapping[str, Any]], *, partial_execution: bool = False
) -> str:
    if observation is None:
        return ""
    api = observation["api_capture"]
    source_pair = api["source_pair"]
    run = api["candidate_run"]
    workflow = api["workflow_source"]
    query = api.get("reference_query")
    candidate_dco = api["candidate_dco"]
    reference_dco = api["reference_dco"]
    provider = observation.get("provider_annotation_capture")
    job_execution_row = render_job_execution_observation(api)

    if run["conclusion"] == "failure":
        if provider_annotation_is_correlated(observation):
            ci_note = (
                "Provider run-page annotation: “{}” "
                "[run page]({}) [GitHub workflow-size limit]({}). "
                "The matching workflow source is {} bytes, SHA-256 `{}`."
            ).format(
                cell(provider["annotation"]),
                provider["run_page_url"],
                "https://docs.github.com/en/actions/reference/limits#workflow-file-size",
                workflow["size_bytes"],
                workflow["raw_sha256"],
)
        else:
            ci_note = (
                "No correlated provider annotation is recorded, so the failure cause is not stated."
            )
    else:
        ci_note = "Run status: {}.".format(run["status"])

    product_note = (
        (
            "No runner jobs were recorded, so this workflow executed no product checks."
            if partial_execution
            else "No runner jobs were recorded, so product execution remains NOT_RUN."
        )
        if run["job_count"] == 0
        else "Workflow execution is repository-level evidence and supplies no additional product results."
    )


    if query is None:
        reference_ci = (
            "The reference Actions query was not captured; matching-run status is unknown."
        )
    elif query["query_status"] == "error":
        reference_ci = (
            "The reference Actions query failed; matching-run status is unknown ({})."
            .format(cell(query["error"]))
        )
    elif query["query_total_count"] == 0:
        reference_ci = (
            "The reference Actions API query completed successfully and returned "
            "0 matching Actions runs for this source tip."
        )
    else:
        reference_ci = (
            "The reference Actions API query completed successfully and returned "
            "{} matching Actions runs for this source tip."
            .format(query["query_total_count"])
        )

    def dco_note(dco: Mapping[str, Any]) -> str:
        count = dco["affected_commit_count"]
        if count == 0:
            findings = "No commits are reported with sign-off problems."
        elif count == 1:
            findings = "1 commit is reported with sign-off problems."
        else:
            findings = "{} commits are reported with sign-off problems.".format(count)
        return "{}; {}".format(
            dco_status_link(
                dco["conclusion"] or "unknown",
                dco.get("result_url"),
            ),
            findings,
        )

    provider_evidence = ""
    if provider is not None:
        provider_evidence = (
            " Provider annotation capture at {}: manifest SHA-256 `{}`; raw run-page "
            "SHA-256 `{}`."
        ).format(
            provider["captured_at"],
            provider["manifest_sha256"],
            provider["raw_page_sha256"],
        )
        if not provider_annotation_is_correlated(observation):
            provider_evidence += (
                " It is not correlated with the current candidate run and workflow source."
            )

    api_evidence = (
        "GitHub API capture at {}: manifest SHA-256 `{}`. "
        "Its source pair is candidate `{}` and reference `{}`."
    ).format(
        api["captured_at"], api["manifest_sha256"],
        source_pair["candidate_head_sha"], source_pair["reference_head_sha"],
    )

    return """\
## Current CI and Developer Certificate of Origin (DCO) observations

| Check | Observation | Scope |
| --- | --- | --- |
| Candidate Actions run [{run_id}]({run_url}) | {conclusion} at `{head_sha}`; {job_count} jobs; {artifact_count} artifacts. {ci_note} | Workflow-level result; product execution remains NOT_RUN. |
{job_execution_row}| Reference Actions query | {reference_ci} | Query source SHA `{reference_sha}`; this is not a paired execution. |
| Candidate DCO status | {candidate_dco_note} | Repository gate at `{candidate_dco_sha}`; separate from product results. |
| Reference DCO status | {reference_dco_note} | Repository gate at `{reference_dco_sha}`; separate from product results. |

{api_evidence} The candidate workflow source is `{workflow_path}` at `{workflow_sha}`, {workflow_bytes} bytes, SHA-256 `{workflow_hash}`.{provider_evidence}

{product_note} {product_scope_note}
""".format(
        run_id=run["run_id"],
        run_url=run["run_url"],
        conclusion=run["conclusion"] or "unknown",
        head_sha=run["head_sha"],
        job_count=run["job_count"],
        artifact_count=run["artifact_count"],
        job_execution_row=job_execution_row,
        ci_note=ci_note,
        reference_ci=reference_ci,
        reference_sha=source_pair["reference_head_sha"],
        candidate_dco_note=dco_note(candidate_dco),
        candidate_dco_sha=candidate_dco["head_sha"],
        reference_dco_note=dco_note(reference_dco),
        reference_dco_sha=reference_dco["head_sha"],
        api_evidence=api_evidence,
        workflow_path=workflow["path"],
        workflow_sha=workflow["commit_sha"],
        workflow_bytes=workflow["size_bytes"],
        workflow_hash=workflow["raw_sha256"],
        provider_evidence=provider_evidence,
        product_note=product_note,
        product_scope_note=(
            "Bounded execution observations are listed separately; API, ownership, exact commands, and complete readiness remain incomplete."
            if partial_execution
            else "Paired, visual, interaction, API, ownership, Ready, and Visibility Complete results remain NOT_RUN."
        ),
    )


def render_latest_ci_snapshot(snapshot: Optional[Mapping[str, Any]]) -> str:
    if snapshot is None:
        return ""
    run = snapshot["candidate_run"]
    query = snapshot["reference_query"]
    dco = snapshot["candidate_dco"]
    if query["query_status"] == "success":
        reference_text = "{} matching runs".format(query["query_total_count"])
    else:
        reference_text = "matching-run status unknown; query failed"
    missing_count = len(dco["reported_missing_signoff_commit_ids"])
    dco_status = dco_status_link(
        dco["conclusion"] or "unknown",
        dco.get("details_url"),
    )
    if missing_count == 1:
        dco_text = "{}; 1 commit is reported with sign-off problems".format(dco_status)
    else:
        dco_text = "{}; {} commits are reported with sign-off problems".format(
            dco_status, missing_count
        )
    cause_text = (
        "Failure cause UNKNOWN_NOT_CAPTURED; no provider annotation was captured."
        if run["conclusion"] == "failure"
        else "No provider annotation was captured."
    )
    return "\n".join([
        "## Latest candidate-source CI snapshot",
        "",
        "This incomplete GitHub source-facts capture applies only to candidate `{}` and reference `{}`. It is not product execution evidence.".format(
            snapshot["source_pair"]["candidate_commit"],
            snapshot["source_pair"]["reference_commit"],
        ),
        "",
        "| Check | Observation | Scope |",
        "| --- | --- | --- |",
        "| Candidate Actions run [{}]({}) | {} at `{}`; {} jobs; artifact count UNKNOWN (not queried). {} | Repository workflow only; product execution is NOT_RUN. |".format(
            run["run_id"], run["run_url"], run["conclusion"] or "unknown",
            run["head_sha"], run["job_count"], cause_text,
        ),
        "| Reference Actions query | {} | Query source `{}`; this is not a paired execution. |".format(
            reference_text, query["head_sha"],
        ),
        "| Candidate DCO status | {} | Repository gate at `{}`; separate from product results. |".format(
            dco_text, dco["head_sha"],
        ),
        "| Reference DCO check | NOT_CAPTURED | No reference DCO result is recorded for this observation. |",
        "",
        "Capture interval: {}–{}. The pinned source-facts record is `{}` (SHA-256 `{}`); raw GitHub API response bodies were not preserved in that record.".format(
            snapshot["captured_interval_utc"][0], snapshot["captured_interval_utc"][1],
            snapshot["source_facts_evidence"]["path"],
            snapshot["source_facts_evidence"]["sha256"],
        ),
        "",
    ])


def validate_current_publication_observation(
    value: Any, latest: Mapping[str, Any], archive_manifest: Optional[Mapping[str, Any]],
    queue_module: Any,
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict)
            and value.get("schema") == "termrock-current-publication-observation/v1",
            "current report-publication observation schema changed")
    require(archive_manifest is not None,
            "current report-publication observation requires the pinned source archive")
    archive_files = {
        item["path"]: item["sha256"] for item in archive_manifest["files"]
    }

    def pinned_json(pin: Any, label: str) -> Mapping[str, Any]:
        require(isinstance(pin, dict), "{}.pin is required".format(label))
        path_value = pin.get("path")
        digest = sha256(pin.get("sha256"), "{}.sha256".format(label))
        require(isinstance(path_value, str) and archive_files.get(path_value) == digest,
                "{} is not an exact member of the verified source archive".format(label))
        parsed = read_pinned_json(pin, label, queue_module)
        require(isinstance(parsed, dict), "{} must contain an object".format(label))
        return parsed

    def publication_side(side: str, expected_commit: str, expected_parent: str,
                         expected_tree: str, expected_paths: int) -> Mapping[str, Any]:
        side_record = value["candidate_report" if side == "candidate" else "reference_report"]
        record = pinned_json(side_record["publication"], "{} report publication".format(side))
        review = pinned_json(side_record["postcommit_review"],
                             "{} report postcommit review".format(side))
        expected_schema = (
            "termrock-candidate-status-publication/v1" if side == "candidate"
            else "termrock-reference-status-publication/v1"
        )
        require(record.get("schema") == expected_schema
                and record.get("commit") == expected_commit
                and record.get("parent") == expected_parent
                and record.get("tree") == expected_tree
                and record.get("paths") == expected_paths
                and record.get("tag_unchanged") is True
                and record.get("visibility") == "INCOMPLETE"
                and record.get("refactor") == "NOT_READY",
                "{} report publication facts changed".format(side))
        reviewed_commit = review.get("commit")
        if isinstance(reviewed_commit, dict):
            reviewed_commit_sha = reviewed_commit.get("object_id")
        else:
            reviewed_commit_sha = review.get("commit")
        if reviewed_commit_sha is None:
            reviewed_commit_sha = review.get("observed_commit", {}).get("commit")
        expected_review_schema = (
            "termrock-postcommit-binding-review/v1" if side == "candidate"
            else "termrock-vis01-reference-status-postcommit-binding-review/v1"
        )
        require(review.get("schema") == expected_review_schema
                and review.get("verdict") == "READY_POSTCOMMIT_BINDING"
                and reviewed_commit_sha == expected_commit,
                "{} postcommit review does not bind its report commit".format(side))
        if side == "candidate":
            require(reviewed_commit.get("parent") == expected_parent
                    and reviewed_commit.get("tree") == expected_tree
                    and reviewed_commit.get("message_matches_approved_bytes") is True,
                    "candidate postcommit review ancestry or message binding changed")
        else:
            require(review.get("parent") == expected_parent
                    and review.get("tree") == expected_tree
                    and review.get("path_binding", {}).get("reviewed_path_count") == expected_paths
                    and review.get("path_binding", {}).get("commit_path_count") == expected_paths
                    and review.get("path_binding", {}).get("exact_status_and_path_set_match") is True,
                    "reference postcommit review path or ancestry binding changed")
        return {"publication": record, "review": review}

    fixed_pair = value.get("fixed_product_pair")
    require(isinstance(fixed_pair, dict)
            and fixed_pair.get("candidate_commit") ==
            latest["candidate_remote"]["head_sha"]
            and fixed_pair.get("reference_commit") == latest["reference_remote"]["head_sha"],
            "current report context must preserve the fixed product source pair")
    candidate = publication_side(
        "candidate", "20f2d485695991632f1ff5210d696576eb602583",
        "f0a05fa55d4fd09b19be63d51f512d4c32076f67",
        "e6eb50897b20b30c873cd00e612009b251535d01", 3,
    )
    reference = publication_side(
        "reference", "28c694037ee06e093a23de2ec0e0f6ed3f7f5569",
        "48e67af8511d4883759c97e384080fcb2b112c7a",
        "ca4ee5c1f0a8108566852269e6d060450147c2cf", 89,
    )
    candidate_queue = pinned_json(
        value["candidate_report"]["queue_publication"], "candidate q59 publication"
    )
    require(candidate_queue.get("schema") == "termrock-root-publication/v1"
            and candidate_queue.get("branch") == "termrock-implementation"
            and candidate_queue.get("queue_revision") == 59
            and candidate_queue.get("commit") == "82ad0dff9eba8ae7748e9bc44623e20fd10d31bc",
            "candidate queue publication is not the archived q59 snapshot")
    candidate_tasks = pinned_json(
        value["candidate_report"]["task_snapshot"], "candidate q59 task snapshot"
    )
    reference_tasks = pinned_json(
        value["reference_report"]["historical_task_snapshot"],
        "historical reference q56 task snapshot",
    )
    require(candidate_tasks.get("schema_version") == 1
            and candidate_tasks.get("queue_revision") == 59
            and reference_tasks.get("schema_version") == 1
            and reference_tasks.get("queue_revision") == 56,
            "candidate/reference task snapshot revisions changed")

    suite = pinned_json(value["paired_suite_publication"], "paired-source publication")
    require(suite.get("schema") == "termrock-paired-source-publication/v1"
            and suite.get("candidate", {}).get("commit") == candidate["publication"]["parent"]
            and suite.get("reference", {}).get("commit") == reference["publication"]["parent"]
            and suite.get("suite_sha256") ==
            "b67efe786fb0c0f64db2aca62c572b700f2a5247d7a2258deab1313bdb581a5d"
            and suite.get("common_package_tree") ==
            "c962085b9c9e6b80ddba31d8c82e01c8d2e97179"
            and suite.get("tag_object") == TAG_OBJECT
            and suite.get("tag_commit") == TAG_COMMIT
            and suite.get("compile_list", {}).get("tests_executed") == 0
            and suite.get("paired_run") == "NOT_RUN"
            and suite.get("corpus") == "NOT_ADMITTED"
            and suite.get("visibility") == "INCOMPLETE"
            and suite.get("refactor") == "NOT_READY",
            "paired-source publication identity or blocked state changed")

    provider = value.get("prepublication_provider_observation")
    require(isinstance(provider, dict)
            and provider.get("source_commit") == candidate["publication"]["parent"]
            and provider.get("raw_actions_run_and_jobs_responses_preserved") is False,
            "pre-publication provider observation scope changed")
    root_observation = pinned_json(provider.get("root_observation"),
                                   "f0a provider root observation")
    page_pin = provider.get("provider_page")
    require(isinstance(page_pin, dict)
            and archive_files.get(page_pin.get("path")) == page_pin.get("sha256"),
            "f0a provider page is not an exact source-archive member")
    page = read_pinned_bytes(page_pin, "f0a provider run page")
    expected_annotation = "Workflow file exceeds the maximum allowed size of 500 KB."
    run = root_observation.get("run", {})
    require(root_observation.get("schema") == "termrock-ci-current-observation/v1"
            and root_observation.get("head") == provider["source_commit"]
            and run.get("id") == 37954103945
            and run.get("status") == "completed"
            and run.get("conclusion") == "failure"
            and run.get("url") ==
            "https://github.com/{}/actions/runs/37954103945".format(REPOSITORY)
            and run.get("annotation") == expected_annotation
            and run.get("sha256") == page_pin["sha256"]
            and "job_count" not in run and "artifact_count" not in run
            and expected_annotation.encode("utf-8") in page,
            "f0a provider run/page facts or unavailable-count boundary changed")
    checks = pinned_json(provider.get("check_runs"), "f0a check-runs response")
    check_runs = checks.get("check_runs")
    dco = next((item for item in check_runs if item.get("name") == "DCO"), None) \
        if isinstance(check_runs, list) else None
    require(checks.get("total_count") == 1
            and isinstance(dco, dict)
            and dco.get("id") == 113900149889
            and dco.get("head_sha") == provider["source_commit"]
            and dco.get("status") == "completed"
            and dco.get("conclusion") == "success"
            and dco.get("html_url") ==
            "https://github.com/{}/runs/113900149889".format(REPOSITORY),
            "f0a DCO check is not a successful canonical result for its source head")
    return {
        "fixed_product_pair": fixed_pair,
        "candidate": candidate,
        "reference": reference,
        "candidate_queue": candidate_queue,
        "candidate_tasks": candidate_tasks,
        "reference_tasks": reference_tasks,
        "suite": suite,
        "provider": {"source_commit": provider["source_commit"],
                     "root_observation": root_observation,
                     "run_page_sha256": page_pin["sha256"],
                     "dco": dco},
    }


def render_current_publication_observation(
    observation: Optional[Mapping[str, Any]],
) -> str:
    if observation is None:
        return ""
    candidate = observation["candidate"]["publication"]
    reference = observation["reference"]["publication"]
    suite = observation["suite"]
    provider = observation["provider"]
    run = provider["root_observation"]["run"]
    dco = provider["dco"]
    dco_link = dco_status_link("completed successfully", dco["html_url"])
    return "\n".join([
        "## Published report commits and source evidence",
        "",
        "| Branch | Published report commit | Parent source commit | Report paths | Queue snapshot |",
        "| --- | --- | --- | ---: | --- |",
        "| Candidate | [{}](https://github.com/{}/commit/{}) | {} | {} | Candidate task snapshot revision 59 (historical) |".format(
            candidate["commit"], REPOSITORY, candidate["commit"], candidate["parent"],
            candidate["paths"],
        ),
        "| Reference | [{}](https://github.com/{}/commit/{}) | {} | {} | Historical reference task snapshot revision 56 |".format(
            reference["commit"], REPOSITORY, reference["commit"], reference["parent"],
            reference["paths"],
        ),
        "",
        "These report commits publish status documents; they are not product source measurements. The f0a provider observations below apply to source commit {}, before the report commits. They do not describe checks on either report commit.".format(
            provider["source_commit"],
        ),
        "The candidate revision-59 and reference revision-56 task records are snapshots. Later queue edits are outside this evidence record.",
        "",
        "| Provider observation | Result | Scope |",
        "| --- | --- | --- |",
        "| [Actions run {}]({}) | {} / {}; provider annotation: {} | Candidate source {}. The retained Root record contains no raw Actions run/jobs responses, so job and artifact counts are not reported. |".format(
            run["id"], run["url"], run["status"], run["conclusion"],
            run["annotation"], provider["source_commit"],
        ),
        "| Candidate DCO status | {} | Repository check on source {}; separate from product results. |".format(
            dco_link, provider["source_commit"],
        ),
        "",
        "The paired-source publication records suite SHA-256 {} and common package tree {}. It records paired execution NOT_RUN, corpus NOT_ADMITTED, Visibility INCOMPLETE, and Refactor / Ready NOT_READY. The fixed product comparison pair remains candidate {} / reference {}; this shared package identity does not qualify product behavior.".format(
            suite["suite_sha256"], suite["common_package_tree"],
            observation["fixed_product_pair"]["candidate_commit"],
            observation["fixed_product_pair"]["reference_commit"],
        ),
        "",
    ])


def render_publication_observations(
    publication: Optional[Mapping[str, Any]],
    ci_observations: Sequence[Mapping[str, Any]],
) -> str:
    if publication is None and not ci_observations:
        return ""
    lines = ["## Report publication observations", ""]
    if publication is not None:
        lines.extend([
            "The pinned root publication record reports commit `{}` as a normal fast-forward from `{}` (tree `{}`) with {} queue/test/documentation paths. Its postcommit review matched the local commit tree and publication record; the reviewer could not independently refresh the remote branch tip. This administrative publication is separate from product execution.".format(
                publication["commit"], publication["parent"], publication["tree"],
                len(publication["changed_paths"]),
            ),
            "",
            "Publication record `{}` (SHA-256 `{}`); postcommit review `{}` (SHA-256 `{}`).".format(
                publication["publication_record"]["path"],
                publication["publication_record"]["sha256"],
                publication["postcommit_review"]["path"],
                publication["postcommit_review"]["sha256"],
            ),
            "",
        ])
    if ci_observations:
        lines.extend([
            "| Publication commit | Actions run | Result | Jobs | Artifacts | Failure cause | Product execution |",
            "| --- | --- | --- | ---: | ---: | --- | --- |",
        ])
        for item in ci_observations:
            artifacts = "UNKNOWN_NOT_QUERIED" if item["artifact_count"] is None else str(
                item["artifact_count"]
            )
            lines.append(
                "| `{}` | [{}]({}) | {} / {} (created {}) | {} | {} | {} | NOT_RUN |".format(
                    item["source_commit"], item["run_id"], item["run_url"],
                    item["status"], item["conclusion"], item["created_at"],
                    item["job_count"], artifacts, item["failure_cause"],
                )
            )
        lines.extend([
            "",
            "These workflow runs belong to their exact report publication commits. Neither run provides product checks on the selected 1d/b274 source pair; no failure cause or artifact count is inferred.",
            "",
        ])
    return "\n".join(lines)


def render_source_observation_history(
    history: Sequence[Mapping[str, Any]],
    evidence_gaps: Optional[Mapping[str, Any]] = None,
) -> str:
    if not history:
        return ""
    rows = []
    for record in history:
        source = record["source_observation"]
        candidate = source["candidate_remote"]["head_sha"]
        reference = source["reference_remote"]["head_sha"]
        evidence = record["source_evidence"]
        observation = record["current_ci_observation"]
        if observation is None:
            ci_text = "No CI/DCO capture recorded; source-only observation."
            capture_text = "No CI capture"
        else:
            api = observation["api_capture"]
            run = api["candidate_run"]
            query = api.get("reference_query")
            query_text = (
                "reference query returned {} runs".format(query["query_total_count"])
                if query is not None and query.get("query_status") == "success"
                else "reference query unavailable/unknown"
            )
            dco = api["candidate_dco"]
            reference_dco = api["reference_dco"]
            ci_text = (
                "run {} {} ({} jobs, {} artifacts); {}; candidate/reference DCO {} / {}"
            ).format(
                run["run_id"], run["conclusion"] or "unknown", run["job_count"],
                run["artifact_count"], query_text, dco["check_run_id"],
                reference_dco["check_run_id"],
            )
            provider = observation.get("provider_annotation_capture")
            capture_text = "API {}".format(api["captured_at"])
            if provider is not None:
                capture_text += "; provider page {}".format(provider["captured_at"])
            ci_text += "; product execution NOT_RUN"
        rows.append(
            "| `{}` / `{}` | {} | {} | {} (evidence SHA-256 `{}`) | {} | {} |".format(
                candidate, reference, source["observed_at"],
                cell(source["method"]), cell(ci_text), evidence["sha256"],
                cell(capture_text), record["superseded_at"],
            )
        )
    return "\n".join([
        "## Previous source observations",
        "",
        "These source snapshots remain historical. Their CI and Developer Certificate of Origin (DCO) captures are bound to the exact pair in each row. They do not qualify product results or change the fixed comparison pair.",
        "",
        "| Candidate / reference source pair | Source observed at | Method | CI and DCO observation (source evidence) | CI capture times | Next source observation at |",
        "| --- | --- | --- | --- | --- | --- |",
        *rows,
        "",
        *([
            "Historical raw-evidence coverage gap: normalized source/CI summaries for {} retained records remain, but their original raw source and CI capture files are not in the repository archive. The gap does not change the fixed source pair or the current readiness assessment; current candidate measurements and paired-result gaps determine readiness separately.".format(
                len(evidence_gaps["records"])
            ),
            "",
        ] if evidence_gaps is not None else []),
    ])



def validate_current_status_observations(
    value: Any, latest: Mapping[str, Any], authority: Mapping[str, Any],
    queue_module: Any,
) -> Optional[Mapping[str, Any]]:
    if value is None:
        return None
    require(isinstance(value, dict)
            and value.get("schema") == "termrock-status-current-observations-v1",
            "unsupported current status observations schema")
    timestamp(value.get("observed_at"), "current status observations observed_at")

    pair = value.get("fixed_comparison_pair")
    source_pair_matches(
        pair, latest["candidate_remote"]["head_sha"],
        latest["reference_remote"]["head_sha"], "current status fixed pair",
    )
    tag = value.get("immutable_visual_tag")
    require(isinstance(tag, dict)
            and tag.get("tag_object_sha") == authority["tag_object_sha"]
            and tag.get("peeled_commit_sha") == authority["commit_sha"],
            "current status immutable visual tag identity changed")

    branch = value.get("branch_tip")
    require(isinstance(branch, dict)
            and branch.get("branch") == "termrock-implementation",
            "current implementation branch observation is required")
    branch_head = sha(branch.get("head_sha"), "current implementation branch head")
    branch_tree = sha(branch.get("tree_sha"), "current implementation branch tree")
    timestamp(branch.get("captured_at"), "current implementation branch capture time")
    branch_api = read_pinned_json(
        branch.get("branch_response"), "current implementation branch API response",
        queue_module,
    )
    require(isinstance(branch_api, dict)
            and branch_api.get("name") == branch["branch"],
            "current branch API response names another branch")
    api_commit = branch_api.get("commit")
    require(isinstance(api_commit, dict)
            and api_commit.get("sha") == branch_head,
            "current branch API does not match the recorded tip")
    api_commit_detail = api_commit.get("commit")
    require(isinstance(api_commit_detail, dict)
            and isinstance(api_commit_detail.get("tree"), dict)
            and api_commit_detail["tree"].get("sha") == branch_tree,
            "current branch API tree does not match the recorded tip")
    runs_by_head = read_pinned_json(
        branch.get("workflow_runs_response"),
        "current implementation branch workflow-runs response", queue_module,
    )
    require(isinstance(runs_by_head, dict)
            and type(runs_by_head.get("total_count")) is int
            and isinstance(runs_by_head.get("workflow_runs"), list),
            "current branch workflow-runs response is malformed")

    gates = value.get("branch_gate_observations")
    require(isinstance(gates, list) and len(gates) >= 1,
            "branch gate observations must be a nonempty array")
    gate_by_head: dict[str, Mapping[str, Any]] = {}
    validated_dco_result_urls: dict[str, Optional[str]] = {}
    for index, gate in enumerate(gates):
        label = "branch_gate_observations[{}]".format(index)
        require(isinstance(gate, dict), "{} must be an object".format(label))
        head = sha(gate.get("head_sha"), "{}.head_sha".format(label))
        sha(gate.get("tree_sha"), "{}.tree_sha".format(label))
        check_id = gate.get("dco_check_run_id")
        require(isinstance(check_id, str) and check_id.isdigit(),
                "{} DCO check ID must be numeric text".format(label))
        require(gate.get("dco_name") == "DCO"
                and gate.get("dco_status") == "completed"
                and gate.get("dco_conclusion") in {"success", "failure"},
                "{} DCO result is incomplete or unknown".format(label))
        timestamp(gate.get("captured_at"), "{} capture time".format(label))
        checks = read_pinned_json(
            gate.get("check_runs_response"), "{} check-runs response".format(label),
            queue_module,
        )
        require(isinstance(checks, dict)
                and isinstance(checks.get("check_runs"), list),
                "{} check-runs response is malformed".format(label))
        matching = [
            item for item in checks["check_runs"]
            if isinstance(item, dict) and str(item.get("id")) == check_id
        ]
        require(len(matching) == 1,
                "{} DCO check ID is absent or duplicated in its capture".format(label))
        check = matching[0]
        require(check.get("name") == gate["dco_name"]
                and check.get("head_sha") == head
                and check.get("status") == gate["dco_status"]
                and check.get("conclusion") == gate["dco_conclusion"],
                "{} DCO fields do not match the captured check".format(label))
        result_url = check.get("html_url")
        require(result_url is None or result_url ==
                "https://github.com/{}/runs/{}".format(REPOSITORY, check_id),
                "{} DCO result URL does not match its ID".format(label))
        validated_dco_result_urls[head] = result_url
        require(head not in gate_by_head,
                "duplicate branch gate observation for one commit")
        gate_by_head[head] = gate

    runs = value.get("workflow_run_observations")
    require(isinstance(runs, list) and len(runs) >= 1,
            "workflow run observations must be a nonempty array")
    run_by_head: dict[str, Mapping[str, Any]] = {}
    for index, observation in enumerate(runs):
        label = "workflow_run_observations[{}]".format(index)
        require(isinstance(observation, dict), "{} must be an object".format(label))
        run_id = observation.get("run_id")
        require(isinstance(run_id, str) and run_id.isdigit(),
                "{} run ID must be numeric text".format(label))
        head = sha(observation.get("head_sha"), "{}.head_sha".format(label))
        tree = sha(observation.get("tree_sha"), "{}.tree_sha".format(label))
        require(head in gate_by_head,
                "{} has no commit-bound DCO observation".format(label))
        require(observation.get("workflow_path") == CURRENT_CI_WORKFLOW,
                "{} workflow path changed".format(label))
        require(observation.get("status") == "completed"
                and observation.get("conclusion") in {"failure", "success"},
                "{} workflow run is incomplete or unknown".format(label))
        timestamp(observation.get("created_at"), "{} creation time".format(label))
        run_url = "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id)
        require(observation.get("run_url") == run_url,
                "{} run URL does not match its ID".format(label))
        captures = observation.get("captures")
        require(isinstance(captures, dict), "{} captures are required".format(label))
        run_detail = read_pinned_json(
            captures.get("run_detail"), "{} run detail".format(label), queue_module,
        )
        require(isinstance(run_detail, dict)
                and str(run_detail.get("id")) == run_id
                and run_detail.get("name") == observation["workflow_path"]
                and run_detail.get("head_sha") == head
                and run_detail.get("head_branch") == branch["branch"]
                and run_detail.get("status") == observation["status"]
                and run_detail.get("conclusion") == observation["conclusion"]
                and run_detail.get("created_at") == observation["created_at"]
                and run_detail.get("html_url") == run_url,
                "{} fields do not match the captured workflow run".format(label))
        head_commit = run_detail.get("head_commit")
        require(isinstance(head_commit, dict)
                and head_commit.get("id") == head
                and head_commit.get("tree_id") == tree,
                "{} commit tree does not match the captured run".format(label))
        jobs = read_pinned_json(
            captures.get("jobs"), "{} jobs response".format(label), queue_module,
        )
        artifacts = read_pinned_json(
            captures.get("artifacts"), "{} artifacts response".format(label),
            queue_module,
        )
        require(isinstance(jobs, dict) and type(jobs.get("total_count")) is int
                and isinstance(jobs.get("jobs"), list),
                "{} jobs response is malformed".format(label))
        require(isinstance(artifacts, dict)
                and type(artifacts.get("total_count")) is int
                and isinstance(artifacts.get("artifacts"), list),
                "{} artifacts response is malformed".format(label))
        require(observation.get("job_count") == jobs["total_count"]
                == len(jobs["jobs"]),
                "{} job count does not match the captured API response".format(label))
        require(observation.get("artifact_count") == artifacts["total_count"]
                == len(artifacts["artifacts"]),
                "{} artifact count does not match the captured API response".format(label))
        provider_annotation = observation.get("provider_annotation")
        require(isinstance(provider_annotation, str) and provider_annotation.strip(),
                "{} provider annotation is required".format(label))
        page_capture = read_pinned_bytes(
            captures.get("provider_page"), "{} provider run page".format(label),
        )
        annotation_prefix = provider_annotation.split(" See ", 1)[0]
        require(page_capture.count(annotation_prefix.encode("utf-8"))
                == observation.get("provider_annotation_count") == 1
                and b"#workflow-file-size" in page_capture
                and b"1 error" in page_capture,
                "{} annotation does not match its provider page".format(label))
        require(run_id.encode("ascii") in page_capture,
                "{} provider page does not identify its run".format(label))
        workflow_source = observation.get("workflow_source")
        require(isinstance(workflow_source, dict),
                "{} workflow source is required".format(label))
        workflow_api = read_pinned_json(
            captures.get("workflow_content_api"),
            "{} workflow content API response".format(label), queue_module,
        )
        workflow_bytes = read_pinned_bytes(
            captures.get("workflow_bytes"), "{} workflow source bytes".format(label),
        )
        require(isinstance(workflow_api, dict)
                and workflow_api.get("path") == observation["workflow_path"]
                and workflow_api.get("size") == workflow_source.get("size_bytes")
                and workflow_api.get("sha") == workflow_source.get("github_blob_sha"),
                "{} workflow content metadata does not match its capture".format(label))
        require(len(workflow_bytes) == workflow_source.get("size_bytes")
                and hashlib.sha256(workflow_bytes).hexdigest()
                == workflow_source.get("sha256")
                and captures["workflow_bytes"]["sha256"] == workflow_source.get("sha256"),
                "{} workflow source bytes do not match their pin".format(label))
        require(workflow_api.get("html_url")
                == "https://github.com/{}/blob/{}/{}".format(
                    REPOSITORY, head, observation["workflow_path"]),
                "{} workflow content is not bound to its run head".format(label))
        require(observation.get("product_execution") == "NOT_RUN",
                "{} workflow metadata cannot record product execution".format(label))
        require(head not in run_by_head,
                "duplicate workflow run observation for one commit")
        run_by_head[head] = observation

    require(branch_head in run_by_head and branch_head in gate_by_head,
            "current branch tip lacks captured workflow and DCO observations")
    current_run_ids = {
        str(item.get("id")) for item in runs_by_head["workflow_runs"]
        if isinstance(item, dict) and item.get("head_sha") == branch_head
    }
    require(run_by_head[branch_head]["run_id"] in current_run_ids,
            "current branch workflow run is absent from the head-SHA query")

    suite = value.get("tag_suite")
    require(isinstance(suite, dict), "tag capture suite identity is required")
    sha256(suite.get("suite_sha256"), "tag capture suite SHA-256")
    freeze = read_pinned_json(
        suite.get("source_freeze"), "tag capture source freeze", queue_module,
    )
    require(isinstance(freeze, dict)
            and freeze.get("schema") == "termrock-e2e/shared-suite-source-freeze-v1"
            and freeze.get("status") == "PREPARED_ONLY_SOURCE_NOT_EXECUTED"
            and freeze.get("suite_sha256") == suite["suite_sha256"]
            and freeze.get("file_count") == suite.get("file_count")
            and freeze.get("source_commit") is None
            and freeze.get("source_tree") is None
            and suite.get("source_commit") is None
            and suite.get("source_tree") is None,
            "tag suite freeze identity or null source identity changed")
    attempts = value.get("tag_capture_attempts")
    require(isinstance(attempts, list) and len(attempts) == 2,
            "tag capture attempt ledger must preserve the two recorded attempts")
    for attempt in attempts:
        require(isinstance(attempt, dict)
                and attempt.get("tag_object_sha") == authority["tag_object_sha"]
                and attempt.get("tag_commit_sha") == authority["commit_sha"],
                "tag attempt does not match the immutable visual tag")
        require(attempt.get("capture_status") == "NOT_RUN"
                and attempt.get("qualification_status") == "NOT_RUN",
                "tag capture and qualification must remain NOT_RUN")
        runner_result = read_pinned_json(
            attempt.get("runner_result"), "{} runner result".format(attempt.get("attempt_id")),
            queue_module,
        )
        require(isinstance(runner_result, dict)
                and runner_result.get("schema")
                == "termrock-rust-tool-test-execution/immutable-tag-capture-result-v1",
                "{} runner result schema changed".format(attempt.get("attempt_id")))
        stdout = read_pinned_bytes(
            attempt.get("stdout"), "{} Nextest stdout".format(attempt.get("attempt_id")),
        )
        stderr = read_pinned_bytes(
            attempt.get("stderr"), "{} Nextest stderr".format(attempt.get("attempt_id")),
        )
        if attempt.get("attempt_id") == "R12":
            require(attempt.get("state") == "runner_stopped_before_tests"
                    and attempt.get("nextest_selected") == 0
                    and attempt.get("nextest_passed") == 0
                    and attempt.get("nextest_failed") == 0
                    and stdout == b""
                    and b"NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1" in stderr,
                    "R12 does not match the captured pre-test tool stop")
        elif attempt.get("attempt_id") == "R13":
            require(attempt.get("state") == "test_failed_before_pty"
                    and attempt.get("nextest_selected") == 1
                    and attempt.get("nextest_passed") == 0
                    and attempt.get("nextest_failed") == 1
                    and attempt.get("nextest_skipped") == 0
                    and runner_result.get("nextest_summaries") is None
                    and b"1 test run: 0 passed, 1 failed, 0 skipped" in stderr
                    and attempt.get("failure_summary", "").encode("utf-8") in stderr,
                    "R13 does not match the pinned pre-PTY Nextest failure")
        else:
            raise ValueError("unknown tag capture attempt ID")

    regression = value.get("tag_validator_regression")
    require(isinstance(regression, dict)
            and regression.get("scope")
            == "synthetic Rust black-box validator regression only",
            "tag validator regression scope changed")
    regression_result = read_pinned_json(
        regression.get("runner_result"), "tag validator regression receipt",
        queue_module,
    )
    regression_stderr = read_pinned_bytes(
        regression.get("stderr"), "tag validator regression stderr",
    )
    summaries = regression_result.get("nextest_summaries")
    require(regression_result.get("schema")
            == "termrock-vis06-tree-oid-rust-regression-run-result/v1"
            and isinstance(summaries, list) and len(summaries) == 1
            and summaries[0] == {
                "selected": regression.get("selected"),
                "passed": regression.get("passed"),
                "failed": regression.get("failed"),
                "skipped": regression.get("skipped"),
            }
            and regression_result.get("source_inputs_unchanged") is True
            and regression.get("source_inputs_unchanged") is True
            and regression.get("capture_status") == "NOT_RUN"
            and regression.get("qualification_status") == "NOT_QUALIFIED"
            and b"tag isolated environment facts differ from the run record" in regression_stderr,
            "tag validator regression does not match its captured test result")

    control = value.get("external_tool_control")
    require(isinstance(control, dict)
            and control.get("tool") == "Velnor Actions CLI"
            and control.get("source_commit") is None
            and control.get("source_git_metadata")
            == "absent; this execution binds tree contents, not a commit or branch"
            and control.get("qualification_status") == "NOT_QUALIFIED",
            "Velnor control must remain unqualified and commit-unbound")
    receipt = read_pinned_json(
        control.get("receipt"), "Velnor actual receipt", queue_module,
    )
    review = read_pinned_json(
        control.get("review"), "Velnor actual run review", queue_module,
    )
    summary = review.get("result_summary") if isinstance(review, dict) else None
    pins = review.get("execution_pins") if isinstance(review, dict) else None
    source = review.get("source_binding") if isinstance(review, dict) else None
    require(isinstance(receipt, dict)
            and receipt.get("record_type") == "velnor_ca3ef_latest_source_gate_receipt_v1"
            and receipt.get("source_tree") == control.get("source_tree")
            and receipt.get("source_commit") is None
            and receipt.get("source_manifest_sha256") == control.get("source_manifest_sha256")
            and receipt.get("source_binding_sha256") == control.get("source_binding_sha256")
            and receipt.get("qualification") == "NOT_QUALIFIED"
            and receipt.get("status") == control.get("status"),
            "Velnor actual receipt does not match the recorded source binding")
    require(isinstance(summary, dict) and isinstance(pins, dict)
            and isinstance(source, dict)
            and pins.get("actual_receipt_sha256") == control["receipt"]["sha256"]
            and source.get("tree") == control["source_tree"]
            and source.get("git_metadata") == control["source_git_metadata"]
            and summary.get("expected_commands") == control.get("expected_commands")
            and summary.get("completed_commands") == control.get("completed_commands")
            and summary.get("expected_tests") == control.get("expected_tests")
            and summary.get("tests_executed") == control.get("executed_tests")
            and summary.get("tests_passed") == control.get("passed_tests")
            and summary.get("tests_failed") == control.get("failed_tests")
            and summary.get("overall_qualification") == "NOT_QUALIFIED",
            "Velnor actual review does not match its receipt summary")
    require(control.get("passed_tests") + control.get("failed_tests")
            == control.get("executed_tests")
            and control.get("executed_tests") + control.get("not_run_tests")
            == control.get("expected_tests"),
            "Velnor test counts do not reconcile")
    require(control.get("cli_build_status") == "PASS"
            and control.get("cli_version_status") == "PASS"
            and control.get("cli_version") == "velnor-actions 0.1.5",
            "Velnor build and version controls must remain separate recorded passes")
    require(isinstance(control.get("test_commands"), list)
            and len(control["test_commands"]) == 4,
            "Velnor completed test-command rows are required")
    completed = review.get("completed_commands", [])
    for command in control["test_commands"]:
        item = next(
            (row for row in completed
             if isinstance(row, dict) and row.get("index") == command.get("index")),
            None,
        )
        require(isinstance(item, dict)
                and item.get("expected") == command.get("expected")
                and item.get("executed") == command.get("executed")
                and item.get("passed") == command.get("passed")
                and item.get("failed") == command.get("failed"),
                "Velnor command {} counts do not match the actual review".format(
                    command.get("index")
                ))

    candidate_run = value.get("candidate_api_deferred_run")
    require(isinstance(candidate_run, dict)
            and candidate_run.get("candidate_commit")
            == latest["candidate_remote"]["head_sha"]
            and candidate_run.get("measurement_status") == "FAILED"
            and candidate_run.get("paired_reference_status") == "NOT_RUN"
            and candidate_run.get("paired_visual_status") == "NOT_RUN"
            and candidate_run.get("acceptance_decision") == "NOT_RECORDED",
            "candidate deferred/API run must remain a source-bound partial failure")
    expected = candidate_run.get("expected")
    executed = candidate_run.get("executed")
    passed = candidate_run.get("passed")
    failed = candidate_run.get("failed")
    require(all(type(count) is int and count >= 0
                for count in (expected, executed, passed, failed))
            and expected == executed and passed + failed == executed and failed > 0
            and candidate_run.get("timed_out") is False
            and candidate_run.get("parse_errors") == [],
            "candidate deferred/API result counts or completion state do not reconcile")
    failure = candidate_run.get("failure")
    require(isinstance(failure, dict)
            and failure.get("requirement_id") == "BD-21"
            and failure.get("case_id") == "W13-05"
            and failure.get("test_name") == "w13_filter_wide_trail_cells_clear"
            and failure.get("registry_status") == "NOT_RUN"
            and isinstance(failure.get("failure_summary"), str),
            "candidate API failure must preserve its requirement and registry states")
    validate_product_evidence_packet(
        candidate_run.get("packet_manifest"), candidate_run, queue_module,
    )
    run_receipt = read_pinned_json(
        candidate_run.get("receipt"), "candidate deferred/API run receipt",
        queue_module,
    )
    run_ledger = read_pinned_json(
        candidate_run.get("ledger"), "candidate deferred/API run ledger",
        queue_module,
    )
    run_review = read_pinned_json(
        candidate_run.get("review"), "candidate deferred/API run independent review",
        queue_module,
    )
    execution = run_receipt.get("execution") if isinstance(run_receipt, dict) else None
    selection = run_receipt.get("selection") if isinstance(run_receipt, dict) else None
    receipt_source = run_receipt.get("source") if isinstance(run_receipt, dict) else None
    ledger_run = run_ledger.get("run") if isinstance(run_ledger, dict) else None
    ledger_counts = run_ledger.get("counts") if isinstance(run_ledger, dict) else None
    review_run = run_review.get("run") if isinstance(run_review, dict) else None
    review_ledger = run_review.get("ledger") if isinstance(run_review, dict) else None
    review_run_counts = review_run.get("counts") if isinstance(review_run, dict) else None
    review_run_selection = review_run.get("selection") if isinstance(review_run, dict) else None
    review_failed_rows = review_run.get("failed_rows") if isinstance(review_run, dict) else None
    failed_row = (
        review_failed_rows[0]
        if isinstance(review_failed_rows, list)
        and len(review_failed_rows) == 1
        and isinstance(review_failed_rows[0], dict)
        else None
    )
    review_ledger_counts = (
        review_ledger.get("counts") if isinstance(review_ledger, dict) else None
    )
    require(isinstance(execution, dict) and isinstance(selection, dict)
            and isinstance(receipt_source, dict)
            and run_receipt.get("schema") == "termrock-vis04-product23-run-receipt/v1"
            and receipt_source.get("candidate_product_commit")
            == candidate_run["candidate_commit"]
            and selection.get("package") == candidate_run.get("package")
            and selection.get("binary") == candidate_run.get("binary")
            and selection.get("expected_count") == expected
            and execution.get("result") == candidate_run.get("measurement_status")
            and execution.get("exit_code") == candidate_run.get("child_exit_code")
            and execution.get("nextest_run_id") == candidate_run.get("nextest_run_id")
            and execution.get("timed_out") is False,
            "candidate deferred/API receipt does not match its measurement")
    require(isinstance(ledger_run, dict) and isinstance(ledger_counts, dict)
            and ledger_run.get("receipt_sha256") == candidate_run["receipt"]["sha256"]
            and ledger_counts.get("expected") == expected
            and ledger_counts.get("passed") == passed
            and ledger_counts.get("failed") == failed,
            "candidate deferred/API ledger does not reconcile with its receipt")
    require(isinstance(run_review, dict)
            and run_review.get("verdict") == candidate_run.get("review_verdict")
            and candidate_run.get("review_verdict") == "VERIFIED_FAILED_RUN"
            and isinstance(review_run, dict)
            and review_run.get("receipt_sha256") == candidate_run["receipt"]["sha256"]
            and review_run.get("nextest_run_id") == candidate_run.get("nextest_run_id")
            and review_run.get("child_exit_code") == candidate_run.get("child_exit_code")
            and review_run.get("timed_out") is False
            and review_run.get("parse_errors") == candidate_run.get("parse_errors")
            and isinstance(review_run_counts, dict)
            and review_run_counts.get("passed") == passed
            and review_run_counts.get("failed") == failed
            and isinstance(review_run_selection, dict)
            and review_run_selection.get("package") == candidate_run.get("package")
            and review_run_selection.get("binary") == candidate_run.get("binary")
            and review_run_selection.get("expected_count") == expected
            and review_run.get("outer_nextest_summary")
            == "{} tests run: {} passed, {} failed, {} skipped".format(
                executed, passed, failed, candidate_run.get("nextest_filtered")
            )
            and isinstance(failed_row, dict)
            and failed_row.get("id") == failure.get("requirement_id")
            and failed_row.get("cases") == [failure.get("case_id")]
            and failed_row.get("test_name") == failure.get("test_name")
            and failed_row.get("registry_status") == failure.get("registry_status")
            and isinstance(review_ledger_counts, dict)
            and review_ledger_counts.get("expected") == expected
            and review_ledger_counts.get("passed") == passed
            and review_ledger_counts.get("failed") == failed,
            "candidate deferred/API independent review does not match its receipts")

    readiness = value.get("report_readiness")
    require(isinstance(readiness, dict), "report readiness states are required")
    for field in (
        "visibility_complete", "refactor_ready", "reference_qualified",
        "command_ready", "evidence_freshness",
    ):
        require(readiness.get(field) == "NOT_RUN",
                "current observations cannot promote {}".format(field))
    validated = dict(value)
    validated["_validated_dco_result_urls"] = validated_dco_result_urls
    return validated



def _provider_archive_members(
    pin: Any,
    queue_module: Any,
    archive_schema: str = PROVIDER_ARCHIVE_SCHEMA,
) -> tuple[str, Mapping[str, Mapping[str, Any]]]:
    require(isinstance(pin, dict) and set(pin) == {"path", "sha256", "bytes"},
            "provider archive manifest pin has missing or unknown fields")
    manifest_path = _product_archive_path(pin.get("path"), "provider archive manifest")
    require(Path(manifest_path).name == "MANIFEST.json",
            "provider archive manifest must be named MANIFEST.json")
    manifest_size = pin.get("bytes")
    require(type(manifest_size) is int and 0 < manifest_size <= 1024 * 1024,
            "provider archive manifest byte count is outside its bound")
    raw = read_pinned_bytes(pin, "provider archive manifest")
    require(len(raw) == manifest_size,
            "provider archive manifest byte count does not match its pin")
    try:
        manifest = queue_module.strict_json_loads(raw)
    except Exception as error:
        raise ValueError("provider archive manifest is not strict JSON: {}".format(error)) from error
    require(isinstance(manifest, dict)
            and set(manifest) == {"schema", "increment", "status", "source_baseline", "files"}
            and manifest.get("schema") == archive_schema
            and isinstance(manifest.get("increment"), str)
            and manifest["increment"] == Path(manifest_path).parent.name
            and isinstance(manifest.get("status"), str)
            and isinstance(manifest.get("source_baseline"), dict),
            "provider archive manifest identity or fields are malformed")
    files = manifest.get("files")
    require(isinstance(files, list) and 0 < len(files) <= 512,
            "provider archive member count is outside its bound")
    archive_root = Path(manifest_path).parent.as_posix()
    member_by_path: dict[str, Mapping[str, Any]] = {}
    total_bytes = 0
    for index, member in enumerate(files):
        label = "provider archive files[{}]".format(index)
        require(isinstance(member, dict)
                and set(member) == {
                    "path", "source_path", "bytes", "sha256", "source_mode", "description",
                }, "{} has missing or unknown fields".format(label))
        relative = member.get("path")
        require(isinstance(relative, str) and not Path(relative).is_absolute()
                and Path(relative).as_posix() == relative
                and bool(Path(relative).parts)
                and all(part not in {"", ".", ".."} for part in Path(relative).parts)
                and relative.startswith("raw/"),
                "{}.path must be a safe path below raw/".format(label))
        full_path = _product_archive_path(
            (Path(archive_root) / relative).as_posix(), "{}.path".format(label),
        )
        require(isinstance(member.get("source_path"), str)
                and member["source_path"].strip()
                and isinstance(member.get("description"), str)
                and member["description"].strip()
                and isinstance(member.get("source_mode"), str)
                and re.fullmatch(r"[0-7]{3,4}", member["source_mode"]) is not None,
                "{}.source provenance fields are malformed".format(label))
        size = member.get("bytes")
        require(type(size) is int and 0 <= size <= MAX_EXECUTION_EVIDENCE_BYTES,
                "{}.bytes is outside its bound".format(label))
        digest = sha256(member.get("sha256"), "{}.sha256".format(label))
        require(full_path not in member_by_path, "provider archive repeats a member path")
        member_by_path[full_path] = {
            "bytes": size,
            "sha256": digest,
            "source_path": member["source_path"],
        }
        total_bytes += size
        require(total_bytes <= PRODUCT_PHASE_ARCHIVE_MAX_BYTES,
                "provider archive exceeds its aggregate byte bound")
    return archive_root, member_by_path


def _read_provider_capture(
    pin: Any, role: str, archive_root: str,
    member_by_path: Mapping[str, Mapping[str, Any]], queue_module: Any,
) -> Any:
    label = "provider observation capture {}".format(role)
    require(isinstance(pin, dict) and set(pin) == {"path", "sha256", "bytes"},
            "{}.pin has missing or unknown fields".format(label))
    path = _product_archive_path(pin.get("path"), label)
    require(path.startswith(archive_root + "/"),
            "{} is outside its pinned archive".format(label))
    member = member_by_path.get(path)
    require(member is not None
            and member.get("bytes") == pin.get("bytes")
            and member.get("sha256") == pin.get("sha256"),
            "{} is not bound to its archive member".format(label))
    expected_filename = PROVIDER_CAPTURE_FILENAMES[role]
    require(Path(path).name == expected_filename
            and Path(member["source_path"]).name == expected_filename
            and member["source_path"].endswith("/raw-api/" + expected_filename),
            "{}.source_path is not the expected raw provider endpoint capture".format(label))
    size = pin.get("bytes")
    require(type(size) is int and 0 <= size <= MAX_EXECUTION_EVIDENCE_BYTES,
            "{}.bytes is outside its bound".format(label))
    raw = read_pinned_bytes(pin, label)
    require(len(raw) == size, "{}.bytes does not match its pin".format(label))
    if role not in PROVIDER_JSON_ROLES:
        return raw
    try:
        value = queue_module.strict_json_loads(raw)
    except Exception as error:
        raise ValueError("{} is not strict JSON: {}".format(label, error)) from error
    require(isinstance(value, dict), "{} must contain a JSON object".format(label))
    return value


def _read_provider_poll_capture(
    pin: Any,
    role: str,
    archive_root: str,
    member_by_path: Mapping[str, Mapping[str, Any]],
    queue_module: Any,
) -> Any:
    label = "provider poll capture {}".format(role)
    require(role in PROVIDER_POLL_CAPTURE_FILENAMES,
            "{} is not a supported provider poll role".format(label))
    require(isinstance(pin, dict) and set(pin) == {"path", "sha256", "bytes"},
            "{} pin has missing or unknown fields".format(label))
    path = _product_archive_path(pin.get("path"), label)
    require(path.startswith(archive_root + "/"),
            "{} is outside its pinned archive".format(label))
    member = member_by_path.get(path)
    require(member is not None
            and member.get("bytes") == pin.get("bytes")
            and member.get("sha256") == pin.get("sha256"),
            "{} is not bound to its archive member".format(label))
    expected_filename = PROVIDER_POLL_CAPTURE_FILENAMES[role]
    require(Path(path).name == expected_filename
            and Path(member["source_path"]).name == expected_filename,
            "{} filename does not match its role".format(label))
    size = pin.get("bytes")
    require(type(size) is int and 0 <= size <= MAX_EXECUTION_EVIDENCE_BYTES,
            "{} byte count is outside its bound".format(label))
    raw = read_pinned_bytes(pin, label)
    require(len(raw) == size, "{} byte count does not match its pin".format(label))
    if role not in PROVIDER_POLL_JSON_ROLES:
        return raw
    try:
        return queue_module.strict_json_loads(raw)
    except Exception as error:
        raise ValueError("{} is not strict JSON: {}".format(label, error)) from error


def _validate_provider_poll_checksum_closure(
    raw: bytes,
    archive_root: str,
    member_by_path: Mapping[str, Mapping[str, Any]],
    excluded_paths: set[str],
) -> int:
    try:
        lines = raw.decode("ascii").splitlines()
    except UnicodeDecodeError as error:
        raise ValueError("provider poll SHA256SUMS is not ASCII") from error
    sums: dict[str, str] = {}
    for index, line in enumerate(lines):
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9._-]+)", line)
        require(match is not None,
                "provider poll SHA256SUMS line {} is malformed".format(index + 1))
        digest, filename = match.groups()
        require(filename not in sums,
                "provider poll SHA256SUMS repeats a member")
        sums[filename] = digest
    archive_files: dict[str, str] = {}
    prefix = archive_root + "/"
    for path, member in member_by_path.items():
        if not path.startswith(prefix):
            continue
        relative = path[len(prefix):]
        if path in excluded_paths:
            continue
        require(relative.startswith("raw/"),
                "provider poll archive member is outside the raw closure")
        filename = Path(relative).name
        require(filename not in archive_files,
                "provider poll archive repeats a raw filename")
        archive_files[filename] = str(member["sha256"])
    require(set(sums) == set(archive_files),
            "provider poll SHA256SUMS does not cover the exact raw archive members")
    for filename, digest in sums.items():
        require(archive_files[filename] == digest,
                "provider poll SHA256SUMS differs from its archive member")
    return len(sums)


def validate_provider_poll_observations(
    value: Any,
    latest: Mapping[str, Any],
    queue_module: Any,
) -> Sequence[Mapping[str, Any]]:
    """Validate historical provider poll records without inventing endpoint results."""
    require(isinstance(value, dict)
            and set(value) == {"schema", "archive_manifest", "observations"}
            and value.get("schema") == PROVIDER_POLL_OBSERVATIONS_SCHEMA,
            "unsupported provider poll observations schema or fields")
    archive_root, member_by_path = _provider_archive_members(
        value.get("archive_manifest"), queue_module,
        archive_schema=PROVIDER_POLL_ARCHIVE_SCHEMA,
    )
    observations = value.get("observations")
    require(isinstance(observations, list) and 0 < len(observations) <= 100,
            "provider poll observations must contain 1 to 100 rows")
    validated: list[Mapping[str, Any]] = []
    seen_run_attempts: set[tuple[str, int]] = set()
    previous_observed_at: Optional[str] = None
    expected_captures = set(PROVIDER_POLL_CAPTURE_FILENAMES)
    for index, observation in enumerate(observations):
        label = "provider poll observations[{}]".format(index)
        require(isinstance(observation, dict)
                and set(observation) == {
                    "sequence", "observed_at", "scope", "kind",
                    "source_binding", "captures",
                }
                and observation.get("kind") == PROVIDER_POLL_VARIANT,
                "{} has missing, unknown, or unsupported variant fields".format(label))
        sequence = observation.get("sequence")
        require(type(sequence) is int and sequence == index + 1,
                "{}.sequence must be contiguous and start at 1".format(label))
        observed_at = timestamp(observation.get("observed_at"), "{}.observed_at".format(label))
        require(previous_observed_at is None or observed_at >= previous_observed_at,
                "provider poll observation timestamps are out of order")
        previous_observed_at = observed_at
        require(observation.get("scope") == "HISTORICAL",
                "{} must remain HISTORICAL".format(label))
        binding = observation.get("source_binding")
        require(isinstance(binding, dict)
                and set(binding) == {"kind", "branch", "commit_sha", "tree_sha"}
                and binding.get("kind") == "candidate_commit"
                and isinstance(binding.get("branch"), str)
                and binding["branch"].strip(),
                "{}.source_binding has invalid fields".format(label))
        source_commit = sha(binding.get("commit_sha"), "{}.source_binding.commit_sha".format(label))
        source_tree = sha(binding.get("tree_sha"), "{}.source_binding.tree_sha".format(label))
        require(binding["branch"] == latest["candidate_remote"]["branch"],
                "{}.source_binding is not on the candidate branch".format(label))
        captures = observation.get("captures")
        require(isinstance(captures, dict) and set(captures) == expected_captures,
                "{}.captures must pin the exact poll evidence roles".format(label))
        capture_paths = [capture.get("path") for capture in captures.values()
                         if isinstance(capture, dict)]
        require(len(capture_paths) == len(expected_captures)
                and all(isinstance(path, str) for path in capture_paths)
                and len(set(capture_paths)) == len(expected_captures),
                "{}.captures must use a unique path for each role".format(label))
        raw = {
            role: _read_provider_poll_capture(
                captures[role], role, archive_root, member_by_path, queue_module,
            )
            for role in expected_captures
        }
        closure_count = _validate_provider_poll_checksum_closure(
            raw["checksums"], archive_root, member_by_path, {
                captures["checksums"]["path"], captures["summary"]["path"],
            },
        )

        summary = raw["summary"]
        require(isinstance(summary, dict)
                and summary.get("schema") == "termrock-live-pr-observation/v1"
                and summary.get("repository") == REPOSITORY,
                "{}.summary is not a supported repository observation".format(label))
        summary_time = timestamp(
            summary.get("observed_at_utc"), "{}.summary.observed_at_utc".format(label),
        )
        require(summary_time == observed_at,
                "{}.observed_at differs from its pinned observation".format(label))
        pull = summary.get("pull_request")
        require(isinstance(pull, dict)
                and pull.get("number") == 17
                and pull.get("head_ref") == binding["branch"]
                and pull.get("requested_head_sha") == source_commit
                and pull.get("observed_head_sha") == source_commit
                and pull.get("remote_head_ref_sha") == source_commit
                and pull.get("head_commit_tree_sha") == source_tree,
                "{}.summary pull-request identity differs from its source binding".format(label))
        summary_ci = summary.get("current_ci_observation")
        require(isinstance(summary_ci, dict)
                and summary_ci.get("workflow_run_id") is not None,
                "{}.summary current CI observation is malformed".format(label))

        pull_raw = raw["pull_request"]
        pull_head = pull_raw.get("head") if isinstance(pull_raw, dict) else None
        pull_base = pull_raw.get("base") if isinstance(pull_raw, dict) else None
        require(isinstance(pull_head, dict) and isinstance(pull_base, dict)
                and pull_raw.get("number") == 17
                and pull_head.get("ref") == binding["branch"]
                and pull_head.get("sha") == source_commit
                and pull_raw.get("base", {}).get("ref") == pull.get("base_ref")
                and pull_raw.get("base", {}).get("sha") == pull.get("observed_base_sha"),
                "{}.pull_request capture differs from its summary".format(label))
        candidate_ref = raw["candidate_ref"]
        require(isinstance(candidate_ref, dict)
                and candidate_ref.get("ref") == "refs/heads/" + binding["branch"]
                and isinstance(candidate_ref.get("object"), dict)
                and candidate_ref["object"].get("type") == "commit"
                and candidate_ref["object"].get("sha") == source_commit,
                "{}.candidate_ref capture differs from its source binding".format(label))
        commit = raw["commit"]
        commit_body = commit.get("commit") if isinstance(commit, dict) else None
        commit_tree = commit_body.get("tree") if isinstance(commit_body, dict) else None
        require(isinstance(commit, dict) and isinstance(commit_tree, dict)
                and commit.get("sha") == source_commit
                and commit_tree.get("sha") == source_tree,
                "{}.commit capture differs from its source binding".format(label))

        run = raw["run_api"]
        run_view = raw["run_view"]
        require(isinstance(run, dict) and isinstance(run_view, dict),
                "{}.run captures must be JSON objects".format(label))
        run_id = run.get("id")
        run_attempt = run.get("run_attempt")
        require(type(run_id) is int and run_id > 0
                and type(run_attempt) is int and run_attempt > 0,
                "{}.run ID and attempt must be positive integers".format(label))
        run_key = (str(run_id), run_attempt)
        require(run_key not in seen_run_attempts,
                "provider poll observations repeat a run attempt")
        seen_run_attempts.add(run_key)
        run_url = "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id)
        run_api_url = "https://api.github.com/repos/{}/actions/runs/{}".format(REPOSITORY, run_id)
        workflow_id = run.get("workflow_id")
        created_at = timestamp(run.get("created_at"), "{}.run.created_at".format(label))
        require(run.get("url") == run_api_url
                and run.get("html_url") == run_url
                and run.get("head_branch") == binding["branch"]
                and run.get("head_sha") == source_commit
                and run.get("status") == "completed"
                and run.get("conclusion") == "failure"
                and run.get("event") == "push"
                and run.get("path") == CURRENT_CI_WORKFLOW
                and type(workflow_id) is int and workflow_id > 0
                and run_view.get("databaseId") == run_id
                and run_view.get("headBranch") == binding["branch"]
                and run_view.get("headSha") == source_commit
                and run_view.get("status") == run["status"]
                and run_view.get("conclusion") == run["conclusion"]
                and run_view.get("event") == run["event"]
                and run_view.get("createdAt") == created_at
                and run_view.get("workflowName") == "CI"
                and run_view.get("url") == run_url
                and run.get("created_at") == summary_ci.get("created_at")
                and run_id == summary_ci.get("workflow_run_id")
                and run.get("run_number") == summary_ci.get("workflow_run_number")
                and workflow_id == summary_ci.get("workflow_id")
                and summary_ci.get("workflow_name") == run_view.get("workflowName")
                and run.get("event") == summary_ci.get("event")
                and run.get("head_sha") == summary_ci.get("head_sha")
                and run.get("status") == summary_ci.get("status")
                and run.get("conclusion") == summary_ci.get("conclusion")
                and created_at <= observed_at,
                "{}.run records do not reconcile with source, poll, and run-view".format(label))
        timestamp(run_view.get("updatedAt"), "{}.run_view.updatedAt".format(label))
        visible_jobs = run_view.get("jobs")
        require(isinstance(visible_jobs, list)
                and visible_jobs == []
                and summary_ci.get("jobs_visible_in_gh_run_view") == len(visible_jobs),
                "{}.run_view job visibility differs from its summary".format(label))

        jobs_error = raw["jobs_api"]
        require(isinstance(jobs_error, dict)
                and set(jobs_error) == {"message", "documentation_url", "status"}
                and jobs_error.get("message") == "Not Found"
                and jobs_error.get("status") == "404"
                and isinstance(jobs_error.get("documentation_url"), str)
                and jobs_error["documentation_url"].startswith("https://docs.github.com/rest")
                and b"Not Found (HTTP 404)" in raw["jobs_api_stderr"]
                and summary_ci.get("jobs_api") == "HTTP 404 Not Found",
                "{}.jobs_api must remain an API error, not a zero-job result".format(label))
        artifacts = raw["artifacts_api"]
        require(isinstance(artifacts, dict)
                and set(artifacts) == {"total_count", "artifacts"}
                and type(artifacts.get("total_count")) is int
                and artifacts["total_count"] == len(artifacts.get("artifacts", []))
                and artifacts["total_count"] == 0
                and summary_ci.get("artifacts_total_count") == artifacts["total_count"],
                "{}.artifacts capture differs from its summary".format(label))
        require(raw["run_log_stdout"] == b""
                and raw["run_log_stderr"].strip() == b"failed to get run log: log not found"
                and summary_ci.get("logs") == "not found; no log body returned"
                and summary_ci.get("provider_failure_cause") == "NOT_EXPOSED",
                "{}.log capture must preserve unavailable body and unknown cause".format(label))

        checks = raw["dco_checks"]
        require(isinstance(checks, dict)
                and type(checks.get("total_count")) is int
                and isinstance(checks.get("check_runs"), list)
                and checks["total_count"] == len(checks["check_runs"]),
                "{}.dco_checks capture is malformed".format(label))
        dco_checks = [
            check for check in checks["check_runs"]
            if isinstance(check, dict) and check.get("name") == "DCO"
        ]
        require(len(dco_checks) == 1
                and dco_checks[0].get("head_sha") == source_commit
                and dco_checks[0].get("status") == "completed"
                and dco_checks[0].get("conclusion") == "success",
                "{}.DCO result is missing, duplicated, or not bound to this source".format(label))
        summary_checks = summary.get("checks")
        require(isinstance(summary_checks, dict)
                and summary_checks.get("dco") == "PASS"
                and summary_checks.get("check_runs_count") == checks["total_count"],
                "{}.summary DCO observation differs from its raw check capture".format(label))

        raw_closure = summary.get("raw_closure")
        checksum_capture = captures["checksums"]
        require(isinstance(raw_closure, dict)
                and set(raw_closure) == {"path", "sha256", "entries", "entry_scope"}
                and raw_closure.get("path") == "SHA256SUMS"
                and raw_closure.get("sha256") == checksum_capture.get("sha256")
                and raw_closure.get("entries") == closure_count
                and raw_closure.get("entry_scope") == "Every file under raw/; checksum file itself is excluded.",
                "{}.summary raw closure differs from its pinned manifest".format(label))
        validated.append({
            "_provider_variant": PROVIDER_POLL_VARIANT,
            "sequence": sequence,
            "observed_at": observed_at,
            "scope": "HISTORICAL",
            "source_binding": binding,
            "run_id": str(run_id),
            "run_attempt": run_attempt,
            "run_url": run_url,
            "event": run["event"],
            "status": run["status"],
            "conclusion": run["conclusion"],
            "cli_visible_job_count": len(visible_jobs),
            "jobs_api_error_status_field": jobs_error["status"],
            "artifact_count": artifacts["total_count"],
            "log_http_status": "NOT_CAPTURED",
            "log_availability": "NOT_FOUND",
            "provider_failure_cause": "NOT_EXPOSED",
            "dco": {
                "status": dco_checks[0]["status"],
                "conclusion": dco_checks[0]["conclusion"],
            },
            "archive_manifest": value["archive_manifest"],
            "captures": captures,
        })
    return validated


def validate_provider_observations(
    value: Any, latest: Mapping[str, Any], queue_module: Any,
) -> Sequence[Mapping[str, Any]]:
    """Validate ordered provider run captures without treating them as product tests."""
    if value is None:
        return []
    if isinstance(value, dict) and value.get("schema") == PROVIDER_POLL_OBSERVATIONS_SCHEMA:
        return validate_provider_poll_observations(value, latest, queue_module)
    require(isinstance(value, dict)
            and set(value) == {"schema", "archive_manifest", "observations"}
            and value.get("schema") == PROVIDER_OBSERVATIONS_SCHEMA,
            "unsupported provider observations schema or fields")
    archive_root, member_by_path = _provider_archive_members(
        value.get("archive_manifest"), queue_module,
    )
    observations = value.get("observations")
    require(isinstance(observations, list) and 0 < len(observations) <= 100,
            "provider observations must contain 1 to 100 rows")
    validated: list[Mapping[str, Any]] = []
    seen_run_attempts: set[tuple[str, int]] = set()
    previous_observed_at: Optional[str] = None
    current_indexes: list[int] = []
    allowed_conclusions = {
        "success", "failure", "neutral", "cancelled", "skipped", "timed_out",
        "action_required", "stale",
    }
    for index, observation in enumerate(observations):
        label = "provider observations[{}]".format(index)
        require(isinstance(observation, dict)
                and set(observation) == {
                    "sequence", "observed_at", "scope", "source_binding", "captures",
                }, "{} has missing or unknown fields".format(label))
        sequence = observation.get("sequence")
        require(type(sequence) is int and sequence == index + 1,
                "{}.sequence must be contiguous and start at 1".format(label))
        observed_at = timestamp(observation.get("observed_at"), "{}.observed_at".format(label))
        require(previous_observed_at is None or observed_at >= previous_observed_at,
                "provider observation timestamps are out of order")
        previous_observed_at = observed_at
        scope = observation.get("scope")
        require(scope in {"CURRENT", "HISTORICAL"},
                "{}.scope must be CURRENT or HISTORICAL".format(label))
        if scope == "CURRENT":
            current_indexes.append(index)
        binding = observation.get("source_binding")
        require(isinstance(binding, dict)
                and set(binding) == {"kind", "branch", "commit_sha", "tree_sha"}
                and binding.get("kind") == "candidate_commit"
                and isinstance(binding.get("branch"), str)
                and binding["branch"].strip(),
                "{}.source_binding has invalid fields".format(label))
        source_commit = sha(binding.get("commit_sha"), "{}.source_binding.commit_sha".format(label))
        source_tree = sha(binding.get("tree_sha"), "{}.source_binding.tree_sha".format(label))
        captures = observation.get("captures")
        require(isinstance(captures, dict) and set(captures) == PROVIDER_CAPTURE_ROLES,
                "{}.captures must pin the exact provider inputs".format(label))
        raw = {
            role: _read_provider_capture(
                captures[role], role, archive_root, member_by_path, queue_module,
            ) for role in PROVIDER_CAPTURE_ROLES
        }
        run = raw["run"]
        run_id = run.get("id")
        require(str(run_id).isdigit(), "{}.run ID must be numeric".format(label))
        run_id = str(run_id)
        run_attempt = run.get("run_attempt")
        require(type(run_attempt) is int and run_attempt > 0,
                "{}.run attempt must be a positive integer".format(label))
        run_key = (run_id, run_attempt)
        require(run_key not in seen_run_attempts,
                "provider observations repeat a run attempt")
        seen_run_attempts.add(run_key)
        workflow = raw["workflow"]
        workflow_id = run.get("workflow_id")
        require(type(workflow_id) is int and workflow_id > 0
                and workflow.get("id") == workflow_id,
                "{}.workflow ID does not match the captured run".format(label))
        workflow_path = workflow.get("path")
        require(isinstance(workflow_path, str)
                and workflow_path == CURRENT_CI_WORKFLOW
                and run.get("name") == workflow_path,
                "{}.workflow path does not match the captured run".format(label))
        require(workflow.get("url") ==
                "https://api.github.com/repos/{}/actions/workflows/{}".format(
                    REPOSITORY, workflow_id,
                ) and isinstance(workflow.get("state"), str),
                "{}.workflow API identity is malformed".format(label))
        require(run.get("head_branch") == binding["branch"]
                and run.get("head_sha") == source_commit
                and run.get("status") in {"queued", "in_progress", "completed"}
                and (run.get("conclusion") is None
                     or run.get("conclusion") in allowed_conclusions)
                and ((run.get("status") == "completed")
                     == (run.get("conclusion") is not None))
                and isinstance(run.get("event"), str) and run["event"].strip(),
                "{}.run fields do not match its source binding".format(label))
        created_at = timestamp(run.get("created_at"), "{}.run.created_at".format(label))
        require(created_at <= observed_at,
                "{}.observed_at predates its provider run".format(label))
        run_url = "https://github.com/{}/actions/runs/{}".format(REPOSITORY, run_id)
        run_api_url = "https://api.github.com/repos/{}/actions/runs/{}".format(
            REPOSITORY, run_id,
        )
        require(run.get("html_url") == run_url
                and run.get("url") == run_api_url
                and run.get("workflow_url") == workflow.get("url")
                and run.get("path") == workflow_path,
                "{}.run URL does not match its ID".format(label))
        head_commit = run.get("head_commit")
        require(isinstance(head_commit, dict)
                and head_commit.get("id") == source_commit
                and head_commit.get("tree_id") == source_tree,
                "{}.run commit/tree do not match its source binding".format(label))
        check_suite_id = run.get("check_suite_id")
        require(type(check_suite_id) is int and check_suite_id > 0,
                "{}.check_suite_id must be a positive integer".format(label))

        suites = raw["check_suites"]
        require(isinstance(suites.get("check_suites"), list)
                and type(suites.get("total_count")) is int
                and suites["total_count"] == len(suites["check_suites"]),
                "{}.check suites response is malformed".format(label))
        matching_suites = [
            suite for suite in suites["check_suites"]
            if isinstance(suite, dict) and str(suite.get("id")) == str(check_suite_id)
        ]
        require(len(matching_suites) == 1,
                "{}.run check suite is absent or duplicated".format(label))
        suite = matching_suites[0]
        suite_url = "https://api.github.com/repos/{}/check-suites/{}/check-runs".format(
            REPOSITORY, check_suite_id,
        )
        require(suite.get("head_sha") == source_commit
                and suite.get("head_branch") == binding["branch"]
                and suite.get("status") == run["status"]
                and suite.get("conclusion") == run.get("conclusion")
                and type(suite.get("latest_check_runs_count")) is int
                and suite.get("latest_check_runs_count") >= 0
                and suite.get("check_runs_url") == suite_url,
                "{}.check suite does not match its workflow run".format(label))
        suite_runs = raw["suite_check_runs"]
        require(isinstance(suite_runs.get("check_runs"), list)
                and type(suite_runs.get("total_count")) is int
                and suite_runs["total_count"] == len(suite_runs["check_runs"])
                and suite_runs["total_count"] == suite["latest_check_runs_count"],
                "{}.suite check-runs response does not match its suite".format(label))
        suite_run_ids: set[str] = set()
        for check in suite_runs["check_runs"]:
            require(isinstance(check, dict)
                    and str(check.get("id", "")).isdigit()
                    and check.get("head_sha") == source_commit
                    and check.get("status") in {"queued", "in_progress", "completed"}
                    and (check.get("conclusion") is None
                         or check.get("conclusion") in allowed_conclusions),
                    "{}.suite check-run is malformed or has another source".format(label))
            suite_run_ids.add(str(check["id"]))
            check_suite = check.get("check_suite")
            if check_suite is not None:
                require(isinstance(check_suite, dict)
                        and str(check_suite.get("id")) == str(check_suite_id),
                        "{}.suite check-run belongs to another suite".format(label))
        require(len(suite_run_ids) == len(suite_runs["check_runs"]),
                "{}.suite check-runs repeat an ID".format(label))

        jobs = raw["jobs"]
        require(isinstance(jobs.get("jobs"), list)
                and type(jobs.get("total_count")) is int
                and jobs["total_count"] == len(jobs["jobs"]),
                "{}.jobs response is malformed".format(label))
        job_ids: set[str] = set()
        for job in jobs["jobs"]:
            require(isinstance(job, dict) and str(job.get("run_id")) == run_id
                    and str(job.get("id", "")).isdigit()
                    and job.get("head_sha") == source_commit,
                    "{}.job is not bound to its workflow run".format(label))
            job_ids.add(str(job["id"]))
        require(len(job_ids) == len(jobs["jobs"]),
                "{}.jobs response repeats a job ID".format(label))

        artifacts = raw["artifacts"]
        require(isinstance(artifacts.get("artifacts"), list)
                and type(artifacts.get("total_count")) is int
                and artifacts["total_count"] == len(artifacts["artifacts"]),
                "{}.artifacts response is malformed".format(label))
        artifact_ids: set[str] = set()
        for artifact in artifacts["artifacts"]:
            require(isinstance(artifact, dict)
                    and str(artifact.get("id", "")).isdigit()
                    and isinstance(artifact.get("name"), str)
                    and isinstance(artifact.get("workflow_run"), dict)
                    and str(artifact["workflow_run"].get("id")) == run_id
                    and artifact["workflow_run"].get("head_sha") == source_commit,
                    "{}.artifact record is malformed".format(label))
            artifact_ids.add(str(artifact["id"]))
        require(len(artifact_ids) == len(artifacts["artifacts"]),
                "{}.artifacts response repeats an artifact ID".format(label))

        checks = raw["check_runs"]
        require(isinstance(checks.get("check_runs"), list)
                and type(checks.get("total_count")) is int
                and checks["total_count"] == len(checks["check_runs"]),
                "{}.commit check-runs response is malformed".format(label))
        check_ids: set[str] = set()
        dco_checks: list[Mapping[str, Any]] = []
        for check in checks["check_runs"]:
            require(isinstance(check, dict)
                    and str(check.get("id", "")).isdigit()
                    and check.get("head_sha") == source_commit
                    and isinstance(check.get("name"), str)
                    and check.get("status") in {"queued", "in_progress", "completed"}
                    and (check.get("conclusion") is None
                         or check.get("conclusion") in allowed_conclusions),
                    "{}.check-run record is malformed or has another head".format(label))
            check_ids.add(str(check["id"]))
            if check["name"] == "DCO":
                dco_checks.append(check)
        require(len(check_ids) == len(checks["check_runs"])
                and len(dco_checks) <= 1,
                "{}.check-run IDs or DCO observation are duplicated".format(label))
        dco = dco_checks[0] if dco_checks else None

        header_bytes = raw["run_logs_headers"]
        first_line = header_bytes.split(b"\n", 1)[0].rstrip(b"\r")
        status_match = re.fullmatch(rb"HTTP/\d(?:\.\d)?[ \t]+([1-5][0-9]{2})(?:[ \t].*)?", first_line)
        status_lines = [
            line for line in header_bytes.splitlines()
            if re.match(rb"HTTP/\d(?:\.\d)?[ \t]+", line)
        ]
        require(status_match is not None and len(status_lines) == 1,
                "{}.run-log headers lack one HTTP status line".format(label))
        http_status = int(status_match.group(1))
        body_bytes = raw["run_logs_body"]
        require(http_status < 400 or len(body_bytes) > 0,
                "{}.failed run-log response body is missing".format(label))
        try:
            error_body = queue_module.strict_json_loads(body_bytes)
        except Exception:
            error_body = None
        if isinstance(error_body, dict) and error_body.get("status") is not None:
            require(str(error_body.get("status")) == str(http_status),
                    "{}.run-log response body status differs from its HTTP headers".format(label))

        if scope == "CURRENT":
            require(binding["branch"] == latest["candidate_remote"]["branch"]
                    and source_commit == latest["candidate_remote"]["head_sha"],
                    "CURRENT provider observation does not match the latest candidate source")
        validated.append({
            "sequence": sequence,
            "observed_at": observed_at,
            "scope": scope,
            "source_binding": binding,
            "workflow_path": workflow_path,
            "run_id": run_id,
            "run_attempt": run_attempt,
            "run_url": run_url,
            "event": run["event"],
            "status": run["status"],
            "conclusion": run.get("conclusion"),
            "created_at": created_at,
            "job_count": jobs["total_count"],
            "artifact_count": artifacts["total_count"],
            "suite_check_run_count": suite_runs["total_count"],
            "log_http_status": http_status,
            "dco": None if dco is None else {
                "id": str(dco["id"]), "status": dco["status"],
                "conclusion": dco.get("conclusion"),
            },
            "archive_manifest": value["archive_manifest"],
            "captures": captures,
        })
    require(len(current_indexes) <= 1
            and (not current_indexes or current_indexes[0] == len(observations) - 1),
            "the CURRENT provider observation must be the final ordered row")
    return validated


def render_provider_observations(
    observations: Sequence[Mapping[str, Any]],
) -> str:
    if not observations:
        return ""
    if observations[0].get("_provider_variant") == PROVIDER_POLL_VARIANT:
        return render_provider_poll_observations(observations)
    rows = []
    tick = chr(96)
    for item in observations:
        binding = item["source_binding"]
        dco = item["dco"]
        dco_text = (
            "DCO NOT_CAPTURED" if dco is None else
            "DCO check {}: {} / {}".format(dco["id"], dco["status"], dco["conclusion"])
        )
        log_text = "logs HTTP {}".format(item["log_http_status"])
        if item["log_http_status"] >= 400:
            log_text += (
                "; workflow cause NOT_EXPOSED"
                if item["conclusion"] == "failure"
                else "; provider log response unavailable"
            )
        evidence_text = "; ".join(
            "{} {}{}{}".format(role, tick, pin["sha256"][:12], tick)
            for role, pin in sorted(item["captures"].items())
        )
        manifest = item["archive_manifest"]
        rows.append(
            "| {} | {} | {}{}{} / {}{}{} (tree {}{}{}) | [{}]({}) attempt {}; {} / {}; event {}; {} jobs; {} artifacts; {} suite checks | {}; {}; {}; observed {}; archive {} {}{}{}; captures {} |".format(
                item["sequence"], item["scope"], tick, binding["branch"], tick,
                tick, binding["commit_sha"], tick, tick, binding["tree_sha"], tick,
                item["run_id"], item["run_url"], item["run_attempt"], item["status"],
                item["conclusion"] or "NOT_REPORTED", item["event"], item["job_count"],
                item["artifact_count"], item["suite_check_run_count"], log_text, dco_text,
                item["workflow_path"], item["observed_at"], cell(manifest["path"]),
                tick, manifest["sha256"][:12], tick, evidence_text,
            )
        )
    return "\n".join([
        "## Provider workflow observations",
        "",
        "These hash-pinned provider records describe workflow metadata and log access. They are not product-test receipts. The provider response recorded zero jobs. An HTTP error while fetching logs does not establish the workflow failure cause. This section does not promote product readiness.",
        "",
        "| # | Scope | Source binding | Run | Provider and log observation |",
        "| --- | --- | --- | --- | --- |",
        *rows,
        "",
    ])


def render_provider_poll_observations(
    observations: Sequence[Mapping[str, Any]],
) -> str:
    rows = []
    tick = chr(96)
    for item in observations:
        binding = item["source_binding"]
        dco_text = "DCO {} / {}".format(
            item["dco"]["status"], item["dco"]["conclusion"],
        )
        evidence_text = "; ".join(
            "{} {}{}{}".format(role, tick, pin["sha256"][:12], tick)
            for role, pin in sorted(item["captures"].items())
        )
        manifest = item["archive_manifest"]
        rows.append(
            "| {} | HISTORICAL | {}{}{} / {}{}{} (tree {}{}{}) | "
            "[{}]({}) attempt {}; {} / {}; event {}; CLI-visible jobs {}; "
            "jobs API error status field {}; artifacts {}; logs body {}; "
            "log HTTP status {}; failure cause {}; {} | archive {} "
            "{}{}{}; captures {} |".format(
                item["sequence"], tick, binding["branch"], tick,
                tick, binding["commit_sha"], tick, tick, binding["tree_sha"], tick,
                item["run_id"], item["run_url"], item["run_attempt"],
                item["status"], item["conclusion"], item["event"],
                item["cli_visible_job_count"], item["jobs_api_error_status_field"],
                item["artifact_count"], item["log_availability"],
                item["log_http_status"], item["provider_failure_cause"], dco_text,
                cell(manifest["path"]), tick, manifest["sha256"][:12], tick,
                evidence_text,
            )
        )
    return "\n".join([
        "## Provider workflow poll observations",
        "",
        "These hash-pinned historical polls preserve what the provider and CLI exposed. "
        "A CLI-visible zero-job list is distinct from a jobs API error; the error does "
        "not establish a jobs count. Missing log bodies have no recorded HTTP status, "
        "and the provider failure cause remains NOT_EXPOSED. This section does not "
        "promote current CI or product readiness.",
        "",
        "| # | Scope | Source binding | Run and provider observations | Evidence |",
        "| --- | --- | --- | --- | --- |",
        *rows,
        "",
    ])

def render_current_status_observations(
    observations: Optional[Mapping[str, Any]],
) -> str:
    if observations is None:
        return ""
    pair = observations["fixed_comparison_pair"]
    tag = observations["immutable_visual_tag"]
    branch = observations["branch_tip"]
    gates = {item["head_sha"]: item for item in observations["branch_gate_observations"]}
    current_gate = gates[branch["head_sha"]]
    current_gate_label = (
        "completed successfully"
        if current_gate["dco_conclusion"] == "success"
        else "completed with failure"
    )
    current_gate_status = dco_status_link(
        current_gate_label,
        observations["_validated_dco_result_urls"].get(branch["head_sha"]),
    )
    workflow_rows = "<br>".join(
        "[{}]({}): completed with {} at {}; {} jobs; {} artifacts; workflow source {} bytes; {}".format(
            item["run_id"], item["run_url"], item["conclusion"],
            item["head_sha"], item["job_count"], item["artifact_count"],
            item["workflow_source"]["size_bytes"], cell(item["provider_annotation"]),
        )
        for item in observations["workflow_run_observations"]
    )
    attempt_rows = "<br>".join(
        "{}: {}; capture {}; qualification {}".format(
            item["attempt_id"],
            (
                "Nextest stopped before selecting tests; 0 selected. "
                + cell(item["failure_summary"])
                if item["attempt_id"] == "R12"
                else "1 selected, 0 passed, 1 failed before PTY launch. "
                + cell(item["failure_summary"])
            ),
            item["capture_status"], item["qualification_status"],
        )
        for item in observations["tag_capture_attempts"]
    )
    regression = observations["tag_validator_regression"]
    control = observations["external_tool_control"]
    command_parts = "; ".join(
        "command {}: {}/{} passed, {} failed".format(
            item["index"], item["passed"], item["executed"], item["failed"]
        )
        for item in control["test_commands"]
    )
    candidate_run = observations["candidate_api_deferred_run"]
    failure = candidate_run["failure"]
    archive = observations["_source_evidence_archive"]
    return """\
## Current implementation-branch and external-tool observations

| Lane | Observation | Scope |
| --- | --- | --- |
| Observation bundle | Recorded at {ledger_observed}; evidence freshness remains NOT_RUN. | Source-pinned current observations. The fixed product pair remains candidate {candidate} / reference {reference}. |
| Repository evidence archive | {archive_files} files / {archive_bytes} bytes verified from `{archive_path}`. | Raw source inputs are repository-relative and hash-pinned; the archive does not qualify product execution or acceptance. |
| Implementation branch | {branch} at {head} (tree {tree}), observed {observed}; DCO status: {dco_status}. | Branch and repository-gate metadata. |
| Provider workflow runs | {workflow_rows} | Provider workflow admission results; the captured runs had zero jobs and artifacts, so product execution remains NOT_RUN for those runs. |
| Candidate API/deferred run | {measurement_status}: {passed}/{expected} passed, {failed} failed, {nextest_filtered} filtered; requirement {requirement} case {case} failed in {test_name}: {failure_summary} | Candidate-only source {candidate}, package {package}, binary {binary}, Nextest run {nextest_run_id}; independent review {review_verdict}. Requirement registry: {requirement_status}; paired reference: {paired_reference_status}; paired visual: {paired_visual_status}; acceptance: {acceptance_decision}. Receipt SHA-256 {receipt_sha}; ledger SHA-256 {ledger_sha}; review SHA-256 {review_sha}. |
| Immutable-tag capture attempts | {attempt_rows} | Tag object {tag_object}, peeled commit {tag_commit}; suite f072 {suite} has no source commit/tree recorded. No tag capture or qualification is accepted. |
| Tag source-validator control | Run {regression_run}: 1 selected, 0 passed, 1 failed, {regression_skipped} skipped; isolated environment facts differed (2 versus 0). | Synthetic validator regression only; source inputs unchanged; no product build or capture. |
| Velnor Actions CLI control | Source tree {velnor_tree}; build and version checks PASS; {executed}/{tool_expected} expected tests executed ({tool_passed} passed, {tool_failed} failed, {tool_not_run} NOT_RUN). {command_parts}. | External tool control, NOT_QUALIFIED; source commit is null and no current Git branch is asserted. |

The source record's readiness fields remain NOT_RUN because no readiness acceptance is recorded. The candidate API/deferred test run FAILED, so Refactor / Ready remains NOT_READY. The requirement registry remains NOT_RUN. The paired reference and visual lanes remain NOT_RUN. Acceptance remains NOT_RECORDED. No complete paired run is accepted. Artifact paths and full pins are recorded in `tools/visibility/source-facts.json` under `current_status_observations`. Repository gates and external-tool controls do not qualify a product comparison.
""".format(
        ledger_observed=observations["observed_at"],
        archive_files=archive["file_count"],
        archive_bytes=archive["total_bytes"],
        archive_path=archive["manifest_path"],
        branch=cell(branch["branch"]),
        head=branch["head_sha"],
        tree=branch["tree_sha"],
        observed=branch["captured_at"],
        dco_status=current_gate_status,
        candidate=pair["candidate_commit"],
        reference=pair["reference_commit"],
        workflow_rows=workflow_rows,
        measurement_status=candidate_run["measurement_status"],
        passed=candidate_run["passed"],
        expected=candidate_run["expected"],
        failed=candidate_run["failed"],
        nextest_filtered=candidate_run["nextest_filtered"],
        package=candidate_run["package"],
        binary=candidate_run["binary"],
        nextest_run_id=candidate_run["nextest_run_id"],
        paired_reference_status=candidate_run["paired_reference_status"],
        paired_visual_status=candidate_run["paired_visual_status"],
        acceptance_decision=candidate_run["acceptance_decision"],
        requirement_status=failure["registry_status"],
        receipt_sha=candidate_run["receipt"]["sha256"],
        ledger_sha=candidate_run["ledger"]["sha256"],
        review_sha=candidate_run["review"]["sha256"],
        requirement=failure["requirement_id"],
        case=failure["case_id"],
        test_name=failure["test_name"],
        failure_summary=cell(failure["failure_summary"]),
        review_verdict=candidate_run["review_verdict"],
        tag_object=tag["tag_object_sha"],
        tag_commit=tag["peeled_commit_sha"],
        suite=observations["tag_suite"]["suite_sha256"],
        attempt_rows=attempt_rows,
        regression_run=regression["nextest_run_id"],
        regression_skipped=regression["skipped"],
        velnor_tree=control["source_tree"],
        executed=control["executed_tests"],
        tool_expected=control["expected_tests"],
        tool_passed=control["passed_tests"],
        tool_failed=control["failed_tests"],
        tool_not_run=control["not_run_tests"],
        command_parts=command_parts,
    )


def validate_execution_attempt_history(
    value: Any, latest: Mapping[str, Any], authority: Mapping[str, Any],
    queue_module: Any,
) -> Sequence[Mapping[str, Any]]:
    """Validate ordered, source-bound attempt rows without changing readiness."""
    if value is None:
        return []
    require(isinstance(value, dict)
            and set(value) == {"schema", "attempts"}
            and value.get("schema") == EXECUTION_ATTEMPT_HISTORY_SCHEMA,
            "unsupported execution attempt-history schema or fields")
    attempts = value.get("attempts")
    require(isinstance(attempts, list) and 0 < len(attempts) <= 100,
            "execution attempt history must contain 1 to 100 records")

    validated: list[Mapping[str, Any]] = []
    seen_ids: set[str] = set()
    previous_time: Optional[str] = None
    for index, attempt in enumerate(attempts):
        label = "execution_attempt_history.attempts[{}]".format(index)
        require(isinstance(attempt, dict), "{} must be an object".format(label))
        lane = attempt.get("lane")
        require(lane in EXECUTION_ATTEMPT_LANES,
                "{}.lane is unsupported".format(label))
        fields = {
            "sequence", "attempt_id", "recorded_at", "lane", "scope",
            "source_binding", "counts", "states", "evidence",
        }
        if lane == "ci_provider":
            fields.add("diagnostic")
        require(set(attempt) == fields,
                "{} has missing or unknown fields".format(label))

        sequence = attempt.get("sequence")
        require(type(sequence) is int and sequence == index + 1,
                "{}.sequence must be contiguous and start at 1".format(label))
        attempt_id = attempt.get("attempt_id")
        require(isinstance(attempt_id, str)
                and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}", attempt_id)
                is not None,
                "{}.attempt_id is malformed".format(label))
        require(attempt_id not in seen_ids,
                "execution attempt history repeats an attempt ID")
        seen_ids.add(attempt_id)
        recorded_at = timestamp(attempt.get("recorded_at"), "{}.recorded_at".format(label))
        require(previous_time is None or recorded_at >= previous_time,
                "execution attempt history timestamps are out of order")
        previous_time = recorded_at

        scope = attempt.get("scope")
        require(scope in {"CURRENT", "HISTORICAL"},
                "{}.scope must be CURRENT or HISTORICAL".format(label))
        binding = attempt.get("source_binding")
        require(isinstance(binding, dict)
                and binding.get("kind") == EXECUTION_ATTEMPT_LANES[lane],
                "{}.source_binding does not match its typed lane".format(label))
        kind = binding["kind"]
        if kind == "candidate_commit":
            require(set(binding) == {"kind", "commit_sha"},
                    "{}.source_binding has invalid candidate-commit fields".format(label))
            commit = sha(binding.get("commit_sha"), "{}.source_binding.commit_sha".format(label))
            is_current_source = commit == latest["candidate_remote"]["head_sha"]
        elif kind == "candidate_reference_pair":
            require(set(binding) == {"kind", "candidate_commit", "reference_commit"},
                    "{}.source_binding has invalid source-pair fields".format(label))
            candidate_commit = sha(
                binding.get("candidate_commit"), "{}.source_binding.candidate_commit".format(label),
            )
            reference_commit = sha(
                binding.get("reference_commit"), "{}.source_binding.reference_commit".format(label),
            )
            is_current_source = (
                candidate_commit == latest["candidate_remote"]["head_sha"]
                and reference_commit == latest["reference_remote"]["head_sha"]
            )
        elif kind == "immutable_visual_tag":
            require(set(binding) == {"kind", "tag_object_sha", "peeled_commit_sha"},
                    "{}.source_binding has invalid visual-tag fields".format(label))
            require(sha(binding.get("tag_object_sha"),
                        "{}.source_binding.tag_object_sha".format(label))
                    == authority["tag_object_sha"]
                    and sha(binding.get("peeled_commit_sha"),
                            "{}.source_binding.peeled_commit_sha".format(label))
                    == authority["commit_sha"],
                    "{}.source_binding differs from the immutable visual tag".format(label))
            is_current_source = True
        else:
            require(set(binding) == {
                "kind", "commit_sha", "tree_sha", "source_sha256",
            }, "{}.source_binding has invalid tool-source fields".format(label))
            commit = sha(binding.get("commit_sha"), "{}.source_binding.commit_sha".format(label))
            sha(binding.get("tree_sha"), "{}.source_binding.tree_sha".format(label))
            sha256(binding.get("source_sha256"), "{}.source_binding.source_sha256".format(label))
            is_current_source = commit == latest["candidate_remote"]["head_sha"]
        require(scope != "CURRENT" or is_current_source,
                "{}.scope cannot label a stale or unrelated source as CURRENT".format(label))

        counts = attempt.get("counts")
        require(isinstance(counts, dict) and set(counts) == set(EXECUTION_ATTEMPT_COUNT_FIELDS),
                "{}.counts has missing or unknown fields".format(label))
        for name in EXECUTION_ATTEMPT_COUNT_FIELDS:
            require(type(counts.get(name)) is int and counts[name] >= 0,
                    "{}.counts.{} must be a nonnegative integer".format(label, name))
        require(counts["inventory"] == counts["selected"] + counts["filtered"]
                and counts["selected"] == counts["started"] + counts["incomplete"]
                and counts["started"] == counts["passed"] + counts["failed"],
                "{}.counts do not reconcile inventory, selection, and terminal results"
                .format(label))

        states = attempt.get("states")
        require(isinstance(states, dict) and set(states) == set(EXECUTION_ATTEMPT_STATE_FIELDS),
                "{}.states has missing or unknown fields".format(label))
        for name, allowed in EXECUTION_ATTEMPT_STATE_FIELDS.items():
            require(states.get(name) in allowed,
                    "{}.states.{} is unsupported".format(label, name))
        if counts["selected"] == 0:
            expected_test_state = "NOT_RUN"
        elif counts["failed"]:
            expected_test_state = "FAIL"
        elif counts["incomplete"]:
            expected_test_state = "INCOMPLETE"
        else:
            expected_test_state = "PASS"
        require(states["test"] == expected_test_state,
                "{}.states.test does not match the reconciled counts".format(label))
        require(states["child"] != "PASS"
                or (counts["failed"] == 0 and counts["incomplete"] == 0),
                "{}.states.child cannot pass with failed or incomplete selected tests"
                .format(label))
        require(states["product_qualification"] != "PASS",
                "execution-attempt history cannot qualify a product result")

        evidence = attempt.get("evidence")
        require(isinstance(evidence, dict) and 0 < len(evidence) <= 16,
                "{}.evidence must contain 1 to 16 pinned files".format(label))
        decoded: dict[str, Any] = {}
        for role, pin in evidence.items():
            require(isinstance(role, str)
                    and re.fullmatch(r"[a-z][a-z0-9_]{0,63}", role) is not None,
                    "{}.evidence has a malformed role".format(label))
            require(isinstance(pin, dict)
                    and set(pin) == {"path", "sha256", "format"},
                    "{}.evidence.{} pin has missing or unknown fields".format(label, role))
            format_name = pin.get("format")
            require(format_name in {"json", "utf8", "bytes"},
                    "{}.evidence.{} format is unsupported".format(label, role))
            raw = read_pinned_bytes(pin, "{}.evidence.{}".format(label, role))
            if format_name == "json":
                try:
                    decoded[role] = queue_module.strict_json_loads(raw)
                except Exception as error:
                    raise ValueError(
                        "{}.evidence.{} is not strict JSON: {}".format(label, role, error)
                    ) from error
                require(isinstance(decoded[role], (dict, list)),
                        "{}.evidence.{} JSON must be an object or array".format(label, role))
            elif format_name == "utf8":
                try:
                    decoded[role] = raw.decode("utf-8")
                except UnicodeDecodeError as error:
                    raise ValueError(
                        "{}.evidence.{} is not UTF-8".format(label, role)
                    ) from error
            else:
                decoded[role] = raw

        if lane == "ci_provider":
            required_roles = {
                "run_detail", "failed_suite", "failed_suite_runs", "jobs",
                "artifacts", "log_cli_output",
            }
            require(set(evidence) == required_roles
                    and all(evidence[name]["format"] == "json"
                            for name in required_roles - {"log_cli_output"})
                    and evidence["log_cli_output"]["format"] == "utf8",
                    "{}.evidence does not contain the exact CI diagnostic inputs".format(label))
            run = decoded["run_detail"]
            suite = decoded["failed_suite"]
            suite_runs = decoded["failed_suite_runs"]
            jobs = decoded["jobs"]
            artifacts = decoded["artifacts"]
            require(isinstance(run, dict)
                    and str(run.get("id", "")).isdigit()
                    and run.get("name") == CURRENT_CI_WORKFLOW
                    and run.get("head_branch") == "termrock-implementation"
                    and run.get("head_sha") == binding["commit_sha"]
                    and run.get("status") == "completed"
                    and run.get("conclusion") == "failure"
                    and str(run.get("check_suite_id", "")).isdigit()
                    and attempt_id == "github-ci-{}".format(run["id"]),
                    "{}.evidence.run_detail does not bind a failed CI run to its source"
                    .format(label))
            require(isinstance(suite, dict)
                    and str(suite.get("id", "")) == str(run["check_suite_id"])
                    and suite.get("head_sha") == binding["commit_sha"]
                    and suite.get("head_branch") == "termrock-implementation"
                    and suite.get("status") == "completed"
                    and suite.get("conclusion") == "failure"
                    and type(suite.get("latest_check_runs_count")) is int
                    and suite["latest_check_runs_count"] == 0
                    and suite.get("check_runs_url") ==
                    "https://api.github.com/repos/{}/check-suites/{}/check-runs".format(
                        REPOSITORY, suite["id"],
                    ),
                    "{}.evidence.failed_suite does not match the failed run".format(label))
            require(isinstance(suite_runs, dict)
                    and type(suite_runs.get("total_count")) is int
                    and suite_runs.get("total_count") == 0
                    and suite_runs.get("check_runs") == [],
                    "{}.evidence.failed_suite_runs must prove zero suite check-runs"
                    .format(label))
            require(isinstance(jobs, dict)
                    and type(jobs.get("total_count")) is int
                    and jobs.get("total_count") == 0 and jobs.get("jobs") == []
                    and isinstance(artifacts, dict)
                    and type(artifacts.get("total_count")) is int
                    and artifacts.get("total_count") == 0
                    and artifacts.get("artifacts") == [],
                    "{}.evidence must prove zero CI jobs and artifacts".format(label))
            require(counts == {
                "inventory": 0, "selected": 0, "started": 0, "passed": 0,
                "failed": 0, "filtered": 0, "incomplete": 0,
            } and states == {
                "test": "NOT_RUN", "child": "NOT_RUN", "wrapper": "NOT_APPLICABLE",
                "collector": "INCOMPLETE", "postflight": "NOT_APPLICABLE",
                "cleanup": "NOT_APPLICABLE", "product_qualification": "BLOCKED",
                "capture": "INCOMPLETE", "admission": "NOT_RUN",
            }, "{}.CI provider run must remain a non-product incomplete observation".format(label))
            diagnostic = attempt.get("diagnostic")
            require(isinstance(diagnostic, dict)
                    and set(diagnostic) == {
                        "status", "log_fetch_outcome", "http_status", "log_cli_exit_code",
                    }
                    and diagnostic.get("status") == "NOT_EXPOSED"
                    and diagnostic.get("log_fetch_outcome") == "CLI_REPORTED_LOG_NOT_FOUND"
                    and diagnostic.get("http_status") == "NOT_CAPTURED"
                    and type(diagnostic.get("log_cli_exit_code")) is int
                    and diagnostic["log_cli_exit_code"] == 1,
                    "{}.diagnostic must preserve the unavailable cause and HTTP status"
                    .format(label))
            require("failed to get run log: log not found" in decoded["log_cli_output"],
                    "{}.evidence.log_cli_output does not match the recorded CLI result"
                    .format(label))
        else:
            require("result_record" in evidence and evidence["result_record"]["format"] == "json",
                    "{}.evidence.result_record JSON is required for this typed lane".format(label))
            result_record = decoded["result_record"]
            require(isinstance(result_record, dict)
                    and set(result_record) == {
                        "schema", "attempt_id", "lane", "source_binding", "counts", "states",
                    }
                    and result_record.get("schema") == "termrock-status-attempt-result/v1"
                    and result_record.get("attempt_id") == attempt_id
                    and result_record.get("lane") == lane
                    and result_record.get("source_binding") == binding
                    and result_record.get("counts") == counts
                    and result_record.get("states") == states,
                    "{}.evidence.result_record does not match its typed attempt row".format(label))

        validated.append({
            "sequence": sequence,
            "attempt_id": attempt_id,
            "recorded_at": recorded_at,
            "lane": lane,
            "scope": scope,
            "source_binding": binding,
            "counts": counts,
            "states": states,
            "evidence": evidence,
            "diagnostic": attempt.get("diagnostic"),
        })
    return validated


def render_execution_attempt_history(
    attempts: Sequence[Mapping[str, Any]],
) -> str:
    if not attempts:
        return ""
    rows = []
    for attempt in attempts:
        counts = attempt["counts"]
        states = attempt["states"]
        if counts["selected"] == 0:
            count_summary = "NOT_RUN"
        else:
            count_summary = (
                "{}/{} selected; {} started; {} passed; {} failed; {} filtered; {} incomplete"
            ).format(
                counts["selected"], counts["inventory"], counts["started"],
                counts["passed"], counts["failed"], counts["filtered"], counts["incomplete"],
            )
        binding = attempt["source_binding"]
        source = " / ".join(
            "`{}`".format(cell(value))
            for key, value in binding.items() if key != "kind"
        )
        evidence = "; ".join(
            "{} `{}`".format(cell(role), pin["sha256"][:12])
            for role, pin in sorted(attempt["evidence"].items())
        )
        diagnostic = attempt.get("diagnostic")
        diagnostic_summary = (
            "diagnostic NOT_EXPOSED; log CLI {} (exit {}); HTTP status {}".format(
                diagnostic["log_fetch_outcome"], diagnostic["log_cli_exit_code"],
                diagnostic["http_status"],
            ) if diagnostic is not None else "not applicable"
        )
        rows.append(
            "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |"
            .format(
                attempt["sequence"], cell(attempt["attempt_id"]), cell(attempt["lane"]),
                attempt["scope"], source, count_summary, states["test"], states["child"],
                states["wrapper"], states["collector"], states["postflight"],
                states["cleanup"], states["product_qualification"], states["capture"],
                states["admission"], "{}; {}; recorded {}".format(
                    diagnostic_summary, evidence, attempt["recorded_at"],
                ),
            )
        )
    return "\n".join([
        "## Execution attempt history",
        "",
        "These source-bound attempts preserve test, wrapper, collection, postflight, and cleanup outcomes separately. They are evidence history only; they do not promote product readiness, qualify a product result, or admit test data.",
        "",
        "| # | Attempt | Lane | Source scope | Source binding | Counts | Test | Child | Wrapper | Collector | Postflight | Cleanup | Product qualification | Capture | Admission | Evidence and diagnostics |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
        *rows,
        "",
    ])


def _product_archive_path(value: Any, label: str) -> str:
    require(isinstance(value, str) and value,
            "{}.path must be nonempty text".format(label))
    require("\\" not in value,
            "{}.path must use normalized repository separators".format(label))
    path = Path(value)
    require(not path.is_absolute() and path.as_posix() == value
            and bool(path.parts)
            and all(part not in {"", ".", ".."} for part in path.parts),
            "{}.path must be a normalized repository-relative path".format(label))
    require(value.startswith(PRODUCT_PHASE_ARCHIVE_PREFIX),
            "{}.path must remain under the reports evidence root".format(label))
    return value


def _product_phase_status(layers: Mapping[str, Any]) -> str:
    statuses = [layers[name]["status"] for name in PRODUCT_PHASE_DISPLAY_NAMES]
    if "FAIL" in statuses:
        return "NOT_READY"
    if "BLOCKED" in statuses:
        return "NOT_READY"
    if "NOT_RUN" in statuses:
        return "NOT_RUN"
    if all(status == "NOT_APPLICABLE" for status in statuses):
        return "NOT_APPLICABLE"
    return "READY"


def validate_product_phase_observations(
    value: Any,
    queue_module: Any,
) -> Sequence[Mapping[str, Any]]:
    """Validate a bounded historical phase view from the immutable reports archive."""
    if value is None:
        return []
    require(isinstance(value, dict)
            and set(value) == {
                "schema", "scope", "source_pair", "archive_manifest", "commands",
            }
            and value.get("schema") == PRODUCT_PHASE_OBSERVATIONS_SCHEMA
            and value.get("scope") == "HISTORICAL",
            "unsupported product phase observations schema, scope, or fields")

    pair = value.get("source_pair")
    require(isinstance(pair, dict)
            and set(pair) == {"candidate_commit", "reference_commit"},
            "product phase source_pair has missing or unknown fields")
    candidate_commit = sha(pair.get("candidate_commit"),
                           "product phase candidate commit")
    reference_commit = sha(pair.get("reference_commit"),
                           "product phase reference commit")
    # This row describes an immutable historical pair. Its role records below
    # must match this pair, but later remote tips and current observations may
    # advance or be absent without invalidating the archived result.

    manifest_pin = value.get("archive_manifest")
    require(isinstance(manifest_pin, dict)
            and set(manifest_pin) == {"path", "sha256", "bytes"},
            "product phase archive manifest pin has missing or unknown fields")
    manifest_path = _product_archive_path(
        manifest_pin.get("path"), "product phase archive manifest",
    )
    require(Path(manifest_path).name == "MANIFEST.json",
            "product phase archive manifest must be named MANIFEST.json")
    manifest_size = manifest_pin.get("bytes")
    require(type(manifest_size) is int and 0 < manifest_size <= 1024 * 1024,
            "product phase archive manifest byte count is outside its bound")
    manifest_raw = read_pinned_bytes(manifest_pin, "product phase archive manifest")
    require(len(manifest_raw) == manifest_size,
            "product phase archive manifest byte count does not match its pin")
    try:
        manifest = queue_module.strict_json_loads(manifest_raw)
    except Exception as error:
        raise ValueError("product phase archive manifest is not strict JSON: {}".format(error)) from error
    require(isinstance(manifest, dict)
            and set(manifest) == {
                "schema", "increment", "source_baseline", "files", "file_count",
                "total_bytes", "limits",
            }
            and manifest.get("schema") == PRODUCT_PHASE_ARCHIVE_SCHEMA,
            "unsupported product phase archive manifest schema or fields")
    archive_name = Path(manifest_path).parent.name
    require(isinstance(manifest.get("increment"), str)
            and manifest["increment"] == archive_name,
            "product phase archive manifest increment does not match its directory")
    require(isinstance(manifest.get("source_baseline"), dict),
            "product phase archive source baseline must be an object")
    limits = manifest.get("limits")
    require(isinstance(limits, list) and 0 < len(limits) <= 32
            and all(isinstance(item, str) and item.strip() for item in limits),
            "product phase archive limits must be a bounded list of descriptions")
    members = manifest.get("files")
    require(isinstance(members, list)
            and 0 < len(members) <= PRODUCT_PHASE_ARCHIVE_MAX_MEMBERS,
            "product phase archive member count is outside its bound")
    require(type(manifest.get("file_count")) is int
            and manifest["file_count"] == len(members),
            "product phase archive file_count does not match its members")
    total_bytes = 0
    member_by_path: dict[str, Mapping[str, Any]] = {}
    for index, member in enumerate(members):
        label = "product phase archive files[{}]".format(index)
        require(isinstance(member, dict)
                and set(member) == {"path", "source_path", "bytes", "sha256", "description"},
                "{} has missing or unknown fields".format(label))
        member_path = _product_archive_path(member.get("path"), label)
        require(member_path.startswith(Path(manifest_path).parent.as_posix() + "/"),
                "{} is outside the pinned archive directory".format(label))
        require(isinstance(member.get("source_path"), str)
                and member["source_path"].strip()
                and isinstance(member.get("description"), str)
                and member["description"].strip(),
                "{} source_path and description are required".format(label))
        member_size = member.get("bytes")
        require(type(member_size) is int and 0 <= member_size <= MAX_EXECUTION_EVIDENCE_BYTES,
                "{}.bytes is outside its bound".format(label))
        member_sha = sha256(member.get("sha256"), "{}.sha256".format(label))
        require(member_path not in member_by_path,
                "product phase archive repeats a member path")
        member_by_path[member_path] = member
        total_bytes += member_size
        require(total_bytes <= PRODUCT_PHASE_ARCHIVE_MAX_BYTES,
                "product phase archive exceeds its aggregate byte bound")
    require(type(manifest.get("total_bytes")) is int
            and manifest["total_bytes"] == total_bytes,
            "product phase archive total_bytes does not match its members")

    commands = value.get("commands")
    require(isinstance(commands, list) and len(commands) == 1,
            "product phase observations must contain exactly one registered command")
    observation = commands[0]
    require(isinstance(observation, dict)
            and set(observation) == {"command_id", "exact_command", "pair_evidence"},
            "product phase command has missing or unknown fields")
    command_id = observation.get("command_id")
    expected_commands = dict(COMMANDS)
    require(command_id == "RUN-03"
            and expected_commands.get(command_id) == observation.get("exact_command"),
            "product phase command differs from the registered RUN-03 command")
    pair_pin = observation.get("pair_evidence")
    require(isinstance(pair_pin, dict)
            and set(pair_pin) == {"path", "sha256", "bytes"},
            "product phase pair evidence pin has missing or unknown fields")
    pair_path = _product_archive_path(pair_pin.get("path"), "product phase pair evidence")
    archive_directory = Path(manifest_path).parent.as_posix()
    require(pair_path.startswith(archive_directory + "/"),
            "product phase pair evidence is outside the pinned archive directory")
    pair_size = pair_pin.get("bytes")
    require(type(pair_size) is int and 0 < pair_size <= MAX_EXECUTION_EVIDENCE_BYTES,
            "product phase pair evidence byte count is outside its bound")
    member = member_by_path.get(pair_path)
    require(member is not None,
            "product phase pair evidence is not listed in the archive manifest")
    require(member["bytes"] == pair_size and member["sha256"] == pair_pin.get("sha256"),
            "product phase pair evidence pin differs from its archive member")
    pair_raw = read_pinned_bytes(pair_pin, "product phase pair evidence")
    require(len(pair_raw) == pair_size,
            "product phase pair evidence byte count does not match its pin")
    try:
        pair_evidence = queue_module.strict_json_loads(pair_raw)
    except Exception as error:
        raise ValueError("product phase pair evidence is not strict JSON: {}".format(error)) from error
    require(isinstance(pair_evidence, dict)
            and set(pair_evidence) == {
                "case_sha256", "compiled_suite_sha256", "receipt", "runs", "schema",
            }
            and pair_evidence.get("schema") == "termrock-e2e/run03-pair-evidence-v1",
            "unsupported RUN-03 pair-evidence schema or fields")
    sha256(pair_evidence.get("case_sha256"), "RUN-03 case digest")
    sha256(pair_evidence.get("compiled_suite_sha256"), "RUN-03 compiled-suite digest")
    require(isinstance(pair_evidence.get("runs"), list)
            and len(pair_evidence["runs"]) <= 32,
            "RUN-03 pair-evidence runs must be a bounded list")
    receipt = pair_evidence.get("receipt")
    require(isinstance(receipt, dict)
            and set(receipt) == {"rows", "schema"}
            and receipt.get("schema") == "termrock-spec/visibility-command-lane-v1",
            "unsupported RUN-03 command-lane receipt schema or fields")
    rows = receipt.get("rows")
    require(isinstance(rows, list) and 0 < len(rows) <= 32,
            "RUN-03 receipt rows must be a bounded nonempty list")
    row_ids = [row.get("id") for row in rows if isinstance(row, dict)]
    require(len(row_ids) == len(rows)
            and all(isinstance(row_id, str) and row_id for row_id in row_ids)
            and len(set(row_ids)) == len(row_ids),
            "RUN-03 receipt row IDs must be unique nonempty text")
    matching_rows = [row for row in rows if row.get("id") == command_id]
    require(len(matching_rows) == 1,
            "RUN-03 evidence must contain exactly one RUN-03 row")
    row = matching_rows[0]
    require(set(row) == {
                "candidate", "direct_executable_diagnostics", "exact_command", "id",
                "reference", "status",
            }
            and row.get("exact_command") == observation["exact_command"]
            and row.get("status") in STATUS_ORDER,
            "RUN-03 row has invalid fields, command, or status")
    require(isinstance(row.get("direct_executable_diagnostics"), list)
            and len(row["direct_executable_diagnostics"]) <= 32,
            "RUN-03 diagnostics must be a bounded list")

    validated_roles: dict[str, Mapping[str, Any]] = {}
    for role, source_commit in (
        ("candidate", candidate_commit), ("reference", reference_commit),
    ):
        record = row.get(role)
        require(isinstance(record, dict)
                and set(record) == {
                    "argv", "cwd", "kind", "layers", "pre_command_executable_sha256",
                    "role", "source_commit", "source_manifest", "source_manifest_sha256",
                },
                "RUN-03 {} role record has missing or unknown fields".format(role))
        require(record.get("role") == role
                and record.get("kind") == "ExactRootCommand"
                and record.get("source_commit") == source_commit
                and record.get("argv") == shlex.split(observation["exact_command"]),
                "RUN-03 {} role does not bind the exact command and source".format(role))
        require(isinstance(record.get("cwd"), str) and record["cwd"]
                and isinstance(record.get("source_manifest"), str)
                and record["source_manifest"],
                "RUN-03 {} role paths are required".format(role))
        sha256(record.get("source_manifest_sha256"),
               "RUN-03 {} source manifest digest".format(role))
        sha256(record.get("pre_command_executable_sha256"),
               "RUN-03 {} executable digest".format(role))
        layers = record.get("layers")
        require(isinstance(layers, dict) and set(layers) == set(PRODUCT_PHASE_NAMES),
                "RUN-03 {} role must contain exactly the nine recorded layers".format(role))
        validated_layers: dict[str, Mapping[str, Any]] = {}
        for name in PRODUCT_PHASE_NAMES:
            layer = layers[name]
            require(isinstance(layer, dict)
                    and set(layer) == {"evidence", "reason", "status"}
                    and layer.get("status") in STATUS_ORDER,
                    "RUN-03 {} layer {} has invalid fields or status".format(role, name))
            require(isinstance(layer.get("reason"), str) and layer["reason"].strip()
                    and isinstance(layer.get("evidence"), list)
                    and len(layer["evidence"]) <= 32
                    and all(isinstance(item, str) for item in layer["evidence"]),
                    "RUN-03 {} layer {} reason or evidence is invalid".format(role, name))
            validated_layers[name] = layer
        validated_roles[role] = validated_layers
    all_statuses = [
        layer["status"] for role in validated_roles.values() for layer in role.values()
    ]
    expected_row_status = (
        "FAIL" if "FAIL" in all_statuses else (
            "BLOCKED" if "BLOCKED" in all_statuses else (
                "NOT_RUN" if "NOT_RUN" in all_statuses else (
                    "NOT_APPLICABLE" if all(
                        status == "NOT_APPLICABLE" for status in all_statuses
                    ) else "PASS"
                )
            )
        )
    )
    require(row["status"] == expected_row_status,
            "RUN-03 row status does not match its recorded layers")
    return [{
        "command_id": command_id,
        "exact_command": observation["exact_command"],
        "source_pair": pair,
        "candidate_layers": validated_roles["candidate"],
        "reference_layers": validated_roles["reference"],
        "candidate_readiness": _product_phase_status(validated_roles["candidate"]),
        "reference_readiness": _product_phase_status(validated_roles["reference"]),
        "pair_status": row["status"],
        "pair_evidence": pair_pin,
    }]


def render_required_command_rows(
    observations: Sequence[Mapping[str, Any]],
) -> str:
    phase_by_command = {item["command_id"]: item for item in observations}
    rows = []
    for run_id, command in COMMANDS:
        phase = phase_by_command.get(run_id)
        if phase is None:
            rows.append(
                "| {} | {} | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |"
                .format(run_id, command)
            )
            continue
        source_pair = phase["source_pair"]
        historical_command = (
            "{} (historical pair: candidate `{}`; reference `{}`)".format(
                command, source_pair["candidate_commit"], source_pair["reference_commit"],
            )
        )
        cells = []
        for dimension in PRODUCT_PHASE_DISPLAY_NAMES:
            cells.append("Reference: {}; Candidate: {}".format(
                phase["reference_layers"][dimension]["status"],
                phase["candidate_layers"][dimension]["status"],
            ))
        cells.append(
            "Current: NOT_RUN; historical required: Reference {}; Candidate {}".format(
                phase["reference_readiness"], phase["candidate_readiness"],
            )
        )
        rows.append("| {} | {} | {} |".format(
            run_id, historical_command, " | ".join(cells),
        ))
    return "\n".join(rows)


def validate_facts(facts: Any) -> Mapping[str, Any]:
    require(isinstance(facts, dict) and facts.get("schema_version") == 2,
            "unsupported source-facts schema; expected version 2")
    require(facts.get("repository") == REPOSITORY, "repository identity changed")
    timestamp(facts.get("observed_at"), "observed_at")

    candidate, reference = facts.get("candidate"), facts.get("reference")
    require(isinstance(candidate, dict) and isinstance(reference, dict),
            "candidate and reference identities are required")
    require(candidate.get("branch") == "termrock-implementation", "wrong candidate branch")
    require(reference.get("branch") == "visual-baseline", "wrong reference branch")
    current = sha(candidate.get("head_sha"), "candidate.head_sha")
    require(sha(candidate.get("local_head_sha"), "candidate.local_head_sha") == current,
            "local candidate identity differs")
    require(sha(candidate.get("remote_head_sha"), "candidate.remote_head_sha") == current,
            "remote candidate identity differs")
    sha(reference.get("head_sha"), "reference.head_sha")

    latest = facts.get("latest_source_observation")
    latest = validate_source_observation(latest, "latest_source_observation")
    remote_candidate, remote_reference = (
        latest["candidate_remote"], latest["reference_remote"]
    )
    if "prior_candidate_remote_observation" in latest:
        prior_candidate = latest["prior_candidate_remote_observation"]
        require(isinstance(prior_candidate, dict), "prior candidate observation is invalid")
        sha(prior_candidate.get("head_sha"), "prior_candidate_remote_observation.head_sha")
        require(prior_candidate.get("observed_at") is None,
                "unrecorded prior observation time must remain null")
        require(isinstance(prior_candidate.get("qualification"), str)
                and prior_candidate["qualification"].strip(),
                "prior candidate observation qualification is required")

    if latest["method"] == "explicit HTTPS refs/heads fetch":
        require("current_ci_observation" in facts,
                "source observation requires current_ci_observation; use null if no capture exists")
    if "current_ci_observation" in facts:
        validate_current_ci_observation(latest, facts["current_ci_observation"])
    validate_source_observation_history(facts, latest)

    local = facts.get("latest_local_checkout_observation")
    require(isinstance(local, dict), "latest local checkout observation is missing")
    timestamp(local.get("observed_at"), "latest_local_checkout_observation.observed_at")
    require(local.get("branch") == candidate["branch"],
            "local checkout branch changed")
    sha(local.get("head_sha"), "latest_local_checkout_observation.head_sha")
    require(isinstance(local.get("signature_status"), str)
            and local["signature_status"] in {"G", "B", "U", "X", "Y", "R", "E", "N"},
            "unknown local signature status")
    require(local.get("signature_command") == "git show -s --format='%G?' HEAD",
            "local signature observation command changed")
    require(local.get("publication_status") in {
        "unpublished_pending_correction", "unpublished", "published", "not_recorded",
    }, "unknown local publication status")
    require(isinstance(local.get("qualification"), str)
            and local["qualification"].strip(),
            "local checkout qualification is required")

    local_dco = facts.get("latest_local_dco_trailer_observation")
    require(isinstance(local_dco, dict), "local DCO trailer observation is missing")
    timestamp(local_dco.get("observed_at"), "latest_local_dco_trailer_observation.observed_at")
    require(sha(local_dco.get("commit_sha"), "latest_local_dco_trailer_observation.commit_sha")
            == local["head_sha"], "local DCO trailer observation is for another commit")
    require(local_dco.get("expected_trailer")
            == "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>",
            "expected local DCO trailer changed")
    require(type(local_dco.get("trailer_present")) is bool,
            "local DCO trailer presence must be boolean")
    parsed_trailers = local_dco.get("parsed_trailers")
    require(isinstance(parsed_trailers, list)
            and all(isinstance(item, str) for item in parsed_trailers),
            "parsed local trailers must be an array of strings")
    expected_trailer_present = any(
        item == local_dco["expected_trailer"] for item in parsed_trailers
    )
    require(local_dco["trailer_present"] == expected_trailer_present,
            "local DCO trailer result does not match parsed trailers")
    require(isinstance(local_dco.get("message_subject"), str)
            and local_dco["message_subject"].strip(),
            "local commit message subject is required")
    require(local_dco.get("inspection_command")
            == "git show -s --format=%B HEAD | git interpret-trailers --parse",
            "local DCO trailer inspection command changed")
    require(isinstance(local_dco.get("qualification"), str)
            and local_dco["qualification"].strip(),
            "local DCO trailer qualification is required")

    e2e_parity = facts.get("current_e2e_subtree_parity_observation")
    if e2e_parity is not None:
        require(isinstance(e2e_parity, dict)
                and e2e_parity.get("schema")
                == "termrock-status-e2e-subtree-parity/v1",
                "unsupported current E2E subtree parity schema")
        timestamp(e2e_parity.get("observed_at"),
                  "current E2E subtree parity observed_at")
        require(sha(e2e_parity.get("candidate_commit"),
                    "current E2E candidate commit")
                == remote_candidate["head_sha"],
                "current E2E subtree parity candidate changed")
        require(sha(e2e_parity.get("reference_commit"),
                    "current E2E reference commit")
                == remote_reference["head_sha"],
                "current E2E subtree parity reference changed")
        require(e2e_parity.get("path") == "crates/termrock-e2e",
                "current E2E subtree path changed")
        candidate_tree = sha(e2e_parity.get("candidate_tree_sha"),
                             "current E2E candidate subtree SHA")
        reference_tree = sha(e2e_parity.get("reference_tree_sha"),
                             "current E2E reference subtree SHA")
        require(candidate_tree == reference_tree,
                "current E2E subtree comparison is not equal but claims parity")
        require(e2e_parity.get("equal") is True,
                "current E2E subtree parity must explicitly record equality")
        require(e2e_parity.get("method")
                == "git rev-parse <commit>:crates/termrock-e2e",
                "current E2E subtree comparison method changed")
        require(e2e_parity.get("raw_api_responses_preserved") is False,
                "current E2E subtree capture preservation state must be explicit")
        require(isinstance(e2e_parity.get("qualification"), str)
                and "not a paired execution" in e2e_parity["qualification"].lower(),
                "current E2E subtree parity must deny execution qualification")

    authority = facts.get("visual_authority")
    require(isinstance(authority, dict), "visual authority is missing")
    require(authority.get("tag") == "visual-baseline"
            and authority.get("tag_object_sha") == TAG_OBJECT
            and authority.get("commit_sha") == TAG_COMMIT,
            "frozen visual tag identity changed")

    ci = facts.get("ci_observation")
    require(isinstance(ci, dict), "CI observation is missing")
    require(isinstance(ci.get("run_id"), str) and ci["run_id"].isdigit(),
            "CI run ID must be numeric text")
    require(isinstance(ci.get("workflow"), str) and ci["workflow"],
            "CI workflow must be text")
    require(ci.get("status") in {"queued", "in_progress", "completed"},
            "unknown CI run status")
    require(ci.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown CI conclusion")
    sha(ci.get("head_sha"), "ci_observation.head_sha")
    timestamp(ci.get("created_at"), "ci_observation.created_at")
    require(type(ci.get("job_count")) is int and ci["job_count"] >= 0,
            "CI job count must be a nonnegative integer")
    require(type(ci.get("artifact_count")) is int and ci["artifact_count"] >= 0,
            "CI artifact count must be a nonnegative integer")
    require(
        ci.get("url")
        == "https://github.com/{}/actions/runs/{}".format(REPOSITORY, ci["run_id"]),
        "CI run URL does not match its ID",
    )
    failure_reason = ci.get("failure_reason")
    if failure_reason is not None:
        require(isinstance(failure_reason, dict), "CI failure reason must be an object")
        require(
            isinstance(failure_reason.get("reason"), str)
            and failure_reason["reason"].strip(),
            "CI failure reason text is required",
        )
        require(
            isinstance(failure_reason.get("source_url"), str)
            and failure_reason["source_url"].startswith("https://"),
            "CI failure reason source URL must be HTTPS",
        )

    dco = facts.get("dco_observation")
    require(isinstance(dco, dict), "DCO observation is missing")
    require(isinstance(dco.get("check_run_id"), str) and dco["check_run_id"].isdigit(),
            "DCO check ID must be numeric text")
    require(isinstance(dco.get("name"), str) and dco["name"], "DCO check name is required")
    require(dco.get("status") in {"queued", "in_progress", "completed"},
            "unknown DCO status")
    require(dco.get("conclusion") in {
        "action_required", "cancelled", "failure", "neutral", "success",
        "skipped", "stale", "timed_out", None,
    }, "unknown DCO conclusion")
    sha(dco.get("head_sha"), "dco_observation.head_sha")
    require(
        type(dco.get("affected_commit_count")) is int and dco["affected_commit_count"] >= 0,
        "DCO finding count must be a nonnegative integer",
    )
    require(
        isinstance(dco.get("details_url"), str)
        and dco["details_url"].startswith("https://"),
        "DCO details URL must be HTTPS",
    )

    ambient = facts.get("ambient_tools")
    require(isinstance(ambient, dict), "ambient tools are required")
    require(all(isinstance(ambient.get(field), str) and ambient[field]
                for field in ("python", "mise_rust", "cargo_nextest",
                              "nextest_command", "host")),
            "ambient tool observations must be nonempty text")
    observation_sources = facts.get("observation_sources")
    require(isinstance(observation_sources, dict), "observation sources are required")
    timestamp(observation_sources.get("observed_at"), "observation_sources.observed_at")
    return facts


def validate_accepted_tasks(records: Any, queue_module: Any) -> Mapping[str, Any]:
    require(isinstance(records, dict), "unsupported accepted-task schema")
    version = records.get("schema_version")
    if type(version) is int and version == 1:
        return queue_module.validate_records(records)
    if type(version) is int and version == 2:
        return queue_module.validate_records_v2(records)
    raise ValueError("unsupported accepted-task schema")


def cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def render_status(
    facts_value: Any,
    tasks_value: Any,
    role: str = "candidate",
    *,
    queue_module: Any,
) -> str:
    require(role in {"candidate", "reference"}, "role must be candidate or reference")
    facts = validate_facts(facts_value)
    archive_manifest = validate_source_evidence_archive(
        facts.get("source_evidence_archive"), queue_module,
        required=facts.get("current_status_observations") is not None,
    )
    history = facts.get("source_observation_history", [])
    historical_evidence_gaps = (
        validate_historical_evidence_gaps(
            facts.get("historical_evidence_gaps"), history,
        )
        if facts.get("source_evidence_archive") is not None
        and "source_observation_history" in facts else None
    )
    tasks = validate_accepted_tasks(tasks_value, queue_module)
    candidate, reference = facts["candidate"], facts["reference"]
    authority, ci, dco = facts["visual_authority"], facts["ci_observation"], facts["dco_observation"]
    ambient = facts["ambient_tools"]
    local_branch = candidate["branch"] if role == "candidate" else reference["branch"]
    local_role = "Candidate" if role == "candidate" else "Reference"
    queue_link = (
        "WORK_QUEUE.md"
        if role == "candidate"
        else "https://github.com/{}/blob/{}/WORK_QUEUE.md".format(
            REPOSITORY, candidate["branch"]
        )
    )
    latest = facts["latest_source_observation"]
    if latest["method"].endswith("local fetched refs verified afterward"):
        latest_source_record = read_pinned_json(
            facts.get("latest_source_observation_evidence"),
            "latest source-observation facts record", queue_module,
        )
        require(isinstance(latest_source_record, dict)
                and latest_source_record.get("latest_source_observation") == latest,
                "latest source observation does not match its pinned facts record")
    local_record = facts["latest_local_checkout_observation"]
    local_dco_record = facts["latest_local_dco_trailer_observation"]
    if "latest_local_checkout_evidence" in facts:
        local_evidence = read_pinned_json(
            facts.get("latest_local_checkout_evidence"),
            "latest local checkout observation", queue_module,
        )
        require(isinstance(local_evidence, dict)
                and local_evidence.get("record_type") == "read_only_local_head_observation"
                and local_evidence.get("observed_at") == local_record["observed_at"]
                and local_evidence.get("head_sha") == local_record["head_sha"]
                and local_evidence.get("tree_sha") == local_record.get("tree_sha")
                and local_evidence.get("signature_status") == local_record["signature_status"]
                and local_evidence.get("publication_status") == local_record["publication_status"]
                and local_evidence.get("head_sha") == local_dco_record["commit_sha"]
                and local_evidence.get("exact_dco_trailer") == local_dco_record["expected_trailer"]
                and local_evidence.get("trailer_present") == local_dco_record["trailer_present"],
                "latest local checkout fields do not match their pinned observation")
    execution_observations = validate_execution_observations(
        facts.get("execution_observations"), latest, queue_module
    )
    historical_execution = validate_historical_execution_observations(
        facts.get("historical_execution_observations"),
        facts.get("source_observation_history", []), queue_module,
    )
    historical_execution_section = "\n\n".join(
        render_execution_observations(item["observations"], historical=item)
        for item in historical_execution
    )
    execution_section = "\n\n".join(
        section for section in (
            render_execution_observations(execution_observations),
            historical_execution_section,
        ) if section
    )
    local = facts["latest_local_checkout_observation"]
    local_history = validate_local_checkout_history(facts.get("local_checkout_history"))
    local_e2e_suite = load_local_e2e_suite_observation(
        facts.get("local_e2e_suite_observation"), local, queue_module, local_history
    )
    status_controls = load_status_report_control_observation(
        facts.get("status_report_control_observation"), queue_module
    )
    current_local_e2e_suite = validate_current_local_e2e_suite_observation(
        facts.get("current_local_e2e_suite_observation"), local
    )
    e2e_subtree_parity_section = render_e2e_subtree_parity_observation(
        facts.get("current_e2e_subtree_parity_observation")
    )
    e2e_subtree_parity_layout = (
        e2e_subtree_parity_section + "\n\n" if e2e_subtree_parity_section else ""
    )
    validation_execution = list(execution_observations)
    validation_execution.extend(
        item
        for record in historical_execution
        for item in record["observations"]
    )
    local_validation_section = render_local_validation_observations(
        local_e2e_suite, status_controls, validation_execution
    )
    local_validation_section = "\n\n".join(section for section in (
        render_current_local_e2e_suite_observation(current_local_e2e_suite),
        local_validation_section,
    ) if section)
    current_ci = validate_current_ci_observation(
        latest, facts.get("current_ci_observation")
    )
    latest_ci_snapshot = validate_latest_ci_snapshot(
        latest, facts.get("latest_ci_snapshot"), queue_module
    )
    current_status_observations = validate_current_status_observations(
        facts.get("current_status_observations"), latest, authority, queue_module
    )
    historical_current_status = validate_historical_current_status_observations(
        facts.get("historical_current_status_observations"), latest, queue_module
    )
    provider_observations = validate_provider_observations(
        facts.get("provider_observations"), latest, queue_module,
    )
    provider_observations_section = render_provider_observations(provider_observations)
    provider_observations_layout = (
        provider_observations_section + "\n\n" if provider_observations_section else ""
    )
    product_phase_observations = validate_product_phase_observations(
        facts.get("product_phase_observations"), queue_module,
    )
    execution_attempt_history = validate_execution_attempt_history(
        facts.get("execution_attempt_history"), latest, authority, queue_module,
    )
    execution_attempt_history_section = render_execution_attempt_history(
        execution_attempt_history
    )
    execution_attempt_history_layout = (
        execution_attempt_history_section + "\n\n"
        if execution_attempt_history_section
        else ""
    )
    current_publication_observation = validate_current_publication_observation(
        facts.get("current_publication_observation"), latest, archive_manifest, queue_module,
    )
    current_publication_section = render_current_publication_observation(
        current_publication_observation
    )
    checklist_revision = (
        current_publication_observation["candidate"]["publication"]["commit"]
        if current_publication_observation is not None
        else candidate["head_sha"]
    )
    if role == "candidate":
        checklist_markdown_link = "[CHECKLIST.md](CHECKLIST.md)"
        checklist_json_link = "[checklist.json](checklist.json)"
        source_readme_link = "[Source facts and observation commands](tools/visibility/README.md)"
    else:
        checklist_markdown_link = (
            "[CHECKLIST.md](https://github.com/{}/blob/{}/CHECKLIST.md)"
            .format(REPOSITORY, checklist_revision)
        )
        checklist_json_link = (
            "[checklist.json](https://github.com/{}/blob/{}/checklist.json)"
            .format(REPOSITORY, checklist_revision)
        )
        source_readme_link = (
            "[Source facts and observation commands](https://github.com/{}/blob/{}/tools/visibility/README.md)"
            .format(REPOSITORY, checklist_revision)
        )
    if current_status_observations is not None:
        require(archive_manifest is not None,
                "current observations require the verified repository evidence archive")
        current_status_observations = dict(current_status_observations)
        current_status_observations["_source_evidence_archive"] = {
            "file_count": archive_manifest["file_count"],
            "total_bytes": archive_manifest["total_bytes"],
            "manifest_path": (STATUS_EVIDENCE_ARCHIVE_ROOT / "MANIFEST.json").as_posix(),
        }
    candidate_api_failure = (
        current_status_observations is not None
        and current_status_observations["candidate_api_deferred_run"]["measurement_status"]
        == "FAILED"
    )
    current_status_observations_section = render_current_status_observations(
        current_status_observations
    )
    current_ci_section = "\n\n".join(
        section for section in (
            render_current_ci_observation(
                current_ci, partial_execution=bool(execution_observations)
            ),
            render_latest_ci_snapshot(latest_ci_snapshot),
        ) if section
    )
    publication = validate_report_publication_observation(
        facts.get("report_publication_observation"), queue_module
    )
    publication_ci = validate_publication_ci_observations(
        facts.get("publication_ci_observations"), queue_module
    )
    # Resolve and validate pinned evidence before rejecting machine-local locators.
    # This preserves field-specific traversal and provenance errors.
    require_repository_relative_source_paths(facts)
    publication_section = render_publication_observations(publication, publication_ci)
    remote_candidate, remote_reference = latest["candidate_remote"], latest["reference_remote"]
    fixed_pair = latest["method"] in {
        "explicit HTTPS refs/heads fetch",
        "read-only GitHub branch API observation; local fetched refs verified afterward",
    }
    if fixed_pair:
        candidate_source_label = "Selected candidate commit (fixed for comparison)"
        reference_source_label = "Selected reference commit (fixed for comparison)"
        source_observed_label = "Pair selected at"
        source_timing_value = (
            "Branch last-update time unknown; commit committer timestamps are metadata only "
            "({} candidate, {} reference)."
        ).format(remote_candidate["commit_committer_at"], remote_reference["commit_committer_at"])
        prior_candidate_description = (
            "Earlier source observations are retained below as history."
        )
        source_statement = (
            "This source pair was fixed for the next comparison attempt. Later branch "
            "observations are separate and do not silently advance this pair."
        )
        source_section_heading = "Fixed source pair and local checkout"
        ci_scope_label = "selected candidate source commit"
        history_section = render_source_observation_history(
            facts.get("source_observation_history", []), historical_evidence_gaps,
        )
    else:
        candidate_source_label = "Candidate remote branch tip"
        reference_source_label = "Reference remote branch tip"
        source_observed_label = "Remote branch tips observed at"
        source_timing_value = (
            "{}; branch last-update time unknown; remote commit committer dates "
            "are metadata only (candidate {}, reference {})."
        ).format(
            latest["observed_at"],
            remote_candidate.get("commit_committer_at", "unknown"),
            remote_reference.get("commit_committer_at", "unknown"),
        )
        prior_candidate = latest.get("prior_candidate_remote_observation", {})
        prior_candidate_description = (
            "{}; its retrieval time was not retained, and this tip is superseded by the "
            "observation above."
        ).format(prior_candidate.get("head_sha", "No earlier candidate tip recorded"))
        source_statement = (
            "The two latest branch tips are separate observations, not a paired execution. "
            "No paired product run is inferred from them."
        )
        source_section_heading = "Latest source and local checkout observations"
        ci_scope_label = "latest observed candidate branch tip"
        history_section = render_source_observation_history(
            facts.get("source_observation_history", []), historical_evidence_gaps,
        )
    local_dco = facts["latest_local_dco_trailer_observation"]
    ci_scope = ("matches" if ci["head_sha"] == remote_candidate["head_sha"]
                else "does not match")
    dco_scope = ("matches" if dco["head_sha"] == remote_candidate["head_sha"]
                 else "does not match")
    signature_note = (
        "Git reported no cryptographic signature" if local["signature_status"] == "N"
        else "Git signature status code {} was recorded".format(local["signature_status"])
    )
    local_dco_status = "present" if local_dco["trailer_present"] else "missing"
    local_dco_trailers = "; ".join(
        cell(item) for item in local_dco["parsed_trailers"]
    ) or "none"
    publication_labels = {
        "unpublished_pending_correction": "Unpublished and pending correction",
        "unpublished": "Unpublished",
        "published": "Published",
        "not_recorded": "NOT_RECORDED",
    }
    publication_status = publication_labels[local["publication_status"]]
    local_qualification = cell(local["qualification"])
    local_publication = (
        local_qualification
        if local["publication_status"] in {"unpublished_pending_correction", "unpublished"}
        else "{}. {}".format(publication_status, local_qualification)
    )
    ci_conclusion = ci["conclusion"] or "unknown"
    failure_reason = ci.get("failure_reason")
    if ci_conclusion == "failure":
        ci_note = (
            "Failure reason: {}; [source]({}).".format(
                cell(failure_reason["reason"]), failure_reason["source_url"]
            )
            if failure_reason is not None
            else "Failure reason is not recorded."
        )
    else:
        ci_note = "Run status: {}.".format(ci["status"])
    dco_conclusion = dco["conclusion"] or "unknown"
    affected_commits = dco["affected_commit_count"]
    if affected_commits == 0:
        dco_findings = "No commits are reported with sign-off problems."
    elif affected_commits == 1:
        dco_findings = "1 commit is reported with sign-off problems."
    else:
        dco_findings = "{} commits are reported with sign-off problems.".format(
            affected_commits
        )
    command_rows = render_required_command_rows(product_phase_observations)
    holla = next(
        (item for item in execution_observations if item["kind"] == "holla_diagnostic"),
        None,
    )
    app_rows_list = []
    for app in APPS:
        if app == "Holla" and holla is not None:
            candidate_counts = holla["role_dimension_counts"]["candidate"]
            reference_counts = holla["role_dimension_counts"]["reference"]
            candidate_visual = summarize_dimension_statuses(candidate_counts.get("visual", {}))
            reference_visual = summarize_dimension_statuses(reference_counts.get("visual", {}))
            candidate_interaction = summarize_dimension_statuses(
                candidate_counts.get("interaction", {})
            )
            reference_interaction = summarize_dimension_statuses(
                reference_counts.get("interaction", {})
            )
            app_rows_list.append(
                "| Holla | {} | {} | {} | {} | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | Partial HELP-HOLLA-004 only; complete app readiness is NOT_READY. |".format(
                    reference_visual, candidate_visual, reference_interaction,
                    candidate_interaction,
                )
            )
        else:
            app_rows_list.append(
                "| {} | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |".format(app)
            )
    app_rows = "\n".join(app_rows_list)

    has_partial_evidence = bool(execution_observations)
    readiness_state = "NOT_READY" if has_partial_evidence else "NOT_RUN"
    refactor_readiness_state = (
        "NOT_READY" if has_partial_evidence or candidate_api_failure else "NOT_RUN"
    )
    deferred_execution = next(
        (item for item in execution_observations if item["kind"] == "deferred_nextest"),
        None,
    )
    holla_status_summary = (
        summarize_status_counts(holla["status_counts"]) if holla is not None
        else "no Holla case result rows"
    )
    deferred_failure_summary = (
        "The deferred subset records {} failed terminal {}.".format(
            deferred_execution["failed"],
            "test" if deferred_execution["failed"] == 1 else "tests",
        )
        if deferred_execution is not None and deferred_execution["failed"]
        else "No failed deferred terminal test is recorded."
    )
    visibility_evidence = (
        "{} Holla result status counts: {}. Required-set coverage is incomplete. "
        "Candidate API and ownership checks remain incomplete.".format(
            deferred_failure_summary, holla_status_summary
        )
        if has_partial_evidence
        else "The shared suite, paired results, and publication checks have not been accepted as a complete current run."
    )
    readiness_evidence = (
        "{} Holla result status counts: {}. The required set is incomplete. "
        "API and ownership checks remain incomplete.".format(
            deferred_failure_summary, holla_status_summary
        )
        if has_partial_evidence
        else (
            "The candidate API/deferred test run FAILED: {} of {} tests passed; {} failed. "
            "The required set and acceptance remain incomplete. The failure was {} "
            "case {} ({}). This candidate-only run does not qualify a paired product result.".format(
                current_status_observations["candidate_api_deferred_run"]["passed"],
                current_status_observations["candidate_api_deferred_run"]["executed"],
                current_status_observations["candidate_api_deferred_run"]["failed"],
                current_status_observations["candidate_api_deferred_run"]["failure"]["requirement_id"],
                current_status_observations["candidate_api_deferred_run"]["failure"]["case_id"],
                current_status_observations["candidate_api_deferred_run"]["failure"]["test_name"],
            )
            if candidate_api_failure
            else "No complete current evidence covers required visual, interaction, API, ownership, and review checks."
        )
    )
    freshness_state = "PARTIAL" if has_partial_evidence else "NOT_RUN"
    freshness_evidence = (
        "Source-bound partial receipts are recorded below; a complete current required-set receipt is unavailable."
        if has_partial_evidence
        else "No validated paired run receipt is available for the measured source pair."
    )
    receipt_statement = (
        "Partial source-bound execution observations appear below. They do not form a complete current required-set receipt or qualify a product pass."
        if has_partial_evidence
        else "No validated paired run receipt is available, so this report cannot show a product pass."
    )
    trend_statement = (
        "No compatible previous complete required-set run is recorded, so trend changes are not measured."
    )
    final_evidence_note = (
        "Partial records are shown with their limits; unmeasured obligations remain NOT_RUN and overall readiness remains NOT_READY."
        if has_partial_evidence
        else (
            "The candidate-only API/deferred run is a measured failure, but no complete paired run receipt exists. "
            "Paired product evidence remains NOT_RUN and acceptance remains NOT_RECORDED; no product pass is shown."
            if candidate_api_failure
            else "If no current run receipt is present, the product result remains NOT_RUN. NOT_RUN is a result status, not a run-receipt status. No historical pass is promoted to current evidence."
        )
    )

    open_tasks = [task for task in tasks["tasks"] if task["state"] != "verified"]
    if open_tasks:
        highest = min(open_tasks, key=lambda task: PRIORITY[task["priority"]])["priority"]
        task_rows = "\n".join(
            "| {} | {} | {} | {} | {} |".format(
                cell(task["work_id"]), cell(task["state"]), cell(task["owner"]),
                cell(task["reviewer"]), cell(task["priority_reason"]),
            )
            for task in sorted(
                (task for task in open_tasks if task["priority"] == highest),
                key=lambda task: task["work_id"],
            )
        )
        task_section = "\n".join(
            [
                "Highest open priority: **{}**, from accepted queue revision {}.".format(
                    highest, tasks["queue_revision"]
                ),
                "",
                "| Work | State | Owner | Reviewer | Priority reason |",
                "| --- | --- | --- | --- | --- |",
                task_rows,
                "",
                "See [WORK_QUEUE.md]({}) for dependencies and claim details.".format(
                    queue_link
                ),
            ]
        )
    else:
        task_section = "No open accepted task is recorded. See [WORK_QUEUE.md]({}).".format(
            queue_link
        )

    return """# Termrock status

| Conclusion | State | Evidence |
| --- | --- | --- |
| Visibility / Complete | {readiness_state} | {visibility_evidence} |
| Refactor / Ready | {refactor_readiness_state} | {readiness_evidence} |
| Reference / Qualified | NOT_RUN | The tag identity is recorded, but no paired reference execution is qualified. |
| Command / Ready | NOT_RUN | No exact root command has current execution evidence. |
| Evidence freshness | {freshness_state} | {freshness_evidence} |

## Required commands

The table reports each metric separately for the reference and candidate. The Ready column is separate from Build, Launch, First frame, Interaction, Exit, Restoration, Visual, and Ownership.
Reference ownership is NOT_APPLICABLE because the reference is not required to use the candidate Termrock architecture.

| ID | Exact root command | Build | Launch | First frame | Interaction | Exit | Restoration | Visual | Ownership | Ready |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
{command_rows}

The status reporter generated this report from measured facts in [source-facts.json](tools/visibility/source-facts.json). A run receipt records execution results tied to an exact source pair and required test set. {receipt_statement} Product requirements remain in {checklist_markdown_link} and {checklist_json_link}.

## Recorded source pair

This is the earlier source pair captured at the timestamp below. The branch tips observed later are listed separately; neither table is product-test evidence.

| Identity | Value |
| --- | --- |
| Local role | {local_role} ({local_branch}) |
| Candidate observed commit (recorded pair) | [{candidate_sha}](https://github.com/{repository}/commit/{candidate_sha}) |
| Reference observed commit (recorded pair) | [{reference_sha}](https://github.com/{repository}/commit/{reference_sha}) |
| Immutable visual tag object | {tag_object} |
| Immutable visual tag commit | [{tag_commit}](https://github.com/{repository}/commit/{tag_commit}) |
| Pair observed at | {recorded_pair_observed_at} |
| Report commit | Set after this report is committed; the generated page cannot include its own commit ID. |
| Shared suite / case-set / expected-generation digests | NOT_RECORDED |

## {source_section_heading}

| Observation | Value |
| --- | --- |
| {candidate_source_label} | [{remote_candidate_sha}](https://github.com/{repository}/commit/{remote_candidate_sha}) |
| {reference_source_label} | [{remote_reference_sha}](https://github.com/{repository}/commit/{remote_reference_sha}) |
| Source observation method | {source_method} |
| {source_observed_label} | {remote_observed_at} |
| Source timestamp meaning | {source_timing_value} |
| Earlier source observations | {prior_candidate_description} |
| Captured local checkout (source facts) | {local_branch_name} at `{local_sha}`; compare this recorded identity with the selected source commit above. |
| Captured local cryptographic commit signature | `{signature_status}` from `{signature_command}` ({signature_note}). |
| Captured local Developer Certificate of Origin (DCO) trailer | Expected line `{expected_trailer}` is {local_dco_status} in commit `{local_dco_sha}`. |
| Parsed local DCO trailers | {local_dco_trailers} |
| Captured local commit subject | {local_dco_subject} |
| Captured local commit observation time | {local_dco_observed_at} |
| Local publication | {local_publication} |

The DCO observations above describe the captured local commit message. They do not report a remote DCO check.

{source_statement}

{history_section}

## Recorded CI and repository checks

| Check | Observation | Scope |
| --- | --- | --- |
| [CI run {ci_id}]({ci_url}) | {ci_conclusion} at {ci_sha}; {ci_jobs} jobs; {ci_artifacts} artifacts. {ci_note} | The run head {ci_scope} the {ci_scope_label}. This recorded workflow result is historical and is not a product test result. |
| [DCO check {dco_id}](https://github.com/{repository}/commit/{dco_sha}/checks) | {dco_conclusion}; {dco_findings} | The check head {dco_scope} the {ci_scope_label}. This is a repository gate, separate from product results; no remote history change is inferred. |

{current_ci_section}

{e2e_subtree_parity_layout}{current_status_observations_section}

{execution_attempt_history_layout}{provider_observations_layout}{current_publication_section}

{publication_section}

Tool versions recorded at {tools_observed_at}: Python {python}, Rust {rust}, and cargo-nextest {nextest} through mise on {host}. This records installed tools only; it does not show that a product check ran.

## Applications

Visual, interaction, API, and ownership remain separate. API and ownership are candidate-only obligations.

| Application | Reference visual | Candidate visual | Reference interaction | Candidate interaction | Reference API | Candidate API | Reference ownership | Candidate ownership | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
{app_rows}

{execution_section}

{local_validation_section}

## Component and coverage inventory

The required shared-case and per-component checkpoint sets are unknown until the case inventory is audited and bound to the common suite. component-ownership.json is a proposal map, not an executed checkpoint set. No per-component result is inferred from imports, checklist text, or historical captures.

{trend_statement}

## Next work

{task_section}

## Evidence navigation

- [Work queue]({queue_link})
- [Implementation PR #17](https://github.com/{repository}/pull/17)
- [Recorded CI run]({ci_url})
- Product requirements: {checklist_markdown_link} and {checklist_json_link}
- {source_readme_link}

{final_evidence_note}
""".format(
        command_rows=command_rows,
        candidate_sha=candidate["head_sha"],
        reference_sha=reference["head_sha"],
        remote_candidate_sha=remote_candidate["head_sha"],
        remote_reference_sha=remote_reference["head_sha"],
        remote_observed_at=latest["observed_at"],
        candidate_source_label=candidate_source_label,
        reference_source_label=reference_source_label,
        source_method=latest["method"],
        source_observed_label=source_observed_label,
        source_timing_value=cell(source_timing_value),
        prior_candidate_description=cell(prior_candidate_description),
        source_statement=source_statement,
        source_section_heading=source_section_heading,
        history_section=history_section,
        local_branch_name=local["branch"],
        local_sha=local["head_sha"],
        signature_status=local["signature_status"],
        signature_command=local["signature_command"],
        signature_note=signature_note,
        expected_trailer=local_dco["expected_trailer"],
        local_dco_status=local_dco_status,
        local_dco_sha=local_dco["commit_sha"],
        local_dco_trailers=local_dco_trailers,
        local_dco_subject=cell(local_dco["message_subject"]),
        local_dco_observed_at=local_dco["observed_at"],
        local_publication=local_publication,
        local_role=local_role,
        local_branch=local_branch,
        queue_link=queue_link,
        repository=REPOSITORY,
        tag_object=authority["tag_object_sha"],
        tag_commit=authority["commit_sha"],
        recorded_pair_observed_at="UNKNOWN",
        tools_observed_at=facts["observation_sources"]["observed_at"],
        ci_id=ci["run_id"],
        ci_url=ci["url"],
        ci_conclusion=ci_conclusion,
        ci_note=ci_note,
        ci_sha=ci["head_sha"],
        ci_jobs=ci["job_count"],
        ci_artifacts=ci["artifact_count"],
        ci_scope=ci_scope,
        ci_scope_label=ci_scope_label,
        dco_id=dco["check_run_id"],
        dco_sha=dco["head_sha"],
        dco_conclusion=dco_conclusion,
        dco_findings=dco_findings,
        dco_scope=dco_scope,
        python=ambient["python"],
        rust=ambient["mise_rust"],
        nextest=ambient["cargo_nextest"],
        host=ambient["host"],
        app_rows=app_rows,
        readiness_state=readiness_state,
        refactor_readiness_state=refactor_readiness_state,
        visibility_evidence=visibility_evidence,
        readiness_evidence=readiness_evidence,
        freshness_state=freshness_state,
        freshness_evidence=freshness_evidence,
        receipt_statement=receipt_statement,
        execution_section=execution_section,
        local_validation_section=local_validation_section,
        trend_statement=trend_statement,
        final_evidence_note=final_evidence_note,
        task_section=task_section,
        current_ci_section=current_ci_section,
        e2e_subtree_parity_layout=e2e_subtree_parity_layout,
        current_status_observations_section=current_status_observations_section,
        execution_attempt_history_layout=execution_attempt_history_layout,
        provider_observations_layout=provider_observations_layout,
        current_publication_section=current_publication_section,
        checklist_markdown_link=checklist_markdown_link,
        checklist_json_link=checklist_json_link,
        source_readme_link=source_readme_link,
        publication_section=publication_section,
    )


def load_source_facts(path: Path, queue_module: Any) -> Any:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise ValueError("cannot read {}: {}".format(path, error)) from error
    return queue_module.strict_json_loads(raw)


def load_queue_module() -> Any:
    queue_path = Path(__file__).resolve().with_name("queue.py")
    if queue_path.is_symlink() or not queue_path.is_file():
        raise ValueError("trusted adjacent queue module is unavailable")
    try:
        spec = importlib.util.spec_from_file_location(
            "_termrock_visibility_queue_for_status", queue_path
        )
        if spec is None or spec.loader is None:
            raise ImportError("queue module has no source loader")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception as error:
        raise ValueError("cannot load trusted adjacent queue module: {}".format(error)) from error
    if not all(callable(getattr(module, name, None)) for name in (
        "strict_json_loads", "validate_records", "validate_records_v2",
    )):
        raise ValueError("trusted adjacent queue module lacks required validators")
    return module


def load_task_records(path: Path, queue_module: Any) -> Any:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise ValueError("cannot read {}: {}".format(path, error)) from error
    return queue_module.strict_json_loads(raw)


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--write", action="store_true", help="write generated STATUS.md")
    modes.add_argument("--check", action="store_true", help="compare STATUS.md with current inputs")
    parser.add_argument(
        "--role",
        choices=("candidate", "reference"),
        default="candidate",
        help="identify this checkout's local presentation role (default: candidate)",
    )
    args = parser.parse_args(argv)
    try:
        queue_module = load_queue_module()
        task_records = load_task_records(TASKS_PATH, queue_module)
        output = render_status(
            load_source_facts(FACTS_PATH, queue_module), task_records,
            role=args.role, queue_module=queue_module
        )
    except ValueError as error:
        print("status generation failed: {}".format(error), file=sys.stderr)
        return 2
    if args.write:
        STATUS_PATH.write_text(output, encoding="utf-8")
        print("wrote {}".format(STATUS_PATH.relative_to(ROOT)))
    elif args.check:
        try:
            current = STATUS_PATH.read_text(encoding="utf-8")
        except OSError as error:
            print("status check failed: {}".format(error), file=sys.stderr)
            return 1
        if current != output:
            print("STATUS.md is stale; run python3 tools/visibility/status.py --write",
                  file=sys.stderr)
            return 1
        print("STATUS.md matches measured inputs")
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
