#!/usr/bin/env python3
"""Write reproducible, concurrency-safe capture provenance."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, NoReturn


SAFE_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")


def now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def fail(message: str) -> NoReturn:
    raise SystemExit(f"capture provenance: {message}")


def file_info(raw_path: str) -> dict[str, Any]:
    """Describe a file without following a symlink at the requested path."""
    path = Path(raw_path)
    info: dict[str, Any] = {"path": raw_path}
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

    digest = hashlib.sha256()
    bytes_read = 0
    descriptor: int | None = None
    try:
        nofollow = getattr(os, "O_NOFOLLOW", None)
        if nofollow is None:
            info.update(
                {
                    "bytes": file_stat.st_size,
                    "sha256": None,
                    "status": "error: O_NOFOLLOW is unavailable",
                }
            )
            return info
        descriptor = os.open(raw_path, os.O_RDONLY | nofollow)
        opened_stat = os.fstat(descriptor)
        if not stat.S_ISREG(opened_stat.st_mode):
            os.close(descriptor)
            info.update({"bytes": None, "sha256": None, "status": "not-regular"})
            return info
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = None
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
                bytes_read += len(chunk)
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        info.update({"bytes": file_stat.st_size, "sha256": None, "status": f"error: {error}"})
        return info

    info.update(
        {
            "bytes": bytes_read,
            "sha256": digest.hexdigest(),
            "status": "ok" if bytes_read else "empty",
        }
    )
    try:
        info["resolved_path"] = str(path.resolve(strict=True))
    except OSError:
        pass
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


def load_json(path: Path) -> Any:
    ensure_not_symlink(path, "JSON state")
    try:
        with path.open(encoding="utf-8") as stream:
            return json.load(stream)
    except FileNotFoundError:
        fail(f"JSON state is missing: {path}")
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot read JSON state {path}: {error}")


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    ensure_not_symlink(path, "JSON state")
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as stream:
            temporary_name = stream.name
            os.chmod(stream.fileno(), 0o600)
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, path)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def write_text_atomic(path: Path, value: str) -> None:
    """Write a small state value without following a destination symlink."""
    ensure_not_symlink(path, "capture state")
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as stream:
            temporary_name = stream.name
            os.chmod(stream.fileno(), 0o600)
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
        ensure_not_symlink(path, "capture state")
        os.replace(temporary_name, path)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def open_stderr(path: Path) -> Any:
    """Open owned stderr state without following a symlink or special file."""
    ensure_not_symlink(path, "capture stderr")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail("cannot safely open capture stderr without O_NOFOLLOW")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | nofollow
    try:
        descriptor = os.open(path, flags, 0o600)
    except OSError as error:
        fail(f"cannot open capture stderr {path}: {error}")
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            fail(f"capture stderr is not a regular file: {path}")
        return os.fdopen(descriptor, "wb")
    except BaseException:
        os.close(descriptor)
        raise


def open_regular_read(path: str, label: str) -> tuple[int, os.stat_result]:
    """Open a regular file without following a symlink at the leaf."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        fail(f"cannot safely open {label} without O_NOFOLLOW")
    try:
        descriptor = os.open(path, os.O_RDONLY | nofollow)
        metadata = os.fstat(descriptor)
    except OSError as error:
        fail(f"cannot open {label} {path}: {error}")
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        fail(f"{label} is not a regular file: {path}")
    return descriptor, metadata


