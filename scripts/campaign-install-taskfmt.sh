#!/usr/bin/env bash
# Install the exact taskfmt source used by campaign gates.
set -euo pipefail

TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_VERSION="0.2.0"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="${TC_TASKFMT_SOURCE:-/Users/donbeave/Projects/taskfmt/task-format}"
TASKFMT_ROOT="${TC_TASKFMT_INSTALL:-/tmp/taskfmt-latest-install}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/campaign-path-guards.sh"

die() {
	echo "campaign-install-taskfmt: $*" >&2
	exit 1
}

materialize_single_link() {
	local bin="$1"
	campaign_require_regular_file "$bin" "taskfmt" 0 1 ||
		die "installed taskfmt failed pre-materialization trust-path validation: $bin"
	local before_sha
	before_sha="$(shasum -a 256 "$bin" | awk '{print $1}')" ||
		die "unable to hash installed taskfmt: $bin"
	local staged
	staged="$(mktemp "${bin}.materialize.XXXXXX")" ||
		die "unable to allocate single-link taskfmt staging file"
	if ! cp "$bin" "$staged"; then
		rm -f "$staged"
		die "unable to copy taskfmt into a single-link inode"
	fi
	if ! chmod 755 "$staged"; then
		rm -f "$staged"
		die "unable to set taskfmt executable mode"
	fi
	local staged_sha
	staged_sha="$(shasum -a 256 "$staged" | awk '{print $1}')" || {
		rm -f "$staged"
		die "unable to hash staged taskfmt"
	}
	if [[ "$staged_sha" != "$before_sha" ]]; then
		rm -f "$staged"
		die "single-link taskfmt materialization changed bytes"
	fi
	if ! mv -f "$staged" "$bin"; then
		rm -f "$staged"
		die "unable to install single-link taskfmt inode"
	fi
	campaign_require_regular_file "$bin" "taskfmt" 1 1 ||
		die "materialized taskfmt is not a regular single-link executable: $bin"
	local actual_sha
	actual_sha="$(shasum -a 256 "$bin" | awk '{print $1}')" ||
		die "unable to hash materialized taskfmt: $bin"
	[[ "$actual_sha" == "$before_sha" ]] ||
		die "materialized taskfmt hash changed: $actual_sha != $before_sha"
}

main() {
	[[ -d "$TASKFMT_SOURCE/.git" ]] || die "taskfmt source is not a git checkout: $TASKFMT_SOURCE"
	[[ -z "$(git -C "$TASKFMT_SOURCE" status --porcelain)" ]] ||
		die "taskfmt source is dirty: $TASKFMT_SOURCE"
	local source_rev
	source_rev="$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)"
	[[ "$source_rev" == "$TASKFMT_REV" ]] ||
		die "taskfmt source is $source_rev; expected latest $TASKFMT_REV"

	cargo install --locked --root "$TASKFMT_ROOT" \
		--path "$TASKFMT_SOURCE/crates/taskfmt" --bin taskfmt
	local bin="$TASKFMT_ROOT/bin/taskfmt"
	[[ -x "$bin" ]] || die "install failed: $bin"
	local installed_sha
	installed_sha="$(shasum -a 256 "$bin" | awk '{print $1}')" ||
		die "unable to hash installed taskfmt: $bin"
	[[ "$installed_sha" == "$TASKFMT_SHA256" ]] ||
		die "taskfmt SHA-256 before materialization is $installed_sha; expected $TASKFMT_SHA256"
	materialize_single_link "$bin"

	local version
	version="$($bin --version)"
	[[ "$version" == "taskfmt $TASKFMT_VERSION (git $TASKFMT_REV)" ]] ||
		die "taskfmt --version is '$version'; expected 'taskfmt $TASKFMT_VERSION (git $TASKFMT_REV)'"
	local actual_sha
	actual_sha="$(shasum -a 256 "$bin" | awk '{print $1}')"
	[[ "$actual_sha" == "$TASKFMT_SHA256" ]] ||
		die "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
	echo "taskfmt: $version"
	echo "taskfmt sha256: $actual_sha"
	echo "taskfmt links: 1"

	cat <<EOF

Installed:
  TC_TASKFMT=$bin
  TC_TASKFMT_SOURCE=$TASKFMT_SOURCE

Export for session:
  export TC_TASKFMT=$bin
  export TC_TASKFMT_SOURCE=$TASKFMT_SOURCE

EOF
}

main "$@"
