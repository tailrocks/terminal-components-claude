#!/usr/bin/env python3
"""Independent real-PTY atomic capture qualification; not a TC application."""
from __future__ import annotations

import argparse
import base64
import copy
import fcntl
import importlib.util
import inspect
import io
import json
import os
from pathlib import Path
import pty
import secrets
import select
import shutil
import struct
import subprocess
import sys
import tarfile
import tempfile
import termios
import time
import tty

HERE = Path(__file__).resolve().parent
PIN = "883d03f19d890bbbf27468798db78b04e85297ac"
TREE = "dadbaa70facc317cfabb52f0374c1f3cdceb46a1"
PRODUCER = "capture-atomicity-bootstrap-producer.rs"
DECODER = "capture-atomicity-bootstrap-decoder.rs"


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, HERE / filename)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


runner = module("atomic_runner", "runner-bootstrap-driver.py")
compiler = module("atomic_compiler", "architecture-bootstrap-main-driver.py")
observer = module("atomic_observer", "host-bootstrap-observer.py")
require, canonical, sha = runner.require, runner.canonical, runner.sha
NEGATIVES = [f"mixed-{mask:03b}" for mask in range(1, 7)] + [
    "foreign-complete", "text-other", "ansi-other", "html-other", "second-live-read",
    "wrong-action", "wrong-tick", "stale-generation", "missing-checkpoint", "duplicate-checkpoint",
    "reordered-checkpoints", "raw-other", "state-bool", "state-float",
]


def source_archive(path):
    require(path.is_dir(), "missing protected tuisnap source")
    require(runner.git(path, "rev-parse", PIN + "^{tree}") == TREE, "tuisnap pinned tree mismatch")
    archive = subprocess.check_output(["/usr/bin/git", "-C", str(path), "archive", "--format=tar", PIN])
    result = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        for member in tar.getmembers():
            relative = Path(member.name)
            require(not relative.is_absolute() and ".." not in relative.parts, "unsafe tuisnap source path")
            if member.isdir():
                continue
            require(member.isfile(), "nonregular tuisnap source input")
            result[member.name] = tar.extractfile(member).read()
    require("Cargo.lock" in result and "src/ansi.rs" in result and "vendor/vt100/src/parser.rs" in result,
            "incomplete pinned decoder source")
    return archive, result


def prepare(tuisnap_source):
    archive, sources = source_archive(tuisnap_source)
    temporary = tempfile.TemporaryDirectory(prefix="tc-atomic-build-", dir="/tmp")
    root = Path(temporary.name).resolve()
    source, target = root / "source", root / "target"
    for name, data in sources.items():
        destination = source / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
    producer_source = (HERE / PRODUCER).read_bytes()
    decoder_source = (HERE / DECODER).read_bytes()
    (source / "src/bin").mkdir(exist_ok=True)
    (source / "src/bin/atomic-decoder.rs").write_bytes(decoder_source)
    (root / "producer.rs").write_bytes(producer_source)
    identity = compiler.compiler_identity()
    env = compiler.compiler_environment(identity)
    env.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL="0")
    compiler.validate_compiler_identity(identity)
    commands = [
        [identity["rustc"]["path"], "--edition=2024", "--deny=warnings", str(root / "producer.rs"), "-o", str(root / "producer")],
        [identity["cargo"]["path"], "build", "--locked", "--offline", "--bin", "atomic-decoder", "--message-format=json"],
    ]
    records = []
    for argv in commands:
        compiler.validate_compiler_identity(identity)
        result = subprocess.run(argv, cwd=source, env=env, capture_output=True, timeout=600, check=False)
        compiler.validate_compiler_identity(identity)
        require(result.returncode == 0, "atomic source compilation failed: " + result.stderr.decode()[-5000:])
        records.append({"argv": argv, "exit": result.returncode,
                        "stdout_sha256": sha(result.stdout), "stderr_sha256": sha(result.stderr)})
    for name, data in sources.items():
        require((source / name).read_bytes() == data, "pinned build source changed: " + name)
    require((source / "src/bin/atomic-decoder.rs").read_bytes() == decoder_source and
            (root / "producer.rs").read_bytes() == producer_source, "qualification source changed")
    binaries = {"producer": (root / "producer").read_bytes(), "decoder": (target / "debug/atomic-decoder").read_bytes()}
    compilation = {"tuisnap_commit": PIN, "tuisnap_tree": TREE, "archive_sha256": sha(archive),
                   "lock_sha256": sha(sources["Cargo.lock"]), "tools": identity, "commands": records,
                   "qualification_sources": {PRODUCER: sha(producer_source), DECODER: sha(decoder_source)},
                   "binaries": {name: sha(data) for name, data in binaries.items()}}
    temporary.cleanup()
    return {"archive": archive, "binaries": binaries, "compilation": compilation,
            "source": {PRODUCER: producer_source, DECODER: decoder_source}}


