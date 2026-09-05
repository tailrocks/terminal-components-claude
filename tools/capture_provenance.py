#!/usr/bin/env python3
"""Write reproducible, concurrency-safe capture provenance."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import secrets
import stat
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, NoReturn


SAFE_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")
WORKSPACE_ROOT = Path(__file__).resolve().parent.parent


def now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def fail(message: str) -> NoReturn:
    raise SystemExit(f"capture provenance: {message}")


def provenance_path(raw_path: str) -> str:
    """Return a stable workspace-relative path for published evidence."""
    path = Path(raw_path)
    if not path.is_absolute():
        return raw_path
    try:
        return path.relative_to(WORKSPACE_ROOT).as_posix()
    except ValueError:
        return raw_path


def file_info(raw_path: str, *, display_path: str | None = None) -> dict[str, Any]:
    """Describe a file without following a symlink at the requested path."""
    path = Path(raw_path)
    info: dict[str, Any] = {"path": raw_path if display_path is None else display_path}
    try:
        file_stat = path.lstat()
    except FileNotFoundError:
        info.update({"bytes": None, "sha256": None, "status": "missing"})
        return info
    except OSError as error:
        info.update({"bytes": None, "sha256": None, "status": f"error: {error}"})
        return info

    if not stat.S_ISREG(file_stat.st_mode) or path.is_symlink():
        info.update({"bytes": None, "sha256": None, "status": "not-regular"})
        return info

    try:
        descriptor, opened_stat = open_regular_read(raw_path, "capture file")
    except SystemExit as error:
        info.update(
            {
                "bytes": file_stat.st_size,
                "sha256": None,
                "status": f"error: {error}",
            }
        )
        return info

    digest = hashlib.sha256()
    bytes_read = 0
    try:
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = None
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
                bytes_read += len(chunk)
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        info.update({"bytes": opened_stat.st_size, "sha256": None, "status": f"error: {error}"})
        return info

    info.update(
        {
            "bytes": bytes_read,
            "sha256": digest.hexdigest(),
            "status": "ok" if bytes_read else "empty",
        }
    )
    info["resolved_path"] = str(path.absolute())
    return info


def parse_exit(raw: str) -> int | str | None:
    try:
        return int(raw)
    except ValueError:
        return raw or None


def parse_env(values: list[str]) -> dict[str, str | None]:
    result: dict[str, str | None] = {}
    for value in values:
        key, separator, raw = value.partition("=")
        if not separator or not key or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key):
            fail(f"invalid environment snapshot entry: {value!r}")
        result[key] = None if raw == "<unset>" else raw
    return result


def parse_pairs(values: list[str], label: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for value in values:
        key, separator, raw = value.partition("=")
        if not separator or not key:
            fail(f"invalid {label} snapshot entry: {value!r}")
        result[key] = raw
    return result


def parse_files(values: list[str]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for value in values:
        key, separator, raw = value.partition("=")
        if not separator or not key or not raw:
            fail(f"invalid tool file entry: {value!r}")
        result[key] = file_info(raw)
    return result


def ensure_regular(path: str, label: str) -> None:
    candidate = Path(path)
    if candidate.is_symlink() or not candidate.is_file():
        fail(f"{label} is not a regular file: {path}")


def ensure_not_symlink(path: Path, label: str) -> None:
    try:
        if path.is_symlink():
            fail(f"{label} is a symlink: {path}")
    except OSError as error:
        fail(f"cannot inspect {label} {path}: {error}")


def open_trusted_directory(path: Path, label: str) -> int:
    """Open every directory component without following any symlink.

    A leaf-only O_NOFOLLOW check is insufficient: an attacker can replace an
    ancestor after a check and redirect a later open/rename. Walking from the
    root with directory descriptors pins each component used by the caller.
    """
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail(f"cannot safely open {label} without O_NOFOLLOW")
    if not path.is_absolute():
        fail(f"{label} must be an absolute path: {path}")
    # macOS exposes /var and /tmp as stable system symlinks into /private.
    # Canonicalize only that first system component; all caller-controlled
    # components are still walked with O_NOFOLLOW below.
    components = path.parts[1:]
    if components and components[0] in {"tmp", "var"}:
        system_component = Path(os.sep) / components[0]
        if system_component.is_symlink():
            try:
                path = system_component.resolve(strict=True).joinpath(*components[1:])
            except OSError as error:
                fail(f"cannot resolve stable system path component for {label}: {error}")
    directory_flag = getattr(os, "O_DIRECTORY", 0)
    flags = os.O_RDONLY | directory_flag | nofollow
    descriptor = os.open(os.sep, flags)
    try:
        for component in path.parts[1:]:
            if component in {"", ".", ".."}:
                os.close(descriptor)
                descriptor = -1
                fail(f"{label} contains an unsafe path component: {path}")
            child = os.open(component, flags, dir_fd=descriptor)
            metadata = os.fstat(child)
            if not stat.S_ISDIR(metadata.st_mode):
                os.close(child)
                os.close(descriptor)
                descriptor = -1
                fail(f"{label} is not a directory: {path}")
            os.close(descriptor)
            descriptor = child
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot open trusted {label} {path}: {error}")
    return descriptor


def create_temporary_at(directory: int, prefix: str) -> tuple[int, str]:
    """Create a mode-0600 temporary regular file below a pinned directory."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail("cannot safely create temporary capture state without O_NOFOLLOW")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | nofollow
    for _ in range(32):
        name = f".{prefix}.{os.getpid()}.{secrets.token_hex(8)}.tmp"
        try:
            return os.open(name, flags, 0o600, dir_fd=directory), name
        except FileExistsError:
            continue
        except OSError as error:
            fail(f"cannot create temporary capture state {name}: {error}")
    fail(f"cannot create a unique temporary capture state for {prefix}")


