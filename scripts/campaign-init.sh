#!/usr/bin/env bash
# Initialize single-branch campaign workspace (pre-arm). Does NOT arm /goal.
set -euo pipefail

ARCH_MAIN="${ARCH_MAIN:-7b27732a8c3c131760ec3438f641cb3c11343a42}"
INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
CATALOG_BRANCH="${CATALOG_BRANCH:-refactor/holla-parity}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.worktrees/campaign}"
CATALOG_MANIFEST_REL="docs/refactoring-plan/task-index.tsv"

repo_root() {
	git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
}

usage() {
	cat <<'EOF'
Usage: scripts/campaign-init.sh [--help|-h|help]

With no arguments, initialize the pre-arm campaign workspace.
Help and invalid arguments are read-only and never create or retarget refs,
worktrees, or ledger files.
EOF
}

die() {
	echo "campaign-init: $*" >&2
	exit 1
}

main() {
	if (($# > 0)); then
		case "$1" in
		help | -h | --help)
			(($# == 1)) || {
				usage >&2
				die "help accepts no additional arguments"
			}
			usage
			return 0
			;;
		*)
			usage >&2
			die "unknown argument: $1"
			;;
		esac
	fi

	local root
	root="$(repo_root)"
	cd "$root"

	local tag_peeled
	tag_peeled="$(git rev-parse "refs/tags/visual-baseline^{commit}" 2>/dev/null || echo UNKNOWN)"
	echo "==> visual-baseline tag (peeled): $tag_peeled"

	local seed_sha="$ARCH_MAIN"
	echo "==> Seeding only a missing $INTEGRATION_BRANCH from architectural main @ ${seed_sha:0:12}"

	if git show-ref --verify --quiet "refs/heads/$INTEGRATION_BRANCH"; then
		echo "==> Branch $INTEGRATION_BRANCH already exists @ $(git rev-parse "$INTEGRATION_BRANCH" | cut -c1-12)"
	else
		git branch "$INTEGRATION_BRANCH" "$seed_sha"
		echo "==> Created branch $INTEGRATION_BRANCH @ ${seed_sha:0:12}"
	fi

	if [[ -d "$WORKTREE_PATH" ]] || [[ -f "$WORKTREE_PATH/.git" ]]; then
		echo "==> Worktree exists: $WORKTREE_PATH"
		local worktree_branch
		worktree_branch="$(git -C "$WORKTREE_PATH" branch --show-current 2>/dev/null || echo detached)"
		[[ "$worktree_branch" == "$INTEGRATION_BRANCH" ]] ||
			die "existing worktree is on $worktree_branch; refusing to retarget it"
	else
		mkdir -p "$(dirname "$WORKTREE_PATH")"
		git worktree add "$WORKTREE_PATH" "$INTEGRATION_BRANCH"
		echo "==> Created worktree $WORKTREE_PATH"
	fi

	mkdir -p .campaign/runs .campaign/evidence
	if [[ ! -f .campaign/ledger.json ]]; then
		cp .campaign/ledger.template.json .campaign/ledger.json
	fi

	local catalog_time head_sha tree_sha catalog_manifest_sha
	catalog_time="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
	head_sha="$(git rev-parse "$INTEGRATION_BRANCH")"
	tree_sha="$(git rev-parse "$INTEGRATION_BRANCH^{tree}")"
	[[ -f "$root/$CATALOG_MANIFEST_REL" && ! -L "$root/$CATALOG_MANIFEST_REL" ]] ||
		die "missing or linked catalog manifest: $root/$CATALOG_MANIFEST_REL"
	catalog_manifest_sha="$(shasum -a 256 "$root/$CATALOG_MANIFEST_REL" | awk '{print $1}')"

	if command -v python3 >/dev/null 2>&1; then
		python3 - "$root" "$catalog_time" "$head_sha" "$tree_sha" "$catalog_manifest_sha" "$CATALOG_MANIFEST_REL" "$CATALOG_BRANCH" "$INTEGRATION_BRANCH" <<'PY'
import json, sys
root, catalog_time, head_sha, tree_sha, catalog_manifest_sha, catalog_manifest_rel, catalog_branch, integration_branch = sys.argv[1:9]
path = f"{root}/.campaign/ledger.json"
with open(path) as f:
    ledger = json.load(f)
ledger["integration_ref"] = f"refs/heads/{integration_branch}"
ledger["integration_head"] = head_sha
ledger["catalog"] = {
    "commit": head_sha,
    "tree": tree_sha,
    "branch": catalog_branch,
    "manifest": {"path": catalog_manifest_rel, "sha256": catalog_manifest_sha},
    "recorded_at": catalog_time,
}
with open(path, "w") as f:
    json.dump(ledger, f, indent=2)
    f.write("\n")
print(f"Updated {path}")
PY
	else
		echo "==> WARNING: python3 missing; edit .campaign/ledger.json manually"
	fi

	cat <<EOF

Campaign workspace initialized (pre-arm).

  Integration branch:  $INTEGRATION_BRANCH @ $(git rev-parse "$INTEGRATION_BRANCH" | cut -c1-12)
  Worktree:            $WORKTREE_PATH
  Ledger:              .campaign/ledger.json
  Catalog branch:      $CATALOG_BRANCH @ ${head_sha:0:12}/${tree_sha:0:12}

Next steps (do NOT arm /goal yet):
  1. ./scripts/campaign-install-taskfmt.sh
  2. Read docs/refactoring-plan/execution-readiness-report.md
  3. TC_CAMPAIGN_WORKTREE="$WORKTREE_PATH" INTEGRATION_BRANCH="$INTEGRATION_BRANCH" ./scripts/campaign-preflight.sh (fails closed while readiness is NO-GO)

Push when ready:
  git push -u origin $INTEGRATION_BRANCH

EOF
}

main "$@"
