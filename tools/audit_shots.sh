#!/usr/bin/env bash
# Representative applications at minimum/small/normal/wide sizes and every
# colour level, including actual NO_COLOR=1 backend suppression.
# Build first: cargo build --bins
# PY=/path/to/python-with-pillow tools/audit_shots.sh
# Override CASES, SIZES, COLORS or SHOT_DIR to rerun one changed surface.
set -euo pipefail
cd "$(dirname "$0")/.."
export PY=${PY:-python3}
export SHOT_DIR=${SHOT_DIR:-shots/audit}
export CAPTURE_SESSION=${CAPTURE_SESSION:-audit}
private_socket=0
if [ -z "${CAPTURE_SOCKET+x}" ]; then private_socket=1; fi
export CAPTURE_SOCKET=${CAPTURE_SOCKET:-junie-audit-$$}
CASES=${CASES:-"showcase-buttons showcase-forms tablepro-production jackin-capsule jackin-accounts holla-rust holla-upgrade"}
SIZES=${SIZES:-"72x20 80x24 100x30 120x40 160x50"}
COLORS=${COLORS:-"truecolor 256 16 none no_color"}
"$PY" -B -c 'from PIL import Image' || {
  echo "PNG capture requires Pillow; set PY to an existing interpreter with Pillow" >&2
  exit 1
}
cleanup() {
  tools/capture.sh stop
  if [ "$private_socket" = 1 ]; then
    tmux -L "$CAPTURE_SOCKET" kill-server 2>/dev/null || true
  elif ! tmux -L "$CAPTURE_SOCKET" list-sessions >/dev/null 2>&1; then
    tmux -L "$CAPTURE_SOCKET" set-option -s exit-empty on 2>/dev/null || true
  fi
}
trap cleanup EXIT
for fixture in $CASES; do
  case "$fixture" in
    showcase-buttons) BIN=target/debug/showcase; base_args="--page buttons" ;;
    showcase-forms) BIN=target/debug/showcase; base_args="--page forms" ;;
    showcase-inputs) BIN=target/debug/showcase; base_args="--page inputs" ;;
    showcase-textareas) BIN=target/debug/showcase; base_args="--page textareas" ;;
    showcase-terminal) BIN=target/debug/showcase; base_args="--page terminal" ;;
    showcase-editor) BIN=target/debug/showcase; base_args="--page codeeditor" ;;
    showcase-datagrid) BIN=target/debug/showcase; base_args="--page datagrid" ;;
    showcase-diff) BIN=target/debug/showcase; base_args="--page diff" ;;
    tablepro-production) BIN=target/debug/tablepro; base_args="--connect Production" ;;
    jackin-capsule) BIN=target/debug/jackin-preview; base_args="--scenario capsule-multi --motion paused --frame 40" ;;
    jackin-accounts) BIN=target/debug/jackin-preview; base_args="--scenario accounts-mixed --motion paused --frame 40" ;;
    jackin-hard-cases) BIN=target/debug/jackin-preview; base_args="--scenario hard-cases --motion paused --frame 40" ;;
    holla-rust) BIN=target/debug/holla; base_args="--scenario rust-dirty --motion paused --frame 40" ;;
    holla-upgrade) BIN=target/debug/holla; base_args="--scenario upgrade-plan --motion paused --frame 40" ;;
    holla-activities) BIN=target/debug/holla; base_args="--scenario activities-multi --motion paused --frame 40" ;;
    holla-hard-cases) BIN=target/debug/holla; base_args="--scenario hard-cases --motion paused --frame 40" ;;
    *) echo "unknown audit case: $fixture" >&2; exit 1 ;;
  esac
  export BIN
  for size in $SIZES; do
    w=${size%x*}; h=${size#*x}
    for color in $COLORS; do
      case "$color" in
        no_color)
          NO_COLOR=1 PRESERVE_NO_COLOR=1 ARGS="$base_args" tools/capture.sh start "$w" "$h"
          ;;
        truecolor|256|16|none)
          PRESERVE_NO_COLOR=0 ARGS="$base_args --color $color" tools/capture.sh start "$w" "$h"
          ;;
        *) echo "unknown audit color: $color" >&2; exit 1 ;;
      esac
      tools/capture.sh shot "${fixture}_${size}_${color}"
      tools/capture.sh stop
    done
  done
done
