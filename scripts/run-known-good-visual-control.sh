#!/usr/bin/env bash
# Known-good visual calibration: peeled visual-baseline tag vs frozen snapshots.
# Scratch-only diagnostic. Never edits the repository, snapshots, or oracle.
set -euo pipefail

ORACLE_TAG_OBJECT="1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
ORACLE_COMMIT="4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
ORACLE_TREE="0b1f13431fdfd6060cf9f45a114afa5a99cc6c26"
ORACLE_SNAPSHOTS_TREE="3f0261c32849e26feda24d87697de4a7ce6b8375"
DEFAULT_FILTER="binary(visual_baseline)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAMPAIGN_ROOT="$(git -C "$SCRIPT_DIR/.." rev-parse --show-toplevel)"

# Every operator-controlled path is explicit. RUN_DIR is only a convenience
# root; its children are still separate, fresh locations.
RUN_DIR="${RUN_DIR:-${CONTROL_RUN_DIR:-}}"
KNOWN_GOOD_SOURCE="${KNOWN_GOOD_SOURCE:-}"
OUT="${OUT:-}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-}"
RUSTUP_CARGO_BIN="${RUSTUP_CARGO_BIN:-}"
NEXTEST_BIN_DIR="${NEXTEST_BIN_DIR:-}"
PROTECTED_BASELINE_WORKTREE="${PROTECTED_BASELINE_WORKTREE:-}"

usage() {
	cat <<'EOF'
Usage: scripts/run-known-good-visual-control.sh [nextest -E filter | nextest args...]

Diagnostic only. It executes the detached visual-baseline tag and compares
fresh actual HTML bytes with the frozen snapshots. It exits nonzero on any
test failure, missing/unexpected output, or exact HTML/provenance mismatch.

Required paths, either directly or through RUN_DIR/CONTROL_RUN_DIR:
  KNOWN_GOOD_SOURCE    detached checkout at the peeled visual-baseline commit
  OUT                  fresh external directory for logs and identity
  CARGO_TARGET_DIR     fresh external Cargo target directory

RUN_DIR/CONTROL_RUN_DIR derives the missing values as:
  $RUN_DIR/source, $RUN_DIR/output, $RUN_DIR/cargo-target

Optional paths:
  RUSTUP_CARGO_BIN     native cargo toolchain bin directory to prepend to PATH
  NEXTEST_BIN_DIR      directory containing cargo-nextest to prepend to PATH
  PROTECTED_BASELINE_WORKTREE
                       registered visual-baseline worktree; auto-discovered

The source checkout's own target/tuisnap scratch is created only when absent.
It is never deleted or reused. No protected baseline worktree, symlinked path,
or hard-linked trust file is accepted. No cargo test command is accepted.
EOF
}

die() {
	echo "known-good-visual-control: FAIL: $*" >&2
	exit 1
}

have_rtk() {
	command -v rtk >/dev/null 2>&1
}

run_cmd() {
	if have_rtk; then
		rtk proxy "$@"
	else
		"$@"
	fi
}

git_in() {
	local dir="$1"
	shift
	# Git output feeds identity/path parsers. Do not pass it through rtk,
	# whose presentation layer may abbreviate absolute paths.
	git -C "$dir" "$@"
}

