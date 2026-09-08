#!/usr/bin/env python3
"""Exact Cargo/libtest inventory. Capture is evidence, never approval."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib


class Invalid(Exception):
    pass


def require(ok, message):
    if not ok:
        raise Invalid(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def unique(values, label):
    require(len(values) == len(set(values)), "duplicate " + label)


def listed(text):
    names = []
    totals = []
    for line in text.splitlines():
        match = re.fullmatch(r"(.+): test", line)
        if match:
            names.append(match[1])
        elif match := re.fullmatch(r"(\d+) tests?, (\d+) benchmarks?", line):
            require(int(match[2]) == 0, "benchmark harness requires separate coverage")
            totals.append(int(match[1]))
        elif line.strip():
            raise Invalid("unexpected libtest listing output")
    unique(names, "listed test identity")
    require(totals and sum(totals) == len(names), "listing count mismatch")
    return sorted(names)


def executed(text, names, status_log=None):
    results = {}
    if status_log is not None:
        for line in status_log.splitlines():
            match = re.fullmatch(r"(ok|failed|ignored(?::[^\r\n]*)?) (\S+)", line)
            require(match is not None, "unexpected libtest status log")
            status, name = match.groups()
            status = status.split(":", 1)[0]
            require(name in names and name not in results, "unknown/duplicate logged identity")
            results[name] = "FAILED" if status == "failed" else status
    totals = [0] * 5
    summaries = 0
    for line in text.splitlines():
        match = re.fullmatch(r"test (.+) \.\.\. (ok|FAILED|ignored)(?:, .*)?", line)
        if match and status_log is None:
            name = match[1]
            # rustdoc execution appends a mode absent from its --list identity.
            if name not in names:
                name = re.sub(r" - (?:compile fail|should panic)$", "", name)
            require(name in names and name not in results, "unknown/duplicate executed identity")
            results[name] = match[2]
        elif match := re.fullmatch(
            r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
            r"(\d+) measured; (\d+) filtered out; finished in .+", line
        ):
            summaries += 1
            totals = [a + int(b) for a, b in zip(totals, match.groups())]
    require(summaries > 0, "missing execution summary")
    require(totals[3:] == [0, 0], "measured/filtered tests cannot satisfy execution")
    require(set(results) == set(names), "listed/executed identities differ")
    require(totals[:3] == [sum(v == s for v in results.values()) for s in ("ok", "FAILED", "ignored")],
            "execution counts differ from identities")
    return results


def safe_env():
    # Do not pass credentials, fixture content, baseline blessing, or arbitrary
    # compiler/test flags. Record only the fingerprint, never these values.
    allowed = ("PATH", "HOME", "TMPDIR", "RUSTUP_HOME", "CARGO_HOME", "RUSTUP_TOOLCHAIN",
               "SYSTEMROOT", "USERPROFILE", "SSL_CERT_FILE", "SSL_CERT_DIR")
    env = {key: os.environ[key] for key in allowed if key in os.environ}
    env.update({"LC_ALL": "C", "CARGO_TERM_COLOR": "never", "RUST_BACKTRACE": "0"})
    return env


def source_fingerprint(root, env):
    run = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                         cwd=root, env=env, capture_output=True, check=True)
    paths = sorted(set(os.fsdecode(p) for p in run.stdout.split(b"\0") if p))
    files = [(p, digest((root / p).read_bytes())) for p in paths if (root / p).is_file()]
    require(files, "source fingerprint is empty")
    return digest(encoded(files))


def validate_profiles(profiles):
    require(profiles, "no feature profiles")
    unique([p["id"] for p in profiles], "profile ID")
    unique([encoded({k: v for k, v in p.items() if k != "id"}) for p in profiles], "profile invocation")
    for p in profiles:
        require(set(p) == {"id", "package", "default_features", "features", "all_features"},
                "profile requires explicit package and feature configuration")
        require(isinstance(p["default_features"], bool) and isinstance(p["all_features"], bool),
                "invalid feature switch")
        require(not (p["all_features"] and p["features"]), "ambiguous all/features profile")
        unique(p["features"], "feature")


def target_inventory(package, profile, messages, custom):
    own = [m for m in messages if m.get("reason") == "compiler-artifact" and
           m.get("package_id") == package["id"] and m["target"]["kind"] != ["custom-build"]]
    feature_sets = {tuple(sorted(m["features"])) for m in own}
    require(len(feature_sets) <= 1, "ambiguous package feature sets in one invocation")
    if feature_sets:
        active = set(next(iter(feature_sets)))
        feature_source = "compiler-artifact"
    else:
        # An entirely gated package can emit no artifacts. There are then no
        # compiled dev-dependencies to unify features back into this package.
        # Resolve only its declared local feature closure; unsupported qualified
        # selectors fail rather than inventing a workspace-wide resolution.
        declared = package["features"]
        active = set(declared) if profile["all_features"] else set(profile["features"])
        if profile["default_features"] and "default" in declared:
            active.add("default")
        require(active <= declared.keys(), "cannot resolve artifact-free qualified feature selector")
        pending = list(active)
        while pending:
            for feature in declared[pending.pop()]:
                if feature in declared and feature not in active:
                    active.add(feature)
                    pending.append(feature)
        feature_source = "artifact-free local feature closure"
    compiled_tests = {(m["target"]["kind"][0], m["target"]["name"]) for m in own
                      if m.get("profile", {}).get("test") and m.get("executable")}
    expected, classifications, blockers = set(), [], []
    for target in package["targets"]:
        kind = target["kind"][0]
        key = (kind, target["name"])
        required = set(target.get("required-features", []))
        reason = None
        if not required <= active:
            reason = "inactive required-features"
        elif key in custom:
            reason = "custom harness unsupported: explicit adapter required"
        elif kind == "bench":
            reason = "benchmark target requires separate benchmark coverage"
        elif not target["test"] and key not in compiled_tests:
            reason = "Cargo test=false"
        # --all-targets emits runnable example test harnesses even though Cargo
        # metadata reports test=false by default. Actual compiler artifacts own
        # execution; the metadata flag cannot erase those test identities.
        if reason is None:
            expected.add(key)
        elif reason in ("custom harness unsupported: explicit adapter required",
                        "benchmark target requires separate benchmark coverage"):
            blockers.append({"profile": profile["id"], "target": list(key), "reason": reason})
        classifications.append({"profile": profile["id"], "target": list(key),
                                "classification": reason or "libtest required",
                                "required_features": sorted(required), "active_features": sorted(active),
                                "feature_source": feature_source})
    artifacts = {}
    for message in own:
        if not message.get("profile", {}).get("test") or not message.get("executable"):
            continue
        key = (message["target"]["kind"][0], message["target"]["name"])
        if key in expected:
            require(key not in artifacts, "duplicate Cargo test target artifact")
            artifacts[key] = message
    for key in sorted(expected - artifacts.keys()):
        blockers.append({"profile": profile["id"], "target": list(key),
                         "reason": "required target absent from this feature build"})
    return artifacts, classifications, blockers


def capture(root, profiles, output, execute, toolchain):
    require(not output.exists(), "refuse existing output")
    require(not output.resolve().is_relative_to(root.resolve()), "evidence must be outside source tree")
    validate_profiles(profiles)
    env = safe_env()
    commands = []
    cargo = ["cargo"] + (["+" + toolchain] if toolchain else [])

    def run(args, cwd=root):
        result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, timeout=1200)
        commands.append({"argv": args, "cwd": str(cwd.resolve()), "exit": result.returncode,
                         "stdout_sha256": digest(result.stdout.encode()),
                         "stderr_sha256": digest(result.stderr.encode())})
        return result

    source = source_fingerprint(root, env)
    lock = digest((root / "Cargo.lock").read_bytes())
    version = run(cargo + ["--version"])
    require(version.returncode == 0, "Cargo version failed")
    rust = run(["rustc"] + (["+" + toolchain] if toolchain else []) + ["-vV"])
    require(rust.returncode == 0, "Rust version failed")
    metadata = run(cargo + ["metadata", "--locked", "--no-deps", "--format-version", "1"])
    require(metadata.returncode == 0, "Cargo metadata failed (diagnostic hash recorded only)")
    meta = json.loads(metadata.stdout)
    packages = {p["name"]: p for p in meta["packages"] if p["id"] in meta["workspace_members"]}
    records = []
    blocked = []
    classifications = []
    with tempfile.TemporaryDirectory(prefix="test-inventory-") as scratch:
        for index, profile in enumerate(profiles):
            package = packages.get(profile["package"])
            require(package is not None, "profile package absent from workspace")
            package_cwd = Path(package["manifest_path"]).parent.resolve()
            flags = ["-p", package["name"]]
            if not profile["default_features"]:
                flags += ["--no-default-features"]
            if profile["all_features"]:
                flags += ["--all-features"]
            if profile["features"]:
                flags += ["--features", ",".join(profile["features"])]
            # A new target directory per profile prevents feature-unified or
            # previous-command executable reuse. Only this command's JSON is used.
            flags += ["--target-dir", str(Path(scratch) / str(index))]
            cargo_manifest = tomllib.loads(Path(package["manifest_path"]).read_text())
            custom = set()
            for kind in ("lib", "bin", "test", "bench", "example"):
                entries = cargo_manifest.get(kind, [])
                if isinstance(entries, dict):
                    entries = [entries]
                for entry in entries:
                    if entry.get("harness") is False:
                        custom.add((kind, entry.get("name", package["name"].replace("-", "_"))))
            built = run(cargo + ["test", "--locked", "--no-run", "--all-targets",
                                 "--message-format=json"] + flags)
            if built.returncode:
                blocked.append({"profile": profile["id"], "reason": "Cargo test compilation failed"})
                continue
            artifacts, target_classes, target_blockers = target_inventory(
                package, profile, [json.loads(line) for line in built.stdout.splitlines()], custom)
            classifications.extend(target_classes)
            blocked.extend(target_blockers)
            for key, artifact in sorted(artifacts.items()):
                exe = Path(artifact["executable"])
                require(exe.resolve().is_relative_to(Path(scratch).resolve()), "artifact outside isolated build")
                sha = digest(exe.read_bytes())
                selector = ["--lib"] if key[0] == "lib" else ["--" + key[0], key[1]]
                execution_command = cargo + ["test", "--locked"] + flags + selector
                listing = run(execution_command + ["--", "--list", "--format", "pretty"], package_cwd)
                require(listing.returncode == 0, "libtest listing failed")
                names = listed(listing.stdout)
                statuses = None
                status_sha256 = None
                if execute:
                    # The dedicated libtest status file cannot be interleaved by
                    # subprocess stdout (which can split pretty result lines).
                    status_path = Path(scratch) / f"statuses-{index}-{len(records)}.txt"
                    require(not status_path.exists(), "refuse existing libtest status log")
                    result = run(execution_command + ["--", "--format", "pretty", "--test-threads=1",
                                                      "--logfile", str(status_path)], package_cwd)
                    require(status_path.is_file() and not status_path.is_symlink(),
                            "missing/unsafe libtest status log")
                    status_bytes = status_path.read_bytes()
                    status_sha256 = digest(status_bytes)
                    statuses = executed(result.stdout, names, status_bytes.decode())
                    require((result.returncode == 0) == all(v != "FAILED" for v in statuses.values()),
                            "execution exit status inconsistent")
                require(digest(exe.read_bytes()) == sha, "executable changed during execution")
                records.append({"profile": profile["id"], "package": package["name"],
                                "kind": key[0], "target": key[1], "features": artifact["features"],
                                "executable_sha256": sha, "cwd": str(package_cwd),
                                "status_log_sha256": status_sha256,
                                "listed": names, "executed": statuses})
            for target in package["targets"]:
                if not target["doctest"]:
                    continue
                require(target["kind"] == ["lib"], "unsupported rustdoc target kind")
                listing = run(cargo + ["test", "--locked", "--doc"] + flags + ["--", "--list"], package_cwd)
                if listing.returncode:
                    blocked.append({"profile": profile["id"], "reason": "rustdoc listing failed"})
                    continue
                names = listed(listing.stdout)
                statuses = None
                if execute:
                    result = run(cargo + ["test", "--locked", "--doc"] + flags + ["--", "--test-threads=1"], package_cwd)
                    statuses = executed(result.stdout, names)
                    require((result.returncode == 0) == all(v != "FAILED" for v in statuses.values()),
                            "rustdoc exit status inconsistent")
                records.append({"profile": profile["id"], "package": package["name"],
                                "kind": "doc", "target": target["name"], "cwd": str(package_cwd), "listed": names,
                                "executed": statuses})
    require(source_fingerprint(root, env) == source, "source changed during capture")
    require(digest((root / "Cargo.lock").read_bytes()) == lock, "lock changed during capture")
    result = {"schema": 1, "classification": "captured-not-approved", "source_sha256": source,
              "lock_sha256": lock, "environment_sha256": digest(encoded(env)),
              "cargo": version.stdout.strip(), "rustc": rust.stdout.strip(),
              "profiles": profiles, "targets": records, "blocked": blocked,
              "classifications": classifications, "commands": commands}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    return result


def verify(captured, required, catalog):
    require(captured["schema"] == required["schema"] == 1, "unknown schema")
    require(not captured["blocked"], "capture has blocked target coverage")
    require(required.get("approval") == "reviewed", "required identities have not been reviewed")
    require(required["profiles"] == captured["profiles"], "feature profile matrix differs")
    records = captured["targets"]
    key = lambda x: (x["profile"], x["package"], x["kind"], x["target"])
    unique([key(x) for x in records], "captured target")
    unique([key(x) for x in required["targets"]], "required target")
    actual = {key(x): x for x in records}
    require(set(actual) == {key(x) for x in required["targets"]}, "required target matrix differs")
    identities = set()
    for target in required["targets"]:
        row = actual[key(target)]
        require(isinstance(row.get("cwd"), str) and Path(row["cwd"]).is_absolute(),
                "missing package execution cwd")
        if row["kind"] != "doc":
            require(re.fullmatch(r"[0-9a-f]{64}", row.get("status_log_sha256") or "") is not None,
                    "missing dedicated libtest status attestation")
        unique(target["identities"], "required test identity")
        require(target["identities"] or target.get("empty_reason"), "empty target lacks explicit review")
        require(set(target["identities"]) == set(row["listed"]), "required/listed identities differ")
        require(row["executed"] is not None, "listing alone is not execution proof")
        require(set(row["executed"]) == set(row["listed"]), "executed coverage differs")
        exclusions = target.get("ignored", {})
        require(set(exclusions) <= set(row["listed"]) and all(exclusions.values()), "invalid ignored mapping")
        for name, status in row["executed"].items():
            require(status == "ok" or (status == "ignored" and name in exclusions), "required test did not pass")
            identities.add((*key(target), name))
    obligations = required["obligations"]
    require(obligations, "no historical obligation mapping")
    unique([x["source_id"] for x in obligations], "historical obligation")
    require(required["catalog_sha256"] == digest(encoded(catalog)), "historical catalog fingerprint differs")
    require({x["source_id"] for x in obligations} == {x["source_id"] for x in catalog["obligations"]},
            "historical obligation omitted or invented")
    for obligation in obligations:
        require(obligation.get("review") and obligation.get("destinations"), "unresolved historical obligation")
        for destination in obligation["destinations"]:
            require(tuple(destination) in identities, "historical destination absent")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    cap = sub.add_parser("capture")
    cap.add_argument("--root", type=Path, required=True)
    cap.add_argument("--profiles", type=Path, required=True)
    cap.add_argument("--output", type=Path, required=True)
    cap.add_argument("--execute", action="store_true")
    cap.add_argument("--toolchain")
    check = sub.add_parser("verify")
    check.add_argument("--capture", type=Path, required=True)
    check.add_argument("--required", type=Path, required=True)
    check.add_argument("--catalog", type=Path, default=Path(__file__).with_name("historical.json"))
    check.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "capture":
            result = capture(args.root.resolve(), json.loads(args.profiles.read_text()),
                             args.output, args.execute, args.toolchain)
            print(f"captured {len(result['targets'])} targets; {len(result['blocked'])} blockers; not approved")
        else:
            captured = json.loads(args.capture.read_text())
            require(captured["source_sha256"] == source_fingerprint(args.root.resolve(), safe_env()),
                    "capture does not match current source")
            require(captured["lock_sha256"] == digest((args.root / "Cargo.lock").read_bytes()),
                    "capture does not match current lock")
            verify(captured, json.loads(args.required.read_text()), json.loads(args.catalog.read_text()))
            print("required identities executed; historical mappings resolved")
    except (Invalid, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        # Never print subprocess output or fixture payloads.
        print("inventory failed: " + (str(error) if isinstance(error, Invalid) else type(error).__name__))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
