#!/usr/bin/env bash
# P6 full matrix: every scenario × 4 sizes × 3 color levels.
# One tmux session per (scenario, color); resize for the four sizes.
set -uo pipefail
cd "$(dirname "$0")/.."
export BIN=target/debug/holla
export PY=/usr/bin/python3
SCENARIOS="${SCENARIOS:-first-use rust-dirty monorepo-root monorepo-child docker-cleanup disk-cleanup upgrade-plan activities-multi remote-host launch-failure hard-cases}"
SIZES="${SIZES:-80x24 100x30 120x40 160x50}"
COLORS="${COLORS:-truecolor 256 mono}"
fail=0
for sc in $SCENARIOS; do
  for color in $COLORS; do
    ARGS="--scenario $sc --motion paused --frame 4000 --color $color" tools/capture.sh start 80 24 >/dev/null
    for sz in $SIZES; do
      cols=${sz%x*}; rows=${sz#*x}
      tools/capture.sh resize "$cols" "$rows" >/dev/null
      name="h_p6_${sc}_${sz}_${color}"
      tools/capture.sh shot "$name" >/dev/null || { echo "FAIL $name"; fail=1; }
    done
    tools/capture.sh stop >/dev/null
  done
done
echo "matrix done (fail=$fail): $(ls shots/h_p6_*.png 2>/dev/null | wc -l) pngs"
