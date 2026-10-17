#!/usr/bin/env python3
"""Build pinned branch subjects or the immutable visual-baseline Holla oracle."""

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
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Dict, List, Mapping, Optional, Sequence, Tuple


ROOT = Path(__file__).resolve().parents[2]
SUBJECT_SCHEMA_PATH = ROOT / "crates/termrock-e2e/schemas/subject-manifest-v2.schema.json"
SUBJECT_SCHEMA_SHA256 = "e2d294145e6e2a9b5b682e0dfcb7793fe0e20e4cf5492b30d458d85ad8bde8f0"
SUBJECT_SCHEMA = "termrock-spec/parity-subject-manifest-v2"
BUILDER_RECEIPT_SCHEMA = "termrock-spec/parity-subject-build-receipt-v1"
TAG_BUILDER_RECEIPT_SCHEMA = "termrock-spec/visual-tag-holla-build-evidence-v1"
TAG_BUILDER_RUN_SCHEMA = "termrock-spec/visual-tag-holla-build-run-v1"
REFERENCE_TAG = "visual-baseline"
REFERENCE_TAG_OBJECT = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
REFERENCE_ORACLE_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
REFERENCE_BRANCH_REF = "refs/remotes/origin/visual-baseline"
CANDIDATE_BRANCH_REF = "refs/remotes/origin/termrock-implementation"
TOOLCHAIN = "1.98.1"
APPS = ("showcase", "jackin-preview", "holla", "tablepro")
ORACLE_ROOTS = ("snapshots", "baselines")
ORACLE_EXCLUSIONS = tuple(root + "/**" for root in ORACLE_ROOTS)
SNAPSHOT_RECIPE = "git-archive-excluding-parity-oracle-roots-v1"
PATH_BLOB_DIGEST_ALGORITHM = "sha256(sorted-raw-path-nul-git-blob-oid-lf-v1)"
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
DRIVE_COMPONENT = re.compile(r"^[A-Za-z]:")
RUN_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
COMMAND_TIMEOUT_SECONDS = 1800
SOURCE_FACTS_LIMIT_BYTES = 4 * 1024 * 1024


class SubjectError(ValueError):
    """A pinned subject could not be resolved or built without ambiguity."""


class TagBuildError(SubjectError):
    """A frozen-tag build failed or was blocked, with its recorded outcome."""

    def __init__(self, message: str, build_result: str) -> None:
        super().__init__(message)
        self.build_result = build_result


@dataclass(frozen=True)
class CommandResult:
    returncode: int
    stdout: bytes
    stderr: bytes
    timed_out: bool = False


@dataclass(frozen=True)
class TargetPackage:
    app: str
    package_id: str
    package_name: str
    package_version: str
    manifest_path: Path
    source_path: Path


@dataclass(frozen=True)
class SourceFactsPins:
    sha256: str
    observed_at: str
    method: str
    reference_branch: str
    reference_commit: str
    candidate_branch: str
    candidate_commit: str


Runner = Callable[[Sequence[str], Path, Mapping[str, str], int], CommandResult]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SubjectError(message)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    try:
        return sha256_bytes(path.read_bytes())
    except OSError as error:
        raise SubjectError("cannot hash {}: {}".format(path, error)) from error


def _reject_duplicate_json_keys(pairs: Sequence[Tuple[str, Any]]) -> Dict[str, Any]:
    value: Dict[str, Any] = {}
    for key, child in pairs:
        require(key not in value, "source-facts JSON contains a duplicate key: {}".format(key))
        value[key] = child
    return value


def _read_source_facts(path: Path, expected_sha256: str) -> SourceFactsPins:
    _validate_digest(expected_sha256, "source_facts_sha256")
    require(hasattr(os, "O_NOFOLLOW"), "platform must support no-follow source-facts reads")
    path = path.expanduser().absolute()
    flags = os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)
    try:
        descriptor = os.open(str(path), flags)
    except OSError as error:
        raise SubjectError("cannot open source-facts file without following symlinks: {}".format(error)) from error
    try:
        before = os.fstat(descriptor)
        require(stat.S_ISREG(before.st_mode), "source-facts input must be a regular file")
        require(before.st_size <= SOURCE_FACTS_LIMIT_BYTES,
                "source-facts input exceeds the bounded size limit")
        chunks: List[bytes] = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(65536, SOURCE_FACTS_LIMIT_BYTES + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            require(total <= SOURCE_FACTS_LIMIT_BYTES,
                    "source-facts input exceeds the bounded size limit")
        after = os.fstat(descriptor)
        require(
            (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns)
            == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns)
            and total == before.st_size,
            "source-facts input changed while it was being read",
        )
    except OSError as error:
        raise SubjectError("cannot read source-facts file: {}".format(error)) from error
    finally:
        os.close(descriptor)

    raw = b"".join(chunks)
    actual_sha256 = sha256_bytes(raw)
    require(actual_sha256 == expected_sha256,
            "source_facts_sha256 does not match the exact source-facts bytes")
    try:
        payload = json.loads(
            raw.decode("utf-8"),
            object_pairs_hook=_reject_duplicate_json_keys,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SubjectError("source-facts input is not valid UTF-8 JSON: {}".format(error)) from error
    require(isinstance(payload, dict), "source-facts root must be a JSON object")
    require(type(payload.get("schema_version")) is int and payload["schema_version"] == 2,
            "source-facts schema_version must be 2")
    latest = payload.get("latest_source_observation")
    require(isinstance(latest, dict),
            "source-facts latest_source_observation is required")
    reference = latest.get("reference_remote")
    candidate = latest.get("candidate_remote")
    require(isinstance(reference, dict) and isinstance(candidate, dict),
            "source-facts latest remote branch observations are required")
    reference_branch = reference.get("branch")
    candidate_branch = candidate.get("branch")
    reference_commit = reference.get("head_sha")
    candidate_commit = candidate.get("head_sha")
    require(reference_branch == "visual-baseline",
            "source-facts reference branch must be visual-baseline")
    require(candidate_branch == "termrock-implementation",
            "source-facts candidate branch must be termrock-implementation")
    _validate_commit(reference_commit, "source-facts reference head_sha")
    _validate_commit(candidate_commit, "source-facts candidate head_sha")
    observed_at = latest.get("observed_at")
    method = latest.get("method")
    require(isinstance(observed_at, str) and observed_at.strip(),
            "source-facts observation timestamp is required")
    require(isinstance(method, str) and method.strip(),
            "source-facts observation method is required")
    return SourceFactsPins(
        sha256=actual_sha256,
        observed_at=observed_at,
        method=method,
        reference_branch=reference_branch,
        reference_commit=reference_commit,
        candidate_branch=candidate_branch,
        candidate_commit=candidate_commit,
    )


def canonical_receipt_sha256(receipt_payload: Mapping[str, Any]) -> str:
    """Match serde_json's compact, sorted-key receipt digest contract."""
    require("sha256" not in receipt_payload, "receipt payload must omit sha256")
    encoded = json.dumps(
        receipt_payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    return sha256_bytes(encoded)


def _atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=".{}.".format(path.name), dir=str(path.parent))
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        os.replace(str(temporary), str(path))
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def _atomic_json(path: Path, value: Any) -> None:
    data = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False).encode("utf-8") + b"\n"
    _atomic_write(path, data)