validate_path() {
	local path="$1" field="$2" kind="$3" allow_missing="$4"
	python3 - "$path" "$field" "$kind" "$allow_missing" <<'PY'
import os
import stat
import sys
from pathlib import Path

path_text, field, kind, allow_missing = sys.argv[1:]
path = Path(path_text)
allow_missing = allow_missing == "1"
allowed_system_symlinks = {
    Path("/etc"): Path("/private/etc"),
    Path("/home"): Path("/System/Volumes/Data/home"),
    Path("/tmp"): Path("/private/tmp"),
    Path("/var"): Path("/private/var"),
}


def fail(message: str) -> None:
    raise SystemExit(f"{field} {message}")


if not path.is_absolute():
    fail(f"must be an absolute path: {path_text or '<empty>'}")

current = Path(path.anchor)
for component in path.parts[1:]:
    current /= component
    try:
        metadata = os.lstat(current)
    except FileNotFoundError:
        break
    except OSError as error:
        fail(f"path component is unreadable: {current}: {error}")
    if stat.S_ISLNK(metadata.st_mode):
        if current == path:
            fail(f"must not be a symlink: {path}")
        if Path(os.path.realpath(current)) != allowed_system_symlinks.get(current):
            fail(f"path component must not be a symlink: {current}")
        continue
    if current != path and not stat.S_ISDIR(metadata.st_mode):
        fail(f"path component is not a directory: {current}")

try:
    metadata = os.lstat(path)
except FileNotFoundError:
    if allow_missing:
        raise SystemExit(0)
    fail(f"is missing: {path}")
except OSError as error:
    fail(f"is unreadable: {path}: {error}")

if stat.S_ISLNK(metadata.st_mode):
    fail(f"must not be a symlink: {path}")
if kind == "dir":
    if not stat.S_ISDIR(metadata.st_mode):
        fail(f"is not a directory: {path}")
elif kind == "file":
    if not stat.S_ISREG(metadata.st_mode):
        fail(f"is not a regular file: {path}")
    if metadata.st_nlink != 1:
        fail(f"must be a single-link regular file: {path} (links={metadata.st_nlink})")
else:
    fail(f"internal error: unsupported path kind {kind}")
PY
}

validate_tree() {
	local root="$1" field="$2"
	python3 - "$root" "$field" <<'PY'
import os
import stat
import sys
from pathlib import Path

root, field = Path(sys.argv[1]), sys.argv[2]


def fail(message: str) -> None:
    raise SystemExit(f"{field} {message}")


try:
    root_metadata = os.lstat(root)
except OSError as error:
    fail(f"is unreadable: {root}: {error}")
if stat.S_ISLNK(root_metadata.st_mode) or not stat.S_ISDIR(root_metadata.st_mode):
    fail(f"must be a real directory: {root}")

for path in (root, *root.rglob("*")):
    try:
        metadata = os.lstat(path)
    except OSError as error:
        fail(f"contains an unreadable path {path}: {error}")
    if stat.S_ISLNK(metadata.st_mode):
        fail(f"contains a symlink: {path}")
    if stat.S_ISDIR(metadata.st_mode):
        continue
    if not stat.S_ISREG(metadata.st_mode):
        fail(f"contains a non-regular file: {path}")
    if metadata.st_nlink != 1:
        fail(f"contains a hard-linked file: {path} (links={metadata.st_nlink})")
PY
}

reject_overlap() {
	local left="$1" right="$2" label="$3"
	python3 - "$left" "$right" "$label" <<'PY'
import os
import sys
from pathlib import Path

left, right, label = (Path(os.path.realpath(value)) for value in sys.argv[1:])
if left == right or left in right.parents or right in left.parents:
    raise SystemExit(f"{label} paths overlap: {left} and {right}")
PY
}

require_same_path() {
	local left="$1" right="$2" label="$3"
	python3 - "$left" "$right" "$label" <<'PY'
import os
import sys
from pathlib import Path

left, right, label = (Path(os.path.realpath(value)) for value in sys.argv[1:])
if left != right:
    raise SystemExit(f"{label} paths differ: {left} and {right}")
PY
}

reject_inside() {
	local candidate="$1" protected="$2" label="$3"
	python3 - "$candidate" "$protected" "$label" <<'PY'
import os
import sys
from pathlib import Path

candidate, protected, label = (Path(os.path.realpath(value)) for value in sys.argv[1:])
if candidate == protected or protected in candidate.parents:
    raise SystemExit(f"{label} must be outside protected path: {candidate} / {protected}")
PY
}

