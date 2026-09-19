#!/usr/bin/env bash
# Shared path guards for verifier-controlled regular files.
#
# This validates the lexical path actually supplied to a command. It permits
# only the documented macOS system symlinks used by native temporary/home
# paths. Callers choose whether their contract also requires a single link.

campaign_require_regular_file() {
	local path="${1-}"
	local field="${2-path}"
	local require_single_link="${3-0}"
	local require_executable="${4-0}"
	python3 - "$path" "$field" "$require_single_link" "$require_executable" <<'PY'
import os
import stat
import sys
from pathlib import Path


path_text, field, require_single_link, require_executable = sys.argv[1:]
path = Path(path_text)

allowed_system_symlinks = {
    Path("/etc"): Path("/private/etc"),
    Path("/home"): Path("/System/Volumes/Data/home"),
    Path("/tmp"): Path("/private/tmp"),
    Path("/var"): Path("/private/var"),
}


def fail(message: str) -> None:
    raise SystemExit(f"{field} {message}")


if not path.is_absolute():
    fail(f"must be an absolute path: {path_text or '<empty>'}")

current = Path(path.anchor)
for component in path.parts[1:-1]:
    current /= component
    try:
        metadata = os.lstat(current)
    except FileNotFoundError:
        fail(f"parent path component is missing: {current}")
    except OSError as error:
        fail(f"parent path component is unreadable: {current}: {error}")
    if stat.S_ISLNK(metadata.st_mode):
        if Path(os.path.realpath(current)) != allowed_system_symlinks.get(current):
            fail(f"parent path component must not be a symlink: {current}")
    elif not stat.S_ISDIR(metadata.st_mode):
        fail(f"parent path component is not a directory: {current}")

try:
    metadata = os.lstat(path)
except OSError as error:
    fail(f"is unreadable: {path}: {error}")

if stat.S_ISLNK(metadata.st_mode):
    fail(f"final path must not be a symlink: {path}")
if not stat.S_ISREG(metadata.st_mode):
    fail(f"is not a regular file: {path}")
if require_single_link == "1" and metadata.st_nlink != 1:
    fail(f"must be a regular single-link file: {path} (links={metadata.st_nlink})")
if require_executable == "1" and not (metadata.st_mode & 0o111):
    fail(f"is not executable: {path}")
PY
}