def load_json(path: Path) -> Any:
    try:
        return json.loads(read_regular_text(path, "JSON state"))
    except json.JSONDecodeError as error:
        fail(f"cannot read JSON state {path}: {error}")


def read_regular_text_at(directory: int, name: str, label: str) -> str:
    """Read a regular leaf below an already pinned directory descriptor."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail(f"cannot safely read {label} without O_NOFOLLOW")
    descriptor = -1
    try:
        descriptor = os.open(name, os.O_RDONLY | nofollow, dir_fd=directory)
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            descriptor = -1
            fail(f"{label} is not a regular file: {name}")
        with os.fdopen(descriptor, "r", encoding="utf-8") as stream:
            descriptor = -1
            return stream.read()
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot read {label} {name}: {error}")


def load_json_at(directory: int, name: str, path: Path) -> Any:
    try:
        return json.loads(read_regular_text_at(directory, name, "JSON state"))
    except json.JSONDecodeError as error:
        fail(f"cannot read JSON state {path}: {error}")


def write_json_at(directory: int, name: str, path: Path, value: Any) -> None:
    temporary_name: str | None = None
    try:
        descriptor, temporary_name = create_temporary_at(directory, name)
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            os.chmod(stream.fileno(), 0o600)
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, path.name, src_dir_fd=directory, dst_dir_fd=directory)
        os.fsync(directory)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name, dir_fd=directory)
            except FileNotFoundError:
                pass


def write_json(path: Path, value: Any) -> None:
    directory = open_trusted_directory(path.parent, "JSON parent directory")
    try:
        write_json_at(directory, path.name, path, value)
    finally:
        os.close(directory)


def write_text_atomic(path: Path, value: str) -> None:
    """Write a small state value without following a destination symlink."""
    directory = open_trusted_directory(path.parent, "capture state parent directory")
    temporary_name: str | None = None
    try:
        descriptor, temporary_name = create_temporary_at(directory, path.name)
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            os.chmod(stream.fileno(), 0o600)
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, path.name, src_dir_fd=directory, dst_dir_fd=directory)
        os.fsync(directory)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name, dir_fd=directory)
            except FileNotFoundError:
                pass
        os.close(directory)


def open_stderr(path: Path) -> Any:
    """Open owned stderr state without following a symlink or special file."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail("cannot safely open capture stderr without O_NOFOLLOW")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | nofollow
    directory = open_trusted_directory(path.parent, "capture stderr parent directory")
    descriptor = -1
    try:
        descriptor = os.open(path.name, flags, 0o600, dir_fd=directory)
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            descriptor = -1
            fail(f"capture stderr is not a regular file: {path}")
        stream = os.fdopen(descriptor, "wb")
        descriptor = -1
        return stream
    except OSError as error:
        fail(f"cannot open capture stderr {path}: {error}")
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        os.close(directory)


