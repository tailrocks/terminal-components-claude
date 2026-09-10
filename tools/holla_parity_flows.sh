#!/usr/bin/env bash
# Flow captures for the Holla parity slices (HP01–HP23 except the deferred
# CLI): the states the journeys in src/bin/holla/app_tests_parity.rs drive,
# captured for visual review. Frames land in shots/h_hp*.{txt,html,png} with
# a manifest and a fidelity sidecar each. Drives tools/capture.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
export BIN=target/debug/holla
export PY=${PY:-python3}
shot() { tools/capture.sh shot "$1" >/dev/null; echo "$1"; }
start() { ARGS="$1" tools/capture.sh start "$2" "$3"; sleep 1.2; }
keys() { tools/capture.sh keys "$@"; }
stop() { tools/capture.sh stop; }

# HP01 discovery: slow and failed sources, config warnings, the config page
start "--scenario parity-discovery --motion full" 120 40
shot h_hp01_discovering
sleep 4; shot h_hp01_discovered
keys "Custom action configuration" Enter; shot h_hp01_config_page
stop

# HP02 recents, remembered query, query editing
start "--scenario parity-history --motion reduced" 120 40
shot h_hp02_recent
keys "pull"; shot h_hp02_query
keys C-a; shot h_hp02_query_selected
keys "x" C-z; shot h_hp02_query_undone
stop

# HP03 find files, actions menu
start "--scenario parity-files --motion reduced" 120 40
keys "Find files under home" Enter; keys "readme"; shot h_hp03_find
keys C-u "café"; shot h_hp03_find_unicode
keys C-u "todo" Enter; shot h_hp03_actions
stop

# HP04 browser, previews, hidden, jump picker
start "--scenario parity-browser --motion reduced" 120 40
keys "Browse ~/work/site" Enter; shot h_hp04_browse
keys C-h; shot h_hp04_hidden
keys C-h Down Down Down; shot h_hp04_preview_big
keys Down Down Down Down; shot h_hp04_preview_control
keys g; shot h_hp04_jump
keys "~/nowhere" Enter; shot h_hp04_jump_error
keys C-u "~/work" Enter; shot h_hp04_jumped
stop

# HP05 git current: blocked pull, merge strategy, rejected push
start "--scenario parity-git-current --motion reduced" 120 40
keys "Pull"; shot h_hp05_pull_blocked
keys Escape "Pull with merge" Enter Right Enter; sleep 2; shot h_hp05_merge
keys M-0 C-u "Push" Enter Right Enter; sleep 2; shot h_hp05_push_rejected
stop

# HP06 sibling batch with one failure
start "--scenario parity-git-batch --motion reduced" 120 40
keys "Pull 4 sibling" Enter Right Enter; sleep 3; shot h_hp06_batch
keys C-g; shot h_hp06_batch_picker
stop

# HP07 task adapters: capped scripts, odd names, diagnostics
start "--scenario parity-task-sources --motion reduced" 120 40
keys "yarn"; shot h_hp07_scripts
keys C-u "Taskfile"; shot h_hp07_diagnostic
stop

# HP08 cargo: clean with its size, the dry run
start "--scenario parity-cargo --motion reduced" 120 40
keys "cargo clean"; shot h_hp08_clean
keys Enter; shot h_hp08_clean_confirm
stop

# HP09 docker: stop-all fails at the daemon's stop stage; remove-all gates
start "--scenario parity-docker --motion reduced" 120 40
keys "Stop all containers" Enter Right Enter; sleep 2; shot h_hp09_stop_all_failed
keys M-0 C-u "Stop and remove all" Enter; shot h_hp09_remove_gate1
keys Right Enter; shot h_hp09_remove_gate2
stop

# HP10 brew services: the capped listing and a failing verb
start "--scenario parity-brew-services --motion reduced" 120 40
keys "Homebrew services" Enter; shot h_hp10_services
keys Escape C-u "Stop svc02" Enter Right Enter; sleep 2; shot h_hp10_stop_failed
stop

