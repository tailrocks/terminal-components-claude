# H00 — Existing Holla design and journey coverage

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Preserve and improve the existing Holla concept flows while adding the HP
representations. This task owns design consistency and existing-flow coverage;
it does not add all unimplemented ideas from CONCEPT.md to the backlog.

Inspect current code and [preview inventory](../holla-preview-inventory.md),
[Holla README](../../src/bin/holla/README.md),
[design note](../../holla-project/notes/04-design-note.md),
[CONCEPT.md](../../holla-project/CONCEPT.md) and [DESIGN.md](../../DESIGN.md).
The historical inventory is a starting point, not current completion evidence.

## Acceptance checklist

- [x] Inventory current Holla pages, modals, actions, input owners and scenario routes;
  record missing current-phase behaviors and baseline failures before editing.
- [x] Preserve Here as the decision/navigation surface and activities/plans as
  running work; keep context, host, cwd, provenance, risk and alternatives clear.
- [x] Prove aliases/pins/hide/reset/why, scope navigation and child/parent context
  do not bypass query matching, availability, trust or destructive eligibility.
- [x] Cover existing PostgreSQL, SSH/remote identity, GitHub clone and specialist
  handoff flows with truthful simulated results, failure and cancellation.
  Fix observed gaps in those existing flows; do not add new provider families.
- [x] Cover dependency-aware editable plans, exclusion/undo/retry/skip/stop,
  in-session activity tabs and return-to-Here continuity without losing reports.
- [x] Make keys, pointer targets, paste, drag/wheel and modal ownership agree.
  Holla modal tests must cover before/after first render, hidden background input,
  cancellation, stale opener, queued events and target drift even while F01 is deferred.
- [x] Inspect current Junie geometry/focus, restrained planes, semantic color,
  contextual hints and readable narrow layouts. Preserve drafts/selection on resize.
- [x] Keep all product effects in simulation; assert exact effects and secret
  redaction rather than accepting generic success or label-only fixtures.
- [x] Retain the scenario journeys below plus every new current HP fixture;
  record tests and inspected captures. Run the shared final repository gates.

## Existing scenario coverage

These names come from the current Holla README and match `Scenario::CONCEPT`
(11); the 23 `parity-*` scenarios of `Scenario::PARITY` are evidenced in their
HP task files. `scenario.rs: names_round_trip` and
`fixtures.rs: every_scenario_builds_and_has_sources` cover all 34.