def _default_runner(
    argv: Sequence[str], cwd: Path, env: Mapping[str, str], timeout: int
) -> CommandResult:
    try:
        result = subprocess.run(
            list(argv),
            cwd=str(cwd),
            env=dict(env),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
        return CommandResult(result.returncode, result.stdout, result.stderr)
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        if isinstance(stdout, str):
            stdout = stdout.encode("utf-8", "replace")
        if isinstance(stderr, str):
            stderr = stderr.encode("utf-8", "replace")
        return CommandResult(124, stdout, stderr + b"\ncommand timed out\n", True)
    except OSError as error:
        return CommandResult(127, b"", (str(error) + "\n").encode("utf-8", "replace"))


def _recorded_command(
    argv: Sequence[str],
    cwd: Path,
    env: Mapping[str, str],
    evidence_dir: Path,
    label: str,
    runner: Runner = _default_runner,
    timeout: int = COMMAND_TIMEOUT_SECONDS,
    on_runner_start: Optional[Callable[[], None]] = None,
) -> CommandResult:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    _atomic_json(evidence_dir / (label + ".argv.json"), list(argv))
    if on_runner_start is not None:
        on_runner_start()
    result = runner(argv, cwd, env, timeout)
    _atomic_write(evidence_dir / (label + ".stdout"), result.stdout)
    _atomic_write(evidence_dir / (label + ".stderr"), result.stderr)
    exit_text = "timeout\n" if result.timed_out else str(result.returncode) + "\n"
    _atomic_write(evidence_dir / (label + ".exit"), exit_text.encode("ascii"))
    return result


def _command_env() -> Dict[str, str]:
    """Remove ambient compiler routing and wrapper paths from child commands."""
    env = dict(os.environ)
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
    for key in list(env):
        if (
            key in {
                "RUSTUP_TOOLCHAIN",
                "RUSTFLAGS",
                "RUSTDOCFLAGS",
                "RUSTC",
                "RUSTC_WRAPPER",
                "RUSTC_WORKSPACE_WRAPPER",
                "RUSTC_BOOTSTRAP",
                "CARGO_BUILD_RUSTC",
                "CARGO_BUILD_RUSTC_WRAPPER",
                "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
                "CARGO_TARGET_DIR",
                "CARGO_BUILD_JOBS",
                "CARGO_BUILD_TARGET",
                "CARGO_ENCODED_RUSTFLAGS",
            }
            or (key.startswith("CARGO_TARGET_") and key.endswith("_RUNNER"))
            or key.startswith("CARGO_PROFILE_")
            or key.startswith("GIT_")
        ):
            env.pop(key, None)
    return env


def _git_command_env() -> Dict[str, str]:
    """Start from the sanitized environment and disable replacement/lazy fetch."""
    env = _command_env()
    env["GIT_NO_REPLACE_OBJECTS"] = "1"
    env["GIT_NO_LAZY_FETCH"] = "1"
    return env


def verify_schema(path: Path = SUBJECT_SCHEMA_PATH) -> str:
    digest = sha256_file(path)
    require(
        digest == SUBJECT_SCHEMA_SHA256,
        "subject schema digest changed: expected {}, found {}".format(
            SUBJECT_SCHEMA_SHA256, digest
        ),
    )
    return digest


def _inside(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except ValueError:
        return False


def _assert_no_ancestor_cargo_config(snapshot_root: Path) -> None:
    """Reject ambient config Cargo could load above an expected snapshot root."""
    snapshot_root = snapshot_root.resolve()
    ancestor = snapshot_root.parent
    while True:
        for name in ("config", "config.toml"):
            candidate = ancestor / ".cargo" / name
            require(
                not candidate.exists() and not candidate.is_symlink(),
                "Cargo config outside pinned source snapshot can affect the build: {}".format(candidate),
            )
        if ancestor.parent == ancestor:
            break
        ancestor = ancestor.parent


def _cargo_config_facts(snapshot_root: Path) -> Dict[str, str]:
    """Reject ambient ancestor Cargo config; report the commit-bound local config."""
    snapshot_root = snapshot_root.resolve()
    _assert_no_ancestor_cargo_config(snapshot_root)

    facts: Dict[str, str] = {}
    local_config_dir = snapshot_root / ".cargo"
    for name in ("config", "config.toml"):
        candidate = local_config_dir / name
        if candidate.exists() or candidate.is_symlink():
            require(candidate.is_file(), "pinned Cargo config is not a regular file: {}".format(candidate))
            require(_inside(candidate, snapshot_root), "pinned Cargo config escaped its source snapshot")
            facts[name] = sha256_file(candidate)
    return facts


def resolve_binary_targets(metadata: Any, snapshot_root: Path) -> Dict[str, TargetPackage]:
    require(isinstance(metadata, dict), "cargo metadata output must be an object")
    require(metadata.get("version") == 1, "unsupported cargo metadata format")
    packages = metadata.get("packages")
    require(isinstance(packages, list), "cargo metadata packages must be an array")
    require(
        Path(str(metadata.get("workspace_root", ""))).resolve() == snapshot_root.resolve(),
        "cargo metadata workspace_root does not match the pinned source snapshot",
    )

    matches: Dict[str, List[TargetPackage]] = {app: [] for app in APPS}
    for package in packages:
        if not isinstance(package, dict):
            raise SubjectError("cargo metadata contains a malformed package")
        package_id = package.get("id")
        package_name = package.get("name")
        package_version = package.get("version")
        manifest = package.get("manifest_path")
        targets = package.get("targets")
        require(
            all(isinstance(value, str) and value for value in (package_id, package_name, package_version, manifest)),
            "cargo metadata package identity is incomplete",
        )
        require(isinstance(targets, list), "cargo metadata targets must be an array")
        manifest_path = Path(manifest).resolve()
        require(_inside(manifest_path, snapshot_root), "package manifest escaped the source snapshot")
        for target in targets:
            if not isinstance(target, dict):
                raise SubjectError("cargo metadata contains a malformed target")
            name = target.get("name")
            kind = target.get("kind")
            source = target.get("src_path")
            if not isinstance(name, str) or not isinstance(kind, list) or "bin" not in kind:
                continue
            if name not in matches:
                continue
            require(isinstance(source, str) and source, "binary target source path is missing")
            source_path = Path(source).resolve()
            require(_inside(source_path, snapshot_root), "binary target source escaped the source snapshot")
            matches[name].append(
                TargetPackage(
                    app=name,
                    package_id=package_id,
                    package_name=package_name,
                    package_version=package_version,
                    manifest_path=manifest_path,
                    source_path=source_path,
                )
            )

    resolved: Dict[str, TargetPackage] = {}
    for app, rows in matches.items():
        require(
            len(rows) == 1,
            "expected exactly one Cargo bin target named {}; found {}".format(app, len(rows)),
        )
        require(rows[0].source_path.is_file(), "binary source is missing for {}".format(app))
        require(rows[0].manifest_path.is_file(), "package manifest is missing for {}".format(app))
        resolved[app] = rows[0]
    return resolved


def parse_metadata_stdout(stdout: bytes) -> Any:
    try:
        return json.loads(stdout.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SubjectError("cargo metadata did not return valid JSON: {}".format(error)) from error


def parse_compiler_artifact(
    stdout: bytes,
    package_id: str,
    target_name: str,
    target_dir: Path,
) -> Path:
    try:
        text = stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SubjectError("cargo build stdout is not UTF-8 JSON") from error
    artifacts: List[Mapping[str, Any]] = []
    for line_number, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise SubjectError(
                "cargo build stdout line {} is not machine-readable JSON: {}".format(line_number, error)
            ) from error
        if not isinstance(event, dict) or event.get("reason") != "compiler-artifact":
            continue
        target = event.get("target")
        if not isinstance(target, dict):
            continue
        if (
            event.get("package_id") == package_id
            and target.get("name") == target_name
            and isinstance(target.get("kind"), list)
            and "bin" in target["kind"]
        ):
            artifacts.append(event)

    require(
        len(artifacts) == 1,
        "expected one matching compiler-artifact for {}::{}, found {}".format(
            package_id, target_name, len(artifacts)
        ),
    )
    artifact = artifacts[0]
    require(artifact.get("fresh") is not True, "Cargo reported a fresh artifact from a non-fresh target directory")
    profile = artifact.get("profile")
    require(isinstance(profile, dict) and profile.get("test") is False,
            "matching compiler-artifact is not a non-test product build")
    executable_value = artifact.get("executable")
    require(isinstance(executable_value, str) and executable_value,
            "matching compiler-artifact has no executable path")
    executable = Path(executable_value)
    require(executable.is_absolute(), "compiler-artifact executable path must be absolute")
    executable = executable.resolve()
    require(_inside(executable, target_dir), "compiler-artifact executable escaped its isolated target directory")
    require(executable.is_file(), "compiler-artifact executable does not exist")
    require(os.access(str(executable), os.X_OK), "compiler-artifact executable is not runnable")
    return executable


def _validate_run_id(value: str) -> str:
    require(RUN_ID.fullmatch(value) is not None and value not in {".", ".."},
            "run_id must be a safe path segment")
    return value


def _validate_digest(value: Any, name: str) -> str:
    require(
        isinstance(value, str) and HEX_64.fullmatch(value) is not None,
        "{} must be a lowercase SHA-256".format(name),
    )
    return value


def _validate_commit(value: Any, name: str) -> str:
    require(
        isinstance(value, str) and HEX_40.fullmatch(value) is not None,
        "{} must be a lowercase full commit SHA".format(name),
    )
    return value


def _validate_git_oid(value: Any, name: str) -> str:
    require(
        isinstance(value, str) and HEX_40.fullmatch(value) is not None,
        "{} must be a lowercase 40-character Git object ID".format(name),
    )
    return value


def _portable_archive_path(raw_path: str, *, directory: bool = False) -> PurePosixPath:
    """Reject archive paths that become absolute or ambiguous on Windows."""
    require(bool(raw_path) and "\0" not in raw_path,
            "source archive contains an unsafe path")
    require("\\" not in raw_path and not raw_path.startswith("/"),
            "source archive contains a non-portable path")
    path_text = raw_path
    if directory and path_text.endswith("/"):
        path_text = path_text[:-1]
    require(bool(path_text) and not path_text.endswith("/"),
            "source archive contains an unsafe path")
    components = path_text.split("/")
    require(
        all(component not in {"", ".", ".."} for component in components),
        "source archive contains an unsafe path",
    )
    require(
        all(DRIVE_COMPONENT.match(component) is None and ":" not in component
            for component in components),
        "source archive contains a non-portable path",
    )
    return PurePosixPath(path_text)


def _validate_symlink_target(member_path: PurePosixPath, target: str) -> None:
    """Check a relative symlink target under both POSIX and Windows rules."""
    require(bool(target) and "\0" not in target,
            "source archive has an unsafe symlink target")
    require("\\" not in target and not target.startswith("/"),
            "source archive has a non-portable symlink target")
    components = target.split("/")
    require(
        all(component != "" for component in components),
        "source archive has an unsafe symlink target",
    )
    require(
        all(DRIVE_COMPONENT.match(component) is None and ":" not in component
            for component in components),
        "source archive has a non-portable symlink target",
    )
    resolved = list(member_path.parent.parts)
    for component in components:
        if component == ".":
            continue
        if component == "..":
            require(bool(resolved), "source archive symlink escapes the snapshot")
            resolved.pop()
        else:
            resolved.append(component)


def _safe_tar_members(
    archive: tarfile.TarFile,
) -> Tuple[List[tarfile.TarInfo], set, set]:
    members = archive.getmembers()
    seen = set()
    member_names = set()
    file_names = set()
    for member in members:
        path = _portable_archive_path(member.name, directory=member.isdir())
        normalized_name = path.as_posix().rstrip("/")
        require(normalized_name not in seen, "source archive contains a duplicate path")
        seen.add(normalized_name)
        member_names.add(normalized_name)
        if member.isdir():
            continue
        require(member.isfile() or member.issym(), "source archive contains an unsupported entry")
        if member.issym():
            _validate_symlink_target(path, member.linkname)
        file_names.add(normalized_name)
    return members, file_names, member_names


def _expected_archive_member_paths(blob_ids: Mapping[str, str]) -> set:
    """Return included blob paths plus every tracked parent directory."""
    paths = set(blob_ids)
    for path in blob_ids:
        parts = PurePosixPath(path).parts
        for index in range(1, len(parts)):
            paths.add("/".join(parts[:index]))
    return paths


def _tracked_blob_ids(stdout: bytes) -> Dict[str, str]:
    paths: Dict[str, str] = {}
    for record in stdout.split(b"\0"):
        if not record:
            continue
        try:
            metadata, path = record.split(b"\t", 1)
            mode, kind, object_id = metadata.decode("ascii").split(" ", 2)
            decoded = path.decode("utf-8", "surrogateescape")
        except (ValueError, UnicodeDecodeError) as error:
            raise SubjectError("git ls-tree returned malformed output") from error
        require(mode != "160000" and kind != "commit", "pinned source contains an unmaterialized submodule")
        require(kind == "blob" and HEX_40.fullmatch(object_id) is not None,
                "pinned source tree has a non-blob leaf or unsupported object id")
        require(decoded not in paths, "git ls-tree returned a duplicate path")
        paths[decoded] = object_id
    return paths


def _partition_tracked_blob_ids(paths: Mapping[str, str]) -> Tuple[Dict[str, str], Dict[str, str]]:
    """Separate the two declared oracle trees from all other tracked blobs."""
    included: Dict[str, str] = {}
    excluded: Dict[str, str] = {}
    for path, object_id in paths.items():
        if any(path.startswith(root + "/") for root in ORACLE_ROOTS):
            excluded[path] = object_id
        else:
            included[path] = object_id
    return included, excluded


def _path_blob_map_sha256(paths: Mapping[str, str]) -> str:
    """Hash sorted raw Git paths and blob IDs with unambiguous delimiters."""
    digest = hashlib.sha256()
    for path in sorted(paths, key=lambda item: item.encode("utf-8", "surrogateescape")):
        digest.update(path.encode("utf-8", "surrogateescape"))
        digest.update(b"\0")
        digest.update(paths[path].encode("ascii"))
        digest.update(b"\n")
    return digest.hexdigest()


def _git_blob_id(data: bytes) -> str:
    header = b"blob " + str(len(data)).encode("ascii") + b"\0"
    return hashlib.sha1(header + data).hexdigest()


def _open_absolute_nofollow(path: Path, *, directory: bool = False) -> int:
    """Open an absolute path one component at a time without following links."""
    require(hasattr(os, "O_NOFOLLOW") and hasattr(os, "O_DIRECTORY"),
            "platform must support descriptor-relative no-follow validation")
    require(path.is_absolute(), "pinned path must be absolute")
    parts = path.parts
    require(parts and parts[0] == "/" and all(part not in {"", ".", ".."} for part in parts[1:]),
            "pinned path contains an unsafe component")
    flags = os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)
    parent = os.open("/", flags | os.O_DIRECTORY)
    try:
        for index, component in enumerate(parts[1:]):
            final = index == len(parts) - 2
            want_directory = not final or directory
            before = os.stat(component, dir_fd=parent, follow_symlinks=False)
            require(not stat.S_ISLNK(before.st_mode), "pinned path contains a symlink component")
            require(stat.S_ISDIR(before.st_mode) if want_directory else stat.S_ISREG(before.st_mode),
                    "pinned path has an unexpected filesystem type")
            child = os.open(
                component,
                flags | (os.O_DIRECTORY if want_directory else 0),
                dir_fd=parent,
            )
            after = os.fstat(child)
            require((before.st_dev, before.st_ino, stat.S_IFMT(before.st_mode))
                    == (after.st_dev, after.st_ino, stat.S_IFMT(after.st_mode)),
                    "pinned path component changed while it was opened")
            os.close(parent)
            parent = child
        if not parts[1:]:
            require(directory, "filesystem root is not a regular file")
            return parent
        result = parent
        parent = -1
        return result
    finally:
        if parent >= 0:
            os.close(parent)


def _read_absolute_nofollow(path: Path, *, maximum: int) -> bytes:
    descriptor = _open_absolute_nofollow(path)
    try:
        before = os.fstat(descriptor)
        require(before.st_size <= maximum, "pinned file exceeds its bounded size limit")
        chunks: List[bytes] = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(1024 * 1024, maximum + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            require(total <= maximum, "pinned file exceeds its bounded size limit")
        after = os.fstat(descriptor)
        require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
                == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns)
                and total == before.st_size,
                "pinned file changed while it was being read")
        return b"".join(chunks)
    except OSError as error:
        raise SubjectError("cannot read pinned file without following symlinks: {}".format(error)) from error
    finally:
        os.close(descriptor)


def _hash_absolute_nofollow(
    path: Path, *, algorithm: str, maximum: int, require_executable: bool = False
) -> str:
    descriptor = _open_absolute_nofollow(path)
    try:
        before = os.fstat(descriptor)
        if require_executable:
            require(bool(before.st_mode & 0o111), "pinned executable has no executable permission bits")
        require(before.st_size <= maximum, "pinned executable exceeds its bounded size limit")
        digest = hashlib.new(algorithm)
        total = 0
        while True:
            chunk = os.read(descriptor, 1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
            total += len(chunk)
            require(total <= maximum, "pinned executable exceeds its bounded size limit")
        after = os.fstat(descriptor)
        require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns)
                == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns)
                and total == before.st_size,
                "pinned executable changed while it was being hashed")
        return digest.hexdigest()
    except OSError as error:
        raise SubjectError("cannot hash pinned executable without following symlinks: {}".format(error)) from error
    finally:
        os.close(descriptor)


def _parse_json_object(raw: bytes, name: str) -> Dict[str, Any]:
    try:
        payload = json.loads(raw.decode("utf-8"), object_pairs_hook=_reject_duplicate_json_keys)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SubjectError("{} is not valid UTF-8 JSON: {}".format(name, error)) from error
    require(isinstance(payload, dict), "{} root must be a JSON object".format(name))
    return payload


def _parse_json_value(raw: bytes, name: str) -> Any:
    try:
        return json.loads(raw.decode("utf-8"), object_pairs_hook=_reject_duplicate_json_keys)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SubjectError("{} is not valid UTF-8 JSON: {}".format(name, error)) from error


def _closed_object(value: Any, keys: set, name: str) -> Dict[str, Any]:
    require(isinstance(value, dict) and set(value) == keys,
            "{} fields differ from the closed evidence schema".format(name))
    return value


def _require_text(value: Any, name: str, *, absolute: bool = False) -> str:
    require(isinstance(value, str) and bool(value.strip()), "{} must be nonempty text".format(name))
    if absolute:
        path = Path(value)
        require(path.is_absolute(), "{} must be an absolute path".format(name))
        require(all(part not in {".", ".."} for part in path.parts[1:]),
                "{} contains an unsafe path component".format(name))
    return value


def _require_count(value: Any, name: str, *, allow_zero: bool = False) -> int:
    require(type(value) is int and value >= (0 if allow_zero else 1),
            "{} must be a bounded nonnegative count".format(name))
    require(value <= 10_000_000, "{} exceeds the supported entry limit".format(name))
    return value


def _is_strict_child(path: Path, parent: Path) -> bool:
    try:
        relative = path.relative_to(parent)
    except ValueError:
        return False
    return bool(relative.parts)


