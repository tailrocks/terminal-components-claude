#!/usr/bin/env bash
# Bounded regression checks for the host-local native proof target contract.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
BUILD="$SCRIPT_DIR/campaign-build-proof.sh"
PREFLIGHT="$SCRIPT_DIR/campaign-preflight.sh"
DISPATCH="$SCRIPT_DIR/campaign-dispatch.sh"
INSTALLER="$SCRIPT_DIR/campaign-install-taskfmt.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/tc-proof-path-contract.XXXXXX")"

cleanup() {
	rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
	echo "test_campaign_proof_paths: FAIL: $*" >&2
	exit 1
}

pass() {
	echo "test_campaign_proof_paths: PASS: $*"
}

repo="$TMP_ROOT/repo"
cache="$TMP_ROOT/shared-cache"
fake_bin="$TMP_ROOT/fake-bin"
external_target="$TMP_ROOT/external-target"
mkdir -p "$repo" "$cache" "$fake_bin" "$external_target"
git init -q "$repo"
git -C "$repo" config user.email test@example.invalid
git -C "$repo" config user.name "Proof Path Test"
printf '%s\n' '[package]' 'name = "fixture"' 'version = "0.1.0"' >"$repo/Cargo.toml"
git -C "$repo" add Cargo.toml
git -C "$repo" commit -q -m fixture

ln -s "$cache" "$repo/target"
if output="$(cd "$repo" && env -u TC_PROOF_TARGET_DIR CARGO_TARGET_DIR= "$BUILD" 2>&1)"; then
	fail "default shared-cache symlink was accepted"
fi
grep -Fq "proof target directory must not be a symlink" <<<"$output" ||
	fail "default symlink rejection had unexpected output: $output"
pass "default symlink rejection"

rm "$repo/target"
linked_parent="$TMP_ROOT/linked-parent"
linked_target="$cache/linked-target"
mkdir -p "$linked_target"
ln -s "$cache" "$linked_parent"
if output="$(
	cd "$repo"
	env -u CARGO_TARGET_DIR \
		TC_PROOF_TARGET_DIR="$linked_parent/linked-target" \
		"$BUILD" 2>&1
)"; then
	fail "target with a symlinked parent was accepted"
fi
grep -Fq "parent path component must not be a symlink" <<<"$output" ||
	fail "symlinked target parent rejection had unexpected output: $output"
pass "symlinked target parent rejection"

missing_target="$TMP_ROOT/missing-target"
if output="$(
	cd "$repo"
	env -u CARGO_TARGET_DIR \
		TC_PROOF_TARGET_DIR="$missing_target" \
		"$BUILD" 2>&1
)"; then
	fail "missing external target was accepted"
fi
grep -Fq "proof target directory is missing" <<<"$output" ||
	fail "missing target rejection had unexpected output: $output"

ambiguous_target="$TMP_ROOT/ambiguous-target"
mkdir -p "$ambiguous_target"
if output="$(
	cd "$repo"
	env TC_PROOF_TARGET_DIR="$external_target" \
		CARGO_TARGET_DIR="$ambiguous_target" \
		"$BUILD" 2>&1
)"; then
	fail "ambiguous target roots were accepted"
fi
grep -Fq "select ambiguous target roots" <<<"$output" ||
	fail "ambiguous target rejection had unexpected output: $output"
pass "missing/ambiguous target rejection"

python3 - "$fake_bin/cargo" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
path.write_text(
    "#!/usr/bin/env bash\n"
    "set -euo pipefail\n"
    "mkdir -p \"$CARGO_TARGET_DIR/debug\"\n"
    "printf '%s\\n' '#!/usr/bin/env bash' 'exit 0' > \"$CARGO_TARGET_DIR/debug/tc-proof\"\n"
    "chmod +x \"$CARGO_TARGET_DIR/debug/tc-proof\"\n",
    encoding="utf-8",
)
path.chmod(0o755)
PY

if ! output="$(
	cd "$repo"
	env -u TC_PROOF_TARGET_DIR \
		CARGO_TARGET_DIR= \
		PATH="$fake_bin:$PATH" \
		TC_PROOF_TARGET_DIR="$external_target" \
		"$BUILD" 2>&1
)"; then
	fail "explicit external target was rejected: $output"
