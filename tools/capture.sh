#!/usr/bin/env bash
# Headless capture harness.
#   tools/capture.sh start  <cols> <rows>          # launch app in tmux session
#   tools/capture.sh keys   <keys...>              # send tmux key names (e.g. Tab Down Enter "j")
#   tools/capture.sh mouse  <x> <y> [move|click|wheelup|wheeldown]   # send SGR mouse event
#   tools/capture.sh shot   <name>                 # capture to shots/<name>.{ansi,html,txt}
#   tools/capture.sh resize <cols> <rows>
#   tools/capture.sh stop
#
# Parameters, all environment variables, all with a default:
#   BIN    the binary to launch            (required; set by the caller)
#   ARGS   its arguments                   (none)
#   COLOR  the colour level to capture at  (truecolor)
#          truecolor | 256 | 16 | mono
#          `mono` runs the app under NO_COLOR=1; it deliberately leaves
#          COLORTERM=truecolor set, so a mono capture also proves NO_COLOR
#          outranks COLORTERM rather than hiding a regression behind an
#          unset variable. See COMPONENT_ARCHITECTURE.md §20.10 item 1.
#   THEME  theme name to record (default junie; inferred from ARGS --theme)
#   CAPTURE_DIR  artifact directory (shots)
#   CAPTURE_MANIFEST  provenance manifest (shots/capture-provenance.json)
#   PY     the python used for the PNG step (python3)
# Example: COLOR=mono tools/capture.sh start 120 40
set -euo pipefail
cd "$(dirname "$0")/.."
S=junie_cap
: "${BIN:?BIN must be set to the owning application binary}"
COLOR=${COLOR:-truecolor}
CAPTURE_DIR=${CAPTURE_DIR:-shots}
CAPTURE_MANIFEST=${CAPTURE_MANIFEST:-$CAPTURE_DIR/capture-provenance.json}
CAPTURE_STATE_DIR=${CAPTURE_STATE_DIR:-$CAPTURE_DIR/.capture-state}
STDERR_FILE=$CAPTURE_DIR/stderr.log
RUN_ID_FILE=$CAPTURE_STATE_DIR/run.id
RUN_EXIT_FILE=$CAPTURE_STATE_DIR/exit.status
RUN_APP_FILE=$CAPTURE_STATE_DIR/app
RUN_REVISION_FILE=$CAPTURE_STATE_DIR/revision
RUN_THEME_FILE=$CAPTURE_STATE_DIR/theme
RUN_COLOR_FILE=$CAPTURE_STATE_DIR/color
RUN_REQUESTED_COLS_FILE=$CAPTURE_STATE_DIR/requested.cols
RUN_REQUESTED_ROWS_FILE=$CAPTURE_STATE_DIR/requested.rows
cmd=${1:-}; shift || true

session_exists() {
  tmux has-session -t "$S" 2>/dev/null
}

require_session() {
  if ! session_exists; then
    echo "capture failed: tmux session '$S' is not running; run start first" >&2
    return 1
  fi
}

configure_session() {
  tmux set-option -t "$S" status off 2>/dev/null || return 1
  tmux set-option -s escape-time 0 2>/dev/null || return 1
  tmux set-option -g default-terminal "tmux-256color" 2>/dev/null || return 1
  tmux set-option -ga terminal-overrides ",*:Tc" 2>/dev/null || return 1
}

state_value() {
  local path=$1 fallback=$2
  if [[ -r "$path" ]]; then
    tr -d '\r\n' < "$path"
  else
    printf '%s\n' "$fallback"
  fi
}

read_exit_status() {
  if [[ -s "$RUN_EXIT_FILE" ]]; then
    state_value "$RUN_EXIT_FILE" unknown
  elif session_exists; then
    printf '%s\n' running
  else
    printf '%s\n' unknown
  fi
}

capture_theme() {
  if [[ -n "${THEME:-}" ]]; then
    printf '%s\n' "$THEME"
    return
  fi
  if [[ "${ARGS:-}" =~ (^|[[:space:]])--theme(=|[[:space:]])([^[:space:]]+) ]]; then
    printf '%s\n' "${BASH_REMATCH[3]}"
    return
  fi
  printf '%s\n' junie
}

