#!/usr/bin/env python3
"""ADJ-13 finite, protected AST corpus; never a production checker or receipt.

--self-test proves source/AST premises only. TASK072 must run --runner against
the submitted checker; expected verdicts never enter its visible profile.
"""
from __future__ import annotations
import argparse
from contextlib import contextmanager
import importlib.util
import json
import os
from pathlib import Path
import secrets
import sys
import tempfile

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("broker_source_transport", HERE / "architecture-bootstrap-source-driver.py")
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)
actual, runner = source.actual, source.runner
require, sha = runner.require, runner.sha
SESSION = "crates/tui/src/runtime/session.rs"
OTHER = "crates/tui/src/runtime/broker_qualification.rs"
GUARD = '#[cfg(all(unix, feature = "crossterm"))]\n'
STORAGE = 'static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>> = std::sync::OnceLock::new();\n'
FIELDS = '''struct SignalBroker {
    inactive: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pending: std::sync::Arc<std::sync::atomic::AtomicBool>,
    leased: bool,
    inactive_registration: Option<signal_hook::SigId>,
    pending_registration: Option<signal_hook::SigId>,
}
'''
BROKER = GUARD + STORAGE + GUARD + FIELDS


def cases(original):
    core = {path: data for path, data in original.items() if path.startswith("crates/tui/src/") and path.endswith(".rs")}
    require(SESSION in core and "crates/tui/src/lib.rs" in core, "complete pinned core unavailable")
    rows = []

    def add(name, suffix=BROKER, *, negative=False, path=SESSION, extra=None, parse_error=False):
        files = dict(core)
        files[path] = files.get(path, b"") + b"\n" + suffix.encode()
        if extra:
            for location, data in extra.items():
                files[location] = files.get(location, b"") + b"\n" + data.encode()
        rows.append({"name": name, "source": files, "category": "ARCHITECTURE" if negative else None,
                     "parse_error": parse_error, "mutated_path": path})

    add("immutable-pinned-core", "")
    add("qualified-storage")
    add("split-effective-guard", BROKER.replace(GUARD, '#[cfg(unix)]\n#[cfg(feature="crossterm")]\n'))
    add("inherited-module-guard", GUARD + "mod broker_backend {\n" + STORAGE + FIELDS + "}\n")
    add("cfg-attr-equivalent", BROKER.replace(GUARD, '#[cfg(unix)]\n#[cfg_attr(unix, cfg(feature="crossterm"))]\n'))
    add("import-renames", 'use std::sync::{OnceLock as BrokerCell, Mutex as BrokerLock};\n' + BROKER.replace("std::sync::OnceLock", "BrokerCell").replace("std::sync::Mutex", "BrokerLock"))
    add("storage-type-alias", GUARD + 'type BrokerStorage = std::sync::OnceLock<std::sync::Mutex<SignalBroker>>;\n' + BROKER.replace("static SIGNAL_BROKER: std::sync::OnceLock<std::sync::Mutex<SignalBroker>>", "static SIGNAL_BROKER: BrokerStorage"))
    add("comment-and-string-no-global", '// static MUTEX: Mutex<State>; thread_local!{}\nconst NOTE: &str = "OnceLock static mut";\n')
    for name, suffix in [
        ("second-singleton", BROKER + GUARD + STORAGE.replace("SIGNAL_BROKER", "SECOND_BROKER")),
        ("renamed-static-mutex", BROKER + 'static HIDDEN: std::sync::Mutex<bool> = std::sync::Mutex::new(false);\n'),
        ("renamed-static-atomic", BROKER + 'static HIDDEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);\n'),
        ("alias-hidden-mutex", BROKER + 'type Hidden = std::sync::Mutex<bool>; static CACHE: Hidden = std::sync::Mutex::new(false);\n'),
        ("alias-hidden-atomic", BROKER + 'use std::sync::atomic::AtomicBool as Flag; static CACHE: Flag = Flag::new(false);\n'),
        ("alias-cycle-unresolved", BROKER + 'type Hidden = Other; type Other = Hidden; static CACHE: Hidden = todo!();\n'),
        ("unresolved-static-type", BROKER + 'static CACHE: Unresolved = unresolved!();\n'),
        ("runtime-field", BROKER.replace("leased: bool,", "leased: bool, runtime: crate::Runtime,")),
        ("theme-field", BROKER.replace("leased: bool,", "leased: bool, theme: crate::Theme,")),
        ("cache-field", BROKER.replace("leased: bool,", "leased: bool, cache: Vec<u8>,")),
        ("extra-flag", BROKER.replace("leased: bool,", "leased: bool, extra: std::sync::Arc<std::sync::atomic::AtomicBool>,")),
        ("extra-registration", BROKER.replace("leased: bool,", "leased: bool, third: Option<signal_hook::SigId>,")),
        ("public-static", BROKER.replace("static SIGNAL_BROKER", "pub static SIGNAL_BROKER")),
        ("crate-visible-static", BROKER.replace("static SIGNAL_BROKER", "pub(crate) static SIGNAL_BROKER")),
        ("public-broker-type", BROKER.replace("struct SignalBroker", "pub struct SignalBroker")),
        ("public-accessor", BROKER + GUARD + "pub fn signal_broker() -> &'static std::sync::OnceLock<std::sync::Mutex<SignalBroker>> { &SIGNAL_BROKER }\n"),
        ("public-reexport", BROKER + GUARD + 'pub use SIGNAL_BROKER as exported_broker;\n'),
        ("missing-unix", BROKER.replace(GUARD, '#[cfg(feature="crossterm")]\n')),
        ("missing-feature", BROKER.replace(GUARD, '#[cfg(unix)]\n')),
        ("no-guard", BROKER.replace(GUARD, "")),
        ("either-guard", BROKER.replace('all(unix, feature = "crossterm")', 'any(unix, feature = "crossterm")')),
        ("wrong-feature", BROKER.replace('feature = "crossterm"', 'feature = "testing"')),
        ("cfg-attr-leaks-nonunix", BROKER.replace(GUARD, '#[cfg_attr(unix, cfg(feature="crossterm"))]\n')),
        ("broker-fields-missing-guard", GUARD + STORAGE + FIELDS),
        ("function-local-storage", GUARD + 'fn hidden() {\n' + STORAGE + '}\n' + GUARD + FIELDS),
        ("method-local-storage", GUARD + 'impl SignalBroker { fn hidden() {\n' + STORAGE + '} }\n' + GUARD + FIELDS),
        ("thread-local-equivalent", GUARD + 'std::thread_local! { static SIGNAL_BROKER: std::cell::RefCell<Option<SignalBroker>> = std::cell::RefCell::new(None); }\n' + GUARD + FIELDS),
        ("macro-hidden-global", BROKER + 'macro_rules! hidden { () => { static CACHE: std::sync::Mutex<bool> = std::sync::Mutex::new(false); }; } hidden!();\n'),
        ("mutable-static", BROKER + 'static mut CACHE: bool = false;\n'),
        ("lazylock", BROKER.replace("OnceLock", "LazyLock")),
        ("wrong-storage-lock", BROKER.replace("std::sync::Mutex", "std::sync::RwLock")),
        ("shadowed-std-path", 'mod std { pub mod sync { pub struct OnceLock<T>(T); pub struct Mutex<T>(T); } }\n' + BROKER),
        ("shadowed-import", 'struct OnceLock<T>(T);\n' + BROKER.replace("std::sync::OnceLock", "OnceLock")),
    ]:
        add(name, suffix, negative=True)
    add("wrong-source-path", negative=True, path=OTHER)
    add("second-other-file", negative=True, extra={OTHER: GUARD + STORAGE.replace("SIGNAL_BROKER", "SECOND_BROKER") + GUARD + FIELDS})
    add("malformed-claimed-source", BROKER + "fn broken( {\n", negative=True, parse_error=True)
    add("unreferenced-malformed-source", negative=True, extra={OTHER: "fn broken( {\n"}, parse_error=True)
    require(len({row["name"] for row in rows}) == len(rows), "duplicate broker case")
    require(len({sha(runner.canonical({path: sha(data) for path, data in row["source"].items()})) for row in rows}) == len(rows), "ineffective or duplicate source mutant")
    return rows


