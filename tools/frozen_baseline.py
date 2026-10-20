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
EXTENSION_MANIFEST = Path("tools/frozen_cli_extensions.json")
EXTENSION_SOURCE = Path("tools/frozen_cli_extensions.rs")
EXTENSION_CASE_PREFIX = "termrock_extensions::showcase::table_sort::"
EXTENSION_MANIFEST_SHA256 = "b995c00f0dc14d4e0a81d1d56140e1c10ba7936a043e0135196f8fe8421cabb8"
EXTENSION_SOURCE_SHA256 = "332bd4eac4dc65be1a1478f97177cc658113a063900d8b9c6a0ba2ddc6dc67d9"
EXTENSION_PATCH_SHA256 = "646d96b5b08c182f9ecdef8e8e8ddf1f6e31095d4b2a073a7eda17d611d69951"
REFERENCE_DECISIONS = Path("tools/frozen_cli_references.json")
REFERENCE_DECISIONS_SHA256 = "d5d8d6489a8968d6e84941de3fd4b778ed09c879376ee2d575c6d1591c59abe3"
REFERENCE_CASE = "showcase::resize::overview_grown::120x40::truecolor"
REFERENCE_TUPLE = ("showcase/resize/overview_grown/120x40/truecolor", 0, "showcase/resize/overview_grown/120x40/truecolor")
BUTTONS_REFERENCE_CASE = "showcase::audit::buttons::120x40::truecolor"
REFERENCE_CASES = frozenset({REFERENCE_CASE, BUTTONS_REFERENCE_CASE})
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
    extension_manifest_path = repo / EXTENSION_MANIFEST
    extension_source_path = repo / EXTENSION_SOURCE
    if not patch_path.is_file() or not helper_path.is_file() or not extension_manifest_path.is_file() or not extension_source_path.is_file():
        raise RunnerError("live harness patch, frame helper, or visual extension inputs missing from candidate checkout")
    if sha256(patch_path) != EXTENSION_PATCH_SHA256:
        raise RunnerError("visual extension test patch differs from the pinned extension revision")
    if sha256(extension_manifest_path) != EXTENSION_MANIFEST_SHA256 or sha256(extension_source_path) != EXTENSION_SOURCE_SHA256:
        raise RunnerError("visual extension manifest or Rust module differs from the pinned extension revision")
    extension_manifest = load_extension_manifest(extension_manifest_path)
    _, reference_decision_hash = load_reference_decisions(repo, root, source)
    before = file_manifest(source)
    shutil.copytree(source, live, ignore=shutil.ignore_patterns("target"), symlinks=True)
    snapshot_hash = digest_tree(source / "snapshots")
    if digest_tree(live / "snapshots") != snapshot_hash:
        raise RunnerError("live source snapshot copy differs from frozen snapshots")
    freeze_tree_readonly(live / "snapshots")
    run(["patch", "--batch", "--forward", "-V", "none", "-p1", "-i", str(patch_path.resolve())], cwd=live, timeout=timeout, quiet=True)
    module = live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"
    extension_module = live / "tests" / "visual_baseline" / "frozen_cli_extensions.rs"
    extension_manifest_copy = live / "tests" / "visual_baseline" / "frozen_cli_extensions.json"
    shutil.copyfile(helper_path, module)
    shutil.copyfile(extension_source_path, extension_module)
    shutil.copyfile(extension_manifest_path, extension_manifest_copy)
    expected = {"tests/visual_baseline/main.rs", "tests/visual_baseline/support.rs", "tests/visual_baseline/scenarios/mod.rs", "tests/visual_baseline/scenarios/interact.rs", "tests/visual_baseline/frozen_cli_pair.rs", "tests/visual_baseline/frozen_cli_extensions.rs", "tests/visual_baseline/frozen_cli_extensions.json"}
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
        "extension_manifest_sha256": sha256(extension_manifest_path),
        "extension_manifest_copy_sha256": sha256(extension_manifest_copy),
        "extension_source_sha256": sha256(extension_source_path),
        "extension_module_sha256": sha256(extension_module),
        "extension_case_count": len(extension_manifest["generated_case_ids"]),
        "reference_decisions_sha256": reference_decision_hash,
        "extension_inventory_sha256": hashlib.sha256(json.dumps(extension_manifest["generated_case_ids"], separators=(",", ":")).encode()).hexdigest(),
        "frozen_snapshots_sha256": snapshot_hash,
        "live_snapshots_sha256": digest_tree(live / "snapshots"),
        "changed_files": sorted(changed),
    }
    env = clean_env()
    env["CARGO_TARGET_DIR"] = str(root / "live-target")
    inventory = nextest_inventory(live, timeout, env, extension_manifest)
    original_inventory, _ = validate_extension_inventory(inventory, extension_manifest)
    info["test_inventory_sha256"] = inventory_digest(inventory)
    info["original_test_inventory_sha256"] = inventory_digest(original_inventory)
    info["registered_scenarios"] = EXPECTED_SCENARIOS
    info["extension_scenarios"] = len(extension_manifest["generated_case_ids"])
    info["ordinary_checks"] = EXPECTED_ORDINARY_TESTS
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
    extension_manifest = repo / EXTENSION_MANIFEST
    extension_source = repo / EXTENSION_SOURCE
    reference_path = repo / REFERENCE_DECISIONS
    reference_decision_hash = sha256(reference_path) if reference_path.is_file() and not reference_path.is_symlink() else ""
    current = {
        "frozen_source_sha256": digest_source(source),
        "live_source_sha256": digest_source(live),
        "patch_sha256": sha256(patch),
        "helper_sha256": sha256(helper),
        "module_sha256": sha256(live / "tests" / "visual_baseline" / "frozen_cli_pair.rs"),
        "extension_manifest_sha256": sha256(extension_manifest),
        "extension_source_sha256": sha256(extension_source),
        "extension_module_sha256": sha256(live / "tests" / "visual_baseline" / "frozen_cli_extensions.rs"),
        "extension_manifest_copy_sha256": sha256(live / "tests" / "visual_baseline" / "frozen_cli_extensions.json"),
        "reference_decisions_sha256": reference_decision_hash,
        "frozen_snapshots_sha256": digest_tree(source / "snapshots"),
        "live_snapshots_sha256": digest_tree(live / "snapshots"),
    }
    for key, value in current.items():
        if recorded.get(key) != value:
            raise RunnerError(f"live source provenance changed: {key}")
    _, validated_reference_hash = load_reference_decisions(repo, root, source)
    if validated_reference_hash != current["reference_decisions_sha256"]:
        raise RunnerError("reference decision provenance changed")
    if current["extension_manifest_sha256"] != EXTENSION_MANIFEST_SHA256 or current["extension_source_sha256"] != EXTENSION_SOURCE_SHA256:
        raise RunnerError("visual extension inputs differ from the pinned extension revision")
    if current["patch_sha256"] != EXTENSION_PATCH_SHA256:
        raise RunnerError("visual extension test patch differs from the pinned extension revision")
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


