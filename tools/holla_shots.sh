#!/usr/bin/env bash
# Capture every holla scenario at the four review sizes plus mono, and the
# key flows. Frames land in shots/h_*.{txt,html,png}. Drives tools/capture.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
export BIN=target/debug/holla
export PY=${PY:-python3}
SCENARIOS="first-use rust-dirty monorepo-root monorepo-child docker-cleanup disk-cleanup upgrade-plan activities-multi remote-host launch-failure hard-cases"
shot() { tools/capture.sh shot "$1" >/dev/null; }
start() { ARGS="$1" tools/capture.sh start "$2" "$3"; sleep 1.2; }
keys() { tools/capture.sh keys "$@"; }
stop() { tools/capture.sh stop; }
for s in $SCENARIOS; do
  n=${s//-/_}
  for size in "80 24" "100 30" "120 40" "160 50"; do
    set -- $size
    start "--scenario $s --motion paused --frame 40" "$1" "$2"
    shot "h_${n}_${1}x${2}"
    stop
  done
done
# mono at 100x30 for the dense scenarios and the plan outline
for s in rust-dirty docker-cleanup upgrade-plan; do
  n=${s//-/_}
  start "--scenario $s --motion paused --frame 40 --color none" 100 30
  shot "h_${n}_mono"
  stop
done
# 16 colours at 100x30: the plan outline and the disk page
for s in upgrade-plan disk-cleanup; do
  n=${s//-/_}
  start "--scenario $s --motion paused --frame 80 --color 16" 100 30
  shot "h_${n}_16"
  stop
done
echo "base frames done"