def _git_readonly(repository: Path, arguments: Sequence[str], *, maximum: int = 128 * 1024 * 1024) -> bytes:
    env = _git_command_env()
    env["GIT_TERMINAL_PROMPT"] = "0"
    env["GIT_CONFIG_NOSYSTEM"] = "1"
    env["GIT_CONFIG_GLOBAL"] = os.devnull
    require(env.get("GIT_NO_REPLACE_OBJECTS") == "1" and env.get("GIT_NO_LAZY_FETCH") == "1",
            "Git replacement objects and lazy fetching must be disabled")
    try:
        result = subprocess.run(
            ["git", "--no-replace-objects", *arguments],
            cwd=str(repository), env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            timeout=COMMAND_TIMEOUT_SECONDS, check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise SubjectError("pinned Git object query failed without fallback: {}".format(error)) from error
    require(result.returncode == 0, "pinned Git object query failed: {}".format(
        result.stderr.decode("utf-8", "replace")[-2000:]
    ))
    require(len(result.stdout) <= maximum, "pinned Git object query exceeded its bounded output limit")
    return result.stdout


def _validate_recorded_executable_argv(actual_argv: Any, expected_argv: Sequence[str], label: str) -> None:
    require(isinstance(actual_argv, list) and bool(actual_argv) and len(actual_argv) == len(expected_argv),
            "recorded {} argv has an unexpected shape".format(label))
    actual_executable = _require_text(
        actual_argv[0], "recorded {} executable path".format(label), absolute=True
    )
    pinned_executable = _require_text(
        expected_argv[0], "pinned {} executable path".format(label), absolute=True
    )
    try:
        actual_resolved = Path(actual_executable).resolve(strict=True)
        pinned_resolved = Path(pinned_executable).resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise SubjectError("recorded {} executable path cannot be resolved: {}".format(label, error)) from error
    require(str(pinned_resolved) == pinned_executable,
            "pinned {} executable path is not canonical".format(label))
    require(str(actual_resolved) == pinned_executable,
            "recorded {} executable path does not resolve to the pinned path".format(label))
    require(actual_argv[1:] == list(expected_argv[1:]),
            "recorded {} argv tail differs from the pinned query".format(label))


def _recorded_successful_command(
    run_root: Path, label: str, expected_argv: Sequence[str], *, maximum_stdout: int = 8 * 1024 * 1024
) -> bytes:
    evidence = run_root / "toolchain"
    argv = _parse_json_value(
        _read_absolute_nofollow(evidence / (label + ".argv.json"), maximum=1024 * 1024),
        label + " argv",
    )
    _validate_recorded_executable_argv(argv, expected_argv, label)
    require(_read_absolute_nofollow(evidence / (label + ".exit"), maximum=128) == b"0\n",
            "recorded {} query did not exit successfully".format(label))
    _read_absolute_nofollow(evidence / (label + ".stderr"), maximum=8 * 1024 * 1024)
    return _read_absolute_nofollow(evidence / (label + ".stdout"), maximum=maximum_stdout)


def _validate_recorded_path_output(raw: bytes, expected_path: Any, label: str) -> None:
    try:
        text = raw.decode("utf-8", "strict")
    except UnicodeDecodeError as error:
        raise SubjectError("recorded rustup {} path output is not UTF-8".format(label)) from error
    lines = text.splitlines()
    require(
        len(lines) == 1 and bool(lines[0]) and text in {lines[0] + "\n", lines[0] + "\r\n"},
        "recorded rustup {} path output must contain exactly one path line".format(label),
    )
    recorded_path = Path(
        _require_text(lines[0], "recorded rustup {} path output".format(label), absolute=True)
    )
    pinned_text = _require_text(
        expected_path, "pinned rustup {} path".format(label), absolute=True
    )
    pinned_path = Path(pinned_text)
    try:
        recorded_resolved = recorded_path.resolve(strict=True)
        pinned_resolved = pinned_path.resolve(strict=True)
    except (OSError, RuntimeError) as error:
        raise SubjectError(
            "recorded rustup {} path cannot be resolved: {}".format(label, error)
        ) from error
    require(
        str(pinned_resolved) == pinned_text,
        "pinned rustup {} path is not canonical".format(label),
    )
    require(
        str(recorded_resolved) == pinned_text,
        "recorded rustup {} path does not resolve to the pinned canonical path".format(label),
    )


def _git_tree_entries(raw: bytes) -> Dict[str, Tuple[str, str, str]]:
    entries: Dict[str, Tuple[str, str, str]] = {}
    for record in raw.split(b"\0"):
        if not record:
            continue
        try:
            metadata, path_bytes = record.split(b"\t", 1)
            mode_bytes, kind_bytes, oid_bytes = metadata.split(b" ", 2)
            mode, kind, oid = mode_bytes.decode("ascii"), kind_bytes.decode("ascii"), oid_bytes.decode("ascii")
            path = path_bytes.decode("utf-8", "surrogateescape")
        except (ValueError, UnicodeDecodeError) as error:
            raise SubjectError("git ls-tree returned a malformed raw entry") from error
        _portable_archive_path(path)
        require(mode in {"100644", "100755", "120000"} and kind == "blob" and HEX_40.fullmatch(oid) is not None,
                "pinned Git tree contains an unsupported mode or object type")
        require(path not in entries, "pinned Git tree contains a duplicate path")
        entries[path] = (mode, kind, oid)
        require(len(entries) <= 10_000_000, "pinned Git tree exceeds the supported entry limit")
    return entries


def _snapshot_path_map(root: Path, git_entries: Mapping[str, Tuple[str, str, str]]) -> Dict[str, str]:
    root_fd = _open_absolute_nofollow(root, directory=True)
    actual: Dict[str, str] = {}
    actual_directories = set()
    expected_directories = set()
    for path in git_entries:
        parts = PurePosixPath(path).parts
        for index in range(1, len(parts)):
            expected_directories.add("/".join(parts[:index]))

    def visit(directory_fd: int, parent: bytes) -> None:
        directory_stat = os.fstat(directory_fd)
        require(stat.S_ISDIR(directory_stat.st_mode) and not directory_stat.st_mode & 0o222,
                "snapshot directory is writable or has the wrong type")
        try:
            names = os.listdir(directory_fd)
        except OSError as error:
            raise SubjectError("cannot enumerate pinned snapshot directory: {}".format(error)) from error
        for name in names:
            name_bytes = os.fsencode(name)
            relative_bytes = name_bytes if not parent else parent + b"/" + name_bytes
            relative = relative_bytes.decode("utf-8", "surrogateescape")
            path = _portable_archive_path(relative)
            before = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
            if stat.S_ISDIR(before.st_mode):
                require(not before.st_mode & 0o222, "snapshot directory is writable")
                child_fd = os.open(
                    name,
                    os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | getattr(os, "O_CLOEXEC", 0),
                    dir_fd=directory_fd,
                )
                try:
                    child_stat = os.fstat(child_fd)
                    require((before.st_dev, before.st_ino) == (child_stat.st_dev, child_stat.st_ino),
                            "snapshot directory changed while it was opened")
                    actual_directories.add(path.as_posix())
                    visit(child_fd, relative_bytes)
                finally:
                    os.close(child_fd)
                continue

            require(relative not in actual, "snapshot contains a duplicate path")
            expected = git_entries.get(relative)
            require(expected is not None, "snapshot contains a path absent from the pinned Git tree")
            mode, kind, object_id = expected
            if stat.S_ISREG(before.st_mode):
                require(mode in {"100644", "100755"} and kind == "blob",
                        "snapshot file type differs from the pinned Git tree")
                require(not before.st_mode & 0o222, "snapshot file is writable")
                executable = bool(before.st_mode & 0o111)
                require(executable == (mode == "100755"),
                        "snapshot executable mode differs from the pinned Git tree")
                file_fd = os.open(
                    name,
                    os.O_RDONLY | os.O_NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0),
                    dir_fd=directory_fd,
                )
                try:
                    opened = os.fstat(file_fd)
                    require((before.st_dev, before.st_ino, stat.S_IFMT(before.st_mode))
                            == (opened.st_dev, opened.st_ino, stat.S_IFMT(opened.st_mode)),
                            "snapshot file changed while it was opened")
                    blob_hash = hashlib.sha1(b"blob " + str(opened.st_size).encode("ascii") + b"\0")
                    total = 0
                    while True:
                        chunk = os.read(file_fd, 1024 * 1024)
                        if not chunk:
                            break
                        total += len(chunk)
                        require(total <= opened.st_size, "snapshot file changed size while being read")
                        blob_hash.update(chunk)
                    after = os.fstat(file_fd)
                    require(total == opened.st_size
                            and (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns, opened.st_ctime_ns)
                            == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns),
                            "snapshot file changed while being read")
                    actual[relative] = blob_hash.hexdigest()
                finally:
                    os.close(file_fd)
            elif stat.S_ISLNK(before.st_mode):
                require(mode == "120000" and kind == "blob",
                        "snapshot symlink type differs from the pinned Git tree")
                target = os.readlink(name, dir_fd=directory_fd)
                target_bytes = os.fsencode(target)
                _validate_symlink_target(path, target)
                after = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
                require((before.st_dev, before.st_ino, stat.S_IFMT(before.st_mode), before.st_size,
                         before.st_mtime_ns, before.st_ctime_ns)
                        == (after.st_dev, after.st_ino, stat.S_IFMT(after.st_mode), after.st_size,
                            after.st_mtime_ns, after.st_ctime_ns),
                        "snapshot symlink changed while inspected")
                actual[relative] = _git_blob_id(target_bytes)
            else:
                raise SubjectError("snapshot contains an unsupported filesystem entry")
        directory_after = os.fstat(directory_fd)
        require((directory_stat.st_dev, directory_stat.st_ino, directory_stat.st_mode,
                 directory_stat.st_size, directory_stat.st_mtime_ns, directory_stat.st_ctime_ns)
                == (directory_after.st_dev, directory_after.st_ino, directory_after.st_mode,
                    directory_after.st_size, directory_after.st_mtime_ns, directory_after.st_ctime_ns),
                "snapshot directory changed during closure validation")

    try:
        root_stat = os.fstat(root_fd)
        require(not root_stat.st_mode & 0o222, "snapshot root is writable")
        visit(root_fd, b"")
        root_after = os.fstat(root_fd)
        require((root_stat.st_dev, root_stat.st_ino, root_stat.st_mtime_ns)
                == (root_after.st_dev, root_after.st_ino, root_after.st_mtime_ns),
                "snapshot root changed during closure validation")
    finally:
        os.close(root_fd)
    require(actual_directories == expected_directories,
            "snapshot directory set differs from directories implied by the pinned Git tree")
    require(set(actual) == set(git_entries),
            "snapshot leaf path set differs from the pinned Git tree")
    return actual