def seal(files, binding):
    return {"binding": copy.deepcopy(binding), "files": files,
            "manifest": [{"path": name, "size": len(data.encode()), "sha256": sha(data.encode()),
                          "generation": binding["generation"]} for name, data in sorted(files.items())]}


def capture(producer, decoder, directory, seed, tree, unreadable=()):
    nonce = secrets.token_hex(32)
    start = b"\x1b]777;" + nonce.encode() + b";"
    profile = observer.sandbox(writable=[], unreadable=list(unreadable), readable=[producer, decoder])
    master, slave = pty.openpty()
    tty.setraw(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 4, 24, 0, 0))
    process = subprocess.Popen(["/usr/bin/sandbox-exec", "-p", profile, str(producer), str(seed), nonce],
                               stdin=slave, stdout=slave, stderr=subprocess.PIPE,
                               env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"}, cwd=directory)
    os.close(slave)
    raw = b""
    references, receipts = [], []
    try:
        for ordinal in range(3):
            if ordinal:
                os.write(master, b"+")
            begin = len(raw)
            deadline = time.monotonic() + 10
            while b"\x07" not in raw[begin:]:
                require(time.monotonic() < deadline, "producer barrier timeout")
                if select.select([master], [], [], 0.1)[0]:
                    part = os.read(master, 65536)
                    require(bool(part) and len(raw) + len(part) <= 65536, "missing or oversized producer stream")
                    raw += part
            segment = raw[begin:]
            require(segment.count(start) == 1 and segment.endswith(b"\x07") and segment.count(b"\x07") == 1,
                    "unbound or repeated producer checkpoint")
            state = runner.load_bytes(segment.split(start, 1)[1][:-1])
            require(set(state) == {"generation", "action_index", "tick", "value", "phase"}, "producer state schema")
            require(all(type(state[name]) is int for name in ("generation", "action_index", "tick", "value")), "producer numeric types")
            require(state == {"generation": ordinal, "action_index": ordinal, "tick": seed * 7 + ordinal * 7,
                              "value": seed + ordinal, "phase": "A" if (seed + ordinal) % 2 == 0 else "B"},
                    "actual source update/extraction did not reach checkpoint")
            decoded = subprocess.run(["/usr/bin/sandbox-exec", "-p", profile, str(decoder), tree],
                                     input=raw, capture_output=True, timeout=15, check=False)
            require(decoded.returncode == 0, "actual PTY decoder failed: " + decoded.stderr.decode()[-1500:])
            output = runner.load_bytes(decoded.stdout)
            require(set(output) == {"schema", "frame", "text", "ansi", "html"} and output["schema"] == "tc-atomic-decoded/v1",
                    "actual decoder schema")
            frame = output["frame"]
            require(frame["version"] == 3 and (frame["cols"], frame["rows"], len(frame["cells"])) == (24, 4, 96),
                    "actual decoder did not produce complete schema-3 screen")
            # Independent narrow source premises, not replacement expected Frames.
            first = frame["cells"][0]
            require(first["symbol"] == state["phase"] and output["text"].splitlines()[0] == state["phase"] + ":" + str(state["value"]),
                    "producer frame is detached from actual state")
            cursor = frame["cursor"]
            require((cursor["x"], cursor["y"], cursor["visible"]) == ((2, 1, True) if state["phase"] == "A" else (7, 2, False)),
                    "actual source cursor did not vary")
            binding = {"checkpoint": ordinal, "action_index": ordinal, "tick": state["tick"],
                       "source_tree": tree, "generation": sha(raw + canonical(state)), "raw_sha256": sha(raw)}
            files = {"frame.json": canonical(frame).decode(), "cursor.json": canonical(cursor).decode(),
                     "state.json": canonical(state).decode(), "screen.txt": output["text"],
                     "screen.ansi": output["ansi"], "screen.html": output["html"]}
            references.append(seal(files, binding))
            receipts.append({"action": "+" if ordinal else "initial", "offset_begin": begin, "offset_end": len(raw),
                             "raw": base64.b64encode(raw).decode(), "state": state, "decoder_exit": decoded.returncode,
                             "decoder_stdout": base64.b64encode(decoded.stdout).decode(), "decoder_stderr": base64.b64encode(decoded.stderr).decode()})
        os.write(master, b"q")
        _, stderr = process.communicate(timeout=10)
        require(process.returncode == 0, "actual producer exit failure: " + stderr.decode()[-1500:])
        return {"references": references, "receipts": receipts, "producer_exit": process.returncode,
                "producer_stderr": base64.b64encode(stderr).decode()}
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)