record_provenance() {
  local name=$1 columns=$2 rows=$3 status=$4 exit_status=$5
  local ansi_path=$6 cursor_path=$7 text_path=$8 html_path=$9 png_path=${10}
  local requested_columns=${11} requested_rows=${12} run_id=${13} revision=${14}
  local app=${15} theme=${16} color=${17} stderr_path=${18}

  python3 - "$CAPTURE_MANIFEST" "$name" "$columns" "$rows" \
    "$status" "$exit_status" "$ansi_path" "$cursor_path" "$text_path" \
    "$html_path" "$png_path" "$requested_columns" "$requested_rows" \
    "$run_id" "$revision" "$app" "$theme" "$color" "$stderr_path" <<'PY'
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


(
    manifest_path,
    name,
    columns,
    rows,
    capture_status,
    exit_status,
    ansi_path,
    cursor_path,
    text_path,
    html_path,
    png_path,
    requested_columns,
    requested_rows,
    run_id,
    revision,
    app,
    theme,
    color,
    stderr_path,
) = sys.argv[1:]


def file_info(raw_path):
    path = Path(raw_path)
    info = {"path": raw_path}
    if not path.is_file():
        info.update({"bytes": None, "sha256": None, "status": "missing"})
        return info

    size = path.stat().st_size
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    info.update(
        {
            "bytes": size,
            "sha256": digest.hexdigest(),
            "status": "ok" if size else "empty",
        }
    )
    return info


def parsed_exit(raw):
    try:
        return int(raw)
    except ValueError:
        return raw or None


record = {
    "schema_version": 1,
    "captured_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    "name": name,
    "run_id": run_id,
    "app": app,
    "revision": revision,
    "theme": theme,
    "color": color,
    "status": capture_status,
    "exit_status": parsed_exit(exit_status),
    "exit_observed": exit_status.lstrip("-").isdigit(),
    "requested_dimensions": {
        "columns": int(requested_columns),
        "rows": int(requested_rows),
    },
    "dimensions": {"columns": int(columns), "rows": int(rows)},
    "stderr": file_info(stderr_path),
    "artifacts": {
        "ansi": file_info(ansi_path),
        "cursor": file_info(cursor_path),
        "txt": file_info(text_path),
        "html": file_info(html_path),
        "png": file_info(png_path),
    },
}

manifest = Path(manifest_path)
records = []
if manifest.exists():
    with manifest.open(encoding="utf-8") as stream:
        loaded = json.load(stream)
    if not isinstance(loaded, list):
        raise SystemExit(f"capture provenance is not a JSON array: {manifest}")
    records = loaded

# A name identifies the current evidence cell. Re-capturing it replaces stale
# provenance while leaving every other cell intact.
records = [item for item in records if item.get("name") != name]
records.append(record)

temporary = manifest.with_name(f".{manifest.name}.tmp")
with temporary.open("w", encoding="utf-8") as stream:
    json.dump(records, stream, indent=2, sort_keys=True)
    stream.write("\n")
    stream.flush()
    os.fsync(stream.fileno())
os.replace(temporary, manifest)
PY
}

