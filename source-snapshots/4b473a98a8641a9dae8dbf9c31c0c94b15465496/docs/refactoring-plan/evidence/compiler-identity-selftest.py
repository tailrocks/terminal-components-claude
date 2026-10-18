#!/usr/bin/env python3
"""Read-only host-tool identity regression checks; not TC product acceptance."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
from types import SimpleNamespace
from unittest.mock import patch


def main():
    here = Path(__file__).resolve().parent
    spec = importlib.util.spec_from_file_location("identity_main", here / "architecture-bootstrap-main-driver.py")
    actual = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(actual)
    identity = actual.compiler_identity()
    actual.validate_compiler_identity(identity)
    checks = []

    def reject(name, operation):
        try:
            operation()
        except ValueError:
            checks.append(name)
        else:
            raise RuntimeError("identity negative accepted: " + name)

    # Real resolved tools execute, not a fake executable reporting version data.
    for name, tool in identity.items():
        result = subprocess.run([tool["path"], "--version", "--verbose"], capture_output=True, text=True, check=True)
        actual.require(result.stdout.strip() == tool["version"], "actual version observation differs")
        actual.validate_compiler_identity(identity)
        checks.append("actual-" + name + "-invocation")
    for name in ("cargo", "rustc"):
        for field in ("path", "sha256", "version", "toolchain"):
            changed = {key: dict(value) for key, value in identity.items()}
            changed[name][field] += "-changed"
            reject(name + "-record-" + field, lambda: actual.validate_compiler_identity(changed))
    original_read = Path.read_bytes
    for name in ("cargo", "rustc"):
        target = Path(identity[name]["path"])
        def changed_bytes(path):
            return b"mutated compiler executable" if path == target else original_read(path)
        with patch.object(Path, "read_bytes", changed_bytes):
            reject(name + "-actual-bytes", lambda: actual.validate_compiler_identity(identity))
    original_version = actual.tool_version
    with patch.object(actual, "tool_version", lambda path: original_version(path) + " changed"):
        reject("actual-version", lambda: actual.validate_compiler_identity(identity))
    selected, paths = actual.selected_compiler_paths()
    with patch.object(actual, "selected_compiler_paths", lambda: (selected + "-changed", paths)):
        reject("selected-toolchain", lambda: actual.validate_compiler_identity(identity))
    with patch.object(actual, "selected_compiler_paths", lambda: (selected, {**paths, "cargo": paths["rustc"]})):
        reject("selected-cargo-path", lambda: actual.validate_compiler_identity(identity))
    with patch.object(actual.shutil, "which", return_value=None):
        reject("unresolved-toolchain", actual.selected_compiler_paths)
    returned = actual.compiler_identity()
    returned["cargo"]["path"] = "caller-mutated-copy"
    actual.require(actual.compiler_identity() == identity, "caller changed frozen identity")
    checks.append("caller-copy-isolated")
    env = actual.compiler_environment(identity, {"KEEP": "value", "RUSTC": "foreign", "RUSTC_WRAPPER": "foreign", "RUSTC_WORKSPACE_WRAPPER": "foreign"})
    actual.require(env == {"KEEP": "value", "RUSTC": identity["rustc"]["path"], "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": ""}, "compiler environment not pinned")
    checks.append("compiler-environment-pinned")

    # Exercise the actual source-wrapper boundary without invoking a TC build.
    spec = importlib.util.spec_from_file_location("identity_source", here / "architecture-bootstrap-source-driver.py")
    source = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source)
    source_identity = source.actual.compiler_identity()
    calls = []
    validate = source.actual.validate_compiler_identity
    def record_validation(value):
        calls.append("validated")
        return validate(value)
    with patch.object(source.actual, "validate_compiler_identity", record_validation):
        record = source.command([source_identity["cargo"]["path"], "--version", "--verbose"], here)
    actual.require(record["exit"] == 0 and len(calls) >= 3, "source wrapper omitted before/after identity checks")
    checks.append("source-wrapper-pre-post")
    for name, module, operation in (
        ("source-post-invocation-change", source.actual,
         lambda: source.command([source_identity["cargo"]["path"], "--version", "--verbose"], here)),
        ("main-post-build-change", actual,
         lambda: actual.compile_execute(here, here, "unused", "unused", None)),
    ):
        completed = [False]
        original_run = subprocess.run
        target = Path(module.compiler_identity()["cargo"]["path"])
        def changed_after_run(*args, **kwargs):
            if kwargs.get("timeout") == 300:
                completed[0] = True
                return SimpleNamespace(stdout=b"", stderr=b"", returncode=0)
            return original_run(*args, **kwargs)
        def bytes_after_run(path):
            return b"changed during invocation" if completed[0] and path == target else original_read(path)
        with patch.object(subprocess, "run", changed_after_run), patch.object(Path, "read_bytes", bytes_after_run):
            try:
                operation()
            except ValueError as error:
                actual.require("compiler bytes changed" in str(error), "unrelated post-invocation rejection")
                checks.append(name)
            else:
                raise RuntimeError("post-invocation identity change accepted: " + name)
    print(json.dumps({"schema": "tc-compiler-identity-selftest/v1", "checks": checks,
                      "count": len(checks), "identity": identity}, indent=2))


if __name__ == "__main__":
    main()