| Scenario | Required journey | Evidence |
| --- | --- | --- |
| first-use | Progressive discovery, suggestions, explore and alternatives | `app_tests.rs: opening_without_typing_suggests_an_action_with_a_reason`, `every_scenario_renders_at_every_size_without_panicking`; `app_tests_flows.rs: esc_ladder_and_quit_rules`, `menu_bar_help_and_about`, `alternatives_pin_alias_hide_and_why`; captures `h_first_use_{80x24,100x30,120x40,160x50,mono}`, `h_flow_help`, `h_flow_alternatives`, `h_flow_discovering` |
| rust-dirty | Git state, review, activity and resulting resource state | `app_tests_flows.rs: finished_runs_change_the_world_they_claimed_to`, `mouse_selects_runs_and_opens_alternatives`; `catalog.rs: rust_dirty_suggests_review_tests_and_pull_with_reasons`; captures `h_rust_dirty_*` |
| monorepo-root | Multi-project plan with correct repository identities | `app_tests_flows.rs: scope_axis_and_tokens_narrow_and_widen`; `fixtures.rs: bulk_git_plans_resolve_primary_branches_and_block_dirty_work`; captures `h_monorepo_root_*`, `h_flow_scope_{children,parent,system}` |
| monorepo-child | Child cwd, parent scope, trust and arguments | `app_tests_flows.rs: nested_child_trusts_then_runs_in_the_child_directory`; `catalog.rs: monorepo_child_ranks_local_first_and_keeps_parent_ecosystem_rows`; captures `h_monorepo_child_*`, `h_flow_child_query`, `h_flow_trust`, `h_flow_trusted_running`, `h_flow_args`, `h_flow_args_port` |
| docker-cleanup | Selection, both gates, drift, execution and outcome | `app_tests_flows.rs: docker_cleanup_takes_two_gates_and_revalidates_before_executing`; `catalog.rs: docker_cleanup_is_first_for_its_query_and_two_gated`; captures `h_docker_cleanup_*`, `h_p3_docker_{root,plan,gate1,gate2,gate2_typed,drift,running,done}` |
| disk-cleanup | Eligibility, exact targets, cleanup gates and honest results | `app_tests_flows.rs: disk_cleanup_selects_by_freshness_and_gates_on_the_path`; captures `h_disk_cleanup_*` (including `h_disk_cleanup_16`), `h_flow_disk_80`, `h_flow_cleanup_gate1`, `h_flow_cleanup_plan` |
| upgrade-plan | Exclusion/dependencies, failure, retry and verification | `app_tests_flows.rs: upgrade_plan_exclusion_recalculates_and_failure_blocks_dependents`; `plan.rs: exclusion_cascades_with_a_reason_and_recomputes`, `failure_blocks_dependents_and_retry_restores_them`, `parallel_eligibility_respects_ancestry_and_locks`; captures `h_upgrade_plan_*` (including `h_upgrade_plan_16`), `h_flow_upgrade_{excluded,cleanup_excluded,confirm,running,failed,failed_step,80,80_facts}` |
| activities-multi | Tab ownership, retained/merged logs, attach/detach and picker | `app_tests_flows.rs: activities_survive_navigation_and_merged_logs_keep_identity`; `domain/activity.rs` executor units; captures `h_activities_multi_*`, `h_flow_logs`, `h_flow_logs_chips`, `h_flow_logs_hidden`, `h_flow_logs_80`, `h_flow_btm`, `h_flow_btm_attached`, `h_flow_picker`, `h_flow_activities_empty` |
| remote-host | Host provenance and host-bound review/cancellation | `app_tests_flows.rs: remote_host_is_unmistakable_and_binds_the_phrase_to_the_host`, `quitting_a_remote_host_with_work_running_names_the_box`; `catalog.rs: remote_host_rows_say_the_host_and_restart_needs_two_gates`; captures `h_remote_host_*`, `h_flow_remote_query`, `h_flow_remote_gate1`, `h_flow_remote_gate2`, `h_flow_pg_blocking`, `h_flow_port_snapshot` |
| launch-failure | Truthful failure, retained output and usable next action | `app_tests_flows.rs: launch_failure_keeps_output_and_offers_follow_ups`; `outcomes.rs: unknown_tools_and_unmodeled_commands_never_succeed`; captures `h_launch_failure_*` |
| hard-cases | Partial/unavailable sources and exceptional resource states | `app_tests_flows.rs: hard_cases_stay_legible_and_report_failed_discovery`, `narrow_root_uses_a_summary_line_and_a_preview_drawer`; captures `h_hard_cases_*`, `h_flow_frontend`, `h_flow_system` |

## Dependencies and evidence

Start inventory before implementation. Finish after the Current tracker slices,
HP23 platform fixtures and F23 Holla proof slices integrate. Use 80×24/120×40
TrueColor/Mono journeys and the additional size/color/input cases required by
changed geometry/state grammar. Capture files alone do not pass review.

## Evidence

**Inventory and baseline:** the historical [preview inventory](../holla-preview-inventory.md) was the starting point; the executable inventory is `app_tests_proofs.rs: inventory` (19 states, focus owners, markers, sizes, palettes). Baseline failures found while implementing and fixed at their owning boundary are recorded in the commit series `c9710d8` to this change (picker eligibility, tree identity, viewport retention and identity, finder ranking and cursor, drawer focus stops, prompt line buffering, fixture seeding, dialog measuring, breadcrumb nesting, cleanup ownership).