fi
binary="$external_target/debug/tc-proof"
receipt="$external_target/debug/tc-proof.build.json"
[[ -x "$binary" && -f "$receipt" ]] || fail "explicit target did not receive binary and receipt"
python3 - "$receipt" "$repo" "$external_target" "$binary" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt, repo, target, binary = sys.argv[1:]
value = json.loads(Path(receipt).read_text(encoding="utf-8"))
assert value["worktree"] == str(Path(repo).resolve())
assert value["target_dir"] == str(Path(target).resolve())
assert value["cargo_target_dir"] == str(Path(target).resolve())
assert value["binary"] == str(Path(binary).resolve())
assert value["commit"]
assert value["tree"]
assert value["binary_sha256"] == hashlib.sha256(Path(binary).read_bytes()).hexdigest()
PY
pass "explicit external target build and receipt binding"

hardlink_target="$TMP_ROOT/hardlink-target"
hardlink_sentinel="$TMP_ROOT/hardlink-sentinel"
mkdir -p "$hardlink_target/debug"
cp "$binary" "$hardlink_sentinel"
ln "$hardlink_sentinel" "$hardlink_target/debug/tc-proof"
cp "$receipt" "$hardlink_target/debug/tc-proof.build.json"
python3 - "$hardlink_target/debug/tc-proof.build.json" "$hardlink_target" \
	"$hardlink_target/debug/tc-proof" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt, target, binary = map(Path, sys.argv[1:])
value = json.loads(receipt.read_text(encoding="utf-8"))
value["target_dir"] = str(target.resolve())
value["cargo_target_dir"] = str(target.resolve())
value["binary"] = str(binary.resolve())
value["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
receipt.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
if output="$(
	cd "$repo"
	env -u TC_PROOF_TARGET_DIR \
		CARGO_TARGET_DIR= \
		PATH="$fake_bin:$PATH" \
		TC_PROOF_TARGET_DIR="$hardlink_target" \
		"$BUILD" 2>&1
)"; then
	fail "hardlinked proof binary was accepted"
fi
grep -Fq "proof binary is not a regular single-link file" <<<"$output" ||
	fail "hardlinked proof binary rejection had unexpected output: $output"
pass "hardlinked proof binary rejection"

installer_repo="$TMP_ROOT/installer-repo"
mkdir -p "$installer_repo/scripts"
cp "$SCRIPT_DIR/campaign-path-guards.sh" "$installer_repo/scripts/campaign-path-guards.sh"
python3 - "$INSTALLER" "$installer_repo/scripts/campaign-install-taskfmt-functions.sh" <<'PY'
import sys
from pathlib import Path

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
lines = source.read_text(encoding="utf-8").splitlines()
try:
    lines.remove('main "$@"')
except ValueError as error:
    raise SystemExit("campaign-install-taskfmt main entry point changed") from error
destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
installer_original="$TMP_ROOT/installed-taskfmt"
installer_hardlink="$TMP_ROOT/taskfmt-hardlink"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' >"$installer_original"
chmod 755 "$installer_original"
ln "$installer_original" "$installer_hardlink"
installer_before_sha="$(shasum -a 256 "$installer_hardlink" | awk '{print $1}')"
if ! output="$(bash -c 'source "$1"; materialize_single_link "$2"' _ \
	"$installer_repo/scripts/campaign-install-taskfmt-functions.sh" \
	"$installer_hardlink" 2>&1)"; then
	fail "single-link taskfmt materialization rejected a valid hardlink source: $output"
fi
python3 - "$installer_hardlink" "$installer_before_sha" <<'PY'
import hashlib
import stat
import sys
from pathlib import Path

path = Path(sys.argv[1])
expected = sys.argv[2]
metadata = path.stat()
if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
    raise SystemExit(f"materialized taskfmt is not regular single-link: {path}")
if not (metadata.st_mode & 0o111):
    raise SystemExit(f"materialized taskfmt is not executable: {path}")
