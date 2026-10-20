#!/usr/bin/env python3
"""Build the frozen CLI oracle and run its PTY suite against paired binaries."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import time


BASELINE_TAG = "visual-baseline"
BASELINE_TAG_OBJECT = "2718a6b8127a522a3d351d0b4bdb7542fc8da542"
BASELINE_COMMIT = "4169783ae883788186d161af09ba9c481bd84e5c"
BASELINE_TREE = "ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297"
TUISCOTTI_REV = "a1ea27f757d75d07b05499276cd932e9f8e067ad"
TEST_TARGET = "visual_baseline"
TEST_PACKAGE = "junie-tui"
APPS = ("showcase", "tablepro", "jackin-preview", "holla")
MAX_TIMEOUT = 230
EXPECTED_SCENARIOS = 28_895
EXPECTED_ORDINARY_TESTS = 20
DEFAULT_CHUNK_SIZE = 32


class RunnerError(RuntimeError):
    pass


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


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def digest_tree(path: Path) -> str:
    h = hashlib.sha256()
    for file in sorted(p for p in path.rglob("*") if p.is_file()):
        h.update(file.relative_to(path).as_posix().encode())
        h.update(b"\0")
        h.update(file.read_bytes())
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


def digest_source(path: Path) -> str:
    h = hashlib.sha256()
    for file in sorted(p for p in path.rglob("*") if p.is_file() and not {"target", "snapshots"}.intersection(p.relative_to(path).parts)):
        h.update(file.relative_to(path).as_posix().encode())
        h.update(b"\0")
        h.update(file.read_bytes())
        h.update(b"\0")
    return h.hexdigest()


def file_manifest(path: Path) -> dict[str, str]:
    return {
        file.relative_to(path).as_posix(): sha256(file)
        for file in sorted(path.rglob("*"))
        if file.is_file() and not {"target", "snapshots"}.intersection(file.relative_to(path).parts)
    }


def freeze_tree_readonly(path: Path) -> None:
    if path.is_symlink() or not path.is_dir():
        raise RunnerError(f"frozen copy must be a real directory: {path}")
    directories = []
    for current, child_dirs, files in os.walk(path, followlinks=False):
        current_path = Path(current)
        directories.append(current_path)
        for name in child_dirs:
            item = current_path / name
            if item.is_symlink() or not stat.S_ISDIR(item.lstat().st_mode):
                raise RunnerError(f"frozen copy contains non-directory or symlink: {item}")
        for name in files:
            item = current_path / name
            if item.is_symlink() or not stat.S_ISREG(item.lstat().st_mode):
                raise RunnerError(f"frozen copy contains non-regular file or symlink: {item}")
            item.chmod(0o444)
    for directory in reversed(directories):
        directory.chmod(0o555)


def safe_extract(tar: tarfile.TarFile, destination: Path) -> None:
    root = destination.resolve()
    for member in tar.getmembers():
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
    tar.extractall(destination)


def paths(repo: Path, root: Path) -> tuple[Path, Path, Path]:
    repo = repo.resolve()
    root = root.resolve()
    source = root / "source"
    workspace = source / "Cargo.toml"
    if not workspace.is_file():
        raise RunnerError(f"frozen source missing; run prepare first: {source}")
    return repo, root, source


def prepare(repo: Path, root: Path, timeout: int) -> None:
    repo = repo.resolve()
    root = root.resolve()
    source = root / "source"
    if source.exists():
        raise RunnerError(f"refusing to replace frozen source: {source}")
    tag_obj = run(["git", "rev-parse", f"{BASELINE_TAG}^{{tag}}"], cwd=repo, timeout=timeout).strip()
    commit = run(["git", "rev-parse", f"{BASELINE_TAG}^{{commit}}"], cwd=repo, timeout=timeout).strip()
    if tag_obj != BASELINE_TAG_OBJECT or commit != BASELINE_COMMIT:
        raise RunnerError(f"tag identity mismatch: object={tag_obj}, commit={commit}")
    root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryFile() as bundle:
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
        source.mkdir()
        bundle.seek(0)
        with tarfile.open(fileobj=bundle, mode="r:") as tar:
            safe_extract(tar, source)
    lock = (source / "Cargo.lock").read_text()
    if f'rev = "{TUISCOTTI_REV}"' not in lock and TUISCOTTI_REV not in lock:
        raise RunnerError(f"frozen Cargo.lock does not record tuiscotti {TUISCOTTI_REV}")
    tree = run(["git", "rev-parse", f"{commit}^{{tree}}"], cwd=repo, timeout=timeout).strip()
    if tree != BASELINE_TREE:
        raise RunnerError(f"frozen tree mismatch: {tree}")
    manifest = {
        "tag": BASELINE_TAG,
        "tag_object": tag_obj,
        "source_commit": commit,
        "source_tree": tree,
        "tuiscotti_rev": TUISCOTTI_REV,
        "source_sha256": digest_source(source),
        "snapshots_sha256": digest_tree(source / "snapshots"),
    }
    (root / "provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


def verify_source(root: Path, source: Path) -> str:
    manifest = json.loads((root / "provenance.json").read_text())
    if manifest.get("source_commit") != BASELINE_COMMIT or manifest.get("tag_object") != BASELINE_TAG_OBJECT or manifest.get("source_tree") != BASELINE_TREE:
        raise RunnerError("frozen provenance identity mismatch")
    current = digest_source(source)
    if current != manifest.get("source_sha256"):
        raise RunnerError("frozen source was modified after extraction")
    if digest_tree(source / "snapshots") != manifest.get("snapshots_sha256"):
        raise RunnerError("frozen snapshots were modified after extraction")
    return current


def prepare_live_source(repo: Path, root: Path, source: Path, timeout: int) -> dict[str, object]:
    live = root / "live-source"
    if live.exists():
        raise RunnerError(f"refusing to replace prepared live source: {live}")
    patch_path = repo / "tools" / "frozen_cli_pair.patch"
    helper_path = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    if not patch_path.is_file() or not helper_path.is_file():
        raise RunnerError("live harness patch or frame helper missing from candidate checkout")
    before = file_manifest(source)
    shutil.copytree(source, live, ignore=shutil.ignore_patterns("target"), symlinks=True)
    snapshot_hash = digest_tree(source / "snapshots")
    if digest_tree(live / "snapshots") != snapshot_hash:
        raise RunnerError("live source snapshot copy differs from frozen snapshots")
    freeze_tree_readonly(live / "snapshots")
    run(["patch", "--batch", "--forward", "-p1", "-i", str(patch_path.resolve())], cwd=live, timeout=timeout, quiet=True)
    module = live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"
    shutil.copyfile(helper_path, module)
    expected = {"tests/visual_baseline/main.rs", "tests/visual_baseline/support.rs", "tests/visual_baseline/scenarios/mod.rs", "tests/visual_baseline/frozen_cli_pair.rs"}
    after = file_manifest(live)
    changed = {name for name in set(before) | set(after) if before.get(name) != after.get(name)}
    if changed != expected:
        raise RunnerError(f"live harness patch changed unexpected frozen source files: {sorted(changed)}")
    live_hash = digest_source(live)
    info = {
        "frozen_source_sha256": digest_source(source),
        "live_source_sha256": live_hash,
        "patch_sha256": sha256(patch_path),
        "helper_sha256": sha256(helper_path),
        "module_sha256": sha256(module),
        "frozen_snapshots_sha256": snapshot_hash,
        "live_snapshots_sha256": digest_tree(live / "snapshots"),
        "changed_files": sorted(changed),
    }
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(root / "live-target")
    inventory = nextest_inventory(live, timeout, env)
    info["test_inventory_sha256"] = inventory_digest(inventory)
    info["registered_scenarios"] = sum(inventory.values())
    info["ordinary_checks"] = len(inventory) - sum(inventory.values())
    (root / "live-source.json").write_text(json.dumps(info, indent=2) + "\n")
    return info


def verify_live_source(repo: Path, root: Path, source: Path, timeout: int) -> tuple[Path, dict[str, object]]:
    live = root / "live-source"
    manifest_path = root / "live-source.json"
    if not live.is_dir() or not manifest_path.is_file():
        raise RunnerError("live source missing; run prepare-live first")
    recorded = json.loads(manifest_path.read_text())
    patch = repo / "tools" / "frozen_cli_pair.patch"
    helper = repo / "tools" / "frozen-cli-frame-support" / "src" / "lib.rs"
    current = {
        "frozen_source_sha256": digest_source(source),
        "live_source_sha256": digest_source(live),
        "patch_sha256": sha256(patch),
        "helper_sha256": sha256(helper),
        "module_sha256": sha256(live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"),
        "frozen_snapshots_sha256": digest_tree(source / "snapshots"),
        "live_snapshots_sha256": digest_tree(live / "snapshots"),
    }
    for key, value in current.items():
        if recorded.get(key) != value:
            raise RunnerError(f"live source provenance changed: {key}")
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


def build(repo: Path, root: Path, timeout: int) -> None:
    repo, root, source = paths(repo, root)
    source_hash = verify_source(root, source)
    target = root / "cargo-target"
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(target)
    run(["cargo", "build", "--locked", "--bins", "--manifest-path", str(source / "Cargo.toml")], cwd=source, timeout=timeout, env=env)
    env["VISUAL_BIN_DIR"] = str(target / "debug")
    env["VISUAL_FULL"] = "1"
    env["VISUAL_REVIEW"] = "0"
    run(["cargo", "nextest", "run", "--no-run", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET], cwd=source, timeout=timeout, env=env)
    hashes = {}
    for app in APPS:
        binary = target / "debug" / app
        if not binary.is_file():
            raise RunnerError(f"missing frozen binary: {binary}")
        hashes[app] = sha256(binary)
    record = {"source_sha256": source_hash, "binaries": hashes}
    (root / "build.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))


def build_candidate(repo: Path, root: Path, target: Path, timeout: int) -> None:
    repo = repo.resolve()
    root = root.resolve()
    target = target.resolve()
    if not (repo / "Cargo.toml").is_file():
        raise RunnerError(f"candidate workspace manifest missing: {repo / 'Cargo.toml'}")
    baseline_target = (root / "cargo-target").resolve()
    if target == baseline_target or baseline_target in target.parents or target in baseline_target.parents:
        raise RunnerError("candidate target directory overlaps immutable frozen build directory")
    revision, dirty_fingerprint, dirty_files = git_state(repo, timeout)
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(target)
    run(["cargo", "build", "--locked", "--bins", "--manifest-path", str(repo / "Cargo.toml")], cwd=repo, timeout=timeout, env=env)
    after_revision, after_fingerprint, after_files = git_state(repo, timeout)
    if (after_revision, after_fingerprint, after_files) != (revision, dirty_fingerprint, dirty_files):
        raise RunnerError("candidate source changed while binaries were building")
    hashes = {}
    for app in APPS:
        binary = target / "debug" / app
        if not binary.is_file():
            raise RunnerError(f"missing candidate binary: {binary}")
        hashes[app] = sha256(binary)
    record = {"repo": str(repo), "revision": revision, "dirty_fingerprint": dirty_fingerprint, "dirty_files": dirty_files, "target_dir": str(target), "bin_dir": str(target / "debug"), "binaries": hashes}
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


def nextest_inventory(source: Path, timeout: int, env: dict[str, str]) -> dict[str, bool]:
    command = ["cargo", "nextest", "list", "--locked", "--manifest-path", str(source / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--message-format", "json"]
    listing = run(command, cwd=source, timeout=timeout, env=env, quiet=True)
    try:
        data = json.loads(listing)
        suites = data["rust-suites"]
        suites = suites.values() if isinstance(suites, dict) else suites
        inventory: dict[str, bool] = {}
        for suite in suites:
            testcases = suite["testcases"]
            if not isinstance(testcases, dict):
                raise TypeError("testcases is not a name-keyed map")
            for name, record in testcases.items():
                if name in inventory:
                    raise RunnerError(f"duplicate Nextest case name: {name}")
                inventory[name] = bool(record.get("ignored"))
    except (ValueError, KeyError, TypeError) as error:
        raise RunnerError("could not parse complete Nextest JSON test inventory") from error
    if len(inventory) != EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS:
        raise RunnerError(f"registered case inventory changed: {len(inventory)}; expected {EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS}")
    scenarios = sum(inventory.values())
    ordinary = len(inventory) - scenarios
    if (scenarios, ordinary) != (EXPECTED_SCENARIOS, EXPECTED_ORDINARY_TESTS):
        raise RunnerError(f"registered coverage changed: {scenarios} scenarios and {ordinary} ordinary; expected {EXPECTED_SCENARIOS} and {EXPECTED_ORDINARY_TESTS}")
    return inventory


def inventory_digest(inventory: dict[str, bool]) -> str:
    serialized = json.dumps(sorted(inventory.items()), separators=(",", ":")).encode()
    return hashlib.sha256(serialized).hexdigest()


def require_inventory_matches(inventory: dict[str, bool], expected_digest: str) -> None:
    actual = inventory_digest(inventory)
    if actual != expected_digest:
        raise RunnerError(f"Nextest inventory hash mismatch: expected {expected_digest}, got {actual}")


def selected_expression(names: list[str]) -> str:
    if not names:
        raise RunnerError("cannot build an empty exact Nextest selection")
    return " | ".join(f"test(={name})" for name in names)


def scenario_case_id(test_name: str) -> str:
    return test_name.replace("::", "/")


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


def read_receipts(receipt_root: Path, selected: list[str], expected_root: Path) -> dict[tuple[str, int, str], str]:
    _, receipt_entries = sealed_tree(receipt_root)
    flat_sealed_store(expected_root, "runtime expected")
    receipts: dict[tuple[str, int, str], str] = {}
    expected_receipt_entries: set[str] = set()
    expected_cases = set(selected)
    for path in sorted(receipt_root.rglob("*.json")):
        try:
            record = json.loads(path.read_text())
            case = record["case_id"]
            ordinal = record["ordinal"]
            logical = record["logical_name"]
            digest = record["sha256"]
            if not isinstance(case, str) or not isinstance(ordinal, int) or ordinal < 0 or not isinstance(logical, str) or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise TypeError("invalid receipt field")
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise RunnerError(f"invalid checkpoint receipt: {path}") from error
        if case not in expected_cases:
            raise RunnerError(f"receipt belongs to an unselected scenario: {case}")
        key = (case, ordinal, logical)
        if key in receipts:
            raise RunnerError(f"duplicate checkpoint receipt: {key}")
        case_hex = case.encode().hex()
        logical_hex = logical.encode().hex()
        expected_receipt = receipt_root / case_hex / f"{ordinal:08}-{logical_hex}.json"
        if path != expected_receipt:
            raise RunnerError(f"checkpoint receipt path does not match its identity: {path}")
        expected_receipt_entries.add(path.relative_to(receipt_root).as_posix())
        expected_frame = expected_root / f"case-{case_hex}-{ordinal:08}-{logical_hex}.ansi"
        if expected_frame.is_symlink() or not expected_frame.is_file() or sha256(expected_frame) != digest:
            raise RunnerError(f"receipt frame missing or hash mismatch: {expected_frame}")
        receipts[key] = digest
    counts = {case: 0 for case in selected}
    for case, _, _ in receipts:
        counts[case] += 1
    missing = sorted(case for case, count in counts.items() if count == 0)
    if missing:
        raise RunnerError(f"successful scenarios missing checkpoint receipts: {missing[:5]}")
    expected_receipt_dirs = {case.encode().hex() + "/" for case, _, _ in receipts}
    expected_receipt_entries.update(expected_receipt_dirs)
    if receipt_entries != expected_receipt_entries:
        raise RunnerError(f"receipt store inventory mismatch: extra={sorted(receipt_entries - expected_receipt_entries)[:3]}, missing={sorted(expected_receipt_entries - receipt_entries)[:3]}")
    if not receipts:
        raise RunnerError("phase produced no checkpoint receipts")
    return receipts


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


def baseline_and_candidate_records(repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, timeout: int) -> tuple[dict[str, object], dict[str, object]]:
    source = root / "source"
    source_hash = verify_source(root, source)
    build_record = json.loads((root / "build.json").read_text())
    if build_record.get("source_sha256") != source_hash:
        raise RunnerError("frozen build provenance does not match extracted source")
    if baseline_bins.resolve() != (root / "cargo-target" / "debug").resolve():
        raise RunnerError("baseline binaries must be the isolated binaries built by this runner")
    if build_record.get("binaries") != {app: sha256(baseline_bins / app) for app in APPS}:
        raise RunnerError("frozen baseline binaries changed after the recorded build")
    candidate = json.loads((root / "candidate-build.json").read_text())
    if Path(candidate["bin_dir"]).resolve() != candidate_bins.resolve():
        raise RunnerError("candidate binary directory does not match candidate-build.json")
    revision, dirty_hash, dirty_files = git_state(repo, timeout)
    if (revision, dirty_hash, dirty_files) != (candidate.get("revision"), candidate.get("dirty_fingerprint"), candidate.get("dirty_files")):
        raise RunnerError("candidate source changed after its recorded build")
    if candidate.get("binaries") != {app: sha256(candidate_bins / app) for app in APPS}:
        raise RunnerError("candidate binaries changed after their recorded build")
    candidate["source_tree"] = run(["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"], cwd=repo, timeout=timeout).strip()
    if baseline_bins.resolve() == candidate_bins.resolve() or not candidate_bins.is_dir():
        raise RunnerError("baseline and candidate binary directories must be distinct existing directories")
    for app in APPS:
        for directory in (baseline_bins, candidate_bins):
            if not (directory / app).is_file():
                raise RunnerError(f"missing binary: {directory / app}")
    return build_record, candidate


def exact_receipt_differences(left: dict[tuple[str, int, str], str], right: dict[tuple[str, int, str], str]) -> list[str]:
    differences = []
    for key in sorted(set(left) | set(right)):
        if key not in left:
            differences.append(f"candidate-only:{key}")
        elif key not in right:
            differences.append(f"baseline-only:{key}")
        elif left[key] != right[key]:
            differences.append(f"checkpoint-digest-differs:{key}")
    return differences


def scenario_chunks(names: list[str], size: int) -> list[list[str]]:
    if not 1 <= size <= 512:
        raise RunnerError("chunk size must be 1..512 cases")
    return [names[start : start + size] for start in range(0, len(names), size)]


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


def run_live_chunk(live: Path, root: Path, run_root: Path, phase: str, bins: Path, expected_root: Path, selected: list[str], index: int, timeout: int) -> dict[str, object]:
    chunk_name = f"chunk-{index:05}"
    chunk_root = run_root / phase / chunk_name
    receipt_root = chunk_root / "receipts"
    logs = chunk_root / "logs"
    receipt_root.mkdir(parents=True, exist_ok=False)
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
    })
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
    receipts: dict[tuple[str, int, str], str] = {}
    if failure is None:
        try:
            receipts = read_receipts(receipt_root, [scenario_case_id(name) for name in selected], expected_root)
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
        "status": "failed" if failure else "passed",
        "failure": failure,
        "selected_cases": selected,
        "successful_test_events": len(events),
        "checkpoint_count": len(receipts),
        "checkpoint_inventory_sha256": hashlib.sha256(json.dumps([[*key, value] for key, value in sorted(receipts.items())], separators=(",", ":")).encode()).hexdigest(),
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


def phase_receipts(run_root: Path, phase: str, selected: list[str], expected_root: Path) -> dict[tuple[str, int, str], str]:
    combined: dict[tuple[str, int, str], str] = {}
    for chunk in sorted((run_root / phase).glob("chunk-*")):
        names = json.loads((chunk / "report.json").read_text())["selected_cases"]
        chunk_cases = [scenario_case_id(name) for name in names]
        chunk_receipts = read_receipts(chunk / "receipts", chunk_cases, expected_root)
        overlap = set(combined) & set(chunk_receipts)
        if overlap:
            raise RunnerError(f"duplicate phase checkpoint identities: {sorted(overlap)[:3]}")
        combined.update(chunk_receipts)
    counts = {scenario_case_id(name): 0 for name in selected}
    for case, _, _ in combined:
        if case not in counts:
            raise RunnerError(f"phase contains unselected scenario receipt: {case}")
        counts[case] += 1
    missing = sorted(name for name, count in counts.items() if count == 0)
    if missing:
        raise RunnerError(f"phase scenario coverage incomplete: {missing[:5]}")
    expected_files = {f"case-{case.encode().hex()}-{ordinal:08}-{logical.encode().hex()}.ansi" for case, ordinal, logical in combined}
    _, actual_files = flat_sealed_store(expected_root, "runtime expected")
    if actual_files != expected_files:
        raise RunnerError(f"runtime expected frame inventory mismatch: missing={sorted(expected_files - actual_files)[:3]}, extra={sorted(actual_files - expected_files)[:3]}")
    return combined


def run_ordinary_checks(live: Path, root: Path, run_root: Path, inventory: dict[str, bool], timeout: int) -> dict[str, object]:
    selected = sorted(name for name, ignored in inventory.items() if not ignored)
    env = clean_env()
    if any(key.startswith("TERMROCK_PAIR_") for key in env):
        raise RunnerError("pair runtime flags leaked into ordinary checks")
    env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
    command = ["cargo", "nextest", "run", "--locked", "--manifest-path", str(live / "Cargo.toml"), "-p", TEST_PACKAGE, "--test", TEST_TARGET, "--filter-expr", selected_expression(selected), "--message-format", "libtest-json-plus", "--no-fail-fast"]
    output_dir = run_root / "ordinary"
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
    result = {"status": "failed" if failure else "passed", "failure": failure, "selected_cases": selected, "passed_test_events": len(events), "stdout_sha256": sha256(stdout_gzip) if stdout_gzip.is_file() else None, "stderr_sha256": sha256(output_dir / "stderr.log") if (output_dir / "stderr.log").is_file() else None}
    (output_dir / "report.json").write_text(json.dumps(result, indent=2) + "\n")
    if failure:
        raise RunnerError(f"ordinary non-scenario checks failed; report saved: {failure}")
    return result


def compare_live(repo: Path, candidate_repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, selectors: list[str] | None, all_cases: bool, chunk_size: int, timeout: int) -> None:
    repo, root, source = paths(repo, root)
    candidate_repo = candidate_repo.resolve()
    _, candidate = baseline_and_candidate_records(candidate_repo, root, baseline_bins, candidate_bins, timeout)
    live, live_record = verify_live_source(repo, root, source, timeout)
    env = clean_env()
    env.update({"CARGO_TARGET_DIR": str(root / "live-target"), "NEXTEST_EXPERIMENTAL_LIBTEST_JSON": "1"})
    inventory = nextest_inventory(live, timeout, env)
    require_inventory_matches(inventory, str(live_record.get("test_inventory_sha256")))
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
        for index, names in enumerate(chunks):
            try:
                phase_chunks.append(run_live_chunk(live, root, run_root, phase, bins, expected_root, names, index, timeout))
            except RunnerError as error:
                phase_chunks.append({"chunk": index, "status": "failed", "failure": str(error), "selected_cases": names})
                report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "complete registered scenarios pending integrity checks" if all_cases else "partial", "acceptance_complete": False, "selected_case_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "remaining_chunks": len(chunks) - index - 1}
                (run_root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
                raise RunnerError(f"{phase} stopped at {index + 1}/{len(chunks)} chunks; evidence saved at {run_root}: {error}")
        try:
            validate_chunk_coverage(selected, phase_chunks, chunk_size)
        except RunnerError as error:
            failed_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", "acceptance_complete": False, "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "failure": str(error)}
            (run_root / "report.json").write_text(json.dumps(failed_report, indent=2) + "\n")
            raise RunnerError(f"{phase} chunk partition incomplete; report saved at {run_root}: {error}")
        phase_summary: dict[str, object] = {"phase": phase, "status": "passed", "selected_cases": len(selected), "chunks": phase_chunks, "elapsed_seconds": round(time.monotonic() - started, 2)}
        try:
            if phase != "baseline-capture" and flat_sealed_store(expected_root, "runtime expected")[0] != expected_before:
                raise RunnerError(f"{phase} modified runtime expected frames")
            phase_summary["receipts"] = phase_receipts(run_root, phase, selected, expected_root)
            phase_summary["runtime_expected_sha256"] = flat_sealed_store(expected_root, "runtime expected")[0]
            verify_source(root, source)
            current_live, current_live_record = verify_live_source(repo, root, source, timeout)
            if current_live != live or current_live_record != live_record:
                raise RunnerError("live harness patch, helper, or source changed during paired run")
        except RunnerError as error:
            phase_summary["status"] = "failed"
            phase_summary["failure"] = str(error)
            phase_reports = [{key: value for key, value in item.items() if key != "receipts"} for item in [*phases, phase_summary]]
            failure_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", "acceptance_complete": False, "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phases": phase_reports, "artifacts_root": str(run_root)}
            (run_root / "report.json").write_text(json.dumps(failure_report, indent=2) + "\n")
            raise RunnerError(f"{phase} phase incomplete; evidence saved at {run_root}: {error}")
        phases.append(phase_summary)
        phase_reports = [{key: value for key, value in item.items() if key != "receipts"} for item in phases]
        (run_root / "report.json").write_text(json.dumps({"phase": phase, "phases": phase_reports}, indent=2) + "\n")
    receipts = [phase["receipts"] for phase in phases]
    differences = exact_receipt_differences(receipts[0], receipts[1]) + exact_receipt_differences(receipts[0], receipts[2])
    if differences:
        raise RunnerError(f"live checkpoint mismatch; see {run_root}: {differences[:5]}")
    ordinary = run_ordinary_checks(live, root, run_root, inventory, timeout) if all_cases else None
    coverage_complete = all_cases and ordinary is not None and len(ordinary["selected_cases"]) == EXPECTED_ORDINARY_TESTS
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
        "candidate_revision": candidate["revision"],
        "candidate_source_tree": candidate["source_tree"],
        "candidate_dirty_fingerprint": candidate["dirty_fingerprint"],
        "candidate_dirty_files": candidate["dirty_files"],
        "candidate_binaries": candidate["binaries"],
        "case_selection": "all registered scenarios" if all_cases else "explicit cases",
        "selection_completeness": "complete" if coverage_complete else "partial",
        "expected_registered_scenarios": EXPECTED_SCENARIOS,
        "test_inventory_sha256": inventory_digest(inventory),
        "selected_scenario_count": len(selected),
        "ordinary_checks": ordinary,
        "phases": [{key: value for key, value in phase.items() if key != "receipts"} for phase in phases],
        "checkpoint_count": len(receipts[0]),
        "checkpoint_receipt_sha256": [hashlib.sha256(json.dumps([[*key, value] for key, value in sorted(receipt.items())], separators=(",", ":")).encode()).hexdigest() for receipt in receipts],
        "checkpoint_differences": differences,
        "acceptance_complete": coverage_complete and not differences,
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
    sub.add_parser("prepare")
    sub.add_parser("prepare-live")
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
    return ap


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            prepare(args.repo, args.root, args.timeout)
        elif args.command == "prepare-live":
            repo, root, source = paths(args.repo, args.root)
            verify_source(root, source)
            record = prepare_live_source(repo, root, source, args.timeout)
            print(json.dumps(record, indent=2))
        elif args.command == "build":
            build(args.repo, args.root, args.timeout)
        elif args.command == "build-candidate":
            root = args.root.resolve()
            build_candidate(args.repo, root, args.target_dir or root / "candidate-target", args.timeout)
        elif args.command == "live-compare":
            root = args.root.resolve()
            baseline_bins = root / "cargo-target" / "debug"
            compare_live(args.repo, args.candidate_repo or args.repo, root, baseline_bins, args.candidate_bin_dir.resolve(), args.case, args.all, args.chunk_size, args.timeout)
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