finalize_provenance() {
  local exit_status=$1
  [[ -f "$CAPTURE_MANIFEST" && -s "$RUN_ID_FILE" ]] || return 0
  local run_id stderr_path
  run_id=$(state_value "$RUN_ID_FILE" unknown)
  stderr_path=$(state_value "$CAPTURE_STATE_DIR/stderr" "$STDERR_FILE")

  python3 - "$CAPTURE_MANIFEST" "$run_id" "$exit_status" "$stderr_path" <<'PY'
import hashlib
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


manifest_path, run_id, raw_exit, stderr_path = sys.argv[1:]
manifest = Path(manifest_path)


def file_info(raw_path):
    path = Path(raw_path)
    info = {"path": raw_path}
    if not path.is_file():
        info.update({"bytes": None, "sha256": None, "status": "missing"})
        return info
    size = path.stat().st_size
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    info.update(
        {
            "bytes": size,
            "sha256": digest.hexdigest(),
            "status": "ok" if size else "empty",
        }
    )
    return info


def parsed_exit(raw):
    try:
        return int(raw)
    except ValueError:
        return raw or None


with manifest.open(encoding="utf-8") as stream:
    records = json.load(stream)
if not isinstance(records, list):
    raise SystemExit(f"capture provenance is not a JSON array: {manifest}")

changed = False
for record in records:
    if record.get("run_id") != run_id or record.get("exit_status") != "running":
        continue
    exit_status = parsed_exit(raw_exit)
    record["exit_status"] = exit_status
    record["exit_observed"] = isinstance(exit_status, int)
    record["termination"] = (
        "natural_exit" if record["exit_observed"] else "capture_stop"
    )
    record["finalized_at"] = (
        datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    )
    record["stderr"] = file_info(stderr_path)
    changed = True

if changed:
    temporary = manifest.with_name(f".{manifest.name}.tmp")
    with temporary.open("w", encoding="utf-8") as stream:
        json.dump(records, stream, indent=2, sort_keys=True)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, manifest)
PY
}

