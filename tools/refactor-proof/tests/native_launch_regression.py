#!/usr/bin/env python3
"""Exercise the real bundled worker through the native Rust launcher."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import tempfile
from pathlib import Path


ORACLE_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(
    command: list[str], *, cwd: Path, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True, check=False)


def write_provider(path: Path) -> None:
    path.write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, sys\n"
        "\n"
        "def emit(request):\n"
        "    mode = os.environ.get('TC_PROOF_PROVIDER_MODE', 'valid')\n"
        "    if mode == 'missing':\n"
        "        return None\n"
        "    response = {\n"
        "        'schema': 'tc-proof-observation/v1',\n"
        "        'nonce': request['nonce'],\n"
        "        'run_id': request['run_id'],\n"
        "        'task_id': request['task_id'],\n"
        "        'check_id': request['check_id'],\n"
        "        'request_id': request['request_id'],\n"
        "        'operation': request['operation'],\n"
        "        'source_commit': request['source_commit'],\n"
        "        'tree': request['tree'],\n"
        "        'exit': 0,\n"
        "        'stdout': 'host-observer',\n"
        "        'stderr': '',\n"
        "        'files': {},\n"
        "        'payload': {'schema': 'tc-proof-host-event/v1', 'request_id': request['request_id']},\n"
        "        'records': [{'schema': 'tc-proof-host-record/v1', 'request_id': request['request_id'], 'status': 'observed'}],\n"
        "    }\n"
        "    if mode == 'wrong-nonce':\n"
        "        response['nonce'] = 'forged-nonce'\n"
        "    elif mode == 'replay':\n"
        "        response['request_id'] = 0\n"
        "    elif mode == 'empty':\n"
        "        response['payload'] = {}\n"
        "        response['records'] = []\n"
        "    return response\n"
        "\n"
        "for line in sys.stdin:\n"
        "    request = json.loads(line)\n"
        "    response = emit(request)\n"
        "    if response is not None:\n"
        "        print(json.dumps(response, sort_keys=True, separators=(',', ':')), flush=True)\n",
        encoding="utf-8",
    )
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def prepare_fixture(native: Path, source: Path, root: Path) -> tuple[Path, dict[str, dict[str, str]]]:
    candidate = root / "candidate"
    cloned = run(["git", "clone", "--local", "--no-hardlinks", str(source), str(candidate)], cwd=source)
    if cloned.returncode != 0:
        raise SystemExit(f"candidate clone failed:\n{cloned.stderr}")
    scope = run(["git", "rev-parse", "HEAD"], cwd=candidate)
    if scope.returncode != 0:
        raise SystemExit(f"candidate scope lookup failed:\n{scope.stderr}")
    run_dir = root / "run"
    run_dir.mkdir()
    prepared = run(
        [
            str(native),
            "prepare",
            "--task-dir",
            str(candidate / "refactoring-tasks/terminal-components/completion/001"),
            "--run-dir",
            str(run_dir),
            "--worktree",
            str(candidate),
            "--scope-base",
            scope.stdout.strip(),
            "--oracle-tag",
            "refs/tags/visual-baseline",
            "--oracle-commit",
            ORACLE_COMMIT,
            "--tool",
            str(candidate / "tools/refactor-proof/bin/tc-proof"),
            "--comparator",
            str(native),
            "--observer-nonce",
            "fixture-observer",
        ],
        cwd=source,
    )
    if prepared.returncode != 0:
        raise SystemExit(f"native preparation failed:\n{prepared.stderr}")
    index = json.loads((run_dir / "context-index.json").read_text(encoding="utf-8"))
    member_keys = {"check_id", "path", "sha256"}
    if any(set(member) != member_keys for member in index["contexts"] + index["results"]):
        raise SystemExit("native context index member ABI is not canonical")
    contexts = {member["check_id"]: member for member in index["contexts"]}
    return run_dir, contexts


def worker_environment(run_dir: Path, native: Path, provider: Path) -> dict[str, str]:
    index = run_dir / "context-index.json"
    environment = os.environ.copy()
    environment.update(
        {
            "TC_PROOF_CONTEXT_INDEX": str(index),
            "TC_PROOF_CONTEXT_INDEX_SHA256": digest(index),
            "TC_PROOF_NATIVE_LAUNCH": "1",
            "TC_PROOF_NATIVE_LAUNCHER": str(native),
            "TC_PROOF_NATIVE_TIMEOUT_MS": "5000",
            "TC_PROOF_OBSERVER_PROVIDER": str(provider),
        }
    )
    return environment


def invoke(bundle: Path, context: Path, *, cwd: Path, environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return run(
        [str(bundle), "preflight", "--context", str(context)],
        cwd=cwd,
        env=environment,
    )


def main() -> None:
    source = Path(__file__).resolve().parents[3]
    bundle = source / "tools/refactor-proof/bin/tc-proof"
    native = (
        Path(os.environ["TC_PROOF_NATIVE_BINARY"])
        if os.environ.get("TC_PROOF_NATIVE_BINARY")
        else source / "target/debug/tc-proof"
    )
    if not bundle.is_file() or not os.access(bundle, os.X_OK):
        raise SystemExit(f"tracked proof bundle is not executable: {bundle}")
    if not native.is_file() or not os.access(native, os.X_OK):
        raise SystemExit(f"native verifier is not built: {native}")

    with tempfile.TemporaryDirectory(prefix="tc-proof-native-launch-") as directory:
        root = Path(directory).resolve()
        provider = root / "observer-provider.py"
        write_provider(provider)
        run_dir, contexts = prepare_fixture(native, source, root)
        context = Path(contexts["CHK-001"]["path"])
        environment = worker_environment(run_dir, native, provider)

        direct = dict(environment)
        direct.pop("TC_PROOF_NATIVE_LAUNCH", None)
        direct["TC_PROOF_NATIVE_CHILD"] = "1"
        rejected = invoke(bundle, context, cwd=source, environment=direct)
        if rejected.returncode == 0 or "native launch handoff is missing" not in rejected.stderr:
            raise SystemExit(
                f"direct child-mode injection was accepted: exit={rejected.returncode}\n{rejected.stderr}"
            )
        if (run_dir / "outputs/CHK-001.result.json").exists():
            raise SystemExit("direct child-mode injection wrote a result")

        launched = invoke(bundle, context, cwd=source, environment=environment)
        if launched.returncode != 0:
            raise SystemExit(f"positive native launch failed:\n{launched.stderr}")
        result = json.loads((run_dir / "outputs/CHK-001.result.json").read_text(encoding="utf-8"))
        if result.get("status") != "passed" or not result.get("observation_digests"):
            raise SystemExit(f"positive launch did not bind observed evidence: {result}")

        if "CHK-002" in contexts:
            adversarial = dict(environment)
            adversarial["TC_PROOF_PROVIDER_MODE"] = "wrong-nonce"
            bad = invoke(
                bundle,
                Path(contexts["CHK-002"]["path"]),
                cwd=source,
                environment=adversarial,
            )
            if bad.returncode == 0:
                raise SystemExit("wrong-nonce observer event was accepted")

    print("native launch regression: PASS")


if __name__ == "__main__":
    main()
