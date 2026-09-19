#!/usr/bin/env bash
# Known-good visual control: peeled visual-baseline tag vs frozen snapshots.
# Scratch-only launcher. Does not edit the git repo, snapshots/, or bless.
set -euo pipefail

ORACLE_TAG_OBJECT="1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
ORACLE_COMMIT="4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
ORACLE_TREE="0b1f13431fdfd6060cf9f45a114afa5a99cc6c26"
ORACLE_SNAPSHOTS_TREE="3f0261c32849e26feda24d87697de4a7ce6b8375"
PREFERRED_SOURCE="/private/tmp/campaign-baseline-source"
# Exact string frozen in HTML provenance.argv[0]. Do not realpath.
FROZEN_TARGET_DIR="/Users/donbeave/Projects/terminal-components-claude/target"
RUSTUP_CARGO_BIN="/Users/donbeave/.rustup/toolchains/1.88.0-aarch64-apple-darwin/bin"
DEFAULT_FILTER="binary(visual_baseline)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAMPAIGN_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# mbx cargo remaps CARGO_TARGET_DIR to ~/Library/Caches/mbx/targets/v1/<hash>,
# which breaks frozen HTML argv[0]. Prefer rustup 1.88 cargo, then ~/.cargo/bin
# (cargo-nextest), then the rest of PATH.
export PATH="${RUSTUP_CARGO_BIN}:/Users/donbeave/.cargo/bin:$PATH"

usage() {
	cat <<'EOF'
Usage: scripts/run-known-good-visual-control.sh [nextest -E filter | nextest args...]

Known-good control: detached visual-baseline tag source
4a79c0a2d40fca46fc406b77157ce3b3f12ec16b against frozen snapshots/.

  (no args)     cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
  EXPR          -E EXPR  (expression; may omit binary() — still nextest -E)
  -E EXPR ...   passed through to cargo nextest after --run-ignored only --locked

Env:
  OUT                  default: <campaign>/target/known-good-visual
  KNOWN_GOOD_SOURCE    default: /private/tmp/campaign-baseline-source
                       (must be exactly the peeled tag commit)
  NEXTEST_TEST_THREADS default: 2

Always:
  CARGO_TARGET_DIR=/Users/donbeave/Projects/terminal-components-claude/target
  rustup 1.88 cargo before mbx on PATH
  uses source .config/nextest.toml pty group
  rtk proxy if rtk exists (full log; rtk cargo nextest is failures-only)
  records $OUT/exit.code and $OUT/visual-nextest.log
  refuses tuisnap accept / BLESS / UPDATE_BASELINE / cargo test
  does not write snapshots/
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
	run_cmd git -C "$dir" "$@"
}

expect_eq() {
	local name="$1" actual="$2" expected="$3"
	if [[ "$actual" != "$expected" ]]; then
		die "oracle hash mismatch: ${name} actual=${actual} expected=${expected}"
	fi
}

