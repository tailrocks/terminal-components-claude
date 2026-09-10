#!/usr/bin/env bash
# Capture every holla scenario at the four review sizes in truecolor, every
# scenario in mono, and the 16-colour frames that prove the tinted rows.
# Frames land in shots/h_*.{txt,html,png}. Drives tools/capture.sh.
#
# Scope is env-overridable so a change to one surface reruns only what moved:
#   SCENARIOS="hard-cases" SIZES="80x24 120x40" COLORS=mono tools/holla_shots.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export BIN=target/debug/holla
export PY=${PY:-python3}
SCENARIOS=${SCENARIOS:-"first-use rust-dirty monorepo-root monorepo-child docker-cleanup disk-cleanup upgrade-plan activities-multi remote-host launch-failure hard-cases"}
SIZES=${SIZES:-"80x24 100x30 120x40 160x50"}
COLORS=${COLORS:-"truecolor mono"}
shot() { tools/capture.sh shot "$1" >/dev/null; }
start() { ARGS="$1" tools/capture.sh start "$2" "$3"; sleep 1.2; }
keys() { tools/capture.sh keys "$@"; }
stop() { tools/capture.sh stop; }
for s in $SCENARIOS; do
  n=${s//-/_}
  for c in $COLORS; do
    if [ "$c" = truecolor ]; then
      for size in $SIZES; do
        w=${size%x*}; h=${size#*x}
        start "--scenario $s --motion paused --frame 40" "$w" "$h"
        shot "h_${n}_${size}"
        stop
      done
    else
      # mono at 100x30: every scenario must lose nothing but hue
      start "--scenario $s --motion paused --frame 40 --color none" 100 30
      shot "h_${n}_mono"
      stop
    fi
  done
done
# 16 colours at 100x30: the plan outline and the disk page
if [ -z "${SCENARIOS_SET:-}" ]; then
  for s in upgrade-plan disk-cleanup; do
    n=${s//-/_}
    start "--scenario $s --motion paused --frame 80 --color 16" 100 30
    shot "h_${n}_16"
    stop
  done
fi
echo "base frames done"
