#!/usr/bin/env bash
# Install the exact taskfmt source used by campaign gates.
set -euo pipefail

TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
TASKFMT_ROOT="${TC_TASKFMT_INSTALL:-/tmp/taskfmt-latest-install}"

die() {
	echo "campaign-install-taskfmt: $*" >&2
	exit 1
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

	local version
	version="$($bin --version)"
	[[ "$version" == *"git $TASKFMT_REV"* ]] ||
		die "taskfmt identity mismatch: $version"
	local actual_sha
	actual_sha="$(shasum -a 256 "$bin" | awk '{print $1}')"
	[[ "$actual_sha" == "$TASKFMT_SHA256" ]] ||
		die "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
	echo "taskfmt: $version"
	echo "taskfmt sha256: $actual_sha"

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
