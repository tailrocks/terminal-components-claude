#!/usr/bin/env bash
# Fail-closed authorization checks for direct taskfmt verification dispatch.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
DISPATCH="$SCRIPT_DIR/campaign-dispatch.sh"
TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_VERSION="0.2.0"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"

fail() {
	echo "test_campaign_dispatch_authorization: FAIL: $*" >&2
	exit 1
}

pass() {
	echo "test_campaign_dispatch_authorization: PASS: $*"
}

TMP_ROOT=""
if TMP_ROOT="$(mktemp -d /private/tmp/tc-dispatch-auth.XXXXXX 2>/dev/null)"; then
	:
else
	TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/tc-dispatch-auth.XXXXXX")"
fi

cleanup() {
	rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

campaign="$TMP_ROOT/campaign"
candidate="$TMP_ROOT/candidate"
catalog="$TMP_ROOT/catalog"
target="$TMP_ROOT/target"
mkdir -p "$campaign/docs/refactoring-plan" "$campaign/.campaign" \
	"$candidate" "$catalog/002" "$candidate/tools/refactor-proof/bin" "$target/debug"
printf '%s\n' 'catalog manifest' >"$campaign/docs/refactoring-plan/task-index.tsv"

git -C "$campaign" init -q
git -C "$campaign" config user.email test@example.invalid
git -C "$campaign" config user.name "Dispatch Authorization Test"
git -C "$campaign" checkout -q -b refactor/holla-parity
printf '%s\n' 'campaign fixture' >"$campaign/README.md"
printf '%s\n' '**NO-GO.**' >"$campaign/docs/refactoring-plan/execution-readiness-report.md"
git -C "$campaign" add README.md docs/refactoring-plan/execution-readiness-report.md
git -C "$campaign" commit -q -m fixture

git -C "$candidate" init -q
git -C "$candidate" config user.email test@example.invalid
git -C "$candidate" config user.name "Dispatch Authorization Test"
git -C "$candidate" checkout -q -b candidate
printf '%s\n' 'candidate fixture' >"$candidate/README.md"
printf '%s\n' '#!/usr/bin/env bash' 'set -euo pipefail' 'exit 0' \
	>"$candidate/tools/refactor-proof/bin/tc-proof"
chmod +x "$candidate/tools/refactor-proof/bin/tc-proof"
git -C "$candidate" add README.md
git -C "$candidate" add tools/refactor-proof/bin/tc-proof
git -C "$candidate" commit -q -m fixture

cp "$SCRIPT_DIR/../refactoring-tasks/terminal-components/completion/002/verify.toml" "$catalog/002/verify.toml"
cp "$SCRIPT_DIR/../refactoring-tasks/terminal-components/completion/002/README.md" "$catalog/002/README.md"

python3 - "$campaign" "$candidate" "$TASKFMT" "$TASKFMT_REV" "$TASKFMT_VERSION" "$TASKFMT_SHA256" <<'PY'
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ORACLE_TAG = "refs/tags/visual-baseline"
ORACLE_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
ORACLE_TREE = "0b1f13431fdfd6060cf9f45a114afa5a99cc6c26"
CATALOG_MANIFEST_REL = "docs/refactoring-plan/task-index.tsv"

campaign = Path(sys.argv[1])
candidate = Path(sys.argv[2])
taskfmt = Path(sys.argv[3])
revision, version, taskfmt_sha = sys.argv[4:]
campaign = campaign.resolve()
candidate = candidate.resolve()
taskfmt = taskfmt.resolve()

campaign_head = __import__("subprocess").check_output(
    ["git", "-C", str(campaign), "rev-parse", "HEAD"], text=True
).strip()
candidate_head = __import__("subprocess").check_output(
    ["git", "-C", str(candidate), "rev-parse", "HEAD"], text=True
).strip()
campaign_tree = __import__("subprocess").check_output(
    ["git", "-C", str(campaign), "rev-parse", "HEAD^{tree}"], text=True
).strip()
candidate_tree = __import__("subprocess").check_output(
    ["git", "-C", str(candidate), "rev-parse", "HEAD^{tree}"], text=True
).strip()

ledger = {
    "schema": "campaign-ledger/v1",
    "integration_ref": "refs/heads/refactor/holla-parity",
    "integration_head": campaign_head,
    "armed": True,
    "armed_at": "2026-09-19T00:00:00Z",
    "catalog": {
    "commit": campaign_head,
        "tree": campaign_tree,
        "branch": "refactor/holla-parity",
        "manifest": {
            "path": "docs/refactoring-plan/task-index.tsv",
            "sha256": hashlib.sha256((campaign / "docs/refactoring-plan/task-index.tsv").read_bytes()).hexdigest(),
        },
        "recorded_at": "2026-09-19T00:00:00Z",
    },
    "toolchain": {
        "taskfmt_revision": revision,
        "taskfmt_version": version,
        "taskfmt_sha256": taskfmt_sha,
        "taskfmt_source": "/Users/donbeave/Projects/taskfmt/task-format",
        "taskfmt_path": str(taskfmt),
    },
    "receipts": {},
    "tasks": [],
}
ledger_path = campaign / ".campaign" / "ledger.json"
ledger_path.write_text(json.dumps(ledger, indent=2, sort_keys=True) + "\n", encoding="utf-8")

run = campaign.parent / "positive-run"
run.mkdir()
preflight = run / "preflight-report.json"
readiness = campaign / "docs/refactoring-plan/execution-readiness-report.md"

sha256 = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
preflight_ledger = run / "preflight-ledger.json"
preflight_ledger_value = json.loads(ledger_path.read_text(encoding="utf-8"))
preflight_ledger_value["armed"] = False
preflight_ledger_value.pop("armed_at", None)
preflight_ledger.write_text(json.dumps(preflight_ledger_value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
preflight_report = {
    "schema": "campaign-preflight-report/v1",
    "verdict": "PASS",
    "operation": "preflight",
    "command": "scripts/campaign-preflight.sh preflight",
    "exit": 0,
    "integration_ref": "refs/heads/refactor/holla-parity",
    "source": {"root": str(campaign), "commit": campaign_head, "tree": campaign_tree},
    "oracle": {"tag": ORACLE_TAG, "commit": ORACLE_COMMIT, "tree": ORACLE_TREE},
    "catalog": {
        "commit": campaign_head,
        "tree": campaign_tree,
        "branch": "refactor/holla-parity",
        "manifest": {
            "path": CATALOG_MANIFEST_REL,
            "sha256": sha256(campaign / CATALOG_MANIFEST_REL),
        },
    },
    "taskfmt": {
        "taskfmt_revision": revision,
        "taskfmt_version": version,
        "taskfmt_sha256": taskfmt_sha,
        "taskfmt_source": "/Users/donbeave/Projects/taskfmt/task-format",
        "taskfmt_path": str(taskfmt),
    },
    "ledger": {
        "path": str(ledger_path),
        "sha256": sha256(preflight_ledger),
        "snapshot": {
            "path": str(preflight_ledger),
            "sha256": sha256(preflight_ledger),
        },
        "integration_head": campaign_head,
        "armed": False,
    },
    "checks": {
        "tag": "PASS", "branch": "PASS", "worktree": "PASS", "readiness": "PASS",
        "ledger": "PASS", "taskfmt": "PASS", "host_local_paths": "PASS",
        "harness": "PASS", "native_proof": "PASS", "plan": "PASS",
    },
    "recorded_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
}
preflight.write_text(json.dumps(preflight_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
authorization = {
    "schema": "campaign-dispatch-authorization/v1",
    "authorization": "AUTHORIZED",
    "operation": "taskfmt-verify",
    "integration_ref": "refs/heads/refactor/holla-parity",
    "campaign_root": str(campaign),
    "campaign_commit": campaign_head,
    "campaign_tree": campaign_tree,
    "candidate_root": str(candidate),
    "candidate_commit": candidate_head,
    "candidate_tree": candidate_tree,
    "task_id": "TASK-002",
    "scope_base": candidate_head,
    "readiness": {
        "path": str(readiness),
        "sha256": sha256(readiness),
        "verdict": "GO",
    },
    "ledger": {
        "path": str(ledger_path),
        "sha256": sha256(ledger_path),
        "integration_head": campaign_head,
        "armed": True,
    },
    "taskfmt": {
        "taskfmt_revision": revision,
        "taskfmt_version": version,
        "taskfmt_sha256": taskfmt_sha,
        "taskfmt_source": "/Users/donbeave/Projects/taskfmt/task-format",
        "taskfmt_path": str(taskfmt),
    },
    "preflight": {
        "command": "scripts/campaign-preflight.sh preflight",
        "exit": 0,
        "evidence": str(preflight),
        "evidence_sha256": sha256(preflight),
    },
}
(run / "authorization.json").write_text(
    json.dumps(authorization, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
PY

expect_reject() {
	local label="$1"
	local expected="$2"
	local run_dir="$TMP_ROOT/run-$label"
	mkdir "$run_dir"
	local output
	if output="$({
		TASK=002 \
			TC_CAMPAIGN_ROOT="$campaign" \
			TC_TASK_WORKTREE="$candidate" \
			TC_TASK_RUN_DIR="$run_dir" \
			TC_TASK_BASE="$(git -C "$candidate" rev-parse HEAD)" \
			TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
			TC_CATALOG_ROOT="$catalog" \
			"$DISPATCH" verify
	} 2>&1)"; then
		fail "$label was accepted"
	fi
	grep -Fq "$expected" <<<"$output" ||
		fail "$label had unexpected rejection: $output"
	pass "$label rejected: $expected"
}

expect_reject "no-go" "readiness report is not exact GO"

python3 - "$campaign/docs/refactoring-plan/execution-readiness-report.md" "$TMP_ROOT/positive-run/authorization.json" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
readiness = Path(sys.argv[1])
authorization = Path(sys.argv[2])
readiness.write_text("**GO.**\n", encoding="utf-8")
value = json.loads(authorization.read_text(encoding="utf-8"))
value["readiness"]["sha256"] = hashlib.sha256(readiness.read_bytes()).hexdigest()
authorization.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

python3 - "$campaign/.campaign/ledger.json" <<'PY'
import json
import sys
from pathlib import Path
path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
value["armed"] = False
value.pop("armed_at", None)
path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
expect_reject "disarmed" "campaign ledger is disarmed"

python3 - "$campaign/.campaign/ledger.json" "$campaign" <<'PY'
import json
import subprocess
import sys
from pathlib import Path
ledger_path, campaign = map(Path, sys.argv[1:])
value = json.loads(ledger_path.read_text(encoding="utf-8"))
value["armed"] = True
value["armed_at"] = "2026-09-19T00:00:00Z"
value["integration_head"] = "0" * 40
ledger_path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
expect_reject "stale-ledger" "campaign ledger integration_head is stale"

python3 - "$campaign/.campaign/ledger.json" "$campaign" <<'PY'
import json
import subprocess
import sys
from pathlib import Path
ledger_path, campaign = map(Path, sys.argv[1:])
value = json.loads(ledger_path.read_text(encoding="utf-8"))
value["integration_head"] = subprocess.check_output(
    ["git", "-C", str(campaign), "rev-parse", "HEAD"], text=True
).strip()
ledger_path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

preflight_report="$TMP_ROOT/positive-run/preflight-report.json"
preflight_report_good="$TMP_ROOT/positive-run/preflight-report.good.json"
preflight_ledger_good="$TMP_ROOT/positive-run/preflight-ledger.good.json"
cp "$preflight_report" "$preflight_report_good"
cp "$TMP_ROOT/positive-run/preflight-ledger.json" "$preflight_ledger_good"

expect_report_reject() {
	local label="$1"
	local expected="$2"
	cp "$preflight_report_good" "$preflight_report"
	cp "$preflight_ledger_good" "$TMP_ROOT/positive-run/preflight-ledger.json"
	python3 - "$preflight_report" "$TMP_ROOT/positive-run/authorization.json" "$label" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

report_path, authorization_path = map(Path, sys.argv[1:3])
label = sys.argv[3]
report = json.loads(report_path.read_text(encoding="utf-8"))
if label == "source":
    report["source"]["tree"] = "0" * 40
elif label == "oracle":
    report["oracle"]["tree"] = "1" * 40
elif label == "taskfmt":
    report["taskfmt"]["taskfmt_sha256"] = "2" * 64
elif label == "ledger":
    report["ledger"]["armed"] = True
elif label == "ledger-hash":
    report["ledger"]["sha256"] = "3" * 64
elif label == "ledger-path":
    report["ledger"]["path"] = str(report_path)
elif label == "ledger-missing":
    report["ledger"]["snapshot"]["path"] = str(report_path.parent / "missing-ledger.json")
elif label == "ledger-forged":
    snapshot_path = Path(report["ledger"]["snapshot"]["path"])
    snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
    snapshot["armed"] = True
    snapshot_path.write_text(json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    forged_hash = hashlib.sha256(snapshot_path.read_bytes()).hexdigest()
    report["ledger"]["sha256"] = forged_hash
    report["ledger"]["snapshot"]["sha256"] = forged_hash
elif label == "checks":
    report["checks"].pop("plan")
else:
    raise SystemExit(f"unknown report mutation {label}")
report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
authorization = json.loads(authorization_path.read_text(encoding="utf-8"))
authorization["preflight"]["evidence_sha256"] = hashlib.sha256(report_path.read_bytes()).hexdigest()
authorization_path.write_text(json.dumps(authorization, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
	expect_reject "report-$label" "$expected"
}

expect_report_reject "source" "source/tree binding"
expect_report_reject "oracle" "frozen oracle binding"
expect_report_reject "taskfmt" "taskfmt binding"
expect_report_reject "ledger" "ledger state"
expect_report_reject "ledger-hash" "snapshot hash"
expect_report_reject "ledger-path" "ledger path is stale"
expect_report_reject "ledger-missing" "preflight ledger snapshot"
expect_report_reject "ledger-forged" "snapshot is armed"
expect_report_reject "checks" "checks are incomplete"
cp "$preflight_report_good" "$preflight_report"
cp "$preflight_ledger_good" "$TMP_ROOT/positive-run/preflight-ledger.json"
python3 - "$TMP_ROOT/positive-run/authorization.json" "$preflight_report" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

authorization_path, report_path = map(Path, sys.argv[1:])
authorization = json.loads(authorization_path.read_text(encoding="utf-8"))
authorization["preflight"]["evidence_sha256"] = hashlib.sha256(report_path.read_bytes()).hexdigest()
authorization_path.write_text(json.dumps(authorization, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

binary="$target/debug/tc-proof"
python3 - "$binary" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text(
    "#!/usr/bin/env bash\n"
    "set -euo pipefail\n"
    "mode=${1:-}\n"
    "shift || true\n"
    "run_dir=\n"
    "while (($#)); do\n"
    "  if [[ $1 == --run-dir ]]; then run_dir=$2; shift 2; else shift; fi\n"
    "done\n"
    "if [[ $mode == prepare ]]; then\n"
    "  printf '%s\\n' '{}' > \"$run_dir/context-index.json\"\n"
    "elif [[ $mode != validate ]]; then\n"
    "  exit 2\n"
    "fi\n",
    encoding="utf-8",
)
path.chmod(0o755)
PY

candidate_head="$(git -C "$candidate" rev-parse HEAD)"
candidate_tree="$(git -C "$candidate" rev-parse 'HEAD^{tree}')"
binary_sha="$(shasum -a 256 "$binary" | awk '{print $1}')"
python3 - "$target/debug/tc-proof.build.json" "$candidate" "$target" "$binary" "$candidate_head" "$candidate_tree" "$binary_sha" <<'PY'
import json
import sys
from pathlib import Path
receipt, candidate, target, binary, commit, tree, binary_sha = sys.argv[1:]
value = {
    "schema": "tc-proof-native-build/v1",
    "worktree": str(Path(candidate).resolve()),
    "target_dir": str(Path(target).resolve()),
    "cargo_target_dir": str(Path(target).resolve()),
    "commit": commit,
    "tree": tree,
    "binary": str(Path(binary).resolve()),
    "binary_sha256": binary_sha,
}
Path(receipt).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

linked_target_parent="$TMP_ROOT/linked-target-parent"
ln -s "$TMP_ROOT" "$linked_target_parent"
set +e
TASK=002 \
	TC_CAMPAIGN_ROOT="$campaign" \
	TC_TASK_WORKTREE="$candidate" \
	TC_TASK_RUN_DIR="$TMP_ROOT/positive-run" \
	TC_TASK_BASE="$candidate_head" \
	TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
	TC_CATALOG_ROOT="$catalog" \
	TC_PROOF_TARGET_DIR="$linked_target_parent/target" \
	CARGO_TARGET_DIR="" \
	TC_TASKFMT="$TASKFMT" \
	"$DISPATCH" verify >"$TMP_ROOT/linked-parent.stdout" 2>"$TMP_ROOT/linked-parent.stderr"
linked_parent_rc=$?
set -e
if ((linked_parent_rc == 0)); then
	fail "dispatch accepted a target with a symlinked parent"
fi
grep -Fq "parent path component must not be a symlink" "$TMP_ROOT/linked-parent.stderr" ||
	fail "dispatch symlinked target parent rejection had unexpected output: $(rtk cat "$TMP_ROOT/linked-parent.stderr")"
pass "dispatch rejects a symlinked target parent"

set +e
TASK=002 \
	TC_CAMPAIGN_ROOT="$campaign" \
	TC_TASK_WORKTREE="$candidate" \
	TC_TASK_RUN_DIR="$TMP_ROOT/positive-run" \
	TC_TASK_BASE="$candidate_head" \
	TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
	TC_CATALOG_ROOT="$catalog" \
	TC_PROOF_TARGET_DIR="$target" \
	TC_TASKFMT="$TASKFMT" \
	"$DISPATCH" verify >"$TMP_ROOT/positive.stdout" 2>"$TMP_ROOT/positive.stderr"
positive_rc=$?
set -e
if ((positive_rc != 0)); then
	fail "valid preflight authorization rejected (exit $positive_rc): stdout=$(rtk cat "$TMP_ROOT/positive.stdout") stderr=$(rtk cat "$TMP_ROOT/positive.stderr")"
fi
pass "valid current preflight authorization accepted"

taskfmt_final_alias="$TMP_ROOT/taskfmt-final-alias"
ln -s "$TASKFMT" "$taskfmt_final_alias"
set +e
TASK=002 \
	TC_CAMPAIGN_ROOT="$campaign" \
	TC_TASK_WORKTREE="$candidate" \
	TC_TASK_RUN_DIR="$TMP_ROOT/positive-run" \
	TC_TASK_BASE="$candidate_head" \
	TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
	TC_CATALOG_ROOT="$catalog" \
	TC_PROOF_TARGET_DIR="$target" \
	CARGO_TARGET_DIR="" \
	TC_TASKFMT="$taskfmt_final_alias" \
	"$DISPATCH" verify >"$TMP_ROOT/taskfmt-final.stdout" 2>"$TMP_ROOT/taskfmt-final.stderr"
taskfmt_final_rc=$?
set -e
if ((taskfmt_final_rc == 0)); then
	fail "dispatch accepted a taskfmt final symlink"
fi
grep -Fq "taskfmt final path must not be a symlink" "$TMP_ROOT/taskfmt-final.stderr" ||
	fail "dispatch taskfmt final symlink rejection had unexpected output: $(rtk cat "$TMP_ROOT/taskfmt-final.stderr")"
pass "dispatch rejects a taskfmt final symlink"

taskfmt_parent_alias="$TMP_ROOT/taskfmt-parent-alias"
ln -s "$(dirname "$TASKFMT")" "$taskfmt_parent_alias"
set +e
TASK=002 \
	TC_CAMPAIGN_ROOT="$campaign" \
	TC_TASK_WORKTREE="$candidate" \
	TC_TASK_RUN_DIR="$TMP_ROOT/positive-run" \
	TC_TASK_BASE="$candidate_head" \
	TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
	TC_CATALOG_ROOT="$catalog" \
	TC_PROOF_TARGET_DIR="$target" \
	CARGO_TARGET_DIR="" \
	TC_TASKFMT="$taskfmt_parent_alias/$(basename "$TASKFMT")" \
	"$DISPATCH" verify >"$TMP_ROOT/taskfmt-parent.stdout" 2>"$TMP_ROOT/taskfmt-parent.stderr"
taskfmt_parent_rc=$?
set -e
if ((taskfmt_parent_rc == 0)); then
	fail "dispatch accepted a taskfmt symlinked parent"
fi
grep -Fq "taskfmt parent path component must not be a symlink" "$TMP_ROOT/taskfmt-parent.stderr" ||
	fail "dispatch taskfmt symlinked parent rejection had unexpected output: $(rtk cat "$TMP_ROOT/taskfmt-parent.stderr")"
pass "dispatch rejects a taskfmt symlinked parent"

hardlink_sentinel="$TMP_ROOT/hardlink-tc-proof"
cp "$binary" "$hardlink_sentinel"
rm "$binary"
ln "$hardlink_sentinel" "$binary"
set +e
TASK=002 \
	TC_CAMPAIGN_ROOT="$campaign" \
	TC_TASK_WORKTREE="$candidate" \
	TC_TASK_RUN_DIR="$TMP_ROOT/positive-run" \
	TC_TASK_BASE="$candidate_head" \
	TC_TASK_PREFLIGHT_EVIDENCE="$TMP_ROOT/positive-run/authorization.json" \
	TC_CATALOG_ROOT="$catalog" \
	TC_PROOF_TARGET_DIR="$target" \
	CARGO_TARGET_DIR="" \
	TC_TASKFMT="$TASKFMT" \
	"$DISPATCH" verify >"$TMP_ROOT/hardlink.stdout" 2>"$TMP_ROOT/hardlink.stderr"
hardlink_rc=$?
set -e
if ((hardlink_rc == 0)); then
	fail "dispatch accepted a hardlinked proof binary"
fi
grep -Fq "native comparator is not a regular single-link file" "$TMP_ROOT/hardlink.stderr" ||
	fail "dispatch hardlink rejection had unexpected output: $(rtk cat "$TMP_ROOT/hardlink.stderr")"
pass "dispatch rejects a hardlinked proof binary"

echo "test_campaign_dispatch_authorization: all checks passed"
