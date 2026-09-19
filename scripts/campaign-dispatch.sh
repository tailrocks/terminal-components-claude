#!/usr/bin/env bash
# Run latest taskfmt validation/verification for one host-local subagent task.
set -euo pipefail

TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_VERSION="0.2.0"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
ORACLE_TAG="refs/tags/visual-baseline"
ORACLE_COMMIT="4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
REPO_ROOT="$(git rev-parse --show-toplevel)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/campaign-path-guards.sh"
INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
CAMPAIGN_ROOT="${TC_CAMPAIGN_ROOT:-$REPO_ROOT}"
CAMPAIGN_LEDGER="${TC_CAMPAIGN_LEDGER:-$CAMPAIGN_ROOT/.campaign/ledger.json}"
READINESS_REPORT="${TC_READINESS_REPORT:-$CAMPAIGN_ROOT/docs/refactoring-plan/execution-readiness-report.md}"
CATALOG_ROOT="${TC_CATALOG_ROOT:-$REPO_ROOT/refactoring-tasks/terminal-components/completion}"
WORKTREE="${TC_TASK_WORKTREE:-}"
RUN_DIR="${TC_TASK_RUN_DIR:-}"
BASE="${TC_TASK_BASE:-}"
PREFLIGHT_EVIDENCE="${TC_TASK_PREFLIGHT_EVIDENCE:-}"

usage() {
	cat <<'EOF'
Usage: campaign-dispatch.sh <lint|verify|help>

Required:
  TASK=001                 Numbered task package.

Optional:
  TC_TASKFMT               Path to the exact pinned standalone taskfmt executable.
  TC_CATALOG_ROOT          Catalog root (default: refactoring-tasks/terminal-components/completion).
  TC_CAMPAIGN_ROOT         Current campaign worktree containing readiness/ledger.
  TC_CAMPAIGN_LEDGER       Current campaign ledger (default: $TC_CAMPAIGN_ROOT/.campaign/ledger.json).
  TC_READINESS_REPORT      Current readiness report (default: campaign report path).
  TC_TASK_WORKTREE         Required isolated subagent worktree for verify.
  TC_TASK_RUN_DIR          Required for verify; unique external run/log directory.
  TC_TASK_BASE             Immutable task scope base commit.
  TC_TASK_PREFLIGHT_EVIDENCE
                           Required external JSON authorization manifest from current preflight.
  TC_PROOF_TARGET_DIR      Existing external native-proof Cargo target directory.

Only the standalone taskfmt lint and verify commands are allowed. This script
never starts containers, invokes taskfmt lifecycle binaries, arms the ledger, or integrates refs.
EOF
}

die() {
	echo "campaign-dispatch: $*" >&2
	exit 1
}

