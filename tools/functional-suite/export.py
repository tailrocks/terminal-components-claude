import argparse
import hashlib
import json
import os
import pathlib
import re
import stat
import subprocess
import sys
import tempfile


SCHEMA = "termrock-suite-export.v1"
PROVENANCE_SCHEMA = "termrock-suite-export-provenance.v1"
SOURCE_SUBTREE = "tools/functional-suite"
MANIFEST_NAME = "export-manifest.json"
ENGINE_RELATIVE = ".engine/tui-test"
BUNDLE_RELATIVE = "engine/tui-test.bundle"
PIN_RELATIVE = "engine/pin.json"
LOCK_RELATIVE = "Cargo.lock"
APPROVAL_LOCK = "expected/.tuiscotti-approval.lock"
ENGINE_BASE_COMMIT = "e15339fd136b5454849e1107bab142c98416b174"
ENGINE_BASE_TREE = "df8bb26a9f4de5058f89c9361fe58f42f79025ec"
ENGINE_COMMIT = "438563a86a3c5d78e6da6e12511fa089c6f15c61"
ENGINE_TREE = "d9957241869fcea938de9baa719de5f04d545453"
ENGINE_BUNDLE_SHA256 = "246c93b389824a85a212cceb37e584c866b6615bd4f69fece6f241320cd26cca"
ENGINE_PATCH_SHA256 = "9a7ed376d18c98791aa1e6a2052ce9b022d2c7aff35450b5c665dce0b6841b23"
TUISCOTTI_COMMIT = "f40513120727f83c0c9713ae8d04eac70dfc955e"
SUITE_LOCK_SHA256 = "66ffb4a40092bcf2d6e70f3342bfb3982f7c081c7633f3b042007f0bf5d73e90"
LEGACY_IMPORT = {
    "commit": "bb055bfcdb1ac37e9855e700497fd955b4a32f4b",
    "tree": "f3434d7c30b9dd3108f25dde6507f5523f51d1f6",
    "scenario_commit": "51a37ff4a0dbd467f2944ae486b23eb2592592b8",
}
PIN = {
    "schema": "termrock-live-suite-engine-pin.v1",
    "engine_base_commit": ENGINE_BASE_COMMIT,
    "engine_base_tree": ENGINE_BASE_TREE,
    "engine_commit": ENGINE_COMMIT,
    "engine_tree": ENGINE_TREE,
    "bundle_sha256": ENGINE_BUNDLE_SHA256,
    "patch_sha256": ENGINE_PATCH_SHA256,
    "tuiscotti_commit": TUISCOTTI_COMMIT,
    "suite_lock_sha256": SUITE_LOCK_SHA256,
}
CASE_DIRECTORIES = (
    "data/first/showcase",
    "data/first/holla",
    "data/first/jackin",
    "data/first/tablepro",
    "data/showcase-help/80x24-v1",
    "data/holla-first-use/action-identity-v1",
)
LOWER_HEX_40 = re.compile(r"^[0-9a-f]{40}$")
LOWER_HEX_64 = re.compile(r"^[0-9a-f]{64}$")
GIT_ENV = {
    "GIT_ALLOW_PROTOCOL": "file",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_OPTIONAL_LOCKS": "0",
    "GIT_TERMINAL_PROMPT": "0",
    "PATH": os.environ.get("PATH", os.defpath),
}


def fail(message):
    raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def strict_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def load_json(data, label):
    try:
        return json.loads(data.decode("utf-8"), object_pairs_hook=strict_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"{label} is not valid UTF-8 JSON: {error}")


def require_keys(value, expected, label):
    if not isinstance(value, dict):
        fail(f"{label} must be an object")
    actual = set(value)
    expected = set(expected)
    if actual != expected:
        fail(f"{label} keys differ: missing={sorted(expected - actual)}, unknown={sorted(actual - expected)}")


def require_hex(value, pattern, label):
    if not isinstance(value, str) or pattern.fullmatch(value) is None:
        fail(f"{label} must be lowercase hexadecimal")


def validate_absolute(value, label):
    if not isinstance(value, str) or not value.startswith("/") or "\x00" in value:
        fail(f"{label} must be an absolute path")
    if any(ord(char) < 32 or ord(char) == 127 for char in value):
        fail(f"{label} contains a control character")
    components = value.split("/")
    if components[0] != "" or any(part in ("", ".", "..") for part in components[1:]):
        fail(f"{label} has an empty, dot, or parent path component")
    return pathlib.Path(value)


def validate_relative(value, label):
    if not isinstance(value, str) or not value or value.startswith("/") or "\\" in value:
        fail(f"{label} must be a relative POSIX path")
    components = value.split("/")
    if any(part in ("", ".", "..") for part in components):
        fail(f"{label} has an empty, dot, or parent path component")
    if any(any(ord(char) < 32 or ord(char) == 127 for char in part) for part in components):
        fail(f"{label} contains a control character")
    return components


def directory_flags():
    return os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)