**Here as the decision surface, activities and plans as running work:** the concept journeys in `src/bin/holla/app_tests_flows.rs` (17 tests, table above) plus the 23 parity journeys in `app_tests_parity.rs`, the 12 row proofs in `app_tests_rows.rs` and the 9 proofs in `app_tests_proofs.rs`; 140 tests in the holla binary. Aliases, pins, hide and why never bypass matching, availability, trust or eligibility: `alternatives_pin_alias_hide_and_why`, `ranking.rs: aliases_beat_everything_and_carry_a_tag`, `risk_never_demotes_a_strong_match`, `app_tests_parity.rs: hp01_…` (an unavailable tool leads to `gh auth login`, never a clone), `hp09_…` (daemon-down rows are unavailable with the daemon's reason), `hp17_…` (an untrusted action lands on the trust page, an unsaved trust runs nothing).

**Existing families kept truthful:** PostgreSQL (`h_flow_pg_blocking`, `remote_host_…` cancel and terminate effects), SSH and remote identity (`remote_host_is_unmistakable_and_binds_the_phrase_to_the_host`, `quitting_a_remote_host_with_work_running_names_the_box`), GitHub clone (`hp01_…`: login before clone), specialist handoffs (`activities_survive_navigation_and_merged_logs_keep_identity`: btm attach and detach, `h_flow_btm_attached`). No new provider family was added.

**Plans and continuity:** `upgrade_plan_exclusion_recalculates_and_failure_blocks_dependents`, `plan.rs` units, `hp13_…` (the everything plan with lanes), `launch_failure_keeps_output_and_offers_follow_ups`, activity tabs surviving navigation with reports intact (`activities_survive_navigation_and_merged_logs_keep_identity`, `hp22_…` reports and history), the world-owned cleanup job that a quit waits for (`hp22_quit_waits_for_a_running_cleanup_and_its_report_lands_first`).

**Input ownership:** keys, pointer, paste, drag, wheel and modal ownership agree: `app_tests_proofs.rs: batched_and_separated_event_sequences_agree_on_focus_hits_drafts_and_cursor` (page, modal, resize, activation, paste, mouse, batched and separated), `facts_page_takes_an_actual_paste_into_the_editing_field_only` (hidden background input never receives a paste), `mouse_selects_runs_and_opens_alternatives`, `docker_cleanup_takes_two_gates_and_revalidates_before_executing` (stale opener and target drift refused by fingerprint), `enter_with_modifiers_maps_to_exact_picker_and_finder_semantics`, `a_finite_flood_of_ignored_input_never_starves_the_tick_check` (queued events).

**Geometry, colour, hints, narrow layouts, resize:** `inventory_reaches_every_state_with_its_focus_owner_at_every_size_and_palette`, `resize_below_minimum_then_normal_then_wide_then_minimum_keeps_every_state_consistent` (drafts and selection kept), `narrow_root_uses_a_summary_line_and_a_preview_drawer`, `too_small_notice_and_recovery`; captures inspected at 80x24 and mono for the concept and parity scenarios (`h_parity_discovery_80x24`, `h_parity_executor_mono`, `h_disk_cleanup_16`, `h_upgrade_plan_16`) and the 120x40 flow frames named in every HP task.

**Simulation only, exact effects, secrets:** every effect lands through `World::apply_effect` and the outcomes table (`finished_runs_change_the_world_they_claimed_to`, `outcomes.rs` units, `parity.rs: every_parity_world_builds_a_catalogue_with_unique_ids`); an unmodeled command fails visibly (`unknown_tools_and_unmodeled_commands_never_succeed`); secrets are masked and never retained (`hp15_…`: `hunter2` is never echoed, stdin records are empty and secret). No real command, service, scanner, persistence, OS clipboard, opener or destructive filesystem action is connected (`HOLLA_NO_HISTORY`, the virtual filesystem and the in-memory stores are the only sinks).

**Captures:** regenerated on the final binary by `tools/holla_shots.sh` (11 concept scenarios and 23 parity scenarios at 80x24, 100x30, 120x40, 160x50 and mono, plus the 16-colour frames), `tools/holla_flows.sh` (concept flows) and `tools/holla_parity_flows.sh` (69 parity frames); each frame has a manifest (source revision `1aaa9b0` with this change set's dirty tree, binary sha256, arguments, geometry, colour environment, tools, fonts) and a fidelity sidecar; inspected frames carry a `review` verdict in the manifest. The `.txt` captures are authoritative for content.

**Repository gates:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all suites), `cargo doc --no-deps`, `git diff --check` pass at the closing commit (see the commit message for the counts).

**Limits and Later:** Linux behaviour is fixture-modeled and captured, not run on Linux; job-control and PTY proofs ran on macOS only; HP16 CLI parity and every operational clause listed under Later in the HP task files remain open; the design note in `holla-project/notes/04-design-note.md` still describes the model accurately and was not changed.