prepare_fresh_dir() {
	local path="$1" field="$2"
	validate_path "$path" "$field" dir 1
	if [[ ! -e "$path" ]]; then
		mkdir -p "$path"
	fi
	validate_path "$path" "$field" dir 0
	if [[ -n "$(find "$path" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
		die "$field must be a fresh empty directory: $path"
	fi
}

discover_protected_baseline() {
	local listing
	listing="$(git_in "$CAMPAIGN_ROOT" worktree list --porcelain)"
	python3 -c '
import sys

current = None
for line in sys.stdin:
    if line.startswith("worktree "):
        current = line.removeprefix("worktree ").rstrip("\n")
    elif line.rstrip("\n") == "branch refs/heads/visual-baseline" and current:
        print(current)
        break
' <<<"$listing"
}

resolve_paths() {
	if [[ -n "$RUN_DIR" ]]; then
		validate_path "$RUN_DIR" RUN_DIR dir 1
		if [[ ! -e "$RUN_DIR" ]]; then
			mkdir -p "$RUN_DIR"
		fi
		validate_path "$RUN_DIR" RUN_DIR dir 0
		KNOWN_GOOD_SOURCE="${KNOWN_GOOD_SOURCE:-$RUN_DIR/source}"
		OUT="${OUT:-$RUN_DIR/output}"
		CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$RUN_DIR/cargo-target}"
	fi

	[[ -n "$KNOWN_GOOD_SOURCE" ]] || die "set KNOWN_GOOD_SOURCE or RUN_DIR"
	[[ -n "$OUT" ]] || die "set OUT or RUN_DIR"
	[[ -n "$CARGO_TARGET_DIR" ]] || die "set CARGO_TARGET_DIR or RUN_DIR"
	validate_path "$KNOWN_GOOD_SOURCE" KNOWN_GOOD_SOURCE dir 0
	validate_path "$OUT" OUT dir 1
	validate_path "$CARGO_TARGET_DIR" CARGO_TARGET_DIR dir 1
	if [[ -n "$RUSTUP_CARGO_BIN" ]]; then
		validate_path "$RUSTUP_CARGO_BIN" RUSTUP_CARGO_BIN dir 0
	fi
	if [[ -n "$NEXTEST_BIN_DIR" ]]; then
		validate_path "$NEXTEST_BIN_DIR" NEXTEST_BIN_DIR dir 0
	fi
}

refuse_forbidden_args() {
	local i arg next prev
	for ((i = 1; i <= $#; i++)); do
		arg="${!i}"
		case "$arg" in
		accept | --accept | tuisnap | BLESS | BLESS=* | UPDATE_BASELINE | UPDATE_BASELINE=* | --bless)
			die "refusing oracle mutation arg: $arg"
			;;
		--target-dir | --target-dir=* | --manifest-path | --manifest-path=* | --config | --config=*)
			die "refusing path/config override arg: $arg"
			;;
		esac
		if [[ "$arg" == *tuisnap* && "$arg" == *accept* ]]; then
			die "refusing oracle mutation arg: $arg"
		fi
		if [[ "$arg" == "test" && $i -gt 1 ]]; then
			prev=$((i - 1))
			if [[ "${!prev}" == "cargo" ]]; then
				die "never cargo test; use cargo nextest"
			fi
		fi
		if [[ "$arg" == "cargo" && $i -lt $# ]]; then
			next=$((i + 1))
			if [[ "${!next}" == "test" ]]; then
				die "never cargo test; use cargo nextest"
			fi
		fi
	done
}

refuse_forbidden_env() {
	local name
	for name in BLESS UPDATE_BASELINE TUISNAP_ACCEPT TUISNAP_BLESS; do
		if [[ -n "${!name:-}" ]]; then
			die "refusing $name=${!name}; unset bless/accept env"
		fi
	done
}

pin_tool_paths() {
	if [[ -n "$RUSTUP_CARGO_BIN" ]]; then
		export PATH="$RUSTUP_CARGO_BIN:$PATH"
	fi
	if [[ -n "$NEXTEST_BIN_DIR" ]]; then
		export PATH="$NEXTEST_BIN_DIR:$PATH"
	fi
}

pin_cargo() {
	local cargo_bin nextest_bin
	cargo_bin="$(command -v cargo || true)"
	[[ -n "$cargo_bin" ]] || die "cargo is not on PATH"
	validate_path "$cargo_bin" cargo file 0
	[[ -x "$cargo_bin" ]] || die "cargo is not executable: $cargo_bin"
	nextest_bin="$(command -v cargo-nextest || true)"
	[[ -n "$nextest_bin" ]] || die "cargo-nextest is not on PATH"
	validate_path "$nextest_bin" cargo-nextest file 0
	[[ -x "$nextest_bin" ]] || die "cargo-nextest is not executable: $nextest_bin"
}

expect_eq() {
	local name="$1" actual="$2" expected="$3"
	if [[ "$actual" != "$expected" ]]; then
		die "oracle hash mismatch: ${name} actual=${actual} expected=${expected}"
	fi
}

locate_source() {
	local candidate="$KNOWN_GOOD_SOURCE" head branch
	validate_path "$candidate" KNOWN_GOOD_SOURCE dir 0
	[[ -d "$candidate/.git" || -f "$candidate/.git" ]] ||
		die "not a git checkout: $candidate"
	head="$(git_in "$candidate" rev-parse HEAD)"
	[[ "$head" == "$ORACLE_COMMIT" ]] ||
		die "source $candidate HEAD=$head (expected detached $ORACLE_COMMIT)"
	branch="$(git_in "$candidate" symbolic-ref --quiet --short HEAD || true)"
	[[ -z "$branch" ]] || die "known-good source must be detached, not on branch $branch"
	printf '%s\n' "$candidate"
}

verify_oracle_hashes() {
	local source="$1"
	local tag_object peeled tree snapshots_tree head head_tree head_snapshots
	tag_object="$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tag}')"
	peeled="$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{commit}')"
	tree="$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tree}')"
	snapshots_tree="$(git_in "$source" rev-parse 'refs/tags/visual-baseline:snapshots')"
	head="$(git_in "$source" rev-parse HEAD)"
	head_tree="$(git_in "$source" rev-parse 'HEAD^{tree}')"
	head_snapshots="$(git_in "$source" rev-parse 'HEAD:snapshots')"
	expect_eq tag_object "$tag_object" "$ORACLE_TAG_OBJECT"
	expect_eq peeled "$peeled" "$ORACLE_COMMIT"
	expect_eq tag_tree "$tree" "$ORACLE_TREE"
	expect_eq snapshots_tree "$snapshots_tree" "$ORACLE_SNAPSHOTS_TREE"
	expect_eq HEAD "$head" "$ORACLE_COMMIT"
	expect_eq HEAD_tree "$head_tree" "$ORACLE_TREE"
	expect_eq HEAD_snapshots "$head_snapshots" "$ORACLE_SNAPSHOTS_TREE"
	[[ -z "$(git_in "$source" status --porcelain --untracked-files=all)" ]] ||
		die "tag source working tree is dirty: $source"
	validate_tree "$source/snapshots" frozen snapshots
	validate_tree "$source/tests/visual_baseline" frozen visual suite
	validate_path "$source/.config/nextest.toml" "frozen nextest config" file 0
	echo "known-good-visual-control: oracle inputs verified; calibration pending"
}

write_identity() {
	local out="$1" source="$2"
	cat >"$out/identity.txt" <<EOF
role=known-good-visual-control
source=$source
source_scratch=$source/target/tuisnap
head=$(git_in "$source" rev-parse HEAD)
tree=$(git_in "$source" rev-parse 'HEAD^{tree}')
snapshots_tree=$(git_in "$source" rev-parse 'HEAD:snapshots')
tag_object=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tag}')
tag_peeled=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{commit}')
tag_tree=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tree}')
campaign_root=$CAMPAIGN_ROOT
protected_baseline_worktree=$PROTECTED_BASELINE_WORKTREE
cargo_bin=$(command -v cargo)
cargo_nextest_bin=$(command -v cargo-nextest)
cargo_target_dir=$CARGO_TARGET_DIR
out=$out
nextest_test_threads=${NEXTEST_TEST_THREADS:-}
nextest_args=${NEXTEST_ARGS[*]}
html_exact_required=true
started_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
rtk=$(command -v rtk 2>/dev/null || echo none)
EOF
}

build_nextest_args() {
	NEXTEST_ARGS=(run --locked --run-ignored only)
	if [[ $# -eq 0 ]]; then
		NEXTEST_ARGS+=(-E "$DEFAULT_FILTER")
		return
	fi
	if [[ "$1" == "-E" || "$1" == "--filter" ]]; then
		NEXTEST_ARGS+=("$@")
		return
	fi
	if [[ "$1" == -* ]]; then
		NEXTEST_ARGS+=("$@")
		local arg
		for arg in "$@"; do
			if [[ "$arg" == "-E" || "$arg" == "--filter" ]]; then
				return
			fi
		done
		NEXTEST_ARGS+=(-E "$DEFAULT_FILTER")
		return
	fi
	NEXTEST_ARGS+=(-E "$1")
	shift
	if [[ $# -gt 0 ]]; then
		NEXTEST_ARGS+=("$@")
	fi
}

prepare_source_scratch() {
	local source="$1"
	local target="$source/target" scratch="$source/target/tuisnap"
	if [[ -e "$target" || -L "$target" ]]; then
		die "known-good source already has target/; use a fresh detached checkout: $target"
	fi
	mkdir -p "$scratch/actual" "$scratch/diff"
	validate_path "$target" source-target dir 0
	validate_tree "$scratch" source-tuisnap-scratch
}

protected_inputs_mutated() {
	local source="$1"
	if [[ -n "$(git_in "$source" diff -- snapshots tests/visual_baseline .config/nextest.toml)" ]]; then
		return 0
	fi
	if [[ -n "$(git_in "$source" diff --cached -- snapshots tests/visual_baseline .config/nextest.toml)" ]]; then
		return 0
	fi
	if [[ -n "$(git_in "$source" status --porcelain --untracked-files=all)" ]]; then
		return 0
	fi
	return 1
}

verify_html_exactness() {
	local source="$1"
	python3 - "$source/snapshots" "$source/target/tuisnap/actual" <<'PY'
import hashlib
import json
import re
import stat
import sys
from pathlib import Path

expected_root, actual_root = map(Path, sys.argv[1:])


def fail(message: str) -> None:
    print(f"HTML_EXACT=false {message}")
    raise SystemExit(1)


def files(root: Path) -> dict[str, Path]:
    if root.is_symlink() or not root.is_dir():
        fail(f"root is not a real directory: {root}")
    found = {}
    for path in root.rglob("*"):
        metadata = path.lstat()
        if stat.S_ISLNK(metadata.st_mode):
            fail(f"trust tree contains symlink: {path}")
        if stat.S_ISDIR(metadata.st_mode):
            continue
        if not stat.S_ISREG(metadata.st_mode):
            fail(f"trust tree contains non-regular file: {path}")
        if metadata.st_nlink != 1:
            fail(f"trust tree contains hard-linked file: {path}")
        if path.suffix == ".html":
            found[str(path.relative_to(root))] = path
    return found


def provenance_argv0(path: Path) -> str | None:
    text = path.read_text(encoding="utf-8")
    match = re.search(r'"argv":\[("(?:\\.|[^"\\])*")', text)
    if match is None:
        return None
    try:
        return str(json.loads(f"[{match.group(1)}]")[0])
    except (ValueError, TypeError, IndexError) as error:
        fail(f"invalid provenance.argv in {path}: {error}")


expected = files(expected_root)
actual = files(actual_root)
missing = sorted(set(expected) - set(actual))
unexpected = sorted(set(actual) - set(expected))
if missing:
    fail(f"missing HTML artifacts={len(missing)} first={missing[0]}")
if unexpected:
    fail(f"unexpected HTML artifacts={len(unexpected)} first={unexpected[0]}")

for relative in sorted(expected):
    expected_path = expected[relative]
    actual_path = actual[relative]
    expected_bytes = expected_path.read_bytes()
    actual_bytes = actual_path.read_bytes()
    if expected_bytes != actual_bytes:
        expected_argv0 = provenance_argv0(expected_path)
        actual_argv0 = provenance_argv0(actual_path)
        if expected_argv0 != actual_argv0:
            fail(
                "frozen provenance argv[0] mismatch "
                f"path={relative} expected={expected_argv0!r} actual={actual_argv0!r}"
            )
        fail(
            "byte mismatch "
            f"path={relative} expected_sha256={hashlib.sha256(expected_bytes).hexdigest()} "
            f"actual_sha256={hashlib.sha256(actual_bytes).hexdigest()}"
        )

print(f"HTML_EXACT=true artifacts={len(expected)}")
PY
}

build_status_report() {
	local out="$1" status="$2" html_status="$3"
	{
		echo "finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
		echo "nextest_exit=$status"
		echo "html_exact_exit=$html_status"
		if ((status == 0 && html_status == 0)); then
			echo "calibration=PASS"
		else
			echo "calibration=FAIL"
		fi
	} >>"$out/identity.txt"
}

main() {
	if [[ $# -eq 1 && ("$1" == "-h" || "$1" == "--help") ]]; then
		usage
		exit 0
	fi
	refuse_forbidden_env
	refuse_forbidden_args "$@"
	resolve_paths

	local discovered_protected
	discovered_protected="$(discover_protected_baseline || true)"
	if [[ -z "$PROTECTED_BASELINE_WORKTREE" ]]; then
		PROTECTED_BASELINE_WORKTREE="$discovered_protected"
	fi
	[[ -n "$PROTECTED_BASELINE_WORKTREE" ]] ||
		die "cannot identify protected visual-baseline worktree"
	validate_path "$PROTECTED_BASELINE_WORKTREE" PROTECTED_BASELINE_WORKTREE dir 0
	if [[ -n "$discovered_protected" ]]; then
		require_same_path "$PROTECTED_BASELINE_WORKTREE" "$discovered_protected" protected-baseline-identity
	fi
	if [[ -n "$RUN_DIR" ]]; then
		reject_inside "$RUN_DIR" "$CAMPAIGN_ROOT" RUN_DIR
		reject_inside "$RUN_DIR" "$PROTECTED_BASELINE_WORKTREE" RUN_DIR
	fi

	pin_tool_paths
	pin_cargo
	local source
	source="$(locate_source)"
	reject_inside "$source" "$CAMPAIGN_ROOT" source
	reject_inside "$source" "$PROTECTED_BASELINE_WORKTREE" source
	reject_inside "$OUT" "$CAMPAIGN_ROOT" OUT
	reject_inside "$OUT" "$PROTECTED_BASELINE_WORKTREE" OUT
	reject_inside "$CARGO_TARGET_DIR" "$CAMPAIGN_ROOT" CARGO_TARGET_DIR
	reject_inside "$CARGO_TARGET_DIR" "$PROTECTED_BASELINE_WORKTREE" CARGO_TARGET_DIR
	reject_overlap "$source" "$OUT" source-output
	reject_overlap "$source" "$CARGO_TARGET_DIR" source-target
	reject_overlap "$OUT" "$CARGO_TARGET_DIR" output-target

	verify_oracle_hashes "$source"
	prepare_fresh_dir "$OUT" OUT
	prepare_fresh_dir "$CARGO_TARGET_DIR" CARGO_TARGET_DIR
	prepare_source_scratch "$source"

	NEXTEST_TEST_THREADS="${NEXTEST_TEST_THREADS:-2}"
	[[ "$NEXTEST_TEST_THREADS" =~ ^[1-9][0-9]*$ ]] ||
		die "NEXTEST_TEST_THREADS must be a positive integer"
	export CARGO_TARGET_DIR NEXTEST_TEST_THREADS
	unset BLESS UPDATE_BASELINE TUISNAP_ACCEPT TUISNAP_BLESS || true

	build_nextest_args "$@"
	write_identity "$OUT" "$source"

	echo "known-good-visual-control: SOURCE=$source" >&2
	echo "known-good-visual-control: OUT=$OUT" >&2
	echo "known-good-visual-control: cargo=$(command -v cargo)" >&2
	echo "known-good-visual-control: CARGO_TARGET_DIR=$CARGO_TARGET_DIR" >&2
	echo "known-good-visual-control: NEXTEST_TEST_THREADS=$NEXTEST_TEST_THREADS" >&2
	echo "known-good-visual-control: cargo nextest ${NEXTEST_ARGS[*]}" >&2

	local status html_status
	set +e
	set -o pipefail
	(
		cd "$source"
		run_cmd cargo nextest "${NEXTEST_ARGS[@]}"
	) 2>&1 | tee "$OUT/visual-nextest.log"
	status="${PIPESTATUS[0]}"
	set -e

	set +e
	verify_html_exactness "$source" >"$OUT/html-check.txt" 2>&1
	html_status=$?
	set -e
	cat "$OUT/html-check.txt" >&2

	if protected_inputs_mutated "$source"; then
		echo "protected_inputs_mutated=true" >>"$OUT/identity.txt"
		die "oracle source inputs changed during calibration (nextest exit was $status)"
	fi
	echo "protected_inputs_mutated=false" >>"$OUT/identity.txt"
	build_status_report "$OUT" "$status" "$html_status"

	if ((status == 0 && html_status == 0)); then
		echo "known-good-visual-control: calibration=PASS" >&2
		exit 0
	fi
	echo "known-good-visual-control: calibration=FAIL nextest_exit=$status html_exact_exit=$html_status" >&2
	if ((status != 0)); then
		exit "$status"
	fi
	exit "$html_status"
}

main "$@"
