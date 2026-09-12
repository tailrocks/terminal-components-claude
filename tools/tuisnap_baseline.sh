#!/usr/bin/env bash
# Capture the tuisnap snapshot baseline: every capturable surface of the four
# binaries, gated into the store at shots/tuisnap/. The matrix and its
# rationale live in docs/baseline/tuisnap-coverage.md; this script is the
# executable source of truth for the 371 capture names.
#
# Usage:
#   tools/tuisnap_baseline.sh            # build + run the whole matrix
#   tools/tuisnap_baseline.sh summary    # reprint the last run + store status
# Env knobs (subset re-runs):
#   SKIP_BUILD=1                         # reuse target/debug binaries
#   APPS="showcase holla"                # default: all four
#   SIZES="120x40"  COLORS="truecolor"   # space-separated allow-lists
#   ONLY='holla_(rust-dirty|upgrade-plan)'   # grep -E filter on capture name
#   SETTLE_MS=400 TIMEOUT_MS=8000 STORE=shots/tuisnap
#
# tuisnap run is fail-closed: on the first run every capture ends as
# missing-approval (expected — logged as "pending", not failed). Approve after
# review with: tuisnap accept --store shots/tuisnap --all
# then re-verify with: tuisnap report --store shots/tuisnap
set -euo pipefail
cd "$(dirname "$0")/.."

STORE=${STORE:-shots/tuisnap}
FRAMES=$STORE/frames
RUNDIR=$STORE/.baseline-run
SETTLE_MS=${SETTLE_MS:-400}
TIMEOUT_MS=${TIMEOUT_MS:-8000}
APPS=${APPS:-"showcase holla tablepro jackin"}
SIZES=${SIZES:-}
COLORS=${COLORS:-}
ONLY=${ONLY:-}
# Determinism: fixture scenarios plus paused motion make holla/jackin frames
# exact (seeded sim, no wall-clock randomness). History side effects off.
export HOLLA_NO_HISTORY=1
# Residual nondeterminism: tick-driven spinners (showcase progress/taskrunner
# busy rows, tablepro connect ticks) are captured after wait_stable and land
# on a stable phase, but a spinner glyph cannot be pinned — if exactly one of
# those flakes at re-verify time, re-run once before suspecting a regression.

if [ "${1:-}" = summary ]; then
  if [ -f "$STORE/last-run.summary" ]; then
    cat "$STORE/last-run.summary"
  else
    echo "no baseline run recorded yet ($STORE/last-run.summary missing)"
  fi
  echo
  tuisnap report --store "$STORE" || true
  exit 0
fi

if [ -z "${SKIP_BUILD:-}" ]; then
  cargo build --bins
fi
BIN_DIR=$PWD/target/debug
for b in showcase tablepro jackin-preview holla; do
  [ -x "$BIN_DIR/$b" ] || { echo "missing binary: $BIN_DIR/$b" >&2; exit 1; }
done
mkdir -p "$FRAMES" "$RUNDIR/logs"
: > "$RUNDIR/status.tsv"
: > "$RUNDIR/failed.txt"

MATCHED=0; PENDING=0; DRIFT=0; FAILED=0; RENDER_FAILED=0; SKIPPED=0

