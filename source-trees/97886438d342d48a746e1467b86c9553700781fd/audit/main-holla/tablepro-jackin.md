# TablePro / Jackin audit

Pinned comparison: MAIN_BASE c12cad8728755cd2d03eefdd8e02891143fca86d; HOLLA_REFERENCE 794b095c196562d38f1b6f7ce379c128af2a023d. Read-only source inspection; no binary execution or parity claim. Source paths below relative to respective worktrees. Original worktree was not modified.

## Source-proven blockers

| ID | Evidence | Required fix / proof |
|---|---|---|
| J-CLI-1 | main apps/jackin-preview/src/main.rs starts Scenario::Returning; reference src/bin/jackin_preview/main.rs starts FirstUse | Restore FirstUse; process startup tests absent option and explicit returning. |
| J-CLI-2 | main var_os(...).is_some(); reference checks nonempty and != 0 | Absent/empty/0 => full, 1 => reduced; explicit full/reduced/paused always wins. Test actual argv+environment parser and binaries. |
| J-CLI-3 | main accepts only long options and ignores unknown/invalid/missing values; no help arm | Preserve -s/-m/-f/-c/-h, valid help stdout exit 0, malformed recognized values stderr exit 2 before terminal setup. Reference unknown options ignored; retain unless narrow reviewed exception. |
| T-CLI-1 | main apps/tablepro/src/app.rs:2442 parse_args returns InvalidInput for help and lacks -c; reference prints help and exits 0 | Return structured help/error outcomes; binary help 0/error 2 before raw mode. Preserve --connect case-insensitive lookup and color detection. |
| J-GEO-1 | main app.rs:6336–6470 draws actual shell/controls then historical_capsule/editor/manager only for selected 120x40 fixture states | One authoritative responsive layout for public controls and app custom rendering. Never retain conditional historical overpainting. Pointer tests on reference-derived coordinates, all rows and menus, resizing into/out of 120x40. |
| T-GEO-1 | main app.rs has paint_legacy_tree_gutters, paint_legacy_filter, paint_action_button and direct connection property drawing | Inspect each against registered controls and state styling; replace duplicated generic interaction/chrome with public parts/renderers. Presence is evidence lead, not proof all helpers incorrect. |

## Routes and workflows

Jackin scenarios: first-use, returning, accounts-mixed, launch-running, launch-failure, capsule-multi, outro-last, hard-cases. Motion full/reduced/paused; --frame explicit fixture tick. Main additive --theme junie/paper and ansi256/ansi16 aliases must survive while restoring reference CLI.

Jackin current routes: Intro, Manager, Prelude, Editor, Accounts, Usage, Settings, Launch, Cockpit, Handoff, Capsule, Outro. Screen adapters: manager, prelude, editor, accounts, usage, settings, cockpit, capsule, inspect, file_browser, op_flow. Reference config/modals responsibilities need explicit mapping into current app/editor/file_browser/op_flow and generic overlays, rather than treating file deletion as completed migration.

TablePro current routes: Connections, Workbench. Connection create/edit/duplicate/test/connect/failure; explorer/object navigation; table/query/history document tabs; data/structure modes; split editor/results; completion; sort/filter; grid edit validation with NULL/default semantics; pending update/insert/delete changes, undo/revert and SQL preview; safe mode and typed acknowledgement. Source fixtures must be supplemented by real-input reachability for each.

TablePro named surfaces: Connections, ConnectionsFailed, WorkbenchDefault, ExplorerFocused, TableGrid, GridCellEditing, PendingChangeBar, StructureView, QueryEditing, CompletionPopup, ResultsGrid, ErrorResult, ExplainPlan, HistoryTab, QuickSwitcher, TabListPicker, SafeModePicker, FilterEditor, SafetyDialogTypedAck, HelpDialog, MaximisedTab. app.rs:469 set_surface now mutates fixture model/routes, including actual execute_query for results/error/safety and actual open_history/open_table. This is fixture setup, not evidence of user reachability. Render tests must identify renderer output and compare full cells/cursor.

