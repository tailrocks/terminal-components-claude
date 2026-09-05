#!/usr/bin/env python3
"""Approve parity evidence only after every frozen contract is proven."""

from __future__ import annotations

import hashlib
import os
import re
import stat
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "baseline" / "before" / "manifest.tsv"
MAPPING = ROOT / "parity" / "recipes.tsv"
EVIDENCE = ROOT / "parity" / "evidence.tsv"
VISUAL_REVIEW = ROOT / "parity" / "visual_review.tsv"

EXPECTED_RECIPE_COUNT = 499
ARTIFACTS = ("txt", "ansi", "cursor", "html", "png")
COMPARED_ARTIFACTS = ("txt", "ansi", "cursor")
MAPPING_HEADER = (
    "recipe_id\tapp\tsurface\tviewport\tcolor\ttheme\tinitial_state\t"
    "historical_command\tsteps\tcurrent_argv\texpected_txt\texpected_ansi\t"
    "expected_cursor\texpected_html\texpected_png\tcurrent_artifact_dir\t"
    "provenance_path\ttrace_path\towner\treplay_policy"
)
EVIDENCE_HEADER = (
    "recipe_id\treplay_status\tcurrent_revision\tsource_fingerprint\tdirty\t"
    "artifact_dir\tprovenance_path\ttrace_path\tvisual_review\treviewer"
)
VISUAL_REVIEW_HEADER = (
    "recipe_id\tpng_path\tpng_sha256\thtml_path\thtml_sha256\treviewer\tdecision"
)
RECIPE_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]*$")
SAFE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")
SHA256 = re.compile(r"^[0-9a-fA-F]{64}$")
REVISION = re.compile(r"^[0-9a-fA-F]{40}$")


def fail(message: str) -> None:
    raise RuntimeError(message)


def _check_real_parents(path: Path, label: str) -> None:
    try:
        relative = path.relative_to(ROOT)
    except ValueError:
        return
    current = ROOT
    for component in relative.parts[:-1]:
        current /= component
        try:
            metadata = current.lstat()
        except FileNotFoundError:
            fail(f"missing {label} parent: {current}")
        except OSError as error:
            fail(f"cannot inspect {label} parent {current}: {error}")
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
            fail(f"{label} parent is not a real directory: {current}")


def regular_bytes(path: Path, label: str) -> bytes:
    _check_real_parents(path, label)
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    except OSError as error:
        fail(f"cannot inspect {label} {path}: {error}")
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        fail(f"{label} is not a regular file: {path}")
    try:
        return path.read_bytes()
    except OSError as error:
        fail(f"cannot read {label} {path}: {error}")


def nonempty_regular_bytes(path: Path, label: str) -> bytes:
    data = regular_bytes(path, label)
    if not data:
        fail(f"empty {label}: {path}")
    return data


def real_directory(path: Path, label: str) -> None:
    _check_real_parents(path, label)
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    except OSError as error:
        fail(f"cannot inspect {label} {path}: {error}")
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
        fail(f"{label} is not a real directory: {path}")


def repo_path(relative: str, label: str) -> Path:
    candidate = Path(relative)
    if (
        not relative
        or candidate.is_absolute()
        or "\\" in relative
        or any(part in ("", ".", "..") for part in candidate.parts)
    ):
        fail(f"unsafe {label} path: {relative}")
    path = ROOT.joinpath(*candidate.parts)
    try:
        path.relative_to(ROOT)
    except ValueError:
        fail(f"unsafe {label} path: {relative}")
    return path


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest(path: Path) -> str:
    return digest_bytes(regular_bytes(path, "artifact"))


def read_text(path: Path, label: str) -> str:
    data = regular_bytes(path, label)
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        fail(f"{label} is not UTF-8: {path}: {error}")