def open_absolute_directory(path, label, allow_missing=False):
    path = validate_absolute(str(path), label)
    descriptor = os.open("/", directory_flags())
    components = path.parts[1:]
    try:
        for index, component in enumerate(components):
            try:
                info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            except FileNotFoundError:
                if allow_missing:
                    os.close(descriptor)
                    return path, None
                fail(f"{label} has a missing path component")
            if stat.S_ISLNK(info.st_mode):
                fail(f"{label} contains a symlink component")
            if not stat.S_ISDIR(info.st_mode):
                fail(f"{label} contains a non-directory component")
            next_descriptor = os.open(component, directory_flags(), dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        info = os.fstat(descriptor)
        return path, (info.st_dev, info.st_ino)
    finally:
        try:
            os.close(descriptor)
        except OSError:
            pass


def inspect_directory(value, label):
    path, identity = open_absolute_directory(value, label)
    return path, identity


def inspect_new_directory(value, label):
    path, identity = open_absolute_directory(value, label, allow_missing=True)
    if identity is not None:
        fail(f"{label} already exists")
    return path


def secure_mkdir_absolute(value, label):
    path = validate_absolute(str(value), label)
    descriptor = os.open("/", directory_flags())
    try:
        for component in path.parts[1:]:
            try:
                os.mkdir(component, 0o755, dir_fd=descriptor)
            except FileExistsError:
                pass
            info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            if not stat.S_ISDIR(info.st_mode):
                fail(f"{label} has a non-directory component")
            next_descriptor = os.open(component, directory_flags(), dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except Exception:
        os.close(descriptor)
        raise


def secure_mkdir_relative(root_descriptor, relative):
    components = validate_relative(relative, "directory path")
    descriptor = os.dup(root_descriptor)
    try:
        for component in components:
            try:
                os.mkdir(component, 0o755, dir_fd=descriptor)
            except FileExistsError:
                pass
            info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            if not stat.S_ISDIR(info.st_mode):
                fail(f"directory path is not a regular directory: {relative}")
            next_descriptor = os.open(component, directory_flags(), dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except Exception:
        os.close(descriptor)
        raise


def secure_write_relative(root_descriptor, relative, data, mode):
    components = validate_relative(relative, "file path")
    descriptor = os.dup(root_descriptor)
    try:
        for component in components[:-1]:
            next_descriptor = os.open(component, directory_flags(), dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
        file_descriptor = os.open(components[-1], flags, mode, dir_fd=descriptor)
        try:
            view = memoryview(data)
            while view:
                written = os.write(file_descriptor, view)
                if written <= 0:
                    fail(f"short write for {relative}")
                view = view[written:]
            os.fchmod(file_descriptor, mode)
        finally:
            os.close(file_descriptor)
    finally:
        os.close(descriptor)


def safe_read_relative(root, relative, label):
    components = validate_relative(relative, label)
    root_path, _ = inspect_directory(root, "export root")
    descriptor = os.open(root_path, directory_flags())
    try:
        for component in components[:-1]:
            next_descriptor = os.open(component, directory_flags(), dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        file_descriptor = os.open(
            components[-1], os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0), dir_fd=descriptor
        )
        try:
            info = os.fstat(file_descriptor)
            if not stat.S_ISREG(info.st_mode):
                fail(f"{label} is not a regular file")
            chunks = []
            while True:
                chunk = os.read(file_descriptor, 1024 * 1024)
                if not chunk:
                    break
                chunks.append(chunk)
            return b"".join(chunks), info
        finally:
            os.close(file_descriptor)
    finally:
        os.close(descriptor)


def is_ancestor(left, right):
    left_parts = left.parts
    right_parts = right.parts
    return len(left_parts) <= len(right_parts) and right_parts[: len(left_parts)] == left_parts


def overlaps(left, right, left_identity=None, right_identity=None):
    return (
        is_ancestor(left, right)
        or is_ancestor(right, left)
        or (left_identity is not None and left_identity == right_identity)
    )


def require_disjoint(left, left_label, right, right_label, left_identity=None, right_identity=None):
    if overlaps(left, right, left_identity, right_identity):
        fail(f"{left_label} overlaps {right_label}")


def git_call(root, arguments, input_bytes=None):
    command = ["git", "-C", str(root), *arguments]
    result = subprocess.run(
        command,
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=GIT_ENV,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        fail(f"git {arguments!r} failed in {root}: {detail}")
    return result.stdout


def git_text(root, arguments):
    try:
        return git_call(root, arguments).decode("utf-8").strip()
    except UnicodeDecodeError as error:
        fail(f"git {arguments!r} returned non-UTF-8 output: {error}")


def git_worktree_root(path, label):
    actual = pathlib.Path(git_text(path, ["rev-parse", "--show-toplevel"]))
    actual, _ = inspect_directory(str(actual), f"{label} Git root")
    if actual != path:
        fail(f"{label} must be the Git worktree root")


def aggregate_digest(files):
    payload = json.dumps(files, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return sha256(payload)


def data_and_reference_digests(files):
    data_files = [entry for entry in files if entry["path"].startswith("data/")]
    reference_files = [
        entry
        for entry in files
        if entry["path"].startswith("expected/") and entry["path"].endswith(".tui.json")
    ]
    if len(reference_files) != 25:
        fail(f"expected 25 frozen references, found {len(reference_files)}")
    return aggregate_digest(data_files), aggregate_digest(reference_files)


def implied_directories(files):
    directories = set()
    for entry in files:
        components = validate_relative(entry["path"], "file path")
        for index in range(1, len(components)):
            directories.add("/".join(components[:index]))
    for case_directory in CASE_DIRECTORIES:
        components = validate_relative(case_directory, "case working directory")
        for index in range(1, len(components) + 1):
            directories.add("/".join(components[:index]))
    return sorted(directories)


def parse_tree_listing(output, prefix):
    files = []
    for record in output.split(b"\0"):
        if not record:
            continue
        try:
            metadata, raw_path = record.split(b"\t", 1)
            mode_bytes, kind_bytes, object_bytes = metadata.split(b" ", 2)
            path = raw_path.decode("utf-8")
            mode = mode_bytes.decode("ascii")
            kind = kind_bytes.decode("ascii")
            object_id = object_bytes.decode("ascii")
        except (ValueError, UnicodeDecodeError) as error:
            fail(f"Git tree listing is malformed: {error}")
        if not path.startswith(prefix):
            fail(f"Git tree entry is outside {prefix}: {path}")
        relative = path[len(prefix) :]
        validate_relative(relative, "Git tree path")
        if relative == APPROVAL_LOCK or relative == ".git" or relative.startswith(".git/") or relative == "target" or relative.startswith("target/") or relative == ".engine" or relative.startswith(".engine/"):
            fail(f"Git tree contains a reserved export path: {relative}")
        if kind != "blob" or mode not in ("100644", "100755") or not LOWER_HEX_40.fullmatch(object_id):
            fail(f"Git tree entry is not a regular file: {relative}")
        files.append((relative, mode, object_id))
    files.sort(key=lambda item: item[0])
    if len({item[0] for item in files}) != len(files):
        fail("Git tree has duplicate suite paths")
    return files


def read_committed_suite(source_root, commit):
    git_worktree_root(source_root, "source root")
    actual_commit = git_text(source_root, ["rev-parse", "HEAD"])
    actual_tree = git_text(source_root, ["rev-parse", "HEAD^{tree}"])
    status = git_call(source_root, ["status", "--porcelain=v1", "--untracked-files=all"])
    if actual_commit != commit or status:
        fail(f"source worktree is not clean at the requested commit: HEAD={actual_commit}")
    suite_tree = git_text(source_root, ["rev-parse", f"{commit}:{SOURCE_SUBTREE}"])
    if not LOWER_HEX_40.fullmatch(suite_tree):
        fail("canonical suite subtree object ID is invalid")
    listing = git_call(source_root, ["ls-tree", "-r", "-z", "--full-tree", commit, "--", SOURCE_SUBTREE])
    tree_files = parse_tree_listing(listing, f"{SOURCE_SUBTREE}/")
    if not tree_files:
        fail("canonical suite subtree is empty")
    files = []
    for relative, mode, object_id in tree_files:
        data = git_call(source_root, ["cat-file", "blob", object_id])
        files.append(
            {
                "path": relative,
                "mode": mode,
                "size": len(data),
                "sha256": sha256(data),
                "data": data,
            }
        )
    file_entries = [{key: entry[key] for key in ("path", "mode", "size", "sha256")} for entry in files]
    by_path = {entry["path"]: entry for entry in files}
    if by_path.get(PIN_RELATIVE) is None or load_json(by_path[PIN_RELATIVE]["data"], PIN_RELATIVE) != PIN:
        fail("canonical engine pin differs from the approved pin")
    if by_path.get(LOCK_RELATIVE) is None or sha256(by_path[LOCK_RELATIVE]["data"]) != SUITE_LOCK_SHA256:
        fail("suite lock differs from the approved lock")
    if by_path.get(BUNDLE_RELATIVE) is None or sha256(by_path[BUNDLE_RELATIVE]["data"]) != ENGINE_BUNDLE_SHA256:
        fail("local engine bundle differs from the approved bundle")
    if by_path.get("export.py") is None:
        fail("canonical suite does not contain export.py")
    return actual_tree, suite_tree, file_entries, files, implied_directories(file_entries)


def boundary_records(source_root, application_roots, bin_roots, run_output_roots):
    source_path, source_id = inspect_directory(source_root, "source root")
    canonical_path, canonical_id = inspect_directory(source_path / SOURCE_SUBTREE, "canonical suite root")
    app_paths = []
    app_ids = []
    for index, value in enumerate(application_roots):
        label = "reference application root" if index == 0 else "candidate application root"
        path, identity = inspect_directory(value, label)
        git_worktree_root(path, label)
        app_paths.append(path)
        app_ids.append(identity)
    if len(app_paths) != 2 or app_paths[0] == app_paths[1] or app_ids[0] == app_ids[1]:
        fail("reference and candidate application roots must be distinct")
    bin_paths = []
    bin_ids = []
    for index, value in enumerate(bin_roots):
        label = "reference binary root" if index == 0 else "candidate binary root"
        path, identity = inspect_directory(value, label)
        bin_paths.append(path)
        bin_ids.append(identity)
    if len(bin_paths) != 2 or bin_paths[0] == bin_paths[1] or bin_ids[0] == bin_ids[1]:
        fail("reference and candidate binary roots must be distinct")
    run_paths = []
    run_ids = []
    for value in run_output_roots:
        path, identity = inspect_directory(value, "run-output root")
        run_paths.append(path)
        run_ids.append(identity)
    if not run_paths:
        fail("at least one run-output root is required")
    if len(set(run_paths)) != len(run_paths) or len(set(run_ids)) != len(run_ids):
        fail("run-output roots must be distinct")
    if canonical_path != source_path / SOURCE_SUBTREE:
        fail("canonical suite root must be inside source root")
    if not is_ancestor(source_path, canonical_path):
        fail("canonical suite root must be inside source root")
    if app_paths[0] == source_path or app_ids[0] == source_id:
        fail("reference application root cannot be the suite source worktree")
    if app_paths[1] != source_path and app_ids[1] != source_id:
        require_disjoint(source_path, "source root", app_paths[1], "candidate application root", source_id, app_ids[1])
    if app_paths[1] == source_path and app_ids[1] != source_id:
        fail("source and candidate application paths have different filesystem identities")
    protected = [
        (source_path, "source root", source_id),
        (canonical_path, "canonical suite root", canonical_id),
        (app_paths[0], "reference application root", app_ids[0]),
        (app_paths[1], "candidate application root", app_ids[1]),
    ]
    for index, (left, left_label, left_id) in enumerate(protected):
        for right, right_label, right_id in protected[index + 1 :]:
            if {left_label, right_label} == {"source root", "canonical suite root"}:
                continue
            if left == source_path and right == app_paths[1] and right == source_path:
                continue
            if right == source_path and left == app_paths[1] and left == source_path:
                continue
            if left == canonical_path and right == app_paths[1] and app_paths[1] == source_path:
                continue
            if right == canonical_path and left == app_paths[1] and app_paths[1] == source_path:
                continue
            require_disjoint(left, left_label, right, right_label, left_id, right_id)
    for path, label, identity in [*protected, *[(p, "binary root", i) for p, i in zip(bin_paths, bin_ids)]]:
        for bin_path, bin_id in zip(bin_paths, bin_ids):
            if label == "binary root" and path == bin_path:
                continue
            require_disjoint(path, label, bin_path, "binary root", identity, bin_id)
    for index, (left, left_id) in enumerate(zip(bin_paths, bin_ids)):
        for right, right_id in list(zip(bin_paths, bin_ids))[index + 1 :]:
            require_disjoint(left, "binary root", right, "binary root", left_id, right_id)
    all_protected = [*protected, *[(p, "binary root", i) for p, i in zip(bin_paths, bin_ids)]]
    for run_path, run_id in zip(run_paths, run_ids):
        for protected_path, label, protected_id in all_protected:
            require_disjoint(run_path, "run-output root", protected_path, label, run_id, protected_id)
    for index, (left, left_id) in enumerate(zip(run_paths, run_ids)):
        for right, right_id in list(zip(run_paths, run_ids))[index + 1 :]:
            require_disjoint(left, "run-output root", right, "run-output root", left_id, right_id)
    boundaries = {
        "source_root": str(source_path),
        "canonical_root": str(canonical_path),
        "reference_application_root": str(app_paths[0]),
        "candidate_application_root": str(app_paths[1]),
        "reference_bin_root": str(bin_paths[0]),
        "candidate_bin_root": str(bin_paths[1]),
        "run_output_roots": sorted(str(path) for path in run_paths),
    }
    identities = {
        str(source_path): source_id,
        str(canonical_path): canonical_id,
        **{str(path): identity for path, identity in zip(app_paths, app_ids)},
        **{str(path): identity for path, identity in zip(bin_paths, bin_ids)},
        **{str(path): identity for path, identity in zip(run_paths, run_ids)},
    }
    return boundaries, identities


def boundaries_from_manifest(value):
    require_keys(
        value,
        (
            "source_root",
            "canonical_root",
            "reference_application_root",
            "candidate_application_root",
            "reference_bin_root",
            "candidate_bin_root",
            "run_output_roots",
        ),
        "manifest boundaries",
    )
    paths = {}
    for key in (
        "source_root",
        "canonical_root",
        "reference_application_root",
        "candidate_application_root",
        "reference_bin_root",
        "candidate_bin_root",
    ):
        path, identity = inspect_directory(value[key], key.replace("_", " "))
        paths[key] = (path, identity)
        if key in ("reference_application_root", "candidate_application_root"):
            git_worktree_root(path, key.replace("_", " "))
    if not isinstance(value["run_output_roots"], list) or not value["run_output_roots"]:
        fail("manifest must list at least one run-output root")
    run_outputs = [inspect_directory(item, "run-output root") for item in value["run_output_roots"]]
    if value["run_output_roots"] != sorted(set(value["run_output_roots"])):
        fail("run-output roots must be unique and sorted")
    if str(paths["canonical_root"][0]) != str(paths["source_root"][0] / SOURCE_SUBTREE):
        fail("canonical suite root is outside source root")
    if str(paths["reference_application_root"][0]) == str(paths["source_root"][0]):
        fail("reference application root equals source root")
    source = paths["source_root"]
    canonical = paths["canonical_root"]
    candidate = paths["candidate_application_root"]
    if candidate[0] == source[0] and candidate[1] != source[1]:
        fail("source and candidate application paths have different filesystem identities")
    same_candidate_worktree = candidate[0] == source[0] and candidate[1] == source[1]
    protected = [
        ("source root", *source),
        ("canonical suite root", *canonical),
        ("reference application root", *paths["reference_application_root"]),
        ("candidate application root", *candidate),
    ]
    for index, (left_label, left_path, left_id) in enumerate(protected):
        for right_label, right_path, right_id in protected[index + 1 :]:
            pair = (left_label, right_label)
            allowed = pair == ("source root", "canonical suite root")
            allowed = allowed or (
                same_candidate_worktree
                and pair in (
                    ("source root", "candidate application root"),
                    ("canonical suite root", "candidate application root"),
                )
            )
            if not allowed:
                require_disjoint(
                    left_path,
                    left_label,
                    right_path,
                    right_label,
                    left_id,
                    right_id,
                )
    bin_roots = [
        ("reference binary root", *paths["reference_bin_root"]),
        ("candidate binary root", *paths["candidate_bin_root"]),
    ]
    all_roots = [*protected, *bin_roots]
    for index, (left_label, left_path, left_id) in enumerate(all_roots):
        for right_label, right_path, right_id in all_roots[index + 1 :]:
            if left_label.endswith("binary root") or right_label.endswith("binary root"):
                require_disjoint(
                    left_path,
                    left_label,
                    right_path,
                    right_label,
                    left_id,
                    right_id,
                )
    for run_path, run_id in run_outputs:
        for label, root_path, root_id in all_roots:
            require_disjoint(run_path, "run-output root", root_path, label, run_id, root_id)
    for index, (left, left_id) in enumerate(run_outputs):
        for right, right_id in run_outputs[index + 1 :]:
            require_disjoint(left, "run-output root", right, "run-output root", left_id, right_id)
    return paths, run_outputs


def validate_export_boundaries(root, boundaries):
    export_path, export_id = inspect_directory(root, "export root")
    paths, run_outputs = boundaries_from_manifest(boundaries)
    protected = [
        paths["source_root"],
        paths["canonical_root"],
        paths["reference_application_root"],
        paths["candidate_application_root"],
        paths["reference_bin_root"],
        paths["candidate_bin_root"],
    ]
    for path, identity in protected:
        require_disjoint(export_path, "export root", path, "protected root", export_id, identity)
    for path, identity in run_outputs:
        require_disjoint(export_path, "export root", path, "run-output root", export_id, identity)
    engine_path = export_path / ENGINE_RELATIVE
    for path, identity in [*protected, *[(path, identity) for path, identity in run_outputs]]:
        require_disjoint(engine_path, "engine root", path, "protected root", None, identity)
    bin_roots = [paths["reference_bin_root"], paths["candidate_bin_root"]]
    for bin_path, bin_identity in bin_roots:
        require_disjoint(engine_path, "engine root", bin_path, "binary root", None, bin_identity)
    return export_path, paths, run_outputs


def validate_selected_paths(root, boundaries, bin_value=None, output_value=None):
    paths, run_outputs = boundaries_from_manifest(boundaries)
    if bin_value is None and output_value is None:
        return
    if bin_value is not None:
        bin_path, bin_identity = inspect_directory(bin_value, "selected binary root")
        sealed_bins = [paths["reference_bin_root"], paths["candidate_bin_root"]]
        if not any(bin_path == path and bin_identity == identity for path, identity in sealed_bins):
            fail("selected --bin-dir is not a sealed binary root")
    if output_value is not None:
        output_path = validate_absolute(output_value, "selected run-output path")
        _, output_identity = open_absolute_directory(output_path, "selected run-output path", allow_missing=True)
        if output_identity is not None:
            fail("selected run-output path already exists")
        if not any(is_ancestor(run_root, output_path) and run_root != output_path for run_root, _ in run_outputs):
            fail("selected --run-out must be a strict descendant of a sealed run-output root")
        for key, (path, identity) in paths.items():
            require_disjoint(output_path, "selected run-output path", path, key.replace("_", " "), None, identity)
        for run_root, identity in run_outputs:
            if output_path != run_root and is_ancestor(run_root, output_path):
                continue
            require_disjoint(output_path, "selected run-output path", run_root, "run-output root", None, identity)


def source_tree_matches(boundaries, source_commit, source_tree, suite_tree):
    source_path, _ = inspect_directory(boundaries["source_root"], "source root")
    canonical_path, _ = inspect_directory(boundaries["canonical_root"], "canonical suite root")
    git_worktree_root(source_path, "source root")
    actual_commit = git_text(source_path, ["rev-parse", "HEAD"])
    actual_tree = git_text(source_path, ["rev-parse", "HEAD^{tree}"])
    status = git_call(source_path, ["status", "--porcelain=v1", "--untracked-files=all"])
    actual_suite_tree = git_text(source_path, ["rev-parse", f"{source_commit}:{SOURCE_SUBTREE}"])
    if actual_commit != source_commit or actual_tree != source_tree or status:
        fail(f"source worktree changed after export creation: HEAD={actual_commit}")
    if actual_suite_tree != suite_tree or canonical_path != source_path / SOURCE_SUBTREE:
        fail("source suite subtree changed after export creation")


def validate_manifest(manifest):
    require_keys(
        manifest,
        (
            "schema",
            "source_commit",
            "source_tree",
            "suite_subtree",
            "legacy_import",
            "pin",
            "boundaries",
            "files",
            "directories",
        ),
        "export manifest",
    )
    if manifest["schema"] != SCHEMA:
        fail("unsupported export manifest schema")
    require_hex(manifest["source_commit"], LOWER_HEX_40, "source commit")
    require_hex(manifest["source_tree"], LOWER_HEX_40, "source tree")
    require_hex(manifest["suite_subtree"], LOWER_HEX_40, "suite subtree")
    if manifest["legacy_import"] != LEGACY_IMPORT:
        fail("legacy import provenance differs from the approved input")
    if manifest["pin"] != PIN:
        fail("manifest pin differs from the approved pin")
    if not isinstance(manifest["files"], list) or not manifest["files"]:
        fail("manifest files must be a nonempty array")
    file_paths = []
    for entry in manifest["files"]:
        require_keys(entry, ("path", "mode", "size", "sha256"), "manifest file entry")
        validate_relative(entry["path"], "manifest file path")
        if entry["mode"] not in ("100644", "100755"):
            fail(f"invalid Git mode for {entry['path']}")
        if not isinstance(entry["size"], int) or isinstance(entry["size"], bool) or entry["size"] < 0:
            fail(f"invalid file size for {entry['path']}")
        require_hex(entry["sha256"], LOWER_HEX_64, f"file digest for {entry['path']}")
        file_paths.append(entry["path"])
    if file_paths != sorted(set(file_paths)):
        fail("manifest files must be unique and sorted")
    if not isinstance(manifest["directories"], list):
        fail("manifest directories must be an array")
    for directory in manifest["directories"]:
        validate_relative(directory, "manifest directory")
    if manifest["directories"] != implied_directories(manifest["files"]):
        fail("manifest directories do not match files and required case working directories")
    for relative in (PIN_RELATIVE, BUNDLE_RELATIVE, LOCK_RELATIVE, "export.py"):
        if relative not in file_paths:
            fail(f"manifest is missing required file {relative}")
    data_digest, reference_digest = data_and_reference_digests(manifest["files"])
    return data_digest, reference_digest


def collect_export_paths(root):
    file_paths = []
    directory_paths = []
    for current, directories, files in os.walk(root, topdown=True, followlinks=False):
        current_path = pathlib.Path(current)
        relative_dir = current_path.relative_to(root).as_posix()
        if relative_dir == ".":
            for generated in ("target", ".engine"):
                if generated in directories:
                    generated_path = current_path / generated
                    info = os.lstat(generated_path)
                    if not stat.S_ISDIR(info.st_mode):
                        fail(f"generated path is not a regular directory: {generated}")
                    directories.remove(generated)
            if ".git" in directories or ".git" in files:
                fail("Git metadata is not allowed in an export")
        for name in list(directories):
            path = current_path / name
            info = os.lstat(path)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                fail(f"export contains a symlink or special directory: {path}")
            relative = (path.relative_to(root)).as_posix()
            if relative == ".git" or relative.endswith("/.git"):
                fail("Git metadata is not allowed in an export")
            directory_paths.append(relative)
        for name in files:
            path = current_path / name
            relative = path.relative_to(root).as_posix()
            info = os.lstat(path)
            if relative == MANIFEST_NAME:
                if not stat.S_ISREG(info.st_mode):
                    fail("export manifest must be a regular file")
                continue
            if relative == APPROVAL_LOCK:
                if not stat.S_ISREG(info.st_mode) or info.st_size != 0:
                    fail("approval lock must be an empty regular file")
                continue
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
                fail(f"export contains a symlink or special file: {path}")
            file_paths.append(relative)
    return sorted(file_paths), sorted(directory_paths)


def verify_export_files(root, manifest):
    actual_files, actual_directories = collect_export_paths(root)
    expected_files = [entry["path"] for entry in manifest["files"]]
    if actual_files != expected_files:
        fail(f"export file inventory differs: missing={sorted(set(expected_files) - set(actual_files))}, extra={sorted(set(actual_files) - set(expected_files))}")
    if actual_directories != manifest["directories"]:
        fail(f"export directory inventory differs: missing={sorted(set(manifest['directories']) - set(actual_directories))}, extra={sorted(set(actual_directories) - set(manifest['directories']))}")
    for entry in manifest["files"]:
        data, info = safe_read_relative(root, entry["path"], entry["path"])
        permissions = stat.S_IMODE(info.st_mode)
        if permissions not in (0o644, 0o755):
            fail(f"sealed file has invalid permissions: {entry['path']}")
        mode = "100755" if permissions == 0o755 else "100644"
        if len(data) != entry["size"] or sha256(data) != entry["sha256"] or mode != entry["mode"]:
            fail(f"sealed file changed: {entry['path']}")
    pin_data, _ = safe_read_relative(root, PIN_RELATIVE, PIN_RELATIVE)
    if load_json(pin_data, PIN_RELATIVE) != PIN:
        fail("exported engine pin differs from the approved pin")
    lock_data, _ = safe_read_relative(root, LOCK_RELATIVE, LOCK_RELATIVE)
    if sha256(lock_data) != SUITE_LOCK_SHA256:
        fail("exported Cargo.lock differs from the approved lock")
    bundle_data, _ = safe_read_relative(root, BUNDLE_RELATIVE, BUNDLE_RELATIVE)
    if sha256(bundle_data) != ENGINE_BUNDLE_SHA256:
        fail("exported engine bundle differs from the approved bundle")


def read_engine_tree(engine_path):
    head = git_text(engine_path, ["rev-parse", "HEAD"])
    tree = git_text(engine_path, ["rev-parse", "HEAD^{tree}"])
    if head != ENGINE_COMMIT or tree != ENGINE_TREE:
        fail(f"engine identity differs: HEAD={head}, tree={tree}")
    if git_text(engine_path, ["rev-parse", "--show-toplevel"]) != str(engine_path):
        fail("engine Git root differs from its sealed path")
    if git_call(engine_path, ["status", "--porcelain=v1", "--untracked-files=all"]):
        fail("engine worktree is not clean")
    if git_text(engine_path, ["remote"]):
        fail("engine repository has a remote")
    git_dir = engine_path / ".git"
    info = os.lstat(git_dir)
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
        fail("engine Git metadata must be a local directory")
    for relative in ("objects/info/alternates", "objects/info/http-alternates"):
        if os.path.lexists(git_dir / relative):
            fail(f"engine repository has an alternate object store: {relative}")
    listing = git_call(engine_path, ["ls-tree", "-r", "-z", "--full-tree", "HEAD"])
    entries = []
    for record in listing.split(b"\0"):
        if not record:
            continue
        try:
            metadata, raw_path = record.split(b"\t", 1)
            mode_bytes, kind_bytes, object_bytes = metadata.split(b" ", 2)
            relative = raw_path.decode("utf-8")
            mode = mode_bytes.decode("ascii")
            kind = kind_bytes.decode("ascii")
            object_id = object_bytes.decode("ascii")
        except (ValueError, UnicodeDecodeError) as error:
            fail(f"engine Git tree is malformed: {error}")
        validate_relative(relative, "engine tree path")
        if kind != "blob" or mode not in ("100644", "100755") or not LOWER_HEX_40.fullmatch(object_id):
            fail(f"engine tree has a non-regular entry: {relative}")
        entries.append((relative, mode, object_id))
    entries.sort(key=lambda item: item[0])
    expected_files = [entry[0] for entry in entries]
    expected_directories = sorted(
        {
            "/".join(components[:index])
            for relative, _, _ in entries
            for components in [validate_relative(relative, "engine tree path")]
            for index in range(1, len(components))
        }
    )
    actual_files = []
    actual_directories = []
    for current, directories, files in os.walk(engine_path, topdown=True, followlinks=False):
        current_path = pathlib.Path(current)
        relative_dir = current_path.relative_to(engine_path).as_posix()
        if relative_dir == ".":
            if ".git" not in directories:
                fail("engine Git metadata is missing")
            directories.remove(".git")
        for name in list(directories):
            path = current_path / name
            info = os.lstat(path)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                fail(f"engine contains a symlink or special directory: {path}")
            actual_directories.append(path.relative_to(engine_path).as_posix())
        for name in files:
            path = current_path / name
            info = os.lstat(path)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
                fail(f"engine contains a symlink or special file: {path}")
            actual_files.append(path.relative_to(engine_path).as_posix())
    if sorted(actual_files) != expected_files or sorted(actual_directories) != expected_directories:
        fail("engine worktree has missing or extra files or directories")
    for relative, mode, object_id in entries:
        actual, info = safe_read_relative(engine_path, relative, f"engine file {relative}")
        expected = git_call(engine_path, ["cat-file", "blob", object_id])
        permissions = stat.S_IMODE(info.st_mode)
        if permissions not in (0o644, 0o755):
            fail(f"engine file has invalid permissions: {relative}")
        actual_mode = "100755" if permissions == 0o755 else "100644"
        if actual != expected or actual_mode != mode:
            fail(f"engine worktree differs from its Git tree: {relative}")
    return {"head": head, "tree": tree}


def create_engine(root, bundle_data):
    engine_parent = root / ".engine"
    engine_path = engine_parent / "tui-test"
    if os.path.lexists(engine_parent):
        parent_path, _ = inspect_directory(engine_parent, "engine parent")
        names = sorted(os.listdir(parent_path))
        if any(name != "tui-test" for name in names):
            fail(".engine contains an unexpected generated entry")
        if os.path.lexists(engine_path):
            verify_engine(engine_path, bundle_data)
            return engine_path
    else:
        root_descriptor = os.open(root, directory_flags())
        try:
            created = secure_mkdir_relative(root_descriptor, ".engine")
            os.close(created)
        finally:
            os.close(root_descriptor)
    if os.path.lexists(engine_path):
        verify_engine(engine_path, bundle_data)
        return engine_path
    with tempfile.TemporaryDirectory(prefix=".tui-test-staging-", dir=engine_parent) as temporary:
        temporary_path = pathlib.Path(temporary)
        staging = temporary_path / "repo"
        staging.mkdir(mode=0o700)
        template = temporary_path / "template"
        template.mkdir(mode=0o700)
        bundle_path = temporary_path / "tui-test.bundle"
        with open(bundle_path, "xb") as bundle_file:
            bundle_file.write(bundle_data)
            bundle_file.flush()
        os.chmod(bundle_path, 0o600)
        git_call(staging, ["init", "--quiet", f"--template={template}"])
        git_call(staging, ["config", "--local", "core.hooksPath", os.devnull])
        git_call(staging, ["bundle", "verify", str(bundle_path)])
        heads = git_call(staging, ["bundle", "list-heads", str(bundle_path)]).decode("utf-8").splitlines()
        expected_head = f"{ENGINE_COMMIT} refs/heads/common-raw-e153"
        if heads != [expected_head]:
            fail("engine bundle does not advertise the sole approved reference")
        git_call(
            staging,
            ["fetch", "--quiet", "--no-tags", "--no-recurse-submodules", str(bundle_path), "refs/heads/common-raw-e153"],
        )
        git_call(staging, ["checkout", "--quiet", "--detach", ENGINE_COMMIT])
        verify_engine(staging, bundle_data)
        if os.path.lexists(engine_path):
            fail("engine destination appeared during materialization")
        os.rename(staging, engine_path)
    verify_engine(engine_path, bundle_data)
    return engine_path


def verify_engine(engine_path, bundle_data):
    if sha256(bundle_data) != ENGINE_BUNDLE_SHA256:
        fail("engine bundle digest differs from the approved bundle")
    inspect_directory(engine_path, "materialized engine")
    identity = read_engine_tree(engine_path)
    return identity


def verify_generated_paths(root, bundle_data):
    engine_parent = root / ".engine"
    engine_path = engine_parent / "tui-test"
    if not os.path.lexists(engine_parent):
        create_engine(root, bundle_data)
        return verify_engine(engine_path, bundle_data)
    inspect_directory(engine_parent, "engine parent")
    entries = sorted(os.listdir(engine_parent))
    if any(name != "tui-test" for name in entries):
        fail(".engine contains an unexpected generated entry")
    if not os.path.lexists(engine_path):
        create_engine(root, bundle_data)
    return verify_engine(engine_path, bundle_data)


def verify_engine_boundaries(root, boundaries):
    engine_path, engine_identity = inspect_directory(root / ENGINE_RELATIVE, "engine root")
    paths, run_outputs = boundaries_from_manifest(boundaries)
    protected = [
        paths["source_root"],
        paths["canonical_root"],
        paths["reference_application_root"],
        paths["candidate_application_root"],
        paths["reference_bin_root"],
        paths["candidate_bin_root"],
    ]
    for path, identity in [*protected, *run_outputs]:
        require_disjoint(engine_path, "engine root", path, "protected root", engine_identity, identity)


def provenance(manifest, seal, data_digest, reference_digest, engine_identity):
    pin = manifest["pin"]
    return {
        "schema": PROVENANCE_SCHEMA,
        "export_sha256": seal,
        "source_commit": manifest["source_commit"],
        "source_tree": manifest["source_tree"],
        "suite_subtree": manifest["suite_subtree"],
        "legacy_import": manifest["legacy_import"],
        "suite_lock_sha256": pin["suite_lock_sha256"],
        "tuiscotti_commit": pin["tuiscotti_commit"],
        "engine": {
            "base_commit": pin["engine_base_commit"],
            "base_tree": pin["engine_base_tree"],
            "head": engine_identity["head"],
            "tree": engine_identity["tree"],
            "bundle_sha256": pin["bundle_sha256"],
            "patch_sha256": pin["patch_sha256"],
        },
        "data_sha256": data_digest,
        "reference_sha256": reference_digest,
        "boundaries": manifest["boundaries"],
    }


def cargo_inputs(root, manifest):
    paths = {str(root / MANIFEST_NAME), str(root / "export.py")}
    for entry in manifest["files"]:
        paths.add(str(root / entry["path"]))
    for relative in manifest["directories"]:
        paths.add(str(root / relative))
    engine_root = root / ENGINE_RELATIVE
    engine_tree = git_call(engine_root, ["ls-tree", "-r", "-z", "--full-tree", "HEAD"])
    for record in engine_tree.split(b"\0"):
        if not record:
            continue
        _, raw_path = record.split(b"\t", 1)
        relative = raw_path.decode("utf-8")
        paths.add(str(engine_root / relative))
        parts = validate_relative(relative, "engine source path")
        for index in range(1, len(parts)):
            paths.add(str(engine_root / "/".join(parts[:index])))
    git_dir = engine_root / ".git"
    paths.update(
        {
            str(engine_root),
            str(git_dir),
            str(git_dir / "HEAD"),
            str(git_dir / "config"),
            str(git_dir / "index"),
            str(git_dir / "packed-refs"),
            str(git_dir / "refs"),
            str(git_dir / "refs" / "heads"),
            str(git_dir / "objects"),
            str(git_dir / "objects" / "info"),
        }
    )
    for value in sorted(paths):
        if any(char in value for char in ("\n", "\r", "\x00")):
            fail("Cargo rerun path contains a control character")
    return sorted(paths)


def verify_export(root_value, expected_seal, bin_value=None, output_value=None, cargo=False, emit_result=True):
    require_hex(expected_seal, LOWER_HEX_64, "external export seal")
    root, _ = inspect_directory(root_value, "export root")
    manifest_bytes, manifest_info = safe_read_relative(root, MANIFEST_NAME, "export manifest")
    if not stat.S_ISREG(manifest_info.st_mode) or sha256(manifest_bytes) != expected_seal:
        fail("export manifest does not match the external seal")
    manifest = load_json(manifest_bytes, "export manifest")
    data_digest, reference_digest = validate_manifest(manifest)
    export_path, _, _ = validate_export_boundaries(root, manifest["boundaries"])
    validate_selected_paths(root, manifest["boundaries"], bin_value, output_value)
    source_tree_matches(
        manifest["boundaries"], manifest["source_commit"], manifest["source_tree"], manifest["suite_subtree"]
    )
    verify_export_files(export_path, manifest)
    bundle_data, _ = safe_read_relative(export_path, BUNDLE_RELATIVE, "engine bundle")
    engine_identity = verify_generated_paths(export_path, bundle_data)
    verify_engine_boundaries(export_path, manifest["boundaries"])
    result = provenance(manifest, expected_seal, data_digest, reference_digest, engine_identity)
    if cargo:
        for path in cargo_inputs(export_path, manifest):
            print(f"cargo:rerun-if-changed={path}")
    elif emit_result:
        print(json.dumps(result, ensure_ascii=False, sort_keys=True, separators=(",", ":")))


def validate_create_arguments(arguments):
    if len(arguments.application_root) != 2:
        fail("create requires one reference and one candidate application root")
    if len(arguments.bin_root) != 2:
        fail("create requires one reference and one candidate binary root")
    if not arguments.run_output_root:
        fail("create requires at least one run-output root")
    require_hex(arguments.source_commit, LOWER_HEX_40, "source commit")
    source_root, _ = inspect_directory(arguments.source_root, "source root")
    git_worktree_root(source_root, "source root")
    commit = git_text(source_root, ["rev-parse", "--verify", f"{arguments.source_commit}^{{commit}}"])
    if commit != arguments.source_commit or git_text(source_root, ["rev-parse", "HEAD"]) != commit:
        fail("source root HEAD does not match --source-commit")
    status = git_call(source_root, ["status", "--porcelain=v1", "--untracked-files=all"])
    if status:
        fail("source root worktree must be clean")
    out_path = inspect_new_directory(arguments.out, "export destination")
    boundaries, identities = boundary_records(
        source_root, arguments.application_root, arguments.bin_root, arguments.run_output_root
    )
    boundary_paths = [
        (pathlib.Path(boundaries[key]), key.replace("_", " "), identities[boundaries[key]])
        for key in (
            "source_root",
            "canonical_root",
            "reference_application_root",
            "candidate_application_root",
            "reference_bin_root",
            "candidate_bin_root",
        )
    ]
    for path, label, identity in boundary_paths:
        require_disjoint(out_path, "export destination", path, label, None, identity)
    for run_root in boundaries["run_output_roots"]:
        require_disjoint(out_path, "export destination", pathlib.Path(run_root), "run-output root")
    return source_root, commit, out_path, boundaries


def create_export(arguments):
    source_root, commit, out_path, boundaries = validate_create_arguments(arguments)
    source_tree, suite_tree, file_entries, files, directories = read_committed_suite(source_root, commit)
    data_digest, reference_digest = data_and_reference_digests(file_entries)
    manifest = {
        "schema": SCHEMA,
        "source_commit": commit,
        "source_tree": source_tree,
        "suite_subtree": suite_tree,
        "legacy_import": LEGACY_IMPORT,
        "pin": PIN,
        "boundaries": boundaries,
        "files": file_entries,
        "directories": directories,
    }
    validate_manifest(manifest)
    manifest_bytes = (json.dumps(manifest, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8")
    seal = sha256(manifest_bytes)
    parent_descriptor = secure_mkdir_absolute(out_path.parent, "export destination parent")
    try:
        try:
            os.mkdir(out_path.name, 0o755, dir_fd=parent_descriptor)
        except FileExistsError:
            fail("export destination already exists")
        root_descriptor = os.open(out_path.name, directory_flags(), dir_fd=parent_descriptor)
    finally:
        os.close(parent_descriptor)
    try:
        for relative in directories:
            created = secure_mkdir_relative(root_descriptor, relative)
            os.close(created)
        for entry in files:
            mode = 0o755 if entry["mode"] == "100755" else 0o644
            secure_write_relative(root_descriptor, entry["path"], entry["data"], mode)
        secure_write_relative(root_descriptor, MANIFEST_NAME, manifest_bytes, 0o644)
    finally:
        os.close(root_descriptor)
    verify_export(out_path, seal, emit_result=False)
    print(f"H={seal}")
    print(f"data_sha256={data_digest}")
    print(f"reference_sha256={reference_digest}")


def unique_store(parser, flag, **kwargs):
    class UniqueStore(argparse.Action):
        def __call__(self, parser, namespace, values, option_string=None):
            if getattr(namespace, self.dest, None) is not None:
                parser.error(f"{option_string} may be used once")
            setattr(namespace, self.dest, values)

    parser.add_argument(flag, action=UniqueStore, **kwargs)


def make_parser():
    parser = argparse.ArgumentParser(prog="export.py")
    commands = parser.add_subparsers(dest="command", required=True)
    create = commands.add_parser("create")
    unique_store(create, "--source-root", required=True)
    unique_store(create, "--source-commit", required=True)
    unique_store(create, "--out", required=True)
    create.add_argument("--application-root", action="append", required=True)
    create.add_argument("--bin-root", action="append", required=True)
    create.add_argument("--run-output-root", action="append", required=True)
    verify = commands.add_parser("verify")
    unique_store(verify, "--root", required=True)
    unique_store(verify, "--sha256", required=True)
    unique_store(verify, "--bin-dir")
    unique_store(verify, "--run-out")
    verify.add_argument("--cargo", action="store_true")
    return parser


def main():
    try:
        arguments = make_parser().parse_args()
        if arguments.command == "create":
            create_export(arguments)
        else:
            verify_export(
                arguments.root,
                arguments.sha256,
                arguments.bin_dir,
                arguments.run_out,
                arguments.cargo,
            )
        return 0
    except Exception as error:
        print(f"export.py: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
