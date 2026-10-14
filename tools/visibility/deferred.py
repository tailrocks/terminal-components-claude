#!/usr/bin/env python3
"""List and execute the accepted deferred Termrock behavior tests."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence, Tuple


ROOT = Path(__file__).resolve().parents[2]
TASKS_PATH = ROOT / "docs/implementation/visibility/tasks.json"
DEFERRED_REGISTRY_PATH = ROOT / "crates/termrock-e2e/cases/deferred-obligations.json"
HISTORICAL_REASONS_PATH = ROOT / "docs/implementation/visibility/evidence/deferred/deferred-reasons-9deb66b.json"
CONTROL_STATES_PATH = ROOT / "crates/termrock-conformance/tests/control_states.rs"
OWNERSHIP_PATH = ROOT / "component-ownership.json"
CASE_REGISTRY_PATH = ROOT / "tests/conformance/required_cases.json"
EVIDENCE_ROOT = ROOT / "docs/implementation/visibility/evidence/deferred"
RUNNER_TEST_PATH = ROOT / "crates/termrock-visibility-tests/tests/deferred.rs"

SCHEMA = "termrock-deferred-run/v2"
SCHEMA_VERSION = 2
WORK_ID = "VIS-04"
CLAIM_PIN_SCHEMA = "termrock-visibility-claim-pin/v1"
PACKAGE = "termrock-conformance"
BINARY = "control_states"
EXPECTED_COUNT = 23
EXPECTED_CASE_REFERENCES = 25
EXPECTED_REGISTRY_SCHEMA = "termrock-e2e/deferred-obligations-v1"
EXPECTED_REGISTRY_SOURCE = (
    "DEFERRED-CASES.md / parity_burndown.rs at "
    "9deb66b48c99f674f23bb5ba183b3a8d0e57e534"
)
HISTORICAL_REASON_SCHEMA = "termrock-visibility/deferred-historical-reasons-v1"
HISTORICAL_REASON_COMMIT = "9deb66b48c99f674f23bb5ba183b3a8d0e57e534"
HISTORICAL_REASON_PATH = "crates/termrock-conformance/tests/parity_burndown.rs"
HISTORICAL_REASON_BLOB = "d7f588218e5128db53a374f641199f5736267b7b"
HISTORICAL_REASON_SHA256 = "037f56cd5b40585b1321c3919bd589031b7f2d56ef0b78ff0338f8b592899014"
HISTORICAL_REASON_SIDECAR_SHA256 = "90a08a85047f0b30fa159ec6a2779375db37472d734199d74b097f5edcdd9d9d"
RUST_TOOLCHAIN = "1.98.1"
BUILD_JOBS = 2
EXPECTED_DEFERRED_IDS = tuple(
    ["BD-{:02d}".format(number) for number in range(1, 19)]
    + ["BD-{:02d}".format(number) for number in range(20, 25)]
)
WRITER_EXCLUSION = (
    r"test(/(?i)(admission|approve|baseline|capture|snapshot|publish|write|"
    r"rebuild_review_html)/)"
)
WRITER_NAME = re.compile(
    r"(?i)(admission|approve|baseline|capture|snapshot|publish|write|rebuild_review_html)"
)


class DeferredError(ValueError):
    """An inventory, selection, or receipt input failed closed."""


@dataclass(frozen=True)
class DeferredCase:
    deferral_id: str
    case_ids: Tuple[str, ...]
    test_name: str
    component: str
    owner: str
    status: str
    source_recorded_reason: str
    source_reason_commit: str
    source_reason_path: str
    source_reason_blob: str
    source_reason_sha256: str
    source_reason_sidecar_path: str
    source_reason_sidecar_sha256: str
    registered_ignored: bool
    ignore_reason: Optional[str]

    def receipt_metadata(self) -> Dict[str, Any]:
        return {
            "requirement_id": self.deferral_id,
            "case_ids": list(self.case_ids),
            "component_id": self.component,
            "component_owner": self.owner,
            "application": None,
            "dimension": "behavior",
            "test_name": self.test_name,
            "registry_status": self.status,
            "source_recorded_reason": self.source_recorded_reason,
            "source_reason_source": {
                "commit": self.source_reason_commit,
                "path": self.source_reason_path,
                "git_blob": self.source_reason_blob,
                "raw_sha256": self.source_reason_sha256,
                "sidecar_path": self.source_reason_sidecar_path,
                "sidecar_sha256": self.source_reason_sidecar_sha256,
                "meaning": (
                    "historical source-recorded context only; not a current diagnosis, ignore "
                    "reason, assertion outcome, or PASS/FAIL result"
                ),
            },
            "registered_ignored": self.registered_ignored,
            "ignore_reason": self.ignore_reason,
        }


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DeferredError("cannot read {}: {}".format(path, error)) from error


def _unique_json_object(pairs: Sequence[Tuple[str, Any]]) -> Dict[str, Any]:
    result: Dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DeferredError("claim pin contains a duplicate field: {}".format(key))
        result[key] = value
    return result


def read_claim_pin(path: Path) -> Tuple[Dict[str, Any], bytes]:
    try:
        raw = path.read_bytes()
        pin = json.loads(
            raw.decode("utf-8"), object_pairs_hook=_unique_json_object
        )
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise DeferredError("cannot read claim pin {}: {}".format(path, error)) from error
    if not isinstance(pin, dict):
        raise DeferredError("claim pin must be an object")
    expected_fields = {
        "schema",
        "work_id",
        "expected_owner",
        "expected_claim_token",
        "expected_accepted_queue_revision",
    }
    if set(pin) != expected_fields:
        raise DeferredError("claim pin fields do not match the required schema")
    for name in ("schema", "work_id", "expected_owner", "expected_claim_token"):
        if type(pin[name]) is not str or not pin[name].strip():
            raise DeferredError("claim pin {} must be a non-empty string".format(name))
    if pin["schema"] != CLAIM_PIN_SCHEMA:
        raise DeferredError("claim pin schema is not supported")
    if pin["work_id"] != WORK_ID:
        raise DeferredError("claim pin work_id does not match {}".format(WORK_ID))
    if type(pin["expected_accepted_queue_revision"]) is not int:
        raise DeferredError("claim pin expected_accepted_queue_revision must be an integer")
    if pin["expected_accepted_queue_revision"] < 1:
        raise DeferredError("claim pin expected_accepted_queue_revision must be positive")
    return pin, raw


def parse_test_metadata(text: str) -> Dict[str, Tuple[bool, Optional[str]]]:
    """Return test names and their current Rust ignore metadata."""
    pattern = re.compile(
        r"(?m)^[ \t]*#\[test\][ \t]*\n"
        r"(?P<attributes>(?:[ \t]*#\[[^\n]*\][ \t]*\n)*)"
        r"[ \t]*fn[ \t]+(?P<name>[A-Za-z_][A-Za-z0-9_]*)[ \t]*\("
    )
    result: Dict[str, Tuple[bool, Optional[str]]] = {}
    for match in pattern.finditer(text):
        name = match.group("name")
        ignore_attributes = re.findall(
            r'(?m)^[ \t]*#\[ignore(?:[ \t]*=[ \t]*"((?:\\.|[^"\\])*)")?\][ \t]*$',
            match.group("attributes"),
        )
        ignore_present = re.search(
            r"(?m)^[ \t]*#\[ignore(?:[ \t]*=[^\]]*)?\][ \t]*$",
            match.group("attributes"),
        ) is not None
        if len(ignore_attributes) > 1 or (not ignore_present and ignore_attributes):
            raise DeferredError("{} has malformed Rust ignore metadata".format(name))
        reason: Optional[str] = None
        if ignore_attributes and ignore_attributes[0]:
            try:
                reason = json.loads('"{}"'.format(ignore_attributes[0]))
            except json.JSONDecodeError as error:
                raise DeferredError("{} has an invalid Rust ignore reason".format(name)) from error
        if name in result:
            raise DeferredError("duplicate Rust test function {}".format(name))
        result[name] = (ignore_present, reason)
    return result


def load_inventory(root: Path = ROOT) -> List[DeferredCase]:
    registry_path = root / DEFERRED_REGISTRY_PATH.relative_to(ROOT)
    reasons_path = root / HISTORICAL_REASONS_PATH.relative_to(ROOT)
    test_path = root / CONTROL_STATES_PATH.relative_to(ROOT)
    ownership_path = root / OWNERSHIP_PATH.relative_to(ROOT)
    cases_path = root / CASE_REGISTRY_PATH.relative_to(ROOT)

    try:
        registry_bytes = registry_path.read_bytes()
        reasons_bytes = reasons_path.read_bytes()
        test_text = test_path.read_text(encoding="utf-8")
    except OSError as error:
        raise DeferredError("cannot read deferred test inventory: {}".format(error)) from error

    try:
        registry = json.loads(registry_bytes.decode("utf-8"))
        reason_source = json.loads(reasons_bytes.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise DeferredError("deferred registry or historical reason sidecar is malformed: {}".format(error)) from error
    if sha256_bytes(reasons_bytes) != HISTORICAL_REASON_SIDECAR_SHA256:
        raise DeferredError("historical reason sidecar content hash changed")

    if not isinstance(registry, dict) or registry.get("schema") != EXPECTED_REGISTRY_SCHEMA:
        raise DeferredError("canonical deferred registry schema is not supported")
    if registry.get("source") != EXPECTED_REGISTRY_SOURCE:
        raise DeferredError("canonical deferred registry source identity changed")
    parsed_rows = registry.get("rows")
    if not isinstance(parsed_rows, list):
        raise DeferredError("canonical deferred registry rows must be an array")
    if type(registry.get("row_count")) is not int or registry["row_count"] != EXPECTED_COUNT:
        raise DeferredError("canonical deferred registry row_count is not 23")
    if type(registry.get("case_reference_count")) is not int or registry["case_reference_count"] != EXPECTED_CASE_REFERENCES:
        raise DeferredError("canonical deferred registry case_reference_count is not 25")
    if registry.get("missing_historical_id") != "BD-19":
        raise DeferredError("canonical deferred registry historical ID gap changed")
    if len(parsed_rows) != EXPECTED_COUNT:
        raise DeferredError(
            "expected {} canonical deferred rows, found {}".format(EXPECTED_COUNT, len(parsed_rows))
        )
    if any(not isinstance(row, dict) for row in parsed_rows):
        raise DeferredError("canonical deferred registry rows must be objects")
    if tuple(row.get("id") for row in parsed_rows) != EXPECTED_DEFERRED_IDS:
        raise DeferredError("deferred IDs changed or were reordered")
    if any(row.get("status") != "NOT_RUN" for row in parsed_rows):
        raise DeferredError("canonical deferred registry contains a non-NOT_RUN status")
    references: List[Tuple[str, str]] = []
    for row in parsed_rows:
        case_ids = row.get("cases")
        if not isinstance(case_ids, list) or not case_ids or any(
            not isinstance(case_id, str) for case_id in case_ids
        ):
            raise DeferredError("{} has invalid canonical case references".format(row.get("id")))
        if len(set(case_ids)) != len(case_ids):
            raise DeferredError("{} repeats a canonical case reference".format(row.get("id")))
        references.extend((row["id"], case_id) for case_id in case_ids)
    if len(references) != EXPECTED_CASE_REFERENCES or len(set(references)) != len(references):
        raise DeferredError(
            "expected {} unique canonical row/case references, found {}".format(
                EXPECTED_CASE_REFERENCES, len(set(references))
            )
        )

    if not isinstance(reason_source, dict) or reason_source.get("schema") != HISTORICAL_REASON_SCHEMA:
        raise DeferredError("historical reason sidecar schema is not supported")
    expected_source = {
        "commit": HISTORICAL_REASON_COMMIT,
        "path": HISTORICAL_REASON_PATH,
        "git_blob": HISTORICAL_REASON_BLOB,
        "raw_sha256": HISTORICAL_REASON_SHA256,
    }
    if reason_source.get("source") != expected_source:
        raise DeferredError("historical reason sidecar source binding changed")
    if not isinstance(reason_source.get("interpretation"), str) or "not current diagnoses" not in reason_source["interpretation"]:
        raise DeferredError("historical reason sidecar omits its context-only limitation")
    reason_rows = reason_source.get("rows")
    if not isinstance(reason_rows, list) or len(reason_rows) != EXPECTED_COUNT:
        raise DeferredError("historical reason sidecar must contain 23 rows")
    if reason_source.get("row_count") != EXPECTED_COUNT or reason_source.get("missing_historical_id") != "BD-19":
        raise DeferredError("historical reason sidecar denominator metadata changed")
    if any(not isinstance(row, dict) for row in reason_rows):
        raise DeferredError("historical reason sidecar rows must be objects")
    if tuple(row.get("id") for row in reason_rows) != EXPECTED_DEFERRED_IDS:
        raise DeferredError("historical reasons changed IDs or ordering")
    reason_by_id: Dict[str, str] = {}
    for row in reason_rows:
        reason = row.get("source_recorded_reason")
        if not isinstance(reason, str) or not reason.strip():
            raise DeferredError("{} has no historical source-recorded reason".format(row.get("id")))
        reason_by_id[row["id"]] = reason

    ownership = read_json(ownership_path)
    if not isinstance(ownership, dict) or not isinstance(ownership.get("components"), list):
        raise DeferredError("component-ownership.json has no components array")
    owner_by_component = {
        row.get("id"): row.get("owner")
        for row in ownership.get("components", [])
        if isinstance(row, dict)
    }
    case_registry = read_json(cases_path)
    if not isinstance(case_registry, dict) or not isinstance(case_registry.get("cases"), list):
        raise DeferredError("required case registry has no cases array")
    registry_ids = {
        row.get("id") for row in case_registry.get("cases", []) if isinstance(row, dict)
    }
    test_metadata = parse_test_metadata(test_text)

    result = []
    seen_tests = set()
    sidecar_sha256 = sha256_bytes(reasons_bytes)
    for row in parsed_rows:
        deferral_id = row.get("id")
        case_ids_value = row.get("cases")
        test_name = row.get("legacy_test")
        owner = row.get("owner")
        if not isinstance(deferral_id, str) or not isinstance(case_ids_value, list) or not case_ids_value:
            raise DeferredError("canonical deferred row has invalid ID or cases")
        if any(not isinstance(case_id, str) for case_id in case_ids_value):
            raise DeferredError("{} has invalid case IDs".format(deferral_id))
        case_ids = tuple(case_ids_value)
        if not isinstance(test_name, str) or not isinstance(owner, str):
            raise DeferredError("{} has invalid legacy_test or owner".format(deferral_id))
        if test_name in seen_tests:
            raise DeferredError("canonical inventory maps multiple rows to {}".format(test_name))
        seen_tests.add(test_name)
        components = {case_id[:3] for case_id in case_ids}
        if len(components) != 1 or not re.fullmatch(r"W\d{2}", next(iter(components))):
            raise DeferredError("{} case IDs do not identify one component".format(deferral_id))
        component = next(iter(components))
        if WRITER_NAME.search(test_name):
            raise DeferredError("canonical deferred inventory includes a writer-like test name")
        if deferral_id not in reason_by_id:
            raise DeferredError("{} has no historical source-recorded reason".format(deferral_id))
        registered = test_metadata.get(test_name)
        if registered is None:
            raise DeferredError("{} is not registered as a Rust integration test".format(test_name))
        registered_ignored, ignore_reason = registered
        if owner_by_component.get(component) != owner:
            raise DeferredError("{} owner differs from component-ownership.json".format(deferral_id))
        missing_cases = sorted(set(case_ids) - registry_ids)
        if missing_cases:
            raise DeferredError(
                "{} contains unknown case IDs {}".format(deferral_id, missing_cases)
            )
        result.append(
            DeferredCase(
                deferral_id=deferral_id,
                case_ids=case_ids,
                test_name=test_name,
                component=component,
                owner=owner,
                status=row["status"],
                source_recorded_reason=reason_by_id[deferral_id],
                source_reason_commit=HISTORICAL_REASON_COMMIT,
                source_reason_path=HISTORICAL_REASON_PATH,
                source_reason_blob=HISTORICAL_REASON_BLOB,
                source_reason_sha256=HISTORICAL_REASON_SHA256,
                source_reason_sidecar_path=HISTORICAL_REASONS_PATH.relative_to(ROOT).as_posix(),
                source_reason_sidecar_sha256=sidecar_sha256,
                registered_ignored=registered_ignored,
                ignore_reason=ignore_reason,
            )
        )
    return result


def filterset_for(cases: Sequence[DeferredCase]) -> str:
    exact_tests = " | ".join(
        "test(={})".format(case.test_name) for case in cases
    )
    return "package(={}) & binary(={}) & ({}) - {}".format(
        PACKAGE, BINARY, exact_tests, WRITER_EXCLUSION
    )


def common_nextest_args(cases: Sequence[DeferredCase]) -> List[str]:
    return [
        "--locked",
        "--user-config-file",
        "none",
        "--package",
        PACKAGE,
        "--test",
        BINARY,
        "--run-ignored",
        "all",
        "--ignore-default-filter",
        "--color",
        "never",
        "--show-progress",
        "none",
        "--filterset",
        filterset_for(cases),
        "--cargo-quiet",
    ]


def list_command(cases: Sequence[DeferredCase]) -> List[str]:
    return ["rustup", "run", RUST_TOOLCHAIN, "cargo", "nextest", "list"] + common_nextest_args(cases) + [
        "--message-format",
        "json",
    ]


def run_command(cases: Sequence[DeferredCase]) -> List[str]:
    return ["rustup", "run", RUST_TOOLCHAIN, "cargo", "nextest", "run"] + common_nextest_args(cases) + [
        "--no-tests",
        "fail",
        "--no-fail-fast",
        "--retries",
        "0",
        "--test-threads",
        "1",
        "--flaky-result",
        "fail",
        "--failure-output",
        "immediate-final",
        "--final-status-level",
        "all",
        "--no-output-indent",
        "--message-format",
        "libtest-json-plus",
        "--message-format-version",
        "0.1",
    ]


def capture_environment() -> Dict[str, str]:
    env = os.environ.copy()
    path_entries = []
    for entry in env.get("PATH", "").split(os.pathsep):
        normalized = entry.lower().replace("\\", "/")
        if "/application support/mbx/bin" in normalized:
            continue
        if "/command-wrappers/bin" in normalized:
            continue
        if entry and entry not in path_entries:
            path_entries.append(entry)
    env["PATH"] = os.pathsep.join(path_entries)
    env["CARGO_TERM_COLOR"] = "never"
    env["CARGO_BUILD_JOBS"] = str(BUILD_JOBS)
    env["RUSTUP_TOOLCHAIN"] = RUST_TOOLCHAIN
    env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"] = "1"
    # Explicit nextest flags below own selection, retries, threads, and reporting.
    for name in list(env):
        if name.startswith("NEXTEST_") and name != "NEXTEST_EXPERIMENTAL_LIBTEST_JSON":
            del env[name]
    rustup = shutil.which("rustup", path=env["PATH"])
    if rustup is None:
        raise DeferredError("rustup is unavailable on the sanitized PATH")
    cargo_path = subprocess.run(
        [rustup, "which", "--toolchain", RUST_TOOLCHAIN, "cargo"],
        cwd=str(ROOT), env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if cargo_path.returncode != 0:
        detail = cargo_path.stderr.decode("utf-8", errors="replace").strip()
        raise DeferredError("cannot resolve Cargo for Rust {}: {}".format(RUST_TOOLCHAIN, detail))
    env["CARGO"] = cargo_path.stdout.decode("utf-8", errors="strict").strip()
    if not Path(env["CARGO"]).is_file():
        raise DeferredError("resolved Cargo executable does not exist: {}".format(env["CARGO"]))
    mise = shutil.which("mise", path=env["PATH"])
    if mise is None:
        raise DeferredError("mise is unavailable on the sanitized PATH")
    nextest_path = subprocess.run(
        [mise, "which", "cargo-nextest"],
        cwd=str(ROOT), env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if nextest_path.returncode != 0:
        detail = nextest_path.stderr.decode("utf-8", errors="replace").strip()
        raise DeferredError("cannot resolve pinned cargo-nextest: {}".format(detail))
    nextest_executable = nextest_path.stdout.decode("utf-8", errors="strict").strip()
    if not Path(nextest_executable).is_file():
        raise DeferredError("resolved cargo-nextest executable does not exist: {}".format(
            nextest_executable
        ))
    env["PATH"] = os.path.dirname(nextest_executable) + os.pathsep + env["PATH"]
    return env


def run_and_capture(command: Sequence[str], stdout_path: Path, stderr_path: Path,
                    env: Mapping[str, str], runner: Any = subprocess.run) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as stdout_file, stderr_path.open("wb") as stderr_file:
        try:
            completed = runner(
                list(command),
                cwd=str(ROOT),
                env=dict(env),
                stdout=stdout_file,
                stderr=stderr_file,
                check=False,
            )
        except OSError as error:
            stderr_file.write(("runner error: {}\n".format(error)).encode("utf-8"))
            return 127
    return int(completed.returncode)


def parse_nextest_list(data: Any, expected: Sequence[DeferredCase]) -> Dict[str, bool]:
    if not isinstance(data, dict):
        raise DeferredError("nextest list output must be a JSON object")
    suites = data.get("rust-suites")
    if not isinstance(suites, dict):
        raise DeferredError("nextest list output has no rust-suites object")

    selected: Dict[str, bool] = {}
    selected_suites = 0
    for suite in suites.values():
        if not isinstance(suite, dict):
            raise DeferredError("nextest list suite metadata must be an object")
        testcases = suite.get("testcases")
        if not isinstance(testcases, dict):
            continue
        matching = {
            name: metadata
            for name, metadata in testcases.items()
            if isinstance(metadata, dict)
            and isinstance(metadata.get("filter-match"), dict)
            and metadata["filter-match"].get("status") == "matches"
        }
        if not matching:
            continue
        selected_suites += 1
        if suite.get("package-name") != PACKAGE or suite.get("binary-name") != BINARY:
            raise DeferredError(
                "nextest selected unexpected suite {}/{}".format(
                    suite.get("package-name"), suite.get("binary-name")
                )
            )
        if suite.get("kind") != "test":
            raise DeferredError("nextest selected a non-integration-test binary")
        for name, metadata in matching.items():
            if name in selected:
                raise DeferredError("nextest listed duplicate selected test {}".format(name))
            ignored = metadata.get("ignored")
            if type(ignored) is not bool:
                raise DeferredError("nextest omitted ignored metadata for {}".format(name))
            selected[name] = ignored

    expected_names = {case.test_name for case in expected}
    if selected_suites != 1:
        raise DeferredError("expected one selected control_states test binary")
    if any(WRITER_NAME.search(name) for name in selected):
        raise DeferredError("nextest selection contains a writer-like test name")
    if set(selected) != expected_names:
        raise DeferredError(
            "nextest selection mismatch; expected={}, listed={}".format(
                sorted(expected_names), sorted(selected)
            )
        )
    expected_ignored = {case.test_name: case.registered_ignored for case in expected}
    metadata_drift = sorted(
        name for name, ignored in selected.items() if ignored != expected_ignored[name]
    )
    if metadata_drift:
        raise DeferredError(
            "nextest ignored metadata differs from current Rust registration for {}".format(
                metadata_drift
            )
        )
    total_count = data.get("test-count")
    if type(total_count) is not int or total_count < len(selected):
        raise DeferredError(
            "nextest test-count is {}, fewer than the {} selected tests".format(
                total_count, len(selected)
            )
        )
    return selected


def normalize_event_name(name: Any, expected_names: Iterable[str]) -> str:
    if not isinstance(name, str):
        raise DeferredError("nextest test event has no string name")
    expected = list(expected_names)
    exact = [candidate for candidate in expected if name == candidate]
    if exact:
        return exact[0]
    suffix = [
        candidate for candidate in expected
        if name.endswith("$" + candidate) or name.endswith("::" + candidate)
    ]
    if len(suffix) == 1:
        return suffix[0]
    raise DeferredError("nextest emitted unexpected test name {!r}".format(name))


def all_nextest_test_names(data: Any) -> set:
    if not isinstance(data, dict) or not isinstance(data.get("rust-suites"), dict):
        raise DeferredError("nextest list output has no rust-suites object")
    target_suites = [
        suite for suite in data["rust-suites"].values()
        if isinstance(suite, dict)
        and suite.get("package-name") == PACKAGE
        and suite.get("binary-name") == BINARY
        and suite.get("kind") == "test"
    ]
    if len(target_suites) != 1 or not isinstance(target_suites[0].get("testcases"), dict):
        raise DeferredError("cannot identify one complete control_states test list")
    return set(target_suites[0]["testcases"])


def event_basename(name: Any) -> str:
    if not isinstance(name, str):
        raise DeferredError("nextest test event has no string name")
    if "$" in name:
        return name.rsplit("$", 1)[1]
    return name.rsplit("::", 1)[-1]


def parse_run_events(stdout: bytes, expected: Sequence[DeferredCase],
                     known_unselected: Optional[Iterable[str]] = None) -> Tuple[Dict[str, str], List[str]]:
    expected_names = [case.test_name for case in expected]
    unselected_names = set(known_unselected or ()) - set(expected_names)
    started = set()
    outcomes: Dict[str, str] = {}
    terminal_events: Dict[str, str] = {}
    errors: List[str] = []
    try:
        decoded = stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        return {}, ["nextest JSON output is not UTF-8: {}".format(error)]

    for line_number, line in enumerate(decoded.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            errors.append("invalid JSON event at line {}: {}".format(line_number, error))
            continue
        if not isinstance(event, dict) or event.get("type") != "test":
            continue
        event_name = event.get("name")
        try:
            basename = event_basename(event_name)
        except DeferredError as error:
            errors.append(str(error))
            continue
        if basename in unselected_names:
            event_kind = event.get("event")
            errors.append("unselected test emitted an event: {} ({})".format(
                basename, event_kind
            ))
            continue
        try:
            test_name = normalize_event_name(event_name, expected_names)
        except DeferredError as error:
            errors.append(str(error))
            continue
        event_kind = event.get("event")
        if event_kind == "started":
            started.add(test_name)
        elif event_kind in ("ok", "failed", "ignored", "timeout", "cancelled", "exec-failed"):
            status = {
                "ok": "passed",
                "failed": "failed",
                "ignored": "skipped",
                "timeout": "incomplete",
                "cancelled": "incomplete",
                "exec-failed": "incomplete",
            }[event_kind]
            previous_event = terminal_events.get(test_name)
            if previous_event is not None:
                errors.append(
                    "duplicate terminal result for {}: {} then {}".format(
                        test_name, previous_event, event_kind
                    )
                )
                # Contradictory terminal events cannot leave a pass or skip in
                # the receipt. Keep any real assertion failure visible.
                if outcomes[test_name] == "failed" or status == "failed":
                    outcomes[test_name] = "failed"
                else:
                    outcomes[test_name] = "incomplete"
            else:
                terminal_events[test_name] = event_kind
                outcomes[test_name] = status
            if event_kind in ("timeout", "cancelled", "exec-failed"):
                started.add(test_name)

    for name in started:
        outcomes.setdefault(name, "incomplete")
    unexpected = sorted(set(outcomes) - set(expected_names))
    if unexpected:
        errors.append("unexpected test results: {}".format(unexpected))
    return outcomes, errors


def file_ref(path: Path) -> Dict[str, Any]:
    data = path.read_bytes()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "size_bytes": len(data),
        "sha256": sha256_bytes(data),
    }


def write_json(path: Path, value: Any) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )


def git_output(*arguments: str) -> str:
    completed = subprocess.run(
        ["git"] + list(arguments), cwd=str(ROOT), stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise DeferredError("git {} failed: {}".format(" ".join(arguments), detail))
    return completed.stdout.decode("utf-8", errors="strict").strip()


def worktree_record() -> Dict[str, Any]:
    entries = git_output("status", "--porcelain=v1", "--untracked-files=all").splitlines()
    return {"state": "dirty" if entries else "clean", "porcelain_v1": entries}


def validate_claim(claim_pin_path: Path) -> Dict[str, Any]:
    pin, pin_bytes = read_claim_pin(claim_pin_path)
    records = read_json(TASKS_PATH)
    if not isinstance(records, dict) or not isinstance(records.get("tasks"), list):
        raise DeferredError("accepted task registry is malformed")
    current_queue_revision = records.get("queue_revision")
    if type(current_queue_revision) is not int or current_queue_revision < 1:
        raise DeferredError("current queue_revision must be a positive integer")
    matches = [task for task in records["tasks"] if isinstance(task, dict)
               and task.get("work_id") == WORK_ID]
    if len(matches) != 1:
        raise DeferredError("expected one accepted VIS-04 claim")
    task = matches[0]
    if task.get("owner") != pin["expected_owner"]:
        raise DeferredError("VIS-04 owner does not match the caller-pinned claim")
    if task.get("claim_token") != pin["expected_claim_token"]:
        raise DeferredError("VIS-04 claim token does not match the caller-pinned claim")
    if task.get("branch") != "termrock-implementation" or task.get("state") not in ("claimed", "in_progress"):
        raise DeferredError("VIS-04 is not active on termrock-implementation")
    accepted_queue_revision = task.get("accepted_queue_revision")
    if type(accepted_queue_revision) is not int or accepted_queue_revision < 1:
        raise DeferredError("VIS-04 accepted_queue_revision must be a positive integer")
    if accepted_queue_revision != pin["expected_accepted_queue_revision"]:
        raise DeferredError("VIS-04 accepted queue revision does not match the caller-pinned claim")
    if current_queue_revision < accepted_queue_revision:
        raise DeferredError("current queue_revision is older than the accepted claim")
    accepted_paths = {
        "tools/visibility/deferred.py",
        "tools/visibility/tests/test_deferred.py",
        "docs/implementation/visibility/evidence/deferred/**",
        "crates/termrock-visibility-tests/tests/deferred.rs",
    }
    allowed_list = task.get("allowed_paths")
    if not isinstance(allowed_list, list) or set(allowed_list) != accepted_paths:
        raise DeferredError("VIS-04 allowed paths changed from the accepted scope")
    return {
        "work_id": WORK_ID,
        "owner": pin["expected_owner"],
        "claim_token": pin["expected_claim_token"],
        "claim_pin_sha256": sha256_bytes(pin_bytes),
        "queue_revision": current_queue_revision,
        "accepted_queue_revision": accepted_queue_revision,
        "accepted_base_sha": task.get("base_sha"),
        "state": task.get("state"),
    }


def relevant_input_digest(root: Path = ROOT) -> str:
    """Hash code, Cargo manifests, registries, and relevant local config inputs."""
    candidates = [
        root / "Cargo.toml",
        root / "Cargo.lock",
        root / "tools/visibility/deferred.py",
        root / "mise.toml",
        root / "rust-toolchain",
        root / "rust-toolchain.toml",
        root / "component-ownership.json",
        root / "tests/conformance/required_cases.json",
        root / "crates/termrock-e2e/cases/deferred-obligations.json",
        root / "docs/implementation/visibility/evidence/deferred/deferred-reasons-9deb66b.json",
        root / "crates/termrock-visibility-tests/tests/deferred.rs",
        root / "crates/termrock-conformance/tests/parity_burndown.rs",
        root / "crates/termrock-conformance/tests/control_states.rs",
    ]
    try:
        cargo_manifest = (root / "Cargo.toml").read_text(encoding="utf-8")
    except OSError as error:
        raise DeferredError("cannot read Cargo.toml for source digest: {}".format(error)) from error
    members = re.search(r"(?ms)^members\s*=\s*\[(.*?)\]", cargo_manifest)
    if members is None:
        raise DeferredError("Cargo.toml workspace member list is missing")
    workspace_roots = [root / member for member in re.findall(r'"([^"]+)"', members.group(1))]
    for workspace_root in workspace_roots:
        if workspace_root.exists():
            candidates.extend(
                path for path in workspace_root.rglob("*")
                if path.is_file()
                and path.name != ".DS_Store"
                and not any(
                    part in {"target", "cache", ".cache"}
                    for part in path.relative_to(root).parts
                )
            )
    cargo_dir = root / ".cargo"
    if cargo_dir.exists():
        candidates.extend(
            path for path in cargo_dir.rglob("*")
            if path.is_file()
            and not any(
                part in {"target", "cache", ".cache"}
                for part in path.relative_to(root).parts
            )
        )
    nextest_config = root / ".config/nextest.toml"
    if nextest_config.is_file():
        candidates.append(nextest_config)

    digest = hashlib.sha256()
    seen = set()
    for path in sorted(set(candidates), key=lambda item: item.relative_to(root).as_posix()):
        relative = path.relative_to(root).as_posix()
        if relative in seen or not path.is_file():
            continue
        seen.add(relative)
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def tool_version(env: Mapping[str, str]) -> str:
    completed = subprocess.run(
        ["rustup", "run", RUST_TOOLCHAIN, "cargo", "nextest", "--version"], cwd=str(ROOT),
        env=dict(env),
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    output = completed.stdout.decode("utf-8", errors="replace").strip()
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise DeferredError("cargo nextest --version failed: {}".format(detail))
    return output.splitlines()[0] if output else "unknown"


def build_receipt_base(run_id: str, started_at: str, branch: str,
                       commit: str, inventory: Sequence[DeferredCase],
                       source_digest: str, nextest_version: str,
                       claim: Mapping[str, Any]) -> Dict[str, Any]:
    paths = {
        "list_stdout": EVIDENCE_ROOT / run_id / "nextest-list.stdout.json",
        "list_stderr": EVIDENCE_ROOT / run_id / "nextest-list.stderr.log",
        "stdout": EVIDENCE_ROOT / run_id / "nextest-run.stdout.jsonl",
        "stderr": EVIDENCE_ROOT / run_id / "nextest-run.stderr.log",
    }
    return {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "run_id": run_id,
        "started_at": started_at,
        "finished_at": None,
        "subject": {
            "repository": "tailrocks/terminal-components-claude",
            "branch": branch,
            "commit": commit,
            "accepted_base_sha": claim.get("accepted_base_sha"),
            "accepted_base_is_ancestor": None,
            "source_inputs_sha256": source_digest,
            "working_tree": None,
        },
        "tool": {
            "name": "cargo nextest",
            "version": nextest_version,
            "list_argv": None,
            "run_argv": None,
            "environment_inputs": {
                "CARGO_TERM_COLOR": "never",
                "CARGO_BUILD_JOBS": str(BUILD_JOBS),
                "CARGO": None,
                "RUSTUP_TOOLCHAIN": RUST_TOOLCHAIN,
                "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1",
            },
            "python_version": platform.python_version(),
            "platform": platform.platform(),
            "cargo_path": shutil.which("cargo"),
            "cargo_nextest_path": shutil.which("cargo-nextest"),
        },
        "tool_inputs": {
            "work_item": WORK_ID,
            "claim": dict(claim),
            "inventory_sha256": inventory_digest(inventory),
            "source_inputs_sha256": source_digest,
            "historical_reason_source": {
                "path": HISTORICAL_REASONS_PATH.relative_to(ROOT).as_posix(),
                "sidecar_sha256": inventory[0].source_reason_sidecar_sha256 if inventory else None,
                "commit": HISTORICAL_REASON_COMMIT,
                "source_path": HISTORICAL_REASON_PATH,
                "git_blob": HISTORICAL_REASON_BLOB,
                "raw_sha256": HISTORICAL_REASON_SHA256,
                "interpretation": "historical source-recorded context, not current diagnoses or results",
            },
            "runner_source": file_ref(Path(__file__).resolve()),
            "runner_test_source": file_ref(RUNNER_TEST_PATH),
        },
        "selection": {
            "lane": "deferred-behavior",
            "evidence_class": "candidate-direct-assertions",
            "qualifies_paired_pty_evidence": False,
            "package": PACKAGE,
            "binary": BINARY,
            "run_ignored": "all",
            "expected_tests": [case.receipt_metadata() for case in inventory],
            "listed_tests": [],
            "listed_test_names": [],
            "listed_test_metadata": [],
            "listed_total_count": None,
            "writer_exclusion": WRITER_EXCLUSION,
            "filterset": filterset_for(inventory),
            "registered_ignored_count": sum(case.registered_ignored for case in inventory),
            "registered_normal_count": sum(not case.registered_ignored for case in inventory),
            "count": len(inventory),
        },
        "execution": {
            "exit_code": None,
            "result": "blocked",
            "executed_tests": [],
            "counts": {
                "expected": len(inventory),
                "listed": 0,
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "skipped": 0,
                "not_run": len(inventory),
            },
            "block_reason": None,
            "nextest_list_exit_code": None,
            "expected_failure_treatment": "none",
        },
        "artifacts": {
            "list_json": None,
            "list_stderr": None,
            "stdout": None,
            "stderr": None,
        },
        "_artifact_paths": paths,
    }


def inventory_digest(inventory: Sequence[DeferredCase]) -> str:
    encoded = json.dumps(
        [asdict(case) for case in inventory], ensure_ascii=False,
        sort_keys=True, separators=(",", ":"),
    ).encode("utf-8")
    return sha256_bytes(encoded)


def finalize_receipt(receipt: Dict[str, Any], receipt_path: Path) -> None:
    artifact_paths = receipt.pop("_artifact_paths")
    artifacts: Dict[str, Optional[Dict[str, Any]]] = {}
    for key, path in artifact_paths.items():
        try:
            artifacts[key] = file_ref(path)
        except OSError:
            artifacts[key] = None
    receipt["artifacts"] = {
        "list_json": artifacts["list_stdout"],
        "list_stderr": artifacts["list_stderr"],
        "stdout": artifacts["stdout"],
        "stderr": artifacts["stderr"],
    }
    receipt["finished_at"] = utc_now()
    write_json(receipt_path, receipt)


def result_rows(inventory: Sequence[DeferredCase], outcomes: Mapping[str, str]) -> List[Dict[str, Any]]:
    rows = []
    for case in inventory:
        status = outcomes.get(case.test_name, "not_run")
        rows.append(dict(case.receipt_metadata(), status=status))
    return rows


def count_results(outcomes: Mapping[str, str], expected: int, listed: int) -> Dict[str, int]:
    values = list(outcomes.values())
    executed = sum(status != "not_run" for status in values)
    return {
        "expected": expected,
        "listed": listed,
        "executed": executed,
        "passed": values.count("passed"),
        "failed": values.count("failed"),
        "skipped": values.count("skipped"),
        "not_run": max(expected - executed, 0),
    }


def classify_execution(run_exit: int, outcomes: Mapping[str, str],
                       parse_errors: Sequence[str], listed: int) -> Tuple[str, Optional[str], int]:
    """Preserve Nextest's positive status; map signal deaths to shell status 128+signal.

    The receipt retains the raw subprocess status, including negative signal codes.
    A zero process status with invalid result records remains a wrapper failure.
    """
    counts = count_results(outcomes, EXPECTED_COUNT, listed)
    if run_exit < 0:
        signal_number = abs(run_exit)
        return (
            "blocked",
            "nextest terminated by signal {}".format(signal_number),
            128 + signal_number,
        )
    if run_exit == 0 and parse_errors:
        return "blocked", "nextest result output could not be reconciled", 2
    if counts["failed"]:
        return "failed", None, run_exit if run_exit > 0 else 1
    if parse_errors:
        return "blocked", "nextest result output could not be reconciled", run_exit
    if run_exit > 0:
        if counts["executed"] == 0:
            return "blocked", "nextest exited before any tests executed", run_exit
        return "blocked", "nextest exited {} without selected-test failures".format(run_exit), run_exit
    if run_exit == 0 and counts["passed"] == EXPECTED_COUNT:
        return "passed", None, 0
    if counts["executed"] == 0:
        return "blocked", "nextest run executed zero tests; inspect build or runner diagnostics", 2
    return "blocked", "nextest run did not produce a complete 23-test result set", 2


def execute(claim_pin_path: Path) -> Tuple[int, Path]:
    started_at = utc_now()
    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    evidence_dir = EVIDENCE_ROOT / run_id
    evidence_dir.mkdir(parents=True, exist_ok=False)
    receipt_path = evidence_dir / "receipt.json"
    for name in (
        "nextest-list.stdout.json", "nextest-list.stderr.log",
        "nextest-run.stdout.jsonl", "nextest-run.stderr.log",
    ):
        (evidence_dir / name).write_bytes(b"")

    inventory: List[DeferredCase] = []
    receipt: Dict[str, Any] = {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "run_id": run_id,
        "started_at": started_at,
        "finished_at": None,
        "subject": {"repository": "tailrocks/terminal-components-claude", "branch": None,
                    "commit": None, "accepted_base_sha": None,
                    "accepted_base_is_ancestor": None,
                    "source_inputs_sha256": None, "working_tree": None},
        "tool": {"name": "cargo nextest", "version": None, "list_argv": None,
                 "run_argv": None, "environment_inputs": {},
                 "python_version": platform.python_version(), "platform": platform.platform(),
                 "cargo_path": shutil.which("cargo"),
                 "cargo_nextest_path": shutil.which("cargo-nextest")},
        "tool_inputs": {"work_item": WORK_ID, "claim": None, "inventory_sha256": None,
                        "source_inputs_sha256": None, "historical_reason_source": None},
        "selection": {"lane": "deferred-behavior",
                       "evidence_class": "candidate-direct-assertions",
                       "qualifies_paired_pty_evidence": False,
                       "package": PACKAGE, "binary": BINARY,
                       "run_ignored": "all", "expected_tests": [], "listed_tests": [],
                       "listed_test_metadata": [],
                       "listed_test_names": [],
                       "listed_total_count": None,
                       "writer_exclusion": WRITER_EXCLUSION, "filterset": None,
                       "registered_ignored_count": 0, "registered_normal_count": 0,
                       "count": EXPECTED_COUNT},
        "execution": {"exit_code": None, "result": "blocked", "executed_tests": [],
                       "counts": {"expected": EXPECTED_COUNT, "listed": 0, "executed": 0,
                                  "passed": 0, "failed": 0, "skipped": 0,
                                  "not_run": EXPECTED_COUNT}, "block_reason": None,
                       "nextest_list_exit_code": None,
                       "expected_failure_treatment": "none"},
        "artifacts": {"list_json": None, "list_stderr": None, "stdout": None, "stderr": None},
        "_artifact_paths": {
            "list_stdout": evidence_dir / "nextest-list.stdout.json",
            "list_stderr": evidence_dir / "nextest-list.stderr.log",
            "stdout": evidence_dir / "nextest-run.stdout.jsonl",
            "stderr": evidence_dir / "nextest-run.stderr.log",
        },
    }
    exit_code = 2
    try:
        claim = validate_claim(claim_pin_path)
        branch = git_output("rev-parse", "--abbrev-ref", "HEAD")
        commit = git_output("rev-parse", "HEAD")
        receipt["tool_inputs"]["claim"] = claim
        receipt["subject"]["branch"] = branch
        receipt["subject"]["commit"] = commit
        receipt["subject"]["accepted_base_sha"] = claim.get("accepted_base_sha")
        receipt["subject"]["working_tree"] = worktree_record()
        source_digest = relevant_input_digest()
        inventory = load_inventory()
        inventory_source_digest = relevant_input_digest()
        if inventory_source_digest != source_digest:
            raise DeferredError("test source inputs changed while reading canonical inventory")
        source_digest = inventory_source_digest
        receipt["selection"]["expected_tests"] = [
            case.receipt_metadata() for case in inventory
        ]
        receipt["selection"]["filterset"] = filterset_for(inventory)
        receipt["tool_inputs"]["inventory_sha256"] = inventory_digest(inventory)
        receipt["subject"]["source_inputs_sha256"] = source_digest
        receipt["tool_inputs"]["source_inputs_sha256"] = source_digest
        if branch != "termrock-implementation":
            raise DeferredError("expected termrock-implementation, found {}".format(branch))
        git_output("merge-base", "--is-ancestor", claim["accepted_base_sha"], commit)
        receipt["subject"]["accepted_base_is_ancestor"] = True
        env = capture_environment()
        version = tool_version(env)
        receipt = build_receipt_base(
            run_id, started_at, branch, commit, inventory, source_digest, version, claim
        )
        receipt["subject"]["accepted_base_is_ancestor"] = True
        receipt["subject"]["working_tree"] = worktree_record()
        receipt["_artifact_paths"] = {
            "list_stdout": evidence_dir / "nextest-list.stdout.json",
            "list_stderr": evidence_dir / "nextest-list.stderr.log",
            "stdout": evidence_dir / "nextest-run.stdout.jsonl",
            "stderr": evidence_dir / "nextest-run.stderr.log",
        }
        receipt["tool"]["cargo_path"] = env["CARGO"]
        receipt["tool"]["cargo_nextest_path"] = shutil.which("cargo-nextest", path=env["PATH"])
        list_argv = list_command(inventory)
        receipt["tool"]["list_argv"] = list_argv
        receipt["tool"]["environment_inputs"] = {
            "CARGO_TERM_COLOR": env["CARGO_TERM_COLOR"],
            "CARGO_BUILD_JOBS": env["CARGO_BUILD_JOBS"],
            "CARGO": env["CARGO"],
            "RUSTUP_TOOLCHAIN": env["RUSTUP_TOOLCHAIN"],
            "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": env["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"],
            "CARGO_NEXTTEST_EXECUTABLE": shutil.which("cargo-nextest", path=env["PATH"]),
            "build": {
                name: os.environ[name]
                for name in (
                    "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_ENCODED_RUSTFLAGS",
                    "RUSTFLAGS", "RUSTUP_TOOLCHAIN",
                )
                if name in os.environ
            },
        }
        list_exit = run_and_capture(
            list_argv,
            evidence_dir / "nextest-list.stdout.json",
            evidence_dir / "nextest-list.stderr.log",
            env,
        )
        receipt["execution"]["nextest_list_exit_code"] = list_exit
        if list_exit != 0:
            receipt["execution"]["block_reason"] = (
                "nextest list failed before test execution (exit status {})".format(list_exit)
            )
            exit_code = 2
            return exit_code, receipt_path

        list_output = read_json(evidence_dir / "nextest-list.stdout.json")
        selected = parse_nextest_list(list_output, inventory)
        all_names = all_nextest_test_names(list_output)
        if not set(selected).issubset(all_names):
            raise DeferredError("selected tests are missing from the complete Nextest list")
        listed_names = sorted(selected)
        selected_cases = [case for case in inventory if case.test_name in selected]
        listed_ids = [case.deferral_id for case in selected_cases]
        receipt["selection"]["listed_tests"] = listed_ids
        receipt["selection"]["listed_test_names"] = listed_names
        receipt["selection"]["listed_test_metadata"] = [
            {
                "requirement_id": case.deferral_id,
                "test_name": case.test_name,
                "ignored": selected[case.test_name],
            }
            for case in inventory
        ]
        receipt["selection"]["listed_total_count"] = list_output["test-count"]
        receipt["execution"]["counts"]["listed"] = len(listed_names)
        if relevant_input_digest() != source_digest:
            receipt["execution"]["block_reason"] = "test source inputs changed after nextest list"
            exit_code = 2
            return exit_code, receipt_path

        run_argv = run_command(inventory)
        receipt["tool"]["run_argv"] = run_argv
        receipt["execution"]["block_reason"] = None
        run_exit = run_and_capture(
            run_argv,
            evidence_dir / "nextest-run.stdout.jsonl",
            evidence_dir / "nextest-run.stderr.log",
            env,
        )
        receipt["execution"]["exit_code"] = run_exit
        raw_stdout = (evidence_dir / "nextest-run.stdout.jsonl").read_bytes()
        outcomes, parse_errors = parse_run_events(
            raw_stdout, inventory, known_unselected=all_names - set(selected)
        )
        counts = count_results(outcomes, EXPECTED_COUNT, len(listed_names))
        receipt["execution"]["counts"] = counts
        receipt["execution"]["executed_tests"] = result_rows(inventory, outcomes)
        receipt["execution"]["parse_errors"] = parse_errors

        result, block_reason, exit_code = classify_execution(
            run_exit, outcomes, parse_errors, len(listed_names)
        )
        receipt["execution"]["result"] = result
        receipt["execution"]["block_reason"] = block_reason
        if relevant_input_digest() != source_digest:
            receipt["subject"]["source_inputs_changed_during_run"] = True
            receipt["execution"]["result"] = "blocked"
            receipt["execution"]["block_reason"] = "test source inputs changed during nextest run"
            exit_code = 2
        return exit_code, receipt_path
    except (DeferredError, OSError, json.JSONDecodeError) as error:
        receipt["execution"]["result"] = "blocked"
        receipt["execution"]["block_reason"] = str(error)
        # A list or source-preflight failure means tests did not execute.
        if receipt["execution"].get("exit_code") is None:
            receipt["execution"]["counts"] = {
                "expected": EXPECTED_COUNT,
                "listed": receipt["execution"]["counts"].get("listed", 0),
                "executed": 0,
                "passed": 0,
                "failed": 0,
                "skipped": 0,
                "not_run": EXPECTED_COUNT,
            }
            receipt["execution"]["executed_tests"] = result_rows(inventory, {}) if inventory else []
        exit_code = 2
        return exit_code, receipt_path
    finally:
        finalize_receipt(receipt, receipt_path)


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action",
        choices=("run",),
        help="build/list the exact canonical selection, then execute all 23 deferred tests",
    )
    parser.add_argument(
        "--claim-pin",
        required=True,
        type=Path,
        help="caller-pinned VIS-04 owner, token, and accepted queue revision JSON",
    )
    args = parser.parse_args(argv)
    if args.action != "run":
        return 2
    exit_code, receipt_path = execute(args.claim_pin)
    try:
        relative_receipt = receipt_path.relative_to(ROOT)
    except ValueError:
        relative_receipt = receipt_path
    print("{}: {}".format(relative_receipt, exit_code))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