def validate_frozen_tag_holla_bundle(
    *, repository: Path, receipt_path: Path, expected_receipt_sha256: str
) -> Dict[str, Any]:
    """Validate tag receipt consistency and immutable source closure; never authenticates build execution."""
    _validate_digest(expected_receipt_sha256, "expected tag receipt SHA-256")
    require(repository.is_absolute(), "tag validation repository path must be absolute")
    repository_fd = _open_absolute_nofollow(repository, directory=True)
    os.close(repository_fd)
    require(receipt_path.is_absolute(), "tag receipt path must be absolute")
    require(receipt_path.name == "tag-builder-receipt.json",
            "tag receipt must use the fixed tag-builder-receipt.json basename")
    receipt_raw = _read_absolute_nofollow(receipt_path, maximum=8 * 1024 * 1024)
    receipt_sha256 = sha256_bytes(receipt_raw)
    require(receipt_sha256 == expected_receipt_sha256,
            "tag receipt raw SHA-256 does not match the caller pin")
    receipt = _parse_json_object(receipt_raw, "tag builder receipt")
    _closed_object(receipt, {
        "schema", "run_id", "purpose", "build_result", "qualification", "oracle_lineage",
        "source_snapshot", "build_artifact_root", "build", "executable", "builder_receipt",
        "builder_receipt_sha256", "build_environment", "capture_status", "admission_status",
    }, "tag builder receipt")
    require(receipt["schema"] == TAG_BUILDER_RECEIPT_SCHEMA,
            "unexpected immutable-tag builder receipt schema")
    run_id = _validate_run_id(receipt["run_id"])
    require(receipt["purpose"] == "frozen-visual-tag-holla-build-only" and receipt["build_result"] == "PASS",
            "tag receipt does not describe a successful Holla build")
    qualification = _closed_object(receipt["qualification"], {"status", "reason"}, "tag qualification")
    require(qualification["status"] == "blocked" and isinstance(qualification["reason"], str)
            and qualification["reason"].strip(), "tag qualification must remain blocked with a reason")
    require(receipt["capture_status"] == "NOT_RUN" and receipt["admission_status"] == "NOT_RUN",
            "tag receipt cannot claim capture or admission")

    run_root = receipt_path.parent
    run_path = run_root / "run.json"
    run_raw = _read_absolute_nofollow(run_path, maximum=16 * 1024 * 1024)
    run_sha256 = sha256_bytes(run_raw)
    run = _parse_json_object(run_raw, "tag builder run record")
    run_keys = {
        "schema", "state", "phase", "build_result", "build_attempted", "qualification", "run_id",
        "purpose", "app", "target_triple", "tag_ref", "expected_tag_object", "expected_tag_commit",
        "capture_status", "admission_status", "oracle_lineage", "git_object_replacement_policy",
        "git_lazy_fetch_policy", "rustup_path", "rustup_sha256", "rustup_home", "toolchain_binaries",
        "cargo_version", "rustc_version", "host_triple", "source_snapshot", "cargo_config_sha256",
        "build_environment", "metadata_targets", "build_artifact_root", "tag_builder_receipt",
        "tag_builder_receipt_sha256", "executable_path", "executable_sha256",
    }
    _closed_object(run, run_keys, "tag builder run record")
    require(run["schema"] == TAG_BUILDER_RUN_SCHEMA and run["state"] == "complete"
            and run["phase"] == "complete" and run["build_result"] == "PASS"
            and run["build_attempted"] is True,
            "tag run record is not a complete successful build")
    require(run["run_id"] == run_id and run["purpose"] == receipt["purpose"] and run["app"] == "holla",
            "tag receipt and run record identities disagree")
    require(run["tag_builder_receipt"] == str(receipt_path)
            and run["tag_builder_receipt_sha256"] == receipt_sha256,
            "tag run record does not cross-bind the exact receipt bytes")
    require(run["capture_status"] == "NOT_RUN" and run["admission_status"] == "NOT_RUN",
            "tag run record cannot claim capture or admission")
    run_qualification = _closed_object(run["qualification"], {"status", "reason"}, "run qualification")
    require(run_qualification == qualification, "tag receipt and run qualification disagree")

    lineage = _closed_object(receipt["oracle_lineage"], {
        "tag_ref", "tag_object", "tag_commit", "git_object_replacement_policy", "git_lazy_fetch_policy",
    }, "oracle lineage")
    require(lineage == run["oracle_lineage"], "tag receipt and run oracle lineage disagree")
    require(lineage == {
        "tag_ref": "refs/tags/" + REFERENCE_TAG,
        "tag_object": REFERENCE_TAG_OBJECT,
        "tag_commit": REFERENCE_ORACLE_COMMIT,
        "git_object_replacement_policy": "disabled-by-option-and-environment",
        "git_lazy_fetch_policy": "disabled-by-environment",
    }, "immutable oracle lineage differs from the pinned tag")
    require(run["expected_tag_object"] == REFERENCE_TAG_OBJECT
            and run["expected_tag_commit"] == REFERENCE_ORACLE_COMMIT
            and run["tag_ref"] == "refs/tags/" + REFERENCE_TAG
            and run["git_object_replacement_policy"] == lineage["git_object_replacement_policy"]
            and run["git_lazy_fetch_policy"] == lineage["git_lazy_fetch_policy"],
            "tag run record uses an unexpected object-resolution policy")

    snapshot = receipt["source_snapshot"]
    snapshot_keys = {
        "source_commit", "tree_oid", "git_object_replacement_policy", "recipe", "archive_argv",
        "git_ls_tree_stdout_sha256", "excluded_roots", "path_blob_digest_algorithm", "tracked_path_blob_map_sha256",
        "tracked_file_count", "included_path_blob_map_sha256", "included_file_count",
        "excluded_path_blob_map_sha256", "excluded_file_count", "archive_sha256", "archive_member_count",
        "read_only", "materialized_root",
    }
    _closed_object(snapshot, snapshot_keys, "tag source snapshot")
    require(snapshot == run["source_snapshot"], "tag receipt and run snapshot records disagree")
    require(snapshot["source_commit"] == REFERENCE_ORACLE_COMMIT
            and snapshot["git_object_replacement_policy"] == lineage["git_object_replacement_policy"]
            and snapshot["recipe"] == SNAPSHOT_RECIPE
            and snapshot["excluded_roots"] == list(ORACLE_EXCLUSIONS)
            and snapshot["path_blob_digest_algorithm"] == PATH_BLOB_DIGEST_ALGORITHM
            and snapshot["read_only"] is True,
            "tag snapshot policy differs from the frozen materializer")
    _validate_git_oid(snapshot["tree_oid"], "snapshot tree_oid")
    for name in ("git_ls_tree_stdout_sha256", "tracked_path_blob_map_sha256",
                 "included_path_blob_map_sha256", "excluded_path_blob_map_sha256", "archive_sha256"):
        _validate_digest(snapshot[name], "snapshot {}".format(name))
    for name in ("tracked_file_count", "included_file_count", "excluded_file_count", "archive_member_count"):
        _require_count(snapshot[name], "snapshot {}".format(name), allow_zero=True)
    source_root = Path(_require_text(snapshot["materialized_root"], "snapshot materialized_root", absolute=True))
    expected_source_root = run_root / "source" / "oracle"
    require(source_root == expected_source_root and run["source_snapshot"] == snapshot,
            "tag snapshot root does not match the run-owned source path")

    expected_archive_argv = [
        "git", "--no-replace-objects", "archive", "--format=tar", REFERENCE_ORACLE_COMMIT,
        "--", ".", ":(top,exclude)snapshots/**", ":(top,exclude)baselines/**",
    ]
    require(snapshot["archive_argv"] == expected_archive_argv,
            "tag snapshot archive argv differs from the pinned recipe")
    git_env = _git_command_env()
    git_env["GIT_TERMINAL_PROMPT"] = "0"
    git_env["GIT_CONFIG_NOSYSTEM"] = "1"
    git_env["GIT_CONFIG_GLOBAL"] = os.devnull
    require(git_env.get("GIT_NO_REPLACE_OBJECTS") == "1" and git_env.get("GIT_NO_LAZY_FETCH") == "1",
            "Git replacement objects and lazy fetching must be disabled")
    tag_type = _git_readonly(repository, ["cat-file", "-t", REFERENCE_TAG_OBJECT], maximum=1024)
    require(tag_type.strip() == b"tag", "pinned oracle object is not an annotated tag")
    tag_raw = _git_readonly(repository, ["cat-file", "-p", REFERENCE_TAG_OBJECT], maximum=1024 * 1024)
    require(tag_raw.startswith(
        ("object {}\ntype commit\ntag {}\n".format(REFERENCE_ORACLE_COMMIT, REFERENCE_TAG)).encode("ascii")
    ), "immutable tag object does not name the pinned oracle commit")
    commit_type = _git_readonly(repository, ["cat-file", "-t", REFERENCE_ORACLE_COMMIT], maximum=1024)
    require(commit_type.strip() == b"commit", "pinned oracle commit object is unavailable")
    tree_oid_raw = _git_readonly(repository, ["rev-parse", "--verify", REFERENCE_ORACLE_COMMIT + "^{tree}"], maximum=1024)
    tree_oid = tree_oid_raw.decode("ascii", "strict").strip()
    _validate_commit(tree_oid, "pinned oracle tree oid")
    require(tree_oid == snapshot["tree_oid"], "tag snapshot tree oid differs from the immutable commit")
    tree_raw = _git_readonly(
        repository,
        ["ls-tree", "-rz", "-r", "--full-tree", REFERENCE_ORACLE_COMMIT],
        maximum=128 * 1024 * 1024,
    )
    require(sha256_bytes(tree_raw) == snapshot["git_ls_tree_stdout_sha256"],
            "tag snapshot Git tree output digest differs from the pinned run")
    entries = _git_tree_entries(tree_raw)
    full_blobs = {path: values[2] for path, values in entries.items()}
    included_blobs, excluded_blobs = _partition_tracked_blob_ids(full_blobs)
    require(_path_blob_map_sha256(full_blobs) == snapshot["tracked_path_blob_map_sha256"]
            and len(full_blobs) == snapshot["tracked_file_count"],
            "tag snapshot full Git tree map differs from its recorded facts")
    require(_path_blob_map_sha256(included_blobs) == snapshot["included_path_blob_map_sha256"]
            and len(included_blobs) == snapshot["included_file_count"],
            "tag snapshot included Git tree map differs from its recorded facts")
    require(_path_blob_map_sha256(excluded_blobs) == snapshot["excluded_path_blob_map_sha256"]
            and len(excluded_blobs) == snapshot["excluded_file_count"],
            "tag snapshot excluded Git tree map differs from its recorded facts")
    actual_blobs = _snapshot_path_map(
        source_root, {path: entry for path, entry in entries.items() if path in included_blobs}
    )
    _assert_no_ancestor_cargo_config(source_root)
    require(actual_blobs == included_blobs
            and _path_blob_map_sha256(actual_blobs) == snapshot["included_path_blob_map_sha256"],
            "materialized tag snapshot does not exactly match the immutable Git tree")
    expected_members = _expected_archive_member_paths(included_blobs)
    require(snapshot["archive_member_count"] == len(expected_members),
            "tag archive member count differs from the pinned filtered tree")

    snapshot_log = run_root / "snapshot-logs" / "oracle"
    archive_argv = _parse_json_value(_read_absolute_nofollow(snapshot_log / "archive.argv.json", maximum=1024 * 1024),
                                    "archive argv")
    require(archive_argv == expected_archive_argv, "recorded archive argv differs from the frozen command")
    require(_read_absolute_nofollow(snapshot_log / "archive.exit", maximum=128) == b"0\n",
            "tag snapshot archive did not exit successfully")
    tree_argv = _parse_json_value(_read_absolute_nofollow(snapshot_log / "tree.argv.json", maximum=1024 * 1024),
                                  "tree argv")
    require(tree_argv == ["git", "--no-replace-objects", "ls-tree", "-rz", "-r", "--full-tree",
                          REFERENCE_ORACLE_COMMIT], "recorded tree argv differs from the frozen command")
    require(_read_absolute_nofollow(snapshot_log / "tree.stdout", maximum=128 * 1024 * 1024) == tree_raw
            and _read_absolute_nofollow(snapshot_log / "tree.exit", maximum=128) == b"0\n",
            "recorded Git tree command output differs from the immutable object query")
    tree_oid_argv = _parse_json_value(_read_absolute_nofollow(snapshot_log / "tree-oid.argv.json", maximum=1024 * 1024),
                                      "tree oid argv")
    require(tree_oid_argv == ["git", "--no-replace-objects", "rev-parse", "--verify",
                              REFERENCE_ORACLE_COMMIT + "^{tree}"],
            "recorded tree oid argv differs from the frozen command")
    require(_read_absolute_nofollow(snapshot_log / "tree-oid.stdout", maximum=1024).decode("ascii").strip() == tree_oid
            and _read_absolute_nofollow(snapshot_log / "tree-oid.exit", maximum=128) == b"0\n",
            "recorded Git tree oid differs from the immutable object query")

    build = _closed_object(receipt["build"], {
        "package_id", "target_name", "features", "default_features", "target_triple", "toolchain", "profile",
    }, "tag build")
    executable = _closed_object(receipt["executable"], {"path", "sha256"}, "tag executable")
    require(build["target_name"] == "holla" and build["toolchain"] == TOOLCHAIN
            and build["profile"] == "release" and build["default_features"] is True
            and build["features"] == [] and build["target_triple"] == run["target_triple"],
            "tag build configuration is not the frozen Holla release build")
    target_triple = _require_text(build["target_triple"], "tag target triple")
    build_root = Path(_require_text(receipt["build_artifact_root"], "tag build artifact root", absolute=True))
    require(build_root == run_root / "targets" / "oracle" and run["build_artifact_root"] == str(build_root),
            "tag build artifact root differs from the run-owned target directory")
    executable_path = Path(_require_text(executable["path"], "tag executable path", absolute=True))
    run_executable_path = Path(_require_text(run["executable_path"], "run executable path", absolute=True))
    require(executable_path == run_executable_path and _is_strict_child(executable_path, build_root),
            "tag executable path differs from the completed build record")
    executable_sha = _validate_digest(executable["sha256"], "tag executable sha256")
    require(run["executable_sha256"] == executable_sha
            and _hash_absolute_nofollow(executable_path, algorithm="sha256", maximum=1024 * 1024 * 1024,
                                        require_executable=True)
            == executable_sha,
            "tag executable bytes do not match the completed build record")

    nested = _closed_object(receipt["builder_receipt"], {
        "schema", "source_commit", "package_id", "target_name", "requested_features", "default_features",
        "target_triple", "toolchain", "profile", "manifest_sha256", "lock_sha256", "mise_config",
        "cargo_version", "rustc_version", "host_triple", "executed_argv", "executable_path",
        "executable_sha256", "sha256",
    }, "nested Cargo builder receipt")
    nested_digest = _validate_digest(nested["sha256"], "nested builder receipt sha256")
    nested_payload = dict(nested)
    del nested_payload["sha256"]
    require(nested_digest == canonical_receipt_sha256(nested_payload)
            and receipt["builder_receipt_sha256"] == nested_digest,
            "nested builder receipt digest is invalid")
    require(nested["schema"] == BUILDER_RECEIPT_SCHEMA
            and nested["source_commit"] == REFERENCE_ORACLE_COMMIT
            and nested["package_id"] == build["package_id"]
            and nested["target_name"] == build["target_name"]
            and nested["requested_features"] == build["features"]
            and nested["default_features"] is True
            and nested["target_triple"] == target_triple
            and nested["toolchain"] == TOOLCHAIN and nested["profile"] == "release"
            and nested["executable_path"] == str(executable_path)
            and nested["executable_sha256"] == executable_sha,
            "nested builder receipt disagrees with the outer tag build identity")
    for digest_field in ("manifest_sha256", "lock_sha256"):
        _validate_digest(nested[digest_field], "nested builder " + digest_field)
    source_manifest = source_root / "Cargo.toml"
    source_lock = source_root / "Cargo.lock"
    require(_hash_absolute_nofollow(source_manifest, algorithm="sha256", maximum=16 * 1024 * 1024)
            == nested["manifest_sha256"], "tag package manifest hash differs from the pinned source")
    require(_hash_absolute_nofollow(source_lock, algorithm="sha256", maximum=64 * 1024 * 1024)
            == nested["lock_sha256"], "tag Cargo.lock hash differs from the pinned source")
    mise = _closed_object(nested["mise_config"], {"present", "path", "sha256"}, "tag mise config")
    require(type(mise["present"]) is bool, "tag mise config present must be boolean")
    if mise["present"]:
        require(mise["path"] == "mise.toml", "tag mise config path is not the pinned root file")
        _validate_digest(mise["sha256"], "tag mise config sha256")
        require(_hash_absolute_nofollow(source_root / "mise.toml", algorithm="sha256", maximum=4 * 1024 * 1024)
                == mise["sha256"], "tag mise config hash differs from the pinned source")
    else:
        require(mise["path"] is None and mise["sha256"] is None and not (source_root / "mise.toml").exists(),
                "tag mise config absence differs from the pinned source")

    env_record = _closed_object(receipt["build_environment"], {
        "cargo_config_sha256", "external_cargo_config_policy", "rustup", "toolchain_binaries",
        "cargo_version", "rustc_version", "host_triple", "target_triple", "environment",
    }, "tag build environment")
    require(env_record["external_cargo_config_policy"] == "reject-ancestor-config-and-hash-source-local-config",
            "tag Cargo configuration policy is unexpected")
    cargo_config = env_record["cargo_config_sha256"]
    require(isinstance(cargo_config, dict) and set(cargo_config) == {"oracle"}
            and cargo_config["oracle"] == run["cargo_config_sha256"],
            "tag Cargo config facts differ from the run record")
    source_configs: Dict[str, str] = {}
    for name in ("config", "config.toml"):
        candidate = source_root / ".cargo" / name
        if candidate.exists() or candidate.is_symlink():
            source_configs[name] = _hash_absolute_nofollow(candidate, algorithm="sha256", maximum=4 * 1024 * 1024)
    require(cargo_config["oracle"] == source_configs,
            "tag Cargo config facts differ from the immutable source tree")
    rustup_facts = _closed_object(env_record["rustup"], {"path", "sha256", "home"}, "tag rustup facts")
    _validate_digest(rustup_facts["sha256"], "tag rustup sha256")
    require(rustup_facts["path"] == run["rustup_path"] and rustup_facts["sha256"] == run["rustup_sha256"]
            and rustup_facts["home"] == run["rustup_home"], "tag rustup facts differ from the run record")
    require(_hash_absolute_nofollow(Path(rustup_facts["path"]), algorithm="sha256", maximum=128 * 1024 * 1024,
                                    require_executable=True)
            == rustup_facts["sha256"], "tag rustup executable digest differs from its recorded bytes")
    binaries = env_record["toolchain_binaries"]
    require(isinstance(binaries, dict) and set(binaries) == {"cargo", "rustc"}
            and binaries == run["toolchain_binaries"], "tag toolchain binaries differ from the run record")
    for name, binary in binaries.items():
        binary = _closed_object(binary, {"path", "sha256"}, "tag {} binary".format(name))
        _validate_digest(binary["sha256"], "tag {} binary sha256".format(name))
        require(_hash_absolute_nofollow(
            Path(_require_text(binary["path"], "tag {} binary path".format(name), absolute=True)),
            algorithm="sha256", maximum=512 * 1024 * 1024, require_executable=True
        ) == binary["sha256"],
                "tag {} binary digest differs from its recorded bytes".format(name))
    for field in ("cargo_version", "rustc_version", "host_triple"):
        _require_text(env_record[field], "tag " + field)
        require(env_record[field] == run[field] == nested[field],
                "tag {} facts differ across run and builder receipt".format(field))
    require(env_record["target_triple"] == target_triple and env_record["host_triple"] == target_triple,
            "tag requested target differs from the toolchain host")
    environment = env_record["environment"]
    require(isinstance(environment, dict) and set(environment) == {"oracle"}
            and environment == run["build_environment"],
            "tag isolated environment facts differ from the run record")
    facts = _closed_object(environment["oracle"], {
        "home", "cargo_home", "cargo_home_config", "cache_home", "cache_links", "rustup_home",
        "tmpdir", "path", "platform_inputs",
    }, "tag isolated environment facts")
    for key in ("home", "cargo_home", "cache_home", "rustup_home", "tmpdir"):
        _require_text(facts[key], "tag environment " + key, absolute=True)
    require(facts["cargo_home_config"] == "absent" and facts["rustup_home"] == rustup_facts["home"]
            and isinstance(facts["path"], str) and facts["path"],
            "tag isolated environment facts are inconsistent")
    normalized_path = facts["path"].lower().replace("\\", "/")
    require("/application support/mbx/bin" not in normalized_path
            and "/command-wrappers/bin" not in normalized_path,
            "tag build PATH contains a filtered command-wrapper directory")
    require(isinstance(facts["cache_links"], dict)
            and set(facts["cache_links"]).issubset({"registry", "git"}),
            "tag Cargo cache links have unexpected fields")
    cache_home = Path(facts["cache_home"])
    cache_fd = _open_absolute_nofollow(cache_home, directory=True)
    os.close(cache_fd)
    rustup_home_fd = _open_absolute_nofollow(Path(facts["rustup_home"]), directory=True)
    os.close(rustup_home_fd)
    for key, value in facts["cache_links"].items():
        _require_text(value, "tag cache link " + key, absolute=True)
        require(Path(value) == cache_home / key,
                "tag Cargo cache link target differs from its recorded cache root")
        cache_entry_fd = _open_absolute_nofollow(Path(value), directory=True)
        os.close(cache_entry_fd)
    require(isinstance(facts["platform_inputs"], dict)
            and set(facts["platform_inputs"]).issubset({"DEVELOPER_DIR", "SDKROOT", "SYSTEMROOT"})
            and all(isinstance(value, str) for value in facts["platform_inputs"].values()),
            "tag platform inputs have unexpected fields")
    cargo_home = Path(facts["cargo_home"])
    environment_root = run_root / "environment" / "oracle"
    require(Path(facts["home"]) == environment_root / "home"
            and cargo_home == environment_root / "cargo-home"
            and Path(facts["tmpdir"]) == environment_root / "tmp",
            "tag isolated environment paths differ from their run-owned roots")
    for directory_path in (Path(facts["home"]), cargo_home, Path(facts["tmpdir"])):
        directory_fd = _open_absolute_nofollow(directory_path, directory=True)
        os.close(directory_fd)
    cargo_home_fd = _open_absolute_nofollow(cargo_home, directory=True)
    try:
        for cache_name in ("registry", "git"):
            link_path = cargo_home / cache_name
            if cache_name in facts["cache_links"]:
                link_metadata = os.stat(cache_name, dir_fd=cargo_home_fd, follow_symlinks=False)
                require(stat.S_ISLNK(link_metadata.st_mode), "tag Cargo cache entry is not a symlink")
                require(os.readlink(cache_name, dir_fd=cargo_home_fd) == facts["cache_links"][cache_name],
                        "tag Cargo cache symlink differs from its recorded target")
            else:
                require(not link_path.exists() and not link_path.is_symlink(),
                        "tag isolated Cargo home has an unrecorded cache entry")
    finally:
        os.close(cargo_home_fd)
    for config_name in ("config", "config.toml"):
        config_path = cargo_home / config_name
        require(not config_path.exists() and not config_path.is_symlink(),
                "tag isolated Cargo home contains a configuration file")

    rustup_path = _require_text(run["rustup_path"], "tag rustup path", absolute=True)
    toolchain_root = run_root / "toolchain"
    _validate_recorded_path_output(
        _recorded_successful_command(
            toolchain_root.parent, "rustup-home", [rustup_path, "show", "home"]
        ),
        rustup_facts["home"],
        "home",
    )
    require(_recorded_successful_command(
        toolchain_root.parent, "cargo-version", [rustup_path, "run", TOOLCHAIN, "cargo", "--version"]
    ).decode("utf-8", "strict").strip() == env_record["cargo_version"],
            "recorded Cargo version output differs from the environment evidence")
    require(_recorded_successful_command(
        toolchain_root.parent, "rustc-version", [rustup_path, "run", TOOLCHAIN, "rustc", "--version"]
    ).decode("utf-8", "strict").strip() == env_record["rustc_version"],
            "recorded rustc version output differs from the environment evidence")
    verbose = _recorded_successful_command(
        toolchain_root.parent, "rustc-verbose-version", [rustup_path, "run", TOOLCHAIN, "rustc", "-Vv"]
    ).decode("utf-8", "strict")
    host_values = [line.partition(":")[2].strip() for line in verbose.splitlines() if line.startswith("host:")]
    require(len(host_values) == 1 and host_values[0] == env_record["host_triple"],
            "recorded rustc host triple differs from the environment evidence")
    for binary_name in ("cargo", "rustc"):
        binary_path = binaries[binary_name]["path"]
        _validate_recorded_path_output(
            _recorded_successful_command(
                toolchain_root.parent,
                binary_name + "-path",
                [rustup_path, "which", "--toolchain", TOOLCHAIN, binary_name],
            ),
            binary_path,
            binary_name,
        )

    metadata_targets = _closed_object(run["metadata_targets"], {"holla"}, "tag metadata targets")
    target = _closed_object(metadata_targets["holla"], {
        "package_id", "package_name", "target_name", "manifest_path", "source_path",
    }, "tag Holla metadata target")
    require(target["package_id"] == build["package_id"] and target["target_name"] == "holla",
            "tag metadata target differs from the recorded build target")
    for field in ("package_id", "package_name", "target_name"):
        _require_text(target[field], "tag metadata " + field)
    manifest_path = Path(_require_text(target["manifest_path"], "tag metadata manifest path", absolute=True))
    source_path = Path(_require_text(target["source_path"], "tag metadata source path", absolute=True))
    require(manifest_path == source_manifest and _is_strict_child(source_path, source_root),
            "tag metadata paths escape or disagree with the pinned source snapshot")

    # Verify recorded metadata and Cargo artifact messages against the immutable source and build facts.
    metadata_dir = run_root / "metadata" / "oracle"
    metadata_argv = _parse_json_value(_read_absolute_nofollow(metadata_dir / "metadata.argv.json", maximum=1024 * 1024),
                                     "metadata argv")
    expected_metadata_argv = [run["rustup_path"], "run", TOOLCHAIN, "cargo", "metadata", "--no-deps",
                              "--format-version", "1", "--locked", "--offline", "--manifest-path",
                              str(source_manifest)]
    _validate_recorded_executable_argv(metadata_argv, expected_metadata_argv, "tag metadata")
    require(_read_absolute_nofollow(metadata_dir / "metadata.exit", maximum=128) == b"0\n",
            "tag Cargo metadata did not exit successfully")
    _read_absolute_nofollow(metadata_dir / "metadata.stderr", maximum=8 * 1024 * 1024)
    metadata_raw = _read_absolute_nofollow(metadata_dir / "metadata.stdout", maximum=128 * 1024 * 1024)
    metadata = parse_metadata_stdout(metadata_raw)
    resolved_targets = resolve_binary_targets(metadata, source_root)
    target_package = resolved_targets["holla"]
    require({
        "package_id": target_package.package_id,
        "package_name": target_package.package_name,
        "target_name": target_package.app,
        "manifest_path": str(target_package.manifest_path),
        "source_path": str(target_package.source_path),
    } == target, "recorded Cargo metadata does not identify the pinned Holla target")
    build_dir = run_root / "build" / "oracle" / "holla"
    build_argv = _parse_json_value(_read_absolute_nofollow(build_dir / "build.argv.json", maximum=1024 * 1024),
                                   "build argv")
    expected_build_argv = [
        run["rustup_path"], "run", TOOLCHAIN, "cargo", "build", "--release", "--locked", "--offline",
        "--jobs", "2", "--color", "never", "--message-format=json-render-diagnostics", "--manifest-path",
        str(source_manifest), "--package", "{}@{}".format(target_package.package_name, target_package.package_version),
        "--bin", "holla", "--target", target_triple, "--target-dir", str(build_root),
    ]
    _validate_recorded_executable_argv(build_argv, expected_build_argv, "tag Cargo build")
    _validate_recorded_executable_argv(
        nested["executed_argv"], expected_build_argv, "nested tag Cargo build"
    )
    require(_read_absolute_nofollow(build_dir / "build.exit", maximum=128) == b"0\n",
            "tag Cargo build did not exit successfully")
    _read_absolute_nofollow(build_dir / "build.stderr", maximum=8 * 1024 * 1024)
    artifact_stdout = _read_absolute_nofollow(build_dir / "build.stdout", maximum=256 * 1024 * 1024)
    parsed_executable = parse_compiler_artifact(artifact_stdout, target_package.package_id, "holla", build_root)
    require(parsed_executable == executable_path,
            "recorded compiler-artifact path differs from the tag executable receipt")

    return {
        "schema": "termrock-spec/visual-tag-holla-validated-identity-v1",
        "receipt": {"path": str(receipt_path), "sha256": receipt_sha256},
        "run": {"path": str(run_path), "sha256": run_sha256},
        "oracle_lineage": dict(lineage),
        "source_snapshot": {
            "source_commit": REFERENCE_ORACLE_COMMIT,
            "tree_oid": tree_oid,
            "materialized_root": str(source_root),
            "included_path_blob_map_sha256": _path_blob_map_sha256(actual_blobs),
            "included_file_count": len(actual_blobs),
        },
        "metadata_target": dict(target),
        "build": dict(build),
        "executable": {"path": str(executable_path), "sha256": executable_sha},
        "builder_receipt_sha256": nested_digest,
        "build_environment": dict(env_record),
        "execution_anchor_status": "unverified",
        "qualification": {
            "status": "blocked",
            "reason": (
                "Receipt consistency and source closure do not independently attest Cargo execution; "
                "the Cargo cache contents and PATH-selected linker or other build tools are not bound."
            ),
        },
        "capture_status": "NOT_RUN",
        "admission_status": "NOT_RUN",
    }


