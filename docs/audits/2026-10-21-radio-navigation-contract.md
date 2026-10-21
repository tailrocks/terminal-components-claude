# Radio navigation contract — 2026-10-21

Status: implementation in isolated WIP; independent review and integration gates pending.

## Decision

`RadioGroup` and its `Form` field adapters now default keyboard arrow/Home/End navigation to immediate choice, matching the baseline/HIG behavior reported by the pinned TablePro probes. `RadioNavigation::Deferred` remains an explicit per-widget option for consumers that intentionally separate cursor movement from the chosen value. The global `Form` Enter policy is unchanged. TablePro's shared connection-form factory uses `EnterPolicy::Never` so Enter in a radio field does not submit the form; Enter on the Save action and Ctrl+S continue to submit through the configured form action.

Pointer press only moves the radio cursor. It does not choose an option; release/click handling retains the existing activation path. Disabled and read-only handling remains owned by the existing shared control/Form routing. Stable item keys continue to identify values independently of row position.

## Evidence and migration

The change addresses the immutable three-test red packet at `branch-consolidation/tablepro-radio-contract-red-c8f92ea-20261021/review.json` (SHA-256 `8d8f141a760850592a6200b8ab4b86d48f04a014b5951895ced32a7f1579cd7b`). The TablePro UI regression now enters valid Name and Database values through the actual form before testing radio Enter, then checks that the form stays open without appending a connection. It separately verifies Enter on Save and Ctrl+S still invoke the configured save action.

The prior `ClippedRadioConsumer` regression specifically proves deferred cursor/value separation, so it now opts into `RadioNavigation::Deferred` explicitly. The direct radio unit test proving the same contract also opts in. New runtime-driven tests cover immediate navigation and deferred opt-out, pointer press/release/click behavior, and immediate choice through both flat and lane `Form` layouts.

The initial external c8 probe run passed both immediate-choice tests. Its valid-form Enter test failed earlier than the radio-Enter assertion because the current source seeds a new TablePro draft by cloning the Production connection (`acme_devradio_probe`) rather than the probe's required blank draft (`radio_probe`). This is a separate prerequisite tracked by the parent (`f076`); preserve the unchanged probe and rerun it after that correction is integrated. Do not treat this failure as a radio-contract pass.

## Focused checks

- `cargo nextest run -p termrock-controls --lib`: 95 passed.
- `cargo nextest run -p termrock-forms --all-targets`: 67 passed.
- `cargo nextest run -p tablepro-ui --test form_radio`: 11 passed.
- `cargo nextest run -p termrock-conformance --test control_states`: radio cases passed; three unrelated existing failures remain: `w02_button_variants_all_states_and_focus_gutter`, `w08_input_masked_clicks_follow_display`, and `w13_filter_backspace_removes_grapheme` (one additional case skipped by that target).

No baseline snapshots or hashes were changed. Full workspace and paired visual gates remain pending; this bounded change is not evidence of complete application parity.
