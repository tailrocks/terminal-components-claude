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

SUPERSEDED_PINS_SHA256 = '2e8245bb0cd3ae48c2b4a31f2203c7cc974d1de6ff15aa2bcae0456623cc1d60'
RETIRED_DOCUMENTATION_SHA256 = {'README.md': 'b6103126b99e0b7b88f6502dadf80556c842ea8078899e76c3da005b8bae17ce',
 'proof/REPORT.md': '8aab52a0cc8abfd75879177d20291b66ec31e35387cb2c4cd3022e3b4591759b',
 'proof/cursor-independent-review/REVIEW.md': '4192f7fbbcaf014f785615c3171c3edae8a35f768e2ee90bcc062ddf855e7b95',
 'proof/cursor-review.md': '4192f7fbbcaf014f785615c3171c3edae8a35f768e2ee90bcc062ddf855e7b95',
 'proof/independent-review/REVIEW.md': 'c3612b4e11a49227e1180637d8a80f9f2a5842beb997f2fb5bbbc62575702d4e',
 'proof/migration-review.md': 'c3612b4e11a49227e1180637d8a80f9f2a5842beb997f2fb5bbbc62575702d4e'}
OPERATIONAL_PAYLOADS = frozenset(('acquire.py',
 'expected-cells.json',
 'fixture.py',
 'harness/Cargo.lock',
 'harness/main.rs',
 'licenses/tui-snap-LICENSE',
 'licenses/tui-test-LICENSE',
 'licenses/vt100-LICENSE',
 'oracle.py',
 'proof/applied-review-verification.json',
 'proof/clean-acquisition.json',
 'proof/clean-metadata.json',
 'proof/cursor-independent-review/SHA256SUMS',
 'proof/cursor-review.sha256',
 'proof/exact-provenance.json',
 'proof/final-test-identities.json',
 'proof/final-tool-gates.json',
 'proof/full-0.log',
 'proof/full-1.log',
 'proof/full-2.log',
 'proof/full-3.log',
 'proof/full-4.log',
 'proof/full-5.log',
 'proof/independent-review/verified.json',
 'proof/matrix-status.json',
 'proof/migration-reviewed-hashes.json',
 'proof/recorded-environment.json',
 'proof/resolved-tools.json',
 'proof/run_final_gates.py',
 'proof/run_repaired.py',
 'proof/source-artifacts.json',
 'proof/test-identities.json',
 'repairs/tuisnap-code.patch',
 'repairs/tuisnap.bundle',
 'repairs/vt100-upstream.json',
 'tests/test_integrity.py',
 'tuitest_smoke.py'))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_payloads(package=PACKAGE):
    pins = json.loads((package / "pins.json").read_text())
    if pins.get("schema") != 2:
        raise RuntimeError("Unsupported qualified payload schema")
    if (pins.get("supersedes_pins_sha256") != SUPERSEDED_PINS_SHA256
            or pins.get("retired_documentation_sha256") != RETIRED_DOCUMENTATION_SHA256):
        raise RuntimeError("Invalid qualified documentation retirement")
    if set(pins.get("payload_sha256", {})) != OPERATIONAL_PAYLOADS:
        raise RuntimeError("Qualified operational payload membership changed")
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