refuse_forbidden_args() {
	local i arg next
	for ((i = 1; i <= $#; i++)); do
		arg="${!i}"
		case "$arg" in
		accept | --accept | tuisnap | BLESS | BLESS=* | UPDATE_BASELINE | UPDATE_BASELINE=* | --bless)
			die "refusing oracle mutation arg: $arg"
			;;
		esac
		if [[ "$arg" == *tuisnap* && "$arg" == *accept* ]]; then
			die "refusing oracle mutation arg: $arg"
		fi
		if [[ "$arg" == "test" && i -gt 1 ]]; then
			local prev=$((i - 1))
			if [[ "${!prev}" == "cargo" ]]; then
				die "never cargo test; use cargo nextest"
			fi
		fi
		if [[ "$arg" == "cargo" && i -lt $# ]]; then
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

pin_rustup_cargo() {
	local cargo_bin
	[[ -x "$RUSTUP_CARGO_BIN/cargo" ]] ||
		die "missing rustup 1.88 cargo: $RUSTUP_CARGO_BIN/cargo"
	cargo_bin="$(command -v cargo)"
	[[ "$cargo_bin" == "$RUSTUP_CARGO_BIN/cargo" ]] ||
		die "cargo is $cargo_bin; rustup 1.88 must precede mbx on PATH"
}

locate_source() {
	local candidate="${KNOWN_GOOD_SOURCE:-$PREFERRED_SOURCE}"
	[[ -d "$candidate" ]] || die "missing detached tag source: $candidate"
	[[ -d "$candidate/.git" || -f "$candidate/.git" ]] ||
		die "not a git checkout: $candidate"
	local head
	head="$(git_in "$candidate" rev-parse HEAD)"
	[[ "$head" == "$ORACLE_COMMIT" ]] ||
		die "source $candidate HEAD=$head (expected detached $ORACLE_COMMIT)"
	printf '%s\n' "$candidate"
}

verify_oracle_hashes() {
	local source="$1"
	local tag_object peeled tree snapshots_tree head head_tree head_snapshots
	# Disambiguate: branch and tag share the name visual-baseline.
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
	if [[ -n "$(git_in "$source" status --porcelain -- snapshots tests/visual_baseline .config/nextest.toml)" ]]; then
		die "tag source working tree is dirty on oracle/suite paths"
	fi
	if [[ -L "$source/snapshots" ]]; then
		die "snapshots/ is a symlink; refuse (oracle must be a real tree)"
	fi
	[[ -d "$source/snapshots" ]] || die "missing $source/snapshots"
	[[ -f "$source/.config/nextest.toml" ]] || die "missing $source/.config/nextest.toml"
	[[ -d "$source/tests/visual_baseline" ]] || die "missing $source/tests/visual_baseline"
	echo "known-good-visual-control: OK: tag $ORACLE_COMMIT tree $ORACLE_TREE snapshots $ORACLE_SNAPSHOTS_TREE"
}

assert_frozen_target() {
	[[ -e "$FROZEN_TARGET_DIR" ]] ||
		die "frozen CARGO_TARGET_DIR missing: $FROZEN_TARGET_DIR (HTML argv[0] pin)"
	local given resolved
	given="$FROZEN_TARGET_DIR"
	resolved="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$given")"
	if [[ "$given" == "$resolved" ]]; then
		echo "known-good-visual-control: CARGO_TARGET_DIR=$given"
	else
		echo "known-good-visual-control: CARGO_TARGET_DIR=$given -> $resolved (keeping given string for argv[0])"
	fi
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
		# User supplied flags; still require a filter if they omitted -E.
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

clean_tuisnap_scratch() {
	local source="$1"
	local scratch="$source/target/tuisnap"
	# Relative target/tuisnap is hardcoded in tests/visual_baseline/support.rs.
	# Wipe only that scratch. Never snapshots/.
	if [[ -e "$scratch" || -L "$scratch" ]]; then
		rm -rf "$scratch"
	fi
	mkdir -p "$scratch/actual" "$scratch/diff"
}

snapshots_mutated() {
	local source="$1"
	if ! git_in "$source" diff --quiet -- snapshots; then
		return 0
	fi
	if ! git_in "$source" diff --quiet --cached -- snapshots; then
		return 0
	fi
	if [[ -n "$(git_in "$source" status --porcelain -- snapshots)" ]]; then
		return 0
	fi
	return 1
}

write_identity() {
	local out="$1" source="$2"
	cat >"$out/identity.txt" <<EOF
role=known-good-visual-control
source=$source
head=$(git_in "$source" rev-parse HEAD)
tree=$(git_in "$source" rev-parse 'HEAD^{tree}')
snapshots_tree=$(git_in "$source" rev-parse 'HEAD:snapshots')
tag_object=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tag}')
tag_peeled=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{commit}')
tag_tree=$(git_in "$source" rev-parse 'refs/tags/visual-baseline^{tree}')
cargo_bin=$(command -v cargo)
cargo_target_dir=$CARGO_TARGET_DIR
nextest_test_threads=${NEXTEST_TEST_THREADS:-}
nextest_args=${NEXTEST_ARGS[*]}
started_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
rtk=$(command -v rtk 2>/dev/null || echo none)
EOF
}

main() {
	if [[ $# -eq 1 && ( "$1" == "-h" || "$1" == "--help" ) ]]; then
		usage
		exit 0
	fi
	refuse_forbidden_env
	refuse_forbidden_args "$@"
	pin_rustup_cargo

	local source out status
	source="$(locate_source)"
	verify_oracle_hashes "$source"
	assert_frozen_target

	out="${OUT:-$CAMPAIGN_ROOT/target/known-good-visual}"
	case "$out" in
	"$source/snapshots" | "$source/snapshots"/* | */snapshots | */snapshots/*)
		die "OUT must not be inside snapshots/: $out"
		;;
	esac
	mkdir -p "$out"

	# Force exact argv0 pin. Do not inherit a relocated target.
	if [[ -n "${CARGO_TARGET_DIR:-}" && "$CARGO_TARGET_DIR" != "$FROZEN_TARGET_DIR" ]]; then
		echo "known-good-visual-control: overriding CARGO_TARGET_DIR=$CARGO_TARGET_DIR -> $FROZEN_TARGET_DIR" >&2
	fi
	export CARGO_TARGET_DIR="$FROZEN_TARGET_DIR"
	export NEXTEST_TEST_THREADS="${NEXTEST_TEST_THREADS:-2}"
	unset BLESS UPDATE_BASELINE TUISNAP_ACCEPT TUISNAP_BLESS || true

	build_nextest_args "$@"
	write_identity "$out" "$source"
	clean_tuisnap_scratch "$source"

	echo "known-good-visual-control: SOURCE=$source" >&2
	echo "known-good-visual-control: OUT=$out" >&2
	echo "known-good-visual-control: cargo=$(command -v cargo)" >&2
	echo "known-good-visual-control: CARGO_TARGET_DIR=$CARGO_TARGET_DIR" >&2
	echo "known-good-visual-control: NEXTEST_TEST_THREADS=$NEXTEST_TEST_THREADS (source pty group still applies)" >&2
	echo "known-good-visual-control: cargo nextest ${NEXTEST_ARGS[*]}" >&2

	set +e
	set -o pipefail
	(
		cd "$source"
		run_cmd cargo nextest "${NEXTEST_ARGS[@]}"
	) 2>&1 | tee "$out/visual-nextest.log"
	status="${PIPESTATUS[0]}"
	set -e
	printf '%s\n' "$status" >"$out/exit.code"
	{
		echo "finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
		echo "exit=$status"
	} >>"$out/identity.txt"

	if snapshots_mutated "$source"; then
		echo "snapshots_mutated=true" >>"$out/identity.txt"
		die "snapshots/ changed during control run; oracle mutation is forbidden (exit was $status)"
	fi
	echo "snapshots_mutated=false" >>"$out/identity.txt"

	echo "known-good-visual-control: exit=$status log=$out/visual-nextest.log" >&2
	exit "$status"
}

main "$@"
