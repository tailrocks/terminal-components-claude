#!/usr/bin/env bash
# Bounded regression checks for preparation-only shell guards.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
INIT="$SCRIPT_DIR/campaign-init.sh"
PREFLIGHT="$SCRIPT_DIR/campaign-preflight.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/tc-preparation-guards.XXXXXX")"

cleanup() {
	rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
	echo "test_campaign_preparation_guards: FAIL: $*" >&2
	exit 1
}

pass() {
	echo "test_campaign_preparation_guards: PASS: $*"
}

copy_without_main() {
	local source="$1"
	local destination="$2"
	python3 - "$source" "$destination" <<'PY'
import sys
from pathlib import Path

source, destination = map(Path, sys.argv[1:])
lines = source.read_text(encoding="utf-8").splitlines()
try:
    lines.remove('main "$@"')
except ValueError as error:
    raise SystemExit(f"{source} main entry point changed") from error
destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

assert_init_read_only() {
	local label="$1"
	local argument="$2"
	local expected_status="$3"
	local root="$TMP_ROOT/init-$label"
	local integration_branch="fixture-$label"
	local catalog_branch="catalog-$label"
	mkdir -p "$root/scripts"
	cp "$INIT" "$root/scripts/campaign-init.sh"
	chmod +x "$root/scripts/campaign-init.sh"
	git init -q "$root"
	git -C "$root" config user.email test@example.invalid
	git -C "$root" config user.name "Preparation Guard Test"
	printf '%s\n' fixture >"$root/README.md"
	git -C "$root" add README.md
	git -C "$root" commit -q -m fixture

	local seed
	seed="$(git -C "$root" rev-parse HEAD)"
	local before_entries before_refs before_worktrees
	before_entries="$(find "$root" -mindepth 1 -print | sort)"
	before_refs="$(git -C "$root" for-each-ref --format='%(refname) %(objectname)' refs/heads)"
	before_worktrees="$(git -C "$root" worktree list --porcelain)"

	local output status
	if output="$(
		env ARCH_MAIN="$seed" \
			INTEGRATION_BRANCH="$integration_branch" \
			CATALOG_BRANCH="$catalog_branch" \
			TC_CAMPAIGN_WORKTREE="$root/worktree" \
			"$root/scripts/campaign-init.sh" "$argument" 2>&1
	)"; then
		status=0
	else
		status=$?
	fi
	[[ "$status" == "$expected_status" ]] ||
		fail "$label returned $status, expected $expected_status: $output"

	local after_entries after_refs after_worktrees
	after_entries="$(find "$root" -mindepth 1 -print | sort)"
	after_refs="$(git -C "$root" for-each-ref --format='%(refname) %(objectname)' refs/heads)"
	after_worktrees="$(git -C "$root" worktree list --porcelain)"
	[[ "$after_entries" == "$before_entries" ]] ||
		fail "$label changed filesystem entries: $output"
	[[ "$after_refs" == "$before_refs" ]] ||
		fail "$label changed refs: $output"
	[[ "$after_worktrees" == "$before_worktrees" ]] ||
		fail "$label changed worktrees: $output"
	printf '%s\n' "$output" | grep -Fq "Usage: scripts/campaign-init.sh" ||
		fail "$label did not print usage: $output"
	pass "campaign-init $label is non-mutating"
}

assert_init_read_only help --help 0
assert_init_read_only unknown unknown 1

preflight_root="$TMP_ROOT/preflight"
mkdir -p "$preflight_root/scripts" "$preflight_root/docs/refactoring-plan"
git init -q "$preflight_root"
git -C "$preflight_root" config user.email test@example.invalid
git -C "$preflight_root" config user.name "Preparation Guard Test"
git -C "$preflight_root" checkout -q -b refactor/holla-parity
printf '%s\n' fixture >"$preflight_root/README.md"
printf '%s\n' '**NO-GO.**' > \
	"$preflight_root/docs/refactoring-plan/execution-readiness-report.md"
git -C "$preflight_root" add README.md docs/refactoring-plan/execution-readiness-report.md
git -C "$preflight_root" commit -q -m fixture
preflight_helper="$preflight_root/scripts/campaign-preflight-functions.sh"
copy_without_main "$PREFLIGHT" "$preflight_helper"
git -C "$preflight_root" add scripts/campaign-preflight-functions.sh
git -C "$preflight_root" commit -q -m helper
cd "$preflight_root"

printf '%s\n' dirty >"$preflight_root/dirty-marker"
if output="$(
	env TC_CAMPAIGN_WORKTREE="$preflight_root" INTEGRATION_BRANCH=refactor/holla-parity \
		bash -c "source \"\$1\"; check_worktree" _ "$preflight_helper" 2>&1
)"; then
	fail "dirty candidate worktree was accepted"
fi
printf '%s\n' "$output" | grep -Fq "worktree $preflight_root is dirty" ||
	fail "dirty worktree had unexpected output: $output"
pass "campaign-preflight rejects a dirty candidate worktree"
rm "$preflight_root/dirty-marker"

if output="$(
	env TC_CAMPAIGN_WORKTREE="$preflight_root" INTEGRATION_BRANCH=refactor/holla-parity \
		bash -c "source \"\$1\"; check_worktree; check_readiness_gate" _ "$preflight_helper" 2>&1
)"; then
	fail "NO-GO readiness report was accepted"
fi
printf '%s\n' "$output" | grep -Fq "worktree $preflight_root on refactor/holla-parity" ||
	fail "clean worktree check did not pass before NO-GO: $output"
printf '%s\n' "$output" | grep -Fq "readiness report is NO-GO" ||
	fail "NO-GO report behavior changed: $output"
pass "campaign-preflight preserves the explicit NO-GO failure"

plan_root="$TMP_ROOT/plan"
mkdir -p "$plan_root/scripts" "$plan_root/docs/refactoring-plan/evidence"
git init -q "$plan_root"
plan_helper="$plan_root/scripts/campaign-preflight-functions.sh"
copy_without_main "$PREFLIGHT" "$plan_helper"
python3 - "$plan_root/docs/refactoring-plan/evidence/validate-plan.py" <<'PY'
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(
    "import sys\n"
    "print('plan-validator diagnostic', file=sys.stderr)\n"
    "print('{\"error_count\": 1}')\n",
    encoding="utf-8",
)
PY
if output="$(
	bash -c "source \"\$1\"; check_validate_plan" _ "$plan_helper" 2>&1
)"; then
	fail "failing plan validator was accepted"
fi
printf '%s\n' "$output" | grep -Fq "plan-validator diagnostic" ||
	fail "plan-validator stderr was suppressed: $output"
printf '%s\n' "$output" | grep -Fq "validate-plan not green or script failed" ||
	fail "plan-validator failure had unexpected output: $output"
pass "campaign-preflight preserves plan-validator diagnostics"

[[ -L "$SCRIPT_DIR/../CLAUDE.md" ]] ||
	fail "repository instruction symlink is missing"
pass "legitimate repository symlink remains allowed"

echo "test_campaign_preparation_guards: all bounded checks passed"
