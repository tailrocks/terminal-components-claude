#!/usr/bin/env bash
# Install pinned taskfmt for campaign gates (pre-arm).
set -euo pipefail

TASKFMT_REV="${TASKFMT_REV:-52d9f1eb7721f409bc47beb9fced7997b5c13ede}"
TASKFMT_REPO="${TASKFMT_REPO:-https://github.com/donbeave/task-format.git}"
TASKFMT_SOURCE="${TC_TASKFMT_SOURCE:-/tmp/taskfmt-qualification}"
TASKFMT_ROOT="${TC_TASKFMT_INSTALL:-/tmp/taskfmt-install}"
EXPECTED_FP="${EXPECTED_TASKFMT_FINGERPRINT:-52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4}"

die() {
  echo "campaign-install-taskfmt: $*" >&2
  exit 1
}

main() {
  if [[ ! -d "$TASKFMT_SOURCE/.git" ]]; then
    git clone "$TASKFMT_REPO" "$TASKFMT_SOURCE"
  fi
  git -C "$TASKFMT_SOURCE" fetch origin --tags
  git -C "$TASKFMT_SOURCE" checkout "$TASKFMT_REV"

  cargo install --locked --root "$TASKFMT_ROOT" \
    --path "$TASKFMT_SOURCE/harness" --bin taskfmt

  local bin="$TASKFMT_ROOT/bin/taskfmt"
  [[ -x "$bin" ]] || die "install failed: $bin"

  echo "taskfmt: $($bin --version 2>&1 || true)"
  local fp
  fp="$("$bin" fingerprint)"
  echo "fingerprint: $fp"
  if [[ "$fp" != "$EXPECTED_FP" ]]; then
    echo "WARNING: fingerprint mismatch (expected $EXPECTED_FP)" >&2
    echo "Use a clean checkout at $TASKFMT_REV" >&2
  fi

  mkdir -p /tmp/tc-proof-bootstrap 2>/dev/null || true
  cp "$TASKFMT_SOURCE/experiment.toml" /tmp/tc-proof-bootstrap/experiment.toml 2>/dev/null || true

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