@contextmanager
def preparation():
    original = actual.main_sources(actual.frozen_archive())
    with tempfile.TemporaryDirectory(prefix="tc-architecture-main-", dir="/tmp") as directory:
        root, target = Path(directory) / "source", Path(directory) / "target"
        sources = dict(original)
        sources["xtask/src/bin/broker_observer.rs"] = (HERE / "broker-bootstrap-observer.rs").read_bytes()
        actual.write_sources(root, sources)
        identity = actual.compiler_identity()
        env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE="true")
        build = source.command([identity["cargo"]["path"], "build", "-p", "xtask", "--bin", "broker_observer", "--locked", "--offline", "--message-format=json"], root, env)
        require(build["exit"] == 0, "broker observer build failed: " + source.decode(build, "stderr").decode()[-3000:])
        artifacts = [row for line in source.decode(build, "stdout").splitlines() if (row := json.loads(line)).get("reason") == "compiler-artifact" and row.get("executable") and row["target"]["name"] == "broker_observer"]
        require(len(artifacts) == 1, "broker parser executable ambiguous")
        executable = Path(artifacts[0]["executable"]).read_bytes()
        rows = cases(original)
        for row in rows:
            row.update(kind="source-policy", files={}, toolchain=identity, observer=executable,
                observer_sha256=sha(executable), observer_build=build, observed_paths=sorted(row["source"]),
                policy_profile={"policy": "ADJ-13-private-unix-signal-broker/v1", "source_roots": ["crates/tui/src"],
                    "required_files": sorted(row["source"]), "exception_path": SESSION, "exception_name": "SIGNAL_BROKER",
                    "storage": "std::sync::OnceLock<std::sync::Mutex<SignalBroker>>", "visibility": "private",
                    "effective_cfg": "all(unix,feature=crossterm)",
                    "bounded_fields": "only inactive/pending Arc<AtomicBool>, lease bool, two optional signal_hook::SigId/install phase",
                    "rules": "Parse every required file and resolve effective modules/cfg/type aliases/imports. At most one exact broker; immutable constant data permitted. Reject all other process-global mutable state, public accessors/reexports, function/thread-local equivalents, unresolved/macro-hidden mutable state. No whole-file exemption.",
                    "proof_boundary": "finite AST/source policy only; backend-free compile/dependency and runtime lease/registration proof remain separate mandatory gates"})
        yield rows