def mutate_capture(observed, name, selected):
    reference = observed["references"][selected]
    other = observed["references"][3 - selected]
    bundle = copy.deepcopy(reference)
    frame = runner.load_bytes(bundle["files"]["frame.json"].encode())
    other_frame = runner.load_bytes(other["files"]["frame.json"].encode())
    mask = int(name[6:], 2) if name.startswith("mixed-") else 7 if name == "foreign-complete" else 0
    if mask & 4:
        frame["cells"] = other_frame["cells"]
    if mask & 2:
        frame["cursor"] = other_frame["cursor"]
        bundle["files"]["cursor.json"] = canonical(frame["cursor"]).decode()
    if mask & 1:
        state = runner.load_bytes(bundle["files"]["state.json"].encode())
        other_state = runner.load_bytes(other["files"]["state.json"].encode())
        # Forge all generation/action/tick labels consistently; only actual
        # semantic payload belongs to the other real production generation.
        state.update(value=other_state["value"], phase=other_state["phase"])
        bundle["files"]["state.json"] = canonical(state).decode()
    bundle["files"]["frame.json"] = canonical(frame).decode()
    for representation in ("text", "ansi", "html"):
        if name == representation + "-other" or name == "second-live-read":
            suffix = "txt" if representation == "text" else representation
            bundle["files"]["screen." + suffix] = other["files"]["screen." + suffix]
    if name in {"wrong-action", "wrong-tick"}:
        bundle["binding"]["action_index" if name == "wrong-action" else "tick"] += 1
    if name == "stale-generation":
        bundle["binding"]["generation"] = other["binding"]["generation"]
    if name == "raw-other":
        bundle["binding"]["raw_sha256"] = other["binding"]["raw_sha256"]
    if name in {"state-bool", "state-float"}:
        state = runner.load_bytes(bundle["files"]["state.json"].encode())
        state["generation"] = bool(state["generation"]) if name == "state-bool" else float(state["generation"])
        bundle["files"]["state.json"] = canonical(state).decode()
    # Every file hash and every manifest generation label is resealed, even
    # for mixed-generation payloads. Per-file integrity alone cannot pass.
    artifacts = [seal(bundle["files"], bundle["binding"])]
    if name == "missing-checkpoint":
        artifacts = []
    elif name == "duplicate-checkpoint":
        artifacts *= 2
    observed["artifacts"] = artifacts
    if name == "reordered-checkpoints":
        observed["receipts"][1:3] = reversed(observed["receipts"][1:3])
    return observed


def judge(payload, profile):
    require(type(payload) is dict and set(payload) == {"schema", "references", "receipts", "producer_exit", "producer_stderr", "artifacts"}, "atomic observation schema")
    require(payload["schema"] == "tc-capture-atomic-observation/v1", "atomic observation version")
    require(type(payload["producer_exit"]) is int and payload["producer_exit"] == 0, "actual producer failed")
    require(len(payload["references"]) == 3 and len(payload["receipts"]) == 3, "atomic source checkpoints missing")
    for ordinal, (reference, receipt) in enumerate(zip(payload["references"], payload["receipts"])):
        require(reference["binding"]["checkpoint"] == ordinal and receipt["state"]["generation"] == ordinal,
                "atomic source checkpoints reordered")
        require(type(receipt["decoder_exit"]) is int and receipt["decoder_exit"] == 0, "actual decoder failed")
        require(sha(base64.b64decode(receipt["raw"], validate=True)) == reference["binding"]["raw_sha256"], "raw observation binding")
    required = profile["required_checkpoint"]
    require(type(required) is int and required in (1, 2), "invalid atomic required checkpoint")
    expected = payload["references"][required]
    require(type(payload["artifacts"]) is list and len(payload["artifacts"]) == 1, "atomic artifact membership")
    actual = payload["artifacts"][0]
    require(type(actual) is dict and set(actual) == {"binding", "files", "manifest"}, "atomic bundle schema")
    require(canonical(actual["binding"]) == canonical(expected["binding"]), "atomic checkpoint provenance")
    require(type(actual["files"]) is dict and set(actual["files"]) == set(expected["files"]), "atomic representation membership")
    manifest = []
    for name, data in sorted(actual["files"].items()):
        require(type(data) is str, "atomic representation type")
        manifest.append({"path": name, "size": len(data.encode()), "sha256": sha(data.encode()),
                         "generation": actual["binding"]["generation"]})
    require(canonical(actual["manifest"]) == canonical(manifest), "atomic manifest integrity")
    require(canonical(actual["files"]) == canonical(expected["files"]), "mixed production generation")


