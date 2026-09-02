#!/usr/bin/env bash
# Headless capture harness.
#   tools/capture.sh start  <cols> <rows>          # launch app in tmux session
#   tools/capture.sh keys   <keys...>              # send tmux key names (e.g. Tab Down Enter "j")
#   tools/capture.sh mouse  <x> <y> [move|click|wheelup|wheeldown]   # send SGR mouse event
#   tools/capture.sh shot   <name>                 # capture to shots/<name>.{ansi,html,txt}
#   tools/capture.sh resize <cols> <rows>
#   tools/capture.sh stop
set -euo pipefail
cd "$(dirname "$0")/.."
S=junie_cap
BIN=${BIN:-target/debug/junie-tui}
cmd=${1:-}; shift || true
case "$cmd" in
  start)
    cols=${1:-120}; rows=${2:-40}
    tmux kill-session -t $S 2>/dev/null || true
    tmux -f /dev/null new-session -d -s $S -x "$cols" -y "$rows" \
      "env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor ${BIN} ${ARGS:-} 2>shots/stderr.log; sleep 30"
    tmux set-option -t $S status off
    tmux set-option -s escape-time 0
    tmux set-option -g default-terminal "tmux-256color"
    tmux set-option -ga terminal-overrides ",*:Tc"
    sleep 0.6
    ;;
  keys)
    for k in "$@"; do tmux send-keys -t $S "$k"; sleep 0.08; done
    sleep 0.15
    ;;
  mouse)
    x=$1; y=$2; kind=${3:-move}
    case "$kind" in
      move)      seq=$(printf '\e[<35;%d;%dM' "$x" "$y") ;;
      click)     seq=$(printf '\e[<0;%d;%dM\e[<0;%d;%dm' "$x" "$y" "$x" "$y") ;;
      down)      seq=$(printf '\e[<0;%d;%dM' "$x" "$y") ;;
      up)        seq=$(printf '\e[<0;%d;%dm' "$x" "$y") ;;
      drag)      seq=$(printf '\e[<32;%d;%dM' "$x" "$y") ;;
      wheelup)   seq=$(printf '\e[<64;%d;%dM' "$x" "$y") ;;
      wheeldown) seq=$(printf '\e[<65;%d;%dM' "$x" "$y") ;;
    esac
    tmux send-keys -t $S -l "$seq"
    sleep 0.15
    ;;
  shot)
    name=${1:-shot}
    cols=$(tmux display -p -t $S '#{pane_width}'); rows=$(tmux display -p -t $S '#{pane_height}')
    tmux capture-pane -t $S -e -p -N > "shots/$name.ansi"
    tmux display -p -t $S "#{cursor_x} #{cursor_y} #{cursor_flag}" > "shots/$name.cursor"
    tmux capture-pane -t $S -p > "shots/$name.txt"
    python3 tools/ansi2html.py "shots/$name.ansi" "shots/$name.html" "$cols" "$rows"
    ${PY:-python3} tools/ansi2png.py "shots/$name.ansi" "shots/$name.png" "$cols" "$rows" "shots/$name.cursor" 2>/dev/null || true
    echo "shots/$name.html ($cols x $rows)"
    ;;
  resize)
    tmux resize-window -t $S -x "$1" -y "$2"; sleep 0.3
    ;;
  stop)
    tmux kill-session -t $S 2>/dev/null || true
    ;;
  *) echo "unknown: $cmd" >&2; exit 1 ;;
esac