## Safety and API dependencies

App source scan found no std::process::Command, filesystem or network execution calls in these application source trees (PrefixCommand is internal enum, not a process API). This is source evidence only: no dynamic spawn/effect guard executed. Jackin sim modules remain onepassword/provider/changes/world/pty/launch; retain deterministic worlds and prove adversarial journeys cannot spawn or touch external systems.

Main-only secret protections explicitly worth preserving: EnvValue custom Debug redacts Plain/OnePassword/HostEnv and Drop zeroizes Plain through Secret (domain/workspace.rs:609–635); editor transient-value debug/clone redaction tests; sim/onepassword secret_debug_redacts_material; fixture references/debug leakage checks. Do not restore reference editor_env_plain_value_stays_masked_and_can_be_shown behavior blindly: its explicit reveal conflicts with stronger current secret policy and requires narrow security adjudication.

Shared API dependencies: runtime presented geometry and nested-modal routing; consistent row keys across manager/explorer/tab mutation; component content/parts hooks supplying accepted geometry and state styling; monotonic clock/wakeup ownership for intro/outro/launch and simulated PTY; public secret-safe inputs and debug behavior. Current public draw(&self) is useful boundary but not whole proof of effect invariance.

## Original uncommitted salvage

Inspected only apps/tablepro and apps/jackin-preview git diff in original terminal-components-claude. Jackin app diff replaces HIST_* RGB constants with theme semantic palette and surface-derived styles; useful design direction, but still retains disallowed fixed 120x40 historical overpainting and resets modifier masks. TablePro diff replaces some direct fg indexing, hardcoded black gutter with semantic color, and modifier-builder calls; assess independently with state/color tests. Added library-wide too_many_lines allowance justified by historical compatibility renderer is not architectural completion. No patch copied or original file changed.

## Test identity preservation (source discovery, not Cargo execution)

Exact file/test/enumeration inventory is source-inventory.json. Regex discovers ordinary #[test] declarations; generated/parameterized/runtime test identities require Cargo --list and gate-agent reconciliation. Name overlap is not proof preserved assertion semantics. Missing names below require old-body/new-body obligation mapping or restoration.

### tablepro
Reference source identities 33; main 63.
Reference identities missing by name:
- `preview_sql_orders_updates_inserts_deletes`

Preserved names (assertion-body equivalence unverified):
`acceptance_flow_keyboard_only`, `acceptance_flow_mouse`, `cancel_running_query`, `classifies_like_tablepro`, `completion_is_context_aware`, `connections_screen_lists_and_connects_with_keyboard`, `editor_completion_and_execution`, `errors_are_specific`, `every_screen_renders_at_representative_sizes`, `execution_error_marks_editor_and_result`, `explain_builds_tree`, `explain_opens_plan_tree`, `explorer_opens_table_and_grid_navigates`, `failed_connection_shows_error_and_retry`, `history_search_is_multi_term_and`, `history_tab_reopens_query`, `mouse_opens_table_and_switches_tabs`, `narrow_terminals_turn_the_explorer_into_a_drawer`, `parses_select_with_predicates_order_limit`, `pending_edits_preview_and_save`, `quick_switcher_opens_table`, `read_only_connection_refuses_writes`, `runs_filtered_sorted_select`, `safe_mode_picker_changes_level_and_strip`, `safety_gate_intercepts_dangerous_statement_on_production`, `safety_gate_typed_token_executes`, `silent_level_runs_scoped_writes_but_confirms_destructive`, `sort_and_filter_on_table_tab`, `splits_and_finds_statement_at_cursor`, `structure_view_toggle`, `switcher_ranks_tables_first_and_prefix_first`, `tab_strip_overflow_and_tab_list`

