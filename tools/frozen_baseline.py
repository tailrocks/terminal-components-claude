#!/usr/bin/env python3
"""Build the frozen CLI oracle and run its PTY suite against paired binaries."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import signal
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


class RunnerError(RuntimeError):
    pass


def run(command: list[str], *, cwd: Path, timeout: int, env: dict[str, str] | None = None, quiet: bool = False, include_stderr: bool = False) -> str:
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
        raise RunnerError(f"command timed out after {timeout}s: {' '.join(command)}\n{output}\n{errors}") from error
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


def digest_source(path: Path) -> str:
    h = hashlib.sha256()
    for file in sorted(p for p in path.rglob("*") if p.is_file() and not {"target", "snapshots"}.intersection(p.relative_to(path).parts)):
        h.update(file.relative_to(path).as_posix().encode())
        h.update(b"\0")
        h.update(file.read_bytes())
        h.update(b"\0")
    return h.hexdigest()


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


def clean_env() -> dict[str, str]:
    blocked = {"BLESS", "COMBO_FILTER", "MIDTEST_BUILD", "PRINT_PAGES", "REACH_DUMP", "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET"}
    blocked_prefixes = ("VISUAL_", "UPDATE_", "NEXTEST_", "NEXTTEST_", "DUMP_", "SHOWCASE_", "INSTA_", "SNAPSHOT", "TUISNAP", "TUISCOTTI", "JUNIE_TUI_", "CARGO_PROFILE_")
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


def parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--repo", type=Path, default=Path.cwd())
    ap.add_argument("--root", type=Path, default=Path("target/frozen-baseline"))
    ap.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    sub = ap.add_subparsers(dest="command", required=True)
    sub.add_parser("prepare")
    sub.add_parser("build")
    candidate_build = sub.add_parser("build-candidate")
    candidate_build.add_argument("--target-dir", type=Path)
    pair = sub.add_parser("compare")
    pair.add_argument("--candidate-bin-dir", type=Path, required=True)
    selection = pair.add_mutually_exclusive_group(required=True)
    selection.add_argument("--case", help="exact ignored visual scenario test name")
    selection.add_argument("--all", action="store_true", help="run the complete frozen visual scenario suite")
    return ap


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            prepare(args.repo, args.root, args.timeout)
        elif args.command == "build":
            build(args.repo, args.root, args.timeout)
        elif args.command == "build-candidate":
            root = args.root.resolve()
            build_candidate(args.repo, root, args.target_dir or root / "candidate-target", args.timeout)
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