def read_tsv(path: Path, header: str, label: str) -> list[dict[str, str]]:
    text = read_text(path, label)
    columns = header.split("\t")
    lines = text.splitlines()
    if not lines or lines[0] != header:
        fail(f"{path} has an invalid header; expected {header}")
    rows: list[dict[str, str]] = []
    for line_number, line in enumerate(lines[1:], start=2):
        if not line:
            fail(f"{path}:{line_number}: blank row")
        fields = line.split("\t")
        if len(fields) != len(columns):
            fail(f"{path}:{line_number}: expected {len(columns)} fields, got {len(fields)}")
        rows.append(dict(zip(columns, fields, strict=True)))
    return rows


def load_manifest_ids() -> set[str]:
    text = read_text(MANIFEST, "historical manifest")
    lines = text.splitlines()
    if len(lines) != EXPECTED_RECIPE_COUNT:
        fail(f"{MANIFEST} has {len(lines)} rows; expected {EXPECTED_RECIPE_COUNT}")
    ids: set[str] = set()
    for line_number, line in enumerate(lines, start=1):
        fields = line.split("\t")
        if len(fields) != 5:
            fail(f"{MANIFEST}:{line_number}: expected 5 fields, got {len(fields)}")
        recipe_id = fields[0]
        if not RECIPE_ID.fullmatch(recipe_id):
            fail(f"{MANIFEST}:{line_number}: unsafe recipe ID {recipe_id!r}")
        if recipe_id in ids:
            fail(f"{MANIFEST}:{line_number}: duplicate recipe {recipe_id}")
        ids.add(recipe_id)
    return ids


def require_exact_ids(label: str, ids: list[str], expected: set[str]) -> None:
    if len(ids) != EXPECTED_RECIPE_COUNT:
        fail(f"{label} has {len(ids)} rows; expected {EXPECTED_RECIPE_COUNT}")
    if len(set(ids)) != EXPECTED_RECIPE_COUNT:
        fail(f"{label} must contain exactly {EXPECTED_RECIPE_COUNT} unique recipe IDs")
    actual = set(ids)
    if actual != expected:
        missing = sorted(expected - actual)[:5]
        extra = sorted(actual - expected)[:5]
        fail(f"{label} IDs differ from frozen manifest; missing={missing} extra={extra}")


def load_mapping(expected_ids: set[str]) -> dict[str, dict[str, str]]:
    rows = read_tsv(MAPPING, MAPPING_HEADER, "parity mapping")
    ids = [row["recipe_id"] for row in rows]
    require_exact_ids("parity mapping", ids, expected_ids)
    mappings: dict[str, dict[str, str]] = {}
    for row in rows:
        recipe_id = row["recipe_id"]
        if not RECIPE_ID.fullmatch(recipe_id):
            fail(f"parity mapping has unsafe recipe ID {recipe_id!r}")
        mappings[recipe_id] = row
        expected_dir = f"parity/replays/{recipe_id}"
        if row["current_artifact_dir"] != expected_dir:
            fail(f"{recipe_id}: current artifact directory differs from mapping contract")
        if row["provenance_path"] != f"{expected_dir}/provenance.json":
            fail(f"{recipe_id}: provenance path differs from mapping contract")
        if row["trace_path"] != f"{expected_dir}/trace.json":
            fail(f"{recipe_id}: trace path differs from mapping contract")
        for extension in ARTIFACTS:
            expected = f"baseline/before/{recipe_id}.{extension}"
            if row[f"expected_{extension}"] != expected:
                fail(f"{recipe_id}: {extension} historical path differs from mapping contract")
    return mappings