case "$cmd" in
  start)
    cols=${1:-120}; rows=${2:-40}
    case "$COLOR" in
      truecolor) app_env="env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor" ;;
      256)       app_env="env -u NO_COLOR -u COLORTERM TERM=xterm-256color" ;;
      16)        app_env="env -u NO_COLOR -u COLORTERM TERM=xterm" ;;
      mono)      app_env="env NO_COLOR=1 TERM=xterm-256color COLORTERM=truecolor" ;;
      *) echo "unknown COLOR: $COLOR (truecolor|256|16|mono)" >&2; exit 1 ;;
    esac
    mkdir -p "$CAPTURE_DIR" "$CAPTURE_STATE_DIR" "$(dirname "$CAPTURE_MANIFEST")"
    run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
    revision="$(git rev-parse --verify HEAD 2>/dev/null || printf '%s' unknown)"
    app=${BIN##*/}
    theme=$(capture_theme)
    rm -f "$RUN_EXIT_FILE"
    printf '%s\n' "$run_id" > "$RUN_ID_FILE"
    printf '%s\n' "$app" > "$RUN_APP_FILE"
    printf '%s\n' "$revision" > "$RUN_REVISION_FILE"
    printf '%s\n' "$theme" > "$RUN_THEME_FILE"
    printf '%s\n' "$COLOR" > "$RUN_COLOR_FILE"
    printf '%s\n' "$cols" > "$RUN_REQUESTED_COLS_FILE"
    printf '%s\n' "$rows" > "$RUN_REQUESTED_ROWS_FILE"
    printf '%s\n' "$STDERR_FILE" > "$CAPTURE_STATE_DIR/stderr"
    printf -v stderr_arg '%q' "$STDERR_FILE"
    printf -v exit_arg '%q' "$RUN_EXIT_FILE"
    tmux kill-session -t "$S" 2>/dev/null || true
    tmux -f /dev/null new-session -d -s "$S" -x "$cols" -y "$rows" \
      "${app_env} ${BIN} ${ARGS:-} 2>${stderr_arg}; rc=\$?; printf '%s\\n' \"\$rc\" >${exit_arg}; exit \"\$rc\""
    if ! session_exists || ! configure_session; then
      exit_status=$(read_exit_status)
      echo "capture failed: tmux session ended during setup (app exit status $exit_status); stderr: $STDERR_FILE" >&2
      exit 1
    fi
    sleep 0.6
    if [[ -s "$RUN_EXIT_FILE" ]]; then
      exit_status=$(read_exit_status)
      echo "capture failed: $BIN exited before capture (status $exit_status); stderr: $STDERR_FILE" >&2
      exit 1
    fi
    if ! session_exists; then
      echo "capture failed: tmux session '$S' ended before capture; stderr: $STDERR_FILE" >&2
      exit 1
    fi
    ;;
  keys)
    require_session
    for k in "$@"; do tmux send-keys -t "$S" "$k"; sleep 0.08; done
    sleep 0.15
    ;;
  mouse)
    require_session
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
    esac
    tmux send-keys -t "$S" -l "$seq"
    sleep 0.15
    ;;
  shot)
    name=${1:-shot}
    require_session
    cols=$(tmux display -p -t "$S" '#{pane_width}'); rows=$(tmux display -p -t "$S" '#{pane_height}')
    base="$CAPTURE_DIR/$name"
    ansi_path="$base.ansi"
    cursor_path="$base.cursor"
    text_path="$base.txt"
    html_path="$base.html"
    png_path="$base.png"
    tmux capture-pane -t "$S" -e -p -N > "$ansi_path"
    tmux display -p -t "$S" "#{cursor_x} #{cursor_y} #{cursor_flag}" > "$cursor_path"
    tmux capture-pane -t "$S" -p > "$text_path"
    conversion_failed=0
    if ! python3 tools/ansi2html.py "$ansi_path" "$html_path" "$cols" "$rows"; then
      echo "capture failed: ansi2html could not convert $ansi_path" >&2
      conversion_failed=1
    fi
    if ! "${PY:-python3}" tools/ansi2png.py "$ansi_path" "$png_path" "$cols" "$rows" "$cursor_path"; then
      echo "capture failed: ansi2png could not convert $ansi_path" >&2
      conversion_failed=1
    fi

    exit_status=$(read_exit_status)
    app_failed=0
    if [[ "$exit_status" =~ ^-?[0-9]+$ ]] && (( exit_status != 0 )); then
      echo "capture failed: $BIN exited with status $exit_status; stderr: $STDERR_FILE" >&2
      app_failed=1
    fi
    if [[ -s "$STDERR_FILE" ]]; then
      echo "capture warning: application stderr is non-empty; inspect $STDERR_FILE" >&2
    fi

    artifact_failed=0
    for artifact in "$ansi_path" "$cursor_path" "$text_path" "$html_path" "$png_path"; do
      if [[ ! -s "$artifact" ]]; then
        echo "capture failed: artifact is missing or empty: $artifact" >&2
        artifact_failed=1
      fi
    done

    requested_columns=$(state_value "$RUN_REQUESTED_COLS_FILE" "$cols")
    requested_rows=$(state_value "$RUN_REQUESTED_ROWS_FILE" "$rows")
    run_id=$(state_value "$RUN_ID_FILE" unknown)
    app=$(state_value "$RUN_APP_FILE" "${BIN##*/}")
    revision=$(state_value "$RUN_REVISION_FILE" unknown)
    theme=$(state_value "$RUN_THEME_FILE" junie)
    color=$(state_value "$RUN_COLOR_FILE" "$COLOR")
    capture_status=ok
    if (( conversion_failed != 0 || app_failed != 0 || artifact_failed != 0 )); then
      capture_status=failed
    fi
    record_provenance "$name" "$cols" "$rows" "$capture_status" "$exit_status" \
      "$ansi_path" "$cursor_path" "$text_path" "$html_path" "$png_path" \
      "$requested_columns" "$requested_rows" "$run_id" "$revision" "$app" "$theme" \
      "$color" "$STDERR_FILE"
    if [[ "$capture_status" == failed ]]; then
      exit 1
    fi
    echo "$html_path ($cols x $rows; app=$app theme=$theme color=$color exit=$exit_status; provenance=$CAPTURE_MANIFEST)"
    ;;
  resize)
    require_session
    tmux resize-window -t "$S" -x "$1" -y "$2"; sleep 0.3
    ;;
  stop)
    final_exit_status=$(read_exit_status)
    tmux kill-session -t "$S" 2>/dev/null || true
    if [[ "$final_exit_status" == running || "$final_exit_status" == unknown ]]; then
      final_exit_status=terminated_by_capture_stop
    fi
    finalize_provenance "$final_exit_status"
    ;;
  *) echo "unknown: $cmd" >&2; exit 1 ;;
esac