def _ensure_real_directories(root: Path, parent: Path) -> None:
    relative = parent.relative_to(root)
    current = root
    for component in relative.parts:
        current = current / component
        if current.exists():
            require(current.is_dir() and not current.is_symlink(),
                    "source archive path traverses a non-directory")
        else:
            current.mkdir()


def _extract_snapshot_archive(
    archive: tarfile.TarFile, members: Sequence[tarfile.TarInfo], destination: Path
) -> None:
    destination.mkdir()
    for member in members:
        if not member.isdir():
            continue
        target = destination.joinpath(*PurePosixPath(member.name).parts)
        _ensure_real_directories(destination, target.parent)
        if target.exists():
            require(target.is_dir() and not target.is_symlink(),
                    "source archive directory collides with another path")
        else:
            target.mkdir()

    for member in members:
        if not member.isfile():
            continue
        target = destination.joinpath(*PurePosixPath(member.name).parts)
        _ensure_real_directories(destination, target.parent)
        stream = archive.extractfile(member)
        require(stream is not None, "source archive file has no content")
        try:
            with target.open("xb") as output:
                shutil.copyfileobj(stream, output)
        finally:
            stream.close()
        target.chmod(member.mode & 0o777)

    for member in members:
        if not member.issym():
            continue
        target = destination.joinpath(*PurePosixPath(member.name).parts)
        _ensure_real_directories(destination, target.parent)
        require(not target.exists() and not target.is_symlink(),
                "source archive symlink collides with another path")
        os.symlink(member.linkname, target)


def _verify_archive_blobs(
    archive: tarfile.TarFile,
    members: Sequence[tarfile.TarInfo],
    expected_blob_ids: Mapping[str, str],
) -> None:
    for member in members:
        if member.isdir():
            continue
        if member.issym():
            content = os.fsencode(member.linkname)
        else:
            stream = archive.extractfile(member)
            require(stream is not None, "source archive file has no content")
            try:
                content = stream.read()
            finally:
                stream.close()
        expected = expected_blob_ids.get(member.name)
        require(expected is not None and _git_blob_id(content) == expected,
                "git archive content differs from the pinned commit blob at {}".format(member.name))


def materialize_readonly_snapshot(
    repository: Path,
    commit: str,
    destination: Path,
    evidence_dir: Path,
    runner: Runner = _default_runner,
) -> Dict[str, Any]:
    """Extract the pinned build inputs, excluding only declared oracle trees."""
    require(not destination.exists(), "source snapshot path already exists: {}".format(destination))
    destination.parent.mkdir(parents=True, exist_ok=True)
    scratch = evidence_dir / "archive.tmp.tar"
    evidence_dir.mkdir(parents=True, exist_ok=True)

    # Stream the archive to disk so large binary assets do not sit in memory.
    archive_argv = [
        "git",
        "--no-replace-objects",
        "archive",
        "--format=tar",
        commit,
        "--",
        ".",
        *[":(top,exclude)" + path for path in ORACLE_EXCLUSIONS],
    ]
    _atomic_json(evidence_dir / "archive.argv.json", archive_argv)
    timed_out = False
    archive_sha256: Optional[str] = None
    archive_member_count = 0
    tree_oid: Optional[str] = None
    full_blob_ids: Dict[str, str] = {}
    included_blob_ids: Dict[str, str] = {}
    excluded_blob_ids: Dict[str, str] = {}
    try:
        with scratch.open("wb") as output:
            process = subprocess.run(
                archive_argv,
                cwd=str(repository),
                env=_git_command_env(),
                stdout=output,
                stderr=subprocess.PIPE,
                timeout=COMMAND_TIMEOUT_SECONDS,
                check=False,
            )
        archive_result = CommandResult(process.returncode, b"", process.stderr)
    except subprocess.TimeoutExpired as error:
        timed_out = True
        stderr = error.stderr or b""
        if isinstance(stderr, str):
            stderr = stderr.encode("utf-8", "replace")
        archive_result = CommandResult(124, b"", stderr + b"\ncommand timed out\n", True)
    except OSError as error:
        archive_result = CommandResult(127, b"", (str(error) + "\n").encode("utf-8", "replace"))
    _atomic_write(evidence_dir / "archive.stderr", archive_result.stderr)
    _atomic_write(evidence_dir / "archive.exit", ("timeout\n" if timed_out else str(archive_result.returncode) + "\n").encode("ascii"))
    try:
        require(archive_result.returncode == 0 and not archive_result.timed_out,
                "git archive failed for pinned commit {}".format(commit))
        archive_sha256 = sha256_file(scratch)
        tree_result = _recorded_command(
            ["git", "--no-replace-objects", "ls-tree", "-rz", "-r", "--full-tree", commit],
            repository,
            _git_command_env(),
            evidence_dir,
            "tree",
            runner=runner,
        )
        require(tree_result.returncode == 0, "git ls-tree failed for pinned commit {}".format(commit))
        tree_oid_result = _recorded_command(
            ["git", "--no-replace-objects", "rev-parse", "--verify", commit + "^{tree}"],
            repository,
            _git_command_env(),
            evidence_dir,
            "tree-oid",
            runner=runner,
        )
        require(tree_oid_result.returncode == 0, "cannot resolve pinned source tree for {}".format(commit))
        tree_oid = tree_oid_result.stdout.decode("ascii").strip()
        require(HEX_40.fullmatch(tree_oid) is not None, "git rev-parse returned an invalid tree object id")
        full_blob_ids = _tracked_blob_ids(tree_result.stdout)
        included_blob_ids, excluded_blob_ids = _partition_tracked_blob_ids(full_blob_ids)
        with tarfile.open(scratch, mode="r:") as archive:
            members, archive_paths, archive_member_paths = _safe_tar_members(archive)
            require(
                archive_paths == set(included_blob_ids),
                "git archive file set differs from the filtered pinned commit tree",
            )
            require(
                archive_member_paths == _expected_archive_member_paths(included_blob_ids),
                "git archive member set differs from the filtered pinned commit tree",
            )
            _verify_archive_blobs(archive, members, included_blob_ids)
            archive_member_count = len(archive_member_paths)
            _extract_snapshot_archive(archive, members, destination)
    except (OSError, tarfile.TarError) as error:
        raise SubjectError("cannot materialize pinned source snapshot: {}".format(error)) from error
    finally:
        try:
            scratch.unlink()
        except FileNotFoundError:
            pass

    for path in sorted(destination.rglob("*"), key=lambda item: len(item.parts), reverse=True):
        if path.is_symlink():
            continue
        mode = stat.S_IMODE(path.stat().st_mode)
        path.chmod(mode & ~0o222)
    destination.chmod(stat.S_IMODE(destination.stat().st_mode) & ~0o222)
    require(not destination.stat().st_mode & 0o222, "source snapshot root is writable")
    require(tree_oid is not None and archive_sha256 is not None,
            "source snapshot evidence is incomplete")
    return {
        "source_commit": commit,
        "tree_oid": tree_oid,
        "git_object_replacement_policy": "disabled-by-option-and-environment",
        "recipe": SNAPSHOT_RECIPE,
        "archive_argv": archive_argv,
        "git_ls_tree_stdout_sha256": sha256_bytes(tree_result.stdout),
        "excluded_roots": list(ORACLE_EXCLUSIONS),
        "path_blob_digest_algorithm": PATH_BLOB_DIGEST_ALGORITHM,
        "tracked_path_blob_map_sha256": _path_blob_map_sha256(full_blob_ids),
        "tracked_file_count": len(full_blob_ids),
        "included_path_blob_map_sha256": _path_blob_map_sha256(included_blob_ids),
        "included_file_count": len(included_blob_ids),
        "excluded_path_blob_map_sha256": _path_blob_map_sha256(excluded_blob_ids),
        "excluded_file_count": len(excluded_blob_ids),
        "archive_sha256": archive_sha256,
        "archive_member_count": archive_member_count,
        "read_only": True,
    }


def _verify_oracle_tag_identity(
    repository: Path,
    run_root: Path,
    runner: Runner,
) -> Dict[str, str]:
    """Verify only the immutable oracle tag; branch refs are irrelevant here."""
    evidence = run_root / "git" / "oracle-tag"
    env = _git_command_env()
    require(env.get("GIT_NO_REPLACE_OBJECTS") == "1" and env.get("GIT_NO_LAZY_FETCH") == "1",
            "Git replacement refs and lazy fetching must be disabled")
    tag_ref = "refs/tags/" + REFERENCE_TAG
    tag_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", tag_ref],
        repository,
        env,
        evidence,
        "tag-object",
        runner=runner,
    )
    require(tag_result.returncode == 0, "cannot resolve immutable visual-baseline tag")
    tag_object = tag_result.stdout.decode("ascii").strip()
    require(HEX_40.fullmatch(tag_object) is not None,
            "visual-baseline tag did not resolve to a full object id")
    tag_type_result = _recorded_command(
        ["git", "--no-replace-objects", "cat-file", "-t", tag_object],
        repository,
        env,
        evidence,
        "tag-object-type",
        runner=runner,
    )
    require(tag_type_result.returncode == 0, "cannot inspect immutable visual-baseline tag type")
    tag_type = tag_type_result.stdout.decode("ascii").strip()
    require(tag_type == "tag", "visual-baseline must remain an annotated tag object")

    commit_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", tag_ref + "^{commit}"],
        repository,
        env,
        evidence,
        "peeled-commit",
        runner=runner,
    )
    require(commit_result.returncode == 0, "cannot peel immutable visual-baseline tag to a commit")
    tag_commit = commit_result.stdout.decode("ascii").strip()
    require(HEX_40.fullmatch(tag_commit) is not None,
            "visual-baseline tag did not peel to a full commit id")
    commit_type_result = _recorded_command(
        ["git", "--no-replace-objects", "cat-file", "-t", tag_commit],
        repository,
        env,
        evidence,
        "peeled-commit-type",
        runner=runner,
    )
    require(commit_type_result.returncode == 0 and commit_type_result.stdout.strip() == b"commit",
            "visual-baseline tag does not peel to a commit object")
    require(tag_commit == REFERENCE_ORACLE_COMMIT,
            "immutable visual-baseline oracle commit changed")
    require(tag_object == REFERENCE_TAG_OBJECT,
            "immutable visual-baseline tag object changed")
    return {
        "tag_ref": tag_ref,
        "tag_object": tag_object,
        "tag_commit": tag_commit,
        "git_object_replacement_policy": "disabled-by-option-and-environment",
        "git_lazy_fetch_policy": "disabled-by-environment",
    }