### jackin
Reference source identities 63; main 111.
Reference identities missing by name:
- `advanced_tree_drives_the_diff_and_modes_toggle`
- `agent_process_emits_boots_and_replies`
- `compact_opens_a_file_and_returns_to_the_list`
- `editor_env_plain_value_stays_masked_and_can_be_shown`
- `every_scenario_builds`
- `fewer_uncommitted_than_touched_keeps_every_touched_file`
- `intro_timeline_follows_the_original_pacing`
- `masks_private_paths`
- `narrow_terminal_stacks_the_advanced_layout`
- `no_secret_shaped_content`
- `offered_agents_skip_the_unconfigured_and_block_the_unusable`
- `outro_skips_and_captions_like_the_original`
- `precedence_order_and_why`
- `split_close_and_nearest`
- `starfield_is_deterministic_and_restrained`
- `workspace_policy_builds_a_deterministic_effective_set`

Preserved names (assertion-body equivalence unverified):
`accounts_plain_key_is_masked_everywhere_and_remove_asks_first`, `accounts_register_with_a_1password_reference_and_never_render_the_secret`, `axes_are_linked_but_distinct`, `capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line`, `change_count_tracks_fields_and_rows`, `clean_plan_walks_all_eleven_stages_in_order`, `clock_is_pure_over_ticks`, `cockpit_resolves_every_effective_account_for_the_container`, `command_palette_scrolls_with_the_wheel_and_keeps_the_selection`, `complete_jackin_flow_keyboard_first`, `credential_error_holds_until_retry`, `detach_reconnect_and_final_exit_plays_one_outro`, `deterministic_and_realistic`, `durations_use_two_units`, `editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on`, `editor_edits_count_once_preview_then_saves_and_returns`, `empty_construct_plays_once_and_join_skips`, `empty_registry_is_empty_health`, `environments_stay_readable_with_a_hundred_roles`, `exit_token_has_one_consumer_and_fails_closed`, `failure_and_blocked_plans_stop_the_frontier`, `first_use_plays_intro_then_manager_and_no_replay_when_returning`, `foreign_claim_suppresses_duplicate_intro`, `hard_cases_refresh_keeps_last_good_and_help_opens_everywhere`, `hidden_statuses_and_actions`, `hint_bar_stays_on_the_last_row_across_layers`, `inspect_changes_opens_from_the_view_menu_in_both_modes`, `launch_failure_returns_to_the_construct_when_another_instance_runs`, `launch_runs_all_stages_and_hands_off_to_the_capsule`, `manager_launch_picker_hides_agents_without_an_account`, `manager_navigation_expand_and_detail_focus`, `masking_helpers`, `masking_never_reveals_the_value`, `menu_bar_opens_switches_and_runs_an_action`, `missing_entry_time_omits_elapsed`, `names_round_trip`, `one_default_per_provider`, `prelude_creates_a_pending_workspace_and_opens_the_editor`, `prelude_refuses_a_duplicate_name_and_cancels_cleanly`, `quota_status_thresholds`, `reduced_motion_and_paused_frames_are_deterministic`, `resolves_only_inside_the_closure`, `settings_trust_toggle_and_failed_save_keep_edits`, `still_inside_feedback_when_other_instances_remain`, `tab_context_menu_renames_and_closes_by_mouse_and_keyboard`, `too_small_state_and_resize_recover`, `usage_overlay_is_read_only_and_hands_off_to_accounts`

## Acceptance work still required

1. Repair CLI with pure parser tests plus process exits/raw-mode guards.
2. Capture immutable reference surfaces and input journeys; use independent reference geometry.
3. Replace compatibility overpainting only after shared layout/content APIs settle.
4. Map all missing and changed test bodies; Cargo execution must enumerate relocated integration and binary-local tests.
5. Add repeated-draw invariance, monotonic timing, modal leakage, stable identity, secret sentinel and spawn/effect guards.
6. Verify every surface in all required color modes and narrow/default/wide viewports, plus actual process pointer/keyboard/paste/resize journeys.