actual = hashlib.sha256(path.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit(f"materialized taskfmt hash changed: {actual} != {expected}")
PY
pass "installer materializes exact taskfmt bytes as one link"

taskfmt_guard_source="$TMP_ROOT/taskfmt-guard-source"
taskfmt_guard_hardlink="$TMP_ROOT/taskfmt-guard-hardlink"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' >"$taskfmt_guard_source"
chmod 755 "$taskfmt_guard_source"
ln "$taskfmt_guard_source" "$taskfmt_guard_hardlink"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/campaign-path-guards.sh"
if output="$(campaign_require_regular_file "$taskfmt_guard_hardlink" "taskfmt" 1 1 2>&1)"; then
	fail "taskfmt hardlink passed the single-link guard"
fi
grep -Fq "must be a regular single-link file" <<<"$output" ||
	fail "taskfmt hardlink guard had unexpected output: $output"
pass "taskfmt hardlink is rejected at the trust guard"

python3 - "$receipt" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
value["tree"] = "0" * 40
path.write_text(json.dumps(value), encoding="utf-8")
PY
if output="$(
	cd "$repo"
	env -u TC_PROOF_TARGET_DIR \
		CARGO_TARGET_DIR= \
		PATH="$fake_bin:$PATH" \
		TC_PROOF_TARGET_DIR="$external_target" \
		"$BUILD" 2>&1
)"; then
	fail "mismatched receipt was accepted"
fi
grep -Fq "stale proof receipt source binding" <<<"$output" ||
	fail "mismatched receipt had unexpected output: $output"
pass "stale/mismatched receipt rejection"

mkdir -p "$repo/scripts"
preflight_helper="$repo/scripts/campaign-preflight-functions.sh"
python3 - "$PREFLIGHT" "$preflight_helper" <<'PY'
import sys
from pathlib import Path

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
lines = source.read_text(encoding="utf-8").splitlines()
try:
    lines.remove('main "$@"')
except ValueError as error:
    raise SystemExit("campaign-preflight main entry point changed") from error
destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
cp "$SCRIPT_DIR/campaign-path-guards.sh" "$repo/scripts/campaign-path-guards.sh"

preflight_target="$TMP_ROOT/preflight-target"
mkdir -p "$preflight_target/debug"
cp "$binary" "$preflight_target/debug/tc-proof"
python3 - "$repo" "$preflight_target" "$preflight_target/debug/tc-proof" \
	"$preflight_target/debug/tc-proof.build.json" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path

repo, target, binary, receipt = map(Path, sys.argv[1:])
commit = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD"], text=True
).strip()
tree = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"], text=True
).strip()
resolved_repo = str(repo.resolve())
resolved_target = str(target.resolve())
resolved_binary = str(binary.resolve())
value = {
    "schema": "tc-proof-native-build/v1",
    "worktree": resolved_repo,
    "target_dir": resolved_target,
    "cargo_target_dir": resolved_target,
    "commit": commit,
    "tree": tree,
    "binary": resolved_binary,
    "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
}
receipt.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")
PY

run_preflight_check() {
	local target="$1"
	env TC_CAMPAIGN_WORKTREE="$repo" TC_PROOF_TARGET_DIR="$target" CARGO_TARGET_DIR= \
		bash -c "source \"\$1\"; check_native_proof" _ "$preflight_helper" 2>&1
}

if output="$(
	run_preflight_check "$preflight_target"
)"; then
	pass "preflight accepts a valid external target and current receipt"
else
	fail "preflight rejected valid external target: $output"
fi

preflight_symlink="$TMP_ROOT/preflight-symlink"
ln -s "$preflight_target" "$preflight_symlink"
if output="$(
	run_preflight_check "$preflight_symlink"
)"; then
	fail "preflight accepted a symlink target"
fi
grep -Fq "TC_PROOF_TARGET_DIR must not be a symlink" <<<"$output" ||
	fail "preflight symlink rejection had unexpected output: $output"
pass "preflight rejects a symlink target"

preflight_linked_parent="$TMP_ROOT/preflight-linked-parent"
ln -s "$TMP_ROOT" "$preflight_linked_parent"
if output="$(
	run_preflight_check "$preflight_linked_parent/preflight-target"
)"; then
	fail "preflight accepted a symlinked target parent"
fi
grep -Fq "parent path component must not be a symlink" <<<"$output" ||
	fail "preflight symlinked target parent rejection had unexpected output: $output"
pass "preflight rejects a symlinked target parent"

preflight_inside="$repo/preflight-target-inside"
mkdir -p "$preflight_inside"
if output="$(
	run_preflight_check "$preflight_inside"
)"; then
	fail "preflight accepted a target inside the worktree"
fi
grep -Fq "proof target directory must be external to the worktree" <<<"$output" ||
	fail "preflight inside-worktree rejection had unexpected output: $output"
pass "preflight rejects an inside-worktree target"

python3 - "$preflight_target/debug/tc-proof.build.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
value["tree"] = "0" * 40
path.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")
PY
if output="$(
	run_preflight_check "$preflight_target"
)"; then
	fail "preflight accepted a stale build receipt"