def _verify_git_identity(
    repository: Path,
    reference_commit: str,
    candidate_commit: str,
    source_facts: SourceFactsPins,
    run_root: Path,
    runner: Runner,
) -> Dict[str, str]:
    git_dir = run_root / "git"
    env = _git_command_env()
    tag_object_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", "refs/tags/" + REFERENCE_TAG],
        repository,
        env,
        git_dir,
        "reference-tag-object",
        runner=runner,
    )
    tag_commit_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", "refs/tags/" + REFERENCE_TAG + "^{commit}"],
        repository,
        env,
        git_dir,
        "reference-tag-commit",
        runner=runner,
    )
    branch_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", REFERENCE_BRANCH_REF],
        repository,
        env,
        git_dir,
        "reference-branch-object",
        runner=runner,
    )
    candidate_branch_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", CANDIDATE_BRANCH_REF],
        repository,
        env,
        git_dir,
        "candidate-branch-object",
        runner=runner,
    )
    reference_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", reference_commit + "^{commit}"],
        repository,
        env,
        git_dir,
        "reference-commit",
        runner=runner,
    )
    candidate_result = _recorded_command(
        ["git", "--no-replace-objects", "rev-parse", "--verify", candidate_commit + "^{commit}"],
        repository,
        env,
        git_dir,
        "candidate-commit",
        runner=runner,
    )
    branch_type_result = branch_result
    if branch_result.returncode == 0:
        branch_type_result = _recorded_command(
            [
                "git",
                "--no-replace-objects",
                "cat-file",
                "-t",
                branch_result.stdout.decode("ascii", "replace").strip(),
            ],
            repository,
            env,
            git_dir,
            "reference-branch-object-type",
            runner=runner,
        )
    candidate_branch_type_result = candidate_branch_result
    if candidate_branch_result.returncode == 0:
        candidate_branch_type_result = _recorded_command(
            [
                "git",
                "--no-replace-objects",
                "cat-file",
                "-t",
                candidate_branch_result.stdout.decode("ascii", "replace").strip(),
            ],
            repository,
            env,
            git_dir,
            "candidate-branch-object-type",
            runner=runner,
        )
    for label, result in (
        ("reference tag object", tag_object_result),
        ("reference tag commit", tag_commit_result),
        ("visual-baseline branch ref", branch_result),
        ("termrock-implementation branch ref", candidate_branch_result),
        ("requested reference commit", reference_result),
        ("candidate commit", candidate_result),
    ):
        require(result.returncode == 0, "cannot resolve pinned {}".format(label))
    tag_object = tag_object_result.stdout.decode("ascii").strip()
    tag_commit = tag_commit_result.stdout.decode("ascii").strip()
    branch_oid = branch_result.stdout.decode("ascii").strip()
    branch_type = branch_type_result.stdout.decode("ascii").strip()
    candidate_branch_oid = candidate_branch_result.stdout.decode("ascii").strip()
    candidate_branch_type = candidate_branch_type_result.stdout.decode("ascii").strip()
    reference = reference_result.stdout.decode("ascii").strip()
    candidate = candidate_result.stdout.decode("ascii").strip()
    require(tag_object == REFERENCE_TAG_OBJECT, "immutable visual-baseline tag object changed")
    require(
        tag_commit == REFERENCE_ORACLE_COMMIT,
        "immutable visual-baseline oracle commit changed",
    )
    require(
        HEX_40.fullmatch(branch_oid) is not None,
        "visual-baseline branch ref did not resolve to a full commit SHA",
    )
    require(
        branch_type_result.returncode == 0,
        "cannot inspect visual-baseline branch ref object type",
    )
    require(
        branch_type == "commit",
        "visual-baseline branch ref must point directly to a commit",
    )
    require(
        candidate_branch_type_result.returncode == 0,
        "cannot inspect termrock-implementation branch ref object type",
    )
    require(
        candidate_branch_type == "commit",
        "termrock-implementation branch ref must point directly to a commit",
    )
    require(
        reference_commit == source_facts.reference_commit,
        "requested reference commit does not match the accepted source-facts reference tip",
    )
    require(
        candidate_commit == source_facts.candidate_commit,
        "requested candidate commit does not match the accepted source-facts candidate tip",
    )
    require(
        branch_oid == source_facts.reference_commit,
        "fetched visual-baseline ref does not match the accepted source-facts reference tip",
    )
    require(
        candidate_branch_oid == source_facts.candidate_commit,
        "fetched termrock-implementation ref does not match the accepted source-facts candidate tip",
    )
    require(
        reference == branch_oid,
        "requested reference commit must equal the recorded visual-baseline branch tip",
    )
    require(
        candidate == candidate_branch_oid,
        "requested candidate commit must equal the recorded termrock-implementation branch tip",
    )
    require(
        reference == reference_commit,
        "requested reference commit did not resolve to the exact lowercase SHA",
    )
    require(candidate == candidate_commit, "candidate commit did not resolve to the requested full SHA")
    membership_result = _recorded_command(
        [
            "git",
            "--no-replace-objects",
            "merge-base",
            "--is-ancestor",
            reference_commit,
            branch_oid,
        ],
        repository,
        env,
        git_dir,
        "reference-branch-membership",
        runner=runner,
    )
    require(
        membership_result.returncode == 0,
        "requested reference commit is not reachable from {}".format(REFERENCE_BRANCH_REF),
    )
    return {
        "reference_tag_object": tag_object,
        "reference_tag_commit": tag_commit,
        "reference_branch_ref": REFERENCE_BRANCH_REF,
        "reference_branch_oid": branch_oid,
        "reference_commit": reference,
        "candidate_commit": candidate,
    }


def _toolchain_facts(
    run_root: Path, runner: Runner
) -> Tuple[str, str, str, str, Path, Dict[str, Dict[str, str]]]:
    evidence = run_root / "toolchain"
    env = _command_env()
    rustup = shutil.which("rustup", path=env["PATH"])
    require(rustup is not None, "rustup is required to select the pinned toolchain")
    home_result = _recorded_command(
        [rustup, "show", "home"],
        ROOT,
        env,
        evidence,
        "rustup-home",
        runner=runner,
    )
    cargo_result = _recorded_command(
        [rustup, "run", TOOLCHAIN, "cargo", "--version"],
        ROOT,
        env,
        evidence,
        "cargo-version",
        runner=runner,
    )
    rustc_result = _recorded_command(
        [rustup, "run", TOOLCHAIN, "rustc", "--version"],
        ROOT,
        env,
        evidence,
        "rustc-version",
        runner=runner,
    )
    verbose_result = _recorded_command(
        [rustup, "run", TOOLCHAIN, "rustc", "-Vv"],
        ROOT,
        env,
        evidence,
        "rustc-verbose-version",
        runner=runner,
    )
    toolchain_binaries: Dict[str, Dict[str, str]] = {}
    for program in ("cargo", "rustc"):
        which_result = _recorded_command(
            [rustup, "which", "--toolchain", TOOLCHAIN, program],
            ROOT,
            env,
            evidence,
            program + "-path",
            runner=runner,
        )
        require(which_result.returncode == 0,
                "cannot resolve pinned {} executable".format(program))
        executable_text = which_result.stdout.decode("utf-8").strip()
        require(bool(executable_text), "rustup which returned an empty {} path".format(program))
        executable = Path(executable_text).expanduser().resolve()
        require(executable.is_file() and os.access(str(executable), os.X_OK),
                "pinned {} executable is not runnable".format(program))
        toolchain_binaries[program] = {
            "path": str(executable),
            "sha256": sha256_file(executable),
        }
    for label, result in (
        ("rustup home", home_result),
        ("Cargo version", cargo_result),
        ("rustc version", rustc_result),
        ("rustc verbose version", verbose_result),
    ):
        require(result.returncode == 0, "cannot query pinned {}".format(label))
    cargo_version = cargo_result.stdout.decode("utf-8").strip()
    rustc_version = rustc_result.stdout.decode("utf-8").strip()
    rustup_home_text = home_result.stdout.decode("utf-8").strip()
    require(bool(rustup_home_text), "rustup show home returned an empty path")
    rustup_home = Path(rustup_home_text).expanduser().resolve()
    require(rustup_home.is_dir(), "resolved RUSTUP_HOME does not exist")
    verbose_text = verbose_result.stdout.decode("utf-8")
    host_matches = [line.partition(":")[2].strip() for line in verbose_text.splitlines() if line.startswith("host:")]
    require(len(host_matches) == 1 and host_matches[0], "rustc -Vv did not report one host triple")
    return rustup, cargo_version, rustc_version, host_matches[0], rustup_home, toolchain_binaries


def _assert_toolchain_unchanged(
    rustup: str,
    rustup_sha256: str,
    toolchain_binaries: Mapping[str, Mapping[str, str]],
) -> None:
    require(sha256_file(Path(rustup)) == rustup_sha256,
            "rustup executable changed after toolchain inspection")
    for name, facts in toolchain_binaries.items():
        executable = Path(facts["path"])
        require(sha256_file(executable) == facts["sha256"],
                "pinned {} executable changed after toolchain inspection".format(name))


def _assert_clean_cargo_home(env: Mapping[str, str]) -> None:
    cargo_home = Path(env["CARGO_HOME"])
    require(cargo_home.is_dir(), "isolated CARGO_HOME is missing")
    for name in ("config", "config.toml"):
        config = cargo_home / name
        require(not config.exists() and not config.is_symlink(),
                "isolated Cargo home unexpectedly contains {}".format(config))


def _mise_config(snapshot: Path) -> Dict[str, Any]:
    path = snapshot / "mise.toml"
    if not path.exists():
        return {"present": False, "path": None, "sha256": None}
    require(path.is_file(), "mise.toml is not a regular file")
    return {"present": True, "path": "mise.toml", "sha256": sha256_file(path)}


def _cargo_metadata(
    source: Path,
    role: str,
    run_root: Path,
    rustup: str,
    env: Mapping[str, str],
    rustup_sha256: str,
    toolchain_binaries: Mapping[str, Mapping[str, str]],
    runner: Runner,
) -> Tuple[Any, Dict[str, TargetPackage]]:
    _cargo_config_facts(source)
    _assert_clean_cargo_home(env)
    _assert_toolchain_unchanged(rustup, rustup_sha256, toolchain_binaries)
    result = _recorded_command(
        [
            rustup,
            "run",
            TOOLCHAIN,
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--manifest-path",
            str(source / "Cargo.toml"),
        ],
        source,
        env,
        run_root / "metadata" / role,
        "metadata",
        runner=runner,
        timeout=COMMAND_TIMEOUT_SECONDS,
    )
    require(result.returncode == 0, "cargo metadata failed for {} source".format(role))
    _assert_clean_cargo_home(env)
    _assert_toolchain_unchanged(rustup, rustup_sha256, toolchain_binaries)
    metadata = parse_metadata_stdout(result.stdout)
    return metadata, resolve_binary_targets(metadata, source)


def _single_build_environment(
    run_root: Path, rustup_home: Path, role: str
) -> Tuple[Dict[str, str], Dict[str, Any]]:
    """Create one clean subject environment with no ambient Cargo config."""
    inherited = _command_env()
    default_cargo_home = Path.home() / ".cargo"
    cache_home = Path(inherited.get("CARGO_HOME", str(default_cargo_home))).expanduser().resolve()
    environment_root = run_root / "environment" / role
    home = environment_root / "home"
    cargo_home = environment_root / "cargo-home"
    temporary = environment_root / "tmp"
    for path in (home, cargo_home, temporary):
        require(not path.exists(), "isolated build environment path already exists: {}".format(path))
        path.mkdir(parents=True)

    cache_links: Dict[str, str] = {}
    for name in ("registry", "git"):
        source = cache_home / name
        if not source.exists():
            continue
        require(source.is_dir(), "Cargo cache path is not a directory: {}".format(source))
        resolved_source = source.resolve()
        os.symlink(str(resolved_source), cargo_home / name, target_is_directory=True)
        cache_links[name] = str(resolved_source)

    for config_name in ("config", "config.toml"):
        cargo_config = cargo_home / config_name
        require(not cargo_config.exists() and not cargo_config.is_symlink(),
                "isolated Cargo home unexpectedly contains {}".format(config_name))

    # Keep only platform discovery inputs. In particular, do not pass through
    # user compiler flags, wrappers, Cargo config selectors, or user HOME.
    env = {"PATH": inherited.get("PATH", "")}
    for key in ("DEVELOPER_DIR", "SDKROOT", "SYSTEMROOT", "COMSPEC", "PATHEXT"):
        if key in inherited:
            env[key] = inherited[key]
    env["HOME"] = str(home)
    env["CARGO_HOME"] = str(cargo_home)
    env["RUSTUP_HOME"] = str(rustup_home)
    env["TMPDIR"] = str(temporary)
    env["CARGO_TERM_COLOR"] = "never"
    env["CARGO_NET_OFFLINE"] = "true"
    facts = {
        "home": str(home),
        "cargo_home": str(cargo_home),
        "cargo_home_config": "absent",
        "cache_home": str(cache_home),
        "cache_links": cache_links,
        "rustup_home": str(rustup_home),
        "tmpdir": str(temporary),
        "path": env["PATH"],
        "platform_inputs": {key: env[key] for key in ("DEVELOPER_DIR", "SDKROOT", "SYSTEMROOT") if key in env},
    }
    return env, facts


def _build_environment(
    run_root: Path, rustup_home: Path
) -> Tuple[Dict[str, Dict[str, str]], Dict[str, Dict[str, Any]]]:
    environments: Dict[str, Dict[str, str]] = {}
    facts_by_role: Dict[str, Dict[str, Any]] = {}
    for role in ("reference", "candidate"):
        environment, facts = _single_build_environment(run_root, rustup_home, role)
        environments[role] = environment
        facts_by_role[role] = facts
    return environments, facts_by_role