def open_regular_read(path: str, label: str) -> tuple[int, os.stat_result]:
    """Open a regular file while pinning every parent directory component."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail(f"cannot safely open {label} without O_NOFOLLOW")
    candidate = Path(path)
    directory = open_trusted_directory(candidate.parent, f"{label} parent directory")
    descriptor = -1
    try:
        descriptor = os.open(candidate.name, os.O_RDONLY | nofollow, dir_fd=directory)
        metadata = os.fstat(descriptor)
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        os.close(directory)
        fail(f"cannot open {label} {path}: {error}")
    os.close(directory)
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        fail(f"{label} is not a regular file: {path}")
    return descriptor, metadata


def read_regular_text(path: Path, label: str) -> str:
    """Read a regular file through one no-follow descriptor."""
    descriptor, _ = open_regular_read(str(path), label)
    try:
        with os.fdopen(descriptor, "r", encoding="utf-8") as stream:
            descriptor = -1
            return stream.read()
    except (OSError, UnicodeError) as error:
        fail(f"cannot read {label} {path}: {error}")
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def hash_descriptor(descriptor: int) -> tuple[str, int]:
    """Hash bytes from an already opened descriptor without reopening its path."""
    os.lseek(descriptor, 0, os.SEEK_SET)
    digest = hashlib.sha256()
    bytes_read = 0
    for chunk in iter(lambda: os.read(descriptor, 1024 * 1024), b""):
        digest.update(chunk)
        bytes_read += len(chunk)
    return digest.hexdigest(), bytes_read


def lock_execution_directory(path: Path) -> tuple[int, int]:
    """Pin a staged path by removing its directory write permission while it runs.

    macOS does not expose fexecve/execveat to this Python process. The staged
    file is mode 0500 and lives in this private mode-0700 directory; holding
    the directory descriptor and removing all directory write permission
    prevents rename/unlink replacement between descriptor verification and
    the interpreter's path-based launch. The original mode is restored after
    the child exits.
    """
    descriptor = -1
    try:
        descriptor = open_trusted_directory(path, "capture run directory")
        metadata = os.fstat(descriptor)
        original_mode = stat.S_IMODE(metadata.st_mode)
        if not original_mode & stat.S_IXUSR:
            fail(f"capture run directory is not searchable by its owner: {path}")
        os.fchmod(descriptor, original_mode & ~0o222)
        return descriptor, original_mode
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot pin capture executable directory {path}: {error}")


def restore_execution_directory(descriptor: int, path: Path, mode: int) -> None:
    """Restore the run directory mode after the path-pinned execution ends."""
    try:
        os.fchmod(descriptor, mode)
    except OSError as error:
        fail(f"cannot restore capture run directory {path}: {error}")


def open_manifest_lock(path: Path, directory: int | None = None) -> Any:
    """Open a manifest lock without following a replaced symlink."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail("cannot safely open capture manifest lock without O_NOFOLLOW")
    owns_directory = directory is None
    if owns_directory:
        directory = open_trusted_directory(
            path.parent, "capture manifest lock parent directory"
        )
    descriptor = -1
    try:
        descriptor = os.open(
            path.name, os.O_RDWR | os.O_CREAT | nofollow, 0o600, dir_fd=directory
        )
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            descriptor = -1
            fail(f"capture manifest lock is not a regular file: {path}")
        lock = os.fdopen(descriptor, "a+")
        descriptor = -1
        return lock
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot open capture manifest lock {path}: {error}")
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        if owns_directory:
            os.close(directory)


def path_exists_without_follow(path: Path, label: str) -> bool:
    """Check presence below a pinned parent without following the leaf."""
    directory = open_trusted_directory(path.parent, f"{label} parent directory")
    try:
        os.stat(path.name, dir_fd=directory, follow_symlinks=False)
        return True
    except FileNotFoundError:
        return False
    except OSError as error:
        fail(f"cannot inspect {label} {path}: {error}")
    finally:
        os.close(directory)


def lock_path_parts(lock: Path, guard: Path) -> tuple[str, str, int]:
    """Pin the common parent used by a lock and its advisory guard."""
    if not lock.is_absolute() or not guard.is_absolute() or lock.parent != guard.parent:
        fail("capture lock and guard must be absolute paths with one common parent")
    if lock.name in {"", ".", ".."} or guard.name in {"", ".", ".."}:
        fail("capture lock names are unsafe")
    directory = open_trusted_directory(lock.parent, "capture lock parent directory")
    return lock.name, guard.name, directory


def open_lock_guard(directory: int, name: str) -> int:
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail("cannot safely open capture lock guard without O_NOFOLLOW")
    descriptor = -1
    try:
        descriptor = os.open(name, os.O_RDWR | os.O_CREAT | nofollow, 0o600, dir_fd=directory)
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            descriptor = -1
            fail(f"capture lock guard is not a regular file: {name}")
        return descriptor
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot open capture lock guard {name}: {error}")


def read_owner_at(directory: int, name: str, label: str) -> str:
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail(f"cannot safely read {label} without O_NOFOLLOW")
    descriptor = -1
    try:
        descriptor = os.open(name, os.O_RDONLY | nofollow, dir_fd=directory)
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            os.close(descriptor)
            descriptor = -1
            fail(f"{label} is not a regular file: {name}")
        with os.fdopen(descriptor, "r", encoding="utf-8") as stream:
            descriptor = -1
            return stream.read(1024)
    except OSError as error:
        if descriptor >= 0:
            os.close(descriptor)
        fail(f"cannot read {label} {name}: {error}")


