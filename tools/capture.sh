#!/usr/bin/env bash
# Headless capture harness.
#   tools/capture.sh start  <cols> <rows>          # launch app in tmux session
#   tools/capture.sh keys   <keys...>              # send tmux key names (e.g. Tab Down Enter "j")
#   tools/capture.sh mouse  <x> <y> [move|click|wheelup|wheeldown]   # send SGR mouse event
#   tools/capture.sh shot   <name>                 # capture ANSI, text, HTML and PNG
#   tools/capture.sh resize <cols> <rows>
#   tools/capture.sh stop
# Environment: BIN, ARGS (shell-quoted arguments), PY, SHOT_DIR,
# CAPTURE_SESSION, CAPTURE_SOCKET. PRESERVE_NO_COLOR=1 passes the caller's
# NO_COLOR value through; by default captures use the explicit --color flag.
# A dedicated tmux socket keeps capture settings away from personal sessions.
set -euo pipefail
cd "$(dirname "$0")/.."
S=${CAPTURE_SESSION:-junie_cap}
SOCKET=${CAPTURE_SOCKET:-junie-capture}
SHOT_DIR=${SHOT_DIR:-shots}
BIN=${BIN:-target/debug/showcase}
PY=${PY:-python3}
tm() { tmux -L "$SOCKET" "$@"; }
cmd=${1:-}; shift || true
case "$cmd" in
  start)
    cols=${1:-120}; rows=${2:-40}
    if [ ! -x "$BIN" ]; then
      echo "missing executable: $BIN (run cargo build --bins)" >&2
      exit 1
    fi
    mkdir -p "$SHOT_DIR"
    printf -v quoted_bin '%q' "$BIN"
    printf -v quoted_stderr '%q' "$SHOT_DIR/stderr.log"
    # Parse quoting, but never evaluate substitutions in arguments.
    quoted_args=$(python3 -B -c 'import shlex, sys; print(shlex.join(shlex.split(sys.argv[1])))' "${ARGS:-}")
    color_env="-u NO_COLOR"
    if [ "${PRESERVE_NO_COLOR:-0}" = 1 ]; then
      printf -v quoted_no_color '%q' "${NO_COLOR:-}"
      color_env="NO_COLOR=$quoted_no_color"
    fi
    tm kill-session -t "=$S" 2>/dev/null || true
    tm -f /dev/null new-session -d -s "$S" -c "$PWD" -x "$cols" -y "$rows" \
      "exec env $color_env TERM=xterm-256color COLORTERM=truecolor $quoted_bin $quoted_args 2>$quoted_stderr"
    tm set-option -t "$S" status off
    # Keep this private server alive between captures to avoid shutdown/start races.
    tm set-option -s exit-empty off
    tm set-option -s escape-time 0
    tm set-option -g default-terminal "tmux-256color"
    tm set-option -ga terminal-overrides ",*:Tc"
    sleep "${CAPTURE_DELAY:-0.6}"
    if ! tm has-session -t "=$S" 2>/dev/null; then
      echo "application exited before capture; inspect $SHOT_DIR/stderr.log" >&2
      exit 1
    fi
    ;;
  keys)
    for k in "$@"; do tm send-keys -t "=$S:" "$k"; sleep 0.08; done
    sleep 0.15
    ;;
  mouse)
    x=$1; y=$2; kind=${3:-move}
    case "$kind" in
      move)      seq=$(printf '\e[<35;%d;%dM' "$x" "$y") ;;
      click)     seq=$(printf '\e[<0;%d;%dM\e[<0;%d;%dm' "$x" "$y" "$x" "$y") ;;
      rclick)    seq=$(printf '\e[<2;%d;%dM\e[<2;%d;%dm' "$x" "$y" "$x" "$y") ;;
      down)      seq=$(printf '\e[<0;%d;%dM' "$x" "$y") ;;
      up)        seq=$(printf '\e[<0;%d;%dm' "$x" "$y") ;;
      drag)      seq=$(printf '\e[<32;%d;%dM' "$x" "$y") ;;
      wheelup)   seq=$(printf '\e[<64;%d;%dM' "$x" "$y") ;;
      wheeldown) seq=$(printf '\e[<65;%d;%dM' "$x" "$y") ;;
      *) echo "unknown mouse action: $kind" >&2; exit 1 ;;
    esac
    tm send-keys -t "=$S:" -l "$seq"
    sleep 0.15
    ;;
  shot)
    name=${1:-shot}
    case "$name" in */*) echo "capture name must not contain /" >&2; exit 1 ;; esac
    cols=$(tm display -p -t "=$S:" '#{pane_width}'); rows=$(tm display -p -t "=$S:" '#{pane_height}')
    tm capture-pane -t "=$S:" -e -p -N > "$SHOT_DIR/$name.ansi"
    tm display -p -t "=$S:" "#{cursor_x} #{cursor_y} #{cursor_flag}" > "$SHOT_DIR/$name.cursor"
    tm capture-pane -t "=$S:" -p -N > "$SHOT_DIR/$name.txt"
    "$PY" -B tools/ansi2html.py "$SHOT_DIR/$name.ansi" "$SHOT_DIR/$name.html" "$cols" "$rows"
    "$PY" -B tools/ansi2png.py "$SHOT_DIR/$name.ansi" "$SHOT_DIR/$name.png" "$cols" "$rows" "$SHOT_DIR/$name.cursor"
    echo "$SHOT_DIR/$name.png ($cols x $rows)"
    ;;
  resize)
    tm resize-window -t "=$S:" -x "$1" -y "$2"; sleep 0.3
    ;;
  stop)
    tm kill-session -t "=$S" 2>/dev/null || true
    ;;
  *) echo "unknown: $cmd" >&2; exit 1 ;;
esac