def _build_binary_artifact(
    source_commit: str,
    source: Path,
    target_dir: Path,
    app: str,
    target_package: TargetPackage,
    target_triple: str,
    rustup: str,
    rustup_sha256: str,
    toolchain_binaries: Mapping[str, Mapping[str, str]],
    cargo_version: str,
    rustc_version: str,
    host_triple: str,
    command_evidence_dir: Path,
    command_context: str,
    build_environment: Mapping[str, str],
    runner: Runner,
    on_build_attempt: Optional[Callable[[], None]] = None,
) -> Dict[str, Any]:
    _cargo_config_facts(source)
    _assert_clean_cargo_home(build_environment)
    _assert_toolchain_unchanged(rustup, rustup_sha256, toolchain_binaries)
    require(target_triple == host_triple,
            "requested target {} differs from pinned toolchain host {}".format(target_triple, host_triple))
    require(not target_dir.exists(), "target directory already exists; refusing stale artifacts")
    target_dir.parent.mkdir(parents=True, exist_ok=True)
    argv = [
        rustup,
        "run",
        TOOLCHAIN,
        "cargo",
        "build",
        "--release",
        "--locked",
        "--offline",
        "--jobs",
        "2",
        "--color",
        "never",
        "--message-format=json-render-diagnostics",
        "--manifest-path",
        str(source / "Cargo.toml"),
        "--package",
        "{}@{}".format(target_package.package_name, target_package.package_version),
        "--bin",
        app,
        "--target",
        target_triple,
        "--target-dir",
        str(target_dir),
    ]
    result = _recorded_command(
        argv,
        source,
        build_environment,
        command_evidence_dir,
        "build",
        runner=runner,
        timeout=COMMAND_TIMEOUT_SECONDS,
        on_runner_start=on_build_attempt,
    )
    require(result.returncode == 0,
            "release build failed for {} source".format(command_context))
    _assert_clean_cargo_home(build_environment)
    _assert_toolchain_unchanged(rustup, rustup_sha256, toolchain_binaries)
    executable = parse_compiler_artifact(
        result.stdout, target_package.package_id, app, target_dir
    )
    manifest_sha = sha256_file(target_package.manifest_path)
    lock_sha = sha256_file(source / "Cargo.lock")
    mise_config = _mise_config(source)
    executable_sha = sha256_file(executable)
    features: List[str] = []
    build = {
        "package_id": target_package.package_id,
        "target_name": app,
        "features": features,
        "default_features": True,
        "target_triple": target_triple,
        "toolchain": TOOLCHAIN,
        "profile": "release",
    }
    build_inputs = {
        "manifest_sha256": manifest_sha,
        "lock_sha256": lock_sha,
        "mise_config": mise_config,
        "cargo_version": cargo_version,
        "rustc_version": rustc_version,
        "host_triple": host_triple,
        "executed_argv": argv,
    }
    receipt_payload = {
        "schema": BUILDER_RECEIPT_SCHEMA,
        "source_commit": source_commit,
        "package_id": target_package.package_id,
        "target_name": app,
        "requested_features": features,
        "default_features": True,
        "target_triple": target_triple,
        "toolchain": TOOLCHAIN,
        "profile": "release",
        "manifest_sha256": manifest_sha,
        "lock_sha256": lock_sha,
        "mise_config": mise_config,
        "cargo_version": cargo_version,
        "rustc_version": rustc_version,
        "host_triple": host_triple,
        "executed_argv": argv,
        "executable_path": str(executable),
        "executable_sha256": executable_sha,
    }
    receipt_sha = canonical_receipt_sha256(receipt_payload)
    receipt = dict(receipt_payload)
    receipt["sha256"] = receipt_sha
    return {
        "source_commit": source_commit,
        "build": build,
        "executable": {"path": str(executable), "sha256": executable_sha},
        "build_inputs": build_inputs,
        "builder_receipt": receipt,
        "builder_receipt_sha256": receipt_sha,
    }


def _build_subject(
    role: str,
    source_commit: str,
    source: Path,
    target_dir: Path,
    app: str,
    target_package: TargetPackage,
    target_triple: str,
    rustup: str,
    rustup_sha256: str,
    toolchain_binaries: Mapping[str, Mapping[str, str]],
    cargo_version: str,
    rustc_version: str,
    host_triple: str,
    run_root: Path,
    actual_output_root: Path,
    build_environment: Mapping[str, str],
    runner: Runner,
) -> Dict[str, Any]:
    artifact = _build_binary_artifact(
        source_commit=source_commit,
        source=source,
        target_dir=target_dir,
        app=app,
        target_package=target_package,
        target_triple=target_triple,
        rustup=rustup,
        rustup_sha256=rustup_sha256,
        toolchain_binaries=toolchain_binaries,
        cargo_version=cargo_version,
        rustc_version=rustc_version,
        host_triple=host_triple,
        command_evidence_dir=run_root / "build" / role / app,
        command_context="{} {}".format(role, app),
        build_environment=build_environment,
        runner=runner,
    )
    return {
        "role": role,
        "source_commit": artifact["source_commit"],
        "build": artifact["build"],
        "executable": artifact["executable"],
        "actual_output_root": str(actual_output_root),
        "build_inputs": artifact["build_inputs"],
        "builder_receipt": artifact["builder_receipt"],
        "builder_receipt_sha256": artifact["builder_receipt_sha256"],
    }


def _validate_subject_manifest(manifest: Mapping[str, Any]) -> None:
    required = {
        "schema",
        "run_id",
        "suite_revision",
        "suite_sha256",
        "expected_generation",
        "build_evidence",
        "subjects",
    }
    require(set(manifest) == required, "subject manifest keys differ from the frozen v2 schema")
    require(manifest["schema"] == SUBJECT_SCHEMA, "unexpected subject manifest schema")
    generation = manifest["expected_generation"]
    if generation is not None:
        require(isinstance(generation, dict)
                and set(generation) == {"id", "sha256", "root"},
                "expected_generation must be null or the complete accepted object")
        require(isinstance(generation["id"], str) and generation["id"].strip(),
                "expected_generation.id must not be blank")
        _validate_digest(generation["sha256"], "expected_generation.sha256")
        require(isinstance(generation["root"], str) and generation["root"],
                "expected_generation.root must not be blank")
    build_evidence = manifest["build_evidence"]
    require(isinstance(build_evidence, dict)
            and set(build_evidence) == {"source_inputs", "build_environment"},
            "subject build_evidence keys differ from the frozen v2 schema")
    for role, basename in (
        ("source_inputs", "source-inputs.json"),
        ("build_environment", "build-environment.json"),
    ):
        reference = build_evidence[role]
        require(isinstance(reference, dict) and set(reference) == {"path", "sha256"},
                "{} evidence reference has unexpected fields".format(role))
        require(reference["path"] == basename,
                "{} evidence path must be the fixed basename {}".format(role, basename))
        _validate_digest(reference["sha256"], "{} evidence sha256".format(role))
    subjects = manifest["subjects"]
    require(isinstance(subjects, list) and len(subjects) == 2, "subject manifest requires one pair")
    require({subject.get("role") for subject in subjects} == {"reference", "candidate"},
            "subject manifest roles must be exactly reference and candidate")
    for subject in subjects:
        require(
            set(subject)
            == {
                "role",
                "source_commit",
                "build",
                "executable",
                "actual_output_root",
                "build_inputs",
                "builder_receipt",
                "builder_receipt_sha256",
            },
            "subject fields differ from the frozen v2 schema",
        )
        require(subject["builder_receipt_sha256"] == subject["builder_receipt"].get("sha256"),
                "subject builder receipt digests do not match")
        receipt_payload = dict(subject["builder_receipt"])
        receipt_digest = receipt_payload.pop("sha256", None)
        require(receipt_digest == canonical_receipt_sha256(receipt_payload),
                "subject builder receipt canonical digest does not match")
        require(subject["build_inputs"]["manifest_sha256"] == subject["builder_receipt"]["manifest_sha256"],
                "manifest input hash differs from builder receipt")
        require(subject["build_inputs"]["lock_sha256"] == subject["builder_receipt"]["lock_sha256"],
                "lock input hash differs from builder receipt")
        require(subject["build_inputs"]["mise_config"] == subject["builder_receipt"]["mise_config"],
                "mise input differs from builder receipt")
        for field in ("cargo_version", "rustc_version", "host_triple", "executed_argv"):
            require(subject["build_inputs"][field] == subject["builder_receipt"][field],
                    "{} differs from builder receipt".format(field))


def _validate_output_roots(reference: Path, candidate: Path, expected_root: Optional[Path]) -> Tuple[Path, Path]:
    reference = reference.expanduser().resolve()
    candidate = candidate.expanduser().resolve()
    require(reference.is_absolute() and candidate.is_absolute(), "actual output roots must be absolute")
    require(reference != candidate, "reference and candidate output roots must differ")
    require(not _inside(reference, candidate) and not _inside(candidate, reference),
            "reference and candidate output roots must be disjoint")
    if expected_root is not None:
        expected = expected_root.expanduser().resolve()
        require(not _inside(reference, expected) and not _inside(expected, reference),
                "reference output root overlaps expected generation")
        require(not _inside(candidate, expected) and not _inside(expected, candidate),
                "candidate output root overlaps expected generation")
    return reference, candidate


def build_subject_manifest(
    *,
    repository: Path,
    reference_commit: str,
    candidate_commit: str,
    source_facts_path: Path,
    source_facts_sha256: str,
    evidence_root: Path,
    run_id: str,
    app: str,
    suite_revision: str,
    suite_sha256: str,
    expected_generation_id: Optional[str],
    expected_generation_sha256: Optional[str],
    expected_generation_root: Optional[Path],
    target_triple: str,
    reference_output_root: Path,
    candidate_output_root: Path,
    runner: Runner = _default_runner,
) -> Path:
    """Build and record one app's pinned source pair. Never invokes an app."""
    _validate_run_id(run_id)
    _validate_commit(reference_commit, "reference_commit")
    _validate_commit(candidate_commit, "candidate_commit")
    source_facts = _read_source_facts(source_facts_path, source_facts_sha256)
    require(
        reference_commit == source_facts.reference_commit,
        "reference_commit does not match the latest accepted source-facts reference tip",
    )
    require(
        candidate_commit == source_facts.candidate_commit,
        "candidate_commit does not match the latest accepted source-facts candidate tip",
    )
    require(app in APPS, "app must be one of {}".format(", ".join(APPS)))
    require(isinstance(suite_revision, str) and suite_revision.strip(), "suite_revision is required")
    _validate_digest(suite_sha256, "suite_sha256")
    generation_values = (
        expected_generation_id is not None,
        expected_generation_sha256 is not None,
        expected_generation_root is not None,
    )
    require(
        all(generation_values) or not any(generation_values),
        "expected generation id, sha256, and root must be supplied together",
    )
    if expected_generation_id is not None:
        require(isinstance(expected_generation_id, str) and expected_generation_id.strip(),
                "expected_generation_id is required when expected generation is supplied")
        _validate_digest(expected_generation_sha256, "expected_generation_sha256")
    require(isinstance(target_triple, str) and target_triple.strip(), "target_triple is required")
    reference_output_root, candidate_output_root = _validate_output_roots(
        reference_output_root, candidate_output_root, expected_generation_root
    )
    schema_sha256 = verify_schema()

    repository = repository.expanduser().resolve()
    require(repository.is_dir(), "git repository path is not a directory")
    evidence_root = evidence_root.expanduser().resolve()
    run_root = evidence_root / run_id
    require(not run_root.exists(), "evidence run path already exists; refusing stale evidence: {}".format(run_root))
    run_root.mkdir(parents=True)
    source_facts_context = (
        "source-facts-sha256={}; reference-ref={}/{}; candidate-ref={}/{}"
    ).format(
        source_facts.sha256,
        REFERENCE_BRANCH_REF,
        source_facts.reference_commit,
        CANDIDATE_BRANCH_REF,
        source_facts.candidate_commit,
    )
    run_qualification_reason = (
        "Blocked: {}; this per-app build does not establish the complete four-app branch pair, "
        "cache/PATH/linker input hashes, or external trust/admission."
    ).format(source_facts_context)
    source_qualification_reason = (
        "Blocked: {}; production-input closure and comparison with the separate immutable "
        "oracle tag remain unproven."
    ).format(source_facts_context)
    status: Dict[str, Any] = {
        "schema": "termrock-spec/parity-subject-build-evidence-v1",
        "state": "running",
        "qualification": {
            "status": "blocked",
            "reason": run_qualification_reason,
        },
        "run_id": run_id,
        "app": app,
        "subject_schema_sha256": schema_sha256,
        "reference_tag_object": REFERENCE_TAG_OBJECT,
        "reference_tag_commit": REFERENCE_ORACLE_COMMIT,
        "reference_branch_ref": REFERENCE_BRANCH_REF,
        "reference_commit": reference_commit,
        "candidate_commit": candidate_commit,
        "target_triple": target_triple,
        "toolchain": TOOLCHAIN,
    }
    _atomic_json(run_root / "run.json", status)
    try:
        identities = _verify_git_identity(
            repository, reference_commit, candidate_commit, source_facts, run_root, runner
        )
        rustup, cargo_version, rustc_version, host_triple, rustup_home, toolchain_binaries = _toolchain_facts(
            run_root, runner
        )
        status["rustup_path"] = str(Path(rustup).resolve())
        status["rustup_sha256"] = sha256_file(Path(rustup))
        status["rustup_home"] = str(rustup_home)
        status["toolchain_binaries"] = toolchain_binaries
        require(target_triple == host_triple,
                "requested target {} differs from pinned toolchain host {}".format(target_triple, host_triple))
        status["git_identity"] = identities
        status["reference_branch_oid"] = identities["reference_branch_oid"]
        status["git_object_replacement_policy"] = "disabled-by-option-and-environment"
        status["cargo_version"] = cargo_version
        status["rustc_version"] = rustc_version
        status["host_triple"] = host_triple

        source_roots = {
            "reference": (reference_commit, run_root / "source" / "reference"),
            "candidate": (candidate_commit, run_root / "source" / "candidate"),
        }
        snapshot_facts: Dict[str, Dict[str, Any]] = {}
        for role, (commit, destination) in source_roots.items():
            _assert_no_ancestor_cargo_config(destination)
            snapshot_facts[role] = materialize_readonly_snapshot(
                repository, commit, destination, run_root / "snapshot-logs" / role, runner
            )
            snapshot_facts[role]["role"] = role
            snapshot_facts[role]["path"] = str(destination)
        status["snapshots"] = {
            role: {"commit": commit, "path": str(path), "read_only": True}
            for role, (commit, path) in source_roots.items()
        }
        status["snapshot_materialization"] = snapshot_facts
        status["cargo_config_sha256"] = {
            role: _cargo_config_facts(path)
            for role, (_commit, path) in source_roots.items()
        }

        source_input_payload = {
            "schema": "termrock-spec/parity-subject-source-inputs-v1",
            "run_id": run_id,
            "qualification_status": "blocked",
            "qualification_reason": source_qualification_reason,
            "git_object_replacement_policy": "disabled-by-option-and-environment",
            "oracle_lineage": {
                "tag_ref": "refs/tags/" + REFERENCE_TAG,
                "tag_object": identities["reference_tag_object"],
                "tag_commit": identities["reference_tag_commit"],
            },
            "reference_branch": {
                "ref": identities["reference_branch_ref"],
                "oid": identities["reference_branch_oid"],
                "selected_commit": identities["reference_commit"],
                "membership": "verified-reachable",
            },
            "recipe": SNAPSHOT_RECIPE,
            "excluded_roots": list(ORACLE_EXCLUSIONS),
            "path_blob_digest_algorithm": PATH_BLOB_DIGEST_ALGORITHM,
            "sources": snapshot_facts,
        }
        source_input_evidence_path = run_root / "source-inputs.json"
        _atomic_json(source_input_evidence_path, source_input_payload)
        source_input_evidence = {
            "path": str(source_input_evidence_path),
            "sha256": sha256_file(source_input_evidence_path),
        }
        status["source_input_evidence"] = source_input_evidence

        build_environment, build_environment_facts = _build_environment(run_root, rustup_home)
        status["build_environment"] = build_environment_facts
        environment_evidence_path = run_root / "build-environment.json"
        _atomic_json(
            environment_evidence_path,
            {
                "schema": "termrock-spec/parity-subject-build-environment-v1",
                "run_id": run_id,
                "source_input_evidence": source_input_evidence,
                "source_commits": {role: commit for role, (commit, _source) in source_roots.items()},
                "reference_branch": source_input_payload["reference_branch"],
                "oracle_lineage": source_input_payload["oracle_lineage"],
                "cargo_config_sha256": status["cargo_config_sha256"],
                "external_cargo_config_policy": "reject-any-config-from-snapshot-parent-through-filesystem-root",
                "rustup": {
                    "path": str(Path(rustup).resolve()),
                    "sha256": status["rustup_sha256"],
                    "home": str(rustup_home),
                },
                "toolchain_binaries": toolchain_binaries,
                "cargo_version": cargo_version,
                "rustc_version": rustc_version,
                "host_triple": host_triple,
                "target_triple": target_triple,
                "environment": build_environment_facts,
            },
        )
        status["build_environment_evidence"] = {
            "path": str(environment_evidence_path),
            "sha256": sha256_file(environment_evidence_path),
        }
        targets_by_role: Dict[str, Dict[str, TargetPackage]] = {}
        for role, (_commit, source) in source_roots.items():
            _metadata, targets = _cargo_metadata(
                source,
                role,
                run_root,
                rustup,
                build_environment[role],
                status["rustup_sha256"],
                toolchain_binaries,
                runner,
            )
            targets_by_role[role] = targets
            status.setdefault("metadata_targets", {})[role] = {
                app_name: {
                    "package_id": target.package_id,
                    "package_name": target.package_name,
                    "target_name": target.app,
                    "manifest_path": str(target.manifest_path),
                    "source_path": str(target.source_path),
                }
                for app_name, target in targets.items()
            }

        subjects: List[Dict[str, Any]] = []
        for role in ("reference", "candidate"):
            commit, source = source_roots[role]
            target_dir = run_root / "targets" / role
            output_root = reference_output_root if role == "reference" else candidate_output_root
            subject = _build_subject(
                role=role,
                source_commit=commit,
                source=source,
                target_dir=target_dir,
                app=app,
                target_package=targets_by_role[role][app],
                target_triple=target_triple,
                rustup=rustup,
                rustup_sha256=status["rustup_sha256"],
                toolchain_binaries=toolchain_binaries,
                cargo_version=cargo_version,
                rustc_version=rustc_version,
                host_triple=host_triple,
                run_root=run_root,
                actual_output_root=output_root,
                build_environment=build_environment[role],
                runner=runner,
            )
            subjects.append(subject)

        expected_generation: Optional[Dict[str, Any]] = None
        if expected_generation_id is not None and expected_generation_sha256 is not None:
            expected_generation = {
                "id": expected_generation_id,
                "sha256": expected_generation_sha256,
                "root": str(expected_generation_root.expanduser().resolve()),
            }
        manifest = {
            "schema": SUBJECT_SCHEMA,
            "run_id": run_id,
            "suite_revision": suite_revision,
            "suite_sha256": suite_sha256,
            "expected_generation": expected_generation,
            "build_evidence": {
                "source_inputs": {
                    "path": "source-inputs.json",
                    "sha256": source_input_evidence["sha256"],
                },
                "build_environment": {
                    "path": "build-environment.json",
                    "sha256": status["build_environment_evidence"]["sha256"],
                },
            },
            "subjects": subjects,
        }
        _validate_subject_manifest(manifest)
        manifest_path = run_root / "subject-manifest.json"
        _atomic_json(manifest_path, manifest)
        status["state"] = "complete"
        status["subject_manifest"] = str(manifest_path)
        status["subject_manifest_sha256"] = sha256_file(manifest_path)
        _atomic_json(run_root / "run.json", status)
        return manifest_path
    except Exception as error:
        status["state"] = "failed"
        status["error"] = str(error)
        _atomic_json(run_root / "run.json", status)
        raise


