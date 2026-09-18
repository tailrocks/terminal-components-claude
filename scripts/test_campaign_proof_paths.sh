#!/usr/bin/env bash
# Bounded regression checks for the host-local native proof target contract.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
BUILD="$SCRIPT_DIR/campaign-build-proof.sh"
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

grep -Fq "export TC_PROOF_TARGET_DIR=\"\$target_dir\" CARGO_TARGET_DIR=\"\$target_dir\"" "$DISPATCH" \
  || fail "dispatch does not export the resolved target root"
grep -Fq "require_native_proof \"\$worktree_root\" \"\$target_dir\" \"\$binary\"" "$DISPATCH" \
  || fail "dispatch does not validate the resolved binary/receipt"
if grep -Fq 'export TC_PROOF_NATIVE_CHILD=' "$DISPATCH"; then
  fail "dispatch exports dead TC_PROOF_NATIVE_CHILD state"
fi
pass "dispatch target binding and no dead native-launch export"

echo "test_campaign_proof_paths: all bounded checks passed"
