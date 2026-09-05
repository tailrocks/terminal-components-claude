#!/usr/bin/env bash
# Execute one capture application without putting its argv through a shell.
# The normal tmux entrypoint has no user-controlled command arguments.  It
# reads the serialized argv from the run metadata and delegates to Python.
set -u

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
PROVENANCE_PYTHON=/usr/bin/python3

if (( $# != 0 )); then
  echo "capture_exec: positional argv mode is unsupported; use serialized run metadata" >&2
  exit 64
fi

if [[ ! -x "$PROVENANCE_PYTHON" ]]; then
  echo "capture_exec: trusted metadata Python is unavailable: $PROVENANCE_PYTHON" >&2
  exit 1
fi

: "${CAPTURE_METADATA_FILE:?capture_exec: metadata path is required}"
: "${CAPTURE_RUN_ID:?capture_exec: run id is required}"
: "${CAPTURE_STDERR_FILE:?capture_exec: stderr path is required}"
: "${CAPTURE_EXIT_FILE:?capture_exec: exit path is required}"
: "${CAPTURE_COLOR_MODE:?capture_exec: color mode is required}"
exec "$PROVENANCE_PYTHON" "$ROOT_DIR/tools/capture_provenance.py" exec \
  --metadata "$CAPTURE_METADATA_FILE" \
  --run-id "$CAPTURE_RUN_ID" \
  --stderr "$CAPTURE_STDERR_FILE" \
  --exit "$CAPTURE_EXIT_FILE" \
  --color "$CAPTURE_COLOR_MODE"