def build_frozen_tag_holla(
    *,
    repository: Path,
    evidence_root: Path,
    run_id: str,
    target_triple: str,
    runner: Runner = _default_runner,
) -> Path:
    """Build Holla from the immutable oracle tag without creating a pair manifest."""
    _validate_run_id(run_id)
    require(isinstance(target_triple, str) and target_triple.strip(),
            "target_triple is required")
    repository = repository.expanduser().resolve()
    require(repository.is_dir(), "git repository path is not a directory")
    evidence_root = evidence_root.expanduser().resolve()
    run_root = evidence_root / run_id
    require(not run_root.exists() and not run_root.is_symlink(),
            "evidence run path already exists; refusing stale evidence: {}".format(run_root))
    try:
        run_root.mkdir(parents=True)
    except OSError as error:
        raise SubjectError("cannot create fresh tag-builder run root: {}".format(error)) from error

    qualification_reason = (
        "Blocked: this immutable-tag Holla build does not prove production-input closure, "
        "cache/PATH/linker provenance, paired branch equivalence, suite capture, or admission."
    )
    status: Dict[str, Any] = {
        "schema": TAG_BUILDER_RUN_SCHEMA,
        "state": "running",
        "phase": "preflight",
        "build_result": "BLOCKED",
        "build_attempted": False,
        "qualification": {"status": "blocked", "reason": qualification_reason},
        "run_id": run_id,
        "purpose": "frozen-visual-tag-holla-build-only",
        "app": "holla",
        "target_triple": target_triple,
        "tag_ref": "refs/tags/" + REFERENCE_TAG,
        "expected_tag_object": REFERENCE_TAG_OBJECT,
        "expected_tag_commit": REFERENCE_ORACLE_COMMIT,
        "capture_status": "NOT_RUN",
        "admission_status": "NOT_RUN",
    }
    run_record_path = run_root / "run.json"
    receipt_path: Optional[Path] = None
    _atomic_json(run_record_path, status)
    try:
        identities = _verify_oracle_tag_identity(repository, run_root, runner)
        status["oracle_lineage"] = identities
        status["git_object_replacement_policy"] = "disabled-by-option-and-environment"
        status["git_lazy_fetch_policy"] = "disabled-by-environment"
        status["phase"] = "toolchain"
        _atomic_json(run_record_path, status)

        rustup, cargo_version, rustc_version, host_triple, rustup_home, toolchain_binaries = (
            _toolchain_facts(run_root, runner)
        )
        rustup_path = Path(rustup).resolve()
        rustup_sha256 = sha256_file(rustup_path)
        require(target_triple == host_triple,
                "requested target {} differs from pinned toolchain host {}".format(
                    target_triple, host_triple
                ))
        status.update({
            "rustup_path": str(rustup_path),
            "rustup_sha256": rustup_sha256,
            "rustup_home": str(rustup_home),
            "toolchain_binaries": toolchain_binaries,
            "cargo_version": cargo_version,
            "rustc_version": rustc_version,
            "host_triple": host_triple,
        })

        source = run_root / "source" / "oracle"
        status["phase"] = "snapshot"
        _atomic_json(run_record_path, status)
        _assert_no_ancestor_cargo_config(source)
        snapshot = materialize_readonly_snapshot(
            repository,
            REFERENCE_ORACLE_COMMIT,
            source,
            run_root / "snapshot-logs" / "oracle",
            runner,
        )
        snapshot["materialized_root"] = str(source)
        status["source_snapshot"] = snapshot
        cargo_config = _cargo_config_facts(source)
        status["cargo_config_sha256"] = cargo_config

        environment, environment_facts = _single_build_environment(
            run_root, rustup_home, "oracle"
        )
        status["build_environment"] = {"oracle": environment_facts}
        status["phase"] = "metadata"
        _atomic_json(run_record_path, status)
        _metadata, targets = _cargo_metadata(
            source,
            "oracle",
            run_root,
            rustup,
            environment,
            rustup_sha256,
            toolchain_binaries,
            runner,
        )
        require("holla" in targets, "pinned oracle source has no unambiguous Holla binary target")
        target_package = targets["holla"]
        status["metadata_targets"] = {
            "holla": {
                "package_id": target_package.package_id,
                "package_name": target_package.package_name,
                "target_name": target_package.app,
                "manifest_path": str(target_package.manifest_path),
                "source_path": str(target_package.source_path),
            }
        }

        target_dir = run_root / "targets" / "oracle"
        status["build_artifact_root"] = str(target_dir)

        def mark_build_attempt() -> None:
            transition = dict(status)
            transition["phase"] = "build"
            transition["build_result"] = "FAIL"
            transition["build_attempted"] = True
            _atomic_json(run_record_path, transition)
            status.update(transition)

        artifact = _build_binary_artifact(
            source_commit=REFERENCE_ORACLE_COMMIT,
            source=source,
            target_dir=target_dir,
            app="holla",
            target_package=target_package,
            target_triple=target_triple,
            rustup=rustup,
            rustup_sha256=rustup_sha256,
            toolchain_binaries=toolchain_binaries,
            cargo_version=cargo_version,
            rustc_version=rustc_version,
            host_triple=host_triple,
            command_evidence_dir=run_root / "build" / "oracle" / "holla",
            command_context="frozen visual-baseline tag Holla",
            build_environment=environment,
            runner=runner,
            on_build_attempt=mark_build_attempt,
        )
        executable_path = Path(artifact["executable"]["path"]).resolve()
        require(_inside(executable_path, target_dir.resolve()),
                "oracle executable escaped the isolated build artifact root")
        require(not source.stat().st_mode & 0o222,
                "oracle source snapshot became writable during build")

        receipt = {
            "schema": TAG_BUILDER_RECEIPT_SCHEMA,
            "run_id": run_id,
            "purpose": "frozen-visual-tag-holla-build-only",
            "build_result": "PASS",
            "qualification": {"status": "blocked", "reason": qualification_reason},
            "oracle_lineage": identities,
            "source_snapshot": snapshot,
            "build_artifact_root": str(target_dir),
            "build": artifact["build"],
            "executable": artifact["executable"],
            "builder_receipt": artifact["builder_receipt"],
            "builder_receipt_sha256": artifact["builder_receipt_sha256"],
            "build_environment": {
                "cargo_config_sha256": {"oracle": cargo_config},
                "external_cargo_config_policy": (
                    "reject-ancestor-config-and-hash-source-local-config"
                ),
                "rustup": {
                    "path": str(rustup_path),
                    "sha256": rustup_sha256,
                    "home": str(rustup_home),
                },
                "toolchain_binaries": toolchain_binaries,
                "cargo_version": cargo_version,
                "rustc_version": rustc_version,
                "host_triple": host_triple,
                "target_triple": target_triple,
                "environment": {"oracle": environment_facts},
            },
            "capture_status": "NOT_RUN",
            "admission_status": "NOT_RUN",
        }
        receipt_path = run_root / "tag-builder-receipt.json"
        _atomic_json(receipt_path, receipt)
        receipt_sha256 = sha256_file(receipt_path)
        status.update({
            "state": "complete",
            "phase": "complete",
            "build_result": "PASS",
            "tag_builder_receipt": str(receipt_path),
            "tag_builder_receipt_sha256": receipt_sha256,
            "executable_path": artifact["executable"]["path"],
            "executable_sha256": artifact["executable"]["sha256"],
        })
        _atomic_json(run_record_path, status)
        return receipt_path
    except Exception as error:
        if status["build_result"] == "PASS":
            status["build_result"] = "FAIL"
        elif status.get("build_attempted"):
            status["build_result"] = "FAIL"
        else:
            status["build_result"] = "BLOCKED"
        if receipt_path is not None:
            try:
                receipt_path.unlink()
            except FileNotFoundError:
                pass
            except OSError as cleanup_error:
                status["receipt_cleanup_error"] = str(cleanup_error)
        status["state"] = "failed"
        status["error"] = str(error)
        _atomic_json(run_record_path, status)
        raise TagBuildError(str(error), status["build_result"]) from error


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=ROOT,
                        help="Git repository containing the pinned commits (default: this checkout)")
    parser.add_argument("--candidate-commit", required=True,
                        help="accepted full candidate source commit SHA")
    parser.add_argument("--reference-commit", required=True,
                        help="full commit SHA equal to the source-facts and fetched branch tip")
    parser.add_argument("--source-facts-path", type=Path, required=True,
                        help="source-facts record accepted by the caller")
    parser.add_argument("--source-facts-sha256", required=True,
                        help="caller-accepted SHA-256 of the exact source-facts bytes")
    parser.add_argument("--evidence-root", type=Path, required=True,
                        help="new evidence directory, normally evidence/subjects")
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--app", choices=APPS, required=True,
                        help="one app per manifest; the suite schema binds exactly one source pair")
    parser.add_argument("--suite-revision", required=True)
    parser.add_argument("--suite-sha256", required=True,
                        help="digest from the independently accepted suite record")
    parser.add_argument("--expected-generation-id")
    parser.add_argument("--expected-generation-sha256")
    parser.add_argument("--expected-generation-root", type=Path)
    parser.add_argument("--target-triple", required=True,
                        help="explicit native Rust target triple")
    parser.add_argument("--reference-output-root", type=Path, required=True)
    parser.add_argument("--candidate-output-root", type=Path, required=True)
    return parser


def _tag_builder_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="subjects.py build-frozen-tag-holla",
        description="Build Holla from the immutable visual-baseline tag only",
    )
    parser.add_argument("--repository", type=Path, default=ROOT,
                        help="Git repository containing the immutable visual-baseline tag")
    parser.add_argument("--evidence-root", type=Path, required=True,
                        help="new external evidence root for this tag-only build")
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--target-triple", required=True,
                        help="explicit native Rust target triple")
    return parser


def _tag_validator_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="subjects.py validate-frozen-tag-holla",
        description="Validate immutable-tag Holla source closure and receipt consistency without trusting build execution",
    )
    parser.add_argument("--repository", type=Path, required=True,
                        help="absolute Git object repository containing the fixed tag commit")
    parser.add_argument("--receipt", type=Path, required=True,
                        help="absolute tag-builder-receipt.json path")
    parser.add_argument("--receipt-sha256", required=True,
                        help="expected raw SHA-256 of the exact receipt bytes")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    raw_argv = list(sys.argv[1:] if argv is None else argv)
    if raw_argv and raw_argv[0] == "validate-frozen-tag-holla":
        args = _tag_validator_parser().parse_args(raw_argv[1:])
        try:
            identity = validate_frozen_tag_holla_bundle(
                repository=args.repository,
                receipt_path=args.receipt,
                expected_receipt_sha256=args.receipt_sha256,
            )
        except (SubjectError, OSError, ValueError, TypeError) as error:
            print("subjects: {}".format(error), file=sys.stderr)
            return 2
        print(json.dumps(identity, sort_keys=True, separators=(",", ":"), ensure_ascii=False))
        return 0
    if raw_argv and raw_argv[0] == "build-frozen-tag-holla":
        args = _tag_builder_parser().parse_args(raw_argv[1:])
        try:
            receipt_path = build_frozen_tag_holla(
                repository=args.repository,
                evidence_root=args.evidence_root,
                run_id=args.run_id,
                target_triple=args.target_triple,
            )
        except TagBuildError as error:
            print("subjects: {}".format(error), file=sys.stderr)
            return 3 if error.build_result == "FAIL" else 2
        except SubjectError as error:
            print("subjects: {}".format(error), file=sys.stderr)
            return 2
        except OSError as error:
            print("subjects: tag build evidence I/O failed: {}".format(error), file=sys.stderr)
            return 2
        print(str(receipt_path))
        return 0

    args = _parser().parse_args(raw_argv)
    try:
        manifest_path = build_subject_manifest(
            repository=args.repository,
            reference_commit=args.reference_commit,
            candidate_commit=args.candidate_commit,
            source_facts_path=args.source_facts_path,
            source_facts_sha256=args.source_facts_sha256,
            evidence_root=args.evidence_root,
            run_id=args.run_id,
            app=args.app,
            suite_revision=args.suite_revision,
            suite_sha256=args.suite_sha256,
            expected_generation_id=args.expected_generation_id,
            expected_generation_sha256=args.expected_generation_sha256,
            expected_generation_root=args.expected_generation_root,
            target_triple=args.target_triple,
            reference_output_root=args.reference_output_root,
            candidate_output_root=args.candidate_output_root,
        )
    except SubjectError as error:
        print("subjects: {}".format(error), file=sys.stderr)
        return 2
    print(str(manifest_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