def nextest_inventory(source: Path, timeout: int, env: dict[str, str], extension_manifest: dict[str, object] | None = None) -> dict[str, bool]:
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
    if extension_manifest is None:
        scenarios = sum(inventory.values())
        ordinary = len(inventory) - scenarios
        if (scenarios, ordinary) != (EXPECTED_SCENARIOS, EXPECTED_ORDINARY_TESTS):
            raise RunnerError(f"registered coverage changed: {scenarios} scenarios and {ordinary} ordinary; expected {EXPECTED_SCENARIOS} and {EXPECTED_ORDINARY_TESTS}")
    else:
        validate_extension_inventory(inventory, extension_manifest)
    return inventory


def validate_overview_reference_document(document: object) -> dict[str, object]:
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


def validate_buttons_reference_decision(decision: object) -> dict[str, object]:
    required = {"decision_id", "tuple", "baseline", "control", "environment", "legacy_approval", "live_frame", "independent_captures", "independent_probe"}
    if not isinstance(decision, dict) or set(decision) != required:
        raise RunnerError("Buttons reference decision fields differ from the reviewed schema")
    expected_tuple = {
        "test_id": BUTTONS_REFERENCE_CASE,
        "case_id": "showcase/audit/buttons/120x40/truecolor",
        "ordinal": 0,
        "logical_name": "showcase/audit/buttons/120x40/truecolor",
        "application": "showcase",
        "cols": 120,
        "rows": 40,
        "color_mode": "truecolor",
    }
    if decision["decision_id"] != "visual-baseline-showcase-buttons-120x40-2026-10-20" or decision["tuple"] != expected_tuple:
        raise RunnerError("Buttons reference tuple or identity changed")
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
    if decision["baseline"] != expected_baseline:
        raise RunnerError("Buttons reference frozen source/test/binary pins changed")
    expected_control = {
        "argv": ["--page", "buttons", "--color", "truecolor"],
        "size": [120, 40],
        "readiness_text": "Junie Design system",
        "quiet_settle_ms": 400,
        "quiet_deadline_seconds": 8,
        "inputs": [],
        "clock_controls": "none; no motion pause, explicit frame, or ClockStep input",
        "checkpoint_order": [expected_tuple["logical_name"]],
    }
    if decision["control"] != expected_control:
        raise RunnerError("Buttons reference interaction controls changed")
    expected_environment = {
        "runner": {"VISUAL_FULL": "1", "VISUAL_REVIEW": "0"},
        "values": {"VISUAL_FULL": "1", "VISUAL_REVIEW": "0"},
        "child_policy": "frozen support::PINNED_ENV plus per-scenario HOME and cwd",
        "child_values": {"SHELL": "/bin/sh", "JUNIE_TUI_CLIENT_PLATFORM": "macos", "TERM": "xterm-256color", "COLORTERM": "truecolor", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC", "PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOLLA_NO_HISTORY": "1"},
        "color_mode": "truecolor",
        "terminal_profile": "tuiscotti-default",
        "home_cwd_policy": "scenario_home(case_id), below the isolated live-source target/tuiscotti/home directory",
    }
    if decision["environment"] != expected_environment:
        raise RunnerError("Buttons reference environment contract changed")
    if decision["legacy_approval"] != {"path": "showcase/audit/buttons/120x40/truecolor.ansi", "sha256": "3a809ef2e455d376556adfcd73c243db0bca6cdfed0e524ed65b6f11efc916c8", "bytes": 16139, "verdict": "known_stale_cells_differ", "paired_exception": "baseline-capture exact checkpoint only; never edits or replaces the frozen approval"}:
        raise RunnerError("Buttons reference legacy approval record changed")
    expected_live = {"ansi_sha256": "545c129be118753a89a1683c838cfa227d0090cb8ea18d975557021afa9a94e0", "ansi_bytes": 19270, "semantic_frame_sha256": "19fdbdd4aa349bc2c7fc13ebf5c3a245093a02cf2cbc1183be7854a83a7b0107", "cols": 120, "rows": 40, "cell_count": 4800, "cursor": {"x": 0, "y": 0, "visible": False, "style": "Bar", "blinking": False}}
    if decision["live_frame"] != expected_live:
        raise RunnerError("Buttons reference live frame hashes or cursor changed")
    expected_captures = [
        {"run_id": "1792502640440881000-933c43607380", "command_sha256": "2a0bfe477ff48a871cf88c437dd94e0890af39c483406becf7760421b41b6b7a", "stdout_sha256": "b0b1307efd5448cb545df8f4f9d5f5f68f88dfe5e13af5de8f19411a4a7ae34b", "stderr_sha256": "c2152c8d74c0b790be79599425f3cb3a5332021f16aa7d67e942a50e943a6607", "report_sha256": "06a7c36c1684fed1b7086f12a9de4c25dcbeb277810c6b460a13de867bce74d0", "ansi_sha256": expected_live["ansi_sha256"], "semantic_frame_sha256": expected_live["semantic_frame_sha256"]},
        {"run_id": "1792502669657768000-6db31d0ee0cb", "command_sha256": "2a0bfe477ff48a871cf88c437dd94e0890af39c483406becf7760421b41b6b7a", "stdout_sha256": "617aff4914b969bd45bcf85361d6a5f5b9c1d9cb1dc57255b0f575c9490f03be", "stderr_sha256": "955f73a94616812c18b2d30fdf5e0616136f158a76072770090be3381f4235a1", "report_sha256": "955fd8223412cf859b208de652a8a63cd98648c7a690f320effa0bdb41e66bdf", "ansi_sha256": expected_live["ansi_sha256"], "semantic_frame_sha256": expected_live["semantic_frame_sha256"]},
    ]
    if decision["independent_captures"] != expected_captures:
        raise RunnerError("Buttons reference independent capture evidence changed")
    expected_probe = {
        "source_sha256": "92f7ad4bd050872e64cb7c7329af1a312086755212918041bc3753267da325f1",
        "exit_code": 0,
        "compile_command": "/opt/homebrew/opt/rust/bin/rustc --edition=2024 probe.rs -L dependency=../live-target/debug/deps --extern tuiscotti=../live-target/debug/deps/libtuiscotti-8c67b7fb0e369def.rlib -o probe",
        "font_profile": "tuiscotti-default",
        "fonts": {"JetBrainsMonoNerdFontMono-Bold.ttf": "bfcf9a917276ffc058867d87cbc8a5b2f1ab0f4b710e9170dc02763ccb80bd4b", "JetBrainsMonoNerdFontMono-BoldItalic.ttf": "9dba502e00e35209f6ed2a151c7376c051657b067cdebbc6e52d06cb9002cf31", "JetBrainsMonoNerdFontMono-Italic.ttf": "31efd6ead98746f5b0afa1ee6dba60267ad48db36428360bee327bec10621f97", "JetBrainsMonoNerdFontMono-Regular.ttf": "f2a5ea6cfab397445ffab00c0370927b66d61e560a05db5db271b42006381c1a", "NotoSansCJKjp-subset.otf": "777bee41f0c6076c00ad919384359a6e396b8822cf9056041fca8fcf2759d897", "NotoSansSymbols-subset.ttf": "6f9cc93e71f8676361c5db286368be046e75d42c0b841afbf3f50da6bb0a2b8a", "NotoSansSymbols2-subset.ttf": "e1d177a40af910100eceb0e825331e55f0cfd005bc0f26087fd4e58fbe60e6c5"},
        "output": {"ansi_sha256": expected_live["ansi_sha256"], "semantic_frame_json_sha256": expected_live["semantic_frame_sha256"], "text_sha256": "2622b728607ed3d1c63161ce40089333f02234f1a77b33e0ea12e71b1501cfca", "approved_decoded_frame_json_sha256": "2c1e281be06e58654a79767802a96d4356c71dd0de5b5bb51e4914a694fa9917", "approved_decoded_text_sha256": "f0da2ffd75f1549af171927c6325e59395ae87aab7834ae59adb5978f4d0d12f"},
        "frame_comparison": {"complete_cells": 4800, "changed_cells": 2723, "approved_cursor": {"x": 0, "y": 0, "visible": False, "style": "Block", "blinking": False}, "live_cursor": expected_live["cursor"]},
    }
    if decision["independent_probe"] != expected_probe:
        raise RunnerError("Buttons reference independent probe evidence changed")
    return decision


def validate_reference_decision_document(document: object) -> list[dict[str, object]]:
    if not isinstance(document, dict) or set(document) != {"schema_version", "decision_scope", "decisions"}:
        raise RunnerError("reference decision document fields differ from schema version 1")
    if document["schema_version"] != 1 or document["decision_scope"] != "two_exact_stale_approval_live_pair_references":
        raise RunnerError("unsupported reference decision scope")
    decisions = document["decisions"]
    if not isinstance(decisions, list) or len(decisions) != 2:
        raise RunnerError("reference decisions must contain exactly the two reviewed checkpoints")
    if not isinstance(decisions[0], dict) or decisions[0].get("decision_id") != "visual-baseline-overview-grown-120x40-2026-10-20" or not isinstance(decisions[1], dict) or decisions[1].get("decision_id") != "visual-baseline-showcase-buttons-120x40-2026-10-20":
        raise RunnerError("reference decision inventory or order changed")
    overview_document = {"schema_version": 1, "decision_scope": "one_exact_stale_approval_live_pair_reference", "decisions": [decisions[0]]}
    overview = validate_overview_reference_document(overview_document)
    buttons = validate_buttons_reference_decision(decisions[1])
    return [overview, buttons]


def load_reference_decisions(repo: Path, root: Path, source: Path) -> tuple[list[dict[str, object]], str]:
    path = repo / REFERENCE_DECISIONS
    if not path.is_file() or path.is_symlink() or sha256(path) != REFERENCE_DECISIONS_SHA256:
        raise RunnerError("reference decision file is missing or differs from the reviewed pin")
    try:
        document = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise RunnerError("could not parse reference decision file") from error
    decisions = validate_reference_decision_document(document)
    baseline_source = decisions[0]["baseline"]
    build = json.loads((root / "build.json").read_text())
    if digest_source(source) != baseline_source["source_sha256"] or digest_tree(source / "tests" / "visual_baseline") != baseline_source["visual_test_tree_sha256"]:
        raise RunnerError("reference decision source or test suite pin does not match extracted frozen source")
    if build.get("source_sha256") != baseline_source["source_sha256"] or build.get("binaries") != baseline_source["binary_sha256"]:
        raise RunnerError("reference decision baseline binaries are not the reviewed frozen application build")
    for decision in decisions:
        approval = source / "snapshots" / Path(str(decision["legacy_approval"]["path"]))
        if not approval.is_file() or sha256(approval) != decision["legacy_approval"]["sha256"] or approval.stat().st_size != decision["legacy_approval"]["bytes"]:
            raise RunnerError("reference decision legacy approval file changed")
    return decisions, REFERENCE_DECISIONS_SHA256


def load_extension_manifest(path: Path) -> dict[str, object]:
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
    return {**manifest, "generated_case_ids": sorted(generated_ids), "checkpoint_obligations": {case["id"]: case["checkpoints"] for case in generated}}


def validate_extension_inventory(inventory: dict[str, bool], manifest: dict[str, object]) -> tuple[dict[str, bool], list[str]]:
    extension_ids = set(manifest["generated_case_ids"])
    if len(inventory) != EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS + len(extension_ids):
        raise RunnerError("registered inventory count differs from original inventory plus declared extensions")
    extension_inventory = {name: inventory[name] for name in inventory if name in extension_ids}
    missing = extension_ids - set(extension_inventory)
    if missing:
        raise RunnerError(f"extension manifest cases missing from Nextest inventory: {sorted(missing)[:5]}")
    if any(not ignored for ignored in extension_inventory.values()):
        raise RunnerError("extension visual cases must be ignored Nextest scenarios")
    original = {name: ignored for name, ignored in inventory.items() if name not in extension_ids}
    if len(original) != EXPECTED_SCENARIOS + EXPECTED_ORDINARY_TESTS:
        raise RunnerError("extension IDs overlap or alter the original registered inventory")
    scenarios = sum(original.values())
    ordinary = len(original) - scenarios
    if (scenarios, ordinary) != (EXPECTED_SCENARIOS, EXPECTED_ORDINARY_TESTS):
        raise RunnerError(f"original registered coverage changed: {scenarios} scenarios and {ordinary} ordinary")
    return original, sorted(extension_ids)


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
        try:
            record = json.loads(path.read_text())
            case = record["case_id"]
            ordinal = record["ordinal"]
            logical = record["logical_name"]
            ansi_digest = record["sha256"]
            semantic_digest = record["semantic_frame_sha256"]
            reference_digest = record.get("reference_decision_sha256")
            if not isinstance(case, str) or type(ordinal) is not int or ordinal < 0 or not isinstance(logical, str) or not isinstance(ansi_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", ansi_digest) or not isinstance(semantic_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", semantic_digest) or (reference_digest is not None and (not isinstance(reference_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", reference_digest))):
                raise TypeError("invalid receipt field")
            if set(record) not in ({"case_id", "ordinal", "logical_name", "sha256", "semantic_frame_sha256"}, {"case_id", "ordinal", "logical_name", "sha256", "semantic_frame_sha256", "reference_decision_sha256"}):
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
    if receipt_entries != expected_receipt_entries:
        raise RunnerError(f"receipt store inventory mismatch: extra={sorted(receipt_entries - expected_receipt_entries)[:3]}, missing={sorted(expected_receipt_entries - receipt_entries)[:3]}")
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


def scenario_chunks(names: list[str], size: int) -> list[list[str]]:
    if not 1 <= size <= 512:
        raise RunnerError("chunk size must be 1..512 cases")
    reference_names = [name for name in names if name in REFERENCE_CASES]
    if len(reference_names) != len(set(reference_names)):
        raise RunnerError("reference checkpoint scenario is duplicated in selection")
    ordinary = [name for name in names if name not in REFERENCE_CASES]
    chunks = [ordinary[start : start + size] for start in range(0, len(ordinary), size)]
    chunks.extend([[name] for name in reference_names])
    return chunks


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


def validate_reference_receipts(receipts: dict[tuple[str, int, str], tuple[str, str, str | None]], selected: list[str], phase: str, decisions: list[dict[str, object]], decision_sha256: str) -> None:
    selected_decisions = [decision for decision in decisions if reference_case_selected(decision, selected)]
    allowed_claims: dict[tuple[str, int, str], dict[str, object]] = {}
    for decision in selected_decisions:
        reference = decision["tuple"]
        key = (reference["case_id"], reference["ordinal"], reference["logical_name"])
        if key in allowed_claims:
            raise RunnerError(f"duplicate selected reference checkpoint: {key}")
        allowed_claims[key] = decision
        matched = receipts.get(key)
        if matched is None:
            raise RunnerError(f"selected stale-approval reference checkpoint is missing: {key}")
        if matched[0] != decision["live_frame"]["ansi_sha256"] or matched[1] != decision["live_frame"]["semantic_frame_sha256"]:
            raise RunnerError(f"reviewed live-frame reference mismatch at {key}")
        expected_decision_hash = decision_sha256 if phase == "baseline-capture" else None
        if matched[2] != expected_decision_hash:
            raise RunnerError(f"reference decision receipt binding mismatch at {key}: {matched[2]}")
    for key, receipt in receipts.items():
        claim = receipt[2]
        if claim is None:
            continue
        decision = allowed_claims.get(key)
        if phase != "baseline-capture" or decision is None or claim != decision_sha256:
            raise RunnerError(f"unselected or unrelated checkpoint claims a reference decision: {key}")


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


def run_live_chunk(live: Path, root: Path, run_root: Path, phase: str, bins: Path, expected_root: Path, selected: list[str], index: int, timeout: int, extension_manifest: dict[str, object] | None = None, reference_decisions: list[dict[str, object]] | None = None, reference_decision_sha256: str | None = None) -> dict[str, object]:
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
    reference_decisions = reference_decisions or []
    active_references = [decision for decision in reference_decisions if reference_case_selected(decision, selected)]
    if len(active_references) > 1:
        raise RunnerError("a chunk may contain at most one reviewed reference checkpoint")
    if phase == "baseline-capture" and active_references:
        if len(active_references) != 1 or reference_decision_sha256 != REFERENCE_DECISIONS_SHA256:
            raise RunnerError("invalid or unpinned paired-reference decision")
        decision = active_references[0]
        for name, value in decision["environment"]["values"].items():
            env[name] = value
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
    if failure is None:
        try:
            receipts = read_receipts(receipt_root, [scenario_receipt_id(name, extension_manifest) for name in selected], expected_root)
            validate_reference_receipts(receipts, [scenario_receipt_id(name, extension_manifest) for name in selected], phase, reference_decisions, reference_decision_sha256 or "")
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


def ordinary_test_groups(inventory: dict[str, bool]) -> tuple[list[str], list[str]]:
    selected = sorted(name for name, ignored in inventory.items() if not ignored)
    if len(selected) != EXPECTED_ORDINARY_TESTS:
        raise RunnerError(f"ordinary test inventory changed: {len(selected)}; expected {EXPECTED_ORDINARY_TESTS}")
    cli = sorted(name for name in selected if name == "isolation::env_isolation" or name.startswith(("modes::", "startup_input::", "startup_resize::")))
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


def compare_live(repo: Path, candidate_repo: Path, root: Path, baseline_bins: Path, candidate_bins: Path, selectors: list[str] | None, all_cases: bool, chunk_size: int, timeout: int) -> None:
    repo, root, source = paths(repo, root)
    candidate_repo = candidate_repo.resolve()
    _, candidate = baseline_and_candidate_records(candidate_repo, root, baseline_bins, candidate_bins, timeout)
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
                phase_chunks.append(run_live_chunk(live, root, run_root, phase, bins, expected_root, names, index, timeout, extension_manifest, reference_decisions, reference_decision_sha256))
            except RunnerError as error:
                phase_chunks.append({"chunk": index, "status": "failed", "failure": str(error), "selected_cases": names})
                report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "complete registered scenarios pending integrity checks" if all_cases else "partial", **cli_coverage_report_fields(False), "selected_case_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "remaining_chunks": len(chunks) - index - 1}
                (run_root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
                raise RunnerError(f"{phase} stopped at {index + 1}/{len(chunks)} chunks; evidence saved at {run_root}: {error}")
        try:
            validate_chunk_coverage(selected, phase_chunks, chunk_size)
        except RunnerError as error:
            failed_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", **cli_coverage_report_fields(False), "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phase": phase, "phase_chunks": phase_chunks, "failure": str(error)}
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
            failure_report = {"tag_object": BASELINE_TAG_OBJECT, "baseline_commit": BASELINE_COMMIT, "tuiscotti_rev": TUISCOTTI_REV, "live_source": live_record, "candidate_revision": candidate["revision"], "candidate_source_tree": candidate["source_tree"], "candidate_dirty_fingerprint": candidate["dirty_fingerprint"], "candidate_binaries": candidate["binaries"], "reference_decisions_sha256": reference_decision_sha256, "legacy_approval_conflicts": paired_reference_report(reference_decisions, selected, reference_decision_sha256), "case_selection": "all registered scenarios" if all_cases else "explicit cases", "selection_completeness": "partial", **cli_coverage_report_fields(False), "selected_scenario_count": len(selected), "test_inventory_sha256": inventory_digest(inventory), "phases": phase_reports, "artifacts_root": str(run_root)}
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
