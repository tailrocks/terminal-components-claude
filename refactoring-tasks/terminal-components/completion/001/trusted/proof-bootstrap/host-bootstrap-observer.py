"""Planner-owned Darwin qualification supervisor, outside the submitted host.

This is a synthetic-test authority, not the production tc-proof-host. It owns
process launches and raw evidence so a submitted host cannot self-attest them.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import sys
import tempfile
import threading


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def quote(path: Path) -> str:
    return json.dumps(str(path.resolve()))


def sandbox(*, writable: list[Path], unreadable: list[Path], readable: list[Path] = (), immutable: list[Path] = ()) -> str:
    # The observer lives outside this profile. A host may inspect itself, but
    # cannot get another process's task port, inspect it, or send it signals.
    return "\n".join([
        "(version 1)", "(allow default)", "(deny file-read*)", "(allow file-read-metadata)", '(allow file-read* (literal "/"))',
        *[f"(allow file-read* (subpath {quote(path)}))" for path in [Path('/System'), Path('/usr/lib'), Path('/usr/bin'), Path('/bin'), Path('/Library/Developer/CommandLineTools'), Path(sys.base_prefix), *writable]],
        '(allow file-read* (literal "/dev/null") (literal "/dev/urandom") (literal "/dev/random"))',
        "(deny file-write*)", '(allow file-write* (literal "/dev/null"))',
        *[f"(allow file-write* (subpath {quote(path)}))" for path in writable],
        *[f"(deny file-write* (subpath {quote(path)}))" for path in immutable],
        *[f"(deny file-read* (subpath {quote(path)}))" for path in unreadable],
        *[f"(allow file-read* (subpath {quote(path)}))" for path in readable],
        "(deny process-info*)", "(allow process-info* (target self))",
        "(deny signal)", "(deny mach-priv-task-port)", "(deny mach-register)",
        "(deny mach-lookup)", '(allow mach-lookup (global-name "com.apple.logd"))',
        "(deny network*)",
    ])


class Observer:
    def __init__(self, fixture_root: Path):
        if sys.platform != "darwin" or not Path("/usr/bin/sandbox-exec").is_file():
            raise AssertionError("qualification needs the protected Darwin observer; unsupported isolation cannot pass")
        if not Path("/Library/Developer/CommandLineTools/usr/bin/git").is_file():
            raise AssertionError("qualification requires the direct Command Line Tools Git executable")
        self.temporary = tempfile.TemporaryDirectory(prefix="tc-observer-", dir="/tmp")
        self.root = Path(self.temporary.name).resolve()
        self.fixture_root = fixture_root.resolve()
        self.nonce = secrets.token_hex(32)
        self.events: list[dict] = []
        self.violations: list[str] = []
        self.armed = False
        self.immutable: list[Path] = []
        self.host_readable: list[Path] = []
        (self.root / "sentinel").write_bytes(secrets.token_bytes(32))

    def close(self) -> None:
        self.temporary.cleanup()

    def arm(self, candidate: Path, progress: Path, package: Path, taskfmt: Path,
            config: Path, parent: str, tree: str, protected: list[Path]) -> None:
        # All arguments originate in the independent fixture driver. Never read
        # these identities or argv from a submitted host request or context.
        self.source = self.root / "source"
        shutil.copytree(candidate, self.source, symlinks=True)
        self.progress = self.root / "progress.md"
        shutil.copyfile(progress, self.progress)
        self.package = self.root / "task"
        shutil.copytree(package, self.package)
        self.config = self.root / "experiment.toml"
        shutil.copyfile(config, self.config)
        self.taskfmt, self.parent, self.tree = taskfmt.resolve(), parent, tree
        self.protected = protected
        self.armed = True

    def _launch(self, step: str) -> dict:
        if not self.armed or step != ["build", "test", "taskfmt"][min(len(self.events), 2)] or len(self.events) >= 3:
            raise AssertionError("unexpected or replayed observer operation")
        output = self.root / ("output-" + step)
        output.mkdir()
        env = {"PATH": os.path.dirname(sys.executable) + ":/Library/Developer/CommandLineTools/usr/bin:/usr/bin:/bin", "LC_ALL": "C", "PYTHONDONTWRITEBYTECODE": "1",
               "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_TERMINAL_PROMPT": "0"}
        if step == "taskfmt":
            argv = [str(self.taskfmt), "verify", "--config", str(self.config), "--root", str(self.source),
                    "--task-dir", str(self.package), "--base", self.parent, "--progress", str(self.progress),
                    "--log-dir", str(output)]
            # taskfmt runs trusted checker bytes in this synthetic fixture.
            profile = sandbox(writable=[output], unreadable=[*self.protected, self.root],
                              readable=[self.source, output, self.package, self.progress, self.config, self.taskfmt])
            profile += f"\n(allow file-read-metadata (literal {quote(self.root)}))"
        else:
            argv = [sys.executable, "-I", "src/worker.py", step]
            env["TC_PROOF_WORKER_OUTPUT"] = str(output)
            profile = sandbox(writable=[output], unreadable=[*self.protected, self.root], readable=[self.source, output])
        result = subprocess.run(["/usr/bin/sandbox-exec", "-p", profile, *argv], cwd=self.source, env=env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
        files = {}
        for path in sorted(output.rglob("*")):
            if path.is_dir():
                continue
            if path.is_symlink() or not path.is_file() or path.stat().st_nlink != 1:
                raise AssertionError("unsafe observer output")
            files[path.relative_to(output).as_posix()] = base64.b64encode(path.read_bytes()).decode()
        event = {"schema": "tc-proof-observation/v1", "step": step, "tree": self.tree,
                 "argv": argv, "exit": result.returncode, "stdout": base64.b64encode(result.stdout).decode(),
                 "stderr": base64.b64encode(result.stderr).decode(), "files": files}
        # This list is in observer memory, inaccessible to the sandboxed host.
        self.events.append(event)
        return event

    def run_host(self, argv: list[str], env: dict, cwd: Path, writable: list[Path] | None = None) -> subprocess.CompletedProcess:
        request_read, request_write = os.pipe()
        response_read, response_write = os.pipe()
        host_env = dict(env, TC_PROOF_OBSERVER_REQUEST_FD=str(request_write),
                        TC_PROOF_OBSERVER_RESPONSE_FD=str(response_read), TC_PROOF_OBSERVER_NONCE=self.nonce,
                        TMPDIR=str(self.fixture_root), PATH=os.path.dirname(sys.executable) + ":/Library/Developer/CommandLineTools/usr/bin:/usr/bin:/bin",
                        GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT="0")

        def serve():
            with os.fdopen(request_read, "rb") as requests, os.fdopen(response_write, "wb") as responses:
                while line := requests.readline(4097):
                    try:
                        if len(line) > 4096 or not line.endswith(b"\n"):
                            raise AssertionError("oversized observer request")
                        def unique(items):
                            result = {}
                            for key, value in items:
                                if key in result:
                                    raise AssertionError("duplicate observer request key")
                                result[key] = value
                            return result
                        request = json.loads(line, object_pairs_hook=unique)
                        if set(request) != {"schema", "nonce", "step"} or request["schema"] != "tc-proof-observer-request/v1" or request["nonce"] != self.nonce:
                            raise AssertionError("invalid observer request binding")
                        reply = self._launch(request["step"])
                    except Exception as error:
                        self.violations.append(str(error))
                        reply = {"schema": "tc-proof-observer-error/v1", "error": str(error)}
                    try:
                        responses.write(json.dumps(reply, sort_keys=True).encode() + b"\n")
                        responses.flush()
                    except BrokenPipeError:
                        return

        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        try:
            result = subprocess.run(["/usr/bin/sandbox-exec", "-p", sandbox(writable=[self.fixture_root] if writable is None else writable,
                                    unreadable=[self.root], readable=[self.fixture_root, Path(argv[0]), *self.host_readable], immutable=self.immutable), *argv],
                                    cwd=cwd, env=host_env, pass_fds=(request_write, response_read), text=True,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=90, check=False)
        finally:
            os.close(request_write)
            os.close(response_read)
            thread.join(timeout=65)
        if thread.is_alive():
            raise AssertionError("observer request did not terminate")
        return result

    def require_success(self, run_dir: Path) -> None:
        if self.violations or [event["step"] for event in self.events] != ["build", "test", "taskfmt"]:
            raise AssertionError("missing, reordered, or forged independent process execution")
        for event in self.events:
            if event["exit"] != 0:
                raise AssertionError("independently observed process failed")
            if event["step"] == "taskfmt":
                if base64.b64decode(event["stdout"]).splitlines()[-1:] != [b"DONE"]:
                    raise AssertionError("independent taskfmt gate did not finish")
                for number in range(1, 6):
                    name = f"CHK-{number:03d}.log"
                    if name not in event["files"]:
                        raise AssertionError("independent taskfmt check log missing")
            else:
                required = {"build.json"} if event["step"] == "build" else {"test.json", "payload.txt"}
                if set(event["files"]) != required:
                    raise AssertionError("independent worker result set is incomplete or contains extras")
                for name, encoded in event["files"].items():
                    path = run_dir / "workers" / event["step"] / name
                    if not path.is_file() or path.read_bytes() != base64.b64decode(encoded):
                        raise AssertionError("host worker artifact differs from independently captured bytes")
