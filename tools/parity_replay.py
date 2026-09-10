#!/usr/bin/env python3
"""Replay every frozen recipe through the real binary capture harness."""

from __future__ import annotations

import csv
import concurrent.futures
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MAPPING = ROOT / "parity" / "recipes.tsv"
FROZEN_MANIFEST = ROOT / "baseline" / "before" / "manifest.tsv"
REPLAYS = ROOT / "parity" / "replays"
EVIDENCE = ROOT / "parity" / "evidence.tsv"
CAPTURE = ROOT / "tools" / "capture.sh"
EXPECTED_RECIPE_COUNT = 499
HEADER = (
    "recipe_id\treplay_status\tcurrent_revision\tsource_fingerprint\tdirty\tartifact_dir\t"
    "provenance_path\ttrace_path\tvisual_review\treviewer"
)
MAPPING_HEADER = (
    "recipe_id\tapp\tsurface\tviewport\tcolor\ttheme\tinitial_state\t"
    "historical_command\tsteps\tcurrent_argv\texpected_txt\texpected_ansi\t"
    "expected_cursor\texpected_html\texpected_png\tcurrent_artifact_dir\t"
    "provenance_path\ttrace_path\towner\treplay_policy"
)
VISUAL_REVIEW = ROOT / "parity" / "visual_review.tsv"
VISUAL_REVIEW_HEADER = (
    "recipe_id\tpng_path\tpng_sha256\thtml_path\thtml_sha256\treviewer\tdecision"
)
SAFE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")


def fail(message: str) -> None:
    raise RuntimeError(message)


def regular_bytes(path: Path, label: str) -> bytes:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        fail(f"{label} is not a regular file: {path}")
    try:
        return path.read_bytes()
    except OSError as error:
        fail(f"cannot read {label} {path}: {error}")


def git_output(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )
    return result.stdout


def git_revision() -> str:
    revision = git_output("rev-parse", "--verify", "HEAD").strip()
    if not re.fullmatch(r"[0-9a-fA-F]{40}", revision):
        fail(f"current Git revision is not full: {revision}")
    return revision


def git_status_paths() -> list[str]:
    """Return every path named by porcelain status, including rename pairs."""
    output = git_output("status", "--porcelain=v1", "--untracked-files=all", "-z")
    records = output.split("\0")
    paths: list[str] = []
    index = 0
    while index < len(records):
        record = records[index]
        index += 1
        if not record:
            continue
        if len(record) < 4 or record[2] != " ":
            fail(f"invalid Git status record: {record!r}")
        paths.append(record[3:])
        if any(status in "RC" for status in record[:2]) and index < len(records):
            paths.append(records[index])
            index += 1
    return paths


def source_path_allowed(raw: str) -> bool:
    path = raw.replace("\\", "/")
    parts = path.split("/")
    if not path or path == ".DS_Store" or any(part == ".DS_Store" for part in parts):
        return False
    if any(part == "target" or part.startswith(".codex-target-") for part in parts):
        return False
    if path == "parity/recipes.tsv":
        return True
    if path == "parity" or path.startswith("parity/"):
        return False
    return True


def source_paths() -> list[str]:
    raw = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ).stdout
    paths = {
        value.decode("utf-8")
        for value in raw.split(b"\0")
        if value and source_path_allowed(value.decode("utf-8"))
    }
    return sorted(paths)


def source_fingerprint() -> str:
    digest = hashlib.sha256()
    for relative in source_paths():
        path = ROOT / relative
        data = regular_bytes(path, "source file")
        encoded_path = relative.encode("utf-8")
        digest.update(encoded_path)
        digest.update(b"\0")
        digest.update(str(len(data)).encode("ascii"))
        digest.update(b"\0")
        digest.update(data)
    return digest.hexdigest()


def source_dirty() -> bool:
    """Report changes that can affect replay, ignoring generated workspace files."""
    return any(source_path_allowed(path) for path in git_status_paths())


