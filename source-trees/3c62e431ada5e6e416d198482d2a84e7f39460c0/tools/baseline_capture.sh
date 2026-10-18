#!/usr/bin/env bash
# Before-refactor visual evidence harness (REFACTORING_GOAL.md §6).
# Wraps the tmux flow of tools/capture.sh but writes every artifact under
# $OUT (default baseline/before) and records each capture's exact recipe in
# $OUT/manifest.tsv, from which `manifest` renders $OUT/MANIFEST.md.
#
#   tools/baseline_capture.sh all            # every plan below, then the manifest
#   tools/baseline_capture.sh showcase|tablepro|jackin
#   tools/baseline_capture.sh manifest       # regenerate MANIFEST.md from manifest.tsv
#   tools/baseline_capture.sh stop
#
# Primitives (also usable interactively for one-off captures):
#   start <bin> <cols> <rows> [args...]   keys <tmux-key>...   type <text>
#   mouse <x> <y> [move|click|rclick|down|up|drag|wheelup|wheeldown] (1-based)
#   find <needle>  -> "x y" (1-based, for mouse)   shot <name>   wait <secs>   resize <cols> <rows>
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
OUT=${OUT:-baseline/before}
S=junie_base
# shellcheck disable=SC1091
source tools/env.sh
mkdir -p "$OUT/stderr"
TSV="$OUT/manifest.tsv"
STEPS=""; CUR_BIN=""; CUR_ARGS=""; CUR_ERR=""; SESSION_N=0

step() { STEPS="${STEPS:+$STEPS · }$1"; }

start() { # bin cols rows [args...]
  local bin=$1 cols=$2 rows=$3; shift 3
  CUR_BIN=$bin; CUR_ARGS="$*"; STEPS=""
  SESSION_N=$((SESSION_N + 1))
  CUR_ERR="$OUT/stderr/$(printf '%03d' "$SESSION_N")_${bin}.log"
  tmux kill-session -t $S 2>/dev/null || true
  tmux -f /dev/null new-session -d -s $S -x "$cols" -y "$rows" \
    "env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor target/debug/${bin} $* 2>$CUR_ERR; sleep 30"
  tmux set-option -t $S status off
  tmux set-option -s escape-time 0
  tmux set-option -g default-terminal "tmux-256color"
  tmux set-option -ga terminal-overrides ",*:Tc"
  sleep 0.7
}

keys() { for k in "$@"; do tmux send-keys -t $S "$k"; sleep 0.08; done; sleep 0.15; step "keys($*)"; }
type_text() { tmux send-keys -t $S -l -- "$1"; sleep 0.2; step "type(\"$1\")"; }
wait_for() { sleep "$1"; step "wait(${1}s)"; }
resize() { tmux resize-window -t $S -x "$1" -y "$2"; sleep 0.4; step "resize(${1}x$2)"; }

mouse() { # x y kind (1-based SGR coordinates)
  local x=$1 y=$2 kind=${3:-move} seq
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
  tmux send-keys -t $S -l "$seq"; sleep 0.2; step "mouse($kind $x,$y)"
}