def parse_lock_owner(raw: str, label: str) -> tuple[int, str]:
    match = re.fullmatch(r"([1-9][0-9]*):([A-Za-z0-9_-]{1,64})\n?", raw)
    if match is None:
        fail(f"{label} is invalid")
    return int(match.group(1)), match.group(2)


def process_is_live(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    except OSError as error:
        fail(f"cannot determine lock owner liveness for pid {pid}: {error}")
    return True


def quarantine_name(prefix: str) -> str:
    return f".{prefix}.stale.{os.getpid()}.{secrets.token_hex(8)}"


def remove_legacy_lock_directory(directory: int, name: str) -> None:
    """Remove only a legacy lock directory containing exactly one owner file."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", 0)
    if nofollow is None:
        fail("cannot safely migrate a legacy shot lock without O_NOFOLLOW")
    child = -1
    try:
        child = os.open(name, os.O_RDONLY | directory_flag | nofollow, dir_fd=directory)
        entries = os.listdir(child)
        if entries != ["owner"]:
            fail(f"legacy shot lock contains unexpected entries: {name}")
        owner_metadata = os.stat("owner", dir_fd=child, follow_symlinks=False)
        if not stat.S_ISREG(owner_metadata.st_mode):
            fail(f"legacy shot lock owner is not a regular file: {name}")
        os.unlink("owner", dir_fd=child)
    except OSError as error:
        fail(f"cannot inspect legacy shot lock {name}: {error}")
    finally:
        if child >= 0:
            os.close(child)
    try:
        os.rmdir(name, dir_fd=directory)
    except OSError as error:
        fail(f"cannot remove legacy shot lock {name}: {error}")


def publish_shot_lock(directory: int, name: str, owner: str) -> None:
    descriptor, temporary_name = create_temporary_at(directory, f"{name}.lock")
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            descriptor = -1
            stream.write(f"{owner}\n")
            stream.flush()
            os.fsync(stream.fileno())
        try:
            os.link(
                temporary_name,
                name,
                src_dir_fd=directory,
                dst_dir_fd=directory,
                follow_symlinks=False,
            )
        except FileExistsError:
            fail(f"shot lock appeared while acquiring: {name}")
        os.unlink(temporary_name, dir_fd=directory)
        os.fsync(directory)
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        try:
            os.unlink(temporary_name, dir_fd=directory)
        except FileNotFoundError:
            pass


def inspect_and_recover_stale_lock(directory: int, name: str) -> None:
    metadata = os.stat(name, dir_fd=directory, follow_symlinks=False)
    if stat.S_ISREG(metadata.st_mode):
        owner = read_owner_at(directory, name, "shot lock owner")
        pid, _ = parse_lock_owner(owner.strip(), "shot lock owner")
        if process_is_live(pid):
            fail(f"shot is already running: {name.removesuffix('.lock')}")
        quarantine = quarantine_name(name)
        os.rename(name, quarantine, src_dir_fd=directory, dst_dir_fd=directory)
        os.unlink(quarantine, dir_fd=directory)
        os.fsync(directory)
        return
    if stat.S_ISDIR(metadata.st_mode):
        # The owner is read from the directory itself before the directory is
        # atomically quarantined. A missing/unsafe owner is never deleted.
        nofollow = getattr(os, "O_NOFOLLOW", None)
        if nofollow is None:
            fail("cannot safely inspect a legacy shot lock without O_NOFOLLOW")
        child = os.open(
            name,
            os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | nofollow,
            dir_fd=directory,
        )
        try:
            owner = read_owner_at(child, "owner", "legacy shot lock owner")
        finally:
            os.close(child)
        pid, _ = parse_lock_owner(owner.strip(), "legacy shot lock owner")
        if process_is_live(pid):
            fail(f"shot is already running: {name.removesuffix('.lock')}")
        quarantine = quarantine_name(name)
        os.rename(name, quarantine, src_dir_fd=directory, dst_dir_fd=directory)
        remove_legacy_lock_directory(directory, quarantine)
        os.fsync(directory)
        return
    fail(f"shot lock path is unsafe: {name}")


def command_shot_lock(args: argparse.Namespace) -> None:
    lock = Path(args.lock)
    guard = Path(args.guard)
    parse_lock_owner(args.owner, "requested shot lock owner")
    name, guard_name, directory = lock_path_parts(lock, guard)
    guard_descriptor = -1
    try:
        guard_descriptor = open_lock_guard(directory, guard_name)
        try:
            fcntl.flock(guard_descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            fail(f"shot is already running: {name.removesuffix('.lock')}")
        try:
            if args.action == "acquire":
                try:
                    os.stat(name, dir_fd=directory, follow_symlinks=False)
                except FileNotFoundError:
                    pass
                else:
                    inspect_and_recover_stale_lock(directory, name)
                publish_shot_lock(directory, name, args.owner)
            else:
                try:
                    metadata = os.stat(name, dir_fd=directory, follow_symlinks=False)
                except FileNotFoundError:
                    fail(f"shot lock is missing: {name}")
                if not stat.S_ISREG(metadata.st_mode):
                    fail(f"shot lock is not a regular file: {name}")
                owner = read_owner_at(directory, name, "shot lock owner").strip()
                if owner != args.owner:
                    fail(f"shot lock ownership changed: {name}")
                quarantine = quarantine_name(name)
                os.rename(name, quarantine, src_dir_fd=directory, dst_dir_fd=directory)
                os.unlink(quarantine, dir_fd=directory)
                os.fsync(directory)
        finally:
            fcntl.flock(guard_descriptor, fcntl.LOCK_UN)
    except OSError as error:
        fail(f"capture shot lock operation failed: {error}")
    finally:
        if guard_descriptor >= 0:
            os.close(guard_descriptor)
        os.close(directory)


def command_stage_binary(args: argparse.Namespace) -> None:
    """Copy one verified executable into the private run directory."""
    source = Path(args.source)
    destination = Path(args.destination)
    ensure_regular(str(source), "capture binary")
    if not os.access(source, os.X_OK):
        fail(f"capture binary is not executable: {source}")

    descriptor, source_metadata = open_regular_read(str(source), "capture binary")
    try:
        destination_directory = open_trusted_directory(
            destination.parent, "capture executable parent directory"
        )
    except BaseException:
        os.close(descriptor)
        raise
    temporary_name: str | None = None
    bytes_copied = 0
    try:
        try:
            os.stat(destination.name, dir_fd=destination_directory, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            fail(f"capture executable already exists: {destination}")
        digest = hashlib.sha256()
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            temporary_descriptor, temporary_name = create_temporary_at(
                destination_directory, destination.name
            )
            with os.fdopen(temporary_descriptor, "wb") as output:
                os.fchmod(output.fileno(), 0o500)
                for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                    digest.update(chunk)
                    output.write(chunk)
                    bytes_copied += len(chunk)
                output.flush()
                os.fsync(output.fileno())
        if source_metadata.st_size == 0 or bytes_copied == 0:
            fail(f"capture binary is empty: {source}")
        try:
            os.link(
                temporary_name,
                destination.name,
                src_dir_fd=destination_directory,
                dst_dir_fd=destination_directory,
                follow_symlinks=False,
            )
        except FileExistsError:
            fail(f"capture executable appeared while staging: {destination}")
        os.unlink(temporary_name, dir_fd=destination_directory)
        os.fsync(destination_directory)
        temporary_name = None
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        if temporary_name is not None:
            try:
                os.unlink(temporary_name, dir_fd=destination_directory)
            except FileNotFoundError:
                pass
        os.close(destination_directory)

    copied = file_info(str(destination))
    if copied.get("status") != "ok" or not copied.get("sha256"):
        fail(f"staged capture binary is not a non-empty regular file: {destination}")
    if copied["sha256"] != digest.hexdigest():
        fail("staged capture binary changed while it was being published")
    staged_descriptor, staged_metadata = open_regular_read(
        str(destination), "staged capture binary"
    )
    os.close(staged_descriptor)
    if not staged_metadata.st_mode & 0o111:
        fail(f"staged capture binary is not executable: {destination}")


def update_manifest(
    manifest_path: Path, update: Any, *, create_if_missing: bool = True
) -> bool:
    """Run update(records) while holding a per-manifest advisory lock."""
    directory = open_trusted_directory(manifest_path.parent, "capture manifest parent directory")
    lock_path = manifest_path.with_name(f".{manifest_path.name}.lock")
    try:
        with open_manifest_lock(lock_path, directory) as lock:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
            try:
                try:
                    os.stat(manifest_path.name, dir_fd=directory, follow_symlinks=False)
                except FileNotFoundError:
                    if not create_if_missing:
                        return False
                    records = []
                else:
                    loaded = load_json_at(directory, manifest_path.name, manifest_path)
                    if not isinstance(loaded, list):
                        fail(f"capture provenance is not a JSON array: {manifest_path}")
                    records = loaded
                updated = update(records)
                write_json_at(directory, manifest_path.name, manifest_path, updated)
                return True
            finally:
                fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
    finally:
        os.close(directory)


def metadata_from_args(args: argparse.Namespace) -> dict[str, Any]:
    ensure_regular(args.binary, "capture binary")
    source_binary = args.source_binary or args.binary
    ensure_regular(source_binary, "capture source binary")
    if not args.argv or args.argv[0] != source_binary:
        fail("exact argv must be non-empty and begin with BIN")
    if not re.fullmatch(r"[0-9a-fA-F]{40}", args.revision):
        fail(f"revision is not a full commit id: {args.revision!r}")
    if not SAFE_NAME.fullmatch(args.run_id):
        fail(f"unsafe run id: {args.run_id!r}")
    return {
        "schema_version": 2,
        "run_id": args.run_id,
        "session": args.session,
        "session_id": args.session_id,
        "started_at": now(),
        "binary": file_info(args.binary),
        "binary_source": file_info(source_binary),
        "argv": args.argv,
        "executed_argv": [args.binary, *args.argv[1:]],
        "git": {"revision": args.revision, "dirty": args.dirty == "true"},
        "environment": parse_env(args.env),
        "tools": {
            "versions": parse_pairs(args.tool, "tool"),
            "files": parse_files(args.tool_file),
        },
        "requested_dimensions": {
            "columns": args.columns,
            "rows": args.rows,
        },
        "capture_dir": args.capture_dir,
        "manifest": args.manifest,
        "stderr": args.stderr,
        "theme": args.theme,
        "color": args.color,
        "status": "running",
        "exit_status": "running",
        "exit_observed": False,
    }


def command_init(args: argparse.Namespace) -> None:
    metadata = metadata_from_args(args)
    write_json(Path(args.metadata), metadata)


def command_set_session(args: argparse.Namespace) -> None:
    path = Path(args.metadata)
    directory = open_trusted_directory(path.parent, "capture metadata parent directory")
    try:
        metadata = load_json_at(directory, path.name, path)
        if not isinstance(metadata, dict):
            fail(f"capture metadata is not an object: {path}")
        if metadata.get("run_id") != args.run_id:
            fail("metadata run id does not match requested run id")
        metadata["session_id"] = args.session_id
        write_json_at(directory, path.name, path, metadata)
    finally:
        os.close(directory)


def command_read_state(args: argparse.Namespace) -> None:
    """Read one shell state value through a no-follow descriptor."""
    value = read_regular_text(Path(args.path), "capture state")
    print(value.replace("\r", "").replace("\n", ""))


def command_write_state(args: argparse.Namespace) -> None:
    """Write one shell state value below a pinned parent directory."""
    write_text_atomic(Path(args.path), f"{args.value}\n")


def recorded_environment(metadata: dict[str, Any]) -> dict[str, str]:
    snapshot = metadata.get("environment")
    if not isinstance(snapshot, dict):
        fail("capture metadata lacks a sanitized environment snapshot")
    environment: dict[str, str] = {}
    for key, value in snapshot.items():
        if not isinstance(key, str) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key):
            fail(f"capture metadata has an invalid environment key: {key!r}")
        if value is not None and not isinstance(value, str):
            fail(f"capture metadata has an invalid environment value for {key}")
        if value is not None:
            environment[key] = value
    return environment


def command_exec(args: argparse.Namespace) -> int:
    """Execute the exact argv serialized in a live capture run's metadata."""
    metadata_path = Path(args.metadata)
    metadata = load_json(metadata_path)
    if not isinstance(metadata, dict) or metadata.get("run_id") != args.run_id:
        fail("capture metadata does not match requested run")

    argv = metadata.get("executed_argv")
    if (
        not isinstance(argv, list)
        or not argv
        or any(not isinstance(argument, str) for argument in argv)
    ):
        fail("capture metadata does not contain a valid argv list")
    binary = argv[0]
    binary_info = metadata.get("binary")
    if not isinstance(binary_info, dict) or binary_info.get("path") != binary:
        fail("capture metadata binary does not match argv[0]")
    expected_hash = binary_info.get("sha256")
    if not isinstance(expected_hash, str) or not expected_hash:
        fail("capture metadata lacks a binary content hash")

    color = metadata.get("color")
    if color != args.color:
        fail("capture color does not match the owning run metadata")
    expected_stderr = metadata.get("stderr")
    if expected_stderr != args.stderr:
        fail("capture stderr path does not match the owning run metadata")
    expected_exit = metadata_path.parent / "exit.status"
    if Path(args.exit) != expected_exit:
        fail("capture exit path does not belong to the owning run directory")
    directory_path = metadata_path.parent
    if Path(binary).parent != directory_path:
        fail("capture binary is not the immutable executable owned by the run directory")

    if color not in {"truecolor", "256", "16", "mono"}:
        fail(f"unsupported capture color: {color!r}")
    environment = recorded_environment(metadata)
    if environment.get("COLOR") != color:
        fail("capture environment COLOR does not match the owning run metadata")

    stderr_path = Path(args.stderr)
    stderr = open_stderr(stderr_path)
    directory_descriptor, directory_mode = lock_execution_directory(directory_path)
    binary_descriptor = -1
    return_code: int
    try:
        binary_descriptor, binary_metadata = open_regular_read(binary, "capture binary")
        if not binary_metadata.st_mode & 0o111:
            fail(f"capture binary is not executable: {binary}")
        current_hash, current_bytes = hash_descriptor(binary_descriptor)
        if not current_bytes or current_hash != expected_hash:
            fail("capture binary changed after provenance was initialized")
        try:
            process = subprocess.Popen(
                argv,
                cwd=Path(__file__).resolve().parent.parent,
                env=environment,
                stderr=stderr,
                shell=False,
            )
        except OSError as error:
            stderr.write(f"capture exec: cannot start application: {error}\n".encode())
            stderr.flush()
            return_code = 127
        else:
            return_code = process.wait()
    finally:
        if binary_descriptor >= 0:
            os.close(binary_descriptor)
        restore_execution_directory(directory_descriptor, directory_path, directory_mode)
        os.close(directory_descriptor)
        stderr.close()
    write_text_atomic(Path(args.exit), f"{return_code}\n")
    return return_code if return_code >= 0 else 128 + (-return_code)


def metadata_record(metadata: dict[str, Any], args: argparse.Namespace) -> dict[str, Any]:
    if metadata.get("run_id") != args.run_id:
        fail("capture metadata run id does not match requested run id")
    if not SAFE_NAME.fullmatch(args.name):
        fail(f"unsafe capture name: {args.name!r}")
    artifacts: dict[str, dict[str, Any]] = {}
    for raw in args.artifact:
        key, separator, path = raw.partition("=")
        if not separator or key not in {"ansi", "cursor", "txt", "html", "png"}:
            fail(f"invalid artifact entry: {raw!r}")
        artifacts[key] = file_info(path, display_path=provenance_path(path))
    if set(artifacts) != {"ansi", "cursor", "txt", "html", "png"}:
        fail("all five capture artifacts are required")

    run_binary = metadata.get("binary")
    source_binary = metadata.get("binary_source", run_binary)
    if not isinstance(run_binary, dict) or run_binary.get("sha256") is None:
        fail("capture metadata lacks a binary content hash")
    if not isinstance(source_binary, dict):
        fail("capture metadata lacks a source binary record")
    record = {
        "schema_version": 2,
        "captured_at": now(),
        "name": args.name,
        "run_id": args.run_id,
        "session": metadata.get("session"),
        "session_id": metadata.get("session_id"),
        "app": Path(str(source_binary.get("path", ""))).name,
        "binary": run_binary,
        "binary_source": source_binary,
        "argv": metadata.get("argv"),
        "executed_argv": metadata.get("executed_argv"),
        "git": metadata.get("git"),
        "revision": metadata.get("git", {}).get("revision"),
        "dirty": metadata.get("git", {}).get("dirty"),
        "environment": metadata.get("environment"),
        "tools": metadata.get("tools"),
        "theme": metadata.get("theme"),
        "color": metadata.get("color"),
        "status": args.status,
        "exit_status": parse_exit(args.exit_status),
        "exit_observed": args.exit_status.lstrip("-").isdigit(),
        "requested_dimensions": metadata.get("requested_dimensions"),
        "dimensions": {"columns": args.columns, "rows": args.rows},
        "stderr": file_info(args.stderr, display_path=provenance_path(args.stderr)),
        "artifacts": artifacts,
    }
    return record


def command_record(args: argparse.Namespace) -> None:
    metadata_path = Path(args.metadata)
    metadata = load_json(metadata_path)
    if not isinstance(metadata, dict):
        fail(f"capture metadata is not an object: {metadata_path}")
    record = metadata_record(metadata, args)

    def replace(records: list[Any]) -> list[Any]:
        return [item for item in records if item.get("name") != args.name] + [record]

    update_manifest(Path(args.manifest), replace)


def command_finalize(args: argparse.Namespace) -> None:
    metadata_path = Path(args.metadata)
    metadata_directory = open_trusted_directory(
        metadata_path.parent, "capture metadata parent directory"
    )
    try:
        metadata = load_json_at(metadata_directory, metadata_path.name, metadata_path)
        if not isinstance(metadata, dict) or metadata.get("run_id") != args.run_id:
            fail("capture metadata does not match requested run")
        parsed = parse_exit(args.exit_status)
        metadata["status"] = "finalized"
        metadata["exit_status"] = parsed
        metadata["exit_observed"] = isinstance(parsed, int)
        metadata["termination"] = "natural_exit" if isinstance(parsed, int) else "capture_stop"
        metadata["finalized_at"] = now()
        metadata["stderr_info"] = file_info(
            args.stderr, display_path=provenance_path(args.stderr)
        )
        write_json_at(metadata_directory, metadata_path.name, metadata_path, metadata)
    finally:
        os.close(metadata_directory)

    manifest = Path(args.manifest)

    def finalize(records: list[Any]) -> list[Any]:
        changed = []
        for record in records:
            if record.get("run_id") != args.run_id or record.get("exit_status") != "running":
                changed.append(record)
                continue
            record = dict(record)
            record["exit_status"] = parsed
            record["exit_observed"] = isinstance(parsed, int)
            record["termination"] = "natural_exit" if isinstance(parsed, int) else "capture_stop"
            record["finalized_at"] = now()
            record["stderr"] = file_info(
                args.stderr, display_path=provenance_path(args.stderr)
            )
            changed.append(record)
        return changed

    update_manifest(manifest, finalize, create_if_missing=False)


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    subparsers = root.add_subparsers(dest="command", required=True)

    init = subparsers.add_parser("init")
    init.add_argument("--metadata", required=True)
    init.add_argument("--run-id", required=True)
    init.add_argument("--session", required=True)
    init.add_argument("--session-id", required=True)
    init.add_argument("--binary", required=True)
    init.add_argument("--source-binary")
    init.add_argument("--revision", required=True)
    init.add_argument("--dirty", choices=("true", "false"), required=True)
    init.add_argument("--columns", type=int, required=True)
    init.add_argument("--rows", type=int, required=True)
    init.add_argument("--capture-dir", required=True)
    init.add_argument("--manifest", required=True)
    init.add_argument("--stderr", required=True)
    init.add_argument("--theme", required=True)
    init.add_argument("--color", required=True)
    init.add_argument("--env", action="append", default=[])
    init.add_argument("--tool", action="append", default=[])
    init.add_argument("--tool-file", action="append", default=[])
    init.add_argument("--argv", nargs=argparse.REMAINDER, required=True)
    init.set_defaults(handler=command_init)

    stage = subparsers.add_parser("stage-binary")
    stage.add_argument("--source", required=True)
    stage.add_argument("--destination", required=True)
    stage.set_defaults(handler=command_stage_binary)

    session = subparsers.add_parser("set-session")
    session.add_argument("--metadata", required=True)
    session.add_argument("--run-id", required=True)
    session.add_argument("--session-id", required=True)
    session.set_defaults(handler=command_set_session)

    state = subparsers.add_parser("read-state")
    state.add_argument("--path", required=True)
    state.set_defaults(handler=command_read_state)

    write_state = subparsers.add_parser("write-state")
    write_state.add_argument("--path", required=True)
    write_state.add_argument("--value", required=True)
    write_state.set_defaults(handler=command_write_state)

    shot_lock = subparsers.add_parser("shot-lock")
    shot_lock.add_argument("action", choices=("acquire", "release"))
    shot_lock.add_argument("--lock", required=True)
    shot_lock.add_argument("--guard", required=True)
    shot_lock.add_argument("--owner", required=True)
    shot_lock.set_defaults(handler=command_shot_lock)

    execute = subparsers.add_parser("exec")
    execute.add_argument("--metadata", required=True)
    execute.add_argument("--run-id", required=True)
    execute.add_argument("--stderr", required=True)
    execute.add_argument("--exit", required=True)
    execute.add_argument(
        "--color", choices=("truecolor", "256", "16", "mono"), required=True
    )
    execute.set_defaults(handler=command_exec)

    record = subparsers.add_parser("record")
    record.add_argument("--metadata", required=True)
    record.add_argument("--manifest", required=True)
    record.add_argument("--run-id", required=True)
    record.add_argument("--name", required=True)
    record.add_argument("--columns", type=int, required=True)
    record.add_argument("--rows", type=int, required=True)
    record.add_argument("--status", choices=("ok", "failed"), required=True)
    record.add_argument("--exit-status", required=True)
    record.add_argument("--stderr", required=True)
    record.add_argument("--artifact", action="append", default=[])
    record.set_defaults(handler=command_record)

    finalize = subparsers.add_parser("finalize")
    finalize.add_argument("--metadata", required=True)
    finalize.add_argument("--manifest", required=True)
    finalize.add_argument("--run-id", required=True)
    finalize.add_argument("--exit-status", required=True)
    finalize.add_argument("--stderr", required=True)
    finalize.set_defaults(handler=command_finalize)
    return root


def main() -> int:
    args = parser().parse_args()
    result = args.handler(args)
    return result if isinstance(result, int) else 0


if __name__ == "__main__":
    sys.exit(main())
