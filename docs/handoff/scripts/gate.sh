#!/bin/sh
# Gate from ONE nextest archive: archive, then unit/linux (2 parts each) and visual check-touched (4 parts); each phase under 4 min.
# usage: gate.sh <phase>   phases: archive | unit | linux | visual
# run from the worktree root
D=${GATE_DIR:?set GATE_DIR to your own scratch dir (archive, logs)}
A=$D/a.tar.zst
case "$1" in
archive)
  cargo xtask visual check-touched --dry-run 2>/dev/null | grep "^test(" | tail -1 > $D/dry.txt
  rm -f $A
  cargo nextest archive --all-targets --archive-file $A > $D/archive.txt 2>&1
  echo "archive exit $?"
  ;;
unit)
  for k in 1 2; do
    cargo nextest run --archive-file $A --workspace-remap . --no-fail-fast --partition hash:$k/2 > $D/unit-$k.txt 2>&1 &
  done
  wait
  for k in 1 2; do grep -E "Summary \[" $D/unit-$k.txt || echo "unit $k NO SUMMARY"; done
  grep -hE "^\s+FAIL \[" $D/unit-*.txt | sort -u | head -40
  ;;
linux)
  for k in 1 2; do
    JUNIE_TUI_CLIENT_PLATFORM=linux cargo nextest run --archive-file $A --workspace-remap . --no-fail-fast --partition hash:$k/2 > $D/linux-$k.txt 2>&1 &
  done
  wait
  for k in 1 2; do grep -E "Summary \[" $D/linux-$k.txt || echo "linux $k NO SUMMARY"; done
  grep -hE "^\s+FAIL \[" $D/linux-*.txt | sort -u | head -40
  ;;
visual)
  F="binary(visual_baseline) & ($(cat $D/dry.txt))"
  for k in 1 2 3 4; do
    cargo nextest run --archive-file $A --workspace-remap . --run-ignored only --no-fail-fast --test-threads 4 --partition hash:$k/4 -E "$F" > $D/vis-$k.txt 2>&1 &
  done
  wait
  for k in 1 2 3 4; do printf "part %s: " $k; grep -E "Summary \[" $D/vis-$k.txt || echo "NO SUMMARY"; done
  grep -hE "^\s+FAIL \[" $D/vis-*.txt | sort -u | head -60
  ;;
esac
