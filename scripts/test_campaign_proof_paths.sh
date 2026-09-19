#!/usr/bin/env bash
# Bounded regression checks for the host-local native proof target contract.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
BUILD="$SCRIPT_DIR/campaign-build-proof.sh"
PREFLIGHT="$SCRIPT_DIR/campaign-preflight.sh"
DISPATCH="$SCRIPT_DIR/campaign-dispatch.sh"
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
printf '%s\n' '[package]' 'name = "fixture"' 'version = "0.1.0"' > "$repo/Cargo.toml"
git -C "$repo" add Cargo.toml
git -C "$repo" commit -q -m fixture

ln -s "$cache" "$repo/target"
if output="$(cd "$repo" && env -u TC_PROOF_TARGET_DIR CARGO_TARGET_DIR= "$BUILD" 2>&1)"; then
  fail "default shared-cache symlink was accepted"
fi
grep -Fq "proof target directory must not be a symlink" <<<"$output" \
  || fail "default symlink rejection had unexpected output: $output"
pass "default symlink rejection"

rm "$repo/target"
missing_target="$TMP_ROOT/missing-target"
if output="$(
  cd "$repo"
  env -u CARGO_TARGET_DIR \
      TC_PROOF_TARGET_DIR="$missing_target" \
      "$BUILD" 2>&1
)"; then
  fail "missing external target was accepted"
fi
grep -Fq "proof target directory is missing" <<<"$output" \
  || fail "missing target rejection had unexpected output: $output"

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
grep -Fq "select ambiguous target roots" <<<"$output" \
  || fail "ambiguous target rejection had unexpected output: $output"
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
grep -Fq "stale proof receipt source binding" <<<"$output" \
  || fail "mismatched receipt had unexpected output: $output"
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

if output="$(
  env TC_CAMPAIGN_WORKTREE="$repo" \
      TC_PROOF_TARGET_DIR="$preflight_target" \
      CARGO_TARGET_DIR= \
      bash -c 'source "$1"; check_native_proof' _ "$preflight_helper" 2>&1
)"; then
  pass "preflight accepts a valid external target and current receipt"
else
  fail "preflight rejected valid external target: $output"
fi

preflight_symlink="$TMP_ROOT/preflight-symlink"
ln -s "$preflight_target" "$preflight_symlink"
if output="$(
  env TC_CAMPAIGN_WORKTREE="$repo" \
      TC_PROOF_TARGET_DIR="$preflight_symlink" \
      CARGO_TARGET_DIR= \
      bash -c 'source "$1"; check_native_proof' _ "$preflight_helper" 2>&1
)"; then
  fail "preflight accepted a symlink target"
fi
grep -Fq "TC_PROOF_TARGET_DIR must not be a symlink" <<<"$output" \
  || fail "preflight symlink rejection had unexpected output: $output"
pass "preflight rejects a symlink target"

preflight_inside="$repo/preflight-target-inside"
mkdir -p "$preflight_inside"
if output="$(
  env TC_CAMPAIGN_WORKTREE="$repo" \
      TC_PROOF_TARGET_DIR="$preflight_inside" \
      CARGO_TARGET_DIR= \
      bash -c 'source "$1"; check_native_proof' _ "$preflight_helper" 2>&1
)"; then
  fail "preflight accepted a target inside the worktree"
fi
grep -Fq "proof target directory must be external to the worktree" <<<"$output" \
  || fail "preflight inside-worktree rejection had unexpected output: $output"
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
  env TC_CAMPAIGN_WORKTREE="$repo" \
      TC_PROOF_TARGET_DIR="$preflight_target" \
      CARGO_TARGET_DIR= \
      bash -c 'source "$1"; check_native_proof' _ "$preflight_helper" 2>&1
)"; then
  fail "preflight accepted a stale build receipt"
fi
grep -Fq "wrong build tree" <<<"$output" \
  || fail "preflight stale receipt rejection had unexpected output: $output"
pass "preflight rejects a stale build receipt"

PYTHONPATH="$SCRIPT_DIR" python3 - <<'PY'
import copy
import tempfile
from datetime import timedelta
from pathlib import Path

from campaign_ledger import LedgerValidationError, validate_preparation_qualification
from test_campaign_ledger import TASKFMT, BRANCH, make_qualification


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
        "expected_taskfmt": TASKFMT,
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

grep -Fq "export TC_PROOF_TARGET_DIR=\"\$target_dir\" CARGO_TARGET_DIR=\"\$target_dir\"" "$DISPATCH" \
  || fail "dispatch does not export the resolved target root"
grep -Fq "require_native_proof \"\$worktree_root\" \"\$target_dir\" \"\$binary\"" "$DISPATCH" \
  || fail "dispatch does not validate the resolved binary/receipt"
if grep -Fq 'export TC_PROOF_NATIVE_CHILD=' "$DISPATCH"; then
  fail "dispatch exports dead TC_PROOF_NATIVE_CHILD state"
fi
pass "dispatch target binding and no dead native-launch export"

echo "test_campaign_proof_paths: all bounded checks passed"