def observe(row):
    fixture = source.SourceFixture(row)
    try:
        source.measured_premise(fixture)
        record = fixture.events[0]["payload"]["records"][0]
        payload = runner.load_bytes(source.decode(record, "stdout"))
        require(any("parse_error" in entry for entry in payload["files"]) == row["parse_error"], "broker parser premise differs: " + row["name"])
        return sha(runner.canonical(payload))
    finally:
        fixture.cleanup()


def transport_self_test(rows):
    """Real broker parser IPC, not a reference broker-policy implementation."""
    results = {}
    for mode in ("positive", "always-pass", "always-reject", "zero", "forged", "forged-digest", "bool", "malformed", "duplicate", "recovery"):
        name = "second-singleton" if mode == "always-pass" else "qualified-storage"
        fixture = source.SourceFixture(next(row for row in rows if row["name"] == name))
        try:
            candidate = fixture.public / "broker-transport-only"
            program = "#!" + sys.executable + "\nimport json,os,hashlib\n"
            if mode == "zero":
                program += "events=[]\n"
            else:
                program += (
                    "request={'schema':'tc-proof-runner-observe/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'operation':'architecture','source_commit':os.environ['TC_PROOF_ORACLE_COMMIT'],'tree':os.environ['TC_PROOF_SOURCE_TREE']}\n"
                    "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']),'w') as f: f.write(json.dumps(request)+'\\n'); f.flush()\n"
                    "with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as f: event=json.loads(f.readline())\n"
                    "events=[event]\n")
            if mode == "forged-digest":
                program += "event['payload']['records']=[]\n"
            program += (
                "digests=[hashlib.sha256(json.dumps(e,sort_keys=True,separators=(',',':')).encode()).hexdigest() for e in events]\n"
                "report={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'architecture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'passed','category':None,'observation_digests':digests,'outputs':{'observations':events}}\n")
            if mode == "forged":
                program += "event['payload']['records']=[]\n"
            elif mode == "bool":
                program += "event['exit']=False\n"
            elif mode == "always-reject":
                program += "report.update(status='rejected',category='ARCHITECTURE',outputs={})\n"
            if mode == "malformed":
                program += "with open(os.environ['TC_PROOF_RESULT'],'w') as f: f.write('{')\n"
            elif mode == "duplicate":
                program += "with open(os.environ['TC_PROOF_RESULT'],'w') as f: f.write(json.dumps(report)[:-1]+',\\\"status\\\":\\\"passed\\\"}')\n"
            else:
                program += "with open(os.environ['TC_PROOF_RESULT'],'w') as f: json.dump(report,f)\n"
            if mode == "always-reject":
                program += "raise SystemExit(1)\n"
            candidate.write_text(program)
            candidate.chmod(0o755)
            expected = {"always-pass": "source negative did not reject exactly", "always-reject": "source positive rejected or forged",
                "zero": "source qualification omitted protected execution", "forged": "forged observation bytes", "forged-digest": "forged observation",
                "bool": "forged observation bytes", "malformed": "Expecting property name enclosed in double quotes", "duplicate": "duplicate JSON key"}
            try:
                fixture.validate(fixture.run(candidate), fixture.row["category"])
            except ValueError as error:
                require(mode in expected and str(error).startswith(expected[mode]), "broker transport rejected for unrelated reason: " + mode + ": " + str(error))
                require(len(fixture.events) == (0 if mode == "zero" else 1), "broker transport observed wrong execution count")
                results[mode] = "rejected: " + expected[mode]
            else:
                require(mode in {"positive", "recovery"}, "broker transport attack escaped: " + mode)
                results[mode] = "accepted"
        finally:
            fixture.cleanup()
    return results