def git_text(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip()
        fail(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def current_revision() -> str:
    revision = git_text("rev-parse", "--verify", "HEAD").strip()
    if not REVISION.fullmatch(revision):
        fail(f"current Git revision is not full: {revision}")
    return revision


def current_dirty() -> bool:
    return bool(git_text("status", "--porcelain", "--untracked-files=all"))


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


def source_fingerprint() -> str:
    result = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", "-z"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        fail(f"git ls-files failed: {detail}")
    paths: set[str] = set()
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        try:
            relative = raw.decode("utf-8")
        except UnicodeDecodeError as error:
            fail(f"source path is not UTF-8: {error}")
        if source_path_allowed(relative):
            paths.add(relative)

    fingerprint = hashlib.sha256()
    for relative in sorted(paths):
        path = repo_path(relative, "source file")
        data = regular_bytes(path, f"source file {relative}")
        fingerprint.update(relative.encode("utf-8"))
        fingerprint.update(b"\0")
        fingerprint.update(str(len(data)).encode("ascii"))
        fingerprint.update(b"\0")
        fingerprint.update(data)
    return fingerprint.hexdigest()


def validate_evidence(
    rows: list[dict[str, str]],
    expected_ids: set[str],
    mappings: dict[str, dict[str, str]],
    revision: str,
    source_digest: str,
    dirty: bool,
) -> None:
    require_exact_ids("parity evidence", [row["recipe_id"] for row in rows], expected_ids)
    for row in rows:
        recipe_id = row["recipe_id"]
        mapping = mappings.get(recipe_id)
        if mapping is None:
            fail(f"evidence references unknown recipe {recipe_id}")
        if row["replay_status"] != "ok":
            fail(f"{recipe_id}: replay status is not ok")
        if row["current_revision"] != revision:
            fail(f"{recipe_id}: evidence revision is not current HEAD")
        if not SHA256.fullmatch(row["source_fingerprint"]):
            fail(f"{recipe_id}: evidence source fingerprint is invalid")
        if row["source_fingerprint"] != source_digest:
            fail(f"{recipe_id}: evidence source fingerprint is stale")
        expected_dirty = str(dirty).lower()
        if row["dirty"] not in ("true", "false"):
            fail(f"{recipe_id}: evidence dirty flag is invalid")
        if row["dirty"] != expected_dirty:
            fail(f"{recipe_id}: evidence dirty binding is stale")
        if row["artifact_dir"] != mapping["current_artifact_dir"]:
            fail(f"{recipe_id}: evidence artifact path differs from mapping")
        if row["provenance_path"] != mapping["provenance_path"]:
            fail(f"{recipe_id}: evidence provenance path differs from mapping")
        if row["trace_path"] != mapping["trace_path"]:
            fail(f"{recipe_id}: evidence trace path differs from mapping")
        for relative, label in (
            (row["artifact_dir"], "evidence artifact directory"),
            (row["provenance_path"], "evidence provenance path"),
            (row["trace_path"], "evidence trace path"),
        ):
            repo_path(relative, label)
        if row["visual_review"] != "pending" or row["reviewer"] != "pending":
            fail(f"{recipe_id}: evidence is not fresh pending replay")


def validate_artifacts(mappings: dict[str, dict[str, str]]) -> None:
    for recipe_id in sorted(mappings):
        mapping = mappings[recipe_id]
        directory = repo_path(mapping["current_artifact_dir"], f"{recipe_id} artifact directory")
        real_directory(directory, f"{recipe_id} artifact directory")
        for extension in ARTIFACTS:
            current_path = repo_path(
                f"{mapping['current_artifact_dir']}/{extension}",
                f"{recipe_id} current {extension}",
            )
            historical_path = repo_path(
                mapping[f"expected_{extension}"],
                f"{recipe_id} historical {extension}",
            )
            current = nonempty_regular_bytes(current_path, f"{recipe_id} current {extension}")
            historical = nonempty_regular_bytes(
                historical_path,
                f"{recipe_id} historical {extension}",
            )
            if extension in COMPARED_ARTIFACTS and current != historical:
                offset = next(
                    (index for index, (left, right) in enumerate(zip(historical, current)) if left != right),
                    min(len(historical), len(current)),
                )
                fail(
                    f"{recipe_id}: {extension} differs; byte_offset={offset} "
                    f"historical={digest_bytes(historical)} current={digest_bytes(current)}"
                )


def validate_visual_review(
    rows: list[dict[str, str]],
    expected_ids: set[str],
    mappings: dict[str, dict[str, str]],
    approval_reviewer: str,
) -> None:
    require_exact_ids("visual review", [row["recipe_id"] for row in rows], expected_ids)
    for row in rows:
        recipe_id = row["recipe_id"]
        mapping = mappings[recipe_id]
        expected_png = f"{mapping['current_artifact_dir']}/png"
        expected_html = f"{mapping['current_artifact_dir']}/html"
        if row["png_path"] != expected_png or row["html_path"] != expected_html:
            fail(f"{recipe_id}: visual review artifact path differs from mapping")
        if not SHA256.fullmatch(row["png_sha256"]):
            fail(f"{recipe_id}: visual review PNG SHA-256 is invalid")
        if not SHA256.fullmatch(row["html_sha256"]):
            fail(f"{recipe_id}: visual review HTML SHA-256 is invalid")
        png = nonempty_regular_bytes(
            repo_path(row["png_path"], f"{recipe_id} visual PNG"),
            f"{recipe_id} visual PNG",
        )
        html = nonempty_regular_bytes(
            repo_path(row["html_path"], f"{recipe_id} visual HTML"),
            f"{recipe_id} visual HTML",
        )
        if row["png_sha256"] != digest_bytes(png):
            fail(f"{recipe_id}: visual review PNG SHA-256 is stale")
        if row["html_sha256"] != digest_bytes(html):
            fail(f"{recipe_id}: visual review HTML SHA-256 is stale")
        if not SAFE.fullmatch(row["reviewer"]) or row["reviewer"] == "pending":
            fail(f"{recipe_id}: visual review reviewer is missing or unsafe")
        if row["reviewer"] == approval_reviewer:
            fail(f"{recipe_id}: visual review must be independent of approval")
        if row["decision"] != "approved":
            fail(f"{recipe_id}: visual review decision is not approved")


def atomic_write(path: Path, data: bytes, label: str) -> None:
    real_directory(path.parent, f"{label} parent")
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    except OSError as error:
        fail(f"cannot inspect {label} {path}: {error}")
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        fail(f"{label} is not a regular file: {path}")
    descriptor, temporary = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
    )
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


def main() -> int:
    if len(sys.argv) != 2 or not SAFE.fullmatch(sys.argv[1]):
        fail("usage: parity_approve.py REVIEWER (safe single token)")
    reviewer = sys.argv[1]

    manifest_ids = load_manifest_ids()
    mappings = load_mapping(manifest_ids)
    evidence_rows = read_tsv(EVIDENCE, EVIDENCE_HEADER, "parity replay evidence")
    revision = current_revision()
    source_digest = source_fingerprint()
    dirty = current_dirty()
    validate_evidence(evidence_rows, manifest_ids, mappings, revision, source_digest, dirty)
    validate_artifacts(mappings)
    review_rows = read_tsv(VISUAL_REVIEW, VISUAL_REVIEW_HEADER, "independent visual review")
    validate_visual_review(review_rows, manifest_ids, mappings, reviewer)

    for row in evidence_rows:
        row["visual_review"] = "approved"
        row["reviewer"] = reviewer
    fields = EVIDENCE_HEADER.split("\t")
    output = "\n".join(
        [EVIDENCE_HEADER] + ["\t".join(row[field] for field in fields) for row in evidence_rows]
    ) + "\n"
    atomic_write(EVIDENCE, output.encode("utf-8"), "parity replay evidence")
    print(f"parity approval: {len(evidence_rows)} exact replays approved by {reviewer}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, UnicodeError, ValueError, KeyError) as error:
        print(f"parity approval: {error}", file=sys.stderr)
        raise SystemExit(1)