require_absolute_paths() {
	[[ "$CATALOG_ROOT" = /* ]] || die "catalog root must be absolute: $CATALOG_ROOT"
	[[ "$TASKFMT_SOURCE" = /* ]] || die "taskfmt source must be absolute: $TASKFMT_SOURCE"
	[[ "$CAMPAIGN_ROOT" = /* ]] || die "campaign root must be absolute: $CAMPAIGN_ROOT"
	[[ "$CAMPAIGN_LEDGER" = /* ]] || die "campaign ledger must be absolute: $CAMPAIGN_LEDGER"
	[[ "$READINESS_REPORT" = /* ]] || die "readiness report must be absolute: $READINESS_REPORT"
}

task_dir() {
	local task="${TASK:-}"
	[[ "$task" =~ ^[0-9]{3}$ ]] || die "set TASK=NNN"
	echo "$CATALOG_ROOT/$task"
}

resolve_target_dir() {
	local worktree_root="$1"
	local tc_target_dir="${TC_PROOF_TARGET_DIR:-}"
	local cargo_target="${CARGO_TARGET_DIR:-}"
	python3 - "$worktree_root" "$tc_target_dir" "$cargo_target" <<'PY'
import os
import stat
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
tc_raw, cargo_raw = sys.argv[2:]


def check_parent_components(path: Path, name: str) -> None:
    allowed_system_symlinks = {
        Path("/etc"): Path("/private/etc"),
        Path("/home"): Path("/System/Volumes/Data/home"),
        Path("/tmp"): Path("/private/tmp"),
        Path("/var"): Path("/private/var"),
    }
    current = Path(path.anchor)
    for component in path.parts[1:-1]:
        current /= component
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            break
        except OSError as error:
            raise SystemExit(
                f"{name} parent path component is unreadable: {current}: {error}"
            ) from error
        if stat.S_ISLNK(metadata.st_mode):
            if current.resolve() != allowed_system_symlinks.get(current):
                raise SystemExit(
                    f"{name} parent path component must not be a symlink: {current}"
                )
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise SystemExit(
                f"{name} parent path component is not a directory: {current}"
            )


def checked_path(raw: str, name: str) -> Path:
    path = Path(raw)
    if not path.is_absolute():
        raise SystemExit(f"{name} must be an absolute path: {raw or '<empty>'}")
    check_parent_components(path, name)
    if path.is_symlink():
        raise SystemExit(f"{name} must not be a symlink: {path}")
    return path

if tc_raw and cargo_raw:
    tc_path = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
    cargo_path = checked_path(cargo_raw, "CARGO_TARGET_DIR")
    if tc_path.resolve() != cargo_path.resolve():
        raise SystemExit(
            "TC_PROOF_TARGET_DIR and CARGO_TARGET_DIR select ambiguous target roots"
        )
    selected = tc_path
elif tc_raw:
    selected = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
elif cargo_raw:
    selected = checked_path(cargo_raw, "CARGO_TARGET_DIR")
else:
    selected = root / "target"
    check_parent_components(selected, "proof target directory")

if selected.is_symlink():
    raise SystemExit(f"proof target directory must not be a symlink: {selected}")
if not selected.exists():
    raise SystemExit(f"proof target directory is missing: {selected}")
if not selected.is_dir():
    raise SystemExit(f"proof target path is not a directory: {selected}")

resolved = selected.resolve()
if resolved == root or root in resolved.parents:
    raise SystemExit("proof target directory must be external to the worktree")
print(resolved)
PY
}

proof_binary() {
	local target_dir="$1"
	echo "$target_dir/debug/tc-proof"
}

require_native_proof() {
	local worktree_root="$1"
	local target_dir="$2"
	local binary="$3"
	local receipt commit tree
	[[ -d "$target_dir" && ! -L "$target_dir" ]] ||
		die "native proof target directory missing or linked: $target_dir"
	[[ -d "$target_dir/debug" && ! -L "$target_dir/debug" ]] ||
		die "native proof debug target directory missing or linked: $target_dir/debug"
	[[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] ||
		die "native tc-proof comparator missing: $binary (run scripts/campaign-build-proof.sh with TC_PROOF_TARGET_DIR)"
	receipt="$target_dir/debug/tc-proof.build.json"
	[[ -f "$receipt" && ! -L "$receipt" ]] ||
		die "native tc-proof build receipt missing: $receipt (run scripts/campaign-build-proof.sh)"
	commit="$(git -C "$worktree_root" rev-parse HEAD)"
	tree="$(git -C "$worktree_root" rev-parse 'HEAD^{tree}')"
	python3 - "$receipt" "$worktree_root" "$target_dir" "$commit" "$tree" "$binary" <<'PY' ||
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

receipt, worktree, target, commit, tree, binary = sys.argv[1:]
try:
    with Path(receipt).open(encoding="utf-8") as stream:
        value = json.load(stream)
except (OSError, ValueError) as error:
    raise SystemExit(f"invalid proof build receipt: {error}") from error

if value.get("schema") != "tc-proof-native-build/v1":
    raise SystemExit("wrong build receipt schema")
if value.get("worktree") != str(Path(worktree).resolve()):
    raise SystemExit("wrong build worktree")
expected_target = str(Path(target).resolve())
if value.get("target_dir") != expected_target or value.get("cargo_target_dir") != expected_target:
    raise SystemExit("wrong build target")
if value.get("commit") != commit or value.get("tree") != tree:
    raise SystemExit("wrong build source binding")
if value.get("binary") != str(Path(binary).resolve()):
    raise SystemExit("wrong build binary")
for path, label in (
    (Path(binary), "native comparator"),
    (Path(receipt), "native build receipt"),
):
    try:
        metadata = os.lstat(path)
    except OSError as error:
        raise SystemExit(f"{label} is unreadable: {error}") from error
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise SystemExit(f"{label} is not a regular single-link file")
actual = hashlib.sha256(Path(binary).read_bytes()).hexdigest()
if value.get("binary_sha256") != actual:
    raise SystemExit("native comparator hash mismatch")
PY
		die "native tc-proof build receipt does not match this worktree/target"
}

require_scope_base() {
	[[ "$BASE" =~ ^[0-9a-f]{40}$ ]] ||
		die "set TC_TASK_BASE to the explicit full scope-base commit"
	local resolved
	resolved="$(git -C "$WORKTREE" rev-parse --verify "${BASE}^{commit}" 2>/dev/null)" ||
		die "scope base is not a commit in the verifier worktree: $BASE"
	[[ "$resolved" == "$BASE" ]] ||
		die "scope base must be an explicit full commit, not an abbreviation: $BASE"
}

require_clean_worktree() {
	[[ -z "$(git -C "$WORKTREE" status --porcelain)" ]] ||
		die "subagent worktree has uncommitted changes; verify only a committed task tree"
}

require_external_run_dir() {
	python3 - "$RUN_DIR" "$WORKTREE" <<'PY' || die "verifier run directory is missing, reused, or inside the worktree"
import os
import stat
import sys
from pathlib import Path


def real_directory(path: Path, field: str) -> Path:
    if not path.is_absolute():
        raise SystemExit(f"{field} must be an absolute directory")
    allowed_system_symlinks = {
        Path("/etc"): Path("/private/etc"),
        Path("/home"): Path("/System/Volumes/Data/home"),
        Path("/tmp"): Path("/private/tmp"),
        Path("/var"): Path("/private/var"),
    }
    current = Path(path.anchor)
    for component in path.parts[1:]:
        current /= component
        try:
            metadata = os.lstat(current)
        except OSError as error:
            raise SystemExit(f"{field} is unreadable: {error}") from error
        if stat.S_ISLNK(metadata.st_mode):
            if current.resolve() != allowed_system_symlinks.get(current):
                raise SystemExit(f"{field} contains a symlinked path component: {current}")
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise SystemExit(f"{field} must be an existing real directory")
    return path.resolve()


run = real_directory(Path(sys.argv[1]), "run directory")
worktree = real_directory(Path(sys.argv[2]), "candidate worktree")
if run == worktree or worktree in run.parents:
    raise SystemExit("run directory must be external to the candidate worktree")
PY
}

require_taskfmt() {
	require_absolute_paths
	campaign_require_regular_file "$TASKFMT" "taskfmt" 1 1 ||
		die "taskfmt path failed trust-path validation: $TASKFMT"
	local actual_sha
	actual_sha="$(shasum -a 256 "$TASKFMT" | awk '{print $1}')"
	[[ "$actual_sha" == "$TASKFMT_SHA256" ]] ||
		die "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
	"$TASKFMT" --version | grep -Fq "git $TASKFMT_REV" ||
		die "taskfmt is not latest $TASKFMT_REV"
	[[ -d "$TASKFMT_SOURCE/.git" ]] ||
		die "taskfmt source is not a git checkout: $TASKFMT_SOURCE"
	[[ -z "$(git -C "$TASKFMT_SOURCE" status --porcelain)" ]] ||
		die "taskfmt source is dirty: $TASKFMT_SOURCE"
	[[ "$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)" == "$TASKFMT_REV" ]] ||
		die "taskfmt source is not latest $TASKFMT_REV"
}

require_dispatch_authorization() {
	local task_id campaign_branch campaign_commit campaign_tree candidate_commit candidate_tree
	task_id="TASK-${TASK:-}"
	[[ "$task_id" =~ ^TASK-[0-9]{3}$ ]] || die "set TASK=NNN"
	[[ -n "$WORKTREE" && "$WORKTREE" = /* ]] ||
		die "set TC_TASK_WORKTREE to an absolute isolated worktree"
	[[ -d "$WORKTREE" ]] || die "subagent worktree missing: $WORKTREE"
	[[ -n "$RUN_DIR" && "$RUN_DIR" = /* ]] ||
		die "set TC_TASK_RUN_DIR to a unique absolute verifier run directory"
	[[ -n "$PREFLIGHT_EVIDENCE" && "$PREFLIGHT_EVIDENCE" = /* ]] ||
		die "set TC_TASK_PREFLIGHT_EVIDENCE to an external authorization manifest"

	campaign_branch="$(git -C "$CAMPAIGN_ROOT" branch --show-current 2>/dev/null)" ||
		die "campaign root is not a Git worktree: $CAMPAIGN_ROOT"
	[[ "$campaign_branch" == "$INTEGRATION_BRANCH" ]] ||
		die "campaign root is on '$campaign_branch', expected '$INTEGRATION_BRANCH'"
	campaign_commit="$(git -C "$CAMPAIGN_ROOT" rev-parse HEAD 2>/dev/null)" ||
		die "campaign root has no resolvable HEAD: $CAMPAIGN_ROOT"
	campaign_tree="$(git -C "$CAMPAIGN_ROOT" rev-parse 'HEAD^{tree}' 2>/dev/null)" ||
		die "campaign root has no resolvable HEAD tree: $CAMPAIGN_ROOT"
	candidate_commit="$(git -C "$WORKTREE" rev-parse HEAD 2>/dev/null)" ||
		die "candidate worktree is not a Git worktree: $WORKTREE"
	candidate_tree="$(git -C "$WORKTREE" rev-parse 'HEAD^{tree}' 2>/dev/null)" ||
		die "candidate worktree has no resolvable HEAD tree: $WORKTREE"

	PYTHONPATH="$SCRIPT_DIR" python3 - \
		"$CAMPAIGN_ROOT" "$CAMPAIGN_LEDGER" "$READINESS_REPORT" \
		"$PREFLIGHT_EVIDENCE" "$RUN_DIR" "$WORKTREE" "$INTEGRATION_BRANCH" \
		"$campaign_branch" "$campaign_commit" "$campaign_tree" "$candidate_commit" \
		"$candidate_tree" "$task_id" "$BASE" "$TASKFMT_REV" "$TASKFMT_VERSION" \
		"$TASKFMT_SHA256" "$TASKFMT_SOURCE" "$TASKFMT" <<'PY' ||
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

from campaign_ledger import validate_ledger_schema, validate_taskfmt_binding

ORACLE_TAG = "refs/tags/visual-baseline"
ORACLE_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
ORACLE_TREE = "0b1f13431fdfd6060cf9f45a114afa5a99cc6c26"
CATALOG_MANIFEST_REL = "docs/refactoring-plan/task-index.tsv"

(
    campaign_root,
    ledger_path,
    readiness_path,
    authorization_path,
    run_dir,
    candidate_root,
    integration_branch,
    campaign_branch,
    campaign_commit,
    campaign_tree,
    candidate_commit,
    candidate_tree,
    task_id,
    scope_base,
    taskfmt_revision,
    taskfmt_version,
    taskfmt_sha256,
    taskfmt_source,
    taskfmt_path,
) = sys.argv[1:]


def fail(message: str) -> None:
    raise SystemExit(message)


def regular(path: Path, field: str) -> Path:
    if not path.is_absolute():
        fail(f"{field} is not an absolute non-symlink regular file: {path}")
    allowed_system_symlinks = {
        Path("/etc"): Path("/private/etc"),
        Path("/home"): Path("/System/Volumes/Data/home"),
        Path("/tmp"): Path("/private/tmp"),
        Path("/var"): Path("/private/var"),
    }
    current = Path(path.anchor)
    for component in path.parts[1:]:
        current /= component
        try:
            metadata = os.lstat(current)
        except OSError as error:
            fail(f"{field} is unreadable: {error}")
        if stat.S_ISLNK(metadata.st_mode):
            if current.resolve() != allowed_system_symlinks.get(current):
                fail(f"{field} contains a symlinked path component: {current}")
            continue
        if not stat.S_ISDIR(metadata.st_mode) and current != path:
            fail(f"{field} contains a non-directory path component: {current}")
    try:
        metadata = path.lstat()
    except OSError as error:
        fail(f"{field} is unreadable: {error}")
    if path.is_symlink() or not path.is_file() or metadata.st_nlink != 1:
        fail(f"{field} is not an absolute non-symlink regular file: {path}")
    return path.resolve()


def directory(path: Path, field: str) -> Path:
    if not path.is_absolute():
        fail(f"{field} is not an absolute non-symlink directory: {path}")
    allowed_system_symlinks = {
        Path("/etc"): Path("/private/etc"),
        Path("/home"): Path("/System/Volumes/Data/home"),
        Path("/tmp"): Path("/private/tmp"),
        Path("/var"): Path("/private/var"),
    }
    current = Path(path.anchor)
    for component in path.parts[1:]:
        current /= component
        try:
            metadata = os.lstat(current)
        except OSError as error:
            fail(f"{field} is unreadable: {error}")
        if stat.S_ISLNK(metadata.st_mode):
            if current.resolve() != allowed_system_symlinks.get(current):
                fail(f"{field} contains a symlinked path component: {current}")
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            fail(f"{field} is not an absolute non-symlink directory: {path}")
    return path.resolve()


def outside(path: Path, parent: Path, field: str) -> None:
    try:
        path.relative_to(parent)
    except ValueError:
        return
    fail(f"{field} must be outside {parent}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def json_file(path: Path, field: str) -> dict:
    regular(path, field)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"{field} is not valid JSON: {error}")
    if not isinstance(value, dict):
        fail(f"{field} must be a JSON object")
    return value


campaign_root = directory(Path(campaign_root), "campaign root")
candidate_root = directory(Path(candidate_root), "candidate worktree")
run_dir = directory(Path(run_dir), "verifier run directory")
authorization_path = regular(Path(authorization_path), "dispatch authorization")
outside(run_dir, candidate_root, "verifier run directory")
outside(authorization_path, candidate_root, "dispatch authorization")
outside(authorization_path, campaign_root, "dispatch authorization")

ledger_path = regular(Path(ledger_path), "campaign ledger")
readiness_path = regular(Path(readiness_path), "readiness report")

readiness = readiness_path.read_text(encoding="utf-8")
readiness_lines = {line.strip() for line in readiness.splitlines()}
if "**NO-GO.**" in readiness_lines or "**GO.**" not in readiness_lines:
    fail("current readiness report is not exact GO")

ledger = json_file(ledger_path, "campaign ledger")
try:
    validate_ledger_schema(ledger)
except Exception as error:
    fail(f"campaign ledger schema invalid: {error}")

expected_ref = "refs/heads/" + integration_branch
if campaign_branch != integration_branch:
    fail("campaign branch identity is not the dispatch branch")
if ledger["integration_ref"] != expected_ref:
    fail("campaign ledger integration_ref is not the dispatch branch")
if ledger["integration_head"] != campaign_commit:
    fail("campaign ledger integration_head is stale")
if ledger["catalog"]["branch"] != integration_branch:
    fail("campaign ledger catalog branch is stale")
catalog_manifest = regular(
    campaign_root / CATALOG_MANIFEST_REL,
    "catalog manifest",
)
expected_catalog = {
    "commit": campaign_commit,
    "tree": campaign_tree,
    "branch": integration_branch,
    "manifest": {
        "path": CATALOG_MANIFEST_REL,
        "sha256": digest(catalog_manifest),
    },
}
for key, value in expected_catalog.items():
    if ledger["catalog"].get(key) != value:
        fail(f"campaign ledger catalog {key} is stale or mismatched")
if ledger["armed"] is not True:
    fail("campaign ledger is disarmed; explicit dispatch arming is required")
if "armed_at" not in ledger:
    fail("campaign ledger armed=true has no armed_at timestamp")

expected_taskfmt = {
    "taskfmt_revision": taskfmt_revision,
    "taskfmt_version": taskfmt_version,
    "taskfmt_sha256": taskfmt_sha256,
    "taskfmt_source": taskfmt_source,
    "taskfmt_path": str(Path(taskfmt_path).resolve()),
}
try:
    validate_taskfmt_binding(ledger, expected_taskfmt)
except Exception as error:
    fail(f"campaign ledger taskfmt binding is stale: {error}")

authorization = json_file(authorization_path, "dispatch authorization")
allowed = {
    "schema",
    "authorization",
    "operation",
    "integration_ref",
    "campaign_root",
    "campaign_commit",
    "campaign_tree",
    "candidate_root",
    "candidate_commit",
    "candidate_tree",
    "task_id",
    "scope_base",
    "readiness",
    "ledger",
    "taskfmt",
    "preflight",
}
unknown = sorted(set(authorization) - allowed)
if unknown:
    fail("dispatch authorization has unknown fields: " + ", ".join(unknown))
if authorization.get("schema") != "campaign-dispatch-authorization/v1":
    fail("dispatch authorization schema is invalid")
if authorization.get("authorization") != "AUTHORIZED":
    fail("dispatch authorization is not AUTHORIZED")
if authorization.get("operation") != "taskfmt-verify":
    fail("dispatch authorization operation is not taskfmt-verify")

expected = {
    "integration_ref": expected_ref,
    "campaign_root": str(campaign_root),
    "campaign_commit": campaign_commit,
    "campaign_tree": campaign_tree,
    "candidate_root": str(candidate_root),
    "candidate_commit": candidate_commit,
    "candidate_tree": candidate_tree,
    "task_id": task_id,
    "scope_base": scope_base,
}
for key, value in expected.items():
    if authorization.get(key) != value:
        fail(f"dispatch authorization {key} is stale or mismatched")

readiness_binding = authorization.get("readiness")
if not isinstance(readiness_binding, dict):
    fail("dispatch authorization readiness binding is missing")
if readiness_binding != {
    "path": str(readiness_path),
    "sha256": digest(readiness_path),
    "verdict": "GO",
}:
    fail("dispatch authorization readiness binding is stale")

ledger_binding = authorization.get("ledger")
if not isinstance(ledger_binding, dict):
    fail("dispatch authorization ledger binding is missing")
if ledger_binding != {
    "path": str(ledger_path),
    "sha256": digest(ledger_path),
    "integration_head": campaign_commit,
    "armed": True,
}:
    fail("dispatch authorization ledger binding is stale")

taskfmt_binding = authorization.get("taskfmt")
if not isinstance(taskfmt_binding, dict):
    fail("dispatch authorization taskfmt binding is missing")
if taskfmt_binding != expected_taskfmt:
    fail("dispatch authorization taskfmt binding is stale")

preflight = authorization.get("preflight")
if not isinstance(preflight, dict):
    fail("dispatch authorization preflight evidence is missing")
if set(preflight) != {"command", "exit", "evidence", "evidence_sha256"}:
    fail("dispatch authorization preflight evidence shape is invalid")
if preflight["exit"] != 0 or preflight["command"] != "scripts/campaign-preflight.sh preflight":
    fail("dispatch authorization preflight did not pass")
preflight_evidence = regular(Path(preflight["evidence"]), "preflight evidence")
outside(preflight_evidence, candidate_root, "preflight evidence")
outside(preflight_evidence, campaign_root, "preflight evidence")
if digest(preflight_evidence) != preflight["evidence_sha256"]:
    fail("dispatch authorization preflight evidence hash is stale")
if preflight_evidence == authorization_path:
    fail("dispatch authorization cannot cite itself as preflight evidence")

report = json_file(preflight_evidence, "preflight report")
report_allowed = {
    "schema",
    "verdict",
    "operation",
    "command",
    "exit",
    "integration_ref",
    "source",
    "oracle",
    "catalog",
    "taskfmt",
    "ledger",
    "checks",
    "recorded_at",
}
if set(report) != report_allowed:
    fail("preflight report shape is invalid")
if report["schema"] != "campaign-preflight-report/v1":
    fail("preflight report schema is invalid")
if report["verdict"] != "PASS" or report["operation"] != "preflight":
    fail("preflight report is not a passing preflight")
if report["command"] != preflight["command"] or report["exit"] != preflight["exit"]:
    fail("preflight report command/exit is not bound to authorization")
if report["integration_ref"] != expected_ref:
    fail("preflight report integration_ref is stale")
if report["source"] != {
    "root": str(campaign_root),
    "commit": campaign_commit,
    "tree": campaign_tree,
}:
    fail("preflight report source/tree binding is stale")
if report["oracle"] != {
    "tag": ORACLE_TAG,
    "commit": ORACLE_COMMIT,
    "tree": ORACLE_TREE,
}:
    fail("preflight report frozen oracle binding is stale")
if report["catalog"] != expected_catalog:
    fail("preflight report catalog binding is stale")
if report["taskfmt"] != expected_taskfmt:
    fail("preflight report taskfmt binding is stale")
ledger_report = report["ledger"]
if not isinstance(ledger_report, dict) or set(ledger_report) != {
    "path", "sha256", "snapshot", "integration_head", "armed"
}:
    fail("preflight report ledger binding is invalid")
if ledger_report["path"] != str(ledger_path):
    fail("preflight report ledger path is stale")
if not isinstance(ledger_report["sha256"], str) or not all(
    character in "0123456789abcdef" for character in ledger_report["sha256"]
) or len(ledger_report["sha256"]) != 64:
    fail("preflight report ledger hash is invalid")
if ledger_report["integration_head"] != campaign_commit or ledger_report["armed"] is not False:
    fail("preflight report ledger state is stale or armed")
snapshot = ledger_report["snapshot"]
if not isinstance(snapshot, dict) or set(snapshot) != {"path", "sha256"}:
    fail("preflight report ledger snapshot binding is invalid")
snapshot_path = regular(Path(snapshot["path"]), "preflight ledger snapshot")
outside(snapshot_path, candidate_root, "preflight ledger snapshot")
outside(snapshot_path, campaign_root, "preflight ledger snapshot")
if snapshot_path == preflight_evidence or snapshot_path == authorization_path:
    fail("preflight ledger snapshot cannot reuse report or authorization")
if snapshot["sha256"] != ledger_report["sha256"]:
    fail("preflight ledger snapshot hash is not bound to report")
if digest(snapshot_path) != ledger_report["sha256"]:
    fail("preflight ledger snapshot hash is forged or stale")
snapshot_ledger = json_file(snapshot_path, "preflight ledger snapshot")
try:
    validate_ledger_schema(snapshot_ledger)
    validate_taskfmt_binding(snapshot_ledger, expected_taskfmt)
except Exception as error:
    fail(f"preflight ledger snapshot is invalid: {error}")
if snapshot_ledger["integration_ref"] != expected_ref:
    fail("preflight ledger snapshot integration_ref is stale")
if snapshot_ledger["integration_head"] != campaign_commit:
    fail("preflight ledger snapshot integration_head is stale")
if snapshot_ledger["armed"] is not False:
    fail("preflight ledger snapshot is armed")
if {
    "commit": snapshot_ledger["catalog"]["commit"],
    "tree": snapshot_ledger["catalog"]["tree"],
    "branch": snapshot_ledger["catalog"]["branch"],
    "manifest": snapshot_ledger["catalog"]["manifest"],
} != expected_catalog:
    fail("preflight ledger snapshot catalog is stale")
expected_checks = {
    "tag", "branch", "worktree", "readiness", "ledger", "taskfmt",
    "host_local_paths", "harness", "native_proof", "plan",
}
if report["checks"] != {name: "PASS" for name in expected_checks}:
    fail("preflight report checks are incomplete or not passing")
try:
    from datetime import datetime, timezone
    recorded_at = datetime.fromisoformat(report["recorded_at"].replace("Z", "+00:00"))
    if recorded_at.tzinfo is None or recorded_at.astimezone(timezone.utc) > datetime.now(timezone.utc):
        fail("preflight report recorded_at is invalid or from the future")
except (AttributeError, TypeError, ValueError):
    fail("preflight report recorded_at is invalid")

print(
    "dispatch authorization: GO, current campaign ledger, explicit armed state, "
    "candidate, task, scope base, taskfmt, and preflight evidence bound"
)
PY
		die "dispatch authorization is missing, stale, unsafe, or invalid"
}

prepare_native_contexts() {
	local dir="$1"
	local binary="$2"
	local -a command=(
		"$binary" prepare
		--task-dir "$dir"
		--run-dir "$RUN_DIR"
		--worktree "$WORKTREE"
		--scope-base "$BASE"
		--oracle-tag "$ORACLE_TAG"
		--oracle-commit "$ORACLE_COMMIT"
		--tool "$WORKTREE/tools/refactor-proof/bin/tc-proof"
		--comparator "$binary"
		--native-build-receipt "$(dirname "$binary")/tc-proof.build.json"
		--taskfmt "$TASKFMT"
		--taskfmt-source "$TASKFMT_SOURCE"
		--taskfmt-revision "$TASKFMT_REV"
		--taskfmt-version "$TASKFMT_VERSION"
		--taskfmt-sha256 "$TASKFMT_SHA256"
	)
	if [[ -n "${TC_TASK_DEPENDENCY_RECEIPTS:-}" ]]; then
		local receipt
		local -a receipts
		IFS=: read -r -a receipts <<<"$TC_TASK_DEPENDENCY_RECEIPTS"
		for receipt in "${receipts[@]}"; do
			[[ -n "$receipt" ]] || die "dependency receipt list contains an empty path"
			command+=(--dependency-receipt "$receipt")
		done
	fi
	"${command[@]}" >/dev/null || die "native verifier preparation failed"
}

cmd_lint() {
	require_taskfmt
	local dir
	dir="$(task_dir)"
	export TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
	"$TASKFMT" lint "$dir"
}

cmd_verify() {
	local dir binary target_dir worktree_root
	require_absolute_paths
	dir="$(task_dir)"
	[[ -n "$WORKTREE" && "$WORKTREE" = /* ]] ||
		die "set TC_TASK_WORKTREE to an absolute isolated worktree"
	[[ -d "$WORKTREE" ]] || die "subagent worktree missing: $WORKTREE"
	[[ -n "$RUN_DIR" && "$RUN_DIR" = /* ]] ||
		die "set TC_TASK_RUN_DIR to a unique absolute verifier run directory"
	worktree_root="$(git -C "$WORKTREE" rev-parse --show-toplevel)" ||
		die "candidate worktree is not a Git worktree: $WORKTREE"
	require_scope_base
	require_external_run_dir
	require_clean_worktree
	require_taskfmt
	require_dispatch_authorization
	target_dir="$(resolve_target_dir "$worktree_root")" ||
		die "native proof target directory is invalid"
	binary="$(proof_binary "$target_dir")"
	export TC_PROOF_TARGET_DIR="$target_dir" CARGO_TARGET_DIR="$target_dir"
	require_native_proof "$worktree_root" "$target_dir" "$binary"
	prepare_native_contexts "$dir" "$binary"
	export TC_PROOF_CONTEXT_INDEX="$RUN_DIR/context-index.json"
	local context_index_sha256
	context_index_sha256="$(shasum -a 256 "$TC_PROOF_CONTEXT_INDEX" | awk '{print $1}')"
	export TC_PROOF_CONTEXT_INDEX_SHA256="$context_index_sha256"
	export RUN_DIR TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
	export TC_PROOF_NATIVE_LAUNCH=1
	export TC_PROOF_NATIVE_BINARY="$binary"
	export TC_PROOF_NATIVE_LAUNCHER="$binary"
	export TC_PROOF_NATIVE_TIMEOUT_MS="${TC_PROOF_NATIVE_TIMEOUT_MS:-600000}"

	local taskfmt_status=0
	"$TASKFMT" verify \
		--root "$WORKTREE" \
		--task-dir "$dir" \
		--base "$BASE" \
		--progress "" \
		--log-dir "$RUN_DIR/taskfmt-logs" ||
		taskfmt_status=$?

	local validate_status=0
	"$binary" validate --run-dir "$RUN_DIR" || validate_status=$?
	if ((taskfmt_status != 0)); then
		return "$taskfmt_status"
	fi
	((validate_status == 0)) || die "native proof validation failed after taskfmt"
}

main() {
	case "${1:-help}" in
	lint) cmd_lint ;;
	verify) cmd_verify ;;
	help | -h | --help) usage ;;
	*) die "unknown command: ${1:-}" ;;
	esac
}

main "$@"