class AtomicFixture(runner.Fixture):
    def __init__(self, prepared, name):
        super().__init__("capture", "valid")
        self.name, self.prepared = name, prepared
        self.seed = secrets.randbelow(900000) + 1000
        self.selected = (1 if (self.seed + 1) % 2 == 0 else 2) if name == "valid-A" else (1 if (self.seed + 1) % 2 else 2) if name == "valid-B" else secrets.choice([1, 2])
        for filename, data in prepared["source"].items():
            (self.source / filename).write_bytes(data)
        (self.source / "tuisnap.tar").write_bytes(prepared["archive"])
        runner.git(self.source, "add", PRODUCER, DECODER, "tuisnap.tar")
        runner.git(self.source, "-c", "user.name=Qualification", "-c", "user.email=qualification@example.invalid", "-c", "commit.gpgsign=false",
                   "commit", "-q", "-s", "-m", "Protected real PTY source", "-m", "Co-authored-by: Codex <codex@openai.com>")
        self.source_commit = runner.git(self.source, "rev-parse", "HEAD")
        self.source_tree = runner.git(self.source, "rev-parse", "HEAD^{tree}")
        self.executables = {}
        for kind, data in prepared["binaries"].items():
            path = self.private / kind
            path.write_bytes(data)
            path.chmod(0o500)
            self.executables[kind] = path
        self.context.update(tree=self.source_tree, oracle_commit=self.source_commit, lane="pty", capture_profile={
            "schema": "tc-capture-atomic-profile/v1", "kind": "atomic-generation", "required_checkpoint": self.selected,
            "tuisnap_commit": PIN, "tuisnap_tree": TREE, "compilation": prepared["compilation"],
            "dimensions": [24, 4], "actions": ["initial", "+", "+"], "seed": self.seed})
        runner.save(self.context_path, self.context)
        self.context_hash = sha(self.context_path.read_bytes())
        self.before = self.snapshot()

    def launch(self, request):
        require(request == {"schema": "tc-proof-runner-observe/v1", "nonce": self.nonce, "operation": "capture",
                            "source_commit": self.source_commit, "tree": self.source_tree}, "atomic observer binding")
        require(not self.events, "atomic observer replay")
        compiler.validate_compiler_identity(self.prepared["compilation"]["tools"])
        for kind, path in self.executables.items():
            require(sha(path.read_bytes()) == self.prepared["compilation"]["binaries"][kind], "atomic executable changed")
        observed = capture(self.executables["producer"], self.executables["decoder"], self.private, self.seed,
                           self.source_tree, unreadable=[self.public, self.output])
        observed["schema"] = "tc-capture-atomic-observation/v1"
        payload = mutate_capture(observed, self.name, self.selected)
        failure = None
        try:
            judge(payload, self.context["capture_profile"])
        except ValueError as error:
            failure = str(error)
        require(bool(failure) == (self.name in NEGATIVES), "atomic fixture lost intended discrimination: " + str(failure))
        self.actual_failure = failure
        for kind, path in self.executables.items():
            require(sha(path.read_bytes()) == self.prepared["compilation"]["binaries"][kind], "atomic executable changed after run")
        compiler.validate_compiler_identity(self.prepared["compilation"]["tools"])
        event = {"operation": "capture", "run_id": self.run_id, "tree": self.source_tree, "source_commit": self.source_commit,
                 "exit": 0, "compilation": self.prepared["compilation"], "payload": payload}
        self.events.append(event)
        return event

    def validate(self, result, category):
        require(len(self.events) == 1, "atomic qualification omitted actual execution")
        require(bool(self.actual_failure) == bool(category), "atomic private expectation changed")
        code, report = result
        if category:
            require(code != 0 and report["status"] == "rejected" and report["category"] == category and report["outputs"] == {},
                    "atomic negative did not reject exactly")
        else:
            require(code == 0 and report["status"] == "passed" and report["category"] is None and
                    canonical(report["outputs"]) == canonical({"observations": self.events}), "atomic coherent capture rejected or forged")