# cap NAME COLS ROWS COLOR BOOT_NEEDLE SENDS -- ARGS...
# COLOR: truecolor|256|16|none (passed as --color) or nocolor (env NO_COLOR=1).
# BOOT_NEEDLE: string on the fully-rendered first screen (empty to skip); sent
# as the first step because --wait-for would run AFTER the sends. SENDS is a
# ;-separated list of further steps (key names, type:<text>, sleep:<ms>,
# wait:<needle>).
cap() {
  local name=$1 cols=$2 rows=$3 color=$4 needle=$5 sends=$6; shift 6
  [ "${1:-}" = -- ] && shift
  local size=${cols}x${rows}
  if [ -n "$SIZES" ] && ! printf '%s\n' $SIZES | grep -qx "$size"; then SKIPPED=$((SKIPPED+1)); return; fi
  if [ -n "$COLORS" ] && ! printf '%s\n' $COLORS | grep -qx "$color"; then SKIPPED=$((SKIPPED+1)); return; fi
  if [ -n "$ONLY" ] && ! printf '%s' "$name" | grep -qE "$ONLY"; then SKIPPED=$((SKIPPED+1)); return; fi

  local -a argv
  if [ "$color" = nocolor ]; then
    argv=(env NO_COLOR=1 "$@")
  else
    argv=("$@" --color "$color")
  fi
  local -a steps=()
  [ -n "$needle" ] && steps+=(--send "wait:$needle")
  if [ -n "$sends" ]; then
    local step parts
    IFS=';' read -ra parts <<< "$sends"
    for step in "${parts[@]}"; do steps+=(--send "$step"); done
  fi

  local log="$RUNDIR/logs/$name.log" status
  if tuisnap run --cols "$cols" --rows "$rows" \
      --settle-ms "$SETTLE_MS" --timeout-ms "$TIMEOUT_MS" \
      "${steps[@]}" \
      --store "$STORE" --name "$name" \
      -- "${argv[@]}" >"$log" 2>&1; then
    status=matched; MATCHED=$((MATCHED+1))
  elif grep -q "missing-approval" "$log"; then
    status=pending; PENDING=$((PENDING+1))
  elif grep -qE "cells-differ|pixels-differ" "$log"; then
    status=DRIFT; DRIFT=$((DRIFT+1)); echo "$name" >> "$RUNDIR/failed.txt"
  else
    status=FAILED; FAILED=$((FAILED+1)); echo "$name" >> "$RUNDIR/failed.txt"
  fi
  printf '%s\t%s\n' "$status" "$name" >> "$RUNDIR/status.tsv"

  # Loose review artifacts, re-rendered offline from the canonical frame
  # (store mode ignores --out, so this is a separate step by design).
  if [ -f "$STORE/actual/$name.frame.json" ]; then
    if ! tuisnap render --input "$STORE/actual/$name.frame.json" \
        --format ansi --format txt --format png --format html \
        --out "$FRAMES/$name" >>"$log" 2>&1; then
      RENDER_FAILED=$((RENDER_FAILED+1)); echo "$name (render)" >> "$RUNDIR/failed.txt"
    fi
  fi
  printf '%-8s %s\n' "$status" "$name"
}

app_in() { printf '%s\n' $APPS | grep -qx "$1"; }

