#!/usr/bin/env python3
"""Acquire exact reviewed capture sources, build outside the application, qualify."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import uuid

PACKAGE = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_payloads(package=PACKAGE):
    pins = json.loads((package / "pins.json").read_text())
    for name, expected in pins["payload_sha256"].items():
        path = package / name
        if not path.is_file() or digest(path) != expected:
            raise RuntimeError(f"Missing or tampered qualified payload: {name}")
    return pins


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True,
                        help="New external directory; never an application checkout")
    args = parser.parse_args()
    pins = validate_payloads()  # Fail before network, source mutation, or execution.
    root = args.root.resolve()
    repository = PACKAGE.parents[1]
    if root == repository or repository in root.parents:
        raise RuntimeError("Build root must be outside the application repository")
    root.mkdir(parents=True, exist_ok=False)
    log = root / "commands.json"
    result = {"status": "running", "pins_sha256": digest(PACKAGE / "pins.json"),
              "platform": platform.platform(), "commands": [], "sources": {}}
    env = os.environ.copy()
    removed = ["NO_COLOR", "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS",
               "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET",
               "CARGO_TARGET_DIR"]
    result["environment"] = {"removed_if_present": [k for k in removed if k in env],
                             "TERM": "xterm-256color", "COLORTERM": "truecolor"}
    for key in removed:
        env.pop(key, None)
    env.update(TERM="xterm-256color", COLORTERM="truecolor")

    def write():
        log.write_text(json.dumps(result, indent=2) + "\n")

    def run(argv, cwd=root, extra=None, timeout=900):
        command = [str(a) for a in argv]
        print("+", " ".join(command), flush=True)
        completed = subprocess.run(command, cwd=cwd, env=env | (extra or {}),
                                   capture_output=True, text=True, timeout=timeout)
        entry = {"argv": command, "cwd": str(cwd), "exit": completed.returncode,
                 "stdout": completed.stdout, "stderr": completed.stderr}
        result["commands"].append(entry)
        write()
        if completed.returncode:
            raise RuntimeError(json.dumps(entry, indent=2))
        return completed.stdout.strip()

    write()
    try:
        rust = pins["rust_toolchain"]
        result["rustc"] = run(["rustc", "+" + rust, "-Vv"])
        if f"release: {rust}\n" not in result["rustc"] + "\n":
            raise RuntimeError("Rust version differs from qualification pin")
        if pins["rust_commit"] not in result["rustc"]:
            raise RuntimeError("Rust compiler commit differs from qualification pin")
        result["cargo"] = run(["cargo", "+" + rust, "-V"])
        result["zig"] = run(["zig", "version"])
        if result["zig"] != pins["zig_version"]:
            raise RuntimeError("Zig version differs from qualification pin")
        result["git"] = run(["git", "--version"])
        result["python"] = sys.version
        for name, pin in pins["sources"].items():
            source = root / "sources" / name
            source.mkdir(parents=True)
            run(["git", "init", "--quiet"], source)
            run(["git", "remote", "add", "origin", pin["url"]], source)
            run(["git", "fetch", "--depth=1", "origin", pin["upstream_commit"]], source)
            run(["git", "checkout", "--detach", pin["upstream_commit"]], source)
            if run(["git", "rev-parse", "HEAD^{tree}"], source) != pin["upstream_tree"]:
                raise RuntimeError(f"Upstream tree mismatch: {name}")
            if "bundle" in pin:
                run(["git", "bundle", "verify", PACKAGE / pin["bundle"]], source)
                run(["git", "fetch", PACKAGE / pin["bundle"], pin["bundle_ref"]], source)
                run(["git", "checkout", "--detach", pin["qualified_commit"]], source)
            if run(["git", "rev-parse", "HEAD"], source) != pin["qualified_commit"]:
                raise RuntimeError(f"Qualified commit mismatch: {name}")
            if run(["git", "rev-parse", "HEAD^{tree}"], source) != pin["qualified_tree"]:
                raise RuntimeError(f"Qualified tree mismatch: {name}")
            if digest(source / "Cargo.lock") != pin["cargo_lock_sha256"]:
                raise RuntimeError(f"Cargo lock mismatch: {name}")
            target = root / "targets" / name
            run(["cargo", "+" + rust, "build", "--locked", *pin["build_args"]], source,
                {"CARGO_TARGET_DIR": str(target)})
            if run(["git", "status", "--porcelain", "--untracked-files=all"], source):
                raise RuntimeError(f"Build mutated source: {name}")
            binary = target / "debug" / pin["binary"]
            result["sources"][name] = {"commit": pin["qualified_commit"],
                                      "tree": pin["qualified_tree"],
                                      "binary": str(binary), "binary_sha256": digest(binary)}
        harness = root / "harness"
        harness.mkdir()
        shutil.copy(PACKAGE / "harness/main.rs", harness / "main.rs")
        shutil.copy(PACKAGE / "harness/Cargo.lock", harness / "Cargo.lock")
        (harness / "Cargo.toml").write_text('''[package]
name = "tool-qualification"
version = "0.1.0"
edition = "2021"
[[bin]]
name = "tool-qualification"
path = "main.rs"
[dependencies]
tuisnap = { path = "../sources/tui-snap" }
[patch.crates-io]
vt100 = { path = "../sources/tui-snap/vendor/vt100" }
''')
        run(["cargo", "+" + rust, "build", "--locked"], harness,
            {"CARGO_TARGET_DIR": str(root / "targets/harness")})
        artifacts = root / "runs" / uuid.uuid4().hex
        artifacts.mkdir(parents=True)
        shutil.copy(PACKAGE / "fixture.py", artifacts / "fixture.py")
        run([root / "targets/harness/debug/tool-qualification", artifacts])
        run([sys.executable, PACKAGE / "tuitest_smoke.py", artifacts,
             result["sources"]["tui-test"]["binary"]])
        run([sys.executable, PACKAGE / "oracle.py", artifacts, "--strict-snap"])
        validate_payloads()
        if digest(PACKAGE / "pins.json") != result["pins_sha256"]:
            raise RuntimeError("Qualification package changed during execution")
        result["artifacts"] = {str(p.relative_to(root)): digest(p)
                               for p in sorted(artifacts.iterdir()) if p.is_file()}
        result["harness_binary_sha256"] = digest(root / "targets/harness/debug/tool-qualification")
        result["status"] = "complete"
        write()
        print(f"Qualified tooling acquisition PASS: {log}")
    except BaseException as error:
        result["status"] = "failed"
        result["error"] = str(error)
        write()
        raise


if __name__ == "__main__":
    main()