def adapter_source(mode):
    prelude = "#!" + sys.executable + "\nimport base64,hashlib,json,os,sys\n"
    for function in (runner.require, runner.canonical, runner.sha, judge):
        prelude += inspect.getsource(function) + "\n"
    body = r'''
context=json.load(open(sys.argv[sys.argv.index('--context')+1])); events=[]; failure=False
if MODE != 'zero':
    request={'schema':'tc-proof-runner-observe/v1','nonce':os.environ['TC_PROOF_OBSERVER_NONCE'],'operation':'capture','source_commit':os.environ['TC_PROOF_ORACLE_COMMIT'],'tree':os.environ['TC_PROOF_SOURCE_TREE']}
    with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_REQUEST_FD']),'w') as stream: stream.write(json.dumps(request)+'\n'); stream.flush()
    with os.fdopen(int(os.environ['TC_PROOF_OBSERVER_RESPONSE_FD'])) as stream: event=json.loads(stream.readline())
    events=[event]
    try: judge(event['payload'],context['capture_profile'])
    except ValueError: failure=True
if MODE in {'always-pass','zero','hash-only','labels-only'}: failure=False
digests=[sha(canonical(e)) for e in events]
if MODE=='forged': events[0]['payload']['producer_stderr']='forged'
if MODE=='typed-bool': events[0]['exit']=False
if MODE=='typed-float': events[0]['exit']=0.0
report={'schema':'tc-proof-runner-result/v1','run_id':os.environ['TC_PROOF_RUN_ID'],'operation':'capture','context_sha256':os.environ['TC_PROOF_CONTEXT_SHA256'],'status':'rejected' if failure else 'passed','category':'EXECUTION' if failure else None,'observation_digests':digests,'outputs':{} if failure else {'observations':events}}
with open(os.environ['TC_PROOF_RESULT'],'w') as stream: json.dump(report,stream)
sys.exit(1 if failure else 0)
'''
    return prelude + "MODE=" + repr(mode) + "\n" + body


def execute(prepared, executable, name):
    fixture = AtomicFixture(prepared, name)
    try:
        fixture.validate(fixture.run(executable), "EXECUTION" if name in NEGATIVES else None)
    finally:
        fixture.cleanup()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tuisnap-source", type=Path, required=True)
    parser.add_argument("--runner", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    require(args.self_test != bool(args.runner), "select exactly one self-test or runner")
    prepared = prepare(args.tuisnap_source.resolve())
    with tempfile.TemporaryDirectory(prefix="tc-atomic-adapter-", dir="/tmp") as directory:
        adapter = Path(directory) / "runner"
        if args.self_test:
            adapter.write_text(adapter_source("valid")); adapter.chmod(0o700)
        else:
            adapter = args.runner.resolve(strict=True)
        executable_hash = sha(adapter.read_bytes())
        positives = ["valid-A", "valid-B"]
        for name in [*positives, *NEGATIVES]:
            execute(prepared, adapter, name)
            if name in NEGATIVES:
                for recovery in positives:
                    execute(prepared, adapter, recovery)
            print("qualified " + name, file=sys.stderr, flush=True)
        require(sha(adapter.read_bytes()) == executable_hash, "submitted runner bytes changed")
        attacks = 0
        if args.self_test:
            for mode in ("always-pass", "zero", "hash-only", "labels-only", "forged", "typed-bool", "typed-float"):
                adapter.write_text(adapter_source(mode)); adapter.chmod(0o700)
                failed = False
                try:
                    execute(prepared, adapter, "mixed-101" if mode in {"always-pass", "hash-only", "labels-only"} else "valid-A")
                except ValueError:
                    failed = True
                require(failed, "atomic attack accepted: " + mode)
                attacks += 1
        print(json.dumps({"schema": "tc-capture-atomic-qualification/v1", "cases": 2 + len(NEGATIVES),
                          "negative_cases": len(NEGATIVES), "fresh_recoveries": 2 * len(NEGATIVES),
                          "attacks_rejected": attacks, "compilation": prepared["compilation"],
                          "driver_sha256": sha(Path(__file__).read_bytes())}, sort_keys=True))


if __name__ == "__main__":
    main()