def qualify(executable=None, transport_only=False):
    with preparation() as rows:
        if executable is None:
            observations = {} if transport_only else {row["name"]: observe(row) for row in rows}
            return {"schema": "tc-broker-corpus-premises/v1", "cases": len(rows), "negatives": sum(bool(row["category"]) for row in rows),
                "observed_ast_sha256": observations, "transport_controls": transport_self_test(rows), "submitted_checker_qualified": False,
                "required_next_command": "broker-bootstrap-driver.py --runner <submitted-dispatcher>"}
        frozen = sha(executable.read_bytes())
        recovery = next(row for row in rows if row["name"] == "qualified-storage")
        secrets.SystemRandom().shuffle(rows)
        for row in rows:
            for selected in [row] + ([recovery] if row["category"] else []):
                require(sha(executable.read_bytes()) == frozen, "submitted broker checker changed")
                fixture = source.SourceFixture(selected)
                try:
                    fixture.validate(fixture.run(executable), selected["category"])
                finally:
                    fixture.cleanup()
        return {"schema": "tc-broker-checker-qualification/v1", "cases": len(rows),
            "recoveries": sum(bool(row["category"]) for row in rows), "checker_sha256": frozen, "failures": []}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--transport-self-test", action="store_true")
    parser.add_argument("--runner", type=Path)
    args = parser.parse_args()
    require(sum((args.self_test, args.transport_self_test, bool(args.runner))) == 1, "choose one diagnostic OR mandatory submitted checker qualification")
    print(json.dumps(qualify(args.runner.resolve() if args.runner else None, args.transport_self_test), sort_keys=True))


if __name__ == "__main__":
    main()
