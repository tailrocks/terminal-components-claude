#!/usr/bin/env bash
# Flow captures for holla: trust, arguments, alternatives, activities,
# exclusion, failure, disk cleanup gate, remote restart, snapshots.
set -euo pipefail
cd "$(dirname "$0")/.."
export BIN=target/debug/holla
export PY=${PY:-python3}
shot() { tools/capture.sh shot "$1" >/dev/null; echo "$1"; }
start() { ARGS="$1" tools/capture.sh start "$2" "$3"; sleep 1.2; }
keys() { tools/capture.sh keys "$@"; }
stop() { tools/capture.sh stop; }
# trust flow: monorepo child, Run tests needs trust
start "--scenario monorepo-child --motion reduced" 120 40
keys "test"; shot h_flow_child_query
keys Enter; shot h_flow_trust
keys Right Enter; sleep 1.5; shot h_flow_trusted_running
stop
# arguments + alternatives + why + alias
start "--scenario rust-dirty --motion reduced" 120 40
keys M-Enter; shot h_flow_alternatives
keys Escape; keys "clone"; keys Enter; shot h_flow_args
keys Escape Escape; keys "port"; keys Enter; shot h_flow_args_port
keys Enter "5173" Enter Tab Enter; shot h_flow_port_snapshot
keys Escape; keys Escape; keys F1; shot h_flow_help
keys Escape; keys C-g; shot h_flow_activities_empty
stop
# activities multi: tabs, merged logs with chips, picker
start "--scenario activities-multi --motion reduced" 120 40
keys M-4; sleep 0.5; shot h_flow_logs
keys Tab Tab; sleep 0.2; shot h_flow_logs_chips
keys Right Space; shot h_flow_logs_hidden
keys M-1; shot h_flow_frontend
keys M-5; shot h_flow_btm
keys Enter; shot h_flow_btm_attached
keys C-]; keys M-0; keys C-g; shot h_flow_picker
stop
# upgrade plan: exclude, confirm, run to failure
start "--scenario upgrade-plan --motion reduced" 120 40
keys Down Down Down Down Down Space; shot h_flow_upgrade_excluded
keys Space Down Space; shot h_flow_upgrade_cleanup_excluded
keys c; shot h_flow_upgrade_confirm
keys Right Enter; sleep 3; shot h_flow_upgrade_running
sleep 14; shot h_flow_upgrade_failed
keys Down Down Down Down Down; shot h_flow_upgrade_failed_step
stop
# disk cleanup: plan + gate
start "--scenario disk-cleanup --motion paused --frame 80" 120 40
keys c; shot h_flow_cleanup_plan
keys c; shot h_flow_cleanup_gate1
stop
# remote restart two gates + snapshots
start "--scenario remote-host --motion reduced" 120 40
keys "restart payments"; shot h_flow_remote_query
keys Enter; shot h_flow_remote_gate1
keys Right Enter; keys Enter "RESTART PAYMENTS ON prod-eu-1"; shot h_flow_remote_gate2
keys Escape Escape Escape Escape; keys "blocking" Enter; shot h_flow_pg_blocking
keys Escape; keys "resources" Enter; shot h_flow_system
stop
# narrow flows
start "--scenario upgrade-plan --motion reduced" 80 24
shot h_flow_upgrade_80
keys p; shot h_flow_upgrade_80_facts
stop
start "--scenario disk-cleanup --motion paused --frame 80" 80 24
shot h_flow_disk_80
stop
start "--scenario activities-multi --motion reduced" 80 24
keys M-4; shot h_flow_logs_80
stop
start "--scenario monorepo-root --motion full" 100 30
shot h_flow_discovering
keys C-Down; shot h_flow_scope_children
keys C-Up C-Up; shot h_flow_scope_parent
keys C-Up; shot h_flow_scope_system
stop
echo "flows done"
# docker cleanup: two gates, drift revalidation, run to completion (P3)
start "--scenario docker-cleanup --motion full" 120 40
shot h_p3_docker_root
keys Enter; shot h_p3_docker_plan
keys c; shot h_p3_docker_gate1
keys Right Enter; shot h_p3_docker_gate2
keys Enter; keys "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"; shot h_p3_docker_gate2_typed
keys Enter Tab Enter; shot h_p3_docker_drift
# revalidation sent us back to gate 1: continue, retype, execute
keys Tab Enter; keys Enter; keys "I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox"; keys Enter Tab Enter
sleep 2; shot h_p3_docker_running
sleep 14; shot h_p3_docker_done
stop
echo "p3 done"
