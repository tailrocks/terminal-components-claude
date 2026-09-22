#!/usr/bin/env bash
# Focused regression checks for taskfmt verify-log capture sealing.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
DISPATCH="$SCRIPT_DIR/campaign-dispatch.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/tc-dispatch-capture.XXXXXX")"

cleanup() {
	rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
	echo "test_campaign_dispatch_capture: FAIL: $*" >&2
	exit 1
}

pass() {
	echo "test_campaign_dispatch_capture: PASS: $*"
}

cd "$SCRIPT_DIR/.."

source_output="$(source "$DISPATCH")"
[[ -z "$source_output" ]] || fail "sourcing dispatch executed main: $source_output"
# shellcheck disable=SC1091
source "$DISPATCH"

log_dir="$TMP_ROOT/logs"
mkdir "$log_dir"
printf '%s\n' 'FORGED LOG CONTENT' >"$log_dir/verify.log"
capture="$TMP_ROOT/capture.log"
expected="$TMP_ROOT/expected.log"
printf '%s\n%s\n' 'captured taskfmt output' 'DONE' >"$capture"
cp "$capture" "$expected"

replace_taskfmt_verify_log "$capture" "$log_dir"
[[ ! -e "$capture" ]] || fail "capture file remained after replacement"
cmp -s "$expected" "$log_dir/verify.log" ||
	fail "regular verify.log was not replaced with exact capture bytes"
[[ "$(tail -n 1 "$log_dir/verify.log")" == DONE ]] ||
	fail "replaced verify.log does not end with DONE"
pass "regular verify.log is atomically replaced with exact DONE capture"

sentinel="$TMP_ROOT/outside-sentinel"
sentinel_expected="$TMP_ROOT/outside-sentinel.expected"
printf '%s\n' 'SENTINEL BYTES' >"$sentinel"
cp "$sentinel" "$sentinel_expected"
rm "$log_dir/verify.log"
ln -s "$sentinel" "$log_dir/verify.log"
symlink_capture="$TMP_ROOT/symlink-capture.log"
printf '%s\n%s\n' 'rejected capture' 'DONE' >"$symlink_capture"
if replace_taskfmt_verify_log "$symlink_capture" "$log_dir" \
	2>"$TMP_ROOT/symlink-rejection.log"; then
	fail "symlinked verify.log was accepted"
fi
cmp -s "$sentinel_expected" "$sentinel" ||
	fail "outside sentinel bytes changed through verify.log symlink"
[[ -L "$log_dir/verify.log" ]] || fail "verify.log symlink was replaced"
[[ -f "$symlink_capture" ]] || fail "rejected symlink capture was consumed"
pass "verify.log symlink is rejected and outside sentinel stays unchanged"

unsafe_target="$TMP_ROOT/unsafe-target"
unsafe_log_dir="$TMP_ROOT/unsafe-log-dir"
mkdir "$unsafe_target"
ln -s "$unsafe_target" "$unsafe_log_dir"
unsafe_capture="$TMP_ROOT/unsafe-capture.log"
printf '%s\n%s\n' 'rejected directory capture' 'DONE' >"$unsafe_capture"
if replace_taskfmt_verify_log "$unsafe_capture" "$unsafe_log_dir" \
	2>"$TMP_ROOT/directory-rejection.log"; then
	fail "symlinked log directory was accepted"
fi
[[ -L "$unsafe_log_dir" ]] || fail "symlinked log directory changed"
[[ -f "$unsafe_capture" ]] || fail "rejected directory capture was consumed"
pass "symlinked log directory is rejected"

echo "test_campaign_dispatch_capture: all checks passed"