# ---------------------------------------------------------------- showcase --
showcase_app() {
  local bin=$BIN_DIR/showcase
  # name-slug:argv-slug — PageId::from_name needs the full normalized label,
  # so "Editable tables"=editabletables and "Chips & selects"=chipsselects.
  local pages="overview:overview buttons:buttons inputs:inputs textareas:textareas \
forms:forms lists:lists trees:trees tables:tables editable:editabletables \
panels:panels sidebars:sidebars dialogs:dialogs progress:progress \
scrolling:scrolling terminal:terminal codeeditor:codeeditor diff:diff \
datagrid:datagrid chips:chipsselects pickers:pickers chrome:chrome \
settings:settings taskrunner:taskrunner"
  local p pair name slug size color
  for pair in $pages; do
    name=${pair%%:*}; slug=${pair#*:}
    for size in 80x24 120x40; do
      for color in truecolor none; do
        cap "showcase_${name}_default_${size}_${color}" "${size%x*}" "${size#*x}" "$color" \
          "Junie Design system" "" -- "$bin" --page "$slug"
      done
    done
  done
  # palette spots on the most palette-sensitive pages
  for p in buttons forms datagrid diff; do
    for color in 256 16; do
      cap "showcase_${p}_default_120x40_${color}" 120 40 "$color" "Junie Design system" "" -- "$bin" --page "$p"
    done
  done
  # real NO_COLOR=1 backend suppression
  for p in overview inputs; do
    cap "showcase_${p}_default_120x40_nocolor" 120 40 nocolor "Junie Design system" "" -- "$bin" --page "$p"
  done
  # minimum-size spots
  for p in overview buttons; do
    cap "showcase_${p}_default_72x20_truecolor" 72 20 truecolor "Junie Design system" "" -- "$bin" --page "$p"
  done
  # interactive states at 120x40 truecolor (sequences per the coverage doc)
  local boot="Junie Design system"
  cap showcase_inputs_editing_120x40_truecolor 120 40 truecolor "$boot" "tab;enter;wait:EDIT" -- "$bin" --page inputs
  cap showcase_inputs_selected_120x40_truecolor 120 40 truecolor "$boot" "tab;enter;ctrl-l;wait:EDIT" -- "$bin" --page inputs
  cap showcase_forms_invalid_120x40_truecolor 120 40 truecolor "$boot" "tab;ctrl-s;wait:Required" -- "$bin" --page forms
  cap showcase_diff_review_120x40_truecolor 120 40 truecolor "$boot" "tab;enter;wait:● Review" -- "$bin" --page diff
  cap showcase_diff_empty_120x40_truecolor 120 40 truecolor "$boot" "tab;enter;backtab;enter;wait:No file selected" -- "$bin" --page diff
  cap showcase_buttons_focus_120x40_truecolor 120 40 truecolor "$boot" "tab" -- "$bin" --page buttons
  cap showcase_lists_moved_120x40_truecolor 120 40 truecolor "$boot" "tab;down;down" -- "$bin" --page lists
  cap showcase_trees_expanded_120x40_truecolor 120 40 truecolor "$boot" "tab;right" -- "$bin" --page trees
  cap showcase_tables_selected_120x40_truecolor 120 40 truecolor "$boot" "tab;down;down" -- "$bin" --page tables
  cap showcase_editable_editing_120x40_truecolor 120 40 truecolor "$boot" "tab;enter" -- "$bin" --page editabletables
  cap showcase_datagrid_selected_120x40_truecolor 120 40 truecolor "$boot" "tab;down;right" -- "$bin" --page datagrid
  cap showcase_dialogs_open_120x40_truecolor 120 40 truecolor "$boot" "tab;enter" -- "$bin" --page dialogs
  cap showcase_pickers_open_120x40_truecolor 120 40 truecolor "$boot" "tab;enter" -- "$bin" --page pickers
  cap showcase_chips_toggled_120x40_truecolor 120 40 truecolor "$boot" "tab;space" -- "$bin" --page chipsselects
  cap showcase_scrolling_scrolled_120x40_truecolor 120 40 truecolor "$boot" "tab;down;down;down" -- "$bin" --page scrolling
  cap showcase_settings_toggled_120x40_truecolor 120 40 truecolor "$boot" "tab;space" -- "$bin" --page settings
  cap showcase_help_overlay_120x40_truecolor 120 40 truecolor "$boot" "?" -- "$bin" --page overview
  cap showcase_inspector_open_120x40_truecolor 120 40 truecolor "$boot" "i" -- "$bin" --page overview
}

# ------------------------------------------------------------------- holla --
HOLLA_SCENARIOS="first-use rust-dirty monorepo-root monorepo-child docker-cleanup \
disk-cleanup upgrade-plan activities-multi remote-host launch-failure hard-cases \
parity-discovery parity-history parity-files parity-browser parity-git-current \
parity-git-batch parity-task-sources parity-cargo parity-docker \
parity-brew-services parity-gradle parity-idea parity-upgrade-managers \
parity-executor parity-task-input parity-custom-actions parity-disk-scan \
parity-disk-navigation parity-insights parity-delete-safety \
parity-cleanup-results parity-platforms parity-platforms-linux"

holla_app() {
  local bin=$BIN_DIR/holla s size
  # core matrix: paused at frame 40, four sizes, truecolor
  for s in $HOLLA_SCENARIOS; do
    for size in 80x24 100x30 120x40 160x50; do
      cap "holla_${s}_default_${size}_truecolor" "${size%x*}" "${size#*x}" truecolor \
        "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
    done
    # mono at 100x30: lose nothing but hue
    cap "holla_${s}_default_100x30_none" 100 30 none \
      "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
  done
  # 16 colours: the plan outline and the disk page (tinted rows)
  for s in upgrade-plan disk-cleanup; do
    cap "holla_${s}_default_100x30_16" 100 30 16 \
      "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 80
  done
  # 256-colour + real NO_COLOR gap closure; minimum-size boundary
  for s in rust-dirty upgrade-plan disk-cleanup hard-cases; do
    cap "holla_${s}_default_100x30_256" 100 30 256 \
      "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
  done
  for s in rust-dirty upgrade-plan; do
    cap "holla_${s}_default_100x30_nocolor" 100 30 nocolor \
      "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
  done
  for s in first-use hard-cases; do
    cap "holla_${s}_default_72x20_truecolor" 72 20 truecolor \
      "holla❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
  done
  # journeys at 120x40 truecolor; motion reduced where the legacy flow
  # scripts needed streamed results (finder/files/browser), paused elsewhere
  cap holla_finder_query_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:pull" -- "$bin" --scenario parity-history --motion reduced
  cap holla_finder_query-selected_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:pull;ctrl-a" -- "$bin" --scenario parity-history --motion reduced
  cap holla_trust_prompt_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:test;enter" -- "$bin" --scenario monorepo-child --motion reduced
  cap holla_files_results_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:Find files under home;enter;type:readme" -- "$bin" --scenario parity-files --motion reduced
  cap holla_files_unicode_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:Find files under home;enter;ctrl-u;type:café" -- "$bin" --scenario parity-files --motion reduced
  cap holla_browser_hidden_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:Browse ~/work/site;enter;ctrl-h" -- "$bin" --scenario parity-browser --motion reduced
  cap holla_browser_preview_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:Browse ~/work/site;enter;down;down;down" -- "$bin" --scenario parity-browser --motion reduced
  cap holla_cleanup_plan_120x40_truecolor 120 40 truecolor "holla❯" \
    "c" -- "$bin" --scenario disk-cleanup --motion paused --frame 80
  cap holla_cleanup_gate-1_120x40_truecolor 120 40 truecolor "holla❯" \
    "c;c" -- "$bin" --scenario disk-cleanup --motion paused --frame 80
  cap holla_upgrade_excluded_120x40_truecolor 120 40 truecolor "holla❯" \
    "down;down;down;down;down;space" -- "$bin" --scenario upgrade-plan --motion reduced
  cap holla_upgrade_confirm_120x40_truecolor 120 40 truecolor "holla❯" \
    "down;down;down;down;down;space;c" -- "$bin" --scenario upgrade-plan --motion reduced
  cap holla_remote_gate-1_120x40_truecolor 120 40 truecolor "holla❯" \
    "type:restart payments;enter" -- "$bin" --scenario remote-host --motion reduced
  cap holla_help_overlay_120x40_truecolor 120 40 truecolor "holla❯" \
    "f1" -- "$bin" --scenario rust-dirty --motion paused --frame 40
  cap holla_activities_overlay_120x40_truecolor 120 40 truecolor "holla❯" \
    "ctrl-g" -- "$bin" --scenario activities-multi --motion reduced
}

# ---------------------------------------------------------------- tablepro --
tablepro_app() {
  local bin=$BIN_DIR/tablepro
  cap tablepro_connections_default_80x24_truecolor 80 24 truecolor "Production" "" -- "$bin"
  cap tablepro_connections_default_120x40_truecolor 120 40 truecolor "Production" "" -- "$bin"
  cap tablepro_connections_default_120x40_none 120 40 none "Production" "" -- "$bin"
  # workbench; 80x24 exercises the <100-col explorer drawer mode
  cap tablepro_workbench_default_80x24_truecolor 80 24 truecolor "Query 1" "" -- "$bin" --connect Production
  cap tablepro_workbench_default_120x40_truecolor 120 40 truecolor "Query 1" "" -- "$bin" --connect Production
  cap tablepro_workbench_default_120x40_256 120 40 256 "Query 1" "" -- "$bin" --connect Production
  cap tablepro_workbench_default_120x40_16 120 40 16 "Query 1" "" -- "$bin" --connect Production
  cap tablepro_workbench_default_120x40_none 120 40 none "Query 1" "" -- "$bin" --connect Production
  cap tablepro_workbench_default_120x40_nocolor 120 40 nocolor "Query 1" "" -- "$bin" --connect Production
  # interactive at 120x40 truecolor; "open orders" = down x5, enter from the
  # fresh workbench (cursor starts on the schema row; orders is the 4th table)
  local wb="Query 1" open="down;down;down;down;down;enter;wait:public › orders"
  cap tablepro_form_new_120x40_truecolor 120 40 truecolor "Production" \
    "ctrl-n" -- "$bin"
  cap tablepro_form_new-filled_120x40_truecolor 120 40 truecolor "Production" \
    "ctrl-n;type:Staging replica" -- "$bin"
  cap tablepro_table_data_120x40_truecolor 120 40 truecolor "$wb" \
    "$open" -- "$bin" --connect Production
  cap tablepro_table_structure_120x40_truecolor 120 40 truecolor "$wb" \
    "$open;ctrl-d;wait:Columns" -- "$bin" --connect Production
  cap tablepro_table_sorted_120x40_truecolor 120 40 truecolor "$wb" \
    "$open;right;right;right;right;right;right;right;right;right;right;right;right;s;wait:sort created_at ▴" -- "$bin" --connect Production
  cap tablepro_table_filtered_120x40_truecolor 120 40 truecolor "$wb" \
    "$open;home;right;right;right;right;f;backtab;backtab;enter;ctrl-l;type:pending;enter;wait:filtered (1)" -- "$bin" --connect Production
  cap tablepro_cell_editing_120x40_truecolor 120 40 truecolor "$wb" \
    "$open;home;right;right;right;right;enter" -- "$bin" --connect Production
  cap tablepro_row_duplicated_120x40_truecolor 120 40 truecolor "$wb" \
    "$open;alt-d" -- "$bin" --connect Production
  cap tablepro_query_completion_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:SELECT * FROM ord;wait:order_items" -- "$bin" --connect Production
  cap tablepro_query_results_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:SELECT * FROM ord;enter;type: WHERE st;tab;type: = 'pending' ORDER BY created_at DESC LIMIT 25;escape;ctrl-r;wait:25 rows" -- "$bin" --connect Production
  cap tablepro_query_error_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:SELECT * FROM missing_table;escape;ctrl-r;sleep:800" -- "$bin" --connect Production
  cap tablepro_query_explain_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:SELECT * FROM orders;escape;ctrl-x;sleep:500" -- "$bin" --connect Production
  cap tablepro_ack_gate_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:UPDATE orders SET status = 'paid' WHERE id = 'x';escape;ctrl-r;wait:Type orders to confirm" -- "$bin" --connect Production
  cap tablepro_ack_armed_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:UPDATE orders SET status = 'paid' WHERE id = 'x';escape;ctrl-r;wait:Type orders to confirm;enter;type:orders;enter;right;wait:Execute" -- "$bin" --connect Production
  cap tablepro_ack_executed_120x40_truecolor 120 40 truecolor "$wb" \
    "tab;i;type:UPDATE orders SET status = 'paid' WHERE id = 'x';escape;ctrl-r;wait:Type orders to confirm;enter;type:orders;enter;right;enter;wait:rows affected" -- "$bin" --connect Production
  cap tablepro_history_tab_120x40_truecolor 120 40 truecolor "$wb" \
    "ctrl-y;wait:History" -- "$bin" --connect Production
  cap tablepro_picker_open_120x40_truecolor 120 40 truecolor "$wb" \
    "ctrl-o;wait:Open Quickly" -- "$bin" --connect Production
  cap tablepro_tablist_open_120x40_truecolor 120 40 truecolor "$wb" \
    "ctrl-g" -- "$bin" --connect Production
  cap tablepro_safemode_picker_120x40_truecolor 120 40 truecolor "$wb" \
    "ctrl-l;wait:Safe Mode" -- "$bin" --connect Production
  cap tablepro_help_overlay_120x40_truecolor 120 40 truecolor "$wb" \
    "?;wait:Keyboard" -- "$bin" --connect Production
}

# ------------------------------------------------------------------ jackin --
JACKIN_SCENARIOS="first-use returning accounts-mixed launch-running launch-failure capsule-multi outro-last hard-cases"

jackin_app() {
  local bin=$BIN_DIR/jackin-preview s size
  for s in $JACKIN_SCENARIOS; do
    for size in 80x24 120x40; do
      cap "jackin_${s}_default_${size}_truecolor" "${size%x*}" "${size#*x}" truecolor \
        "jackin❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
    done
  done
  # first-use intro phases (INTRO_END = tick 308): warp + post-intro manager
  cap jackin_first-use_f300_120x40_truecolor 120 40 truecolor \
    "jackin❯" "" -- "$bin" --scenario first-use --motion paused --frame 300
  cap jackin_first-use_f400_120x40_truecolor 120 40 truecolor \
    "jackin❯" "" -- "$bin" --scenario first-use --motion paused --frame 400
  for s in first-use capsule-multi accounts-mixed hard-cases; do
    cap "jackin_${s}_default_120x40_none" 120 40 none \
      "jackin❯" "" -- "$bin" --scenario "$s" --motion paused --frame 40
  done
  cap jackin_capsule-multi_default_120x40_256 120 40 256 \
    "jackin❯" "" -- "$bin" --scenario capsule-multi --motion paused --frame 40
  cap jackin_accounts-mixed_default_120x40_16 120 40 16 \
    "jackin❯" "" -- "$bin" --scenario accounts-mixed --motion paused --frame 40
  cap jackin_first-use_default_120x40_nocolor 120 40 nocolor \
    "jackin❯" "" -- "$bin" --scenario first-use --motion paused --frame 40
  cap jackin_first-use_default_72x20_truecolor 72 20 truecolor \
    "jackin❯" "" -- "$bin" --scenario first-use --motion paused --frame 40
}

# ------------------------------------------------------------------- driver --
app_in showcase && showcase_app
app_in holla && holla_app
app_in tablepro && tablepro_app
app_in jackin && jackin_app

TOTAL=$((MATCHED+PENDING+DRIFT+FAILED))
{
  echo "tuisnap baseline run — $(date '+%Y-%m-%d %H:%M:%S')"
  echo "store:   $STORE (frames: $FRAMES, logs: $RUNDIR/logs)"
  echo "total:   $TOTAL captures ($SKIPPED skipped by filters)"
  echo "matched: $MATCHED (gated clean against approved)"
  echo "pending: $PENDING (missing approval — expected on first run; review report.html, then: tuisnap accept --store $STORE --all)"
  echo "drift:   $DRIFT (cells/pixels differ from approved — inspect diff/, fix or re-accept)"
  echo "failed:  $FAILED (spawn/timeout/tool errors — NOT expected)"
  echo "render failures: $RENDER_FAILED"
  if [ -s "$RUNDIR/failed.txt" ]; then
    echo "attention needed:"
    sed 's/^/  - /' "$RUNDIR/failed.txt"
  fi
} | tee "$STORE/last-run.summary"

[ "$FAILED" -eq 0 ] && [ "$RENDER_FAILED" -eq 0 ]