def remove_safely(path: Path, label: str) -> None:
    """Remove one explicit replay output without following symlinks."""
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode):
        fail(f"refusing symlink {label}: {path}")
    if stat.S_ISDIR(metadata.st_mode):
        for child in path.iterdir():
            remove_safely(child, label)
        try:
            path.rmdir()
        except OSError as error:
            fail(f"cannot remove {label} directory {path}: {error}")
        return
    if not stat.S_ISREG(metadata.st_mode):
        fail(f"refusing non-regular {label}: {path}")
    try:
        path.unlink()
    except OSError as error:
        fail(f"cannot remove {label} {path}: {error}")


def clear_replay_outputs() -> None:
    parity_dir = ROOT / "parity"
    if parity_dir.is_symlink() or not parity_dir.is_dir():
        fail(f"parity directory is missing or unsafe: {parity_dir}")
    if REPLAYS.exists() and (REPLAYS.is_symlink() or not REPLAYS.is_dir()):
        fail(f"replay output root is missing or unsafe: {REPLAYS}")
    remove_safely(REPLAYS, "stale replay output")
    try:
        REPLAYS.mkdir(mode=0o700)
    except FileExistsError:
        if REPLAYS.is_symlink() or not REPLAYS.is_dir():
            fail(f"replay output root is unsafe: {REPLAYS}")
    remove_safely(EVIDENCE, "stale replay evidence")
    for temporary in EVIDENCE.parent.glob(f".{EVIDENCE.name}.*.tmp"):
        remove_safely(temporary, "stale replay evidence temporary")


def atomic_write(path: Path, data: bytes, label: str) -> None:
    parent = path.parent
    if parent.is_symlink() or not parent.is_dir():
        fail(f"{label} parent is missing or unsafe: {parent}")
    if path.is_symlink():
        fail(f"{label} is a symlink: {path}")
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=parent)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            descriptor = -1
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        temporary = ""
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        if temporary:
            try:
                os.unlink(temporary)
            except FileNotFoundError:
                pass


def read_json(path: Path, label: str) -> object:
    try:
        return json.loads(regular_bytes(path, label))
    except json.JSONDecodeError as error:
        fail(f"{label} is not valid JSON: {path}: {error}")


def write_json(path: Path, value: object, label: str) -> bytes:
    data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    atomic_write(path, data, label)
    return data