# Print "x y" (1-based) of the first cell where `needle` starts, or fail.
find_text() {
  tmux capture-pane -t $S -p | python3 -c '
import sys
needle = sys.argv[1]
for y, line in enumerate(sys.stdin.read().split("\n"), start=1):
    i = line.find(needle)
    if i >= 0:
        print(i + 1, y); sys.exit(0)
sys.exit(1)' "$1"
}
# mouse action on a needle, with a column offset
mouse_on() { # needle kind [dx]
  local pos; pos=$(find_text "$1") || { echo "  !! not on screen: $1" >&2; return 1; }
  local x=${pos% *} y=${pos#* }
  mouse $((x + ${3:-0})) "$y" "$2"
  step "(on \"$1\")"
}

shot() {
  local name=$1 cols rows
  cols=$(tmux display -p -t $S '#{pane_width}'); rows=$(tmux display -p -t $S '#{pane_height}')
  tmux capture-pane -t $S -e -p -N > "$OUT/$name.ansi"
  tmux display -p -t $S "#{cursor_x} #{cursor_y} #{cursor_flag}" > "$OUT/$name.cursor"
  tmux capture-pane -t $S -p > "$OUT/$name.txt"
  python3 tools/ansi2html.py "$OUT/$name.ansi" "$OUT/$name.html" "$cols" "$rows"
  "${PY:-python3}" tools/ansi2png.py "$OUT/$name.ansi" "$OUT/$name.png" "$cols" "$rows" "$OUT/$name.cursor" 2>>"$OUT/stderr/png.log"
  printf '%s\t%sx%s\t%s %s\t%s\t%s\n' "$name" "$cols" "$rows" "$CUR_BIN" "$CUR_ARGS" "${STEPS:-(none)}" "$(basename "$CUR_ERR")" >> "$TSV"
  echo "$OUT/$name ($cols x $rows)"
}

stop() { tmux kill-session -t $S 2>/dev/null || true; }

# ---------------------------------------------------------------- showcase
PAGES=(overview buttons inputs textareas forms lists trees tables editabletables panels sidebars dialogs progress scrolling terminal codeeditor datagrid chipsselects pickers chrome settings taskrunner)

plan_showcase() {
  for size in 80x24 100x30 120x40 160x50; do
    for p in "${PAGES[@]}"; do
      start showcase "${size%x*}" "${size#*x}" --page "$p"
      shot "showcase_${p}_default_${size}"
    done
  done
  for lvl in 256 16 none; do
    for p in overview buttons inputs tables; do
      start showcase 120 40 --page "$p" --color "$lvl"
      shot "showcase_${p}_default_120x40_${lvl}"
    done
  done
  # --- representative states at 120x40
  start showcase 120 40 --page buttons
  keys Tab; shot showcase_buttons_focused_120x40
  keys Tab; shot showcase_buttons_focused2_120x40
  mouse_on "Run task" move 2; shot showcase_buttons_hovered_120x40
  mouse_on "Run task" down 2; shot showcase_buttons_pressed_120x40
  mouse_on "Run task" up 2; shot showcase_buttons_activated_120x40
  mouse_on "Disabled primary" move 2; shot showcase_buttons_disabled_hover_120x40
  mouse_on "Disabled primary" click 2; shot showcase_buttons_disabled_click_120x40
  keys "?"; shot showcase_buttons_help_120x40
  keys Escape; keys i; shot showcase_buttons_inspector_120x40
  keys i Escape; shot showcase_buttons_navfocus_120x40

  start showcase 120 40 --page inputs
  keys Tab Enter; shot showcase_inputs_editing_120x40
  keys End; type_text "-v2"; shot showcase_inputs_editing_typed_120x40
  keys Enter; shot showcase_inputs_committed_120x40
  keys Tab Tab; shot showcase_inputs_focused3_120x40
  keys i; shot showcase_inputs_inspector_120x40

  start showcase 120 40 --page forms
  keys C-s; shot showcase_forms_error_120x40
  keys Enter; type_text "Fix login bug"; shot showcase_forms_editing_120x40
  keys Enter C-s; shot showcase_forms_submitted_120x40

  start showcase 120 40 --page lists
  keys Tab Down Down; shot showcase_lists_selected_120x40
  mouse_on "TypeScript" move; shot showcase_lists_hovered_120x40
  keys Tab Space; shot showcase_lists_multi_toggle_120x40
  keys a; shot showcase_lists_multi_all_120x40
  keys Tab Tab; shot showcase_lists_focus_walk_120x40

  start showcase 120 40 --page tables
  keys Tab; shot showcase_tables_focused_row_120x40
  keys Down Down; shot showcase_tables_selected_row_120x40
  keys s; shot showcase_tables_sorted_asc_120x40
  keys s; shot showcase_tables_sorted_desc_scrolled_120x40
  mouse_on "#1042" move 20; shot showcase_tables_hovered_row_120x40
  mouse_on "Owner" click; shot showcase_tables_header_click_sort_120x40
  keys Tab; shot showcase_tables_empty_focus_120x40

  start showcase 120 40 --page editabletables
  keys Tab Enter; shot showcase_editabletables_editing_120x40
  keys End; type_text " now"; shot showcase_editabletables_editing_typed_120x40
  keys Enter; shot showcase_editabletables_committed_120x40
  keys Right Right Right Right Enter C-l; type_text "abc"; keys Enter; shot showcase_editabletables_validation_error_120x40
  keys Escape

  start showcase 120 40 --page textareas
  mouse_on " 1. Read" wheeldown 0; shot showcase_textareas_wheel_scrolled_120x40
  keys Tab Enter; shot showcase_textareas_editing_120x40
  for _ in $(seq 30); do tmux send-keys -t $S Down; sleep 0.03; done; sleep 0.2; step "keys(Down x30)"
  shot showcase_textareas_overflow_end_120x40

  start showcase 120 40 --page trees
  keys Tab; shot showcase_trees_focused_120x40
  keys Left; shot showcase_trees_collapsed_120x40
  keys Right Down Right Down; shot showcase_trees_expanded_depth2_120x40

  start showcase 120 40 --page scrolling
  keys Tab; shot showcase_scrolling_focused_120x40
  keys End; shot showcase_scrolling_overflow_end_120x40
  keys Home; mouse_on "Row 001" wheeldown 4; mouse_on "Row 0" wheeldown 4; shot showcase_scrolling_wheel_120x40

  start showcase 120 40 --page dialogs
  keys Tab Enter; shot showcase_dialogs_confirm_open_120x40
  keys Tab; shot showcase_dialogs_confirm_focus2_120x40
  keys Escape; shot showcase_dialogs_cancelled_120x40
  keys Tab Enter; shot showcase_dialogs_prompt_open_120x40
  keys Enter C-l BSpace Enter; shot showcase_dialogs_prompt_error_120x40
  keys Enter; type_text "Ship it"; shot showcase_dialogs_prompt_editing_120x40
  keys Enter; shot showcase_dialogs_prompt_done_120x40
  keys Tab Enter; shot showcase_dialogs_third_open_120x40
  keys Escape

  start showcase 120 40 --page settings
  keys Tab Right; shot showcase_settings_members_tab_120x40
  keys Tab Down Tab Tab Enter; shot showcase_settings_remove_dialog_120x40
  keys Escape; keys Tab Right Right; shot showcase_settings_environment_tab_120x40
  keys Tab; shot showcase_settings_environment_focused_120x40

  start showcase 120 40 --page taskrunner
  keys r; wait_for 1.2; shot showcase_taskrunner_busy_120x40
  keys Tab Tab Enter; shot showcase_taskrunner_cancel_dialog_120x40
  keys y; shot showcase_taskrunner_cancelled_120x40

  start showcase 120 40 --page progress
  wait_for 1.0; shot showcase_progress_ticked_120x40
  keys Tab; shot showcase_progress_focused_120x40

  start showcase 120 40 --page chipsselects
  keys Tab; shot showcase_chipsselects_focused_120x40
  keys Right Right; shot showcase_chipsselects_chip_moved_120x40
  keys x; shot showcase_chipsselects_chip_removed_120x40
  mouse_on "created_at" move; shot showcase_chipsselects_select_hovered_120x40
  mouse_on "created_at" click; shot showcase_chipsselects_select_open_120x40
  keys Down Down; shot showcase_chipsselects_select_moved_120x40
  keys Enter; shot showcase_chipsselects_select_chosen_120x40
  mouse_on "PostgreSQL" click; shot showcase_chipsselects_select_disabled_click_120x40
  mouse_on "match all" click; shot showcase_chipsselects_mode_select_open_120x40
  keys Escape

  start showcase 120 40 --page pickers
  keys Tab Enter; shot showcase_pickers_quick_open_120x40
  type_text "ord"; shot showcase_pickers_quick_filtered_120x40
  keys Tab; shot showcase_pickers_quick_scope_120x40
  keys Escape Escape Escape; mouse_on "Switch tab" click 2; shot showcase_pickers_tabs_open_120x40
  keys Down; shot showcase_pickers_tabs_moved_120x40
  keys Escape; mouse_on "Choose a level" click 2; shot showcase_pickers_level_open_120x40
  keys Down; shot showcase_pickers_level_moved_120x40
  keys Enter; shot showcase_pickers_level_chosen_120x40

  start showcase 120 40 --page chrome
  keys Tab; shot showcase_chrome_bar_focused_120x40
  keys Enter; shot showcase_chrome_menu_open_120x40
  keys Right; shot showcase_chrome_menu_second_120x40
  keys Down Down; shot showcase_chrome_menu_item_moved_120x40
  keys Escape; keys Tab; shot showcase_chrome_sessions_focused_120x40
  keys Down; keys m; shot showcase_chrome_context_menu_120x40
  keys Escape; keys F10; shot showcase_chrome_f10_menu_120x40
  keys Escape; mouse_on "3 Shell" rclick; shot showcase_chrome_context_menu_rclick_120x40
  keys Escape
  mouse_on "File" click; shot showcase_chrome_menu_mouse_120x40
  keys Escape

  start showcase 120 40 --page codeeditor
  keys Tab; shot showcase_codeeditor_focused_120x40
  keys i; shot showcase_codeeditor_insert_120x40
  keys End Enter; type_text "let x = orders."; shot showcase_codeeditor_typed_120x40
  keys C-Space; shot showcase_codeeditor_completion_120x40
  keys Escape Escape

  start showcase 120 40 --page datagrid
  keys Tab; shot showcase_datagrid_focused_120x40
  keys Down Right Right; shot showcase_datagrid_moved_120x40
  keys Enter; shot showcase_datagrid_editing_120x40
  keys C-l; type_text "42"; keys Enter; shot showcase_datagrid_pending_120x40
  keys End; shot showcase_datagrid_hscroll_120x40

  start showcase 120 40 --page terminal
  keys Tab; shot showcase_terminal_focused_120x40
  keys PageUp; shot showcase_terminal_scrollback_120x40

  start showcase 120 40 --page panels
  keys Tab Tab; shot showcase_panels_focused_120x40
  start showcase 120 40 --page sidebars
  keys Tab Down; shot showcase_sidebars_focused_120x40
  keys Tab; shot showcase_sidebars_focus2_120x40

  start showcase 120 40 --page overview
  keys Down Down; shot showcase_nav_cursor_moved_120x40
  keys Enter; shot showcase_nav_enter_opens_inputs_120x40
  mouse_on "Tables" move; shot showcase_nav_hovered_120x40
  mouse_on "Tables" click; shot showcase_nav_click_opens_tables_120x40
  keys "]"; shot showcase_nav_next_page_key_120x40

  # too small / minimum
  start showcase 72 20 --page buttons; shot showcase_buttons_default_72x20
  start showcase 60 18 --page buttons; shot showcase_buttons_toosmall_60x18
  resize 100 30; shot showcase_buttons_resized_recovered_100x30
  stop
}

# ---------------------------------------------------------------- tablepro
plan_tablepro() {
  for size in 120x40 80x24 160x50 100x30; do
    start tablepro "${size%x*}" "${size#*x}"; shot "tablepro_connections_default_${size}"
  done
  start tablepro 120 40
  keys Down; shot tablepro_connections_selected2_120x40
  for _ in $(seq 7); do tmux send-keys -t $S Down; sleep 0.05; done; sleep 0.2; step "keys(Down x7)"
  shot tablepro_connections_production_120x40
  keys Tab; shot tablepro_connections_focus_form_120x40
  keys Tab Tab; shot tablepro_connections_focus3_120x40
  mouse_on "Analytics" move; shot tablepro_connections_hover_120x40
  mouse_on "Analytics" click; shot tablepro_connections_analytics_120x40
  mouse_on "▎Connect" click 2; shot tablepro_connections_connecting_120x40
  wait_for 2.5; shot tablepro_connections_error_120x40
  keys "?"; shot tablepro_connections_help_120x40
  keys Escape
  start tablepro 120 40
  keys Down Down Down Down Down Down Down Down Enter; shot tablepro_connections_connecting_production_120x40
  wait_for 2.5; shot tablepro_workbench_after_connect_120x40

  for size in 120x40 80x24 160x50 100x30; do
    start tablepro "${size%x*}" "${size#*x}" --connect Production; wait_for 0.5
    shot "tablepro_workbench_default_${size}"
  done

  start tablepro 120 40 --connect Production; wait_for 0.5
  shot tablepro_explorer_focused_120x40
  mouse_on "customers" move; shot tablepro_explorer_hovered_120x40
  keys Down Down; shot tablepro_explorer_moved_120x40
  keys Tab; shot tablepro_editor_focused_120x40
  keys i; shot tablepro_editor_insert_120x40
  type_text "SELECT * FROM ord"; shot tablepro_editor_completion_auto_120x40
  keys Enter; type_text " WHERE st"; shot tablepro_editor_completion_column_120x40
  keys Tab; type_text " = 'pending' ORDER BY created_at DESC LIMIT 25"; shot tablepro_editor_editing_120x40
  keys Escape; shot tablepro_editor_nav_120x40
  keys C-r; shot tablepro_editor_running_120x40
  wait_for 1.5; shot tablepro_results_grid_120x40
  keys Tab; shot tablepro_results_tabs_focused_120x40
  keys Tab; shot tablepro_results_grid_focused_120x40
  keys Down Down Right Right; shot tablepro_results_grid_moved_120x40
  keys End; shot tablepro_results_grid_hscroll_120x40
  keys "?"; shot tablepro_workbench_help_120x40
  keys Escape

  start tablepro 120 40 --connect Production; wait_for 0.5
  keys Tab i; type_text "SELECT * FROM orders WHERE "; keys C-Space; shot tablepro_editor_completion_ctrlspace_120x40
  keys Escape; keys C-l; type_text "SELECT nope FROM orders"; keys Escape C-r; wait_for 1.5; shot tablepro_editor_error_120x40
  keys C-l; type_text "SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10"; keys Escape M-x; wait_for 1.5; shot tablepro_results_explain_120x40
  keys Tab Tab; shot tablepro_results_explain_tree_focused_120x40
  keys r; shot tablepro_results_explain_raw_120x40
  keys C-l; type_text "SELECT * FROM events"; keys Escape C-r; shot tablepro_editor_running_events_120x40
  keys Escape; shot tablepro_editor_cancelled_120x40

  start tablepro 120 40 --connect Production; wait_for 0.5
  keys Down Down Down Down Down Enter; wait_for 0.5; shot tablepro_grid_table_orders_120x40
  keys Down Right Right; shot tablepro_grid_moved_120x40
  mouse_on "order_number" move; shot tablepro_grid_header_hover_120x40
  mouse_on "order_number" click; shot tablepro_grid_header_sorted_120x40
  keys Home; keys Right Right Right Right Right Right; shot tablepro_grid_currency_cell_120x40
  keys Enter; shot tablepro_grid_cell_editing_120x40
  keys C-l; type_text "EUR"; shot tablepro_grid_cell_editing_typed_120x40
  keys Enter; shot tablepro_grid_pending_bar_120x40
  keys p; shot tablepro_grid_pending_preview_120x40
  keys Escape; keys C-s; shot tablepro_grid_save_dialog_120x40
  keys Enter; type_text "orders"; keys Enter; shot tablepro_grid_save_dialog_token_120x40
  keys Right Enter; wait_for 1.5; shot tablepro_grid_saved_120x40
  keys Home Right Right Right Right; keys f; shot tablepro_filter_editor_120x40
  keys BTab BTab Enter C-l; type_text "pending"; shot tablepro_filter_editor_editing_120x40
  keys Enter; shot tablepro_grid_filtered_120x40
  keys C-d; shot tablepro_grid_structure_120x40
  mouse_on "Data" click; shot tablepro_grid_modetab_click_120x40
  keys End; shot tablepro_grid_hscroll_end_120x40

  start tablepro 120 40 --connect Production; wait_for 0.5
  keys C-t; shot tablepro_tabs_new_query_120x40
  keys C-t C-t; shot tablepro_tabs_three_queries_120x40
  keys C-g; shot tablepro_tabs_list_picker_120x40
  keys Down; shot tablepro_tabs_list_picker_moved_120x40
  keys Escape; keys C-o; shot tablepro_open_quickly_120x40
  type_text "cust"; shot tablepro_open_quickly_filtered_120x40
  keys Enter; shot tablepro_open_quickly_opened_120x40
  keys C-y; shot tablepro_history_120x40
  keys /; type_text "payments"; shot tablepro_history_search_editing_120x40
  keys Enter Down; shot tablepro_history_filtered_120x40
  keys Enter; shot tablepro_history_reopened_120x40
  keys C-l; shot tablepro_safemode_picker_120x40
  keys Down; shot tablepro_safemode_picker_moved_120x40
  keys Enter; shot tablepro_safemode_full_120x40
  keys z; shot tablepro_explorer_hidden_z_120x40
  keys Escape
  mouse_on "Query 1" click; shot tablepro_tabs_click_query1_120x40
  mouse_on "customers" move; shot tablepro_tabs_hover_120x40
  mouse_on "+" click; shot tablepro_tabs_plus_click_120x40
  keys F10; shot tablepro_f10_noop_120x40
  keys Escape

  start tablepro 120 40 --connect Production; wait_for 0.5
  keys Tab i; type_text "DELETE FROM orders"; keys Escape C-r; shot tablepro_safety_dialog_delete_120x40
  keys Enter; type_text "wrong"; keys Enter; shot tablepro_safety_dialog_wrong_token_120x40
  keys Escape; shot tablepro_safety_dialog_cancelled_120x40
  keys C-r Enter; type_text "orders"; keys Enter; shot tablepro_safety_dialog_token_ok_120x40
  keys Escape
  keys i C-l; type_text "DROP TABLE orders"; keys Escape C-r; shot tablepro_safety_dialog_drop_120x40
  keys Escape
  keys i C-l; type_text "UPDATE orders SET status = 'paid' WHERE id = 'x'"; keys Escape C-r; shot tablepro_safety_dialog_update_120x40
  keys Enter; type_text "orders"; keys Enter Right Enter; wait_for 1.5; shot tablepro_safety_executed_120x40

  # narrow drawer mode
  start tablepro 80 24 --connect Production; wait_for 0.5
  shot tablepro_drawer_open_80x24
  keys Tab; shot tablepro_drawer_closed_editor_80x24
  keys i; type_text "SELECT * FROM orders LIMIT 5"; keys Escape C-r; wait_for 1.5; shot tablepro_results_grid_80x24
  keys 0; shot tablepro_drawer_reopened_80x24
  keys Down Down Down Down Down Enter; wait_for 0.5; shot tablepro_grid_table_orders_80x24
  keys C-o; shot tablepro_open_quickly_80x24
  keys Escape Escape; keys C-g; shot tablepro_tabs_list_picker_80x24
  keys Escape; keys C-y; shot tablepro_history_80x24
  keys C-t i; type_text "DELETE FROM orders"; keys Escape C-r; shot tablepro_safety_dialog_delete_80x24
  keys Escape; keys C-l; shot tablepro_safemode_picker_80x24
  keys Escape; keys "?"; shot tablepro_workbench_help_80x24
  keys Escape

  start tablepro 160 50 --connect Production; wait_for 0.5
  keys Down Down Down Down Down Enter; wait_for 0.5; shot tablepro_grid_table_orders_160x50
  keys C-t i; type_text "SELECT * FROM orders WHERE status = 'pending' LIMIT 25"; keys Escape C-r; wait_for 1.5; shot tablepro_results_grid_160x50
  keys C-g; shot tablepro_tabs_list_picker_160x50
  keys Escape C-o; shot tablepro_open_quickly_160x50
  keys Escape Escape; keys C-t i; type_text "DELETE FROM orders"; keys Escape C-r; shot tablepro_safety_dialog_delete_160x50
  keys Escape; keys C-y; shot tablepro_history_160x50

  start tablepro 72 20 --connect Production; wait_for 0.5; shot tablepro_workbench_default_72x20
  start tablepro 60 18; shot tablepro_connections_toosmall_60x18
  start tablepro 60 18 --connect Production; wait_for 0.5; shot tablepro_workbench_toosmall_60x18
  resize 100 30; shot tablepro_workbench_resized_recovered_100x30
  stop
}

# ---------------------------------------------------------------- jackin
J="--motion paused"
plan_jackin() {
  # scenarios, paused, stable frames
  start jackin-preview 120 40 --scenario first-use $J --frame 0;   shot jackin_intro_frame0_120x40
  start jackin-preview 120 40 --scenario first-use $J --frame 45;  shot jackin_intro_phrase_frame45_120x40
  start jackin-preview 120 40 --scenario first-use $J --frame 282; shot jackin_intro_warp_frame282_120x40
  start jackin-preview 120 40 --scenario first-use $J --frame 45; keys Enter Enter; shot jackin_manager_empty_firstuse_120x40
  keys Home Enter; shot jackin_manager_launch_picker_noaccounts_120x40
  keys Escape; keys End Enter; shot jackin_prelude_firstuse_120x40
  keys Escape
  start jackin-preview 120 40 --scenario returning $J --frame 0;   shot jackin_manager_default_120x40
  start jackin-preview 120 40 --scenario accounts-mixed $J --frame 0; shot jackin_accounts_default_120x40
  start jackin-preview 120 40 --scenario launch-running $J --frame 0;   shot jackin_cockpit_frame0_120x40
  start jackin-preview 120 40 --scenario launch-running $J --frame 60;  shot jackin_cockpit_frame60_120x40
  start jackin-preview 120 40 --scenario launch-running $J --frame 150; shot jackin_cockpit_frame150_120x40
  keys b; shot jackin_cockpit_build_log_120x40
  keys Escape
  start jackin-preview 120 40 --scenario launch-running $J --frame 300; shot jackin_cockpit_frame300_120x40
  start jackin-preview 120 40 --scenario launch-failure $J --frame 0;   shot jackin_launchfailure_frame0_120x40
  start jackin-preview 120 40 --scenario launch-failure $J --frame 200; shot jackin_launchfailure_frame200_120x40
  start jackin-preview 120 40 --scenario launch-failure $J --frame 600; shot jackin_launchfailure_frame600_120x40
  keys Escape; shot jackin_launchfailure_back_to_manager_120x40
  start jackin-preview 120 40 --scenario capsule-multi $J --frame 0; shot jackin_capsule_default_120x40
  start jackin-preview 120 40 --scenario outro-last $J --frame 0;   shot jackin_outrolast_capsule_frame0_120x40
  start jackin-preview 120 40 --scenario outro-last $J --frame 50;  shot jackin_outro_frame50_120x40
  start jackin-preview 120 40 --scenario outro-last $J --frame 88;  shot jackin_outro_frame88_120x40
  start jackin-preview 120 40 --scenario outro-last $J --frame 105; shot jackin_outro_frame105_120x40
  start jackin-preview 120 40 --scenario outro-last $J --frame 140; shot jackin_outro_frame140_120x40
  start jackin-preview 120 40 --scenario hard-cases $J --frame 0;   shot jackin_hardcases_manager_120x40
  keys Down Down; shot jackin_hardcases_manager_selected_120x40
  keys Enter; shot jackin_hardcases_launch_picker_120x40
  keys Escape; keys c; shot jackin_hardcases_accounts_120x40
  keys Escape; keys s 5; shot jackin_hardcases_settings_trust_120x40
  keys Escape; keys Down e 4 Enter; shot jackin_hardcases_editor_env_roles_120x40
  keys End Enter; shot jackin_hardcases_editor_role_override_picker_120x40
  keys Escape Escape; keys 3 Enter End; shot jackin_hardcases_editor_roles_load_more_120x40

  # host manager interactions
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys Down; shot jackin_manager_selected_120x40
  keys Right; shot jackin_manager_expanded_120x40
  keys Down Tab; shot jackin_manager_detail_focused_120x40
  keys Escape; mouse_on "infra-control-plane" move; shot jackin_manager_hovered_120x40
  mouse_on "infra-control-plane" click; shot jackin_manager_clicked_120x40
  keys Home Down Enter; shot jackin_manager_launch_picker_120x40
  keys Down; shot jackin_manager_launch_picker_moved_120x40
  keys Escape; keys "?"; shot jackin_manager_help_120x40
  keys Escape; keys F10; shot jackin_manager_menu_file_120x40
  keys Right; shot jackin_manager_menu_go_120x40
  keys Right; shot jackin_manager_menu_help_120x40
  keys Right; shot jackin_manager_menu_lockup_120x40
  keys Down Down; shot jackin_manager_menu_item_moved_120x40
  keys Escape; mouse_on "Go" click; shot jackin_manager_menu_go_mouse_120x40
  keys Escape; keys End Enter; shot jackin_prelude_step1_120x40
  keys BSpace; shot jackin_prelude_browser_up_120x40
  keys Down Down Space; shot jackin_prelude_step2_120x40
  keys Enter; shot jackin_prelude_step4_120x40
  keys Enter; shot jackin_prelude_step5_120x40
  keys Enter; shot jackin_editor_new_workspace_120x40
  keys C-s; shot jackin_editor_create_dialog_120x40
  keys Escape

  # editor tabs (digit keys switch tabs only while the tab strip owns focus)
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys Down e; shot jackin_editor_general_120x40
  keys 2; shot jackin_editor_mounts_120x40
  keys Enter; shot jackin_editor_mounts_focused_120x40
  keys r; shot jackin_editor_mounts_readonly_toggled_120x40
  keys i; shot jackin_editor_mounts_isolation_120x40
  keys Escape; keys 3; shot jackin_editor_roles_120x40
  keys Enter; shot jackin_editor_roles_focused_120x40
  keys Escape Escape; shot jackin_editor_leave_dialog_120x40
  keys Escape; keys C-s; shot jackin_editor_save_dialog_120x40
  keys Right; shot jackin_editor_save_dialog_focus2_120x40
  keys Escape; keys F10; shot jackin_editor_menu_file_120x40
  keys Escape
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys Down e Tab; shot jackin_editor_general_focused_120x40
  keys Enter; shot jackin_editor_general_editing_120x40
  keys End; type_text "-2"; shot jackin_editor_general_typed_120x40
  keys Escape
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys Down e 4; shot jackin_editor_env_120x40
  keys Enter; shot jackin_editor_env_focused_120x40
  keys m; shot jackin_editor_env_shown_120x40
  keys m a; shot jackin_editor_env_add_form_120x40
  keys Enter; type_text "NEW_SECRET"; keys Tab Tab Enter; type_text "sk-live-abcdefghijklmnop1234"; shot jackin_editor_env_add_masked_typing_120x40
  keys Tab; shot jackin_editor_env_add_masked_120x40
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys Down e 5; shot jackin_editor_accounts_120x40
  keys Enter; shot jackin_editor_accounts_focused_120x40
  keys Space; shot jackin_editor_accounts_toggled_120x40
  keys Down Down Down p; shot jackin_editor_accounts_prefer_toggled_120x40

  # settings
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys s; shot jackin_settings_general_120x40
  keys 2; shot jackin_settings_tab2_120x40
  keys 3; shot jackin_settings_tab3_120x40
  keys 4; shot jackin_settings_tab4_120x40
  keys 5; shot jackin_settings_trust_120x40
  keys Enter; shot jackin_settings_trust_focused_120x40
  keys Space; shot jackin_settings_trust_toggled_120x40
  keys C-s; shot jackin_settings_save_dialog_120x40
  keys Escape; keys F10; shot jackin_settings_menu_file_120x40
  keys Escape; keys "?"; shot jackin_settings_help_120x40
  keys Escape

  # accounts & usage
  start jackin-preview 120 40 --scenario accounts-mixed $J --frame 0
  keys Down; shot jackin_accounts_selected_120x40
  keys Down Down; shot jackin_accounts_selected3_120x40
  keys Tab; shot jackin_accounts_detail_focus_120x40
  keys x; shot jackin_accounts_remove_dialog_120x40
  keys Escape; keys a; shot jackin_accounts_new_form_120x40
  keys Enter; type_text "Spare"; shot jackin_accounts_form_editing_120x40
  keys Tab Tab Tab; shot jackin_accounts_form_source_120x40
  keys Down Down; shot jackin_accounts_form_apikey_120x40
  keys Tab Enter; type_text "sk-ant-valid-abcdef1234"; shot jackin_accounts_form_key_masked_typing_120x40
  keys Tab; shot jackin_accounts_form_key_masked_120x40
  keys Escape; keys a Enter; type_text "Team"; keys Tab Tab Tab Tab Enter; shot jackin_accounts_op_picker_120x40
  keys Escape Escape; keys "?"; shot jackin_accounts_help_120x40
  keys Escape; keys r; shot jackin_accounts_refreshing_120x40
  keys v; shot jackin_accounts_key_v_120x40
  keys Escape; keys F10; shot jackin_accounts_menu_file_120x40
  keys Escape
  start jackin-preview 120 40 --scenario returning $J --frame 0
  keys u; shot jackin_usage_overview_120x40
  keys Down; shot jackin_usage_limits_120x40
  keys Down; shot jackin_usage_row3_120x40
  keys "?"; shot jackin_usage_help_120x40
  keys Escape; keys m; shot jackin_usage_to_accounts_120x40

  # capsule (F10 opens File; Right walks Edit, View, Session, Help, then the lockup menu)
  start jackin-preview 120 40 --scenario capsule-multi $J --frame 0
  keys F10; shot jackin_capsule_menu_file_120x40
  keys Right; shot jackin_capsule_menu_edit_120x40
  keys Right; shot jackin_capsule_menu_view_120x40
  keys Right; shot jackin_capsule_menu_session_120x40
  keys Right; shot jackin_capsule_menu_help_120x40
  keys Right; shot jackin_capsule_menu_wrap_file_120x40
  keys Escape; mouse_on "jackin❯" click; shot jackin_capsule_menu_lockup_120x40
  keys Escape; mouse_on "View" click; shot jackin_capsule_menu_view_mouse_120x40
  mouse_on "Usage" click; shot jackin_capsule_usage_dialog_120x40
  keys Escape; mouse_on "Shell" rclick; shot jackin_capsule_tabmenu_rclick_120x40
  keys Enter; shot jackin_capsule_rename_dialog_120x40
  keys Enter; type_text "ops"; shot jackin_capsule_rename_editing_120x40
  keys Escape Escape; keys C-b; shot jackin_capsule_prefix_hint_120x40
  keys m; shot jackin_capsule_tabmenu_keyboard_120x40
  keys End; shot jackin_capsule_tabmenu_last_120x40
  keys Enter; shot jackin_capsule_close_tab_dialog_120x40
  keys Escape; keys C-b c; shot jackin_capsule_newtab_picker_120x40
  keys Enter; shot jackin_capsule_newtab_account_picker_120x40
  keys Escape Escape; keys C-b u; shot jackin_capsule_usage_overlay_120x40
  keys Escape; keys 'C-\'; shot jackin_capsule_ctrl_backslash_120x40
  keys F10 Right Right Right Right Down Enter; shot jackin_capsule_palette_120x40
  type_text "split"; shot jackin_capsule_palette_filtered_120x40
  keys Escape Escape; keys C-b z; shot jackin_capsule_zoom_120x40
  keys C-b z; keys C-b %; shot jackin_capsule_split_picker_120x40
  keys Down Down Down Down Enter; shot jackin_capsule_split_shell_120x40
  keys C-q; shot jackin_capsule_quit_dialog_120x40
  keys Down Down; shot jackin_capsule_quit_dialog_moved_120x40
  keys Escape; type_text "hello"; shot jackin_capsule_typed_120x40
  keys PageUp; shot jackin_capsule_scrollback_120x40
  keys End; keys F10 Right Right End Enter; shot jackin_capsule_inspect_changes_120x40
  keys Enter; shot jackin_capsule_inspect_diff_120x40
  keys m; shot jackin_capsule_inspect_compact_120x40
  keys Escape Escape; keys C-b d; shot jackin_capsule_detached_manager_120x40
  keys Enter; shot jackin_capsule_reconnected_120x40
  start jackin-preview 120 40 --scenario capsule-multi $J --frame 0
  mouse_on "Shell" move; shot jackin_capsule_tab_hovered_120x40
  mouse_on "Shell" click; shot jackin_capsule_tab_clicked_120x40
  mouse_on "File" move; shot jackin_capsule_menubar_hovered_120x40
  mouse_on "Session" click; shot jackin_capsule_menu_session_mouse_120x40
  keys Down Down; shot jackin_capsule_menu_session_moved_120x40
  keys Escape; keys C-b i; shot jackin_capsule_ctrl_b_i_120x40
  mouse_on "View" click; mouse_on "Container info" click; shot jackin_capsule_container_info_120x40
  keys Escape

  # responsive
  for size in 80x24 100x30 160x50 72x20 60x18; do
    start jackin-preview "${size%x*}" "${size#*x}" --scenario returning $J --frame 0; shot "jackin_manager_default_${size}"
    start jackin-preview "${size%x*}" "${size#*x}" --scenario capsule-multi $J --frame 0; shot "jackin_capsule_default_${size}"
  done
  for size in 80x24 160x50; do
    start jackin-preview "${size%x*}" "${size#*x}" --scenario capsule-multi $J --frame 0
    keys F10; shot "jackin_capsule_menu_file_${size}"
    keys Escape; mouse_on "Shell" rclick; shot "jackin_capsule_tabmenu_rclick_${size}"
    keys Escape; keys C-q; shot "jackin_capsule_quit_dialog_${size}"
    keys Escape
    start jackin-preview "${size%x*}" "${size#*x}" --scenario accounts-mixed $J --frame 0; shot "jackin_accounts_default_${size}"
    keys a; shot "jackin_accounts_new_form_${size}"
    start jackin-preview "${size%x*}" "${size#*x}" --scenario returning $J --frame 0
    keys s; shot "jackin_settings_general_${size}"
    keys Escape; keys u; shot "jackin_usage_overview_${size}"
    keys Escape; keys Down e; shot "jackin_editor_general_${size}"
    keys Escape; keys F10; shot "jackin_manager_menu_file_${size}"
    keys Escape
    start jackin-preview "${size%x*}" "${size#*x}" --scenario launch-running $J --frame 150; shot "jackin_cockpit_frame150_${size}"
    start jackin-preview "${size%x*}" "${size#*x}" --scenario first-use $J --frame 282; shot "jackin_intro_warp_frame282_${size}"
  done
  start jackin-preview 60 18 --scenario returning $J --frame 0; shot jackin_manager_toosmall_60x18
  resize 80 24; shot jackin_manager_resized_recovered_80x24
  start jackin-preview 60 18 --scenario capsule-multi $J --frame 0; shot jackin_capsule_toosmall_60x18
  resize 100 30; shot jackin_capsule_resized_recovered_100x30
  stop
}

manifest() {
  local md="$OUT/MANIFEST.md"
  {
    echo "# Before-refactor visual evidence (REFACTORING_GOAL.md §6)"
    echo
    echo "Generated by \`tools/baseline_capture.sh all\` on $(date -u +%Y-%m-%dT%H:%MZ) at commit $(git rev-parse --short HEAD)."
    echo "Each capture is \`<name>.{ansi,txt,cursor,html,png}\`; \`.ansi\` is the tmux pane with SGR"
    echo "attributes, \`.txt\` plain cells, \`.cursor\` = \`x y visible-flag\`, \`.html\`/\`.png\` renderings."
    echo "Terminal: tmux 3.7 pane, TERM=xterm-256color, COLORTERM=truecolor, NO_COLOR unset."
    echo "Mouse coordinates are 1-based SGR cells; \`(on \"text\")\` records the on-screen anchor used."
    echo "Steps run in order after the app started with the listed command; \`wait\` is wall-clock."
    echo
    if [ -f "$OUT/NOTES.md" ]; then cat "$OUT/NOTES.md"; echo; fi
    echo "## Captures ($(wc -l < "$TSV" | tr -d ' '))"
    echo
    echo "| Capture | Size | Command | Steps | stderr |"
    echo "|---|---|---|---|---|"
    awk -F'\t' '{ gsub(/\|/, "\\|", $4); printf "| `%s` | %s | `%s` | %s | %s |\n", $1, $2, $3, $4, $5 }' "$TSV"
    echo
    echo "## stderr"
    echo
    local any=0
    for f in "$OUT"/stderr/*.log; do
      [ -s "$f" ] || continue
      any=1
      echo "### $(basename "$f")"; echo; echo '```'; cat "$f"; echo '```'; echo
    done
    [ $any = 1 ] || echo "Every session's stderr log was empty (no panics, no warnings)."
  } > "$md"
  echo "$md"
}

# `BASELINE_LIB=1 source tools/baseline_capture.sh` exposes the primitives without dispatching.
if [ "${BASELINE_LIB:-}" = 1 ]; then return 0; fi
cmd=${1:-}; shift || true
case "$cmd" in
  start) start "$@" ;;
  keys) keys "$@" ;;
  type) type_text "$1" ;;
  mouse) mouse "$@" ;;
  find) find_text "$1" ;;
  shot) shot "$1" ;;
  wait) wait_for "$1" ;;
  resize) resize "$@" ;;
  stop) stop ;;
  showcase) plan_showcase ;;
  tablepro) plan_tablepro ;;
  jackin) plan_jackin ;;
  manifest) manifest ;;
  all) : > "$TSV"; rm -f "$OUT"/stderr/*.log; plan_showcase; plan_tablepro; plan_jackin; manifest ;;
  *) echo "unknown: $cmd" >&2; exit 1 ;;
esac
