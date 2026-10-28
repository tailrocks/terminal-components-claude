#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SUITE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
APP_ROOT="$(cd "$SUITE_ROOT/../.." && pwd -P)"
LEDGER="$SCRIPT_DIR/ci-ledger.py"
GUARD="$(type -P rust-build-guard 2>/dev/null || true)"
if [[ -z "$GUARD" && -x "${HOME:-}/.local/bin/rust-build-guard" ]]; then
    GUARD="${HOME}/.local/bin/rust-build-guard"
fi
if [[ -n "$GUARD" && "$GUARD" != /* ]]; then
    GUARD="$(cd "$(dirname "$GUARD")" 2>/dev/null && pwd -P)/$(basename "$GUARD")"
fi
if [[ -n "$GUARD" ]]; then
    GUARD="$(python3 -c 'import pathlib,sys; print(pathlib.Path(sys.argv[1]).resolve(strict=True))' "$GUARD" 2>/dev/null || true)"
fi
PR26_SHA="5b3dbd7ae799f325efbb4f4cc11b2f5518dec948"
PR26_REF="refs/pull/26/head"
APP_BINARIES=(showcase holla jackin-preview tablepro)

MODE="normal"
EXECUTION_MODE="local-guarded"
EXPORT_ARG=""
RECONCILE_ARG=""
DIAGNOSTIC_FILTER=""
NATIVE_FLAG=0
while (($#)); do
    case "$1" in
        --native-ci)
            [[ "$MODE" == normal && "$NATIVE_FLAG" == 0 && "$EXECUTION_MODE" == local-guarded && -z "$EXPORT_ARG" && -z "$RECONCILE_ARG" ]] || {
                printf '%s\n' 'run-ci: --native-ci cannot be combined with another mode' >&2
                exit 2
            }
            EXECUTION_MODE="velnor-native"
            NATIVE_FLAG=1
            shift
            ;;
        --verify-export)
            [[ "$MODE" == normal && "$EXECUTION_MODE" == local-guarded && -z "$DIAGNOSTIC_FILTER" && -z "$EXPORT_ARG" && -z "$RECONCILE_ARG" && $# -ge 2 ]] || {
                printf '%s\n' 'run-ci: invalid --verify-export arguments' >&2
                exit 2
            }
            MODE="verify-export"
            EXPORT_ARG="$2"
            shift 2
            ;;
        --reconcile-existing)
            [[ "$MODE" == normal && "$EXECUTION_MODE" == local-guarded && -z "$DIAGNOSTIC_FILTER" && -z "$RECONCILE_ARG" && -z "$EXPORT_ARG" && $# -ge 2 ]] || {
                printf '%s\n' 'run-ci: invalid --reconcile-existing arguments' >&2
                exit 2
            }
            MODE="reconcile-existing"
            RECONCILE_ARG="$2"
            shift 2
            ;;
        --diagnostic-filter)
            [[ "$MODE" == normal && -z "$DIAGNOSTIC_FILTER" && -z "$EXPORT_ARG" && -z "$RECONCILE_ARG" && $# -ge 2 && -n "$2" ]] || {
                printf '%s\n' 'run-ci: invalid --diagnostic-filter arguments' >&2
                exit 2
            }
            DIAGNOSTIC_FILTER="$2"
            shift 2
            ;;
        *)
            printf 'run-ci: unknown argument: %s\n' "$1" >&2
            exit 2
            ;;
    esac
done

if [[ "$MODE" == verify-export ]]; then
    exec python3 "$LEDGER" verify-export --export-dir "$EXPORT_ARG"
fi
if [[ "$MODE" == reconcile-existing ]]; then
    exec python3 "$LEDGER" reconcile-existing --run-root "$RECONCILE_ARG"
fi
unset TERMROCK_PARITY_INPUT_CONTROL
if [[ -n "$DIAGNOSTIC_FILTER" && "$DIAGNOSTIC_FILTER" == *$'\n'* ]]; then
    printf '%s\n' 'run-ci: diagnostic filter must be one line' >&2
    exit 2
fi

RUNNER_TEMP_ROOT="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
mkdir -p "$RUNNER_TEMP_ROOT" || {
    printf 'run-ci: cannot create runner temporary directory: %s\n' "$RUNNER_TEMP_ROOT" >&2
    exit 2
}
RUNNER_TEMP_ROOT="$(cd "$RUNNER_TEMP_ROOT" && pwd -P)" || {
    printf 'run-ci: cannot resolve runner temporary directory: %s\n' "$RUNNER_TEMP_ROOT" >&2
    exit 2
}
if [[ -n "${GITHUB_RUN_ID:-}" ]]; then
    WORKFLOW_RUN_ID="$GITHUB_RUN_ID"
else
    WORKFLOW_RUN_ID="local-$(python3 "$LEDGER" new-run-id)"
fi
if [[ ! "$WORKFLOW_RUN_ID" =~ ^[A-Za-z0-9_.-]{1,128}$ || "$WORKFLOW_RUN_ID" == "." || "$WORKFLOW_RUN_ID" == ".." ]]; then
    printf '%s\n' 'run-ci: workflow run ID is not a safe path component' >&2
    exit 2
fi
RUN_ID="$(python3 "$LEDGER" new-run-id)" || exit 2
RUN_ROOT="$RUNNER_TEMP_ROOT/termrock-functional-suite-ci/$WORKFLOW_RUN_ID/$RUN_ID"
ARTIFACT_ROOT="$APP_ROOT/target/termrock-functional-suite-ci/$WORKFLOW_RUN_ID/$RUN_ID"
SOURCE_EXPORT="$RUNNER_TEMP_ROOT/termrock-functional-suite-source/$RUN_ID"
PR26_ROOT="$RUNNER_TEMP_ROOT/termrock-functional-suite-worktrees/$RUN_ID/pr26"
PR26_APP_TARGET="$RUNNER_TEMP_ROOT/termrock-functional-suite-build/$RUN_ID/pr26"
CANDIDATE_APP_TARGET="$RUNNER_TEMP_ROOT/termrock-functional-suite-build/$RUN_ID/candidate"
NEXTTEST_TARGET="$RUNNER_TEMP_ROOT/termrock-functional-suite-target/$RUN_ID"
CARGO_HOME_RAW="${CARGO_HOME:-${HOME:-}/.cargo}"
CARGO_HOME_VALUE="$(python3 - "$CARGO_HOME_RAW" <<'PY'
import os
import sys
from pathlib import Path

print(Path(os.path.expanduser(sys.argv[1])).resolve(strict=False))
PY
)" || exit 2
if [[ "$EXECUTION_MODE" == velnor-native ]]; then
    CARGO_ROUTE=(mise exec --)
else
    CARGO_ROUTE=("$GUARD" -- mise exec --)
fi
CONTEXT_GUARD="$GUARD"
if [[ "$EXECUTION_MODE" == velnor-native ]]; then
    CONTEXT_GUARD=""
fi
EXECUTION_CONTEXT_JSON="$(python3 - \
    "$CONTEXT_GUARD" "$CARGO_HOME_VALUE" \
    "${GITHUB_ACTIONS:-}" "${GITHUB_WORKSPACE:-}" "${GITHUB_RUN_ID:-}" \
    "${GITHUB_RUN_ATTEMPT:-}" "${RUNNER_OS:-}" "${RUNNER_TEMP:-}" \
    "${HOME:-}" "${CARGO_BUILD_JOBS:-}" "${NEXTEST_TEST_THREADS:-}" <<'PY'
import json
import sys

keys = (
    "guard_path", "cargo_home", "github_actions", "github_workspace",
    "github_run_id", "github_run_attempt", "runner_os", "runner_temp",
    "home", "cargo_build_jobs", "nextest_test_threads",
)
values = [value if value else None for value in sys.argv[1:]]
print(json.dumps(dict(zip(keys, values)), sort_keys=True, separators=(",", ":")))
PY
)" || exit 2

CHILD_PID=""
CHILD_PGID=""
SIGNAL_ESCALATOR_PID=""
ACTIVE_PARTICIPANT=""
ACTIVE_NAME=""
INTERRUPTED_SIGNAL=""
INTERRUPTED_CHILD_PID=""
INTERRUPTED_CHILD_PGID=""
INTERRUPTED_PARTICIPANT=""
INTERRUPTED_COMMAND=""
CHILD_WAITING=0
WAIT_INTERRUPTED=0

start_signal_escalator() {
    [[ -n "$CHILD_PGID" && -z "$SIGNAL_ESCALATOR_PID" ]] || return 0
    python3 -c 'import os,signal,sys,time; time.sleep(5); os.killpg(int(sys.argv[1]), signal.SIGKILL)' "$CHILD_PGID" >/dev/null 2>&1 &
    SIGNAL_ESCALATOR_PID=$!
}

terminate_active_child() {
    if [[ -n "$CHILD_PGID" ]]; then
        if ! python3 -c 'import os,signal,sys; os.killpg(int(sys.argv[1]), signal.SIGTERM)' "$CHILD_PGID" >/dev/null 2>&1; then
            [[ -z "$CHILD_PID" ]] || kill -TERM "$CHILD_PID" >/dev/null 2>&1 || true
        fi
        start_signal_escalator
    elif [[ -n "$CHILD_PID" ]]; then
        kill -TERM "$CHILD_PID" >/dev/null 2>&1 || true
    fi
}

on_signal() {
    local signal_name="$1"
    [[ -z "$INTERRUPTED_SIGNAL" ]] || return 0
    INTERRUPTED_SIGNAL="$signal_name"
    INTERRUPTED_CHILD_PID="$CHILD_PID"
    INTERRUPTED_CHILD_PGID="$CHILD_PGID"
    INTERRUPTED_PARTICIPANT="$ACTIVE_PARTICIPANT"
    INTERRUPTED_COMMAND="$ACTIVE_NAME"
    if [[ "$CHILD_WAITING" == 1 ]]; then
        WAIT_INTERRUPTED=1
    fi
    trap '' TERM INT HUP
    terminate_active_child
}

trap 'on_signal TERM' TERM
trap 'on_signal INT' INT
trap 'on_signal HUP' HUP

set_invocation() {
    local key="$1"
    local value_json="$2"
    python3 "$LEDGER" set-invocation --run-root "$RUN_ROOT" --key "$key" --value "$value_json"
}

set_participant() {
    local side="$1"
    local key="$2"
    local value_json="$3"
    python3 "$LEDGER" set-participant --run-root "$RUN_ROOT" --side "$side" --key "$key" --value "$value_json"
}

record_command() {
    local participant="$1"
    local name="$2"
    local cwd="$3"
    local started_at="$4"
    local ended_at="$5"
    local duration="$6"
    local rc="$7"
    local signal_name="$8"
    local pid="$9"
    local pgid="${10}"
    local argv_path="${11}"
    local stdout_path="${12}"
    local stderr_path="${13}"
    local -a args=(
        python3 "$LEDGER" record-command
        --run-root "$RUN_ROOT"
        --participant "$participant"
        --name "$name"
        --argv-json "$argv_path"
        --cwd "$cwd"
        --started-at "$started_at"
        --ended-at "$ended_at"
        --duration-seconds "$duration"
        --pid "$pid"
        --process-group "$pgid"
        --stdout "$stdout_path"
        --stderr "$stderr_path"
    )
    if [[ -n "$rc" ]]; then
        args+=(--exit-status "$rc")
    fi
    if [[ -n "$signal_name" ]]; then
        args+=(--signal "$signal_name")
    fi
    "${args[@]}"
}

LEDGER_IO_FAILURE=""

run_command() {
    local participant="$1"
    local name="$2"
    local cwd="$3"
    shift 3
    local participant_root="$RUN_ROOT/global"
    if [[ "$participant" != global ]]; then
        participant_root="$RUN_ROOT/participants/$participant"
    fi
    local stdout_path="$participant_root/logs/$name.stdout.log"
    local stderr_path="$participant_root/logs/$name.stderr.log"
    local argv_path="$participant_root/commands/$name.argv.json"
    local started_at
    local ended_at
    local started_epoch
    local ended_epoch
    local duration
    local rc
    local signal_name=""
    CHILD_WAITING=0
    WAIT_INTERRUPTED=0

    mkdir -p "$(dirname "$stdout_path")" "$(dirname "$argv_path")"
    python3 -c 'import json,pathlib,sys; pathlib.Path(sys.argv[1]).write_text(json.dumps(sys.argv[2:], ensure_ascii=False), encoding="utf-8")' "$argv_path" "$@" || return 2
    started_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    started_epoch="$(python3 -c 'import time; print(time.time())')"
    ACTIVE_PARTICIPANT="$participant"
    ACTIVE_NAME="$name"
    python3 -c 'import os,sys; os.chdir(sys.argv[1]); os.setsid(); os.execvp(sys.argv[2], sys.argv[2:])' "$cwd" "$@" >"$stdout_path" 2>"$stderr_path" &
    CHILD_PID=$!
    CHILD_PGID="$CHILD_PID"
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        terminate_active_child
    fi

    if [[ "$name" == nextest-run && ( "$participant" == pr26 || "$participant" == candidate ) ]]; then
        set_participant "$participant" run_launched true || true
        set_participant "$participant" run_pid "$CHILD_PID" || true
        set_participant "$participant" run_process_group "$CHILD_PGID" || true
    fi

    CHILD_WAITING=1
    wait "$CHILD_PID"
    rc=$?
    CHILD_WAITING=0
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        if [[ "$WAIT_INTERRUPTED" == 1 ]]; then
            # The first wait was interrupted by the trapped signal. Wait for
            # the bounded SIGKILL escalator, then reap the direct command child
            # to capture its actual exit status.
            if [[ -n "$SIGNAL_ESCALATOR_PID" ]]; then
                wait "$SIGNAL_ESCALATOR_PID" 2>/dev/null || true
                SIGNAL_ESCALATOR_PID=""
            fi
            # Bash keeps the child's wait status after the process disappears;
            # collect it unconditionally instead of using kill -0 as a proxy.
            wait "$CHILD_PID"
            rc=$?
        fi
        local remaining_checks=100
        while ((remaining_checks > 0)) && python3 -c 'import os,sys; os.killpg(int(sys.argv[1]), 0)' "$CHILD_PGID" >/dev/null 2>&1; do
            sleep 0.1
            ((remaining_checks-=1))
        done
        if python3 -c 'import os,sys; os.killpg(int(sys.argv[1]), 0)' "$CHILD_PGID" >/dev/null 2>&1; then
            LEDGER_IO_FAILURE="owned child process group remained after SIGTERM/SIGKILL cleanup"
        fi
        if [[ -n "$rc" ]]; then
            INTERRUPTED_RAW_EXIT="$rc"
        else
            INTERRUPTED_RAW_EXIT=""
        fi
    fi
    if [[ -n "$SIGNAL_ESCALATOR_PID" ]]; then
        kill -TERM "$SIGNAL_ESCALATOR_PID" >/dev/null 2>&1 || true
        wait "$SIGNAL_ESCALATOR_PID" 2>/dev/null || true
        SIGNAL_ESCALATOR_PID=""
    fi
    ended_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    ended_epoch="$(python3 -c 'import time; print(time.time())')"
    duration="$(python3 -c 'import sys; print(max(0.0, float(sys.argv[2])-float(sys.argv[1])))' "$started_epoch" "$ended_epoch")"
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        signal_name="$INTERRUPTED_SIGNAL"
    fi
    if ! record_command "$participant" "$name" "$cwd" "$started_at" "$ended_at" "$duration" "$rc" "$signal_name" "$CHILD_PID" "$CHILD_PGID" "$argv_path" "$stdout_path" "$stderr_path"; then
        LEDGER_IO_FAILURE="could not record command: $participant/$name"
    fi
    CHILD_PID=""
    CHILD_PGID=""
    ACTIVE_PARTICIPANT=""
    ACTIVE_NAME=""
    if [[ -n "$LEDGER_IO_FAILURE" ]]; then
        return 2
    fi
    return "$rc"
}

write_interruption() {
    [[ -n "$INTERRUPTED_SIGNAL" ]] || return 0
    local value_json
    value_json="$(python3 -c 'import json,sys; print(json.dumps({"signal":sys.argv[1],"raw_child_exit":int(sys.argv[2]) if sys.argv[2] else None,"child_pid":int(sys.argv[3]) if sys.argv[3] else None,"child_process_group":int(sys.argv[4]) if sys.argv[4] else None,"participant":sys.argv[5] or None,"command":sys.argv[6] or None}))' "$INTERRUPTED_SIGNAL" "${INTERRUPTED_RAW_EXIT:-}" "$INTERRUPTED_CHILD_PID" "$INTERRUPTED_CHILD_PGID" "$INTERRUPTED_PARTICIPANT" "$INTERRUPTED_COMMAND")"
    set_invocation interruption "$value_json"
}

json_string() {
    python3 -c 'import json,sys; print(json.dumps(sys.argv[1], ensure_ascii=False))' "$1"
}

initialize_run() {
    local init_stdout="$RUNNER_TEMP_ROOT/termrock-functional-suite-ci/$WORKFLOW_RUN_ID/.${RUN_ID}.init.stdout"
    local init_stderr="$RUNNER_TEMP_ROOT/termrock-functional-suite-ci/$WORKFLOW_RUN_ID/.${RUN_ID}.init.stderr"
    local init_rc
    mkdir -p "$(dirname "$init_stdout")"
    local -a init_command=(python3 "$LEDGER" init --suite-root "$SUITE_ROOT" --run-root "$RUN_ROOT" --run-id "$RUN_ID" --execution-mode "$EXECUTION_MODE" --execution-context-json "$EXECUTION_CONTEXT_JSON")
    if [[ -n "$DIAGNOSTIC_FILTER" ]]; then
        init_command+=(--diagnostic-filter "$DIAGNOSTIC_FILTER")
    fi
    "${init_command[@]}" >"$init_stdout" 2>"$init_stderr"
    init_rc=$?
    if [[ -d "$RUN_ROOT" ]]; then
        mkdir -p "$RUN_ROOT/global/logs" "$RUN_ROOT/global/commands"
        mv "$init_stdout" "$RUN_ROOT/global/logs/initialize.stdout.log" 2>/dev/null || true
        mv "$init_stderr" "$RUN_ROOT/global/logs/initialize.stderr.log" 2>/dev/null || true
    fi
    return "$init_rc"
}

publish_artifact() {
    if [[ ! -d "$RUN_ROOT" ]]; then
        printf 'run-ci: evidence directory is unavailable: %s\n' "$RUN_ROOT" >&2
        return 2
    fi
    if [[ -e "$ARTIFACT_ROOT" || -L "$ARTIFACT_ROOT" ]]; then
        printf 'run-ci: refusing to overwrite artifact directory: %s\n' "$ARTIFACT_ROOT" >&2
        return 2
    fi
    mkdir -p "$(dirname "$ARTIFACT_ROOT")" || return 2
    cp -R "$RUN_ROOT" "$ARTIFACT_ROOT" || return 2
    python3 "$LEDGER" seal-artifact --run-root "$RUN_ROOT" --artifact-root "$ARTIFACT_ROOT"
}

finalize_and_publish() {
    local result=0
    write_interruption || result=1
    python3 "$LEDGER" finalize --run-root "$RUN_ROOT" || result=1
    publish_artifact || result=1
    return "$result"
}

write_source_identity() {
    local output="$RUN_ROOT/source-identities.json"
    python3 - "$APP_ROOT" "$PR26_ROOT" "$SUITE_ROOT" "$SOURCE_EXPORT" "$output" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path

candidate = Path(sys.argv[1])
pr26 = Path(sys.argv[2])
suite = Path(sys.argv[3])
exported = Path(sys.argv[4])
output = Path(sys.argv[5])

def git(root: Path, *args: str) -> str:
    result = subprocess.run(["git", "-C", str(root), *args], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return result.stdout.decode("utf-8", "replace").strip()

def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def identity(root: Path, expected_sha: str | None = None) -> dict[str, object]:
    if not root.is_dir():
        return {"root": str(root), "available": False, "expected_sha": expected_sha}
    diff = subprocess.run(["git", "-C", str(root), "diff", "--binary", "HEAD"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    status = git(root, "status", "--porcelain", "--untracked-files=all").splitlines()
    lock = root / "Cargo.lock"
    return {
        "root": str(root),
        "available": True,
        "git_head": git(root, "rev-parse", "HEAD"),
        "expected_sha": expected_sha,
        "tracked_worktree_diff_sha256": hashlib.sha256(diff).hexdigest(),
        "status_lines": status,
        "cargo_lock_sha256": sha(lock),
    }

inventory = exported / ".termrock-export-manifest.json"
document = {
    "schema_version": 1,
    "candidate": identity(candidate),
    "pr26": identity(pr26, "5b3dbd7ae799f325efbb4f4cc11b2f5518dec948"),
    "suite_cargo_lock_sha256": sha(suite / "Cargo.lock"),
    "suite_export_manifest_sha256": sha(inventory),
    "suite_export_manifest": str(inventory),
}
output.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

write_binary_inventory() {
    local side="$1"
    local binary_dir="$2"
    python3 - "$side" "$binary_dir" "$RUN_ROOT" "$RUN_ROOT/participants/$side/binary-inventory.json" <<'PY'
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

side, raw_dir, raw_root, raw_output = sys.argv[1:]
directory = Path(raw_dir)
root = Path(raw_root)
output = Path(raw_output)
names = ("showcase", "holla", "jackin-preview", "tablepro")
records = []
for name in names:
    path = directory / name
    mode = path.lstat().st_mode
    if not stat.S_ISREG(mode) or path.is_symlink():
        raise SystemExit(f"binary is missing or unsafe: {path}")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    records.append({"name": name, "path": path.relative_to(root).as_posix(),
                    "sha256": digest.hexdigest(), "size": path.stat().st_size, "mode": stat.S_IMODE(mode)})
output.write_text(json.dumps({"schema_version": 1, "participant": side, "binaries": records}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

initialize_run
INITIALIZE_RC=$?
if [[ ! -d "$RUN_ROOT" ]]; then
    printf '%s\n' 'run-ci: unable to create the initialized evidence inventory' >&2
    exit 2
fi
if [[ "$INITIALIZE_RC" -ne 0 ]]; then
    finalize_and_publish || true
    exit 2
fi

OVERALL_RC=0
GLOBAL_FAILURE=""
EXECUTION_CONTEXT_VALID=1
mkdir -p "$RUN_ROOT/global/logs"
if ! python3 "$LEDGER" validate-execution-context --run-root "$RUN_ROOT" \
    >"$RUN_ROOT/global/logs/execution-context.stdout.log" \
    2>"$RUN_ROOT/global/logs/execution-context.stderr.log"; then
    EXECUTION_CONTEXT_VALID=0
    OVERALL_RC=1
    GLOBAL_FAILURE="execution mode/context validation failed; see global/logs/execution-context.stderr.log"
fi

set_never_launched() {
    local side="$1"
    local reason="$2"
    set_participant "$side" never_launched_reason "$(json_string "$reason")"
}

set_build_result() {
    local side="$1"
    local rc="$2"
    local built="$3"
    set_participant "$side" build_exit "$rc"
    set_participant "$side" built "$built"
}

setup_pr26_worktree() {
    local found_sha=""
    if git -C "$APP_ROOT" cat-file -e "$PR26_SHA^{commit}" 2>/dev/null; then
        found_sha="$PR26_SHA"
    else
        if ! run_command global fetch-pr26 "$APP_ROOT" git -C "$APP_ROOT" fetch origin "$PR26_REF"; then
            GLOBAL_FAILURE="could not fetch PR26 oracle source"
            return 1
        fi
        found_sha="$(git -C "$APP_ROOT" rev-parse FETCH_HEAD 2>/dev/null || true)"
    fi
    if [[ "$found_sha" != "$PR26_SHA" ]]; then
        GLOBAL_FAILURE="PR26 ref resolved to ${found_sha:-unknown}, expected $PR26_SHA"
        return 1
    fi
    mkdir -p "$(dirname "$PR26_ROOT")"
    if ! run_command global add-pr26-worktree "$APP_ROOT" git -C "$APP_ROOT" worktree add --detach "$PR26_ROOT" "$PR26_SHA"; then
        GLOBAL_FAILURE="could not create clean PR26 worktree"
        return 1
    fi
    local actual_sha
    actual_sha="$(git -C "$PR26_ROOT" rev-parse HEAD 2>/dev/null || true)"
    if [[ "$actual_sha" != "$PR26_SHA" ]]; then
        GLOBAL_FAILURE="created PR26 worktree has unexpected HEAD ${actual_sha:-unknown}"
        return 1
    fi
    return 0
}

prepare_suite_export() {
    if [[ -e "$SOURCE_EXPORT" || -L "$SOURCE_EXPORT" ]]; then
        GLOBAL_FAILURE="source export destination already exists"
        return 1
    fi
    if ! run_command global export-source "$APP_ROOT" python3 "$SUITE_ROOT/scripts/export-source.py" "$SOURCE_EXPORT" --forbid-root "$APP_ROOT" --forbid-root "$PR26_ROOT"; then
        GLOBAL_FAILURE="source export failed"
        return 1
    fi
    if ! run_command global prepare-engines "$SOURCE_EXPORT" python3 "$SOURCE_EXPORT/scripts/prepare-engines.py"; then
        GLOBAL_FAILURE="pinned engine preparation failed"
        return 1
    fi
    if ! run_command global write-export-manifest "$SOURCE_EXPORT" python3 "$LEDGER" write-export-manifest --export-dir "$SOURCE_EXPORT"; then
        GLOBAL_FAILURE="could not seal the prepared source export"
        return 1
    fi
    if ! run_command global verify-export-initial "$SOURCE_EXPORT" python3 "$LEDGER" verify-export --export-dir "$SOURCE_EXPORT"; then
        GLOBAL_FAILURE="prepared source export failed verification"
        return 1
    fi
    cp "$SOURCE_EXPORT/.termrock-export-manifest.json" "$RUN_ROOT/export-file-manifest.json" || return 1
    cp "$SOURCE_EXPORT/engine-pins.json" "$RUN_ROOT/engine-pins.json" || return 1
    cp "$SOURCE_EXPORT/expected/BASELINE-APPROVAL.json" "$RUN_ROOT/BASELINE-APPROVAL.json" || return 1
    set_invocation export_verified true || return 1
    set_invocation expected_store_verified true || return 1
    local identity_json
    identity_json="$(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1], encoding="utf-8"))))' "$RUN_ROOT/global/logs/verify-export-initial.stdout.log")" || return 1
    set_invocation export_identity "$identity_json" || return 1
    return 0
}

verify_shared_export() {
    local stage="$1"
    if ! run_command global "verify-export-$stage" "$SOURCE_EXPORT" python3 "$LEDGER" verify-export --export-dir "$SOURCE_EXPORT"; then
        GLOBAL_FAILURE="source export or expected store changed during paired execution"
        set_invocation export_verified false || true
        set_invocation expected_store_verified false || true
        return 1
    fi
    local identity_json
    identity_json="$(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1], encoding="utf-8"))))' "$RUN_ROOT/global/logs/verify-export-$stage.stdout.log")" || return 1
    set_invocation export_identity "$identity_json" || return 1
    set_invocation export_verified true || return 1
    set_invocation expected_store_verified true || return 1
    return 0
}

write_suite_run_identity() {
    write_source_identity || return 1
    local source_json
    source_json="$(python3 -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1], encoding="utf-8"))))' "$RUN_ROOT/source-identities.json")" || return 1
    set_invocation source_identity "$source_json" || return 1
    return 0
}

write_toolchain_identity() {
    local tool
    for tool in rustc mbx cargo-nextest; do
        if ! run_command global "tool-$tool-version" "$APP_ROOT" env MISE_DISABLE_TOOLS=java MISE_CONFIG_FILE="$APP_ROOT/mise.toml" CARGO_HOME="$CARGO_HOME_VALUE" mise exec -- "$tool" --version; then
            return 1
        fi
    done
    local value_json
    value_json="$(python3 - "$APP_ROOT/mise.toml" "$APP_ROOT/mise.lock" "$APP_ROOT/rust-toolchain.toml" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

records = {}
for raw in sys.argv[1:]:
    path = Path(raw)
    if path.is_file() and not path.is_symlink():
        records[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
print(json.dumps({"tool_config_sha256": records}, sort_keys=True))
PY
)" || return 1
    set_invocation toolchain_identity "$value_json"
}

copy_export_receipts() {
    cp "$SOURCE_EXPORT/.termrock-export-manifest.json" "$RUN_ROOT/export-file-manifest.json" || return 1
    cp "$SOURCE_EXPORT/engine-pins.json" "$RUN_ROOT/engine-pins.json" || return 1
    cp "$SOURCE_EXPORT/expected/BASELINE-APPROVAL.json" "$RUN_ROOT/BASELINE-APPROVAL.json" || return 1
    return 0
}

build_app_side() {
    local side="$1"
    local source_root="$2"
    local target_root="$3"
    local side_root="$RUN_ROOT/participants/$side"
    local binary_dir="$side_root/bin"
    local build_rc=0
    mkdir -p "$target_root" "$binary_dir"
    set_participant "$side" build_target_root "$(json_string "$target_root")" || return 1
    if run_command "$side" app-build "$source_root" env MISE_DISABLE_TOOLS=java MISE_CONFIG_FILE="$APP_ROOT/mise.toml" CARGO_HOME="$CARGO_HOME_VALUE" CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=1 "${CARGO_ROUTE[@]}" cargo build --locked --offline --jobs 1 -p showcase -p holla -p jackin-preview -p tablepro --bins; then
        build_rc=0
    else
        build_rc=$?
    fi
    set_build_result "$side" "$build_rc" false || return 1
    if [[ "$build_rc" -ne 0 ]]; then
        set_never_launched "$side" "application binary build failed with exit $build_rc" || true
        OVERALL_RC=1
        return 1
    fi
    local name
    for name in "${APP_BINARIES[@]}"; do
        if [[ ! -f "$target_root/debug/$name" || -L "$target_root/debug/$name" ]]; then
            set_never_launched "$side" "built binary is missing or unsafe: $name" || true
            set_build_result "$side" 0 false || true
            OVERALL_RC=1
            return 1
        fi
        if ! cp "$target_root/debug/$name" "$binary_dir/$name"; then
            set_never_launched "$side" "could not stage application binary: $name" || true
            OVERALL_RC=1
            return 1
        fi
        chmod 755 "$binary_dir/$name" || return 1
    done
    if ! write_binary_inventory "$side" "$binary_dir"; then
        set_never_launched "$side" "staged application binary inventory is invalid" || true
        OVERALL_RC=1
        return 1
    fi
    set_build_result "$side" 0 true || return 1
    return 0
}

fetch_cargo_workspace() {
    local name="$1"
    local workspace="$2"
    run_command global "$name" "$workspace" \
        env MISE_DISABLE_TOOLS=java MISE_CONFIG_FILE="$APP_ROOT/mise.toml" \
        CARGO_HOME="$CARGO_HOME_VALUE" CARGO_BUILD_JOBS=1 \
        "${CARGO_ROUTE[@]}" cargo fetch --locked
}

prepare_cargo_dependencies() {
    local pr26_available="$1"
    local result=0
    local rc

    if fetch_cargo_workspace cargo-fetch-suite "$SOURCE_EXPORT"; then
        :
    else
        rc=$?
        result=1
        GLOBAL_FAILURE="Suite locked dependency fetch exited $rc"
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        result=1
    elif [[ "$pr26_available" == 1 ]]; then
        if fetch_cargo_workspace cargo-fetch-pr26 "$PR26_ROOT"; then
            :
        else
            rc=$?
            result=1
            GLOBAL_FAILURE="PR26 locked dependency fetch exited $rc"
        fi
    else
        result=1
        GLOBAL_FAILURE="PR26 workspace is unavailable for locked dependency preparation"
    fi
    if [[ -z "$INTERRUPTED_SIGNAL" ]]; then
        if fetch_cargo_workspace cargo-fetch-candidate "$APP_ROOT"; then
            :
        else
            rc=$?
            result=1
            GLOBAL_FAILURE="candidate locked dependency fetch exited $rc"
        fi
    fi
    if [[ "$result" -eq 0 ]]; then
        set_invocation dependency_sources_ready true || return 1
        set_invocation dependency_sources_error null || return 1
        return 0
    fi
    OVERALL_RC=1
    set_invocation dependency_sources_ready false || true
    set_invocation dependency_sources_error "$(json_string "${GLOBAL_FAILURE:-locked dependency preparation was incomplete}")" || true
    return 1
}

build_both_sides() {
    local pr26_available=0
    if setup_pr26_worktree; then
        pr26_available=1
    else
        set_never_launched pr26 "${GLOBAL_FAILURE:-PR26 source setup failed}" || true
        set_build_result pr26 1 false || true
        OVERALL_RC=1
    fi

    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        return 1
    fi
    prepare_cargo_dependencies "$pr26_available" || true
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        return 1
    fi
    if [[ "$pr26_available" == 1 ]]; then
        build_app_side pr26 "$PR26_ROOT" "$PR26_APP_TARGET" || true
    fi
    if build_app_side candidate "$APP_ROOT" "$CANDIDATE_APP_TARGET"; then
        :
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        return 1
    fi
    write_suite_run_identity || {
        GLOBAL_FAILURE="could not record source identities"
        return 1
    }
    write_toolchain_identity || {
        GLOBAL_FAILURE="could not record Mise tool versions"
        return 1
    }
    return 0
}

list_suite_tests() {
    local list_path="$RUN_ROOT/nextest-list.raw.json"
    local nextest_rc=0
    local helper_rc=0
    mkdir -p "$NEXTTEST_TARGET"
    local -a command=(
        env
        MISE_DISABLE_TOOLS=java
        MISE_CONFIG_FILE="$APP_ROOT/mise.toml"
        CARGO_HOME="$CARGO_HOME_VALUE"
        CARGO_TARGET_DIR="$NEXTTEST_TARGET"
        CARGO_BUILD_JOBS=1
        TERMROCK_PARITY_BIN_DIR="$RUN_ROOT/participants/pr26/bin"
        TERMROCK_PARITY_OUT_DIR="$RUN_ROOT/participants/pr26/out"
        TERMROCK_PARITY_RUN_ID="$RUN_ID-pr26"
        "${CARGO_ROUTE[@]}" cargo nextest list
        --test functional
        --locked --offline
        --profile ci
        --cargo-profile dev
        --config-file "$SOURCE_EXPORT/.config/nextest.toml"
        --user-config-file none
        --build-jobs 1
        --ignore-default-filter
        --message-format json
    )
    if [[ -n "$DIAGNOSTIC_FILTER" ]]; then
        command+=(--filter-expr "$DIAGNOSTIC_FILTER")
    fi
    if run_command global nextest-list "$SOURCE_EXPORT" "${command[@]}"; then
        nextest_rc=0
    else
        nextest_rc=$?
    fi
    cp "$RUN_ROOT/global/logs/nextest-list.stdout.log" "$list_path" 2>/dev/null || true
    local mode="normal"
    [[ -z "$DIAGNOSTIC_FILTER" ]] || mode="diagnostic"
    if run_command global validate-list "$APP_ROOT" python3 "$LEDGER" record-list \
        --run-root "$RUN_ROOT" --list-json "$list_path" \
        --export-root "$SOURCE_EXPORT" --target-root "$NEXTTEST_TARGET" --mode "$mode"; then
        helper_rc=0
    else
        helper_rc=$?
    fi
    if [[ "$nextest_rc" -ne 0 ]]; then
        set_invocation list_valid false || true
        set_invocation list_error "$(json_string "Nextest LIST exited $nextest_rc")" || true
        return 1
    fi
    if [[ "$helper_rc" -ne 0 ]]; then
        return 1
    fi
    return 0
}

run_suite_side() {
    local side="$1"
    local side_root="$RUN_ROOT/participants/$side"
    local state_path="$side_root/state.json"
    local side_run_id="$RUN_ID-$side"
    local junit_config="$side_root/nextest.toml"
    local junit_source="$NEXTTEST_TARGET/nextest/ci/junit-$side_run_id.xml"
    local junit_source_copy="$side_root/junit-source-original.xml"
    local junit_copy="$side_root/junit.xml"
    local run_rc=0
    local -a command=(
        env
        MISE_DISABLE_TOOLS=java
        MISE_CONFIG_FILE="$APP_ROOT/mise.toml"
        CARGO_HOME="$CARGO_HOME_VALUE"
        CARGO_TARGET_DIR="$NEXTTEST_TARGET"
        CARGO_BUILD_JOBS=1
        TERMROCK_PARITY_BIN_DIR="$side_root/bin"
        TERMROCK_PARITY_OUT_DIR="$side_root/out"
        TERMROCK_PARITY_RUN_ID="$side_run_id"
        "${CARGO_ROUTE[@]}" cargo nextest run
        --test functional
        --locked --offline
        --profile ci
        --cargo-profile dev
        --config-file "$junit_config"
        --user-config-file none
        --build-jobs 1
        --test-threads 1
        --retries 0
        --no-fail-fast
        --no-tests fail
        --ignore-default-filter
    )
    if [[ -n "$DIAGNOSTIC_FILTER" ]]; then
        command+=(--filter-expr "$DIAGNOSTIC_FILTER")
    fi
    if [[ ! -f "$state_path" ]]; then
        GLOBAL_FAILURE="$side state is missing"
        return 1
    fi
    if ! python3 - "$state_path" <<'PY'
import json
import sys
from pathlib import Path
state = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
raise SystemExit(0 if state.get("built") is True else 1)
PY
    then
        return 0
    fi
    if run_command "$side" nextest-config "$SOURCE_EXPORT" python3 "$LEDGER" write-junit-config --base-config "$SOURCE_EXPORT/.config/nextest.toml" --output-config "$junit_config" --run-id "$side_run_id"; then
        :
    else
        GLOBAL_FAILURE="could not create unique $side JUnit config"
        return 1
    fi
    if [[ -e "$junit_source" || -L "$junit_source" || -e "$junit_source_copy" || -L "$junit_source_copy" || -e "$junit_copy" || -L "$junit_copy" ]]; then
        GLOBAL_FAILURE="refusing pre-existing JUnit report path for $side"
        return 1
    fi
    if ! run_command global "verify-functional-binary-$side" "$APP_ROOT" \
        python3 "$LEDGER" verify-list-binary --run-root "$RUN_ROOT" --target-root "$NEXTTEST_TARGET"; then
        set_never_launched "$side" "functional test binary identity check failed before Nextest RUN" || true
        GLOBAL_FAILURE="functional test binary identity check failed for $side"
        OVERALL_RC=1
        return 1
    fi
    if run_command "$side" nextest-run "$SOURCE_EXPORT" "${command[@]}"; then
        run_rc=0
    else
        run_rc=$?
    fi
    set_participant "$side" run_exit "$run_rc" || true
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        set_participant "$side" run_signal "$(json_string "$INTERRUPTED_SIGNAL")" || true
    fi
    if [[ -f "$junit_source" && ! -L "$junit_source" ]]; then
        cp "$junit_source" "$junit_source_copy" || GLOBAL_FAILURE="could not preserve $side JUnit source report"
        cp "$junit_source" "$junit_copy" || GLOBAL_FAILURE="could not copy $side JUnit report"
        if [[ -f "$junit_source_copy" && -f "$junit_copy" ]]; then
            python3 - "$junit_source" "$junit_source_copy" "$junit_copy" "$side_root/junit-source.json" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
rows = []
for raw in sys.argv[1:4]:
    path = Path(raw)
    rows.append({"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "size": path.stat().st_size})
Path(sys.argv[4]).write_text(json.dumps({"schema_version": 1, "files": rows}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
        fi
    fi
    if [[ "$run_rc" -ne 0 ]]; then
        OVERALL_RC=1
    fi
    return 0
}

record_runner_failure() {
    local failure_file="$RUN_ROOT/global/runner-failure.json"
    mkdir -p "$(dirname "$failure_file")"
    python3 - "$failure_file" "$GLOBAL_FAILURE" "$OVERALL_RC" "$INTERRUPTED_SIGNAL" "$LEDGER_IO_FAILURE" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

path = Path(sys.argv[1])
payload = {
    "schema_version": 1,
    "created_at": datetime.now(timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z"),
    "global_failure": sys.argv[2] or None,
    "overall_exit_status": int(sys.argv[3]),
    "interruption_signal": sys.argv[4] or None,
    "ledger_io_failure": sys.argv[5] or None,
}
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

run_normal_pipeline() {
    if ! prepare_suite_export; then
        OVERALL_RC=1
        return 1
    fi
    if ! build_both_sides; then
        OVERALL_RC=1
        GLOBAL_FAILURE="${GLOBAL_FAILURE:-application build or identity preparation failed}"
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        OVERALL_RC=1
        return 1
    fi
    if ! verify_shared_export before-nextest-list; then
        OVERALL_RC=1
        return 1
    fi
    if ! list_suite_tests; then
        OVERALL_RC=1
        return 1
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        OVERALL_RC=1
        return 1
    fi
    if ! verify_shared_export before-pr26-run; then
        OVERALL_RC=1
        return 1
    fi
    if ! run_suite_side pr26; then
        OVERALL_RC=1
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        OVERALL_RC=1
        return 1
    fi
    if ! verify_shared_export before-candidate-run; then
        OVERALL_RC=1
        return 1
    fi
    if ! run_suite_side candidate; then
        OVERALL_RC=1
    fi
    if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
        OVERALL_RC=1
        return 1
    fi
    if ! verify_shared_export after-paired-runs; then
        OVERALL_RC=1
        return 1
    fi
    return 0
}

if [[ "$EXECUTION_CONTEXT_VALID" != 1 ]]; then
    OVERALL_RC=1
elif [[ -n "$INTERRUPTED_SIGNAL" ]]; then
    OVERALL_RC=1
else
    run_normal_pipeline || OVERALL_RC=1
fi
if [[ -n "$GLOBAL_FAILURE" || -n "$LEDGER_IO_FAILURE" ]]; then
    OVERALL_RC=1
fi
if [[ -n "$INTERRUPTED_SIGNAL" ]]; then
    case "$INTERRUPTED_SIGNAL" in
        HUP) OVERALL_RC=129 ;;
        INT) OVERALL_RC=130 ;;
        TERM) OVERALL_RC=143 ;;
        *) OVERALL_RC=1 ;;
    esac
fi
if ! record_runner_failure; then
    [[ -n "$INTERRUPTED_SIGNAL" ]] || OVERALL_RC=1
fi
if ! finalize_and_publish; then
    [[ -n "$INTERRUPTED_SIGNAL" ]] || OVERALL_RC=1
fi

if [[ "$OVERALL_RC" -eq 0 ]]; then
    if python3 - "$RUN_ROOT/invocation.json" <<'PY'
import json
import sys
from pathlib import Path

invocation = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
raise SystemExit(0 if invocation.get("report_complete") is True and invocation.get("success") is True else 1)
PY
    then
        printf 'run-ci: run_id=%s report_complete=true success=true artifact=%s\n' "$RUN_ID" "$ARTIFACT_ROOT"
        exit 0
    fi
    OVERALL_RC=1
fi
printf 'run-ci: run_id=%s report_complete/status recorded; exit=%s artifact=%s\n' "$RUN_ID" "$OVERALL_RC" "$ARTIFACT_ROOT" >&2
exit "$OVERALL_RC"