# HP11 gradle: the recursive cleanup review and its prerequisite
start "--scenario parity-gradle --motion reduced" 120 40
keys "Clean Gradle outputs" Enter Right; shot h_hp11_cleanup
keys d; shot h_hp11_gate1
stop

# HP12 idea: metadata review with the unreadable folder named
start "--scenario parity-idea --motion reduced" 120 40
keys "Clean IntelliJ" Enter Right; shot h_hp12_cleanup
keys d Right Enter; shot h_hp12_gate2
stop

# HP13 upgrade managers: the brew batch and the everything plan
start "--scenario parity-upgrade-managers --motion reduced" 120 40
keys "Upgrade Homebrew packages" Enter Right Enter; sleep 4; shot h_hp13_brew_batch
keys M-0 C-u "Upgrade everything" Enter; shot h_hp13_plan
stop

# HP14 output: the mixed stream and the burst with its retention drop
start "--scenario parity-executor --motion reduced" 120 40
keys "Emit a mixed stream" Enter; sleep 1.5; shot h_hp14_stream
keys M-0 C-u "Emit a burst" Enter; sleep 3; shot h_hp14_burst
keys "/" "line 44"; shot h_hp14_find
stop

# HP15 prompts and cancellation
start "--scenario parity-task-input --motion reduced" 120 40
keys "Deploy the release" Enter Right Enter; sleep 1; shot h_hp15_prompt
keys i; shot h_hp15_input_mode
keys "hunter2" Enter; sleep 1; shot h_hp15_answered
keys "n" Enter; sleep 1; shot h_hp15_cancelled
keys M-0 C-u "Run the stubborn worker" Enter; sleep 0.5; keys s; shot h_hp15_stopping
sleep 1.5; shot h_hp15_killed
stop

# HP17 custom actions: the trust page and the configuration page
start "--scenario parity-custom-actions --motion reduced" 120 40
keys "Deploy preview" Enter; shot h_hp17_trust
keys Right Enter; sleep 1; shot h_hp17_trusted_running
keys M-0 C-u "Custom action configuration" Enter; shot h_hp17_config
stop

# HP18 disk scan: streaming, complete with errors, cancelled partial
start "--scenario parity-disk-scan --motion reduced" 120 40
keys "Analyze disk usage" Enter; sleep 1; shot h_hp18_scanning
sleep 4; shot h_hp18_complete
keys r; sleep 0.5; keys x; shot h_hp18_cancelled
stop

# HP19 tree navigation and top files
start "--scenario parity-disk-navigation --motion reduced" 120 40
keys "Analyze disk usage" Enter; sleep 4; shot h_hp19_tree
keys s; shot h_hp19_apparent
keys s Down Down f; shot h_hp19_unfolded
keys f Space; shot h_hp19_selected
keys t; shot h_hp19_top_files
stop

# HP20 insights
start "--scenario parity-insights --motion reduced" 120 40
keys "Review cleanup candidates" Enter; shot h_hp20_categories
keys Right; shot h_hp20_derived_data
keys End; shot h_hp20_artifacts
stop

# HP21 deletion authorization: the gate and the typed phrase
start "--scenario parity-delete-safety --motion reduced" 120 40
keys "Analyze disk usage" Enter; sleep 3; keys Down Space d; shot h_hp21_gate1
keys Right Enter; shot h_hp21_gate2
keys Enter "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"; shot h_hp21_gate2_typed
keys Enter Tab Enter; shot h_hp21_report
stop

# HP22 results: the history and a report
start "--scenario parity-cleanup-results --motion reduced" 120 40
keys "Show cleanup history" Enter; shot h_hp22_history
stop

# HP23 platforms: macOS Spotlight timeout; Linux without a Trash backend
start "--scenario parity-platforms --motion reduced" 120 40
keys "Top files on this Mac" Enter; shot h_hp23_mac_top_files
stop
start "--scenario parity-platforms-linux --motion reduced" 120 40
keys "Review cleanup candidates" Enter Right Down Space d; shot h_hp23_linux_gate
keys Right Enter; shot h_hp23_linux_gate2
keys Enter "TRASH 1 UNDER /home/alex ON devbox" Enter Tab Enter; shot h_hp23_linux_report
stop
echo "parity flows done"