fi
grep -Fq "wrong build tree" <<<"$output" ||
	fail "preflight stale receipt rejection had unexpected output: $output"
pass "preflight rejects a stale build receipt"

python3 - "$preflight_target/debug/tc-proof.build.json" "$repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

receipt, repo = map(Path, sys.argv[1:])
value = json.loads(receipt.read_text(encoding="utf-8"))
value["commit"] = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD"], text=True
).strip()
value["tree"] = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"], text=True
).strip()
receipt.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")
PY
rm "$preflight_target/debug/tc-proof"
ln "$binary" "$preflight_target/debug/tc-proof"
if output="$(
	run_preflight_check "$preflight_target"
)"; then
	fail "preflight accepted a hardlinked proof binary"
fi
grep -Fq "native comparator is not a regular single-link file" <<<"$output" ||
	fail "preflight hardlinked proof binary rejection had unexpected output: $output"
pass "preflight rejects a hardlinked proof binary"

PYTHONPATH="$SCRIPT_DIR" python3 - <<'PY'
import copy
import tempfile
from datetime import timedelta
from pathlib import Path

from campaign_ledger import LedgerValidationError, validate_preparation_qualification
from test_campaign_ledger import BRANCH, make_qualification


def expect_reject(label, operation, expected):
    try:
        operation()
    except LedgerValidationError as error:
        if expected not in str(error):
            raise SystemExit(f"{label} rejected for the wrong reason: {error}")
        print(f"test_campaign_proof_paths: PASS: {label}")
        return
    raise SystemExit(f"{label} was accepted")


with tempfile.TemporaryDirectory(prefix="tc-proof-qualification-") as directory:
    qualification, paths, oracle, qualified = make_qualification(Path(directory))
    common = {
        "worktree": paths["candidate"],
        "current_head": "a" * 40,
        "current_tree": "b" * 40,
        "integration_branch": BRANCH,
        "expected_oracle": oracle,
        "expected_taskfmt": paths["taskfmt"],
        "repository_root": paths["candidate"],
        "now": qualified + timedelta(seconds=1),
    }
    common["catalog_identity"] = qualification["catalog"]
    common["task_graph_identity"] = qualification["task_graph"]
    validate_preparation_qualification(qualification, **common)
    print("test_campaign_proof_paths: PASS: valid external preparation qualification")

    stale_tree = copy.deepcopy(qualification)
    stale_tree["candidate_tree"] = "c" * 40
    expect_reject(
        "stale preparation candidate tree",
        lambda: validate_preparation_qualification(stale_tree, **common),
        "candidate tree",
    )

    wrong_oracle = dict(oracle)
    wrong_oracle["tree"] = "1" * 40
    expect_reject(
        "wrong protected oracle identity",
        lambda: validate_preparation_qualification(
            qualification, **{**common, "expected_oracle": wrong_oracle}
        ),
        "protected baseline tree",
    )

    wrong_graph = copy.deepcopy(qualification["task_graph"])
    wrong_graph["tree"] = "c" * 40
    expect_reject(
        "wrong task-graph identity",
        lambda: validate_preparation_qualification(
            qualification, **{**common, "task_graph_identity": wrong_graph}
        ),
        "task_graph.tree",
    )
PY

grep -Fq 'campaign_require_regular_file "$TASKFMT_BIN" "taskfmt" 1 1' "$PREFLIGHT" ||
	fail "preflight does not require a single-link taskfmt"
grep -Fq 'campaign_require_regular_file "$TASKFMT" "taskfmt" 1 1' "$DISPATCH" ||
	fail "dispatch does not require a single-link taskfmt"
pass "preflight and dispatch require a single-link taskfmt"

grep -Fq "export TC_PROOF_TARGET_DIR=\"\$target_dir\" CARGO_TARGET_DIR=\"\$target_dir\"" "$DISPATCH" ||
	fail "dispatch does not export the resolved target root"
grep -Fq "require_native_proof \"\$worktree_root\" \"\$target_dir\" \"\$binary\"" "$DISPATCH" ||
	fail "dispatch does not validate the resolved binary/receipt"
if grep -Fq 'export TC_PROOF_NATIVE_CHILD=' "$DISPATCH"; then
	fail "dispatch exports dead TC_PROOF_NATIVE_CHILD state"
fi
pass "dispatch target binding and no dead native-launch export"

echo "test_campaign_proof_paths: all bounded checks passed"