def command(args: list[str], env: dict[str, str]) -> None:
    result = subprocess.run(
        ["/bin/bash", str(CAPTURE), *args],
        cwd=ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode:
        detail = " | ".join(part.strip() for part in (result.stdout, result.stderr) if part.strip())
        fail(f"capture {' '.join(args[:2])} failed ({result.returncode}): {detail}")


def snapshot(session: str) -> tuple[str, str, str]:
    def tmux(*args: str) -> str:
        result = subprocess.run(
            ["tmux", *args],
            cwd=ROOT,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        return result.stdout

    ansi = tmux("capture-pane", "-t", session, "-e", "-p", "-N")
    text = tmux("capture-pane", "-t", session, "-p", "-N")
    cursor = tmux("display", "-p", "-t", session, "#{cursor_x} #{cursor_y} #{cursor_flag}").strip()
    return ansi, text, cursor


def fingerprint(ansi: str, text: str, cursor: str) -> str:
    payload = json.dumps([ansi, text, cursor], ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def trace_hash(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def augment_provenance(
    path: Path,
    recipe_id: str,
    revision: str,
    source_digest: str,
    source_is_dirty: bool,
    trace_path: str,
    trace_digest: str,
) -> bool:
    value = read_json(path, "replay provenance")
    if isinstance(value, list):
        records = value
        matches = [record for record in records if isinstance(record, dict) and record.get("name") == recipe_id]
        if len(matches) != 1:
            fail(f"{recipe_id}: provenance has {len(matches)} matching records")
        record = matches[0]
    elif isinstance(value, dict):
        records = None
        record = value
    else:
        fail(f"{recipe_id}: provenance is not an object or array")
    git = record.get("git")
    if not isinstance(git, dict):
        fail(f"{recipe_id}: capture provenance git binding is invalid")
    if record.get("revision") != revision or git.get("revision") != revision:
        fail(f"{recipe_id}: capture provenance revision is stale")
    if not isinstance(git.get("dirty"), bool):
        fail(f"{recipe_id}: capture provenance dirty flag is invalid")
    # capture.sh observes the worktree when each row starts. Replay outputs
    # make that raw observation change after the first row. Bind both
    # validator-facing dirty fields to the one pre-replay source snapshot.
    record["dirty"] = source_is_dirty
    git["dirty"] = source_is_dirty
    record["parity"] = {
        "recipe_id": recipe_id,
        "revision": revision,
        "source_fingerprint": source_digest,
        "trace_path": trace_path,
        "trace_sha256": trace_digest,
    }
    write_json(path, value, "replay provenance")
    return source_is_dirty


def parse_steps(raw: str) -> list[tuple[str, object]]:
    if raw == "(none)":
        return []
    parts: list[str] = []
    start = 0
    depth = 0
    quoted = False
    escaped = False
    separator = " · "
    for index, char in enumerate(raw):
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
            continue
        if char == '"':
            quoted = True
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        if depth == 0 and raw.startswith(separator, index):
            parts.append(raw[start:index].strip())
            start = index + len(separator)
    parts.append(raw[start:].strip())

    steps: list[tuple[str, object]] = []
    for part in parts:
        if part.startswith("keys(") and part.endswith(")"):
            steps.append(("keys", part[5:-1].split()))
        elif part.startswith('type("') and part.endswith('")'):
            steps.append(("type", json.loads(part[5:-1])))
        elif part.startswith("mouse(") and part.endswith(")"):
            kind, coordinate = part[6:-1].split()
            x, y = coordinate.split(",")
            steps.append(("mouse", (kind, int(x), int(y))))
        elif part.startswith("resize(") and part.endswith(")"):
            width, height = part[7:-1].split("x")
            steps.append(("resize", (int(width), int(height))))
        elif part.startswith("wait(") and part.endswith("s)"):
            steps.append(("wait", part[5:-2]))
        elif part.startswith('(on "') and part.endswith('")'):
            steps.append(("anchor", part[5:-2]))
        else:
            fail(f"unrecognized recipe step {part!r}")
    return steps


def expand_keys(keys: list[str]) -> list[str]:
    expanded: list[str] = []
    previous: str | None = None
    for key in keys:
        match = re.fullmatch(r"x([0-9]+)", key)
        if match:
            if previous is None:
                fail(f"key repeat has no previous key: {keys!r}")
            expanded.extend([previous] * int(match.group(1)))
        else:
            expanded.append(key)
            previous = key
    return expanded


def run_recipe(
    row: dict[str, str],
    index: int,
    revision: str,
    source_digest: str,
    source_is_dirty: bool,
    state_root: Path,
    attempt: int = 0,
) -> bool:
    recipe_id = row["recipe_id"]
    if not SAFE.fullmatch(recipe_id):
        fail(f"unsafe recipe id {recipe_id!r}")
    width, height = (int(value) for value in row["viewport"].split("x"))
    argv = json.loads(row["current_argv"])
    if not isinstance(argv, list) or not argv or not all(isinstance(value, str) for value in argv):
        fail(f"invalid current argv for {recipe_id}")
    color = row["color"]
    run_id = f"parity-{index:03d}" if attempt == 0 else f"parity-{index:03d}-retry{attempt}"
    # Each worker owns a private capture-state directory.  capture.sh keeps
    # one lock per state root, while tmux sessions and artifact paths are
    # already unique per recipe; sharing that lock would serialize the replay.
    recipe_state_root = state_root / f"{index:03d}-attempt{attempt}"
    recipe_state_root.mkdir(parents=True, exist_ok=True)
    artifact_dir = row["current_artifact_dir"]
    manifest = f"{artifact_dir}/provenance.json"
    env = os.environ.copy()
    env.update(
        {
            "BIN": argv[0],
            "COLOR": color,
            # The Homebrew interpreter may lack Pillow while the trusted
            # system interpreter used by capture.sh already provides it.
            "PY": "/usr/bin/python3",
            "CAPTURE_DIR": "parity/replays",
            "CAPTURE_MANIFEST": manifest,
            "CAPTURE_STATE_DIR": str(recipe_state_root),
            "CAPTURE_RUN_ID": run_id,
        }
    )
    session_workspace = "".join(character if character.isalnum() else "_" for character in str(ROOT.resolve())) + "_"
    session = f"junie_cap_{session_workspace}_{run_id}"
    trace_steps: list[dict[str, object]] = []
    previous_text = ""
    primary_error: BaseException | None = None
    try:
        command(["start", str(width), str(height), "--", *argv], env)
        ansi, text, cursor = snapshot(session)
        trace_steps.append(
            {
                "index": 0,
                "event": "initial",
                "state_sha256": fingerprint(ansi, text, cursor),
            }
        )
        for step_index, (step_kind, value) in enumerate(parse_steps(row["steps"]), start=1):
            if step_kind != "anchor":
                previous_text = text
            if step_kind == "keys":
                keys = expand_keys(value)  # type: ignore[arg-type]
                command(["keys", *keys], env)
                event = f"keys:{' '.join(keys)}"
            elif step_kind == "type":
                command(["type", value], env)  # type: ignore[list-item]
                event = f"type:{value}"
            elif step_kind == "mouse":
                kind, x, y = value  # type: ignore[misc]
                command(["mouse", str(x), str(y), kind], env)
                event = f"mouse:{kind}:{x},{y}"
            elif step_kind == "resize":
                next_width, next_height = value  # type: ignore[misc]
                command(["resize", str(next_width), str(next_height)], env)
                event = f"resize:{next_width}x{next_height}"
            elif step_kind == "wait":
                command(["wait", value], env)  # type: ignore[list-item]
                event = f"wait:{value}s"
            elif step_kind == "anchor":
                # A historical anchor can name content visible immediately
                # before a scroll/click changes the final frame. The final
                # artifact is still compared exactly; this guard only proves
                # the recipe reached either side of that transition.
                if value not in text and value not in previous_text:
                    fail(f"{recipe_id}: anchor not observed: {value!r}")
                event = f"anchor:{value}"
            else:
                fail(f"{recipe_id}: unsupported replay step {step_kind}")
            if step_kind != "anchor":
                ansi, text, cursor = snapshot(session)
            else:
                ansi, text, cursor = snapshot(session)
            trace_steps.append(
                {
                    "index": len(trace_steps),
                    "step_index": step_index - 1,
                    "kind": step_kind,
                    "event": event,
                    "observed": True,
                    **({"anchor": value} if step_kind == "anchor" else {}),
                    "state_sha256": fingerprint(ansi, text, cursor),
                }
            )
        command(["shot", recipe_id], env)
    except BaseException as error:
        primary_error = error
        raise
    finally:
        try:
            command(["stop"], env)
        except RuntimeError:
            if primary_error is not None:
                # Preserve the capture/anchor failure. A missing session is
                # cleanup fallout, not the cause and must not hide evidence.
                pass
            elif (ROOT / artifact_dir).is_dir():
                raise
            else:
                raise

    trace = {
        "schema_version": 2,
        "recipe_id": recipe_id,
        "revision": revision,
        "source_fingerprint": source_digest,
        "steps": trace_steps,
    }
    trace_path = ROOT / row["trace_path"]
    trace_data = (json.dumps(trace, indent=2) + "\n").encode("utf-8")
    atomic_write(trace_path, trace_data, "replay trace")
    provenance_path = ROOT / row["provenance_path"]
    return augment_provenance(
        provenance_path,
        recipe_id,
        revision,
        source_digest,
        source_is_dirty,
        row["trace_path"],
        trace_hash(trace_data),
    )


def main() -> int:
    with MAPPING.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        if reader.fieldnames != MAPPING_HEADER.split("\t"):
            fail(f"{MAPPING} has an invalid header")
        rows = list(reader)
    if len(rows) != EXPECTED_RECIPE_COUNT:
        fail(f"{MAPPING} has {len(rows)} rows; expected {EXPECTED_RECIPE_COUNT}")
    ids = [row.get("recipe_id", "") for row in rows]
    if any(not SAFE.fullmatch(recipe_id) for recipe_id in ids) or len(set(ids)) != EXPECTED_RECIPE_COUNT:
        fail("parity mapping must contain exactly 499 unique safe recipe IDs")
    revision = git_revision()
    source_digest = source_fingerprint()
    source_is_dirty = source_dirty()
    clear_replay_outputs()
    dirty_by_recipe: dict[str, bool] = {}
    with tempfile.TemporaryDirectory(prefix="junie-parity-") as state:
        state_root = Path(state) / "state"
        state_root.mkdir()
        workers_raw = os.environ.get("PARITY_REPLAY_WORKERS", "1")
        try:
            workers = int(workers_raw)
        except ValueError:
            fail(f"PARITY_REPLAY_WORKERS must be a positive integer: {workers_raw!r}")
        if workers < 1:
            fail(f"PARITY_REPLAY_WORKERS must be a positive integer: {workers_raw!r}")

        def replay_one(item: tuple[int, dict[str, str]]) -> tuple[str, bool]:
            index, row = item
            print(f"replay {index}/{len(rows)}: {row['recipe_id']}", flush=True)
            for attempt in range(3):
                try:
                    dirty = run_recipe(
                        row,
                        index,
                        revision,
                        source_digest,
                        source_is_dirty,
                        state_root,
                        attempt,
                    )
                    return row["recipe_id"], dirty
                except RuntimeError:
                    if attempt == 2:
                        raise
                    print(
                        f"replay retry {index}/{len(rows)}: {row['recipe_id']} (attempt {attempt + 2})",
                        flush=True,
                    )
            raise AssertionError("unreachable replay retry loop")

        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            futures = [pool.submit(replay_one, item) for item in enumerate(rows, start=1)]
            try:
                for future in concurrent.futures.as_completed(futures):
                    recipe_id, dirty = future.result()
                    dirty_by_recipe[recipe_id] = dirty
            except BaseException:
                # The serial loop stopped at the first failure; restore that.
                # Cancel every queued recipe and stop accepting new work
                # without waiting here (in-flight recipes still run their own
                # capture cleanup); the `with` block then joins the workers
                # before the original error propagates.
                pool.shutdown(wait=False, cancel_futures=True)
                raise

    evidence = [HEADER]
    for row in rows:
        evidence.append(
            "\t".join(
                [
                    row["recipe_id"],
                    "ok",
                    revision,
                    source_digest,
                    str(dirty_by_recipe[row["recipe_id"]]).lower(),
                    row["current_artifact_dir"],
                    row["provenance_path"],
                    row["trace_path"],
                    "pending",
                    "pending",
                ]
            )
        )
    (ROOT / "parity" / "evidence.tsv").write_text("\n".join(evidence) + "\n", encoding="utf-8")
    print(f"replay complete: {len(rows)} recipes; visual review pending")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError, json.JSONDecodeError, ValueError) as error:
        print(f"parity replay: {error}", file=sys.stderr)
        raise SystemExit(1)