def command_stage_binary(args: argparse.Namespace) -> None:
    """Copy one verified executable into the private run directory."""
    source = Path(args.source)
    destination = Path(args.destination)
    ensure_regular(str(source), "capture binary")
    if not os.access(source, os.X_OK):
        fail(f"capture binary is not executable: {source}")
    ensure_not_symlink(destination, "capture executable")
    if destination.exists():
        fail(f"capture executable already exists: {destination}")
    if destination.parent.is_symlink() or not destination.parent.is_dir():
        fail(f"capture executable parent is not a private directory: {destination.parent}")

    descriptor, source_metadata = open_regular_read(str(source), "capture binary")
    temporary_name: str | None = None
    bytes_copied = 0
    try:
        digest = hashlib.sha256()
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            with tempfile.NamedTemporaryFile(
                mode="wb",
                dir=destination.parent,
                prefix=f".{destination.name}.",
                suffix=".tmp",
                delete=False,
            ) as output:
                temporary_name = output.name
                os.chmod(output.fileno(), 0o500)
                for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                    digest.update(chunk)
                    output.write(chunk)
                    bytes_copied += len(chunk)
                output.flush()
                os.fsync(output.fileno())
        if source_metadata.st_size == 0 or bytes_copied == 0:
            fail(f"capture binary is empty: {source}")
        ensure_not_symlink(destination, "capture executable")
        os.replace(temporary_name, destination)
        temporary_name = None
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass

    copied = file_info(str(destination))
    if copied.get("status") != "ok" or not copied.get("sha256"):
        fail(f"staged capture binary is not a non-empty regular file: {destination}")
    if copied["sha256"] != digest.hexdigest():
        fail("staged capture binary changed while it was being published")
    if not os.access(destination, os.X_OK):
        fail(f"staged capture binary is not executable: {destination}")


def update_manifest(manifest_path: Path, update: Any) -> None:
    """Run update(records) while holding a per-manifest advisory lock."""
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    ensure_not_symlink(manifest_path, "capture manifest")
    lock_path = manifest_path.with_name(f".{manifest_path.name}.lock")
    ensure_not_symlink(lock_path, "capture manifest lock")
    with lock_path.open("a+") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        if manifest_path.exists():
            loaded = load_json(manifest_path)
            if not isinstance(loaded, list):
                fail(f"capture provenance is not a JSON array: {manifest_path}")
            records = loaded
        else:
            records = []
        updated = update(records)
        write_json(manifest_path, updated)
        fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


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
    metadata = load_json(path)
    if not isinstance(metadata, dict):
        fail(f"capture metadata is not an object: {path}")
    if metadata.get("run_id") != args.run_id:
        fail("metadata run id does not match requested run id")
    metadata["session_id"] = args.session_id
    write_json(path, metadata)


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
    ensure_regular(binary, "capture binary")
    if not os.access(binary, os.X_OK):
        fail(f"capture binary is not executable: {binary}")
    current_binary = file_info(binary)
    if current_binary.get("status") != "ok" or current_binary.get("sha256") != expected_hash:
        fail("capture binary changed after provenance was initialized")

    color = metadata.get("color")
    if color != args.color:
        fail("capture color does not match the owning run metadata")
    expected_stderr = metadata.get("stderr")
    if expected_stderr != args.stderr:
        fail("capture stderr path does not match the owning run metadata")
    expected_exit = metadata_path.parent / "exit.status"
    if Path(args.exit) != expected_exit:
        fail("capture exit path does not belong to the owning run directory")

    if color not in {"truecolor", "256", "16", "mono"}:
        fail(f"unsupported capture color: {color!r}")
    environment = recorded_environment(metadata)
    if environment.get("COLOR") != color:
        fail("capture environment COLOR does not match the owning run metadata")

    stderr_path = Path(args.stderr)
    stderr = open_stderr(stderr_path)
    try:
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
            write_text_atomic(Path(args.exit), "127\n")
            return 127
        return_code = process.wait()
        write_text_atomic(Path(args.exit), f"{return_code}\n")
        return return_code if return_code >= 0 else 128 + (-return_code)
    finally:
        stderr.close()


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
        artifacts[key] = file_info(path)
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
        "stderr": file_info(args.stderr),
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
    metadata = load_json(metadata_path)
    if not isinstance(metadata, dict) or metadata.get("run_id") != args.run_id:
        fail("capture metadata does not match requested run")
    parsed = parse_exit(args.exit_status)
    metadata["status"] = "finalized"
    metadata["exit_status"] = parsed
    metadata["exit_observed"] = isinstance(parsed, int)
    metadata["termination"] = "natural_exit" if isinstance(parsed, int) else "capture_stop"
    metadata["finalized_at"] = now()
    metadata["stderr_info"] = file_info(args.stderr)
    write_json(metadata_path, metadata)

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
            record["stderr"] = file_info(args.stderr)
            changed.append(record)
        return changed

    if manifest.exists():
        update_manifest(manifest, finalize)


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
